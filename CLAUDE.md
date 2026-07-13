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

`ChildElement` is a discriminated union keyed on `type` (string tag):

| Category            | Types                                                                                                                         |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| **Samplers (8)**    | `HttpSampler`, `GraphQlSampler`, `SseSampler`, `MqttSampler`, `WebSocketSampler`, `GrpcSampler`, `TcpSampler`, `RedisSampler` |
| **Controllers (5)** | `LoopController`, `IfController`, `WhileController`, `TransactionController`, `ThroughputController`                          |
| **Assertions (3)**  | `ResponseAssertion`, `JsonAssertion`, `DurationAssertion`                                                                     |
| **Timers (3)**      | `ConstantTimer`, `UniformRandomTimer`, `GaussianRandomTimer`                                                                  |
| **Extractors (3)**  | `RegexExtractor`, `JsonExtractor`, `BoundaryExtractor`                                                                        |
| **Config (4)**      | `HttpDefaults`, `CsvDataSet`, `UserVariables`, `UserParameters`                                                               |
| **Listeners**       | `ListenerConfig` (6 variants: tree, summary, aggregate, etc.)                                                                 |
| **Processors**      | `ProcessorUnion`                                                                                                              |

Controllers recursively nest `children: ChildElement[]`. HttpSampler body supports `none`, `raw`, `form-data` (with text/file fields), and `x-www-form-urlencoded` modes. ThreadGroup has `warmUp` (seconds), `onErrorAction`, `duration`, `rampUp`, `numThreads` fields.

All sampler types are defined in `src/types/sampler.ts` with factory functions (e.g., `createDefaultHttpSampler()`). The full `ChildElement` union and `ThreadGroup`/`TestPlan` live in `src/types/testplan.ts`.

The **Rust backend mirrors these types** in `src-tauri/src/engine/plan.rs` using serde's `#[serde(tag = "type")]` for the same tagged-enum deserialization. The frontend serializes the test plan to JSON and passes it via `invoke('start_test', { planJson })`.

### Stores (Pinia)

| Store            | Path                           | Purpose                                                                                                                                                                                                                                                                                                 |
| ---------------- | ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `testPlanStore`  | `src/stores/testPlanStore.ts`  | Full test plan tree CRUD: add/remove/update/move/duplicate nodes, JSON export/import                                                                                                                                                                                                                    |
| `executionStore` | `src/stores/executionStore.ts` | Test execution lifecycle: `invoke('start_test')` / `invoke('stop_test')`, listens for `test://result` and `test://status` Tauri events, computes aggregate stats (percentiles, throughput, error rates, assertions). Uses `resultsVersion` shallowRef as a batching throttle for computed recalculation |
| `uiStore`        | `src/stores/uiStore.ts`        | UI mode (`classic`/`modern`), active result tab, split pane sizes, persisted to `localStorage`                                                                                                                                                                                                          |

`@/` maps to `src/` via Vite alias.

### Rust backend engine (`src-tauri/src/engine/`)

| Module            | Purpose                                                                                                                                                                                                                                      |
| ----------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `plan.rs`         | All test plan type definitions mirroring the frontend types                                                                                                                                                                                  |
| `result.rs`       | `SampleResult` struct, `StatusPayload` (250ms intervals with p50/p90/p99/avg/throughput), `ExecutionContext`, `AssertionResult`                                                                                                              |
| `runner.rs`       | Entry point: `start_test_plan()` creates reqwest Client (HTTP) + rumqttc (MQTT) + redis (Redis) clients, global percentile buffer (`ELAPSED_BUFFER`), status-emitter task, response body cache. Cancellation via `OnceLock<Arc<AtomicBool>>` |
| `thread_group.rs` | Virtual user loop: warm-up phase, ramp-up delay, duration limit, iteration counting, CSV data injection, walks children via `execute_level()`. Handles `onErrorAction` (continue/stopThread/stopTest/startNextLoop)                          |
| `sampler.rs`      | HTTP/GraphQL/SSE/MQTT/WebSocket/gRPC/TCP/Redis request execution. URL construction, header/auth/body handling, variable interpolation (`{{var}}`, `${__threadNum}`, `${__time()}`), `HttpDefaults` collection, retry with count/delay        |
| `controller.rs`   | Controller evaluation returning `ControllerAction::Execute`/`Skip`/`Break`. Per-instance loop counters and throughput-timing maps                                                                                                            |
| `assertion.rs`    | Evaluates `ResponseAssertion` (contains/matches/equals with negation), `JsonAssertion` (exists/notExists/equals), `DurationAssertion`                                                                                                        |
| `timer.rs`        | Timer evaluation returning `TimerAction::Delay(duration)`. Gaussian timer uses Box-Muller transform                                                                                                                                          |

`lib.rs` registers Tauri commands: `start_test`, `stop_test`, `get_response_body`. Plugins: opener, dialog, fs, store, http, notification, window-state.

### Frontend ↔ Backend communication

1. Frontend calls `invoke('start_test', { planJson })` — Rust deserializes JSON into `TestPlan` and spawns virtual users.
2. Each request emits `test://result` with a `SampleResult` payload.
3. A status task emits `test://status` every 250ms with `{ status, threadsActive, totalSamples, errorCount, p50, p90, p99, avgResponseTime, throughput }`.
4. Frontend calls `invoke('stop_test')` to set the global cancel flag.

