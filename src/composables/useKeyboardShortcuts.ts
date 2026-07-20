import { onMounted, onUnmounted } from 'vue'
import { useTestPlanStore, useExecutionStore } from '@/stores'

export function useKeyboardShortcuts() {
  const testPlan = useTestPlanStore()
  const execution = useExecutionStore()

  function handleKeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement
    // Don't fire shortcuts when typing in inputs
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT') return
    if (target.isContentEditable) return

    const mod = e.ctrlKey || e.metaKey

    // Ctrl+N: New test plan
    if (mod && e.key === 'n') {
      e.preventDefault()
      window.dispatchEvent(new CustomEvent('file:new'))
      return
    }

    // Ctrl+O: Open test plan
    if (mod && e.key === 'o') {
      e.preventDefault()
      window.dispatchEvent(new CustomEvent('file:open'))
      return
    }

    // Ctrl+S: Save
    if (mod && e.key === 's' && !e.shiftKey) {
      e.preventDefault()
      window.dispatchEvent(new CustomEvent('app:save'))
      return
    }

    // Ctrl+Shift+S: Save As
    if (mod && e.key === 's' && e.shiftKey) {
      e.preventDefault()
      window.dispatchEvent(new CustomEvent('app:saveas'))
      return
    }

    // Delete: remove selected node
    if (e.key === 'Delete' && testPlan.selectedNodeId) {
      const node = testPlan.selectedNode
      if (node && node.type !== 'TestPlan') {
        testPlan.removeNode(testPlan.selectedNodeId)
      }
      return
    }

    // F5 (no Ctrl): run test
    if (e.key === 'F5') {
      e.preventDefault()
      if (!execution.isRunning) {
        execution.startTest(testPlan.toJSON())
      }
      return
    }

    // Ctrl+R: run test
    if (mod && e.key === 'r') {
      e.preventDefault()
      if (!execution.isRunning) {
        execution.startTest(testPlan.toJSON())
      }
      return
    }

    // Esc: stop test
    if (e.key === 'Escape' && execution.isRunning) {
      execution.stopTest()
      return
    }
  }

  onMounted(() => window.addEventListener('keydown', handleKeydown))
  onUnmounted(() => window.removeEventListener('keydown', handleKeydown))
}
