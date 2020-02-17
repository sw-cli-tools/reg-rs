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
{ pass_count } tests have passed
{ fail_count } tests have failed
{ not_run_count } tests have not yet been run
 -----
{ test_count } tests match pattern { test_pattern }
";
