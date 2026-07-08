<template>
  <div class="panel results-panel">
    <div class="panel-header">
      <div class="header-left-group">
        <span>Results</span>
        <button
          v-if="execution.results.length > 0"
          class="export-btn"
          title="Export results as CSV"
          @click="exportCsv"
        >CSV</button>
      </div>
      <div class="header-tabs">
        <button
          v-for="tab in tabs" :key="tab.key"
          :class="{ active: ui.activeResultTab === tab.key }"
          @click="ui.setActiveResultTab(tab.key)"
        >{{ tab.label }}</button>
      </div>
    </div>
    <div class="results-body">
      <!-- Empty state -->
      <div v-if="execution.results.length === 0" class="empty-state">
        <p>Run the test to see results</p>
      </div>

      <!-- Summary Table -->
      <div v-else-if="ui.activeResultTab === 'summary'" class="summary-table">
        <table>
          <thead>
            <tr>
              <th>Label</th><th>#Samples</th><th>Avg</th><th>Min</th><th>Max</th>
              <th>Median</th><th>p90</th><th>p95</th><th>p99</th>
              <th>Error%</th><th>Throughput</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="s in execution.aggregateByLabel" :key="s.label">
              <td>{{ s.label }}</td>
              <td>{{ s.count }}</td>
              <td>{{ s.avg }}ms</td>
              <td>{{ s.min }}ms</td>
              <td>{{ s.max }}ms</td>
              <td>{{ s.median }}ms</td>
              <td>{{ s.p90 }}ms</td>
              <td>{{ s.p95 }}ms</td>
              <td>{{ s.p99 }}ms</td>
              <td :class="{ error: s.errorRate > 0 }">{{ s.errorRate }}%</td>
              <td>{{ s.throughput }}/s</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Results Tree -->
      <div v-else-if="ui.activeResultTab === 'tree'" class="results-list" @scroll="onTreeScroll">
        <div class="scroll-spacer" :style="{ height: visibleResults.totalHeight + 'px' }">
          <div class="visible-window" :style="{ transform: `translateY(${visibleResults.offsetY}px)` }">
            <div
              v-for="item in visibleResults.items" :key="item.result.id"
              :class="['result-item', { fail: !item.result.success }]"
            >
              <span class="idx">{{ item.index + 1 }}</span>
              <span :class="['dot', item.result.success ? 'ok' : 'fail']"></span>
              <span class="r-label">{{ item.result.label }}</span>
              <span class="r-code">{{ item.result.responseCode }}</span>
              <span class="r-time">{{ item.result.elapsed }}ms</span>
              <span class="r-size">{{ (item.result.bytes / 1024).toFixed(1) }}KB</span>
            </div>
          </div>
        </div>
        <div class="result-count">
          {{ execution.results.length.toLocaleString() }} samples
        </div>
      </div>

      <!-- Charts -->
      <div v-else class="charts-grid">
        <div class="chart-container">
          <div class="chart-title">Response Time</div>
          <v-chart class="chart" :option="responseTimeOption" autoresize />
        </div>
        <div class="chart-row">
          <div class="chart-container half">
            <div class="chart-title">Throughput (req/s)</div>
            <v-chart class="chart" :option="throughputOption" autoresize />
          </div>
          <div class="chart-container half">
            <div class="chart-title">Error Distribution</div>
            <v-chart class="chart" :option="errorPieOption" autoresize />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { computed, defineAsyncComponent, ref } from 'vue'
import { useUIStore, useExecutionStore } from '@/stores'

const VChart = defineAsyncComponent(async () => {
  const { default: VChartComp } = await import('vue-echarts')
  const { use: echartsUse } = await import('echarts/core')
  const { LineChart, BarChart, PieChart } = await import('echarts/charts')
  const { TitleComponent, TooltipComponent, GridComponent, LegendComponent, DataZoomComponent } = await import('echarts/components')
  const { CanvasRenderer } = await import('echarts/renderers')

  echartsUse([LineChart, BarChart, PieChart, TitleComponent, TooltipComponent, GridComponent, LegendComponent, DataZoomComponent, CanvasRenderer])

  return VChartComp
})

const ui = useUIStore()
const execution = useExecutionStore()

// Virtual scroll for results tree
const ITEM_HEIGHT = 26
const BUFFER = 20
const treeScrollTop = ref(0)
const treeViewHeight = ref(400)

