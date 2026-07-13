import { computed, type Ref } from 'vue'

interface TickData {
  time: number
  p50: number
  p90: number
  p99: number
  avg: number
  throughput: number
  errorRate: number
  threads: number
}

/* Chart colors aligned with the Precision Instrument palette */
const C = {
  accent: '#f0a040',
  accentCool: '#4dc9b0',
  accentGlow: 'rgba(240, 160, 64, 0.12)',
  success: '#4dc9b0',
  warning: '#e5b866',
  danger: '#e0556a',
  dangerGlow: 'rgba(224, 85, 106, 0.12)',
  purple: '#9b8ec4',
  blue: '#6d9ed4',
  textPri: '#e4e6f0',
  textSec: '#9699ab',
  textMut: '#606378',
}

export function useChartOptions(ticks: Ref<TickData[]>) {
  const baseGrid = { top: 30, right: 15, bottom: 30, left: 48 }
  const baseXAxis = { type: 'value' as const, name: 's', nameTextStyle: { color: C.textMut, fontSize: 9 } }
  const baseLineStyle = { symbol: 'none' as const, smooth: true }

  const throughputOption = computed(() => ({
    backgroundColor: 'transparent',
    tooltip: { trigger: 'axis' as const },
    legend: { data: ['Throughput'], textStyle: { color: C.textSec, fontSize: 10 } },
    grid: baseGrid,
    xAxis: baseXAxis,
    yAxis: { type: 'value' as const, name: 'req/s', nameTextStyle: { color: C.textMut, fontSize: 9 } },
    series: [
      {
        name: 'Throughput',
        type: 'line' as const,
        ...baseLineStyle,
        lineStyle: { color: C.accentCool, width: 1.5 },
        areaStyle: { color: 'rgba(77, 201, 176, 0.1)' },
        data: ticks.value.map(t => [t.time, t.throughput]),
      },
    ],
  }))

  const responseTimeOption = computed(() => ({
    backgroundColor: 'transparent',
    tooltip: { trigger: 'axis' as const },
    legend: { data: ['Avg', 'p50', 'p90', 'p99'], textStyle: { color: C.textSec, fontSize: 10 } },
    grid: baseGrid,
    xAxis: baseXAxis,
    yAxis: { type: 'value' as const, name: 'ms', nameTextStyle: { color: C.textMut, fontSize: 9 } },
    series: [
      {
        name: 'Avg',
        type: 'line' as const,
        ...baseLineStyle,
        lineStyle: { color: C.accentCool, width: 1.5 },
        data: ticks.value.map(t => [t.time, t.avg]),
      },
      {
        name: 'p50',
        type: 'line' as const,
        ...baseLineStyle,
        lineStyle: { color: C.blue, width: 1.5 },
        data: ticks.value.map(t => [t.time, t.p50]),
      },
      {
        name: 'p90',
        type: 'line' as const,
        ...baseLineStyle,
        lineStyle: { color: C.accent, width: 1 },
        data: ticks.value.map(t => [t.time, t.p90]),
      },
      {
        name: 'p99',
        type: 'line' as const,
        ...baseLineStyle,
        lineStyle: { color: C.danger, width: 1 },
        data: ticks.value.map(t => [t.time, t.p99]),
      },
    ],
  }))

  const errorRateOption = computed(() => ({
    backgroundColor: 'transparent',
    tooltip: { trigger: 'axis' as const },
    legend: { data: ['Error Rate %'], textStyle: { color: C.textSec, fontSize: 10 } },
    grid: baseGrid,
    xAxis: baseXAxis,
    yAxis: { type: 'value' as const, name: '%', max: 100, nameTextStyle: { color: C.textMut, fontSize: 9 } },
    series: [
      {
        name: 'Error Rate %',
        type: 'line' as const,
        ...baseLineStyle,
        lineStyle: { color: C.danger, width: 1.5 },
        areaStyle: { color: 'rgba(224, 85, 106, 0.1)' },
        data: ticks.value.map(t => [t.time, t.errorRate]),
      },
    ],
  }))

  const threadsOption = computed(() => ({
    backgroundColor: 'transparent',
    tooltip: { trigger: 'axis' as const },
    legend: { data: ['Active Threads'], textStyle: { color: C.textSec, fontSize: 10 } },
    grid: baseGrid,
    xAxis: baseXAxis,
    yAxis: { type: 'value' as const, name: 'threads', nameTextStyle: { color: C.textMut, fontSize: 9 } },
    series: [
      {
        name: 'Active Threads',
        type: 'bar' as const,
        data: ticks.value.map(t => [t.time, t.threads]),
        itemStyle: { color: C.purple },
      },
    ],
  }))

  return { throughputOption, responseTimeOption, errorRateOption, threadsOption }
}
