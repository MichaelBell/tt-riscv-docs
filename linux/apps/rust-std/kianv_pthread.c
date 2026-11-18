// SPDX-License-Identifier: MIT
// Minimal pthread layer for Rust std on KianV uLinux (uClibc-ng built with
// BR2_PTHREADS_NONE, no MMU, no TLS). Provides what Rust std links against:
// thread-specific keys, pthread_self/create/join/detach and attr helpers.
// Threads are clone(CLONE_VM|CLONE_THREAD) tasks on malloc'd stacks; the
// current thread is found by its stack pointer. malloc/free/realloc/calloc/
// memalign/posix_memalign are serialized with a futex lock via -Wl,--wrap.
// Limits: errno is shared between threads; uClibc stdio is not thread safe.
#define _GNU_SOURCE
#include <errno.h>
#include <sched.h>
#include <signal.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <linux/futex.h>

#define MAX_KEYS 32
#define MAX_THREADS 8
#define DTOR_ITERATIONS 4

typedef unsigned long pthread_t;
typedef unsigned int pthread_key_t;
typedef struct { size_t stacksize; int detached; int pad[7]; } kv_attr_t;

struct tcb {
    volatile int tid;          // cleared by the kernel on exit (CLONE_CHILD_CLEARTID)
    int used, detached;
    char *stack_lo, *stack_hi; // malloc'd stack; main thread: 0..0
    void *(*fn)(void *);
    void *arg, *ret;
    void *val[MAX_KEYS];
};

static struct tcb threads[MAX_THREADS]; // [0] = main thread
static void (*key_dtor[MAX_KEYS])(void *);
static unsigned char key_used[MAX_KEYS];
static volatile int big_lock; // protects threads[] / keys / malloc

static long sys_futex(volatile int *a, int op, int v) {
    return syscall(SYS_futex, a, op, v, 0, 0, 0);
}
static void lock(volatile int *l) {
    int c = __sync_val_compare_and_swap(l, 0, 1);
    if (c == 0) return;
    do {
        if (c == 2 || __sync_val_compare_and_swap(l, 1, 2) != 0)
            sys_futex(l, FUTEX_WAIT, 2);
    } while ((c = __sync_val_compare_and_swap(l, 0, 2)) != 0);
}
static void unlock(volatile int *l) {
    if (__sync_fetch_and_sub(l, 1) != 1) { *l = 0; sys_futex(l, FUTEX_WAKE, 1); }
}

static struct tcb *self_tcb(void) {
    char *sp = (char *)__builtin_frame_address(0);
    for (int i = 1; i < MAX_THREADS; i++)
        if (threads[i].used && sp >= threads[i].stack_lo && sp < threads[i].stack_hi)
            return &threads[i];
    return &threads[0];
}

pthread_t pthread_self(void) { return (pthread_t)self_tcb(); }
int pthread_equal(pthread_t a, pthread_t b) { return a == b; }

int pthread_key_create(pthread_key_t *k, void (*dtor)(void *)) {
    lock(&big_lock);
    for (unsigned i = 0; i < MAX_KEYS; i++)
        if (!key_used[i]) {
            key_used[i] = 1; key_dtor[i] = dtor;
            for (int t = 0; t < MAX_THREADS; t++) threads[t].val[i] = 0;
            *k = i; unlock(&big_lock); return 0;
        }
    unlock(&big_lock);
    return EAGAIN;
}
int pthread_key_delete(pthread_key_t k) {
    if (k >= MAX_KEYS) return EINVAL;
    key_used[k] = 0; key_dtor[k] = 0; return 0;
}
void *pthread_getspecific(pthread_key_t k) { return k < MAX_KEYS ? self_tcb()->val[k] : 0; }
int pthread_setspecific(pthread_key_t k, const void *v) {
    if (k >= MAX_KEYS) return EINVAL;
    self_tcb()->val[k] = (void *)v; return 0;
}

int pthread_attr_init(void *a) { memset(a, 0, sizeof(kv_attr_t)); return 0; }
int pthread_attr_destroy(void *a) { (void)a; return 0; }
int pthread_attr_setstacksize(void *a, size_t s) {
    if (s < 16384) return EINVAL;
    ((kv_attr_t *)a)->stacksize = s; return 0;
}
int pthread_attr_getstacksize(const void *a, size_t *s) { *s = ((const kv_attr_t *)a)->stacksize; return 0; }
int pthread_attr_setdetachstate(void *a, int d) { ((kv_attr_t *)a)->detached = d; return 0; }
int pthread_getattr_np(pthread_t t, void *a) { (void)t; (void)a; return ENOSYS; }
int pthread_attr_getstack(const void *a, void **addr, size_t *s) { (void)a; *addr = 0; *s = 0; return ENOSYS; }
int pthread_attr_getguardsize(const void *a, size_t *s) { (void)a; *s = 0; return 0; }
int pthread_setname_np(pthread_t t, const char *n) { (void)t; (void)n; return 0; }
int pthread_getname_np(pthread_t t, char *n, size_t l) { (void)t; if (l) n[0] = 0; return 0; }
int pthread_sigmask(int how, const sigset_t *set, sigset_t *old) {
    return sigprocmask(how, set, old) ? errno : 0;
}

