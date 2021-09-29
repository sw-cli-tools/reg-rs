/// status view template
pub static STATUS_VIEW_TEMPLATE: &str = "<small>(server started { server_started })</small>
<small>(state updated { state_updated })</small><small>(version TBD)</small>
<h2>RTT Status Server</h2>
<br />

<div><h3>Summary</h3>
<div><span>{ status_counts.fail_count }<span> failed</div>
<div>{ status_counts.not_run_count } not yet run</div>
<div>{ status_counts.pass_count } passed</div>
<div>=====</div>
<div>{ status_counts.test_count } matched pattern: { test_pattern }</div>
</div>

<div><h3>Details</h3>
<div>{{ if status_flags.no_failed_tests }}
No Failed Tests{{ else }}
{ fail_symbol } Failures: <ul>{{ for run in test_runs }}{{ if run.diffs }}<li>{ run.name }</li>{{ endif }}{{ endfor }}</ul>{{ endif }}</div>
<div>{{- if status_flags.no_not_yet_run_tests }}{{ else }}
{ warn_symbol } Not Yet Run: <ul>{{ for run in test_runs }}{{ if not run.last_ran }}<li>{ run.name }</li>{{ endif }}{{ endfor }}</ul>{{ endif -}}</div>
<div>{{ if status_flags.no_passed_tests }}
No Passed Tests{{ else }}
{ pass_symbol } Passed: <ul>{{ for run in test_runs }}{{ if run.last_ran }}{{ if not run.diffs }}<li>{ run.name }</li>{{ endif }}{{ endif }}{{ endfor }}</ul>{{ endif }}</div>

<div><h3>Failures</h3><ul>
{{ for run in test_runs }}{{ if run.diffs }}
<li>{ fail_symbol } { run.name } - created: { run.created }, failed: { run.last_ran }

<div><p>Differences</p><ul>
  {{for diff in run.diffs }}
   <li>{ diff }</li>{{ endfor }}</ul>
{{ endif }}<br /></li>{{ endfor }}</ul></div>

<div><h3>Passes</h3><ul>
{{ for run in test_runs }}{{ if not run.diffs }}{{ if run.last_ran }}
<li>{ pass_symbol } { run.name } - created: { run.created }, last ran: { run.last_ran }</li>
{{ endif }}{{ endif }}{{ endfor }}</ul></div>

";
