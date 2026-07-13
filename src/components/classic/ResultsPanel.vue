<template>
  <div class="panel results-panel">
    <div class="panel-header">
      <div class="header-left-group">
        <span>Results</span>
        <button v-if="execution.results.length > 0" class="export-btn" title="Export results as CSV" @click="exportCsv">
          CSV
        </button>
      </div>
      <div class="header-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          :class="{ active: ui.activeResultTab === tab.key }"
          @click="ui.setActiveResultTab(tab.key)"
        >
          {{ tab.label }}
        </button>
      </div>
    </div>
    <div class="results-body">
      <!-- Live Dashboard -->
      <div v-if="ui.activeResultTab === 'live'" class="live-tab">
        <LiveDashboard ref="liveDashboard" />
      </div>

      <!-- Empty state -->
      <div v-else-if="execution.results.length === 0" class="empty-state">
        <p>Run the test to see results</p>
      </div>

      <!-- Summary Table -->
      <div v-else-if="ui.activeResultTab === 'summary'" class="summary-table">
        <table>
          <thead>
            <tr>
              <th>Label</th>
              <th>#Samples</th>
              <th>Avg</th>
              <th>Min</th>
              <th>Max</th>
              <th>Median</th>
              <th>p90</th>
              <th>p95</th>
              <th>p99</th>
              <th>Error%</th>
              <th>Throughput</th>
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
              v-for="item in visibleResults.items"
              :key="item.result.id"
              :class="['result-item', { fail: !item.result.success, selected: selectedResultId === item.result.id }]"
              @click="selectResult(item.result.id)"
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
        <div class="result-count">{{ execution.results.length.toLocaleString() }} samples</div>

        <!-- Result Detail -->
        <div v-if="selectedResult" class="result-detail">
          <div class="detail-header">
            <span class="detail-title">{{ selectedResult.label }}</span>
            <span :class="['detail-badge', selectedResult.success ? 'ok' : 'fail']">
              {{ selectedResult.success ? 'PASS' : 'FAIL' }}
            </span>
            <span class="detail-close" @click="selectedResultId = null">&times;</span>
          </div>

          <div class="detail-meta">
            <div class="meta-item">
              <span class="meta-label">URL</span><span class="meta-value mono">{{ selectedResult.url }}</span>
            </div>
            <div class="meta-row">
              <div class="meta-item">
                <span class="meta-label">Method</span><span class="meta-value">{{ selectedResult.method }}</span>
              </div>
              <div class="meta-item">
                <span class="meta-label">Status</span
                ><span class="meta-value">{{ selectedResult.responseCode }} {{ selectedResult.responseMessage }}</span>
              </div>
              <div class="meta-item">
                <span class="meta-label">Time</span><span class="meta-value">{{ selectedResult.elapsed }}ms</span>
              </div>
              <div class="meta-item">
                <span class="meta-label">Size</span
                ><span class="meta-value">{{ (selectedResult.bytes / 1024).toFixed(1) }}KB</span>
              </div>
            </div>
            <div v-if="selectedResult.errorMessage" class="meta-item error-msg">
              <span class="meta-label">Error</span>
              <span class="meta-value">{{ selectedResult.errorMessage }}</span>
            </div>
          </div>

          <div class="detail-tabs">
            <span :class="['dtab', { active: detailTab === 'response' }]" @click="detailTab = 'response'"
              >Response</span
            >
            <span :class="['dtab', { active: detailTab === 'request' }]" @click="detailTab = 'request'">Request</span>
            <span :class="['dtab', { active: detailTab === 'assertions' }]" @click="detailTab = 'assertions'">
              Assertions
              <span v-if="selectedResult.assertionResults.length" class="tab-badge">{{
                selectedResult.assertionResults.length
              }}</span>
            </span>
          </div>

          <div class="detail-content">
            <!-- Response tab -->
            <div v-if="detailTab === 'response'">
              <div class="prop-section" v-if="selectedResult.responseHeaders.length">
                <div class="section-title">Response Headers</div>
                <div v-for="(h, i) in selectedResult.responseHeaders" :key="i" class="header-line">
                  <span class="hdr-key">{{ h.key }}:</span>
                  <span class="hdr-value">{{ h.value }}</span>
                </div>
              </div>
              <div class="prop-section">
                <div class="section-title">Response Body</div>
                <pre class="body-view">
                  <span v-if="bodyLoading">Loading...</span>
                  <template v-else>{{ displayBody() || '(empty response)' }}</template>
                </pre>
              </div>
            </div>

            <!-- Request tab -->
            <div v-if="detailTab === 'request'">
              <div class="prop-section" v-if="selectedResult.requestHeaders.length">
                <div class="section-title">Request Headers</div>
                <div v-for="(h, i) in selectedResult.requestHeaders" :key="i" class="header-line">
                  <span class="hdr-key">{{ h.key }}:</span>
                  <span class="hdr-value">{{ h.value }}</span>
                </div>
              </div>
            </div>

            <!-- Assertions tab -->
            <div v-if="detailTab === 'assertions'">
              <div v-if="!selectedResult.assertionResults.length" class="empty-detail">No assertions configured</div>
              <div v-else>
                <div
                  v-for="(ar, i) in selectedResult.assertionResults"
                  :key="i"
                  :class="['assertion-row', ar.failure ? 'fail' : 'pass']"
                >
                  <span class="assert-icon">{{ ar.failure ? '&#10007;' : '&#10003;' }}</span>
                  <div class="assert-info">
                    <span class="assert-name">{{ ar.name }}</span>
                    <span v-if="ar.failureMessage" class="assert-msg">{{ ar.failureMessage }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Charts (loaded on demand when tab is activated) -->
      <div v-else-if="VChart" class="charts-grid">
        <div class="chart-container">
          <div class="chart-title">Response Time</div>
          <component :is="VChart" class="chart" :option="responseTimeOption" autoresize />
        </div>
        <div class="chart-row">
          <div class="chart-container half">
            <div class="chart-title">Throughput (req/s)</div>
            <component :is="VChart" class="chart" :option="throughputOption" autoresize />
          </div>
          <div class="chart-container half">
            <div class="chart-title">Error Distribution</div>
            <component :is="VChart" class="chart" :option="errorPieOption" autoresize />
          </div>
        </div>
      </div>
      <div v-else class="empty-state"><p>Loading charts...</p></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, shallowRef, watch, useTemplateRef, type Component } from 'vue'
