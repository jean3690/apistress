import { defineStore } from 'pinia'
import { shallowRef, ref, computed } from 'vue'
import type { SampleResult, AggregateStats } from '@/types'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export type ExecutionStatus = 'idle' | 'running' | 'stopping' | 'completed'

export const useExecutionStore = defineStore('execution', () => {
  const status = shallowRef<ExecutionStatus>('idle')
  const results = ref<SampleResult[]>([])
  const errorCount = shallowRef(0)
  const startTime = shallowRef<number | null>(null)
  const threadsActive = shallowRef(0)
  const totalSamples = shallowRef(0)
  const unlistenResult: (() => void)[] = []

  const isRunning = computed(() => status.value === 'running')
  const isIdle = computed(() => status.value === 'idle')
  const elapsedSeconds = computed(() => {
    if (!startTime.value) return 0
    return ((Date.now() - startTime.value) / 1000).toFixed(1)
  })

  const aggregateByLabel = computed<AggregateStats[]>(() => {
    const groups = new Map<string, SampleResult[]>()
    for (const r of results.value) {
      const list = groups.get(r.label) || []
      list.push(r)
      groups.set(r.label, list)
    }
    const stats: AggregateStats[] = []
    for (const [label, samples] of groups) {
      const elapsed = samples.map(s => s.elapsed).sort((a, b) => a - b)
      const n = elapsed.length
      const sum = elapsed.reduce((a, b) => a + b, 0)
      const errors = samples.filter(s => !s.success).length
      const duration = (samples[n - 1].timestamp - samples[0].timestamp) / 1000
      stats.push({
        label,
        count: n,
        avg: Math.round(sum / n),
        min: elapsed[0],
        max: elapsed[n - 1],
        median: percentile(elapsed, 50),
        p90: percentile(elapsed, 90),
        p95: percentile(elapsed, 95),
        p99: percentile(elapsed, 99),
        errorRate: Math.round((errors / n) * 10000) / 100,
        throughput: duration > 0 ? Math.round((n / duration) * 100) / 100 : 0,
        receivedKBPerSec: 0,
        sentKBPerSec: 0,
      })
    }
    return stats
  })

  function clear() {
    results.value = []
    errorCount.value = 0
    startTime.value = null
    threadsActive.value = 0
    totalSamples.value = 0
  }

  async function startTest(planJson: string) {
    clear()
    status.value = 'running'
    startTime.value = Date.now()

    // Listen for result events from the Rust backend
    try {
      const unlisten1 = await listen<SampleResult>('test://result', (event) => {
        results.value.push(event.payload)
        totalSamples.value++
        if (!event.payload.success) errorCount.value++
      })
      const unlisten2 = await listen<{ status: string; threadsActive: number; totalSamples: number; errorCount: number }>(
        'test://status',
        (event) => {
          threadsActive.value = event.payload.threadsActive
          totalSamples.value = event.payload.totalSamples
          errorCount.value = event.payload.errorCount
        }
      )
      unlistenResult.push(unlisten1, unlisten2)
    } catch (_e) {
      // Tauri events won't work in browser-only dev mode, ignore
    }

    try {
      await invoke('start_test', { planJson })
      status.value = 'completed'
      // Send system notification on completion
      sendNotification('Test Completed', `Samples: ${totalSamples.value} | Errors: ${errorCount.value}`)
    } catch (e) {
      status.value = 'idle'
      console.error('Test execution error:', e)
    }

    // Clean up listeners
    for (const fn of unlistenResult) fn()
    unlistenResult.length = 0
  }

  async function stopTest() {
    status.value = 'stopping'
    try {
      await invoke('stop_test')
    } catch (_e) {
      // ignore
    }
    status.value = 'idle'
  }

  return {
    status,
    results,
    errorCount,
    startTime,
    threadsActive,
    totalSamples,
    isRunning,
    isIdle,
    elapsedSeconds,
    aggregateByLabel,
    clear,
    startTest,
    stopTest,
  }
})

async function sendNotification(title: string, body: string) {
  try {
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      const { isPermissionGranted, requestPermission, sendNotification: notify } = await import('@tauri-apps/plugin-notification')
      let granted = await isPermissionGranted()
      if (!granted) {
        const perm = await requestPermission()
        granted = perm === 'granted'
      }
      if (granted) {
        notify({ title, body })
      }
    }
  } catch {
    // Silently fail if notification not available
  }
}

function percentile(sorted: number[], p: number): number {
  const idx = Math.ceil((p / 100) * sorted.length) - 1
  return sorted[Math.max(0, idx)] || 0
}
