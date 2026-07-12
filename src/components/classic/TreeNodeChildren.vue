<template>
  <template v-for="child in children" :key="child.id">
    <div
      :class="['tree-node', { selected: selectedId === child.id }]"
      :style="{ paddingLeft: (depth * 20 + 12) + 'px' }"
      @click="emit('select', child)"
      @contextmenu.prevent.stop="emit('select', child); emit('contextmenu', $event, child)"
    >
      <span
        v-if="hasKids(child)"
        :class="['arrow', { expanded: isExpanded(child.id) }]"
        @click.stop="emit('toggle', child.id)"
      >▶</span>
      <span v-else class="arrow-spacer"></span>
      <span class="node-icon">{{ getIcon(child.type) }}</span>
      <span class="node-label">{{ child.name }}</span>
      <span class="node-type">{{ child.type }}</span>
    </div>
    <div v-if="hasKids(child) && isExpanded(child.id)" class="tree-children">
      <TreeNodeChildren
        :children="getKids(child)"
        :depth="depth + 1"
        :expanded-ids="expandedIds"
        :selected-id="selectedId"
        @select="(n: TestElementUnion) => emit('select', n)"
        @contextmenu="(e, n) => emit('contextmenu', e, n)"
        @toggle="(id: string) => emit('toggle', id)"
      />
    </div>
  </template>
</template>

<script setup lang="ts">
import type { TestElementUnion, ChildElement } from '@/types'

const p = defineProps<{
  children: ChildElement[]
  depth: number
  expandedIds: Set<string>
  selectedId: string | null
}>()

const emit = defineEmits<{
  select: [node: TestElementUnion]
  contextmenu: [event: MouseEvent, node: TestElementUnion]
  toggle: [id: string]
}>()

function getIcon(type: string): string {
  const icons: Record<string, string> = {
    HttpSampler: '▸', LoopController: '↻', IfController: '?',
    WhileController: '⧖', TransactionController: '▧', ThroughputController: '⇄',
    ResponseAssertion: '✓', JsonAssertion: '◊', DurationAssertion: '⏱',
    ConstantTimer: '⏲', UniformRandomTimer: '⚃', GaussianRandomTimer: '≈',
    RegexExtractor: 'R', JsonExtractor: 'J', BoundaryExtractor: '⊂',
    HttpDefaults: '⚙', CsvDataSet: '≡', UserVariables: '$',
    UserParameters: '☺', ViewResultsTree: '≡', SummaryReport: 'Σ',
    AggregateReport: 'Σ',
  }
  return icons[type] || '●'
}

function hasKids(node: ChildElement): boolean {
  return 'children' in node && Array.isArray((node as unknown as Record<string, unknown>).children)
}

function getKids(node: ChildElement): ChildElement[] {
  return ((node as unknown as Record<string, unknown>).children as ChildElement[]) ?? []
}

function isExpanded(id: string): boolean {
  return p.expandedIds.has(id)
}
</script>

<style scoped>
.tree-node {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  cursor: pointer;
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tree-node:hover {
  background: var(--bg-hover);
}

.tree-node.selected {
  background: var(--accent);
  color: var(--bg-primary);
}

.arrow {
  font-size: 9px;
  width: 12px;
  flex-shrink: 0;
  color: var(--text-muted);
  transition: transform 0.15s;
  cursor: pointer;
}

.arrow.expanded {
  transform: rotate(90deg);
}

.arrow-spacer {
  width: 12px;
  flex-shrink: 0;
}

.node-icon {
  font-size: 14px;
  width: 18px;
  text-align: center;
  flex-shrink: 0;
}

.node-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
}

.node-type {
  font-size: 10px;
  color: var(--text-muted);
  flex-shrink: 0;
  opacity: 0.7;
}

.tree-children {
  /* children container */
}
</style>
