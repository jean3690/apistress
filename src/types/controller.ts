import type { TestElement, TestElementUnion } from './index'

export interface LoopController extends TestElement {
  type: 'LoopController'
  loops: number // -1 = forever
  children: TestElementUnion[]
}

export interface IfController extends TestElement {
  type: 'IfController'
  condition: string
  useExpression: boolean
  evaluateAll: boolean
  children: TestElementUnion[]
}

export interface WhileController extends TestElement {
  type: 'WhileController'
  condition: string
  children: TestElementUnion[]
}

export interface TransactionController extends TestElement {
  type: 'TransactionController'
  includeDuration: boolean
  children: TestElementUnion[]
}

export interface ThroughputController extends TestElement {
  type: 'ThroughputController'
  throughput: number // 每分钟执行次数
  perThread: boolean
  children: TestElementUnion[]
}

export type ControllerUnion =
  | LoopController
  | IfController
  | WhileController
  | TransactionController
  | ThroughputController

export function createDefaultLoopController(id: string): LoopController {
  return {
    id,
    type: 'LoopController',
    name: 'Loop Controller',
    enabled: true,
    loops: 1,
    children: [],
  }
}

export function createDefaultIfController(id: string): IfController {
  return {
    id,
    type: 'IfController',
    name: 'If Controller',
    enabled: true,
    condition: '',
    useExpression: true,
    evaluateAll: false,
    children: [],
  }
}

export function createDefaultWhileController(id: string): WhileController {
  return {
    id,
    type: 'WhileController',
    name: 'While Controller',
    enabled: true,
    condition: '',
    children: [],
  }
}

export function createDefaultTransactionController(id: string): TransactionController {
  return {
    id,
    type: 'TransactionController',
    name: 'Transaction Controller',
    enabled: true,
    includeDuration: false,
    children: [],
  }
}

export function createDefaultThroughputController(id: string): ThroughputController {
  return {
    id,
    type: 'ThroughputController',
    name: 'Throughput Controller',
    enabled: true,
    throughput: 60,
    perThread: false,
    children: [],
  }
}
