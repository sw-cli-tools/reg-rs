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

# Create baseline test
Type "# Step 1: Create a baseline test"
Enter
Sleep 500ms
Type "mkdir -p demo/data"
Enter
Sleep 500ms
Type "echo 'version 1.0.0' > demo/data/version.txt"
Enter
Sleep 500ms
Type "rtt1 create -t demo/version_test.db -c 'cat demo/data/version.txt'"
Enter
Sleep 2s

# Run test - should pass
Type "# Step 2: Run the test - should pass (no changes)"
Enter
Sleep 500ms
Type "rtt1 run -p demo/version_test.db"
Enter
Sleep 2s

# Report - no differences
Type "# Step 3: Report results - no differences"
Enter
Sleep 500ms
Type "rtt1 report -p demo/version_test.db -vvv"
Enter
Sleep 2s

# Simulate a change (regression)
Type "# Step 4: Simulate a regression (version change)"
Enter
Sleep 500ms
Type "echo 'version 2.0.0' > demo/data/version.txt"
Enter
Sleep 1s

# Run test again - should detect regression
Type "# Step 5: Run the test again - detects the change"
Enter
Sleep 500ms
Type "rtt1 run -p demo/version_test.db"
Enter
Sleep 2s

# Report with differences
Type "# Step 6: Report shows the differences"
Enter
Sleep 500ms
Type "rtt1 report -p demo/version_test.db -vvv"
Enter
Sleep 3s

# Clean up
Type "# Cleaning up..."
Enter
Sleep 500ms
Type "rm -rf demo/data demo/version_test.db"
Enter
Sleep 1s

Type "# Done! RTT1 successfully detected the regression."
Enter
Sleep 2s
