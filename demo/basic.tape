# RTT1 Basic Workflow Demo
# This VHS tape demonstrates the basic workflow: create, run, report, remove
#
# To generate the GIF:
#   vhs demo/basic.tape

# Setup
Output demo/basic.gif
Set FontSize 14
Set Width 800
Set Height 600
Set Theme "Dracula"

# Title
Type "# RTT1 - Regression Test Tool"
Enter
Type "# Basic Workflow Demo"
Enter
Sleep 1s

# Show help
Type "rtt1 --help"
Enter
Sleep 2s

# Create a test
Type "# Step 1: Create a test"
Enter
Sleep 500ms
Type "rtt1 create -t demo/hello.db -c 'echo hello world'"
Enter
Sleep 2s

# Run the test
Type "# Step 2: Run the test (should pass - same output)"
Enter
Sleep 500ms
Type "rtt1 run -p demo/hello.db"
Enter
Sleep 2s

# Report results
Type "# Step 3: Report on test results"
Enter
Sleep 500ms
Type "rtt1 report -p demo/hello.db -v"
Enter
Sleep 2s

# Clean up
Type "# Step 4: Remove the test"
Enter
Sleep 500ms
Type "rtt1 remove -p demo/hello.db"
Enter
Sleep 2s

Type "# Done!"
Enter
Sleep 2s
