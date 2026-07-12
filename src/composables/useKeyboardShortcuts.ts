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

    // Ctrl+S: trigger save via custom event (ToolBar listens)
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault()
      window.dispatchEvent(new CustomEvent('app:save'))
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

    // Ctrl+R / F5: run test
    if ((e.ctrlKey && e.key === 'r') || e.key === 'F5') {
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
