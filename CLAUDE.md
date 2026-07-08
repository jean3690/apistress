# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

ApiStress is a **JMeter-like API stress-testing desktop app** built with **Tauri v2** (Rust backend) + **Vue 3 / TypeScript / Pinia** (frontend). It allows building test plans with a tree of HTTP samplers, controllers, assertions, timers, and listeners, then executing them with concurrency.

## Commands

```bash
pnpm dev              # Start Vite dev server (port 1420) — browser-only, no Tauri backend
pnpm build            # Type-check (vue-tsc --noEmit) then production build (vite build)
pnpm preview          # Preview production build
pnpm tauri dev        # Start Tauri dev (Vite frontend + Rust backend together)
pnpm tauri build      # Build native desktop binary
```

`pnpm dev` runs the frontend standalone (useful for UI development). Rust-backed features (test execution via `invoke('start_test')`) only work under `pnpm tauri dev` or `pnpm tauri build`.

## Architecture

### Type hierarchy (JMeter model)

`TestPlan` → `ThreadGroup[]` → `children: ChildElement[]`

`ChildElement` is a union of `HttpSampler`, `ControllerUnion`, `AssertionUnion`, `ConfigUnion`, `TimerUnion`, `ProcessorUnion`, and `ListenerConfig`. Controllers (`LoopController`, `IfController`, `WhileController`, `TransactionController`, `ThroughputController`) can recursively nest `children`. All types are in `src/types/`.

### Stores (Pinia)

| Store | Purpose |
|---|---|
| `testPlanStore` | Full test plan tree CRUD: add/remove/update/move/duplicate nodes, JSON export/import |
| `executionStore` | Test execution lifecycle: calls `invoke('start_test')` / `invoke('stop_test')` on the Rust backend, listens for `test://result` and `test://status` Tauri events, computes aggregate stats |
| `uiStore` | UI mode (`classic`/`modern`) and active result tab, persisted to `localStorage` |

`@/` maps to `src/` via Vite alias.

### Frontend ↔ Backend communication

The frontend invokes two Tauri commands on the Rust side:
- `invoke('start_test', { planJson })` — begins executing a serialized test plan
- `invoke('stop_test')` — signals the backend to stop

Results stream back via Tauri events (`listen('test://result', ...)`, `listen('test://status', ...)`).

**Note:** These Tauri commands are NOT yet implemented in `src-tauri/src/lib.rs` (only a stub `greet` command exists). The frontend gracefully degrades when running in browser-only dev mode.

### Layout modes

- **Classic** (`ClassicLayout.vue`): Three-panel JMeter-style — collapsible tree on the left, properties editor in the center, results table/tree/chart on the right.
- **Modern** (`ModernLayout.vue`): Postman-style — flat request list on the left, request editor (URL bar + headers) in the center, response viewer on the right. Includes a one-off "Send" button that calls `fetch()` directly (not via the Rust engine).

Both modes share the same Pinia stores and tree utils.

### Key utilities

`src/utils/tree-utils.ts` provides recursive tree operations (`findNodeById`, `removeNodeById`, `addChildToParent`, `walkTree`, `buildTreeData`, `collectAllSamplers`) used by both the stores and the UI components.
