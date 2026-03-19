/// CSS styles for landing page
pub const LANDING_CSS: &str = r##"
  :root {
    --pass: #16a34a; --fail: #dc2626; --warn: #d97706; --muted: #6b7280;
    --bg: #f9fafb; --card: #fff; --border: #e5e7eb;
    --font: system-ui, -apple-system, sans-serif;
    --mono: ui-monospace, "SF Mono", Menlo, monospace;
  }
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body { font-family: var(--font); background: var(--bg); color: #111; padding: 40px 20px;
          font-size: 18px; }
  .container { max-width: 680px; margin: 0 auto; }
  h1 { font-size: 2em; margin-bottom: 4px; }
  .meta { color: var(--muted); font-size: 1em; margin-bottom: 20px; }
  .meta code { font-family: var(--mono); background: #f3f4f6; padding: 2px 6px;
               border-radius: 3px; }
  .summary { background: var(--card); border: 1px solid var(--border); border-radius: 8px;
             padding: 24px; margin-bottom: 24px; }
  .status-indicator { font-size: 1.3em; font-weight: 600; }
  .status-indicator.pass { color: var(--pass); }
  .status-indicator.fail { color: var(--fail); }
  .status-indicator.pending { color: var(--warn); }
  .counts { display: flex; gap: 24px; margin-top: 14px; font-size: 1.05em; color: var(--muted); }
  .counts span { font-family: var(--mono); }
  .views { list-style: none; padding: 0; }
  .views li { margin-bottom: 12px; }
  .views a { display: block; background: var(--card); border: 1px solid var(--border);
              border-radius: 8px; padding: 18px; text-decoration: none; color: #111;
              transition: border-color 0.15s; }
  .views a:hover { border-color: #3b82f6; }
  .views .view-title { font-size: 1.15em; font-weight: 600; }
  .views .view-desc { color: var(--muted); font-size: 1em; margin-top: 4px; }
  footer { text-align: center; color: var(--muted); font-size: 1em; margin-top: 32px; }
"##;

/// JavaScript for SSE live updates on landing page.
/// Server sends JSON: {"pass":N,"fail":N,"pending":N,"total":N}
/// JS updates DOM elements by ID directly — no fetch, no DOM parsing.
pub const LANDING_SCRIPT: &str = r##"
var n = 0;
var src = new EventSource('/events');
src.onmessage = function(e) {
  n++;
  document.getElementById('sse-badge').textContent = 'SSE: ' + n;
  var d = JSON.parse(e.data);
  document.getElementById('c-pass').textContent = d.pass + ' passed';
  document.getElementById('c-fail').textContent = d.fail + ' failed';
  document.getElementById('c-pending').textContent = d.pending + ' pending';
  document.getElementById('c-total').textContent = d.total + ' total';
  var s = document.getElementById('status-line');
  if (d.fail > 0) {
    s.innerHTML = '<span class="status-indicator fail">&#10007; ' + d.fail + ' failed</span>';
  } else if (d.pending > 0) {
    s.innerHTML = '<span class="status-indicator pending">? ' + d.pending + ' not yet run</span>';
  } else {
    s.innerHTML = '<span class="status-indicator pass">&#10003; All ' + d.total + ' tests passing</span>';
  }
};
src.onerror = function() {
  document.getElementById('sse-badge').style.background = '#dc2626';
  document.getElementById('sse-badge').textContent = 'SSE: off';
};
"##;

/// CSS for status dashboard (base + layout + stats + sections + items)
pub const STATUS_CSS: &str = r##"
  :root {
    --pass: #16a34a; --fail: #dc2626; --warn: #d97706; --muted: #6b7280;
    --bg: #f9fafb; --card: #fff; --border: #e5e7eb;
    --font: system-ui, -apple-system, sans-serif;
    --mono: ui-monospace, "SF Mono", Menlo, monospace;
  }
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body { font-family: var(--font); background: var(--bg); color: #111; padding: 20px; font-size: 16px; }
  .container { max-width: 1000px; margin: 0 auto; }
  header { display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 24px; border-bottom: 1px solid var(--border); padding-bottom: 16px; }
  header h1 { font-size: 1.5em; }
  header .meta { color: var(--muted); font-size: 0.9em; }
  header code { font-family: var(--mono); background: #f3f4f6; padding: 2px 4px; border-radius: 3px; }
  nav { display: flex; gap: 16px; margin-bottom: 24px; }
  nav a { text-decoration: none; color: #3b82f6; font-weight: 500; font-size: 0.9em; }
  nav a:hover { text-decoration: underline; }
  .overview { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 16px; margin-bottom: 32px; }
  .stat { background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 16px; text-align: center; }
  .stat .number { font-family: var(--mono); font-size: 1.8em; font-weight: 700; margin-bottom: 4px; }
  .stat .label { font-size: 0.85em; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; }
  .stat.fail .number { color: var(--fail); }
  .stat.pass .number { color: var(--pass); }
  .stat.pending .number { color: var(--warn); }
  .section { background: var(--card); border: 1px solid var(--border); border-radius: 8px; margin-bottom: 24px; overflow: hidden; }
  .section-header { padding: 12px 16px; background: #f8fafc; border-bottom: 1px solid var(--border); display: flex; align-items: center; gap: 12px; cursor: pointer; user-select: none; }
  .section-header h2 { font-size: 1.1em; flex-grow: 1; }
  .section.collapsed .section-body { display: none; }
  .section.collapsed .arrow { transform: rotate(-90deg); }
  .arrow { transition: transform 0.2s; font-size: 0.8em; color: var(--muted); }
  .badge { padding: 2px 8px; border-radius: 12px; font-size: 0.8em; font-weight: 600; font-family: var(--mono); }
  .badge-fail { background: #fee2e2; color: var(--fail); }
  .badge-pass { background: #dcfce7; color: var(--pass); }
  .badge-warn { background: #fef3c7; color: var(--warn); }
  .section-body { padding: 0; }
  .test-item { padding: 12px 16px; border-bottom: 1px solid var(--border); display: flex; gap: 12px; }
  .test-item:last-child { border-bottom: none; }
  .test-name { font-weight: 600; font-family: var(--mono); margin-bottom: 2px; }
  .test-time { font-size: 0.8em; color: var(--muted); margin-bottom: 8px; }
  .icon { font-size: 1.2em; flex-shrink: 0; }
  .icon-fail { color: var(--fail); }
  .icon-pass { color: var(--pass); }
  .icon-warn { color: var(--warn); }
  .diffs { font-family: var(--mono); font-size: 0.85em; background: #1e293b; color: #f8fafc; padding: 12px; border-radius: 4px; overflow-x: auto; white-space: pre; margin-top: 8px; }
  .empty { padding: 24px; text-align: center; color: var(--muted); font-style: italic; }
  footer { text-align: center; color: var(--muted); font-size: 0.9em; margin: 40px 0; border-top: 1px solid var(--border); padding-top: 20px; }
  footer a { color: #3b82f6; text-decoration: none; }
"##;

/// JavaScript for SSE live updates on status page.
/// Uses fetch to reload the full page content since the status view has
/// complex test listings that can't be updated with simple JSON.
pub const STATUS_SCRIPT: &str = r##"
var n = 0;
var badge = document.createElement('div');
badge.id = 'sse-badge';
badge.style.cssText = 'position:fixed;top:16px;right:16px;z-index:9999;background:#16a34a;color:#fff;font-family:ui-monospace,monospace;font-size:1.5em;font-weight:700;padding:8px 16px;border-radius:8px;box-shadow:0 2px 8px rgba(0,0,0,0.15);';
badge.textContent = 'SSE: 0';
document.body.appendChild(badge);
var src = new EventSource('/events');
src.onmessage = function() {
  n++;
  badge.textContent = 'SSE: ' + n;
  fetch('/status', {cache: 'no-store'}).then(function(r) {
    return r.text();
  }).then(function(html) {
    var doc = new DOMParser().parseFromString(html, 'text/html');
    var nc = doc.querySelector('.container');
    var oc = document.querySelector('.container');
    if (nc && oc) oc.innerHTML = nc.innerHTML;
  });
};
src.onerror = function() {
  badge.style.background = '#dc2626';
  badge.textContent = 'SSE: off';
};
"##;

/// Status view template (body content only — wrapped in HTML page by server)
pub const STATUS_VIEW_TEMPLATE: &str = "
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