import { useUIStore, useExecutionStore } from '@/stores'
import { invoke } from '@tauri-apps/api/core'
import LiveDashboard from './LiveDashboard.vue'

const ui = useUIStore()
const execution = useExecutionStore()
const liveDashboardRef = useTemplateRef<InstanceType<typeof LiveDashboard>>('liveDashboard')

// Feed live status ticks to the dashboard
watch(
  () => execution.statusTick,
  tick => {
    if (tick && liveDashboardRef.value) {
      liveDashboardRef.value.pushTick(tick)
    }
  },
)

// ECharts — only load when user switches to chart tab
const VChart = shallowRef<Component | null>(null)
let chartLoaded = false

async function loadCharts() {
  if (chartLoaded) return
  chartLoaded = true
  const [{ default: VChartComp }, { use: echartsUse }] = await Promise.all([
    import('vue-echarts'),
    import('echarts/core'),
  ])
  const [
    { LineChart, BarChart, PieChart },
    { TitleComponent, TooltipComponent, GridComponent, LegendComponent, DataZoomComponent },
    { CanvasRenderer },
  ] = await Promise.all([import('echarts/charts'), import('echarts/components'), import('echarts/renderers')])
  echartsUse([
    LineChart,
    BarChart,
    PieChart,
    TitleComponent,
    TooltipComponent,
    GridComponent,
    LegendComponent,
    DataZoomComponent,
    CanvasRenderer,
  ])
  VChart.value = VChartComp
}

watch(
  () => ui.activeResultTab,
  tab => {
    if (tab === 'chart') loadCharts()
  },
)

// Result detail view
const selectedResultId = shallowRef<string | null>(null)
const detailTab = shallowRef<'response' | 'request' | 'assertions'>('response')
const fetchedBody = shallowRef('')
const bodyLoading = shallowRef(false)

const selectedResult = computed(() => {
  if (!selectedResultId.value) return null
  return execution.results.find(r => r.id === selectedResultId.value) || null
})

function selectResult(id: string) {
  if (selectedResultId.value === id) {
    selectedResultId.value = null
    fetchedBody.value = ''
    return
  }
  selectedResultId.value = id
  fetchedBody.value = ''
  bodyLoading.value = true
  invoke<string>('get_response_body', { id })
    .then(body => {
      fetchedBody.value = body
    })
    .catch(() => {
      fetchedBody.value = ''
    })
    .finally(() => {
      bodyLoading.value = false
    })
}

function displayBody(): string {
  const body = fetchedBody.value || selectedResult.value?.responseBody || ''
  try {
    return JSON.stringify(JSON.parse(body), null, 2)
  } catch {
    return body
  }
}

