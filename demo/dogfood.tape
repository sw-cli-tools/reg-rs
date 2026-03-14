# reg-rs Dogfood Demo
# reg-rs tests its own CLI output for regressions, then migrates to .rgt format
#
# To generate the GIF:
#   vhs demo/dogfood.tape

# Setup
Output demo/dogfood.gif
Set FontSize 14
Set Width 1000
Set Height 700
Set Theme "Dracula"

# Title
Type "# reg-rs dogfood: testing itself for regressions"
Enter
Sleep 1s

# Build (silently)
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

# Clean previous
Type "rm -f $REG_RS_DATA_DIR/reg_rs_*.tdb $REG_RS_DATA_DIR/reg_rs_*.tdb.lock"
Type " $REG_RS_DATA_DIR/reg_rs_*.rgt $REG_RS_DATA_DIR/reg_rs_*.out"
Enter
Sleep 500ms

# Step 1: Create self-tests
Type "# Step 1: Create regression tests for each reg-rs command"
Enter
Sleep 500ms
Type "adrg reg_rs_help 'reg-rs -h'"
Enter
Sleep 1500ms

Type "adrg reg_rs_create_help 'reg-rs create -h'"
Enter
Sleep 1500ms

Type "adrg reg_rs_run_help 'reg-rs run -h'"
Enter
Sleep 1500ms

Type "adrg reg_rs_report_help 'reg-rs report -h'"
Enter
Sleep 1500ms

# Step 2: List tests
Type "# Step 2: List the self-tests"
Enter
Sleep 500ms
Type "lsrg reg_rs"
Enter
Sleep 2s

# Step 3: Run self-tests
Type "# Step 3: Run the self-tests (compare against baselines)"
Enter
Sleep 500ms
Type "rnrg reg_rs"
Enter
Sleep 2s

# Step 4: Report
Type "# Step 4: Report - all self-tests should pass"
Enter
Sleep 500ms
Type "reg-rs report -p reg_rs -vv"
Enter
Sleep 3s

# Step 5: Migrate to .rgt
Type "# Step 5: Migrate to git-friendly .rgt format"
Enter
Sleep 500ms
Type "mgrg reg_rs"
Enter
Sleep 2s

# Step 6: Show a migrated test
Type "# Step 6: Show a migrated test"
Enter
Sleep 500ms
Type "shrg reg_rs_help -v"
Enter
Sleep 3s

# Clean up
Type "# reg-rs eats its own dog food!"
Enter
Sleep 500ms
Type "rmrg reg_rs"
Enter
Sleep 500ms
Type "rm -f $REG_RS_DATA_DIR/reg_rs_*.rgt $REG_RS_DATA_DIR/reg_rs_*.out"
Enter
Sleep 1s

Type "# Done!"
Enter
Sleep 2s