### Layout modes

- **Classic** (`ClassicLayout.vue`): Three-panel JMeter-style with drag-to-resize. Collapsible tree on left, properties editor in center, results panel on right with Live/Summary/Tree/Chart tabs. Context menu via right-click.
- **Modern** (`ModernLayout.vue`): Postman-style three-panel with drag-to-resize and per-panel collapse. Flat request list grouped by thread group on left, request editor (URL bar + Headers/Body/Auth/Settings tabs) in center, response viewer (Body/Headers tabs) on right. Bottom execution bar with live stats and collapsible aggregate results panel.

Both modes share the same Pinia stores and tree utils. Shell: `AppShell.vue` (layout switcher), `ToolBar.vue` (run/stop/import/export), `StatusBar.vue` (live thread/sample/error counts).

### Properties editor structure

`PropertiesPanel.vue` delegates to three sub-components under `src/components/classic/properties/`:

| Component            | Handles                                                                       |
| -------------------- | ----------------------------------------------------------------------------- |
| `PlanEditor.vue`     | TestPlan + ThreadGroup properties                                             |
| `SamplerEditors.vue` | All 8 sampler types (one editor per type via `v-if` on `node.type`)           |
| `ElementEditors.vue` | Controllers, assertions, timers, extractors, config elements, user parameters |

The parent narrows the generic `TestElementUnion` node into typed computed refs (`planNode`, `samplerNode`, `elementNode`) before passing to each sub-editor.

### Key utilities

- `src/utils/tree-utils.ts` — Recursive tree operations: `findNodeById`, `removeNodeById`, `addChildToParent`, `walkTree`, `buildTreeData`, `collectAllSamplers`, `collectSamplersFromChildren` (public).
- `src/utils/jmx.ts` — Full bidirectional JMX import/export. `pushChildElement()` is a large switch function (~25 cases) that needs updating when new element types are added. Parse functions read XML; the export side must write all fields (e.g., `retryCount`/`retryDelay` for round-trip fidelity).
- `src/composables/useFileIO.ts` — File open/save via Tauri dialog + fs plugins with browser fallback.
- `src/composables/useKeyboardShortcuts.ts` — Global keyboard shortcuts: Delete, Ctrl+S, Ctrl+R/F5, Esc. Skips when focus is in input elements.
- `src/composables/useChartOptions.ts` — ECharts chart options (throughput, response time, error rate, threads) for the live dashboard. Uses inline hex colors since ECharts doesn't support CSS variables.

### Vue conventions

- **`shallowRef` for primitives**: Use `shallowRef()` instead of `ref()` for string, number, boolean, null values. Use `ref()` only for objects/arrays that need deep reactivity (e.g., arrays mutated via `.push()`/`.splice()` or objects with `v-model` on nested properties).
- **`useTemplateRef` for DOM refs**: Vue 3.5+ — use `useTemplateRef<T>('name')` instead of `ref<T | null>(null)` for template refs.
- **Script → Template → Style** order in all SFCs. PascalCase component names.
- **Props and emits are typed**: `defineProps<{ ... }>()`, `defineEmits<{ ... }>()`.
- CSS: global tokens in `App.vue` (`<style>` without scoped), component styles use `<style scoped>` with class selectors.

### Design system

CSS variables defined in `App.vue` (`<style>` block, not scoped):

| Variable                                               | Role                                                              |
| ------------------------------------------------------ | ----------------------------------------------------------------- |
| `--bg-deep` / `--bg-secondary`                         | Deepest background (panels, sidebars)                             |
| `--bg-base` / `--bg-primary`                           | Main background                                                   |
| `--bg-surface`                                         | Raised surfaces (headers, cards)                                  |
| `--bg-hover`                                           | Hover state                                                       |
| `--border`                                             | Grid lines, separators                                            |
| `--accent`                                             | Primary accent (amber `#f0a040`) — active states, Run, selections |
| `--accent-cool` / `--success`                          | Teal — success, throughput                                        |
| `--danger`                                             | Error states                                                      |
| `--warning`                                            | Dirty state, warnings                                             |
| `--text-primary` / `--text-secondary` / `--text-muted` | Text hierarchy                                                    |

Do **not** use `@import url()` for external fonts in CSS — Tauri's webview cannot load external resources during initial render. Use system font stacks.

### Caveats

- **`RequestList.vue`**: Must use `collectSamplersFromChildren(tg.children)` for per-thread-group sampler collection. Do NOT spread a ThreadGroup into `collectAllSamplers()` — it expects a `TestPlan` with `threadGroups`.
- **JMX round-trips**: When adding a new field to sampler types, update both the parse function AND the corresponding case in `pushChildElement()` (jmx.ts) to avoid data loss on import→export→import.
- **ECharts colors**: Chart option objects in `ResultsPanel.vue` and `useChartOptions.ts` use inline hex values, not CSS variables. Match the design palette manually.
- **`ELAPSED_BUFFER`** (runner.rs): Global `OnceLock<Arc<Mutex<Vec<u64>>>>` — must be initialized via `get_or_init` before use, not a separate local `Arc::new`.
