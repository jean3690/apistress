<script setup lang="ts">
import { computed, ref } from 'vue'
import { useTestPlanStore } from '@/stores'
import type { ChildElement, ThreadGroup } from '@/types'
import { collectSamplersFromChildren } from '@/utils/tree-utils'

defineProps<{
  samplers: (ChildElement & { type: 'HttpSampler' })[]
  selectedId: string | null
}>()

const emit = defineEmits<{
  select: [id: string]
  delete: [id: string]
  add: []
  collapse: []
}>()

const testPlan = useTestPlanStore()

const groupedSamplers = computed(() => {
  const groups: { tg: ThreadGroup; samplers: (ChildElement & { type: 'HttpSampler' })[] }[] = []
  for (const tg of testPlan.plan.threadGroups) {
    const list = collectSamplersFromChildren(tg.children).filter(
      (s): s is ChildElement & { type: 'HttpSampler' } => 'type' in s && (s as ChildElement).type === 'HttpSampler',
    )
    if (list.length > 0) {
      groups.push({ tg, samplers: list })
    }
  }
  return groups
})

const collapsedGroups = ref<Record<string, boolean>>({})

function toggleGroup(id: string) {
  collapsedGroups.value[id] = !collapsedGroups.value[id]
}

function getPath(s: ChildElement & { type: 'HttpSampler' }): string {
  try {
    const sampler = s as unknown as { domain?: string; path?: string; port?: number; protocol?: string }
    const domain = sampler.domain || ''
    const path = sampler.path || '/'
    if (!domain) return path
    return `${domain}${path}`
  } catch {
    return ''
  }
}

function getMethod(s: ChildElement & { type: 'HttpSampler' }): string {
  return (s as unknown as { method?: string }).method || 'GET'
}
</script>

<template>
  <div class="flex flex-col overflow-hidden bg-deep">
    <div class="panel-header">
      <span>Requests</span>
      <div class="flex gap-1 items-center">
        <button class="action-btn" title="Add HTTP Request" @click="emit('add')">+</button>
        <button class="action-btn !text-8px" title="Collapse panel" @click="emit('collapse')">◀</button>
      </div>
    </div>
    <div class="flex-1 overflow-y-auto">
      <template v-for="group in groupedSamplers" :key="group.tg.id">
        <div
          class="flex items-center gap-1.5 px-3 py-1.5 cursor-pointer bg-surface border-b border-outline text-11px font-semibold select-none hover:bg-hover"
          @click="toggleGroup(group.tg.id)"
        >
          <span class="text-7px text-muted w-10px">{{ collapsedGroups[group.tg.id] ? '&#9654;' : '&#9660;' }}</span>
          <span class="flex-1 text-secondary truncate">{{ group.tg.name }}</span>
          <span class="text-9px text-muted bg-base px-1.5 py-0.5 rounded-2 font-mono">{{ group.samplers.length }}</span>
        </div>
        <template v-if="!collapsedGroups[group.tg.id]">
          <div
            v-for="s in group.samplers"
            :key="s.id"
            :class="['request-item', { selected: selectedId === s.id }]"
            @click="emit('select', s.id)"
          >
            <span :class="['method-badge', getMethod(s).toLowerCase()]">
              {{ getMethod(s) }}
            </span>
            <div class="flex-1 min-w-0 flex flex-col gap-0.5">
              <span class="font-medium truncate">{{ s.name }}</span>
              <span class="text-10px text-muted truncate font-mono">{{ getPath(s) }}</span>
            </div>
            <button
              v-if="samplers.length > 1"
              class="remove-btn"
              title="Delete request"
              @click.stop="emit('delete', s.id)"
            >
              &times;
            </button>
          </div>
        </template>
      </template>

      <div v-if="groupedSamplers.length === 0" class="py-10 px-5 text-center text-muted text-12px">
        <p>No HTTP Requests</p>
        <p class="mt-1.5 text-11px">Click + to add a request to the first thread group</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 7px 12px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.7px;
  color: var(--text-secondary);
}
.action-btn {
  width: 20px;
  height: 20px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
  transition: all 0.12s;
}
.action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  border-color: var(--text-muted);
}

.request-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  font-size: 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.02);
  border-left: 2px solid transparent;
  transition: background 0.1s;
}
.request-item:hover {
  background: var(--bg-hover);
}
.request-item.selected {
  background: var(--accent-glow);
  color: var(--text-primary);
  border-left-color: var(--accent);
}
.request-item.selected .request-path {
  color: var(--text-muted);
}
.request-item.selected .method-badge {
  opacity: 0.9;
}

.method-badge {
  width: 40px;
  padding: 2px 0;
  border-radius: 2px;
  font-size: 8px;
  font-weight: 800;
  text-align: center;
  background: var(--bg-surface);
  flex-shrink: 0;
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  letter-spacing: 0.3px;
}
.method-badge.get {
  color: var(--accent-cool);
}
.method-badge.post {
  color: var(--accent);
}
.method-badge.put {
  color: var(--warning);
}
.method-badge.delete {
  color: var(--danger);
}
.method-badge.patch {
  color: #9b8ec4;
}
.method-badge.head {
  color: #6d9ed4;
}
.method-badge.options {
  color: #6d9ed4;
}

.remove-btn {
  padding: 0 4px;
  border: none;
  background: transparent;
  color: var(--danger);
  font-size: 16px;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s;
  line-height: 1;
}
.request-item:hover .remove-btn {
  opacity: 0.7;
}
.remove-btn:hover {
  opacity: 1 !important;
}
</style>
