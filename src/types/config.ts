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

export function createDefaultHttpDefaults(id: string): HttpDefaults {
  return { id, type: 'HttpDefaults', name: 'HTTP Request Defaults', enabled: true, protocol: 'https', domain: '', port: 443, path: '/', headers: [] }
}

export function createDefaultCsvDataSet(id: string): CsvDataSet {
  return { id, type: 'CsvDataSet', name: 'CSV Data Set Config', enabled: true, filename: '', variableNames: '', delimiter: ',', recycleOnEof: true, stopThreadOnEof: false, ignoreFirstLine: true }
}

export function createDefaultUserVariables(id: string): UserVariables {
  return { id, type: 'UserVariables', name: 'User Defined Variables', enabled: true, variables: [] }
}
