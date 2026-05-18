#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAAS_DIR="$ROOT_DIR/faas"

mapfile -t rust_tests < <(
  find "$FAAS_DIR" \
    -path "$FAAS_DIR/target" -prune -o \
    \( -path "$FAAS_DIR/nebula-*/src/*.rs" -o \
    -path "$FAAS_DIR/nebula-*/tests/*.rs" -o \
    -path "$FAAS_DIR/adapters/*/src/*.rs" -o \
    -path "$FAAS_DIR/adapters/*/tests/*.rs" \) \
    -type f -print |
    sort |
    xargs grep -EIl '#\[(tokio::test|test|rstest|test_case)\]'
)

missing_annotations=()
for test_file in "${rust_tests[@]}"; do
  if ! awk '
    /#\[(tokio::test|test|rstest|test_case)\]/ {
      getline next_line
      if (next_line !~ /^[[:space:]]*\/\/ spec: [a-z0-9][a-z0-9-]*(#[a-z0-9][a-z0-9-]*)?/) {
        missing = 1
      }
    }
    END { exit missing ? 1 : 0 }
  ' "$test_file"; then
    missing_annotations+=("${test_file#$ROOT_DIR/}")
  fi
done

required_specs=()
while IFS= read -r -d '' spec_file; do
  capability="$(basename "$(dirname "$spec_file")")"
  if [[ -d "$FAAS_DIR/nebula-$capability" ]] || grep -qE '^## Location:.*`?faas/' "$spec_file"; then
    required_specs+=("$capability")
  fi
done < <(find "$ROOT_DIR/openspec/specs" "$ROOT_DIR/openspec/changes" -path '*/specs/*/spec.md' -print0 2>/dev/null)

mapfile -t required_specs < <(printf '%s\n' "${required_specs[@]}" | sort -u)

missing_specs=()
for capability in "${required_specs[@]}"; do
  if ! grep -R --include='*.rs' --exclude-dir=target -qE "// spec: ${capability}(#|$)" "$FAAS_DIR"; then
    missing_specs+=("$capability")
  fi
done

if (( ${#missing_annotations[@]} > 0 )); then
  printf 'Rust test files missing // spec annotations after #[test]:\n' >&2
  printf '  - %s\n' "${missing_annotations[@]}" >&2
fi

if (( ${#missing_specs[@]} > 0 )); then
  printf 'OpenSpec capabilities without Rust test coverage tags:\n' >&2
  printf '  - %s\n' "${missing_specs[@]}" >&2
fi

if (( ${#missing_annotations[@]} > 0 || ${#missing_specs[@]} > 0 )); then
  exit 1
fi

printf 'OpenSpec traceability verified for %s Rust test files and %s FaaS specs.\n' \
  "${#rust_tests[@]}" "${#required_specs[@]}"
