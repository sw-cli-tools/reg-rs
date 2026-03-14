# reg-rs Regression Detection Demo
# This VHS tape demonstrates how reg-rs detects regressions and supports rebase
#
# To generate the GIF:
#   vhs demo/workflow.tape

# Setup
Output demo/workflow.gif
Set FontSize 14
Set Width 900
Set Height 700
Set Theme "Dracula"

# Title
Type "# reg-rs - Regression Detection & Rebase Demo"
Enter
Sleep 1s

# Build and setup (silently)
Type "cargo build --release 2>/dev/null && cargo install --path . 2>/dev/null"
Enter
Sleep 2s

# Load aliases
Type "source bin/source-rg.sh"
Enter
Sleep 500ms

# Set data dir to project-local work directory
Type "export REG_RS_DATA_DIR=./work/reg-rs && mkdir -p $REG_RS_DATA_DIR/testdata"
Enter
Sleep 500ms

# Clean any previous test
Type "rm -rf $REG_RS_DATA_DIR/version_test.tdb* $REG_RS_DATA_DIR/version_test.rgt"
Type " $REG_RS_DATA_DIR/version_test.out $REG_RS_DATA_DIR/testdata"
Enter
Sleep 500ms
Type "mkdir -p $REG_RS_DATA_DIR/testdata"
Enter
Sleep 500ms

# Create baseline test
Type "# Step 1: Create a baseline test"
Enter
Sleep 500ms
Type "echo 'version 1.0.0' > $REG_RS_DATA_DIR/testdata/version.txt"
Enter
Sleep 500ms
Type "adrg version_test 'cat $REG_RS_DATA_DIR/testdata/version.txt'"
Enter
Sleep 2s

# Migrate to .rgt format
Type "# Step 2: Migrate to git-friendly .rgt format"
Enter
Sleep 500ms
Type "mgrg version_test"
Enter
Sleep 2s

# Run test - should pass
Type "# Step 3: Run the test - should pass"
Enter
Sleep 500ms
Type "rnrg version_test"
Enter
Sleep 2s

# List - shows PASS
Type "# Step 4: List shows PASS"
Enter
Sleep 500ms
Type "lsrg version_test"
Enter
Sleep 2s

# Simulate a change (regression)
Type "# Step 5: Simulate a regression (change the output)"
Enter
Sleep 500ms
Type "echo 'version 2.0.0' > $REG_RS_DATA_DIR/testdata/version.txt"
Enter
Sleep 1s

# Run test again - should detect regression
Type "# Step 6: Run the test again"
Enter
Sleep 500ms
Type "rnrg version_test"
Enter
Sleep 2s

# Show with diffs
Type "# Step 7: Show details with diffs"
Enter
Sleep 500ms
Type "shrg version_test -vv"
Enter
Sleep 3s

# Rebase - accept the new output
Type "# Step 8: Accept the new output (rebase)"
Enter
Sleep 500ms
Type "uprg version_test"
Enter
Sleep 2s

# Run again - should pass now
Type "# Step 9: Run again - passes with new baseline"
Enter
Sleep 500ms
Type "rnrg version_test"
Enter
Sleep 1s
Type "lsrg version_test"
Enter
Sleep 2s

# Clean up
Type "# Cleaning up"
Enter
Sleep 500ms
Type "rm -rf $REG_RS_DATA_DIR/testdata $REG_RS_DATA_DIR/version_test.*"
Enter
Sleep 1s

Type "# Done! reg-rs detected the regression, then accepted the new baseline."
Enter
Sleep 2s
