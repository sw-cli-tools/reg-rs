/// Status view template (body content only — wrapped in HTML page by server)
pub static STATUS_VIEW_TEMPLATE: &str = "
<header>
  <h1>reg-rs</h1>
  <span class=\"meta\">pattern: <code>{ test_pattern }</code></span>
  <span class=\"meta\">updated { state_updated }</span>
</header>

<nav>
  <a href=\"#overview\">Overview</a>
  <a href=\"#failures\">Failures</a>
  <a href=\"#passes\">Passes</a>
  <a href=\"#pending\">Pending</a>
</nav>

<div class=\"overview\" id=\"overview\">
  <div class=\"stat fail\">
    <div class=\"number\">{ status_counts.fail_count }</div>
    <div class=\"label\">&#10007; failed</div>
  </div>
  <div class=\"stat pass\">
    <div class=\"number\">{ status_counts.pass_count }</div>
    <div class=\"label\">&#10003; passed</div>
  </div>
  <div class=\"stat pending\">
    <div class=\"number\">{ status_counts.not_run_count }</div>
    <div class=\"label\">? not yet run</div>
  </div>
  <div class=\"stat total\">
    <div class=\"number\">{ status_counts.test_count }</div>
    <div class=\"label\">total</div>
  </div>
</div>

<div class=\"section\" id=\"failures\">
  <div class=\"section-header\" onclick=\"this.parentElement.classList.toggle('collapsed')\">
    <span class=\"arrow\">&#9660;</span>
    <h2>Failures</h2>
    <span class=\"badge badge-fail\">{ status_counts.fail_count }</span>
  </div>
  <div class=\"section-body\">
    {{ if status_flags.no_failed_tests }}<div class=\"empty\">No failed tests</div>{{ else }}
    {{ for run in test_runs }}{{ if run.diffs }}
    <div class=\"test-item\">
      <span class=\"icon icon-fail\" title=\"Failed\">&#10007;</span>
      <div>
        <div class=\"test-name\">{ run.name }</div>
        <div class=\"test-time\">created { run.created } &middot; failed { run.last_ran }</div>
        <div class=\"diffs\">{{ for diff in run.diffs }}{ diff | unescaped }{{ endfor }}</div>
      </div>
    </div>
    {{ endif }}{{ endfor }}
    {{ endif }}
  </div>
</div>

<div class=\"section\" id=\"passes\">
  <div class=\"section-header\" onclick=\"this.parentElement.classList.toggle('collapsed')\">
    <span class=\"arrow\">&#9660;</span>
    <h2>Passes</h2>
    <span class=\"badge badge-pass\">{ status_counts.pass_count }</span>
  </div>
  <div class=\"section-body\">
    {{ if status_flags.no_passed_tests }}<div class=\"empty\">No passed tests</div>{{ else }}
    {{ for run in test_runs }}{{ if run.last_ran }}{{ if not run.diffs }}
    <div class=\"test-item\">
      <span class=\"icon icon-pass\" title=\"Passed\">&#10003;</span>
      <div>
        <div class=\"test-name\">{ run.name }</div>
        <div class=\"test-time\">created { run.created } &middot; last ran { run.last_ran }</div>
      </div>
    </div>
    {{ endif }}{{ endif }}{{ endfor }}
    {{ endif }}
  </div>
</div>

<div class=\"section\" id=\"pending\">
  <div class=\"section-header\" onclick=\"this.parentElement.classList.toggle('collapsed')\">
    <span class=\"arrow\">&#9660;</span>
    <h2>Not Yet Run</h2>
    <span class=\"badge badge-warn\">{ status_counts.not_run_count }</span>
  </div>
  <div class=\"section-body\">
    {{ if status_flags.no_not_yet_run_tests }}<div class=\"empty\">All tests have been run</div>{{ else }}
    {{ for run in test_runs }}{{ if not run.last_ran }}
    <div class=\"test-item\">
      <span class=\"icon icon-warn\" title=\"Not yet run\">?</span>
      <div>
        <div class=\"test-name\">{ run.name }</div>
        <div class=\"test-time\">created { run.created }</div>
      </div>
    </div>
    {{ endif }}{{ endfor }}
    {{ endif }}
  </div>
</div>
";
