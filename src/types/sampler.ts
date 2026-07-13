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
  retryCount: number
  retryDelay: number
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
    retryCount: 0,
    retryDelay: 1000,
  }
}

// ---- GraphQL Sampler ----

export interface GraphQlSampler extends TestElement {
  type: 'GraphQlSampler'
  url: string
  query: string
  variables: string
  headers: KeyValuePair[]
  timeout: number
  retryCount: number
  retryDelay: number
}

export function createDefaultGraphQlSampler(id: string): GraphQlSampler {
  return {
    id,
    type: 'GraphQlSampler',
    name: 'GraphQL Request',
    enabled: true,
    url: '',
    query: '',
    variables: '{}',
    headers: [{ key: 'Content-Type', value: 'application/json' }],
    timeout: 30000,
    retryCount: 0,
    retryDelay: 1000,
  }
}

// ---- SSE Sampler ----

export interface SseSampler extends TestElement {
  type: 'SseSampler'
  url: string
  headers: KeyValuePair[]
  timeout: number
  maxEvents: number
  retryCount: number
  retryDelay: number
}

export function createDefaultSseSampler(id: string): SseSampler {
  return {
    id,
    type: 'SseSampler',
    name: 'SSE Stream',
    enabled: true,
    url: '',
    headers: [],
    timeout: 30000,
    maxEvents: 0,
    retryCount: 0,
    retryDelay: 1000,
  }
}

// ---- MQTT Sampler ----

export interface MqttSampler extends TestElement {
  type: 'MqttSampler'
  brokerUrl: string
  clientId: string
  topic: string
  qos: 0 | 1 | 2
  message: string
  timeout: number
  mode: 'publish' | 'pubsub'
  retryCount: number
  retryDelay: number
}

export function createDefaultMqttSampler(id: string): MqttSampler {
  return {
    id,
    type: 'MqttSampler',
    name: 'MQTT Publish',
    enabled: true,
    brokerUrl: 'tcp://localhost:1883',
    clientId: '',
    topic: '',
    qos: 0,
    message: '',
    timeout: 30000,
    mode: 'publish',
    retryCount: 0,
    retryDelay: 1000,
  }
}

// ---- WebSocket Sampler ----

export interface WebSocketSampler extends TestElement {
  type: 'WebSocketSampler'
  url: string
  headers: KeyValuePair[]
  message: string
  timeout: number
  mode: 'connect' | 'sendReceive' | 'keepAlive'
  retryCount: number
  retryDelay: number
}

export function createDefaultWebSocketSampler(id: string): WebSocketSampler {
  return {
    id,
    type: 'WebSocketSampler',
    name: 'WebSocket Request',
    enabled: true,
    url: 'wss://',
    headers: [],
    message: '',
    timeout: 30000,
    mode: 'sendReceive',
    retryCount: 0,
    retryDelay: 1000,
  }
}

// ---- gRPC Sampler ----

export interface GrpcSampler extends TestElement {
  type: 'GrpcSampler'
  endpoint: string
  serviceName: string
  methodName: string
  requestJson: string
  metadata: KeyValuePair[]
  timeout: number
  useTls: boolean
  retryCount: number
  retryDelay: number
}

export function createDefaultGrpcSampler(id: string): GrpcSampler {
  return {
    id,
    type: 'GrpcSampler',
    name: 'gRPC Request',
    enabled: true,
    endpoint: 'http://localhost:50051',
    serviceName: '',
    methodName: '',
    requestJson: '{}',
    metadata: [],
    timeout: 30000,
    useTls: false,
    retryCount: 0,
    retryDelay: 1000,
  }
}

// ---- TCP Sampler ----

export interface TcpSampler extends TestElement {
  type: 'TcpSampler'
  host: string
  port: number
  payload: string
  payloadType: 'text' | 'hex'
  timeout: number
  eolByte: number
  retryCount: number
  retryDelay: number
}

export function createDefaultTcpSampler(id: string): TcpSampler {
  return {
    id,
    type: 'TcpSampler',
    name: 'TCP Request',
    enabled: true,
    host: '',
    port: 8080,
    payload: '',
    payloadType: 'text',
    timeout: 30000,
    eolByte: 10,
    retryCount: 0,
    retryDelay: 1000,
  }
}

// ---- Redis Sampler ----

export interface RedisSampler extends TestElement {
  type: 'RedisSampler'
  host: string
  port: number
  password: string
  command: string
  timeout: number
  retryCount: number
  retryDelay: number
}

export function createDefaultRedisSampler(id: string): RedisSampler {
  return {
    id,
    type: 'RedisSampler',
    name: 'Redis Command',
    enabled: true,
    host: 'localhost',
    port: 6379,
    password: '',
    command: 'PING',
    timeout: 30000,
    retryCount: 0,
    retryDelay: 1000,
  }
}
