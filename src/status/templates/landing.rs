/// Generate HTML status indicator for landing page
fn status_indicator(fail: usize, pending: usize, total: usize, pass: usize) -> String {
    if fail > 0 {
        format!(
            r#"<span class="status-indicator fail">&#10007; {} failed</span>"#,
            fail
        )
    } else if total == 0 {
        r#"<span class="status-indicator pending">No tests found</span>"#.to_string()
    } else if pending > 0 {
        format!(
            r#"<span class="status-indicator pending">? {} not yet run</span>"#,
            pending
        )
    } else {
        format!(
            r#"<span class="status-indicator pass">&#10003; All {} tests passing</span>"#,
            pass
        )
    }
}

/// CSS styles for landing page
fn landing_css() -> &'static str {
    r##"
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
"##
}

/// JavaScript for SSE live updates on landing page
fn landing_script() -> &'static str {
    r##"
(function() {
  var eventCount = 0;
  function addCounter() {
    var footer = document.querySelector('footer');
    if (footer && !document.getElementById('sse-count')) {
      var span = document.createElement('span');
      span.id = 'sse-count';
      span.style.cssText = 'display:block;margin-top:8px;font-family:var(--mono);font-size:1em;opacity:0.8;';
      span.textContent = 'live ● 0 updates';
      footer.appendChild(span);
    }
  }
  function updateCounter() {
    var el = document.getElementById('sse-count');
    if (el) el.textContent = 'live ● ' + eventCount + ' update' + (eventCount === 1 ? '' : 's');
  }
  addCounter();
  if (typeof EventSource !== 'undefined') {
    var es = new EventSource('/events');
    es.onmessage = function() {
      eventCount++;
      fetch(location.href).then(function(r) { return r.text(); }).then(function(html) {
        var doc = new DOMParser().parseFromString(html, 'text/html');
        var newBody = doc.querySelector('.container');
        var oldBody = document.querySelector('.container');
        if (newBody && oldBody) {
          oldBody.innerHTML = newBody.innerHTML;
          addCounter();
          updateCounter();
        }
      });
    };
    es.onerror = function() {
      var el = document.getElementById('sse-count');
      if (el) el.textContent = 'disconnected ○ ' + eventCount + ' updates';
      setTimeout(function() { es.close(); }, 5000);
    };
  }
}})();
"##
}

/// Generate HTML for landing page
pub fn landing_page(
    pattern: &str,
    server_started: &str,
    fail: usize,
    pass: usize,
    pending: usize,
    total: usize,
) -> String {
    let status_line = status_indicator(fail, pending, total, pass);
    let css = landing_css();
    let script = landing_script();
    format!(
        r##"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>reg-rs</title>
<style>{}</style>
</head><body>
<div class="container">
  <h1>reg-rs</h1>
  <div class="meta">pattern: <code>{}</code></div>

  <div class="summary">
    {}
    <div class="counts">
      <span>{} passed</span>
      <span>{} failed</span>
      <span>{} pending</span>
      <span>{} total</span>
    </div>
  </div>

  <ul class="views">
    <li><a href="/status">
      <div class="view-title">Status Dashboard</div>
      <div class="view-desc">Pass/fail details, collapsible sections, diffs for failures</div>
    </a></li>
  </ul>

  <footer>started {}</footer>
</div>
<script>{}</script>
</body></html>"##,
        css, status_line, pattern, pass, fail, pending, total, server_started, script
    )
}

/// Generate HTML for status dashboard page
pub fn status_page(_pattern: &str, server_started: &str, status_view_html: &str) -> String {
    let css = status_css();
    let script = status_script();
    format!(
        r##"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>reg-rs: status</title>
<style>{}</style>
</head><body>
<div class="container">
  {}
  <footer>
    started {} &middot; <a href="/">Back to Home</a>
  </footer>
</div>
<script>{}</script>
</body></html>"##,
        css, status_view_html, server_started, script
    )
}

fn status_script() -> &'static str {
    r##"
(function() {
  if (typeof EventSource !== 'undefined') {
    var es = new EventSource('/events');
    es.onmessage = function() {
      fetch(location.href).then(function(r) { return r.text(); }).then(function(html) {
        var doc = new DOMParser().parseFromString(html, 'text/html');
        var newBody = doc.querySelector('.container');
        var oldBody = document.querySelector('.container');
        if (newBody && oldBody) {
          oldBody.innerHTML = newBody.innerHTML;
        }
      });
    };
  }
})();
"##
}

fn status_css() -> &'static str {
    r##"
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
"##
}
