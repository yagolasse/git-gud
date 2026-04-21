#!/bin/bash

# Create a test repository
TEST_REPO="/tmp/git-gud-test-repo"
echo "Creating test repository at: $TEST_REPO"

# Clean up any existing test repo
rm -rf "$TEST_REPO"

# Create and initialize repository
mkdir -p "$TEST_REPO"
cd "$TEST_REPO" || exit 1
git init
echo "Test file 1" > file1.txt
echo "Test file 2" > file2.txt
git add file1.txt
git commit -m "Initial commit"
echo "Modified content" >> file1.txt
git add file2.txt

echo "Repository created with:"
echo "  - 1 committed file (file1.txt)"
echo "  - 1 staged file (file2.txt)" 
echo "  - 1 modified file (file1.txt modified but not staged)"

# Build the project
echo -e "\nBuilding Git Gud..."
cd - || exit 1
cargo build

# Test 1: Run GUI with repository path
echo -e "\nTest 1: Running GUI with repository path..."
echo "Command: cargo run -- gui \"$TEST_REPO\""
cargo run -- gui "$TEST_REPO" 2>&1 | head -20

# Test 2: Run GUI without path (should show open dialog)
echo -e "\nTest 2: Running GUI without path..."
echo "Command: cargo run -- gui"
cargo run -- gui 2>&1 | head -20

echo -e "\nTest completed. Repository remains at: $TEST_REPO"
echo "You can manually inspect it or run: rm -rf \"$TEST_REPO\""