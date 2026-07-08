import type { TestElement } from './common'

export interface ResponseAssertion extends TestElement {
  type: 'ResponseAssertion'
  testField: 'responseCode' | 'responseMessage' | 'responseBody' | 'responseHeaders' | 'requestHeaders' | 'url'
  patternMatching: 'contains' | 'matches' | 'equals' | 'substring' | 'notContains' | 'notMatches'
  patterns: string[]
  assumeSuccess: boolean
}

export interface JsonAssertion extends TestElement {
  type: 'JsonAssertion'
  jsonPath: string
  expectedValue: string
  comparisonMode: 'equals' | 'notEquals' | 'contains' | 'regex' | 'exists' | 'notExists'
  expectNull: boolean
}

export interface DurationAssertion extends TestElement {
  type: 'DurationAssertion'
  maxDuration: number // ms
}

export type AssertionUnion = ResponseAssertion | JsonAssertion | DurationAssertion

export function createDefaultResponseAssertion(id: string): ResponseAssertion {
  return {
    id,
    type: 'ResponseAssertion',
    name: 'Response Assertion',
    enabled: true,
    testField: 'responseBody',
    patternMatching: 'contains',
    patterns: [''],
    assumeSuccess: false,
  }
}

export function createDefaultJsonAssertion(id: string): JsonAssertion {
  return {
    id,
    type: 'JsonAssertion',
    name: 'JSON Assertion',
    enabled: true,
    jsonPath: '$',
    expectedValue: '',
    comparisonMode: 'exists',
    expectNull: false,
  }
}

export function createDefaultDurationAssertion(id: string): DurationAssertion {
  return {
    id,
    type: 'DurationAssertion',
    name: 'Duration Assertion',
    enabled: true,
    maxDuration: 3000,
  }
}
