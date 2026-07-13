<script setup lang="ts">
import { shallowRef, computed, onMounted, useTemplateRef } from 'vue'
import { useTestPlanStore } from '@/stores'
import { usePanelResize } from '@/composables/usePanelResize'
import type { TestElementUnion } from '@/types'
import TreeView from '@/components/classic/TreeView.vue'
import ContextMenu from '@/components/classic/ContextMenu.vue'
import PropertiesPanel from '@/components/classic/PropertiesPanel.vue'
import RequestEditor from '@/components/modern/RequestEditor.vue'
import ResponsePanel from '@/components/modern/ResponsePanel.vue'
import ResultsPanel from '@/components/classic/ResultsPanel.vue'
import LiveDashboard from '@/components/classic/LiveDashboard.vue'
import ExecutionBar from './ExecutionBar.vue'

const testPlan = useTestPlanStore()

// ---- Panel resize ----
const containerRef = useTemplateRef<HTMLElement>('container')
const { MIN_W, sizes, dragging, initFromContainer, panelStyle, togglePanel, onHandleMousedown } = usePanelResize(
  containerRef,
  {
    initial: [260, 500, 360],
  },
)

onMounted(() => initFromContainer())

// ---- Node selection ----
const SAMPLER_TYPES = new Set([
  'HttpSampler',
  'GraphQlSampler',
  'SseSampler',
  'MqttSampler',
  'WebSocketSampler',
  'GrpcSampler',
  'TcpSampler',
  'RedisSampler',
])

const selectedNode = computed(() => testPlan.selectedNode)
const isSampler = computed(() => {
  const node = selectedNode.value
  return node ? SAMPLER_TYPES.has(node.type) : false
})

// ---- Request / Response ----
const responseData = shallowRef<{
  status: string
  statusCode?: number
  body: string
  headers: string
  time?: number
  size?: number
} | null>(null)
const responseLoading = shallowRef(false)
const rightTab = shallowRef<'response' | 'results' | 'charts'>('response')

function onResponse(data: typeof responseData.value) {
  responseData.value = data
}

// ---- Context menu ----
const contextMenu = shallowRef<{ node: TestElementUnion; x: number; y: number } | { node: null }>({ node: null })

function onTreeContextMenu(event: MouseEvent, node: TestElementUnion) {
  contextMenu.value = { node, x: event.clientX, y: event.clientY }
}
function closeContextMenu() {
  contextMenu.value = { node: null }
}
</script>

