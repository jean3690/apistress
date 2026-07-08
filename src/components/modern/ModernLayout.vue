<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useTestPlanStore } from '@/stores'
import { collectAllSamplers } from '@/utils/tree-utils'
import { createDefaultHttpSampler } from '@/types'
import type { ChildElement } from '@/types'
import RequestList from './RequestList.vue'
import RequestEditor from './RequestEditor.vue'
import ResponsePanel from './ResponsePanel.vue'

const testPlan = useTestPlanStore()

// ---- Shared state ----
const selectedId = ref<string | null>(null)
const responseData = ref<{ status: string; body: string; headers: string } | null>(null)
const responseLoading = ref(false)

const samplers = computed(() =>
  collectAllSamplers(testPlan.plan).filter(
    (s): s is ChildElement & { type: 'HttpSampler' } => s.type === 'HttpSampler'
  )
)

watch(samplers, (list) => {
  if (!selectedId.value && list.length > 0) {
    selectedId.value = list[0].id
  }
}, { immediate: true })

// ---- Request list events ----
function onSelect(id: string) {
  selectedId.value = id
  testPlan.setSelectedNode(id)
}

function onAdd() {
  const tgs = testPlan.plan.threadGroups
  if (tgs.length === 0) {
    testPlan.addThreadGroup()
    nextTick(() => addSampler())
    return
  }
  addSampler()
}

function addSampler() {
  const tgs = testPlan.plan.threadGroups
  if (tgs.length === 0) return
  const sampler = createDefaultHttpSampler(testPlan.newNodeId())
  sampler.name = `HTTP Request ${samplers.value.length + 1}`
  testPlan.addChild(tgs[0].id, sampler)
  nextTick(() => { selectedId.value = sampler.id })
}

function onDelete(id: string) {
  const idx = samplers.value.findIndex(s => s.id === id)
  const next = samplers.value[idx + 1] || samplers.value[idx - 1]
  testPlan.removeNode(id)
  if (next) {
    nextTick(() => { selectedId.value = next.id })
  } else {
    selectedId.value = null
  }
}

// ---- Editor events ----
function onResponse(data: { status: string; body: string; headers: string } | null) {
  responseData.value = data
}

// Expose loading state to response panel via the editor's own state would be complex.
// Keep it simple: the editor manages its own loading internally.
// We just pass response data through.
</script>

<template>
  <div class="modern-layout">
    <RequestList
      :samplers="samplers"
      :selected-id="selectedId"
      @select="onSelect"
      @delete="onDelete"
      @add="onAdd"
    />
    <RequestEditor
      :sampler-id="selectedId"
      @response="onResponse"
      @update:loading="(v: boolean) => responseLoading = v"
    />
    <ResponsePanel
      :loading="responseLoading"
      :response="responseData"
    />
  </div>
</template>

<style scoped>
.modern-layout {
  display: flex;
  flex: 1;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

/* Right-side border between panels comes from child components */
.modern-layout > :not(:last-child) {
  border-right: 1px solid var(--border);
}
</style>
