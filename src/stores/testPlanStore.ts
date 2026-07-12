import { defineStore } from 'pinia'
import { shallowRef, ref, computed } from 'vue'
import {
  type TestPlan,
  type ThreadGroup,
  type TestElementUnion,
  type ChildElement,
  createDefaultTestPlan,
  createDefaultThreadGroup,
} from '@/types'
import { findNodeById, removeNodeById, addChildToParent } from '@/utils/tree-utils'

export const useTestPlanStore = defineStore('testPlan', () => {
  const plan = ref<TestPlan>(createDefaultTestPlan())
  const selectedNodeId = shallowRef<string | null>(null)
  const dirty = shallowRef(false)

  const allThreadGroups = computed(() => plan.value.threadGroups)

  const selectedNode = computed(() => {
    if (!selectedNodeId.value) return null
    return findNodeById(plan.value, selectedNodeId.value)
  })

  function newNodeId(): string {
    return crypto.randomUUID()
  }

  function setSelectedNode(id: string | null) {
    selectedNodeId.value = id
  }

  function newPlan() {
    plan.value = createDefaultTestPlan()
    selectedNodeId.value = null
    dirty.value = false
  }

  function loadPlan(json: string) {
    plan.value = JSON.parse(json)
    selectedNodeId.value = null
    dirty.value = false
  }

  function toJSON(): string {
    return JSON.stringify(plan.value, null, 2)
  }

  function addThreadGroup() {
    const tg = createDefaultThreadGroup(newNodeId())
    tg.name = `Thread Group ${plan.value.threadGroups.length + 1}`
    plan.value.threadGroups.push(tg)
    dirty.value = true
  }

  function removeThreadGroup(id: string) {
    plan.value.threadGroups = plan.value.threadGroups.filter(tg => tg.id !== id)
    if (selectedNodeId.value === id) selectedNodeId.value = null
    dirty.value = true
  }

  function addChild(parentId: string, element: ChildElement) {
    // Check thread groups
    for (const tg of plan.value.threadGroups) {
      if (tg.id === parentId) {
        tg.children.push(element)
        dirty.value = true
        return
      }
    }
    // Use tree utility for nested controllers
    const added = addChildToParent(plan.value, parentId, element)
    if (added) dirty.value = true
  }

  function removeNode(id: string) {
    if (removeNodeById(plan.value, id)) {
      if (selectedNodeId.value === id) selectedNodeId.value = null
      dirty.value = true
    }
  }

  function updateNode(id: string, patch: Partial<TestElementUnion>) {
    const found = findNodeById(plan.value, id)
    if (found) {
      Object.assign(found, patch)
      dirty.value = true
    }
  }

  function moveNode(id: string, newParentId: string, index: number) {
    // Find the node first
    const node = findNodeById(plan.value, id)
    if (!node) return

    // Clone to avoid reference issues
    const clone = JSON.parse(JSON.stringify(node)) as ChildElement

    // Remove from old location
    removeNodeById(plan.value, id)

    // Add to new parent
    const newParent = findNodeById(plan.value, newParentId)
    if (newParent && 'children' in newParent) {
      const children = (newParent as ThreadGroup | { children: ChildElement[] }).children
      if (index >= 0 && index < children.length) {
        children.splice(index, 0, clone)
      } else {
        children.push(clone)
      }
      dirty.value = true
    }
  }

  function duplicateNode(id: string) {
    const node = findNodeById(plan.value, id)
    if (!node) return
    const clone = JSON.parse(JSON.stringify(node)) as ChildElement
    clone.id = newNodeId()
    clone.name = `${clone.name} (copy)`
    // Try to add to the parent of the original node
    for (const tg of plan.value.threadGroups) {
      const idx = tg.children.findIndex(c => c.id === id)
      if (idx >= 0) {
        tg.children.splice(idx + 1, 0, clone)
        dirty.value = true
        return
      }
      // Check deeper
      if (addToParentContainer(tg.children, id, clone)) {
        dirty.value = true
        return
      }
    }
  }

  return {
    plan,
    selectedNodeId,
    dirty,
    allThreadGroups,
    selectedNode,
    newNodeId,
    setSelectedNode,
    newPlan,
    loadPlan,
    toJSON,
    addThreadGroup,
    removeThreadGroup,
    addChild,
    removeNode,
    updateNode,
    moveNode,
    duplicateNode,
  }
})

/** Helper: find container of a node and add clone after it */
function addToParentContainer(children: ChildElement[], targetId: string, clone: ChildElement): boolean {
  for (let i = 0; i < children.length; i++) {
    if (children[i].id === targetId) {
      children.splice(i + 1, 0, clone)
      return true
    }
    if ('children' in children[i]) {
      const nested = (children[i] as unknown as { children: ChildElement[] }).children
      if (nested && addToParentContainer(nested, targetId, clone)) {
        return true
      }
    }
  }
  return false
}
