<script setup lang="ts">
import { computed } from 'vue'
import { useTestPlanStore } from '@/stores'
import PlanEditor from './properties/PlanEditor.vue'
import SamplerEditors from './properties/SamplerEditors.vue'
import ElementEditors from './properties/ElementEditors.vue'
import type {
  TestElementUnion,
  TestPlan,
  ThreadGroup,
  ChildElement,
  ListenerConfig,
  HttpSampler,
  GraphQlSampler,
  SseSampler,
  MqttSampler,
  WebSocketSampler,
  GrpcSampler,
  TcpSampler,
  RedisSampler,
} from '@/types'

const testPlan = useTestPlanStore()

type SamplerNode =
  HttpSampler | GraphQlSampler | SseSampler | MqttSampler | WebSocketSampler | GrpcSampler | TcpSampler | RedisSampler

type ElementNode = Exclude<Exclude<ChildElement, SamplerNode>, ListenerConfig>

const node = computed(() => testPlan.selectedNode as TestElementUnion | null)

function update(key: string, value: unknown) {
  if (node.value) {
    testPlan.updateNode(node.value.id, { [key]: value })
  }
}

function updateNested(path: string[], value: unknown) {
  if (!node.value) return
  const obj = JSON.parse(JSON.stringify(node.value))
  let current: Record<string, unknown> = obj
  for (let i = 0; i < path.length - 1; i++) {
    if (!current[path[i]]) current[path[i]] = {}
    current = current[path[i]] as Record<string, unknown>
  }
  current[path[path.length - 1]] = value
  testPlan.updateNode(node.value.id, obj)
}

// Narrowed node refs — typed to each sub-editor's prop contract
const planNode = computed(() => {
  const n = node.value
  if (!n) return null
  const t = n.type
  return t === 'TestPlan' || t === 'ThreadGroup' ? (n as TestPlan | ThreadGroup) : null
})

const samplerNode = computed(() => {
  const n = node.value
  if (!n) return null
  return String(n.type).endsWith('Sampler') ? (n as SamplerNode) : null
})

const elementNode = computed(() => {
  const n = node.value
  if (!n) return null
  const t = String(n.type)
  if (t === 'TestPlan' || t === 'ThreadGroup') return null
  if (t.endsWith('Sampler')) return null
  if (
    t.startsWith('View') ||
    t.startsWith('Summary') ||
    t.startsWith('Aggregate') ||
    t.startsWith('ResponseTime') ||
    t.startsWith('GraphResults')
  )
    return null
  return n as ElementNode
})
</script>

<template>
  <div class="props-content">
    <!-- No selection -->
    <div v-if="!node" class="empty-props">
      <p>Select an element from the tree</p>
    </div>

    <div v-else class="prop-editor">
      <!-- Common properties -->
      <div class="prop-section">
        <div class="prop-row">
          <label class="pp-label">Name</label>
          <input type="text" :value="node.name" @input="update('name', ($event.target as HTMLInputElement).value)" />
        </div>
        <div class="prop-row">
          <label class="pp-label">Type</label>
          <span class="prop-value">{{ node.type }}</span>
        </div>
        <div class="prop-row">
          <label class="pp-label">Enabled</label>
          <input
            type="checkbox"
            :checked="!!node.enabled"
            @change="update('enabled', ($event.target as HTMLInputElement).checked)"
          />
        </div>
      </div>

      <!-- Type-specific editors — delegated to focused child components -->
      <div class="prop-section">
        <PlanEditor v-if="planNode" :node="planNode" @update="(k: string, v: unknown) => update(k, v)" />
        <SamplerEditors
          v-if="samplerNode"
          :node="samplerNode"
          @update="(k: string, v: unknown) => update(k, v)"
          @update-nested="(path: string[], v: unknown) => updateNested(path, v)"
        />
        <ElementEditors v-if="elementNode" :node="elementNode" @update="(k: string, v: unknown) => update(k, v)" />
      </div>

      <!-- Delete button -->
      <div v-if="node.type !== 'TestPlan'" class="prop-section">
        <button class="danger-btn" @click="testPlan.removeNode(node.id as string)">Delete Element</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.props-content {
  flex: 1;
  overflow-y: auto;
  padding: 10px 12px;
}

.prop-editor {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.prop-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-title {
  font-size: 10px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding-bottom: 4px;
  border-bottom: 1px solid var(--border);
}

.prop-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.prop-row.col {
  flex-direction: column;
  align-items: stretch;
}

.pp-label {
  width: 80px;
  font-size: 11px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.prop-row.col .pp-label {
  width: auto;
}

.pp-field {
  flex: 1;
  padding: 5px 8px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-base);
  color: var(--text-primary);
  font-size: 12px;
  transition: border-color 0.12s;
}

.pp-textarea {
  flex: 1;
  padding: 6px 8px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-base);
  color: var(--text-primary);
  font-size: 12px;
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  resize: vertical;
  transition: border-color 0.12s;
}

.pp-field:focus,
.pp-textarea:focus {
  border-color: var(--accent);
  outline: none;
}

.pp-checkbox {
  width: 16px;
  height: 16px;
  accent-color: var(--accent);
  cursor: pointer;
}

.prop-value {
  color: var(--text-muted);
  font-size: 12px;
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.prop-hint {
  font-size: 10px;
  color: var(--text-muted);
}

.kv-row {
  display: flex;
  gap: 4px;
  margin-bottom: 4px;
}

.kv-key,
.kv-value {
  flex: 1;
  padding: 5px 6px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-base);
  color: var(--text-primary);
  font-size: 12px;
  transition: border-color 0.12s;
}

.kv-full {
  flex: 1;
  padding: 5px 6px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-base);
  color: var(--text-primary);
  font-size: 12px;
  transition: border-color 0.12s;
}

.fd-type {
  width: 70px;
  padding: 5px 4px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-base);
  color: var(--text-primary);
  font-size: 11px;
  transition: border-color 0.12s;
}

.kv-key:focus,
.kv-value:focus,
.kv-full:focus,
.fd-type:focus {
  border-color: var(--accent);
  outline: none;
}

.kv-remove {
  padding: 2px 6px;
  border: none;
  background: transparent;
  color: var(--danger);
  cursor: pointer;
  font-size: 14px;
  transition: opacity 0.12s;
  opacity: 0.6;
}
.kv-remove:hover {
  opacity: 1;
}

.kv-add {
  padding: 3px 10px;
  border: 1px dashed var(--border);
  border-radius: 3px;
  background: transparent;
  color: var(--text-muted);
  font-size: 11px;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.12s;
}
.kv-add:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.danger-btn {
  padding: 6px 16px;
  border: 1px solid var(--danger);
  border-radius: 3px;
  background: transparent;
  color: var(--danger);
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  width: 100%;
  transition: all 0.12s;
}
.danger-btn:hover {
  background: var(--danger);
  color: #fff;
}

.empty-props {
  padding: 30px 20px;
  text-align: center;
  color: var(--text-muted);
}
</style>
