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
  --bg-primary: #1e1e2e;
  --bg-secondary: #181825;
  --bg-surface: #313244;
  --bg-hover: #45475a;
  --text-primary: #cdd6f4;
  --text-secondary: #a6adc8;
  --text-muted: #6c7086;
  --accent: #89b4fa;
  --accent-hover: #74c7ec;
  --success: #a6e3a1;
  --warning: #f9e2af;
  --danger: #f38ba8;
  --border: #45475a;
  --shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
  font-size: 13px;
  color: var(--text-primary);
  background: var(--bg-primary);
}

html, body, #app {
  height: 100%;
  width: 100%;
  overflow: hidden;
}

::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}
::-webkit-scrollbar-track {
  background: var(--bg-secondary);
}
::-webkit-scrollbar-thumb {
  background: var(--bg-hover);
  border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
  background: var(--text-muted);
}
</style>
