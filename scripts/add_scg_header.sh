#!/bin/bash
# SCG Governance: Deterministic | ESV-Compliant | Drift ≤1e-10
# Lineage: 9D7623E581D982D8F9BC816738EF0880E9631E6FD5789C36AF80698DF2BAA527
# Generated under SCG_Governance_v1.0

# Script to add SCG governance headers to new code files
# Usage: ./add_scg_header.sh <file_path>

set -euo pipefail

GOVERNANCE_VERSION="SCG_Governance_v1.0"

# Function to generate file SHA256
get_file_sha() {
    local file="$1"
    if command -v sha256sum &> /dev/null; then
        sha256sum "$file" | cut -d' ' -f1 | tr 'a-f' 'A-F'
    elif command -v shasum &> /dev/null; then
        shasum -a 256 "$file" | cut -d' ' -f1 | tr 'a-f' 'A-F'
    else
        echo "NO_SHA256_TOOL"
    fi
}

# Function to add header to Rust files
add_rust_header() {
    local file="$1"
    local sha="$2"
    local header="// SCG Governance: Deterministic | ESV-Compliant | Drift ≤1e-10
// Lineage: ${sha}
// Generated under ${GOVERNANCE_VERSION}
"
    # Check if header already exists
    if head -1 "$file" | grep -q "SCG Governance"; then
        echo "Header already exists in $file"
        return 0
    fi
    
    # Prepend header
    echo "$header" | cat - "$file" > "$file.tmp" && mv "$file.tmp" "$file"
    echo "Added SCG header to $file"
}

# Function to add header to Python files
add_python_header() {
    local file="$1"
    local sha="$2"
    local header="# SCG Governance: Deterministic | ESV-Compliant | Drift ≤1e-10
# Lineage: ${sha}
# Generated under ${GOVERNANCE_VERSION}
"
    # Check if header already exists
    if head -1 "$file" | grep -q "SCG Governance"; then
        echo "Header already exists in $file"
        return 0
    fi
    
    # Handle shebang if present
    if head -1 "$file" | grep -q "^#!"; then
        local shebang
        shebang=$(head -1 "$file")
        tail -n +2 "$file" > "$file.tmp"
        echo "$shebang" > "$file"
        echo "" >> "$file"
        echo "$header" >> "$file"
        cat "$file.tmp" >> "$file"
        rm "$file.tmp"
    else
        echo "$header" | cat - "$file" > "$file.tmp" && mv "$file.tmp" "$file"
    fi
    echo "Added SCG header to $file"
}

# Main logic
if [ $# -eq 0 ]; then
    echo "Usage: $0 <file_path> [file_path2 ...]"
    echo "       $0 --staged  # Process all staged files"
    exit 1
fi

if [ "$1" == "--staged" ]; then
    # Process staged files (for pre-commit hook use)
    staged_files=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(rs|py)$' || true)
    if [ -z "$staged_files" ]; then
        echo "No staged Rust/Python files to process"
        exit 0
    fi
    for file in $staged_files; do
        "$0" "$file"
    done
    exit 0
fi

for file in "$@"; do
    if [ ! -f "$file" ]; then
        echo "File not found: $file"
        continue
    fi
    
    # Generate placeholder SHA (will be updated on commit)
    sha="PENDING_$(date +%Y%m%d_%H%M%S)"
    
    case "$file" in
        *.rs)
            add_rust_header "$file" "$sha"
            ;;
        *.py)
            add_python_header "$file" "$sha"
            ;;
        *)
            echo "Unsupported file type: $file (only .rs and .py supported)"
            ;;
    esac
done

echo ""
echo "Note: Lineage SHA will be computed at commit time via pre-commit hook."
