<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  loading: boolean
  response: { status: string; body: string; headers: string } | null
}>()

const activeTab = ref<'body' | 'headers'>('body')
</script>

<template>
  <div class="panel response-panel">
    <div class="panel-header">
      <span>Response</span>
      <div class="resp-tabs">
        <span :class="['tab', { active: activeTab === 'body' }]" @click="activeTab = 'body'">Body</span>
        <span :class="['tab', { active: activeTab === 'headers' }]" @click="activeTab = 'headers'">Headers</span>
      </div>
    </div>
    <div class="response-content">
      <div v-if="loading" class="loading-state">Loading...</div>
      <div v-else-if="response">
        <div :class="['status-line', response.status.startsWith('Error') ? 'error' : 'ok']">
          {{ response.status }}
        </div>
        <div v-if="activeTab === 'headers' && response.headers" class="resp-section">
          <pre>{{ response.headers }}</pre>
        </div>
        <div v-if="activeTab === 'body'" class="resp-section">
          <pre>{{ response.body }}</pre>
        </div>
      </div>
      <div v-else class="empty-state">
        <p>Select a request and click Send</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.panel { display: flex; flex-direction: column; overflow: hidden; }

.response-panel {
  width: 420px;
  min-width: 280px;
  background: var(--bg-primary);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.resp-tabs { display: flex; gap: 2px; }

.resp-tabs .tab {
  padding: 4px 12px;
  font-size: 11px;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 4px;
}

.resp-tabs .tab.active { background: var(--accent); color: var(--bg-primary); }

.response-content { flex: 1; overflow-y: auto; padding: 10px; }

.loading-state { padding: 20px; text-align: center; color: var(--text-muted); }

.status-line {
  padding: 8px 12px;
  border-radius: 4px;
  margin-bottom: 10px;
  font-weight: 600;
  font-size: 13px;
}

.status-line.ok { background: rgba(166, 227, 161, 0.12); color: var(--success); }
.status-line.error { background: rgba(243, 139, 168, 0.12); color: var(--danger); }

.resp-section pre {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 10px;
  font-size: 12px;
  font-family: 'SF Mono', 'Consolas', 'Fira Code', monospace;
  line-height: 1.5;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 400px;
  overflow-y: auto;
}

.empty-state { padding: 60px 20px; text-align: center; color: var(--text-muted); }
</style>
