#!/bin/sh
# Run the grammar's own tests, then parse every fixture and fail if one
# contains an ERROR node.
#
#   tests/check-fixtures.sh
#
# Needs a C compiler and the tree-sitter CLI (npm install -g tree-sitter-cli).
# It uses the grammar in the working tree, not the commit pinned in
# extension.toml.
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root/tree-sitter-jinja"

tree-sitter test

status=0
for file in "$root"/tests/fixtures/*.j2; do
  if tree-sitter parse -q "$file" >/dev/null 2>&1; then
    echo "ok     ${file#"$root"/}"
  else
    echo "ERROR  ${file#"$root"/}"
    tree-sitter parse "$file" 2>/dev/null | grep -n "ERROR\|MISSING" || true
    status=1
  fi
done

exit $status
