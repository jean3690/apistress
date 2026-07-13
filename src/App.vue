<template>
  <AppShell />
</template>

<script setup lang="ts">
import AppShell from './components/layout/AppShell.vue'
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  --bg-deep: #181825;
  --bg-base: #1e1e2e;
  --bg-surface: #313244;
  --bg-hover: #45475a;
  --border: #45475a;
  --border-light: #585b70;

  --text-primary: #cdd6f4;
  --text-secondary: #a6adc8;
  --text-muted: #6c7086;

  --accent: #f0a040;
  --accent-glow: rgba(240, 160, 64, 0.15);
  --accent-cool: #4dc9b0;
  --success: #4dc9b0;
  --warning: #f9e2af;
  --danger: #f38ba8;
  --danger-glow: rgba(243, 139, 168, 0.1);

  --shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  --shadow-lg: 0 8px 24px rgba(0, 0, 0, 0.5);

  --bg-primary: var(--bg-base);
  --bg-secondary: var(--bg-deep);
  --accent-hover: #f5b850;

  font-family:
    'Segoe UI',
    system-ui,
    -apple-system,
    sans-serif;
  font-size: 13px;
  color: var(--text-primary);
  background: var(--bg-base);
}

html,
body,
#app {
  height: 100%;
  width: 100%;
  overflow: hidden;
}

/* Scrollbar — precision thin */
::-webkit-scrollbar {
  width: 5px;
  height: 5px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 0;
}
::-webkit-scrollbar-thumb:hover {
  background: var(--text-muted);
}

/* Selection */
::selection {
  background: rgba(240, 160, 64, 0.3);
  color: var(--text-primary);
}

/* Focus ring — amber, subtle */
:focus-visible {
  outline: 1px solid var(--accent);
  outline-offset: 1px;
}

/* Monospace utility */
.mono,
.metric,
.number {
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-variant-numeric: tabular-nums;
}
</style>
