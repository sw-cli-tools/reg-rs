pub static DIFFERENCES_REPORT_TEMPLATE: &str = "
** Differences **
⍨ { failed_test_name } 
{{ for difference in differences }}{ difference.type_name } - { difference.chunk }
{{ endfor }}
";

pub static SUMMARY_REPORT_TEMPLATE: &str = "RTT Summary Report { report_date }
{ pass_count } passed
{ fail_count } failed
{ not_run_count } not yet run
 -----
{ test_count } matched pattern: { test_pattern }";

pub static DETAILS_REPORT_TEMPLATE: &str ="
* Details *{{ if no_failed_tests }}
No Failed Tests{{ else }}
⍨ Failures: {{ for failed_test in failed_test_names }}{ failed_test }{{ if not @last }}, {{ endif }}{{ endfor }}{{ endif }}
{{- if no_not_yet_run_tests }}{{ else }}
? Not Yet Run: {{ for not_yet_run_test in not_yet_run_test_names }}{ not_yet_run_test }{{ if not @last }}, {{ endif }}{{ endfor }}{{ endif -}}
{{ if no_passed_tests }}
No Passed Tests{{ else }}
✓ Passed: {{ for passed_test in passed_test_names }}{ passed_test }{{ if not @last }}, {{ endif }}{{ endfor }}{{ endif }}
";

pub static FAILURES_REPORT_TEMPLATE: &str =
    "⍨ { failed_test_name } - created: { time_created }, failed: { time_last_ran }, differences count: { differences_count }
{{- for difference in difference_types }}
{{- if @first }}, difference types:{{ endif -}}{ required_blank }{ difference }
{{- if not @last }},{{ endif -}}{{ endfor -}}
";
