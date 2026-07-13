<template>
  <div class="flex flex-col overflow-hidden min-w-280px h-full bg-background">
    <div class="panel-header">
      <span>Response</span>
      <div class="flex gap-0.5">
        <span :class="['tab', { active: activeTab === 'body' }]" @click="activeTab = 'body'">Body</span>
        <span :class="['tab', { active: activeTab === 'headers' }]" @click="activeTab = 'headers'">
          Headers<span v-if="parsedHeaders.length" class="tab-count">{{ parsedHeaders.length }}</span>
        </span>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto">
      <div v-if="loading" class="flex flex-col items-center justify-center py-60px px-5 gap-3 text-muted">
        <div class="spinner" />
        <p>Sending request...</p>
      </div>

      <template v-else-if="response">
        <div class="flex items-center gap-2.5 px-3 py-2 border-b border-outline">
          <span :class="['status-badge', response.statusCode && response.statusCode < 400 ? 'ok' : 'error']">
            {{ response.status }}
          </span>
          <span v-if="response.time !== undefined" class="text-11px text-muted font-mono"> {{ response.time }}ms </span>
          <span class="text-11px text-muted font-mono">
            {{ formatBytes(response.size ?? bodySize) }}
          </span>
        </div>

        <div v-if="activeTab === 'body'" class="flex-1 overflow-hidden flex flex-col h-full">
          <div v-if="isJson" class="flex-1 min-h-200px">
            <CodeMirrorEditor language="json" :model-value="response.body" :read-only="true" />
          </div>
          <pre v-else class="body-pre">{{ response.body || '(empty response)' }}</pre>
        </div>

        <div v-if="activeTab === 'headers'" class="flex-1 overflow-hidden flex flex-col h-full">
          <div v-if="parsedHeaders.length">
            <div
              v-for="(h, i) in parsedHeaders"
              :key="i"
              class="flex items-start gap-3 px-3 py-1.5 text-12px border-b border-white/3"
            >
              <span class="font-semibold text-primary min-w-140px shrink-0 font-mono">{{ h.key }}</span>
              <span class="text-muted break-all">{{ h.value }}</span>
            </div>
          </div>
          <div v-else class="p-5 text-center text-muted text-12px">No response headers captured</div>
        </div>
      </template>

      <div v-else class="py-60px px-5 text-center text-muted">
        <div class="text-36px mb-2 opacity-30">&#8593;</div>
        <p>Select a request and click <strong>Send</strong></p>
        <p class="text-11px mt-1 opacity-50">or click <strong>Run</strong> in the toolbar to execute the test plan</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { shallowRef, computed } from 'vue'
import CodeMirrorEditor from '@/components/editors/CodeMirrorEditor.vue'

const props = defineProps<{
  loading: boolean
  response: { status: string; statusCode?: number; body: string; headers: string; time?: number; size?: number } | null
}>()

const activeTab = shallowRef<'body' | 'headers'>('body')

const isJson = computed(() => {
  if (!props.response?.body) return false
  const trimmed = props.response.body.trim()
  return (trimmed.startsWith('{') || trimmed.startsWith('[')) && trimmed.length > 1
})

const parsedHeaders = computed(() => {
  if (!props.response?.headers) return []
  try {
    const lines = props.response.headers.split('\n').filter(Boolean)
    return lines.map(line => {
      const sep = line.indexOf(':')
      if (sep === -1) return { key: '', value: line }
      return { key: line.substring(0, sep).trim(), value: line.substring(sep + 1).trim() }
    })
  } catch {
    return []
  }
})

const bodySize = computed(() => {
  const body = props.response?.body || ''
  return new Blob([body]).size
})

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`
}
</script>

<style scoped>
.panel-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.7px;
  color: var(--text-secondary);
}
.tab {
  padding: 4px 10px;
  font-size: 10px;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 3px;
  transition: all 0.15s;
  font-weight: 500;
}
.tab:hover {
  color: var(--text-primary);
}
.tab.active {
  background: var(--accent);
  color: var(--bg-deep);
  font-weight: 600;
}
.tab-count {
  display: inline-block;
  margin-left: 4px;
  padding: 0 4px;
  font-size: 8px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 8px;
  font-weight: 600;
}

.status-badge {
  padding: 3px 10px;
  border-radius: 2px;
  font-weight: 700;
  font-size: 12px;
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}
.status-badge.ok {
  background: rgba(77, 201, 176, 0.12);
  color: var(--accent-cool);
}
.status-badge.error {
  background: var(--danger-glow);
  color: var(--danger);
}

.body-pre {
  background: var(--bg-deep);
  border: 1px solid var(--border);
  border-radius: 3px;
  padding: 12px;
  margin: 8px;
  font-size: 12px;
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
  flex: 1;
  overflow-y: auto;
  max-height: calc(100vh - 200px);
}

.spinner {
  width: 28px;
  height: 28px;
  border: 3px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
