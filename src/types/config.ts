import type { TestElement, KeyValuePair } from './common'

export interface HttpDefaults extends TestElement {
  type: 'HttpDefaults'
  protocol: string
  domain: string
  port: number
  path: string
  headers: KeyValuePair[]
}

export interface CsvDataSet extends TestElement {
  type: 'CsvDataSet'
  filename: string
  variableNames: string
  delimiter: string
  recycleOnEof: boolean
  stopThreadOnEof: boolean
  ignoreFirstLine: boolean
}

export interface UserVariables extends TestElement {
  type: 'UserVariables'
  variables: KeyValuePair[]
}

export type ConfigUnion = HttpDefaults | CsvDataSet | UserVariables
