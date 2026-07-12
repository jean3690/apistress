import type { TestElement, KeyValuePair } from './common'

export interface RegexExtractor extends TestElement {
  type: 'RegexExtractor'
  referenceName: string
  regex: string
  template: string
  matchNo: number
  defaultValue: string
  useHeaders: boolean
  useBody: boolean
}

export interface JsonExtractor extends TestElement {
  type: 'JsonExtractor'
  referenceName: string
  jsonPath: string
  defaultValue: string
}

export interface BoundaryExtractor extends TestElement {
  type: 'BoundaryExtractor'
  referenceName: string
  leftBoundary: string
  rightBoundary: string
  matchNo: number
  defaultValue: string
}

export interface UserParameters extends TestElement {
  type: 'UserParameters'
  parameters: KeyValuePair[]
}

export type ProcessorUnion = RegexExtractor | JsonExtractor | BoundaryExtractor | UserParameters

export function createDefaultRegexExtractor(id: string): RegexExtractor {
  return { id, type: 'RegexExtractor', name: 'Regex Extractor', enabled: true, referenceName: '', regex: '', template: '$1', matchNo: 1, defaultValue: '', useHeaders: false, useBody: true }
}

export function createDefaultJsonExtractor(id: string): JsonExtractor {
  return { id, type: 'JsonExtractor', name: 'JSON Extractor', enabled: true, referenceName: '', jsonPath: '$', defaultValue: '' }
}

export function createDefaultBoundaryExtractor(id: string): BoundaryExtractor {
  return { id, type: 'BoundaryExtractor', name: 'Boundary Extractor', enabled: true, referenceName: '', leftBoundary: '', rightBoundary: '', matchNo: 1, defaultValue: '' }
}

export function createDefaultUserParameters(id: string): UserParameters {
  return { id, type: 'UserParameters', name: 'User Parameters', enabled: true, parameters: [] }
}
