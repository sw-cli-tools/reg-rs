# reg-rs Basic Workflow Demo
# This VHS tape demonstrates the basic workflow: create, run, report, remove
#
# To generate the GIF:
#   vhs demo/basic.tape

# Setup
Output demo/basic.gif
Set FontSize 14
Set Width 900
Set Height 600
Set Theme "Dracula"

# Title
Type "# reg-rs - Regression Test Tool - Basic Workflow"
Enter
Sleep 1s

# Build the binary first (silently)
Type "cargo build --release 2>/dev/null"
Enter
Sleep 2s

# Set data dir to project-local work directory
Type "export REG_RS_DATA_DIR=./work/reg-rs && mkdir -p $REG_RS_DATA_DIR"
Enter
Sleep 500ms

# Clean any previous test
Type "rm -f $REG_RS_DATA_DIR/hello.tdb $REG_RS_DATA_DIR/hello.tdb.lock"
Enter
Sleep 500ms

# Show short help
Type "# Show available commands"
Enter
Sleep 500ms
Type "./target/release/reg-rs -h"
Enter
Sleep 2s

# Create a test
Type "# Step 1: Create a test"
Enter
Sleep 500ms
Type "./target/release/reg-rs create -t hello -c 'echo hello world'"
Enter
Sleep 2s

# Run the test
Type "# Step 2: Run the test (compare against baseline)"
Enter
Sleep 500ms
Type "./target/release/reg-rs run -p hello"
Enter
Sleep 2s

# Report results
Type "# Step 3: Report results"
Enter
Sleep 500ms
Type "./target/release/reg-rs report -p hello -v"
Enter
Sleep 2s

# Clean up
Type "# Step 4: Clean up"
Enter
Sleep 500ms
Type "./target/release/reg-rs remove -p hello"
Enter
Sleep 1s

Type "# Done!"
Enter
Sleep 2s
