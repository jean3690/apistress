<script setup lang="ts">
import { reactive } from 'vue'
import { useTestPlanStore } from '@/stores'
import type { TestElementUnion } from '@/types'
import TreeNodeChildren from './TreeNodeChildren.vue'

const testPlan = useTestPlanStore()

const emit = defineEmits<{
  contextmenu: [event: MouseEvent, node: TestElementUnion]
}>()

// Keep track of expanded node IDs
const expandedIds = reactive(new Set<string>())

function isExpanded(id: string): boolean {
  return expandedIds.has(id)
}

function toggleExpand(id: string) {
  if (expandedIds.has(id)) {
    expandedIds.delete(id)
  } else {
    expandedIds.add(id)
  }
}

function onNodeClick(node: TestElementUnion) {
  testPlan.setSelectedNode(node.id)
}

function onNodeContextMenu(event: MouseEvent, node: TestElementUnion) {
  event.preventDefault()
  event.stopPropagation()
  testPlan.setSelectedNode(node.id)
  emit('contextmenu', event, node)
}

function onArrowClick(e: MouseEvent, id: string) {
  e.stopPropagation()
  toggleExpand(id)
}
</script>

<template>
  <div class="tree-view">
    <!-- TestPlan Root -->
    <div
      :class="['tree-node root', { selected: testPlan.selectedNodeId === testPlan.plan.id }]"
      @click="onNodeClick(testPlan.plan)"
      @contextmenu="onNodeContextMenu($event, testPlan.plan)"
    >
      <span class="node-icon">▣</span>
      <span class="node-label">{{ testPlan.plan.name || 'Test Plan' }}</span>
    </div>

    <!-- Thread Groups -->
    <div v-for="tg in testPlan.plan.threadGroups" :key="tg.id" class="tree-branch">
      <div
        :class="['tree-node', { selected: testPlan.selectedNodeId === tg.id }]"
        @click="onNodeClick(tg)"
        @contextmenu="onNodeContextMenu($event, tg)"
      >
        <span :class="['arrow', { expanded: isExpanded(tg.id) }]" @click="onArrowClick($event, tg.id)">▶</span>
        <span class="node-icon">▤</span>
        <span class="node-label">{{ tg.name }}</span>
        <span class="node-badge">{{ tg.numThreads }} threads</span>
      </div>
      <div v-if="isExpanded(tg.id)" class="tree-children">
        <TreeNodeChildren
          v-if="tg.children.length > 0"
          :children="tg.children"
          :depth="1"
          :expanded-ids="expandedIds"
          :selected-id="testPlan.selectedNodeId"
          @select="(n: TestElementUnion) => onNodeClick(n)"
          @contextmenu="(e, n) => onNodeContextMenu(e, n)"
          @toggle="(id: string) => toggleExpand(id)"
        />
        <div v-else class="empty-children">Right-click to add elements</div>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="testPlan.plan.threadGroups.length === 0" class="empty-tree">
      <p>No Thread Groups</p>
      <button @click="testPlan.addThreadGroup()">Add Thread Group</button>
    </div>
  </div>
</template>

<style scoped>
.tree-view {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
  user-select: none;
}

.tree-node {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 12px;
  cursor: pointer;
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  border-left: 2px solid transparent;
  transition:
    background 0.1s,
    border-color 0.1s;
}

.tree-node.root {
  font-weight: 600;
  font-size: 13px;
}

.tree-node:hover {
  background: var(--bg-hover);
}

.tree-node.selected {
  background: var(--accent-glow);
  color: var(--text-primary);
  border-left-color: var(--accent);
}

.arrow {
  font-size: 8px;
  width: 12px;
  flex-shrink: 0;
  color: var(--text-muted);
  transition: transform 0.15s;
  cursor: pointer;
}

.arrow.expanded {
  transform: rotate(90deg);
}

.node-icon {
  font-size: 13px;
  width: 16px;
  text-align: center;
  flex-shrink: 0;
  opacity: 0.7;
}

.node-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
}

.node-badge {
  font-size: 9px;
  color: var(--text-muted);
  flex-shrink: 0;
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

.empty-children {
  padding: 4px 10px 4px 36px;
  font-size: 11px;
  color: var(--text-muted);
  font-style: italic;
}

.empty-tree {
  padding: 20px;
  text-align: center;
  color: var(--text-muted);
}

.empty-tree button {
  margin-top: 8px;
  padding: 5px 14px;
  border: 1px solid var(--accent);
  border-radius: 3px;
  background: transparent;
  color: var(--accent);
  font-weight: 500;
  cursor: pointer;
  transition: all 0.12s;
}
.empty-tree button:hover {
  background: var(--accent-glow);
}
</style>
