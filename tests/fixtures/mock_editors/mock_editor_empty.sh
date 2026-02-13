#!/bin/bash
# Mock editor that empties the file
# Used for testing /edit command's empty content path
> "$1"
exit 0
