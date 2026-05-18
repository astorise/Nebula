#!/usr/bin/env bash
set -euo pipefail

ROOTFS="${1:-rootfs}"
mkdir -p "$ROOTFS/toolchain"
MANIFEST="$ROOTFS/toolchain/manifest.txt"
cat > "$MANIFEST" <<'EOF'
rustc
cargo
cargo-component
wasm32-wasip1
EOF
sha256sum "$MANIFEST" > "$MANIFEST.sha256"
echo "nebula wasm foundry rootfs prepared at $ROOTFS"
