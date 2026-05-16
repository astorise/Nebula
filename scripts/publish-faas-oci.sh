#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAAS_DIR="${ROOT_DIR}/faas"
REGISTRY="${OCI_REGISTRY:-ghcr.io/astorise/nebula}"
TAG="${OCI_TAG:-dev}"

cd "${FAAS_DIR}"

for manifest in ./*/Cargo.toml; do
  crate_dir="$(dirname "${manifest}")"
  crate_name="$(basename "${crate_dir}")"

  if [[ "${crate_name}" == "target" ]]; then
    continue
  fi

  if [[ "${crate_name}" == "nebula-eval-ast" ]]; then
    echo "Skipping ${crate_name}; it is packaged as a microVM rootfs."
    continue
  fi

  echo "Building ${crate_name}"
  cargo component build --release --package "${crate_name}"

  wasm_name="${crate_name//-/_}.wasm"
  component_path="${FAAS_DIR}/target/wasm32-wasip1/release/${wasm_name}"

  if [[ ! -f "${component_path}" ]]; then
    echo "Unable to find component output: ${component_path}" >&2
    exit 1
  fi

  echo "Publishing ${crate_name} to ${REGISTRY}/${crate_name}:${TAG}"
  wkg push "${REGISTRY}/${crate_name}:${TAG}" "${component_path}"
done
