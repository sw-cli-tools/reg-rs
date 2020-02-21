pub static STATUS_VIEW_TEMPLATE: &str = "<h2>RTT Status Server</h2>
<small>(server started { server_started })</small>
<div>
<span>{ fail_count }<span> failed
{ not_run_count } not yet run
{ pass_count } passed
 -----
{ test_count } matched pattern: { test_pattern }
</div>";
