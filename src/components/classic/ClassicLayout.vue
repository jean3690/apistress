<template>
  <div class="classic-layout">
    <div class="panel tree-panel">
      <div class="panel-header">
        <span>Test Plan</span>
        <div class="header-actions">
          <button class="action-btn" @click="testPlan.addThreadGroup()" title="Add Thread Group">+TG</button>
        </div>
      </div>
      <TreeView @contextmenu="onTreeContextMenu" />
    </div>

    <div class="panel props-panel">
      <div class="panel-header">
        <span>Properties</span>
      </div>
      <PropertiesPanel />
    </div>

    <ResultsPanel />

    <ContextMenu
      v-if="contextMenu.node"
      :node="contextMenu.node"
      :x="contextMenu.x"
      :y="contextMenu.y"
      @close="closeContextMenu"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useTestPlanStore } from '@/stores'
import TreeView from './TreeView.vue'
import PropertiesPanel from './PropertiesPanel.vue'
import ResultsPanel from './ResultsPanel.vue'
import ContextMenu from './ContextMenu.vue'
import type { TestElementUnion } from '@/types'

const testPlan = useTestPlanStore()

const contextMenu = ref<{
  node: TestElementUnion | null
  x: number
  y: number
}>({ node: null, x: 0, y: 0 })

function onTreeContextMenu(event: MouseEvent, node: TestElementUnion) {
  contextMenu.value = {
    node,
    x: Math.min(event.clientX, window.innerWidth - 220),
    y: Math.min(event.clientY, window.innerHeight - 300),
  }
}

function closeContextMenu() {
  contextMenu.value.node = null
}
</script>

<style scoped>
.classic-layout {
  display: flex;
  flex: 1;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.panel {
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
  overflow: hidden;
}

.panel:last-child { border-right: none; }

.tree-panel {
  width: 250px;
  min-width: 180px;
  background: var(--bg-secondary);
}

.props-panel {
  width: 300px;
  min-width: 200px;
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
  user-select: none;
}

.header-actions { display: flex; gap: 4px; }

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
</style>
