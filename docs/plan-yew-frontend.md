# Plan: Yew/WASM Frontend

## Status: Possible Future Feature

## Summary
Replace the current server-rendered HTML + JS EventSource client with a
Yew/Rust/WASM single-page application for the status dashboard.

## Motivation
- All-Rust stack (no JS)
- Incremental DOM updates without full page reload
- Type-safe component model for the UI
- Potential for richer interactions (filtering, sorting, search)

## Architecture

### New crate: `frontend/`
- Yew components: `App`, `StatusDashboard`, `TestItem`, `DiffView`, `Overview`
- SSE client via `gloo-net::eventsource` or `web-sys::EventSource`
- JSON API from axum replaces HTML template rendering

### Build
- `trunk build` compiles Rust → WASM + JS glue + index.html
- Output in `frontend/dist/`
- Axum serves `dist/` as static files via `tower-http::services::ServeDir`

### New axum routes
- `GET /api/status` — JSON status payload (replaces HTML rendering)
- `GET /events` — SSE stream (unchanged)
- `GET /` — serves `index.html` (WASM app entry)

### Dependencies
- `yew = "0.21"`
- `gloo-net = "0.5"` (SSE client)
- `serde`, `serde_json` (shared types between server and frontend)
- `trunk` CLI tool (build)

## Trade-offs
- **Pro:** All Rust, incremental updates, type-safe UI, animation support
- **Con:** ~2-3MB WASM bundle, trunk build dependency, slower dev iteration,
  more complex CI pipeline, harder to debug in browser

## Current Alternative
The current implementation uses server-rendered HTML with a lightweight JS
EventSource client that fetches updated content and swaps the DOM on SSE
events. This achieves automatic updates with minimal complexity.

## Prerequisites
- Stabilize the current SSE + incremental update approach
- Determine if the UI complexity justifies the WASM toolchain
- Evaluate bundle size impact for the target audience
