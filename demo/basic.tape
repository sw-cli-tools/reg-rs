# reg-rs Basic Workflow Demo
# This VHS tape demonstrates the basic workflow: create, run, list, show, report, remove
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
Type "cargo build --release 2>/dev/null && cargo install --path . 2>/dev/null"
Enter
Sleep 2s

# Load aliases
Type "source bin/source-rg.sh"
Enter
Sleep 500ms

# Set data dir to project-local work directory
Type "export REG_RS_DATA_DIR=./work/reg-rs && mkdir -p $REG_RS_DATA_DIR"
Enter
Sleep 500ms

# Clean any previous test
Type "rm -f $REG_RS_DATA_DIR/hello.*"
Enter
Sleep 500ms

# Show alias help
Type "# Show available aliases"
Enter
Sleep 500ms
Type "hlrg"
Enter
Sleep 2s

# Create a test
Type "# Step 1: Create a test"
Enter
Sleep 500ms
Type "adrg hello 'echo hello world' --desc 'Hello output test'"
Enter
Sleep 2s

# List tests
Type "# Step 2: List tests (shows pending — not yet run)"
Enter
Sleep 500ms
Type "lsrg hello"
Enter
Sleep 2s

# Run the test
Type "# Step 3: Run the test (compare against baseline)"
Enter
Sleep 500ms
Type "rnrg hello"
Enter
Sleep 2s

# Show test details
Type "# Step 4: Show test details"
Enter
Sleep 500ms
Type "shrg hello -v"
Enter
Sleep 2s

# Report results
Type "# Step 5: Report results"
Enter
Sleep 500ms
Type "reg-rs report -p hello -v"
Enter
Sleep 2s

# Clean up
Type "# Step 6: Clean up"
Enter
Sleep 500ms
Type "rmrg hello"
Enter
Sleep 1s

Type "# Done!"
Enter
Sleep 2s
