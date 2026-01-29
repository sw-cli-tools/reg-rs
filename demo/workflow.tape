# RTT1 Regression Detection Demo
# This VHS tape demonstrates how RTT1 detects regressions when command output changes
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
Type "# RTT1 - Regression Detection Demo"
Enter
Sleep 1s

# Build and setup (silently)
Type "cargo build --release 2>/dev/null && mkdir -p data/testdata"
Enter
Sleep 2s

# Clean any previous test
Type "rm -rf data/version_test.tdb* data/testdata"
Enter
Sleep 500ms
Type "mkdir -p data/testdata"
Enter
Sleep 500ms

# Create baseline test
Type "# Step 1: Create a baseline test"
Enter
Sleep 500ms
Type "echo 'version 1.0.0' > data/testdata/version.txt"
Enter
Sleep 500ms
Type "./target/release/rtt1 create -t data/version_test.tdb -c 'cat data/testdata/version.txt'"
Enter
Sleep 2s

# Run test - should pass
Type "# Step 2: Run the test - should pass"
Enter
Sleep 500ms
Type "./target/release/rtt1 run -p version_test"
Enter
Sleep 2s

# Report - no differences
Type "# Step 3: Report shows PASSED"
Enter
Sleep 500ms
Type "./target/release/rtt1 report -p version_test -v"
Enter
Sleep 2s

# Simulate a change (regression)
Type "# Step 4: Simulate a regression (change the output)"
Enter
Sleep 500ms
Type "echo 'version 2.0.0' > data/testdata/version.txt"
Enter
Sleep 1s

# Run test again - should detect regression
Type "# Step 5: Run the test again"
Enter
Sleep 500ms
Type "./target/release/rtt1 run -p version_test"
Enter
Sleep 2s

# Report with differences
Type "# Step 6: Report shows FAILED with differences"
Enter
Sleep 500ms
Type "./target/release/rtt1 report -p version_test -vvv"
Enter
Sleep 3s

# Clean up
Type "# Cleaning up"
Enter
Sleep 500ms
Type "rm -rf data/testdata data/version_test.tdb*"
Enter
Sleep 1s

Type "# Done! RTT1 detected the regression."
Enter
Sleep 2s
