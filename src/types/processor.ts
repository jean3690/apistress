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
