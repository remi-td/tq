#!/bin/bash
# Read Memory Guard - Prevents reading large files without pagination
# Exit code 2 blocks the operation and provides feedback to the agent

# Read the JSON input from stdin
input=$(cat)

# Extract parameters from JSON
file_path=$(echo "$input" | jq -r '.parameters.file_path // empty')
offset=$(echo "$input" | jq -r '.parameters.offset // empty')
limit=$(echo "$input" | jq -r '.parameters.limit // empty')

if [ -z "$file_path" ]; then
  exit 0
fi

# Check if file exists
if [ ! -f "$file_path" ]; then
  # File doesn't exist, let the tool handle the error
  exit 0
fi

# Size limit: 50MB
SIZE_LIMIT=$((50 * 1024 * 1024))
# Line limit: 50,000 lines
LINE_LIMIT=50000

# Get file size
file_size=$(stat -f%z "$file_path" 2>/dev/null || stat -c%s "$file_path" 2>/dev/null)

# Get line count (efficiently - only count if file is not too large)
if [ "$file_size" -lt "$SIZE_LIMIT" ]; then
  line_count=$(wc -l < "$file_path" 2>/dev/null || echo "0")
else
  line_count=$((LINE_LIMIT + 1))  # Force failure for large files
fi

# Check if file exceeds limits and pagination is not used
if [ "$file_size" -gt "$SIZE_LIMIT" ] || [ "$line_count" -gt "$LINE_LIMIT" ]; then
  if [ -z "$offset" ] || [ -z "$limit" ]; then
    echo "BLOCKED: File is too large to read without pagination."
    echo ""
    echo "File: $file_path"
    echo "Size: $(numfmt --to=iec-i --suffix=B $file_size 2>/dev/null || echo "${file_size} bytes")"
    echo "Lines: $line_count"
    echo ""
    echo "Limits: 50MB or 50,000 lines"
    echo ""
    echo "To read this file, use the offset and limit parameters:"
    echo "  - offset: Starting line number"
    echo "  - limit: Number of lines to read"
    echo ""
    echo "Example: Read first 1000 lines:"
    echo "  { \"file_path\": \"$file_path\", \"offset\": 0, \"limit\": 1000 }"
    exit 2
  fi
fi

# Allow the read to proceed
exit 0