const visibleResults = computed(() => {
  const total = execution.results.length
  if (total === 0) return { items: [], offsetY: 0, totalHeight: 0 }
  const start = Math.max(0, Math.floor(treeScrollTop.value / ITEM_HEIGHT) - BUFFER)
  const end = Math.min(total, Math.ceil((treeScrollTop.value + treeViewHeight.value) / ITEM_HEIGHT) + BUFFER)
  return {
    items: execution.results.slice(start, end).map((r, i) => ({ result: r, index: start + i })),
    offsetY: start * ITEM_HEIGHT,
    totalHeight: total * ITEM_HEIGHT,
  }
})

function onTreeScroll(e: Event) {
  treeScrollTop.value = (e.target as HTMLElement).scrollTop
  treeViewHeight.value = (e.target as HTMLElement).clientHeight
}

// ---- CSV Export ----
function exportCsv() {
  if (execution.results.length === 0) return
  const headers = ['timestamp', 'label', 'elapsed', 'responseCode', 'success', 'bytes', 'latency', 'threadName', 'url', 'errorMessage']
  const rows = execution.results.map(r =>
    [r.timestamp, escapeCsv(r.label), r.elapsed, r.responseCode, r.success, r.bytes, r.latency, escapeCsv(r.threadName), escapeCsv(r.url), escapeCsv(r.errorMessage || '')].join(',')
  )
  const csv = [headers.join(','), ...rows].join('\n')

  // Download via Tauri or browser
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    import('@tauri-apps/plugin-fs').then(({ writeTextFile }) => {
      import('@tauri-apps/plugin-dialog').then(({ save }) => {
        save({ defaultPath: 'results.csv', filters: [{ name: 'CSV', extensions: ['csv'] }] }).then(fp => {
          if (fp) writeTextFile(fp, csv)
        })
      })
    })
  } else {
    const blob = new Blob([csv], { type: 'text/csv' })
    const a = document.createElement('a')
    a.href = URL.createObjectURL(blob)
    a.download = 'results.csv'
    a.click()
  }
}

function escapeCsv(s: string): string {
  if (s.includes(',') || s.includes('"') || s.includes('\n')) {
    return `"${s.replace(/"/g, '""')}"`
  }
  return s
}

const tabs = [
  { key: 'tree', label: 'Tree' },
  { key: 'summary', label: 'Summary' },
  { key: 'chart', label: 'Chart' },
]

// ---- Chart options ----

const responseTimeOption = computed(() => {
  const data = execution.results.slice(-2000) // last 2000 samples for performance
  return {
    tooltip: {
      trigger: 'axis',
      formatter: (params: { value: number[]; seriesName: string }[]) => {
        if (!params.length) return ''
        const pt = params[0]
        return `#${pt.value[0]}<br/>${pt.seriesName}: ${pt.value[1]}ms`
      },
    },
    legend: { data: ['Success', 'Error'], textStyle: { color: '#a6adc8', fontSize: 11 } },
    grid: { top: 40, right: 20, bottom: 40, left: 55 },
    xAxis: { type: 'value', name: 'Sample #', nameTextStyle: { color: '#6c7086', fontSize: 10 } },
    yAxis: { type: 'value', name: 'ms', nameTextStyle: { color: '#6c7086', fontSize: 10 } },
    dataZoom: [{ type: 'inside' }, { type: 'slider', height: 16, bottom: 4, borderColor: '#45475a' }],
    series: [
      {
        name: 'Success',
        type: 'line',
        symbol: 'none',
        lineStyle: { color: '#a6e3a1', width: 1 },
        data: data.filter(r => r.success).map((r, i) => [i, r.elapsed]),
      },
      {
        name: 'Error',
        type: 'line',
        symbol: 'none',
        lineStyle: { color: '#f38ba8', width: 1 },
        data: data.filter(r => !r.success).map((r, i) => [i, r.elapsed]),
      },
    ],
  }
})

