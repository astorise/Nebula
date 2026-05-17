#!/usr/bin/env bash
set -euo pipefail

ROOTFS="${1:-rootfs}"
mkdir -p "$ROOTFS/toolchain"
cat > "$ROOTFS/toolchain/manifest.txt" <<'EOF'
rustc
cargo
cargo-component
wasm32-wasip1
EOF
echo "nebula wasm foundry rootfs prepared at $ROOTFS"
