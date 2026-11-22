#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf -- "$tmp"' EXIT
cp "$root/install.sh" "$tmp/"
mkdir -p "$tmp/plugins/example"
printf '%s\n' '#!/usr/bin/env bash' 'echo "built example"' > "$tmp/plugins/example/build.sh"
[[ $(bash "$tmp/install.sh" example) == *'built example'* ]]
if bash "$tmp/install.sh" ../example >/dev/null 2>&1; then exit 1; fi
if bash "$tmp/install.sh" </dev/null >/dev/null 2>&1; then exit 1; fi
output=$(printf '1\n' | script -q -e -c "bash '$tmp/install.sh'" /dev/null)
[[ $output == *'built example'* ]]
if printf '2\n' | script -q -e -c "bash '$tmp/install.sh'" /dev/null >/dev/null; then exit 1; fi
echo 'Installer tests passed.'
