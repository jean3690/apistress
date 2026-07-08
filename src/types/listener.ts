import type { KeyValuePair } from './common'

export type ListenerType =
  | 'ViewResultsTree'
  | 'SummaryReport'
  | 'AggregateReport'
  | 'AggregateGraph'
  | 'ResponseTimeGraph'
  | 'GraphResults'

export interface ListenerConfig {
  id: string
  type: ListenerType
  name: string
  enabled: boolean
  config?: Record<string, unknown>
}

export interface SampleResult {
  id: string
  timestamp: number
  threadName: string
  samplerName: string
  label: string
  elapsed: number
  connectTime: number
  latency: number
  bytes: number
  sentBytes: number
  responseCode: string
  responseMessage: string
  success: boolean
  url: string
  method: string
  requestHeaders: KeyValuePair[]
  responseHeaders: KeyValuePair[]
  responseBody: string
  assertionResults: AssertionResult[]
  errorMessage?: string
  threadGroup: string
  groupThreads: number
  allThreads: number
}

export interface AssertionResult {
  name: string
  failure: boolean
  failureMessage: string
}

export interface AggregateStats {
  label: string
  count: number
  avg: number
  min: number
  max: number
  median: number
  p90: number
  p95: number
  p99: number
  errorRate: number
  throughput: number // requests/sec
  receivedKBPerSec: number
  sentKBPerSec: number
}
