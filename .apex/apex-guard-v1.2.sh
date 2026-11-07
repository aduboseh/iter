#!/usr/bin/env bash
# APEX v1.2 - heuristic only. CI must execute trusted base-branch copy.
set -euo pipefail
export LC_ALL=C
err(){ echo "APEX ERROR: $*" >&2; exit 2; }
base="$1"
manifest=".apex/protected-paths.txt"
[[ -n "$base" ]] || err "base ref required"
[[ -n "$APEX_GLOBAL_BYTES" && "$APEX_GLOBAL_BYTES" =~ ^[0-9]+$ ]] || err "APEX_GLOBAL_BYTES integer required, 0 if unknown"
git rev-parse --verify --quiet "$base^{commit}" >/dev/null || err "invalid base ref"
cd "$(git rev-parse --show-toplevel)"
[[ -f "$manifest" && -f AGENTS.md ]] || err "missing required manifest or AGENTS.md"
mb="$(git merge-base "$base" HEAD)" || err "no common ancestor"
fail=0; budget=32768; total="$APEX_GLOBAL_BYTES"
while IFS= read -r -d '' file; do
 case "$file" in AGENTS.md|*/AGENTS.md|AGENTS.override.md|*/AGENTS.override.md)
  [[ -f "$file" ]] || continue
  dir="$(dirname "$file")"; size="$APEX_GLOBAL_BYTES"
  while :; do
   chosen=""
   [[ -f "$dir/AGENTS.md" ]] && chosen="$dir/AGENTS.md"
   [[ -f "$dir/AGENTS.override.md" ]] && chosen="$dir/AGENTS.override.md"
   [[ -z "$chosen" ]] || size=$((size+$(wc -c < "$chosen")))
   [[ "$dir" == "." ]] && break
   dir="$(dirname "$dir")"
  done
  (( size <= total )) || total="$size"
  ;;
 esac
done < <(git ls-files -co --exclude-standard -z)
if (( total>budget )); then echo "FAIL instruction budget"; fail=1; fi
while IFS= read -r -d '' path; do
 protected=0
 case "$path" in AGENTS.md|*/AGENTS.md|AGENTS.override.md|*/AGENTS.override.md|.apex/*|.github/*|CODEOWNERS|*/CODEOWNERS|Cargo.toml|Cargo.lock|Makefile|pyproject.toml) protected=1;; esac
 while IFS= read -r pattern || [[ -n "$pattern" ]]; do
  [[ -z "$pattern" || "$pattern" == \#* ]] && continue
  case "$path" in $pattern) protected=1;; esac
 done < "$manifest"
 if (( protected )); then echo "FAIL protected: $path"; fail=1; fi
done < <(git diff --name-only -z "$mb" HEAD)
while IFS= read -r -d '' path; do
 case "$path" in tests/*|*/tests/*|test_*.py|*_test.rs|*.test.js|*.test.ts) echo "FAIL deleted test: $path"; fail=1;; esac
done < <(git diff --name-only --diff-filter=D -z "$mb" HEAD)
patch="$(git diff -U0 "$mb" HEAD -- '*.rs' '*.py' '*.js' '*.ts' '*.sh' '*.yml' '*.yaml' '*.toml' Makefile)" || err "diff failed"
added="$(printf '%s\n' "$patch" | grep -E '^\+' | grep -vE '^\+\+\+' || true)"
removed="$(printf '%s\n' "$patch" | grep -E '^-' | grep -vE '^---' || true)"
if printf '%s\n' "$added" | grep -E -e '#\[ignore|pytest\.mark\.(skip|xfail)|type: ignore|noqa|eslint-disable|continue-on-error:[[:space:]]*true|^\+[[:space:]]*assert[[:space:]]+True|^\+[[:space:]]*if:[[:space:]]*false|^\+[[:space:]]*true[[:space:]]*$' >/dev/null; then echo "FAIL possible gate weakening"; fail=1; fi
if printf '%s\n' "$removed" | grep -E -e '-D[[:space:]]*warnings|--locked|fail_under|set -e' >/dev/null; then echo "FAIL strictness removed"; fail=1; fi
if (( fail )); then echo "APEX findings: independent review required"; else echo "APEX heuristic clean; NOT approval"; fi
exit "$fail"
