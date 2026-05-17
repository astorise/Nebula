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
SHA256SUM="$(sha256sum "$MANIFEST" | sed 's/^\\//' | awk '{print $1}')"
printf 'sha256sum %s\n' "$SHA256SUM" >> "$MANIFEST"
echo "nebula wasm foundry rootfs prepared at $ROOTFS"