// Virtual scroll for results tree
const ITEM_HEIGHT = 26
const BUFFER = 20
const treeScrollTop = shallowRef(0)
const treeViewHeight = shallowRef(400)

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
  const headers = [
    'timestamp',
    'label',
    'elapsed',
    'responseCode',
    'success',
    'bytes',
    'latency',
    'threadName',
    'url',
    'errorMessage',
  ]
  const rows = execution.results.map(r =>
    [
      r.timestamp,
      escapeCsv(r.label),
      r.elapsed,
      r.responseCode,
      r.success,
      r.bytes,
      r.latency,
      escapeCsv(r.threadName),
      escapeCsv(r.url),
      escapeCsv(r.errorMessage || ''),
    ].join(','),
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
  { key: 'live', label: 'Live' },
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
    legend: { data: ['Success', 'Error'], textStyle: { color: '#9396a8', fontSize: 11 } },
    grid: { top: 40, right: 20, bottom: 40, left: 55 },
    xAxis: { type: 'value', name: 'Sample #', nameTextStyle: { color: '#5c6078', fontSize: 10 } },
    yAxis: { type: 'value', name: 'ms', nameTextStyle: { color: '#5c6078', fontSize: 10 } },
    dataZoom: [{ type: 'inside' }, { type: 'slider', height: 16, bottom: 4, borderColor: '#2a2d3d' }],
    series: [
      {
        name: 'Success',
        type: 'line',
        symbol: 'none',
        lineStyle: { color: '#4dc9b0', width: 1 },
        data: data.filter(r => r.success).map((r, i) => [i, r.elapsed]),
      },
      {
        name: 'Error',
        type: 'line',
        symbol: 'none',
        lineStyle: { color: '#e0556a', width: 1 },
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
    data.push([(bucket * bucketMs) / 1000, count])
  })
  data.sort((a, b) => a[0] - b[0])

  return {
    tooltip: {
      trigger: 'axis',
      formatter: (p: { value: number[] }[]) => `t=${p[0]?.value[0]}s<br/>${p[0]?.value[1]} req/s`,
    },
    grid: { top: 10, right: 20, bottom: 35, left: 55 },
    xAxis: { type: 'value', name: 'Time (s)', nameTextStyle: { color: '#5c6078', fontSize: 10 } },
    yAxis: { type: 'value', name: 'req/s', nameTextStyle: { color: '#5c6078', fontSize: 10 } },
    dataZoom: [{ type: 'inside' }, { type: 'slider', height: 16, bottom: 4, borderColor: '#2a2d3d' }],
    series: [
      {
        name: 'Throughput',
        type: 'bar',
        data,
        itemStyle: { color: '#4dc9b0' },
      },
    ],
  }
})