static void run_dtors(struct tcb *t) {
    for (int it = 0; it < DTOR_ITERATIONS; it++) {
        int any = 0;
        for (int k = 0; k < MAX_KEYS; k++) {
            void *v = t->val[k];
            if (v && key_used[k] && key_dtor[k]) { t->val[k] = 0; key_dtor[k](v); any = 1; }
        }
        if (!any) break;
    }
}

static void free_tcb(struct tcb *t) {
    lock(&big_lock);
    free(t->stack_lo);
    t->used = 0;
    unlock(&big_lock);
}

static int start(void *p) {
    struct tcb *t = p;
    t->ret = t->fn(t->arg);
    run_dtors(t);
    return 0; // __thread_start issues SYS_exit; kernel clears t->tid and wakes joiners
}

int pthread_create(pthread_t *out, const void *attr, void *(*fn)(void *), void *arg) {
    size_t ss = attr && ((const kv_attr_t *)attr)->stacksize ? ((const kv_attr_t *)attr)->stacksize : 65536;
    int det = attr ? ((const kv_attr_t *)attr)->detached : 0;
    lock(&big_lock);
    struct tcb *t = 0;
    for (int i = 1; i < MAX_THREADS; i++) // reclaim exited detached threads
        if (threads[i].used && threads[i].detached && threads[i].tid == 0) {
            free(threads[i].stack_lo); threads[i].used = 0;
        }
    for (int i = 1; i < MAX_THREADS; i++)
        if (!threads[i].used) { t = &threads[i]; break; }
    if (!t) { unlock(&big_lock); return EAGAIN; }
    memset(t, 0, sizeof *t);
    t->used = 1;
    unlock(&big_lock);
    char *stk = malloc(ss);
    if (!stk) { t->used = 0; return EAGAIN; }
    t->stack_lo = stk; t->stack_hi = stk + ss;
    t->fn = fn; t->arg = arg; t->detached = det;
    t->tid = 1;
    int flags = CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND | CLONE_THREAD |
                CLONE_SYSVSEM | CLONE_CHILD_CLEARTID | CLONE_PARENT_SETTID;
    int r = clone(start, (void *)(((uintptr_t)t->stack_hi) & ~(uintptr_t)15), flags, t,
                  (int *)&t->tid, 0, (int *)&t->tid);
    if (r < 0) { int e = errno; free(stk); t->used = 0; return e; }
    *out = (pthread_t)t;
    return 0;
}

int pthread_join(pthread_t th, void **ret) {
    struct tcb *t = (struct tcb *)th;
    int tid;
    while ((tid = t->tid) != 0) sys_futex(&t->tid, FUTEX_WAIT, tid);
    if (ret) *ret = t->ret;
    free_tcb(t);
    return 0;
}

int pthread_detach(pthread_t th) {
    struct tcb *t = (struct tcb *)th;
    t->detached = 1; // slot and stack are reclaimed by a later pthread_create
    return 0;
}

// Serialize the allocator (uClibc without threads has no malloc lock). The lock
// is recursive: uClibc's realloc/memalign call malloc/free internally, and
// those calls are wrapped too.
void *__real_malloc(size_t); void __real_free(void *); void *__real_realloc(void *, size_t);
void *__real_calloc(size_t, size_t); void *__real_memalign(size_t, size_t);
int __real_posix_memalign(void **, size_t, size_t);
static volatile int heap_lock;
static struct tcb *volatile heap_owner;
static int heap_depth;
static void heap_enter(void) {
    struct tcb *t = self_tcb();
    if (heap_owner == t) { heap_depth++; return; }
    lock(&heap_lock);
    heap_owner = t; heap_depth = 1;
}
static void heap_leave(void) {
    if (--heap_depth == 0) { heap_owner = 0; unlock(&heap_lock); }
}
void *__wrap_malloc(size_t n) { heap_enter(); void *p = __real_malloc(n); heap_leave(); return p; }
void __wrap_free(void *p) { heap_enter(); __real_free(p); heap_leave(); }
void *__wrap_realloc(void *o, size_t n) { heap_enter(); void *p = __real_realloc(o, n); heap_leave(); return p; }
void *__wrap_calloc(size_t a, size_t b) { heap_enter(); void *p = __real_calloc(a, b); heap_leave(); return p; }
void *__wrap_memalign(size_t a, size_t n) { heap_enter(); void *p = __real_memalign(a, n); heap_leave(); return p; }
int __wrap_posix_memalign(void **o, size_t a, size_t n) { heap_enter(); int r = __real_posix_memalign(o, a, n); heap_leave(); return r; }

// No MMU: fork() cannot exist. std::process::Command reports ENOSYS instead of failing to link.
pid_t fork(void) { errno = ENOSYS; return -1; }
