<script setup lang="ts">
import { shallowRef, onUnmounted, type Component } from 'vue'
import { useExecutionStore } from '@/stores'
import { useChartOptions } from '@/composables/useChartOptions'
import { formatElapsed, now } from '@/utils/time'

const execution = useExecutionStore()

// ECharts — lazy loaded when component mounts (charts tab logic handles this)
const VChart = shallowRef<Component | null>(null)
let loaded = false

async function loadCharts() {
  if (loaded) return
  loaded = true
  const [{ default: Comp }, { use: echartsUse }] = await Promise.all([import('vue-echarts'), import('echarts/core')])
  const [
    { LineChart, BarChart },
    { TitleComponent, TooltipComponent, GridComponent, LegendComponent },
    { CanvasRenderer },
  ] = await Promise.all([import('echarts/charts'), import('echarts/components'), import('echarts/renderers')])
  echartsUse([LineChart, BarChart, TitleComponent, TooltipComponent, GridComponent, LegendComponent, CanvasRenderer])
  VChart.value = Comp
}

// Initialize on mount
loadCharts()

// --- Time-series data (rolling window, last 120 data points = 30s at 250ms intervals) ---
const MAX_POINTS = 120

interface Tick {
  time: number
  throughput: number
  p50: number
  p90: number
  p99: number
  avg: number
  errorRate: number
  threads: number
  totalSamples: number
}

const ticks = shallowRef<Tick[]>([])
const startTime = shallowRef(0)

// Called externally when new status arrives
function pushTick(status: {
  threadsActive: number
  totalSamples: number
  errorCount: number
  p50?: number
  p90?: number
  p99?: number
  throughput?: number
  avgResponseTime?: number
}) {
  if (!startTime.value) startTime.value = now()
  const t = (now() - startTime.value) / 1000
  const total = status.totalSamples || 1
  ticks.value = [
    ...ticks.value.slice(-(MAX_POINTS - 1)),
    {
      time: Math.round(t * 10) / 10,
      throughput: status.throughput ?? 0,
      p50: status.p50 ?? 0,
      p90: status.p90 ?? 0,
      p99: status.p99 ?? 0,
      avg: status.avgResponseTime ?? 0,
      errorRate: total > 0 ? Math.round((status.errorCount / total) * 1000) / 10 : 0,
      threads: status.threadsActive,
      totalSamples: total,
    },
  ]
}

// Expose for parent
defineExpose({ pushTick })

// Chart options from shared composable
const { throughputOption, responseTimeOption, errorRateOption, threadsOption } = useChartOptions(ticks)

onUnmounted(() => {
  ticks.value = []
  startTime.value = 0
})
</script>

<template>
  <div class="live-dashboard">
    <div class="live-stats-bar">
      <div class="live-stat">
        <span class="live-stat-val">{{ execution.totalSamples.toLocaleString() }}</span>
        <span class="live-stat-lbl">Samples</span>
      </div>
      <div class="live-stat">
        <span class="live-stat-val">{{ execution.threadsActive }}</span>
        <span class="live-stat-lbl">Threads</span>
      </div>
      <div class="live-stat">
        <span class="live-stat-val" :style="{ color: execution.errorCount > 0 ? 'var(--danger)' : 'var(--success)' }">
          {{ execution.errorCount }}
        </span>
        <span class="live-stat-lbl">Errors</span>
      </div>
      <div class="live-stat">
        <span class="live-stat-val">{{ formatElapsed(execution.elapsedSeconds) }}</span>
        <span class="live-stat-lbl">Elapsed</span>
      </div>
    </div>

    <div class="live-charts-grid" v-if="VChart">
      <div class="live-chart-box">
        <div class="live-chart-title">Throughput (req/s)</div>
        <component :is="VChart" class="live-chart" :option="throughputOption" autoresize />
      </div>
      <div class="live-chart-box">
        <div class="live-chart-title">Response Time (ms)</div>
        <component :is="VChart" class="live-chart" :option="responseTimeOption" autoresize />
      </div>
      <div class="live-chart-box">
        <div class="live-chart-title">Error Rate (%)</div>
        <component :is="VChart" class="live-chart" :option="errorRateOption" autoresize />
      </div>
      <div class="live-chart-box">
        <div class="live-chart-title">Active Threads</div>
        <component :is="VChart" class="live-chart" :option="threadsOption" autoresize />
      </div>
    </div>
    <div v-else class="loading-charts">Loading charts...</div>
  </div>
</template>

<style scoped>
.live-dashboard {
  display: flex;
  flex-direction: column;
  gap: 8px;
  height: 100%;
  overflow: hidden;
}

.live-stats-bar {
  display: flex;
  gap: 2px;
  padding: 0;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  overflow: hidden;
}

.live-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  flex: 1;
  padding: 10px 8px;
  border-right: 1px solid var(--border);
}
.live-stat:last-child {
  border-right: none;
}

.live-stat-val {
  font-size: 22px;
  font-weight: 700;
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-variant-numeric: tabular-nums;
  color: var(--accent);
}

.live-stat-lbl {
  font-size: 9px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 500;
}

.live-charts-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-template-rows: 1fr 1fr;
  gap: 8px;
  flex: 1;
  min-height: 0;
}

.live-chart-box {
  background: var(--bg-deep);
  border: 1px solid var(--border);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.live-chart-title {
  padding: 5px 10px;
  font-size: 9px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
}

.live-chart {
  flex: 1;
  width: 100%;
  min-height: 0;
}

.loading-charts {
  padding: 40px;
  text-align: center;
  color: var(--text-muted);
}
</style>