const errorPieOption = computed(() => {
  const ok = execution.results.filter(r => r.success).length
  const err = execution.results.length - ok
  if (ok + err === 0) return {}

  return {
    tooltip: { trigger: 'item', formatter: '{b}: {c} ({d}%)' },
    legend: { orient: 'vertical', left: 0, top: 20, textStyle: { color: '#9396a8', fontSize: 11 } },
    series: [
      {
        type: 'pie',
        radius: ['45%', '75%'],
        center: ['55%', '55%'],
        avoidLabelOverlap: false,
        label: { show: false },
        emphasis: { label: { show: true, fontWeight: 'bold' } },
        data: [
          { value: ok, name: 'Success', itemStyle: { color: '#4dc9b0' } },
          { value: err, name: 'Error', itemStyle: { color: '#e0556a' } },
        ],
      },
    ],
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
  background: var(--bg-base);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 7px 12px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.7px;
  user-select: none;
  color: var(--text-secondary);
}

.header-left-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-tabs {
  display: flex;
  gap: 2px;
}

.header-tabs button,
.export-btn {
  padding: 3px 12px;
  border: none;
  border-radius: 3px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 10px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.12s;
}

.header-tabs button.active {
  background: var(--accent);
  color: var(--bg-deep);
  font-weight: 600;
}
.header-tabs button:hover:not(.active) {
  color: var(--text-primary);
}

.export-btn {
  color: var(--accent-cool);
  font-weight: 600;
  padding: 2px 8px;
  border: 1px solid var(--accent-cool);
  border-radius: 3px;
}
.export-btn:hover {
  background: rgba(77, 201, 176, 0.1);
}

.results-body {
  flex: 1;
  overflow-y: auto;
  padding: 4px;
}

.empty-state {
  padding: 40px 20px;
  text-align: center;
  color: var(--text-muted);
}

/* Summary table */
.summary-table {
  font-size: 11px;
  overflow-x: auto;
}

.summary-table table {
  width: 100%;
  border-collapse: collapse;
}

.summary-table th,
.summary-table td {
  padding: 5px 10px;
  text-align: left;
  white-space: nowrap;
  border-bottom: 1px solid var(--border);
}

.summary-table th {
  background: var(--bg-surface);
  font-weight: 600;
  position: sticky;
  top: 0;
  z-index: 1;
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.4px;
}

.summary-table td {
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 11px;
}
.summary-table td.error {
  color: var(--danger);
  font-weight: 600;
}

/* Results tree */
.results-list {
  font-size: 12px;
  overflow-y: auto;
}

.scroll-spacer {
  position: relative;
  width: 100%;
}

.visible-window {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
}

.result-count {
  position: sticky;
  bottom: 0;
  background: var(--bg-surface);
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
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 10px;
}

.result-item {
  height: 26px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 3px 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  border-left: 2px solid transparent;
}

.result-item.fail {
  background: var(--danger-glow);
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dot.ok {
  background: var(--accent-cool);
}
.dot.fail {
  background: var(--danger);
}

.r-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.r-code {
  width: 50px;
  text-align: center;
  color: var(--text-secondary);
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 11px;
}
.r-time {
  width: 60px;
  text-align: right;
  font-variant-numeric: tabular-nums;
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}
.r-size {
  width: 50px;
  text-align: right;
  color: var(--text-muted);
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 11px;
}

.result-item {
  cursor: pointer;
}
.result-item:hover {
  background: var(--bg-hover);
}
.result-item.selected {
  background: var(--accent-glow);
  border-left-color: var(--accent);
}

/* Result Detail */
.result-detail {
  border-top: 2px solid var(--accent);
  background: var(--bg-deep);
  max-height: 50%;
  overflow-y: auto;
  font-size: 12px;
}

.detail-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  position: sticky;
  top: 0;
  z-index: 1;
}

.detail-title {
  font-weight: 600;
  flex: 1;
}

.detail-badge {
  padding: 1px 8px;
  border-radius: 2px;
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}
.detail-badge.ok {
  background: rgba(77, 201, 176, 0.15);
  color: var(--accent-cool);
}
.detail-badge.fail {
  background: var(--danger-glow);
  color: var(--danger);
}

.detail-close {
  font-size: 18px;
  color: var(--text-muted);
  cursor: pointer;
  padding: 0 4px;
}
.detail-close:hover {
  color: var(--danger);
}

.detail-meta {
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  border-bottom: 1px solid var(--border);
}

.meta-item {
  display: flex;
  gap: 8px;
  align-items: baseline;
}
.meta-row {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.meta-label {
  font-size: 9px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.4px;
  min-width: 50px;
}

.meta-value {
  color: var(--text-primary);
  font-size: 12px;
}
.meta-value.mono {
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 11px;
}

.error-msg .meta-value {
  color: var(--danger);
}

.detail-tabs {
  display: flex;
  gap: 2px;
  padding: 0 10px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
}

.dtab {
  padding: 6px 14px;
  font-size: 11px;
  color: var(--text-muted);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  display: flex;
  align-items: center;
  gap: 4px;
  font-weight: 500;
}

.dtab.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

.tab-badge {
  background: var(--accent);
  color: var(--bg-deep);
  padding: 0 5px;
  border-radius: 8px;
  font-size: 9px;
  font-weight: 600;
}

.detail-content {
  padding: 8px 10px;
}

.header-line {
  display: flex;
  gap: 4px;
  padding: 1px 0;
  font-size: 11px;
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}
.hdr-key {
  color: var(--accent);
  white-space: nowrap;
}
.hdr-value {
  color: var(--text-secondary);
  word-break: break-all;
}

.body-view {
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: 3px;
  padding: 10px;
  font-size: 11px;
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 300px;
  overflow-y: auto;
  color: var(--text-primary);
}

.empty-detail {
  padding: 20px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}

.assertion-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 6px 8px;
  margin-bottom: 4px;
  border-radius: 3px;
  background: var(--bg-base);
  border: 1px solid var(--border);
}
.assertion-row.pass {
  border-left: 3px solid var(--accent-cool);
}
.assertion-row.fail {
  border-left: 3px solid var(--danger);
}

.assert-icon {
  font-size: 14px;
  flex-shrink: 0;
}
.assertion-row.pass .assert-icon {
  color: var(--accent-cool);
}
.assertion-row.fail .assert-icon {
  color: var(--danger);
}

.assert-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.assert-name {
  font-weight: 600;
  font-size: 12px;
}
.assert-msg {
  color: var(--danger);
  font-size: 11px;
}

/* Charts */
.charts-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 8px;
  height: 100%;
}

.chart-container {
  background: var(--bg-deep);
  border: 1px solid var(--border);
  border-radius: 0;
  overflow: hidden;
}

.chart-container:first-child {
  flex: 1;
  min-height: 200px;
}

.chart-row {
  display: flex;
  gap: 8px;
  height: 220px;
}

.chart-container.half {
  flex: 1;
}

.chart-title {
  padding: 6px 12px;
  font-size: 10px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
}

.chart {
  width: 100%;
  height: calc(100% - 28px);
}
</style>