<template>
  <div class="flex flex-col flex-1 w-full h-full overflow-hidden bg-background">
    <div ref="containerRef" class="flex flex-1 min-h-0 overflow-hidden">
      <!-- Panel 0: Tree -->
      <div class="flex flex-col overflow-hidden h-full border-r border-border bg-[#181825]" :style="panelStyle(0)">
        <div v-if="sizes[0] <= MIN_W" class="flex flex-col items-center h-full py-2 px-1">
          <button class="collapse-btn" @click="togglePanel(0)">▶</button>
          <span class="collapse-label">Tree</span>
        </div>
        <template v-else>
          <div class="panel-header">
            <span>Test Plan</span>
            <div class="flex gap-1 items-center">
              <button class="action-btn" @click="testPlan.addThreadGroup()" title="Add Thread Group">+TG</button>
              <button class="collapse-btn" @click="togglePanel(0)">◀</button>
            </div>
          </div>
          <TreeView @contextmenu="onTreeContextMenu" />
        </template>
      </div>

      <div :class="['resize-handle', { active: dragging === 0 }]" @mousedown="onHandleMousedown(0, $event)" />

      <!-- Panel 1: Editor / Properties -->
      <div class="flex flex-col overflow-hidden h-full border-r border-border bg-[#1e1e2e]" :style="panelStyle(1)">
        <div v-if="sizes[1] <= MIN_W" class="flex flex-col items-center h-full py-2 px-1">
          <button class="collapse-btn" @click="togglePanel(1)">◀</button>
          <span class="collapse-label">Editor</span>
        </div>
        <template v-else>
          <RequestEditor
            v-if="isSampler"
            :key="selectedNode?.id"
            :sampler-id="selectedNode?.id ?? null"
            @response="onResponse"
            @update:loading="(v: boolean) => (responseLoading = v)"
          />
          <template v-else>
            <div class="panel-header">
              <span v-if="selectedNode">Properties</span>
              <span v-else class="text-muted-foreground">Select a node</span>
              <button class="collapse-btn ml-auto" @click="togglePanel(1)">▶</button>
            </div>
            <PropertiesPanel v-if="selectedNode" />
            <div v-else class="flex-1 flex items-center justify-center text-muted-foreground text-sm">
              Select an element from the tree to edit its properties
            </div>
          </template>
        </template>
      </div>

      <div :class="['resize-handle', { active: dragging === 1 }]" @mousedown="onHandleMousedown(1, $event)" />

      <!-- Panel 2: Output -->
      <div class="flex flex-col overflow-hidden h-full bg-[#1e1e2e]" :style="panelStyle(2)">
        <div v-if="sizes[2] <= MIN_W" class="flex flex-col items-center h-full py-2 px-1">
          <span class="collapse-label">Output</span>
          <button class="collapse-btn" @click="togglePanel(2)">◀</button>
        </div>
        <template v-else>
          <div class="panel-header">
            <span>Output</span>
            <div class="flex gap-0.5 ml-auto">
              <span :class="['tab', { active: rightTab === 'response' }]" @click="rightTab = 'response'">Response</span>
              <span :class="['tab', { active: rightTab === 'results' }]" @click="rightTab = 'results'">Results</span>
              <span :class="['tab', { active: rightTab === 'charts' }]" @click="rightTab = 'charts'">Charts</span>
            </div>
            <button class="collapse-btn ml-2" @click="togglePanel(2)">▶</button>
          </div>

          <ResponsePanel
            v-if="rightTab === 'response'"
            :loading="responseLoading"
            :response="responseData"
            class="flex-1 overflow-hidden"
          />
          <ResultsPanel v-else-if="rightTab === 'results'" class="flex-1 overflow-y-auto" />
          <LiveDashboard v-else class="flex-1 overflow-hidden" />
        </template>
      </div>
    </div>

    <ExecutionBar />

    <ContextMenu
      v-if="contextMenu.node"
      :node="contextMenu.node"
      :x="contextMenu.x"
      :y="contextMenu.y"
      @close="closeContextMenu"
    />
  </div>
</template>

<style scoped>
.panel-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.7px;
  color: var(--secondary);
  flex-shrink: 0;
}
.collapse-btn {
  width: 20px;
  height: 20px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--surface);
  color: var(--muted-foreground);
  font-size: 9px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.12s;
}
.collapse-btn:hover {
  background: var(--muted);
  color: var(--foreground);
  border-color: var(--muted-foreground);
}
.action-btn {
  width: 20px;
  height: 20px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: transparent;
  color: var(--foreground);
  font-size: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
  transition: all 0.12s;
}
.action-btn:hover {
  background: var(--muted);
  color: var(--foreground);
  border-color: var(--muted-foreground);
}
.collapse-label {
  writing-mode: vertical-rl;
  text-orientation: mixed;
  font-size: 9px;
  color: var(--muted-foreground);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-top: 8px;
}
.resize-handle {
  width: 2px;
  cursor: col-resize;
  flex-shrink: 0;
  background: var(--border);
  position: relative;
  z-index: 10;
  transition: background 0.15s;
}
.resize-handle:hover,
.resize-handle.active {
  background: var(--primary);
}
.resize-handle::after {
  content: '';
  position: absolute;
  left: -4px;
  right: -4px;
  top: 0;
  bottom: 0;
}
.tab {
  padding: 4px 10px;
  font-size: 10px;
  color: var(--muted-foreground);
  cursor: pointer;
  border-radius: 3px;
  transition: all 0.15s;
  font-weight: 500;
}
.tab:hover {
  color: var(--foreground);
}
.tab.active {
  background: var(--primary);
  color: var(--primary-foreground);
  font-weight: 600;
}
</style>
