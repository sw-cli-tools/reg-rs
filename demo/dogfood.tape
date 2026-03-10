# reg-rs Dogfood Demo
# reg-rs tests its own CLI output for regressions
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
Type "cargo build --release 2>/dev/null && mkdir -p data"
Enter
Sleep 2s

# Clean previous
Type "rm -f data/reg_rs_*.tdb data/reg_rs_*.tdb.lock"
Enter
Sleep 500ms

# Step 1: Create self-tests
Type "# Step 1: Create regression tests for each reg-rs command"
Enter
Sleep 500ms
Type "./target/release/reg-rs c -t data/reg_rs_help.tdb -c './target/release/reg-rs -h'"
Enter
Sleep 1500ms

Type "./target/release/reg-rs c -t data/reg_rs_create_help.tdb -c './target/release/reg-rs create -h'"
Enter
Sleep 1500ms

Type "./target/release/reg-rs c -t data/reg_rs_run_help.tdb -c './target/release/reg-rs run -h'"
Enter
Sleep 1500ms

Type "./target/release/reg-rs c -t data/reg_rs_report_help.tdb -c './target/release/reg-rs report -h'"
Enter
Sleep 1500ms

# Step 2: Run self-tests
Type "# Step 2: Run the self-tests (compare against baselines)"
Enter
Sleep 500ms
Type "./target/release/reg-rs run -p reg_rs"
Enter
Sleep 2s

# Step 3: Report
Type "# Step 3: Report - all self-tests should pass"
Enter
Sleep 500ms
Type "./target/release/reg-rs report -p reg_rs -vv"
Enter
Sleep 3s

# Clean up
Type "# reg-rs eats its own dog food!"
Enter
Sleep 500ms
Type "./target/release/reg-rs remove -p reg_rs"
Enter
Sleep 1s

Type "# Done!"
Enter
Sleep 2s
