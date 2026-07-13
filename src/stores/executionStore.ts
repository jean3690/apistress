import { defineStore } from 'pinia'
import { shallowRef, ref, computed } from 'vue'
import type { SampleResult, AggregateStats } from '@/types'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { now } from '@/utils/time'

export type ExecutionStatus = 'idle' | 'running' | 'stopping' | 'completed'

export const useExecutionStore = defineStore('execution', () => {
  const status = shallowRef<ExecutionStatus>('idle')
  const results = ref<SampleResult[]>([])
  const errorCount = shallowRef(0)
  const startTime = shallowRef<number | null>(null)
  const threadsActive = shallowRef(0)
  const totalSamples = shallowRef(0)
  const resultsVersion = shallowRef(0) // bumped after each batch to throttle computed
  const unlistenResult: (() => void)[] = []

  // Live dashboard metrics
  const statusTick = shallowRef<{
    threadsActive: number
    totalSamples: number
    errorCount: number
    p50?: number
    p90?: number
    p99?: number
    throughput?: number
    avgResponseTime?: number
  } | null>(null)

  // Test-level assertion results
  const assertionResults = shallowRef<
    { metric: string; operator: string; expected: number; actual: number; passed: boolean }[]
  >([])

  const isRunning = computed(() => status.value === 'running')
  const isIdle = computed(() => status.value === 'idle')
  const elapsedSeconds = shallowRef(0)
  let elapsedTimer: ReturnType<typeof setInterval> | null = null

  const aggregateByLabel = computed<AggregateStats[]>(() => {
    void resultsVersion.value // throttle: only recompute once per batch, not once per sample
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
    if (elapsedTimer) {
      clearInterval(elapsedTimer)
      elapsedTimer = null
    }
    results.value = []
    resultsVersion.value = 0
    errorCount.value = 0
    startTime.value = null
    elapsedSeconds.value = 0
    threadsActive.value = 0
    totalSamples.value = 0
    statusTick.value = null
    assertionResults.value = []
  }

  async function startTest(planJson: string) {
    clear()
    status.value = 'running'
    startTime.value = now()
    elapsedTimer = setInterval(() => {
      if (startTime.value) {
        elapsedSeconds.value = parseFloat(((now() - startTime.value) / 1000).toFixed(1))
      }
    }, 100)

    // Listen for batched result events from the Rust backend (emitted every ~100ms)
    try {
      const unlisten1 = await listen<{ results: SampleResult[] }>('test://batch-result', event => {
        const batch = event.payload.results
        for (let i = 0; i < batch.length; i++) {
          results.value.push(batch[i])
          if (!batch[i].success) errorCount.value++
        }
        totalSamples.value += batch.length
        resultsVersion.value++ // trigger computed recalculation once per batch
      })
      const unlisten2 = await listen<{
        status: string
        threadsActive: number
        totalSamples: number
        errorCount: number
        p50?: number
        p90?: number
        p99?: number
        throughput?: number
        avgResponseTime?: number
      }>('test://status', event => {
        threadsActive.value = event.payload.threadsActive
        totalSamples.value = event.payload.totalSamples
        errorCount.value = event.payload.errorCount
        statusTick.value = event.payload
      })
      const unlisten3 = await listen<{
        assertions: { metric: string; operator: string; expected: number; actual: number; passed: boolean }[]
      }>('test://assertion-result', event => {
        assertionResults.value = event.payload.assertions
      })
      unlistenResult.push(unlisten1, unlisten2, unlisten3)
    } catch (_e) {
      // Tauri events won't work in browser-only dev mode, ignore
    }

    try {
      await invoke('start_test', { planJson })
      status.value = 'completed'
      sendNotification('Test Completed', `Samples: ${totalSamples.value} | Errors: ${errorCount.value}`)
    } catch (e) {
      status.value = 'idle'
      console.error('Test execution error:', e)
    } finally {
      if (elapsedTimer) {
        clearInterval(elapsedTimer)
        elapsedTimer = null
      }
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
    if (elapsedTimer) {
      clearInterval(elapsedTimer)
      elapsedTimer = null
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
    statusTick,
    assertionResults,
    clear,
    startTest,
    stopTest,
  }
})

async function sendNotification(title: string, body: string) {
  try {
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      const {
        isPermissionGranted,
        requestPermission,
        sendNotification: notify,
      } = await import('@tauri-apps/plugin-notification')
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
