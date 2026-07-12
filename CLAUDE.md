# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

ApiStress is a **JMeter-like API stress-testing desktop app** built with **Tauri v2** (Rust backend) + **Vue 3 / TypeScript / Pinia** (frontend). It allows building test plans with a tree of HTTP samplers, controllers, assertions, timers, and listeners, then executing them with concurrency. Supports bidirectional **JMX import/export** for JMeter compatibility.

## Commands

```bash
pnpm dev              # Start Vite dev server (port 1420) — browser-only, no Tauri backend
pnpm build            # Type-check (vue-tsc --noEmit) then production build (vite build)
pnpm preview          # Preview production build
pnpm tauri dev        # Start Tauri dev (Vite frontend + Rust backend together)
pnpm tauri build      # Build native desktop binary
```

`pnpm dev` runs the frontend standalone (useful for UI development). Rust-backed features (test execution via `invoke('start_test')`) only work under `pnpm tauri dev` or `pnpm tauri build`. The frontend gracefully degrades in browser-only mode — Tauri event listeners and plugin imports are wrapped in try/catch.

## Architecture

### Type hierarchy (JMeter model)

`TestPlan` → `ThreadGroup[]` → `children: ChildElement[]`

`ChildElement` is a discriminated union keyed on `type` (string tag). Includes `HttpSampler`, 5 controllers (`LoopController`, `IfController`, `WhileController`, `TransactionController`, `ThroughputController`), 3 assertions (`ResponseAssertion`, `JsonAssertion`, `DurationAssertion`), 3 config elements (`HttpDefaults`, `CsvDataSet`, `UserVariables`), 3 timers (`ConstantTimer`, `UniformRandomTimer`, `GaussianRandomTimer`), 3 extractors (`RegexExtractor`, `JsonExtractor`, `BoundaryExtractor`), `UserParameters`, and 6 listener types. Controllers recursively nest `children: ChildElement[]`. HttpSampler body supports `none`, `raw`, `form-data` (with text/file fields), and `x-www-form-urlencoded` modes.

The **Rust backend mirrors these types** in `src-tauri/src/engine/plan.rs` using serde's `#[serde(tag = "type")]` for the same tagged-enum deserialization. The frontend serializes the test plan to JSON and passes it via `invoke('start_test', { planJson })`.

### Stores (Pinia)

| Store | Path | Purpose |
|---|---|---|
| `testPlanStore` | `src/stores/testPlanStore.ts` | Full test plan tree CRUD: add/remove/update/move/duplicate nodes, JSON export/import |
| `executionStore` | `src/stores/executionStore.ts` | Test execution lifecycle: calls `invoke('start_test')` / `invoke('stop_test')`, listens for `test://result` and `test://status` Tauri events, computes aggregate stats (percentiles, throughput, error rates) |
| `uiStore` | `src/stores/uiStore.ts` | UI mode (`classic`/`modern`), active result tab, split pane sizes, persisted to `localStorage` |

`@/` maps to `src/` via Vite alias.

### Rust backend engine (`src-tauri/src/engine/`)

| Module | Purpose |
|---|---|
| `plan.rs` | All test plan type definitions mirroring the frontend types (`TestPlan`, `ThreadGroup`, `ChildElement` enum with all variants, etc.) |
| `result.rs` | `SampleResult` struct (emitted per-request), `StatusPayload` (emitted at 250ms intervals), `ExecutionContext`, `AssertionResult` |
| `runner.rs` | Entry point: `start_test_plan()` creates reqwest `Client`s, spawns a status-emitter task (250ms loop), spawns one tokio task per virtual user, uses `OnceLock<Arc<AtomicBool>>` for global cancellation |
| `thread_group.rs` | Virtual user loop: ramp-up delay, duration limit, iteration counting, CSV data injection per-iteration, calls `execute_level()` which walks children recursively. Handles `onErrorAction` (continue/stopThread/stopTest/startNextLoop) |
| `sampler.rs` | HTTP request execution via reqwest: URL construction, header/auth/body handling, variable interpolation (`{{var}}`, `${__threadNum}`, `${__time()}`), `HttpDefaults` collection and application |
| `controller.rs` | Controller evaluation returning `ControllerAction::Execute`/`Skip`/`Break`. Maintains per-instance loop counters and throughput-timing maps |
| `assertion.rs` | Evaluates `ResponseAssertion` (contains/matches/equals with negation), `JsonAssertion` (exists/notExists/equals), `DurationAssertion` |
| `timer.rs` | Timer evaluation returning `TimerAction::Delay(duration)`. Gaussian timer uses Box-Muller transform |

`lib.rs` registers two Tauri commands: `start_test` (deserializes JSON plan → calls `runner::start_test_plan`) and `stop_test` (sets cancellation flag). Plugins: opener, dialog, fs, store, http, notification, window-state.

### Frontend ↔ Backend communication

1. Frontend calls `invoke('start_test', { planJson })` — Rust deserializes the JSON into `TestPlan` and spawns virtual users.
2. Each request emits `test://result` with a `SampleResult` payload.
3. A status task emits `test://status` every 250ms with `{ status, threadsActive, totalSamples, errorCount }`.
4. Frontend calls `invoke('stop_test')` to set the global cancel flag.

### Layout modes

- **Classic** (`ClassicLayout.vue`): Three-panel JMeter-style — collapsible tree (`TreeView.vue`) on the left, properties editor (`PropertiesPanel.vue`) in the center, results panel (`ResultsPanel.vue`) with table/tree/chart tabs on the right. Includes context menu support (`ContextMenu.vue`).
- **Modern** (`ModernLayout.vue`): Postman-style — flat request list (`RequestList.vue`) on the left, request editor (`RequestEditor.vue`) with URL bar + headers + body editor in the center, response viewer (`ResponsePanel.vue`) on the right. Includes a one-off "Send" button that calls `fetch()` directly.

Both modes share the same Pinia stores and tree utils. Shell components: `AppShell.vue` (top-level layout switcher), `ToolBar.vue` (run/stop/import/export buttons), `StatusBar.vue` (execution status bar).

### Key utilities

- `src/utils/tree-utils.ts` — Recursive tree operations (`findNodeById`, `removeNodeById`, `addChildToParent`, `walkTree`, `buildTreeData`, `collectAllSamplers`) used by both stores and UI components.
- `src/utils/jmx.ts` — Full bidirectional JMX import/export. `importJmx(xml)` parses JMeter XML (handling the `<hashTree>` sibling structure) into an ApiStress `TestPlan`. `exportJmx(plan)` serializes back to JMeter-compatible XML. Covers all element types including form-data and multipart file uploads.
- `src/composables/useFileIO.ts` — File open/save via Tauri dialog + fs plugins with browser fallback (uses `<input type="file">` and download links when Tauri isn't available).
- `src/composables/useKeyboardShortcuts.ts` — Global keyboard shortcuts: Delete (remove selected node), Ctrl+S (save), Ctrl+R/F5 (run test), Esc (stop test). Skips when focus is in input elements.

### Result detail view

Clicking a sample in the results tree opens a detail panel with three tabs:
- **Response**: Response headers and formatted body
- **Request**: Request headers that were sent
- **Assertions**: Per-assertion pass/fail results with failure messages

### Editor components

- `src/components/editors/MonacoJsonEditor.vue` — Lazy-loaded Monaco editor for JSON body editing in the PropertiesPanel. Uses dark theme with JSON syntax highlighting. Dynamically imported to keep initial bundle size reasonable.
