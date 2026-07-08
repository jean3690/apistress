<script setup lang="ts">
import type { ChildElement } from '@/types'

defineProps<{
  samplers: (ChildElement & { type: 'HttpSampler' })[]
  selectedId: string | null
}>()

const emit = defineEmits<{
  select: [id: string]
  delete: [id: string]
  add: []
}>()
</script>

<template>
  <div class="panel request-list-panel">
    <div class="panel-header">
      <span>Requests</span>
      <button class="action-btn" title="Add HTTP Request" @click="emit('add')">+</button>
    </div>
    <div class="request-list">
      <div
        v-for="s in samplers"
        :key="s.id"
        :class="['request-item', { selected: selectedId === s.id }]"
        @click="emit('select', s.id)"
      >
        <span :class="['method-badge', s.method.toLowerCase()]">{{ s.method }}</span>
        <span class="request-name">{{ s.name }}</span>
        <button
          v-if="samplers.length > 1"
          class="remove-btn"
          title="Delete request"
          @click.stop="emit('delete', s.id)"
        >x</button>
      </div>
      <div v-if="samplers.length === 0" class="empty-list">
        <p>No HTTP Requests</p>
        <p class="hint">Click + to add a request</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.request-list-panel {
  width: 220px;
  min-width: 160px;
  background: var(--bg-secondary);
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

.action-btn {
  padding: 2px 8px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
}

.action-btn:hover { background: var(--bg-hover); color: var(--text-primary); }

.request-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.request-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  cursor: pointer;
  font-size: 12px;
}

.request-item:hover { background: var(--bg-hover); }
.request-item.selected { background: var(--accent); color: var(--bg-primary); }

.method-badge {
  width: 42px;
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 700;
  text-align: center;
  background: var(--bg-surface);
  color: var(--text-primary);
  flex-shrink: 0;
}

.method-badge.get { color: var(--success); }
.method-badge.post { color: var(--accent); }
.method-badge.put { color: var(--warning); }
.method-badge.delete { color: var(--danger); }
.method-badge.patch { color: #cba6f7; }

.request-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.remove-btn {
  padding: 0 4px;
  border: none;
  background: transparent;
  color: var(--danger);
  font-size: 14px;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s;
}

.request-item:hover .remove-btn { opacity: 1; }

.empty-list {
  padding: 30px 16px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}

.empty-list .hint { margin-top: 6px; font-size: 11px; color: var(--text-muted); }
</style>
