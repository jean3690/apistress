import type { TestElement, HttpMethod, HttpBody, KeyValuePair, HttpAuth } from './common'

export interface HttpSampler extends TestElement {
  type: 'HttpSampler'
  protocol: 'http' | 'https'
  domain: string
  port: number
  path: string
  method: HttpMethod
  headers: KeyValuePair[]
  queryParams: KeyValuePair[]
  body: HttpBody
  auth: HttpAuth
  followRedirects: boolean
  timeout: number
  useKeepAlive: boolean
}

export function createDefaultHttpSampler(id: string): HttpSampler {
  return {
    id,
    type: 'HttpSampler',
    name: 'HTTP Request',
    enabled: true,
    protocol: 'https',
    domain: '',
    port: 443,
    path: '/',
    method: 'GET',
    headers: [],
    queryParams: [],
    body: { mode: 'none' },
    auth: { type: 'none' },
    followRedirects: true,
    timeout: 30000,
    useKeepAlive: true,
  }
}
