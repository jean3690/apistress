<script setup lang="ts">
import CodeMirrorEditor from '@/components/editors/CodeMirrorEditor.vue'
import type {
  HttpSampler,
  GraphQlSampler,
  SseSampler,
  MqttSampler,
  WebSocketSampler,
  GrpcSampler,
  TcpSampler,
  RedisSampler,
} from '@/types'

defineProps<{
  node:
    HttpSampler | GraphQlSampler | SseSampler | MqttSampler | WebSocketSampler | GrpcSampler | TcpSampler | RedisSampler
}>()

const emit = defineEmits<{
  update: [key: string, value: unknown]
  updateNested: [path: string[], value: unknown]
}>()

function update(key: string, value: unknown) {
  emit('update', key, value)
}
function updateNested(path: string[], value: unknown) {
  emit('updateNested', path, value)
}
</script>

<template>
  <datalist id="http-headers">
    <option value="Accept" />
    <option value="Accept-Encoding" />
    <option value="Accept-Language" />
    <option value="Authorization" />
    <option value="Cache-Control" />
    <option value="Connection" />
    <option value="Content-Encoding" />
    <option value="Content-Length" />
    <option value="Content-Type" />
    <option value="Cookie" />
    <option value="Host" />
    <option value="Origin" />
    <option value="Referer" />
    <option value="User-Agent" />
    <option value="X-Request-ID" />
    <option value="X-API-Key" />
    <option value="X-Correlation-ID" />
  </datalist>

  <!-- HttpSampler -->
  <template v-if="node.type === 'HttpSampler'">
    <div class="prop-row">
      <label class="pp-label">Protocol</label>
      <select
        class="pp-field"
        :value="(node as HttpSampler).protocol"
        @change="update('protocol', ($event.target as HTMLSelectElement).value)"
      >
        <option value="https">HTTPS</option>
        <option value="http">HTTP</option>
      </select>
      <label class="pp-label" style="margin-left: 8px">Method</label>
      <select
        class="pp-field"
        :value="(node as HttpSampler).method"
        @change="update('method', ($event.target as HTMLSelectElement).value)"
      >
        <option value="GET">GET</option>
        <option value="POST">POST</option>
        <option value="PUT">PUT</option>
        <option value="DELETE">DELETE</option>
        <option value="PATCH">PATCH</option>
        <option value="HEAD">HEAD</option>
        <option value="OPTIONS">OPTIONS</option>
      </select>
    </div>
    <div class="prop-row">
      <label class="pp-label">Domain</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as HttpSampler).domain"
        @input="update('domain', ($event.target as HTMLInputElement).value)"
        placeholder="api.example.com"
      />
      <label class="pp-label" style="margin-left: 8px">Port</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as HttpSampler).port"
        style="width: 70px"
        @input="update('port', parseInt(($event.target as HTMLInputElement).value) || 443)"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Path</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as HttpSampler).path"
        @input="update('path', ($event.target as HTMLInputElement).value)"
        placeholder="/api/v1/resource"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Timeout (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as HttpSampler).timeout"
        min="0"
        @input="update('timeout', parseInt(($event.target as HTMLInputElement).value) || 30000)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Follow Redirects</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as HttpSampler).followRedirects"
        @change="update('followRedirects', ($event.target as HTMLInputElement).checked)"
      />
      <label class="pp-label" style="margin-left: 12px">Keep-Alive</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as HttpSampler).useKeepAlive"
        @change="update('useKeepAlive', ($event.target as HTMLInputElement).checked)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Retry Count</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as HttpSampler).retryCount"
        min="0"
        @input="update('retryCount', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
      <label class="pp-label" style="margin-left: 8px">Delay (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as HttpSampler).retryDelay"
        min="0"
        style="width: 70px"
        @input="update('retryDelay', parseInt(($event.target as HTMLInputElement).value) || 1000)"
      />
    </div>
    <!-- Auth -->
    <div class="prop-section">
      <div class="section-title">Authentication</div>
      <select
        class="pp-field"
        :value="(node as HttpSampler).auth?.type || 'none'"
        @change="
          update('auth', {
            type: ($event.target as HTMLSelectElement).value,
            username: (node as HttpSampler).auth?.username || '',
            password: (node as HttpSampler).auth?.password || '',
            token: (node as HttpSampler).auth?.token || '',
          })
        "
      >
        <option value="none">No Auth</option>
        <option value="basic">Basic Auth</option>
        <option value="bearer">Bearer Token</option>
      </select>
      <template v-if="(node as HttpSampler).auth?.type === 'basic'">
        <div class="prop-row">
          <label class="pp-label">Username</label>
          <input
            class="pp-field"
            type="text"
            :value="(node as HttpSampler).auth?.username || ''"
            @input="
              update('auth', { ...(node as HttpSampler).auth, username: ($event.target as HTMLInputElement).value })
            "
          />
        </div>
        <div class="prop-row">
          <label class="pp-label">Password</label>
          <input
            class="pp-field"
            type="password"
            :value="(node as HttpSampler).auth?.password || ''"
            @input="
              update('auth', { ...(node as HttpSampler).auth, password: ($event.target as HTMLInputElement).value })
            "
          />
        </div>
      </template>
      <template v-if="(node as HttpSampler).auth?.type === 'bearer'">
        <div class="prop-row col">
          <label class="pp-label">Token</label>
          <input
            class="pp-field"
            type="text"
            :value="(node as HttpSampler).auth?.token || ''"
            @input="update('auth', { ...(node as HttpSampler).auth, token: ($event.target as HTMLInputElement).value })"
          />
        </div>
      </template>
    </div>
    <!-- Body -->
    <div class="prop-section">
      <div class="section-title">Body</div>
      <select
        class="pp-field"
        :value="(node as HttpSampler).body.mode"
        @change="updateNested(['body', 'mode'], ($event.target as HTMLSelectElement).value)"
      >
        <option value="none">None</option>
        <option value="raw">Raw</option>
        <option value="form-data">Form Data</option>
        <option value="x-www-form-urlencoded">x-www-form-urlencoded</option>
      </select>
      <template v-if="(node as HttpSampler).body.mode === 'raw'">
        <div class="prop-row col">
          <label class="pp-label">Content-Type</label>
          <input
            class="pp-field"
            type="text"
            :value="(node as HttpSampler).body.contentType || ''"
            @input="updateNested(['body', 'contentType'], ($event.target as HTMLInputElement).value)"
            placeholder="application/json"
          />
        </div>
        <div class="prop-row col">
          <label class="pp-label">Body</label>
          <CodeMirrorEditor
            :model-value="(node as HttpSampler).body.raw || ''"
            @update:model-value="(v: string) => updateNested(['body', 'raw'], v)"
          />
        </div>
      </template>
    </div>
    <!-- Headers -->
    <div class="prop-section">
      <div class="section-title">Headers</div>
      <div v-for="(h, i) in (node as HttpSampler).headers" :key="i" class="kv-row">
        <input
          type="text"
          :value="h.key"
          placeholder="Name"
          class="kv-key"
          list="http-headers"
          @input="
            h.key = ($event.target as HTMLInputElement).value
            update('headers', [...(node as HttpSampler).headers])
          "
        />
        <input
          type="text"
          :value="h.value"
          placeholder="Value"
          class="kv-value"
          @input="
            h.value = ($event.target as HTMLInputElement).value
            update('headers', [...(node as HttpSampler).headers])
          "
        />
        <button
          class="kv-remove"
          @click="
            () => {
              const hdrs = [...(node as HttpSampler).headers]
              hdrs.splice(i, 1)
              update('headers', hdrs)
            }
          "
        >
          x
        </button>
      </div>
      <button
        class="kv-add"
        @click="
          () => {
            const hdrs = [...(node as HttpSampler).headers]
            hdrs.push({ key: '', value: '' })
            update('headers', hdrs)
          }
        "
      >
        + Add Header
      </button>
    </div>
    <!-- Query Params -->
    <div class="prop-section">
      <div class="section-title">Query Parameters</div>
      <div v-for="(p, i) in (node as HttpSampler).queryParams" :key="i" class="kv-row">
        <input
          type="text"
          :value="p.key"
          placeholder="Name"
          class="kv-key"
          @input="
            p.key = ($event.target as HTMLInputElement).value
            update('queryParams', [...(node as HttpSampler).queryParams])
          "
        />
        <input
          type="text"
          :value="p.value"
          placeholder="Value"
          class="kv-value"
          @input="
            p.value = ($event.target as HTMLInputElement).value
            update('queryParams', [...(node as HttpSampler).queryParams])
          "
        />
        <button
          class="kv-remove"
          @click="
            () => {
              const qp = [...(node as HttpSampler).queryParams]
              qp.splice(i, 1)
              update('queryParams', qp)
            }
          "
        >
          x
        </button>
      </div>
      <button
        class="kv-add"
        @click="
          () => {
            const qp = [...(node as HttpSampler).queryParams]
            qp.push({ key: '', value: '' })
            update('queryParams', qp)
          }
        "
      >
        + Add Query Param
      </button>
    </div>
  </template>

  <!-- GraphQL Sampler -->
  <template v-if="node.type === 'GraphQlSampler'">
    <div class="prop-row col">
      <label class="pp-label">URL</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as GraphQlSampler).url"
        @input="update('url', ($event.target as HTMLInputElement).value)"
        placeholder="https://api.example.com/graphql"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Timeout (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as GraphQlSampler).timeout"
        @input="update('timeout', parseInt(($event.target as HTMLInputElement).value) || 30000)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Retry Count</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as GraphQlSampler).retryCount"
        min="0"
        @input="update('retryCount', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
      <label class="pp-label" style="margin-left: 8px">Delay (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as GraphQlSampler).retryDelay"
        min="0"
        style="width: 70px"
        @input="update('retryDelay', parseInt(($event.target as HTMLInputElement).value) || 1000)"
      />
    </div>
    <div class="prop-section">
      <div class="section-title">Query</div>
      <CodeMirrorEditor
        language="graphql"
        :model-value="(node as GraphQlSampler).query"
        @update:model-value="update('query', $event)"
      />
    </div>
    <div class="prop-section">
      <div class="section-title">Variables</div>
      <CodeMirrorEditor
        :model-value="(node as GraphQlSampler).variables"
        @update:model-value="update('variables', $event)"
      />
    </div>
    <div class="prop-section">
      <div class="section-title">Headers</div>
      <div v-for="(h, i) in (node as GraphQlSampler).headers" :key="i" class="kv-row">
        <input
          type="text"
          :value="h.key"
          placeholder="Name"
          class="kv-key"
          list="http-headers"
          @input="
            h.key = ($event.target as HTMLInputElement).value
            update('headers', [...(node as GraphQlSampler).headers])
          "
        />
        <input
          type="text"
          :value="h.value"
          placeholder="Value"
          class="kv-value"
          @input="
            h.value = ($event.target as HTMLInputElement).value
            update('headers', [...(node as GraphQlSampler).headers])
          "
        />
        <button
          class="kv-remove"
          @click="
            () => {
              const hdrs = [...(node as GraphQlSampler).headers]
              hdrs.splice(i, 1)
              update('headers', hdrs)
            }
          "
        >
          x
        </button>
      </div>
      <button
        class="kv-add"
        @click="
          () => {
            const hdrs = [...(node as GraphQlSampler).headers]
            hdrs.push({ key: '', value: '' })
            update('headers', hdrs)
          }
        "
      >
        + Add Header
      </button>
    </div>
  </template>

  <!-- SSE / MQTT / WebSocket / gRPC / TCP / Redis -->
  <template v-if="node.type === 'SseSampler'">
    <div class="prop-row col">
      <label class="pp-label">URL</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as SseSampler).url"
        @input="update('url', ($event.target as HTMLInputElement).value)"
        placeholder="https://api.example.com/events"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Timeout (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as SseSampler).timeout"
        @input="update('timeout', parseInt(($event.target as HTMLInputElement).value) || 30000)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Max Events</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as SseSampler).maxEvents"
        min="0"
        @input="update('maxEvents', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
      <span class="prop-hint">0 = unlimited</span>
    </div>
    <div class="prop-row">
      <label class="pp-label">Retry Count</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as SseSampler).retryCount"
        min="0"
        @input="update('retryCount', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
      <label class="pp-label" style="margin-left: 8px">Delay (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as SseSampler).retryDelay"
        min="0"
        style="width: 70px"
        @input="update('retryDelay', parseInt(($event.target as HTMLInputElement).value) || 1000)"
      />
    </div>
    <div class="prop-section">
      <div class="section-title">Headers</div>
      <div v-for="(h, i) in (node as SseSampler).headers" :key="i" class="kv-row">
        <input
          type="text"
          :value="h.key"
          placeholder="Name"
          class="kv-key"
          list="http-headers"
          @input="
            h.key = ($event.target as HTMLInputElement).value
            update('headers', [...(node as SseSampler).headers])
          "
        />
        <input
          type="text"
          :value="h.value"
          placeholder="Value"
          class="kv-value"
          @input="
            h.value = ($event.target as HTMLInputElement).value
            update('headers', [...(node as SseSampler).headers])
          "
        />
        <button
          class="kv-remove"
          @click="
            () => {
              const hdrs = [...(node as SseSampler).headers]
              hdrs.splice(i, 1)
              update('headers', hdrs)
            }
          "
        >
          x
        </button>
      </div>
      <button
        class="kv-add"
        @click="
          () => {
            const hdrs = [...(node as SseSampler).headers]
            hdrs.push({ key: '', value: '' })
            update('headers', hdrs)
          }
        "
      >
        + Add Header
      </button>
    </div>
  </template>

  <template v-if="node.type === 'MqttSampler'">
    <div class="prop-row col">
      <label class="pp-label">Broker URL</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as MqttSampler).brokerUrl"
        @input="update('brokerUrl', ($event.target as HTMLInputElement).value)"
        placeholder="tcp://localhost:1883"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Client ID</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as MqttSampler).clientId"
        @input="update('clientId', ($event.target as HTMLInputElement).value)"
        placeholder="apistress-client"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Topic</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as MqttSampler).topic"
        @input="update('topic', ($event.target as HTMLInputElement).value)"
        placeholder="test/topic"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">QoS</label>
      <select
        class="pp-field"
        :value="(node as MqttSampler).qos"
        @change="update('qos', parseInt(($event.target as HTMLSelectElement).value))"
      >
        <option :value="0">0 - At Most Once</option>
        <option :value="1">1 - At Least Once</option>
        <option :value="2">2 - Exactly Once</option>
      </select>
    </div>
    <div class="prop-row">
      <label class="pp-label">Mode</label>
      <select
        class="pp-field"
        :value="(node as MqttSampler).mode"
        @change="update('mode', ($event.target as HTMLSelectElement).value)"
      >
        <option value="publish">Publish Only</option>
        <option value="pubsub">Publish &amp; Subscribe</option>
      </select>
    </div>
    <div class="prop-row">
      <label class="pp-label">Timeout (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as MqttSampler).timeout"
        @input="update('timeout', parseInt(($event.target as HTMLInputElement).value) || 30000)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Retry Count</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as MqttSampler).retryCount"
        min="0"
        @input="update('retryCount', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
      <label class="pp-label" style="margin-left: 8px">Delay (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as MqttSampler).retryDelay"
        min="0"
        style="width: 70px"
        @input="update('retryDelay', parseInt(($event.target as HTMLInputElement).value) || 1000)"
      />
    </div>
    <div class="prop-section">
      <div class="section-title">Message Payload</div>
      <CodeMirrorEditor :model-value="(node as MqttSampler).message" @update:model-value="update('message', $event)" />
    </div>
  </template>

  <template v-if="node.type === 'WebSocketSampler'">
    <div class="prop-row col">
      <label class="pp-label">URL</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as WebSocketSampler).url"
        @input="update('url', ($event.target as HTMLInputElement).value)"
        placeholder="wss://echo.example.com/ws"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Mode</label>
      <select
        class="pp-field"
        :value="(node as WebSocketSampler).mode"
        @change="update('mode', ($event.target as HTMLSelectElement).value)"
      >
        <option value="connect">Connect Only</option>
        <option value="sendReceive">Send &amp; Receive</option>
        <option value="keepAlive">Keep Alive</option>
      </select>
    </div>
    <div class="prop-row">
      <label class="pp-label">Timeout (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as WebSocketSampler).timeout"
        @input="update('timeout', parseInt(($event.target as HTMLInputElement).value) || 30000)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Retry Count</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as WebSocketSampler).retryCount"
        min="0"
        @input="update('retryCount', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
      <label class="pp-label" style="margin-left: 8px">Delay (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as WebSocketSampler).retryDelay"
        min="0"
        style="width: 70px"
        @input="update('retryDelay', parseInt(($event.target as HTMLInputElement).value) || 1000)"
      />
    </div>
    <div class="prop-section">
      <div class="section-title">Headers</div>
      <div v-for="(h, i) in (node as WebSocketSampler).headers" :key="i" class="kv-row">
        <input
          type="text"
          :value="h.key"
          placeholder="Name"
          class="kv-key"
          list="http-headers"
          @input="
            h.key = ($event.target as HTMLInputElement).value
            update('headers', [...(node as WebSocketSampler).headers])
          "
        />
        <input
          type="text"
          :value="h.value"
          placeholder="Value"
          class="kv-value"
          @input="
            h.value = ($event.target as HTMLInputElement).value
            update('headers', [...(node as WebSocketSampler).headers])
          "
        />
        <button
          class="kv-remove"
          @click="
            () => {
              const hdrs = [...(node as WebSocketSampler).headers]
              hdrs.splice(i, 1)
              update('headers', hdrs)
            }
          "
        >
          x
        </button>
      </div>
      <button
        class="kv-add"
        @click="
          () => {
            const hdrs = [...(node as WebSocketSampler).headers]
            hdrs.push({ key: '', value: '' })
            update('headers', hdrs)
          }
        "
      >
        + Add Header
      </button>
    </div>
    <div class="prop-section">
      <div class="section-title">Message</div>
      <CodeMirrorEditor
        :model-value="(node as WebSocketSampler).message"
        @update:model-value="update('message', $event)"
      />
    </div>
  </template>

  <template v-if="node.type === 'GrpcSampler'">
    <div class="prop-row col">
      <label class="pp-label">Endpoint</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as GrpcSampler).endpoint"
        @input="update('endpoint', ($event.target as HTMLInputElement).value)"
        placeholder="http://localhost:50051"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Service Name</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as GrpcSampler).serviceName"
        @input="update('serviceName', ($event.target as HTMLInputElement).value)"
        placeholder="greeter.Greeter"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Method Name</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as GrpcSampler).methodName"
        @input="update('methodName', ($event.target as HTMLInputElement).value)"
        placeholder="SayHello"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Timeout (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as GrpcSampler).timeout"
        @input="update('timeout', parseInt(($event.target as HTMLInputElement).value) || 30000)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Use TLS</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as GrpcSampler).useTls"
        @change="update('useTls', ($event.target as HTMLInputElement).checked)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Retry Count</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as GrpcSampler).retryCount"
        min="0"
        @input="update('retryCount', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
      <label class="pp-label" style="margin-left: 8px">Delay (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as GrpcSampler).retryDelay"
        min="0"
        style="width: 70px"
        @input="update('retryDelay', parseInt(($event.target as HTMLInputElement).value) || 1000)"
      />
    </div>
    <div class="prop-section">
      <div class="section-title">Metadata</div>
      <div v-for="(h, i) in (node as GrpcSampler).metadata" :key="i" class="kv-row">
        <input
          type="text"
          :value="h.key"
          placeholder="Name"
          class="kv-key"
          list="http-headers"
          @input="
            h.key = ($event.target as HTMLInputElement).value
            update('metadata', [...(node as GrpcSampler).metadata])
          "
        />
        <input
          type="text"
          :value="h.value"
          placeholder="Value"
          class="kv-value"
          @input="
            h.value = ($event.target as HTMLInputElement).value
            update('metadata', [...(node as GrpcSampler).metadata])
          "
        />
        <button
          class="kv-remove"
          @click="
            () => {
              const hdrs = [...(node as GrpcSampler).metadata]
              hdrs.splice(i, 1)
              update('metadata', hdrs)
            }
          "
        >
          x
        </button>
      </div>
      <button
        class="kv-add"
        @click="
          () => {
            const hdrs = [...(node as GrpcSampler).metadata]
            hdrs.push({ key: '', value: '' })
            update('metadata', hdrs)
          }
        "
      >
        + Add Metadata
      </button>
    </div>
    <div class="prop-section">
      <div class="section-title">Request JSON</div>
      <CodeMirrorEditor
        :model-value="(node as GrpcSampler).requestJson"
        @update:model-value="update('requestJson', $event)"
      />
    </div>
  </template>

  <template v-if="node.type === 'TcpSampler'">
    <div class="prop-row">
      <label class="pp-label">Host</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as TcpSampler).host"
        @input="update('host', ($event.target as HTMLInputElement).value)"
        placeholder="localhost"
      />
      <label class="pp-label" style="margin-left: 8px">Port</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as TcpSampler).port"
        style="width: 80px"
        @input="update('port', parseInt(($event.target as HTMLInputElement).value) || 8080)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Payload Type</label>
      <select
        class="pp-field"
        :value="(node as TcpSampler).payloadType"
        @change="update('payloadType', ($event.target as HTMLSelectElement).value)"
      >
        <option value="text">Text</option>
        <option value="hex">Hex</option>
      </select>
    </div>
    <div class="prop-row">
      <label class="pp-label">Timeout (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as TcpSampler).timeout"
        @input="update('timeout', parseInt(($event.target as HTMLInputElement).value) || 30000)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Retry Count</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as TcpSampler).retryCount"
        min="0"
        @input="update('retryCount', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
      <label class="pp-label" style="margin-left: 8px">Delay (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as TcpSampler).retryDelay"
        min="0"
        style="width: 70px"
        @input="update('retryDelay', parseInt(($event.target as HTMLInputElement).value) || 1000)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">EOL Byte</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as TcpSampler).eolByte"
        min="0"
        max="255"
        @input="update('eolByte', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
      <span class="prop-hint">10 = \n, 0 = none</span>
    </div>
    <div class="prop-section">
      <div class="section-title">Payload</div>
      <textarea
        class="pp-textarea"
        :value="(node as TcpSampler).payload"
        @input="update('payload', ($event.target as HTMLTextAreaElement).value)"
        placeholder="Enter text or hex payload..."
        rows="4"
      />
    </div>
  </template>

  <template v-if="node.type === 'RedisSampler'">
    <div class="prop-row">
      <label class="pp-label">Host</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as RedisSampler).host"
        @input="update('host', ($event.target as HTMLInputElement).value)"
        placeholder="localhost"
      />
      <label class="pp-label" style="margin-left: 8px">Port</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as RedisSampler).port"
        style="width: 80px"
        @input="update('port', parseInt(($event.target as HTMLInputElement).value) || 6379)"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Password</label>
      <input
        class="pp-field"
        type="password"
        :value="(node as RedisSampler).password"
        @input="update('password', ($event.target as HTMLInputElement).value)"
        placeholder="(optional)"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Command</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as RedisSampler).command"
        @input="update('command', ($event.target as HTMLInputElement).value)"
        placeholder="GET mykey"
      />
      <span class="prop-hint">e.g. GET key, SET key val, PUBLISH ch msg</span>
    </div>
    <div class="prop-row">
      <label class="pp-label">Timeout (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as RedisSampler).timeout"
        @input="update('timeout', parseInt(($event.target as HTMLInputElement).value) || 30000)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Retry Count</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as RedisSampler).retryCount"
        min="0"
        @input="update('retryCount', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
      <label class="pp-label" style="margin-left: 8px">Delay (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as RedisSampler).retryDelay"
        min="0"
        style="width: 70px"
        @input="update('retryDelay', parseInt(($event.target as HTMLInputElement).value) || 1000)"
      />
    </div>
  </template>
</template>
