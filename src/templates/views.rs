pub static STATUS_VIEW_TEMPLATE: &str = "<small>(server started { server_started })</small>
<h2>RTT Status Server</h2>
<br />
<div><h3>Summary</h3>
<div><span>{ fail_count }<span> failed</div>
<div>{ not_run_count } not yet run</div>
<div>{ pass_count } passed</div>
<div>=====</div>
<div>{ test_count } matched pattern: { test_pattern }</div>
</div>
<div><h3>Details</h3>
<div>{{ if no_failed_tests }}
No Failed Tests{{ else }}
{ fail_symbol } Failures: {{ for failed_test in failed_test_names }}{ failed_test }{{ if not @last }},&nbsp;{{ endif }}{{ endfor }}{{ endif }}</div>
<div>{{- if no_not_yet_run_tests }}{{ else }}
{ warn_symbol } Not Yet Run: {{ for not_yet_run_test in not_yet_run_test_names }}{ not_yet_run_test }{{ if not @last }}, {{ endif }}{{ endfor }}{{ endif -}}</div>
<div>{{ if no_passed_tests }}
No Passed Tests{{ else }}
{ pass_symbol } Passed: {{ for passed_test in passed_test_names }}{ passed_test }{{ if not @last }}, {{ endif }}{{ endfor }}{{ endif }}</div>
</div>
<div><h3>Failures</h3>
<div><h4>Differences</h4>
</div>
<div><h3>Passes</h3>
</div>

";

// see mono-rust/tinytemplate2

// <div>{{- for failed_test_name in failed_test_names -}}
// <div>{ fail_symbol } { failed_test_name } - created: { time_created }, failed: { time_last_ran }, differences count: { differences_count }</div>

// <div>{{- for difference in difference_types }}
// {{- if @first }}, difference types:{{ endif -}}{ required_blank }{ difference }
// {{- if not @last }},{{ endif -}}{{ endfor -}}</div>
// </div>

// {{ endfor }}</div>


