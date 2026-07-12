<template>
  <div v-if="node" class="context-menu" :style="menuStyle()">
    <div class="menu-header">
      <span class="menu-header-type">{{ node.type }}</span>
      <span class="menu-header-name">{{ node.name }}</span>
    </div>

    <div class="menu-divider"></div>

    <template v-if="canHaveChildren()">
      <div class="menu-item has-sub" @mouseenter="addSamplerOpen = true" @mouseleave="addSamplerOpen = false">
        <span class="menu-label">Add Sampler</span>
        <span class="menu-arrow">&#9654;</span>
        <div v-show="addSamplerOpen" class="submenu">
          <div class="menu-item" @click="addHttpSampler()">HTTP Request</div>
        </div>
      </div>

      <div class="menu-item has-sub" @mouseenter="addControllerOpen = true" @mouseleave="addControllerOpen = false">
        <span class="menu-label">Add Controller</span>
        <span class="menu-arrow">&#9654;</span>
        <div v-show="addControllerOpen" class="submenu">
          <div class="menu-item" @click="addLoopController()">Loop Controller</div>
          <div class="menu-item" @click="addIfController()">If Controller</div>
          <div class="menu-item" @click="addWhileController()">While Controller</div>
          <div class="menu-item" @click="addTransactionController()">Transaction Controller</div>
          <div class="menu-item" @click="addThroughputController()">Throughput Controller</div>
        </div>
      </div>

      <div class="menu-item has-sub" @mouseenter="addAssertionOpen = true" @mouseleave="addAssertionOpen = false">
        <span class="menu-label">Add Assertion</span>
        <span class="menu-arrow">&#9654;</span>
        <div v-show="addAssertionOpen" class="submenu">
          <div class="menu-item" @click="addResponseAssertion()">Response Assertion</div>
          <div class="menu-item" @click="addJsonAssertion()">JSON Assertion</div>
          <div class="menu-item" @click="addDurationAssertion()">Duration Assertion</div>
        </div>
      </div>

      <div class="menu-item has-sub" @mouseenter="addTimerOpen = true" @mouseleave="addTimerOpen = false">
        <span class="menu-label">Add Timer</span>
        <span class="menu-arrow">&#9654;</span>
        <div v-show="addTimerOpen" class="submenu">
          <div class="menu-item" @click="addConstantTimer()">Constant Timer</div>
          <div class="menu-item" @click="addUniformRandomTimer()">Uniform Random Timer</div>
          <div class="menu-item" @click="addGaussianRandomTimer()">Gaussian Random Timer</div>
        </div>
      </div>

      <div class="menu-item has-sub" @mouseenter="addExtractorOpen = true" @mouseleave="addExtractorOpen = false">
        <span class="menu-label">Add Extractor</span>
        <span class="menu-arrow">&#9654;</span>
        <div v-show="addExtractorOpen" class="submenu">
          <div class="menu-item" @click="addRegexExtractor()">Regex Extractor</div>
          <div class="menu-item" @click="addJsonExtractor()">JSON Extractor</div>
          <div class="menu-item" @click="addBoundaryExtractor()">Boundary Extractor</div>
        </div>
      </div>

      <div class="menu-item has-sub" @mouseenter="addConfigOpen = true" @mouseleave="addConfigOpen = false">
        <span class="menu-label">Add Config</span>
        <span class="menu-arrow">&#9654;</span>
        <div v-show="addConfigOpen" class="submenu">
          <div class="menu-item" @click="addHttpDefaults()">HTTP Request Defaults</div>
          <div class="menu-item" @click="addCsvDataSet()">CSV Data Set Config</div>
          <div class="menu-item" @click="addUserVariables()">User Defined Variables</div>
          <div class="menu-item" @click="addUserParameters()">User Parameters</div>
        </div>
      </div>

      <div class="menu-divider"></div>
    </template>

    <div v-if="canDelete()" class="menu-item" @click="removeNode()">
      <span class="menu-label">Delete</span>
      <span class="menu-shortcut">Del</span>
    </div>
    <div v-if="node.type !== 'TestPlan'" class="menu-item" @click="duplicateNode()">
      <span class="menu-label">Duplicate</span>
      <span class="menu-shortcut">Ctrl+D</span>
    </div>
    <div v-if="node.type !== 'TestPlan'" class="menu-item" @click="toggleEnabled()">
      <span class="menu-label">{{ node.enabled ? 'Disable' : 'Enable' }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useTestPlanStore } from '@/stores'
import type { TestElementUnion, ChildElement } from '@/types'
import {
  createDefaultHttpSampler,
  createDefaultLoopController,
  createDefaultIfController,
  createDefaultWhileController,
  createDefaultTransactionController,
  createDefaultThroughputController,
  createDefaultConstantTimer,
  createDefaultUniformRandomTimer,
  createDefaultGaussianRandomTimer,
  createDefaultResponseAssertion,
  createDefaultJsonAssertion,
  createDefaultDurationAssertion,
  createDefaultRegexExtractor,
  createDefaultJsonExtractor,
  createDefaultBoundaryExtractor,
  createDefaultHttpDefaults,
  createDefaultCsvDataSet,
  createDefaultUserVariables,
  createDefaultUserParameters,
} from '@/types'

const props = defineProps<{
  node: TestElementUnion | null
  x: number
  y: number
}>()

const emit = defineEmits<{
  close: []
}>()

const testPlan = useTestPlanStore()

