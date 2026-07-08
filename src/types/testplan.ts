import type { HttpSampler } from './sampler'
import type { ControllerUnion } from './controller'
import type { AssertionUnion } from './assertion'
import type { ConfigUnion } from './config'
import type { TimerUnion } from './timer'
import type { ProcessorUnion } from './processor'
import type { ListenerConfig, SampleResult } from './listener'
import type { OnErrorAction, KeyValuePair } from './common'

/** ThreadGroup 中 children 可包含的元素类型 */
export type ChildElement =
  | HttpSampler
  | ControllerUnion
  | AssertionUnion
  | ConfigUnion
  | TimerUnion
  | ProcessorUnion
  | ListenerConfig

/** 所有 TestElement 的联合类型 */
export type TestElementUnion = ChildElement | ThreadGroup | TestPlan

export interface ThreadGroup {
  id: string
  type: 'ThreadGroup'
  name: string
  enabled: boolean
  comments?: string
  numThreads: number
  rampUp: number // seconds
  loops: number // -1 = forever
  duration: number // seconds, 0 = unlimited
  delay: number // seconds
  scheduler: boolean
  onErrorAction: OnErrorAction
  sameUserOnEachIteration: boolean
  children: ChildElement[]
}

export function createDefaultThreadGroup(id: string): ThreadGroup {
  return {
    id,
    type: 'ThreadGroup',
    name: 'Thread Group',
    enabled: true,
    numThreads: 10,
    rampUp: 5,
    loops: 1,
    duration: 0,
    delay: 0,
    scheduler: false,
    onErrorAction: 'continue',
    sameUserOnEachIteration: true,
    children: [],
  }
}

export interface TestPlan {
  id: string
  type: 'TestPlan'
  name: string
  enabled: boolean
  comments: string
  threadGroups: ThreadGroup[]
  variables: KeyValuePair[]
  listeners: ListenerConfig[]
  results: SampleResult[]
}

export function createDefaultTestPlan(): TestPlan {
  const planId = crypto.randomUUID()
  const tgId = crypto.randomUUID()

  const defaultSampler: HttpSampler = {
    id: crypto.randomUUID(),
    type: 'HttpSampler',
    name: 'GET Example API',
    enabled: true,
    protocol: 'https',
    domain: 'jsonplaceholder.typicode.com',
    port: 443,
    path: '/posts/1',
    method: 'GET',
    headers: [{ key: 'Accept', value: 'application/json' }],
    queryParams: [],
    body: { mode: 'none' },
    auth: { type: 'none' },
    followRedirects: true,
    timeout: 30000,
    useKeepAlive: true,
  }

  const postSampler: HttpSampler = {
    id: crypto.randomUUID(),
    type: 'HttpSampler',
    name: 'POST Example API',
    enabled: true,
    protocol: 'https',
    domain: 'jsonplaceholder.typicode.com',
    port: 443,
    path: '/posts',
    method: 'POST',
    headers: [
      { key: 'Content-Type', value: 'application/json' },
      { key: 'Accept', value: 'application/json' },
    ],
    queryParams: [],
    body: {
      mode: 'raw',
      raw: '{\n  "title": "foo",\n  "body": "bar",\n  "userId": 1\n}',
      contentType: 'application/json',
    },
    auth: { type: 'none' },
    followRedirects: true,
    timeout: 30000,
    useKeepAlive: true,
  }

  return {
    id: planId,
    type: 'TestPlan',
    name: 'Sample Test Plan',
    enabled: true,
    comments: 'A sample test plan with example HTTP requests',
    threadGroups: [
      {
        id: tgId,
        type: 'ThreadGroup' as const,
        name: 'Sample Thread Group',
        enabled: true,
        numThreads: 5,
        rampUp: 2,
        loops: 10,
        duration: 0,
        delay: 0,
        scheduler: false,
        onErrorAction: 'continue',
        sameUserOnEachIteration: true,
        children: [
          defaultSampler,
          postSampler,
        ],
      },
    ],
    variables: [],
    listeners: [
      { id: crypto.randomUUID(), type: 'ViewResultsTree', name: 'View Results Tree', enabled: true },
      { id: crypto.randomUUID(), type: 'SummaryReport', name: 'Summary Report', enabled: true },
    ],
    results: [],
  }
}
