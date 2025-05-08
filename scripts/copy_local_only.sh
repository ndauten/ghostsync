#!/bin/bash

SOURCE="$1"
DEST="$2"

if [[ -z "$SOURCE" || -z "$DEST" ]]; then
  echo "Usage: $0 <source_path> <destination_path>"
  exit 1
fi

mkdir -p "$DEST"

echo "Copying local (non-dataless) files from: $SOURCE"
echo "To destination: $DEST"
echo

find "$SOURCE" -type f | while read -r file; do
  flags=$(ls -ldO "$file" 2>/dev/null | awk '{print $5}')
  if [[ "$flags" != *dataless* ]]; then
    relpath="${file#$SOURCE/}"
    target="$DEST/$relpath"
    if [[ -f "$target" ]]; then
      echo "Already exists, skipping: $relpath"
    else
      mkdir -p "$(dirname "$target")"
      cp -p "$file" "$target"
      echo "Copied: $relpath"
    fi
  else
    echo "Skipped (dataless): $file"
  fi
done