const addSamplerOpen = ref(false)
const addControllerOpen = ref(false)
const addAssertionOpen = ref(false)
const addTimerOpen = ref(false)
const addExtractorOpen = ref(false)
const addConfigOpen = ref(false)

function close() {
  emit('close')
}

function addElement(element: ChildElement) {
  if (props.node) {
    testPlan.addChild(props.node.id, element)
  }
  close()
}

function newElement(element: ChildElement): ChildElement {
  ;(element as unknown as Record<string, unknown>).id = testPlan.newNodeId()
  return element
}

function addHttpSampler() {
  addElement(newElement(createDefaultHttpSampler('')) as unknown as ChildElement)
}
function addLoopController() {
  addElement(newElement(createDefaultLoopController('')) as unknown as ChildElement)
}
function addIfController() {
  addElement(newElement(createDefaultIfController('')) as unknown as ChildElement)
}
function addWhileController() {
  addElement(newElement(createDefaultWhileController('')) as unknown as ChildElement)
}
function addTransactionController() {
  addElement(newElement(createDefaultTransactionController('')) as unknown as ChildElement)
}
function addThroughputController() {
  addElement(newElement(createDefaultThroughputController('')) as unknown as ChildElement)
}
function addResponseAssertion() {
  addElement(newElement(createDefaultResponseAssertion('')) as unknown as ChildElement)
}
function addJsonAssertion() {
  addElement(newElement(createDefaultJsonAssertion('')) as unknown as ChildElement)
}
function addDurationAssertion() {
  addElement(newElement(createDefaultDurationAssertion('')) as unknown as ChildElement)
}
function addConstantTimer() {
  addElement(newElement(createDefaultConstantTimer('')) as unknown as ChildElement)
}
function addUniformRandomTimer() {
  addElement(newElement(createDefaultUniformRandomTimer('')) as unknown as ChildElement)
}
function addGaussianRandomTimer() {
  addElement(newElement(createDefaultGaussianRandomTimer('')) as unknown as ChildElement)
}
function addRegexExtractor() {
  addElement(newElement(createDefaultRegexExtractor('')) as unknown as ChildElement)
}
function addJsonExtractor() {
  addElement(newElement(createDefaultJsonExtractor('')) as unknown as ChildElement)
}
function addBoundaryExtractor() {
  addElement(newElement(createDefaultBoundaryExtractor('')) as unknown as ChildElement)
}
function addHttpDefaults() {
  addElement(newElement(createDefaultHttpDefaults('')) as unknown as ChildElement)
}
function addCsvDataSet() {
  addElement(newElement(createDefaultCsvDataSet('')) as unknown as ChildElement)
}
function addUserVariables() {
  addElement(newElement(createDefaultUserVariables('')) as unknown as ChildElement)
}
function addUserParameters() {
  addElement(newElement(createDefaultUserParameters('')) as unknown as ChildElement)
}

function removeNode() {
  if (props.node) {
    testPlan.removeNode(props.node.id)
  }
  close()
}

function duplicateNode() {
  if (props.node) {
    testPlan.duplicateNode(props.node.id)
  }
  close()
}

function toggleEnabled() {
  if (props.node) {
    testPlan.updateNode(props.node.id, { enabled: !props.node.enabled })
  }
  close()
}

function canHaveChildren(): boolean {
  if (!props.node) return false
  const t = props.node.type
  return t === 'TestPlan' || t === 'ThreadGroup' ||
    t === 'LoopController' || t === 'IfController' ||
    t === 'WhileController' || t === 'TransactionController' ||
    t === 'ThroughputController'
}

function canDelete(): boolean {
  if (!props.node) return false
  return props.node.type !== 'TestPlan'
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') close()
}

function onMousedown(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('.context-menu')) {
    close()
  }
}

onMounted(() => {
  document.addEventListener('keydown', onKeydown)
  setTimeout(() => {
    document.addEventListener('mousedown', onMousedown)
  }, 50)
})

onUnmounted(() => {
  document.removeEventListener('keydown', onKeydown)
  document.removeEventListener('mousedown', onMousedown)
})

function menuStyle(): Record<string, string> {
  return {
    left: props.x + 'px',
    top: props.y + 'px',
  }
}
</script>

<style scoped>
.context-menu {
  position: fixed;
  z-index: 1000;
  min-width: 200px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  padding: 4px 0;
  font-size: 12px;
  user-select: none;
}

.menu-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
}

.menu-header-type {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 3px;
  background: var(--bg-hover);
  color: var(--text-secondary);
  text-transform: uppercase;
}

.menu-header-name {
  font-weight: 600;
  color: var(--text-primary);
}

.menu-divider {
  height: 1px;
  background: var(--border);
  margin: 4px 0;
}

.menu-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  cursor: pointer;
  position: relative;
  white-space: nowrap;
}

.menu-item:hover {
  background: var(--accent);
  color: var(--bg-primary);
}

.menu-arrow {
  font-size: 9px;
  color: var(--text-muted);
  margin-left: 16px;
}

.menu-item:hover .menu-arrow {
  color: var(--bg-primary);
}

.menu-shortcut {
  font-size: 10px;
  color: var(--text-muted);
  margin-left: 24px;
}

.menu-item:hover .menu-shortcut {
  color: rgba(0, 0, 0, 0.5);
}

.submenu {
  position: absolute;
  left: 100%;
  top: -4px;
  min-width: 180px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 6px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  padding: 4px 0;
}
</style>