const throughputOption = computed(() => {
  const results = execution.results
  if (results.length < 2) return {}

  const bucketMs = 1000
  const minTs = results[0].timestamp
  const buckets = new Map<number, number>()

  for (const r of results) {
    const bucket = Math.floor((r.timestamp - minTs) / bucketMs)
    buckets.set(bucket, (buckets.get(bucket) || 0) + 1)
  }

  const data: [number, number][] = []
  buckets.forEach((count, bucket) => {
    data.push([bucket * bucketMs / 1000, count])
  })
  data.sort((a, b) => a[0] - b[0])

  return {
    tooltip: { trigger: 'axis', formatter: (p: { value: number[] }[]) => `t=${p[0]?.value[0]}s<br/>${p[0]?.value[1]} req/s` },
    grid: { top: 10, right: 20, bottom: 35, left: 55 },
    xAxis: { type: 'value', name: 'Time (s)', nameTextStyle: { color: '#6c7086', fontSize: 10 } },
    yAxis: { type: 'value', name: 'req/s', nameTextStyle: { color: '#6c7086', fontSize: 10 } },
    dataZoom: [{ type: 'inside' }, { type: 'slider', height: 16, bottom: 4, borderColor: '#45475a' }],
    series: [{
      name: 'Throughput',
      type: 'bar',
      data,
      itemStyle: { color: '#89b4fa' },
    }],
  }
})

const errorPieOption = computed(() => {
  const ok = execution.results.filter(r => r.success).length
  const err = execution.results.length - ok
  if (ok + err === 0) return {}

  return {
    tooltip: { trigger: 'item', formatter: '{b}: {c} ({d}%)' },
    legend: { orient: 'vertical', left: 0, top: 20, textStyle: { color: '#a6adc8', fontSize: 11 } },
    series: [{
      type: 'pie',
      radius: ['45%', '75%'],
      center: ['55%', '55%'],
      avoidLabelOverlap: false,
      label: { show: false },
      emphasis: { label: { show: true, fontWeight: 'bold' } },
      data: [
        { value: ok, name: 'Success', itemStyle: { color: '#a6e3a1' } },
        { value: err, name: 'Error', itemStyle: { color: '#f38ba8' } },
      ],
    }],
  }
})
</script>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.results-panel {
  flex: 1;
  min-width: 300px;
  background: var(--bg-primary);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  user-select: none;
}

.header-left-group { display: flex; align-items: center; gap: 8px; }

.header-tabs { display: flex; gap: 2px; }

.header-tabs button, .export-btn {
  padding: 2px 10px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
}

.header-tabs button.active { background: var(--accent); color: var(--bg-primary); }

.export-btn { color: var(--success); }
.export-btn:hover { background: rgba(166, 227, 161, 0.12); }

.results-body { flex: 1; overflow-y: auto; padding: 4px; }

.empty-state { padding: 40px 20px; text-align: center; color: var(--text-muted); }

/* Summary table */
.summary-table { font-size: 11px; overflow-x: auto; }

.summary-table table { width: 100%; border-collapse: collapse; }

.summary-table th, .summary-table td {
  padding: 4px 8px;
  text-align: left;
  white-space: nowrap;
  border-bottom: 1px solid var(--border);
}

.summary-table th { background: var(--bg-surface); font-weight: 600; position: sticky; top: 0; z-index: 1; }

.summary-table td.error { color: var(--danger); font-weight: 600; }

/* Results tree */
.results-list { font-size: 12px; overflow-y: auto; }

.scroll-spacer { position: relative; width: 100%; }

.visible-window { position: absolute; top: 0; left: 0; right: 0; }

.result-count {
  position: sticky;
  bottom: 0;
  background: var(--bg-secondary);
  color: var(--text-muted);
  font-size: 10px;
  padding: 2px 8px;
  text-align: center;
  border-top: 1px solid var(--border);
}

.idx {
  width: 36px;
  text-align: right;
  color: var(--text-muted);
  flex-shrink: 0;
}

.result-item {
  height: 26px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 3px 8px;
  border-bottom: 1px solid var(--border);
}

.result-item.fail { background: rgba(243, 139, 168, 0.08); }

.dot {
  width: 8px; height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dot.ok { background: var(--success); }
.dot.fail { background: var(--danger); }

.r-label { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.r-code { width: 50px; text-align: center; color: var(--text-secondary); }
.r-time { width: 60px; text-align: right; font-variant-numeric: tabular-nums; }
.r-size { width: 50px; text-align: right; color: var(--text-muted); }

/* Charts */
.charts-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 8px;
  height: 100%;
}

.chart-container {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
}

.chart-container:first-child { flex: 1; min-height: 200px; }

.chart-row {
  display: flex;
  gap: 8px;
  height: 220px;
}

.chart-container.half { flex: 1; }

.chart-title {
  padding: 6px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.3px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
}

.chart {
  width: 100%;
  height: calc(100% - 28px);
}
</style>
