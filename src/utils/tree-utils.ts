import type { TestPlan, ThreadGroup, TestElementUnion, ChildElement } from '@/types'

/**
 * Walk the entire test plan tree and call fn for every node.
 */
export function walkTree(plan: TestPlan, fn: (node: TestElementUnion, parent: TestElementUnion | null) => void) {
  fn(plan, null)
  for (const tg of plan.threadGroups) {
    fn(tg, plan)
    walkChildren(tg.children, tg, fn)
  }
}

function walkChildren(
  children: ChildElement[],
  parent: TestElementUnion,
  fn: (node: TestElementUnion, parent: TestElementUnion | null) => void,
) {
  for (const child of children) {
    fn(child, parent)
    if ('children' in child) {
      walkChildren((child as unknown as { children: ChildElement[] }).children, child, fn)
    }
  }
}

/**
 * Find a node by id anywhere in the plan.
 */
export function findNodeById(plan: TestPlan, id: string): TestElementUnion | null {
  if (plan.id === id) return plan

  for (const tg of plan.threadGroups) {
    if (tg.id === id) return tg
    const found = findInChildren(tg.children, id)
    if (found) return found
  }
  return null
}

function findInChildren(children: ChildElement[], id: string): TestElementUnion | null {
  for (const child of children) {
    if (child.id === id) return child
    if ('children' in child) {
      const found = findInChildren((child as unknown as { children: ChildElement[] }).children, id)
      if (found) return found
    }
  }
  return null
}

/**
 * Remove a node by id. Returns true if removed.
 */
export function removeNodeById(plan: TestPlan, id: string): boolean {
  for (let i = 0; i < plan.threadGroups.length; i++) {
    if (plan.threadGroups[i].id === id) {
      plan.threadGroups.splice(i, 1)
      return true
    }
    if (removeFromChildren(plan.threadGroups[i].children, id)) return true
  }
  return false
}

function removeFromChildren(children: ChildElement[], id: string): boolean {
  for (let i = 0; i < children.length; i++) {
    if (children[i].id === id) {
      children.splice(i, 1)
      return true
    }
    if ('children' in children[i]) {
      const nested = (children[i] as unknown as { children: ChildElement[] }).children
      if (nested && removeFromChildren(nested, id)) return true
    }
  }
  return false
}

/**
 * Add a child element to a parent node that has children. Returns true if added.
 */
export function addChildToParent(plan: TestPlan, parentId: string, element: ChildElement): boolean {
  for (const tg of plan.threadGroups) {
    if (tg.id === parentId) {
      tg.children.push(element)
      return true
    }
    if (addToContainer(tg.children, parentId, element)) return true
  }
  return false
}

function addToContainer(children: ChildElement[], parentId: string, element: ChildElement): boolean {
  for (const child of children) {
    if (child.id === parentId && 'children' in child) {
      (child as unknown as { children: ChildElement[] }).children.push(element)
      return true
    }
    if ('children' in child) {
      const nested = (child as unknown as { children: ChildElement[] }).children
      if (nested && addToContainer(nested, parentId, element)) return true
    }
  }
  return false
}

/**
 * Get a flat list of nodes for the tree view.
 */
export interface TreeNodeData {
  id: string
  label: string
  type: string
  enabled: boolean
  children: TreeNodeData[]
}

export function buildTreeData(plan: TestPlan): TreeNodeData {
  return {
    id: plan.id,
    label: plan.name,
    type: 'TestPlan',
    enabled: true,
    children: plan.threadGroups.map(tg => threadGroupToTreeNode(tg)),
  }
}

function threadGroupToTreeNode(tg: ThreadGroup): TreeNodeData {
  return {
    id: tg.id,
    label: tg.name,
    type: 'ThreadGroup',
    enabled: tg.enabled,
    children: tg.children.map(childToTreeNode),
  }
}

export function childToTreeNode(child: ChildElement): TreeNodeData {
  const node: TreeNodeData = {
    id: child.id,
    label: child.name,
    type: child.type,
    enabled: child.enabled,
    children: [],
  }
  if ('children' in child) {
    node.children = (child as unknown as { children: ChildElement[] }).children.map(childToTreeNode)
  }
  return node
}

/**
 * Collect all HTTP samplers from the plan (for modern mode flat list).
 */
export function collectAllSamplers(plan: TestPlan): ChildElement[] {
  const result: ChildElement[] = []
  for (const tg of plan.threadGroups) {
    collectSamplersFromChildren(tg.children, result)
  }
  return result
}

function collectSamplersFromChildren(children: ChildElement[], out: ChildElement[]) {
  for (const child of children) {
    if (child.type === 'HttpSampler') {
      out.push(child)
    }
    if ('children' in child) {
      collectSamplersFromChildren((child as unknown as { children: ChildElement[] }).children, out)
    }
  }
}
