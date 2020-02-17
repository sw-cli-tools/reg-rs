pub static DETAILS_TEMPLATE: &str = "
{heading}
---

{{ for detail in values }} {detail} {{ endfor }}
";

pub static DIFFERENCES_TEMPLATE: &str = "
{heading}
===

{{ for diff in values }}
{diff.type} - {diff.chunk}
{{ endfor }}
";

pub static REPORT_TEMPLATE: &str = "
{heading}
===

Test {{ for name in tests }} {name} {{ endfor }}
---
{{ call details_template with details }}
---
{{ call differences_template with differences }}
---
";

pub static SUMMARY_REPORT_TEMPLATE: &str = "
RTT Summary Report { report_date }
{ pass_count } passed
{ fail_count } failed
{ not_run_count } not yet run
 -----
{ test_count } matched pattern: { test_pattern }
";

pub static DETAILS_REPORT_TEMPLATE: &str = "
Details
===
Failed tests: 
{{ for failed_test in failed_test_names }} 
  { failed_test } 
{{ endfor }}

Not Yet Run tests:
{{ for not_yet_run_test in not_yet_run_test_names }} 
  { not_yet_run_test } 
{{ endfor }}

Passed tests:
{{ for passed_test in passed_test_names }} 
  { passed_test } 
{{ endfor }}
";
