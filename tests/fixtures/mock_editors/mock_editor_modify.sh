#!/bin/bash
# Mock editor that modifies the file content
# Used for testing /edit command's happy path
echo "SELECT 1 + 1;" > "$1"
exit 0
