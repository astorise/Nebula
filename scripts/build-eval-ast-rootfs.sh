#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAAS_DIR="${ROOT_DIR}/faas"
OUT_DIR="${ROOT_DIR}/artifacts/microvm/nebula-eval-ast"
ROOTFS="${OUT_DIR}/rootfs.ext4"
STAGING="${OUT_DIR}/rootfs"

mkdir -p "${STAGING}/usr/local/bin" "${STAGING}/run"

cargo build --release --package nebula-eval-ast --target x86_64-unknown-linux-musl --manifest-path "${FAAS_DIR}/Cargo.toml"
cp "${FAAS_DIR}/target/x86_64-unknown-linux-musl/release/nebula-eval-ast" "${STAGING}/usr/local/bin/nebula-eval-ast"

cat > "${STAGING}/init" <<'EOF'
#!/bin/sh
exec /usr/local/bin/nebula-eval-ast
EOF
chmod +x "${STAGING}/init"

rm -f "${ROOTFS}"
dd if=/dev/zero of="${ROOTFS}" bs=1M count="${ROOTFS_SIZE_MB:-64}"
mkfs.ext4 -d "${STAGING}" -F "${ROOTFS}"

echo "${ROOTFS}"
