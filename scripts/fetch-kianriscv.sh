#!/usr/bin/env bash
# Clone kianRiscV at the pinned commit and apply patches/kianRiscV.
# Usage: scripts/fetch-kianriscv.sh [dest-dir]   (default: build/kianRiscV)
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
. "$ROOT/pins.env"
DEST=${1:-$ROOT/build/kianRiscV}
# init + fetch rather than clone: DEST may already hold a restored Buildroot cache.
mkdir -p "$DEST"
cd "$DEST"
if [ ! -d .git ]; then
  git init -q
  git remote add origin "$KIANRISCV_REPO"
fi
git fetch -q --filter=blob:none origin "$KIANRISCV_COMMIT"
git -c advice.detachedHead=false checkout -q -f "$KIANRISCV_COMMIT"
git am --abort >/dev/null 2>&1 || true
git -c user.name=tt-riscv -c user.email=tt-riscv@localhost am -q --keep-cr "$ROOT"/patches/kianRiscV/*.patch
echo "kianRiscV at $(git rev-parse --short HEAD) ($(git rev-list --count "$KIANRISCV_COMMIT"..HEAD) patches on $KIANRISCV_COMMIT)"
