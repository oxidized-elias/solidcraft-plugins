#!/usr/bin/env bash
set -euo pipefail

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
shopt -s nullglob
builds=("$root"/plugins/*/build.sh)
if (( ${#builds[@]} == 0 )); then
  echo "No plugins found." >&2
  exit 1
fi

plugin=${1:-}
if (( $# > 1 )); then
  echo "Usage: ./install.sh [plugin]" >&2
  exit 1
fi
if [[ -z $plugin ]]; then
  if [[ ! -t 0 ]]; then
    echo "Specify a plugin for non-interactive use." >&2
    exit 1
  fi
  echo "Choose a plugin:"
  for i in "${!builds[@]}"; do
    printf '%s) %s\n' "$((i + 1))" "$(basename "$(dirname "${builds[i]}")")"
  done
  read -r -p 'Selection: ' selection
  if [[ ! $selection =~ ^[1-9][0-9]*$ ]] || (( ${#selection} > 5 || 10#$selection > ${#builds[@]} )); then
    echo "Invalid selection." >&2
    exit 1
  fi
  plugin=$(basename "$(dirname "${builds[10#$selection - 1]}")")
fi
if [[ ! $plugin =~ ^[a-z0-9]+(-[a-z0-9]+)*$ || ! -f "$root/plugins/$plugin/build.sh" ]]; then
  echo "Unknown plugin: $plugin" >&2
  exit 1
fi

bash "$root/plugins/$plugin/build.sh"
echo "Install the resulting ZIP in SolidCraft via Plugins → Install plugin…."
