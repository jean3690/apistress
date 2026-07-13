<script setup lang="ts">
import { shallowRef, ref, watch, computed } from 'vue'
import { useTestPlanStore } from '@/stores'
import { findNodeById } from '@/utils/tree-utils'
import CodeMirrorEditor from '@/components/editors/CodeMirrorEditor.vue'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Send, Save, Plus, X } from '@lucide/vue'
import type { HttpSampler, HttpBody, KeyValuePair, FormDataItem } from '@/types'

const testPlan = useTestPlanStore()

const props = defineProps<{
  samplerId: string | null
}>()

const emit = defineEmits<{
  response: [
    data: { status: string; statusCode?: number; body: string; headers: string; time?: number; size?: number } | null,
  ]
  'update:loading': [value: boolean]
  saved: []
}>()

// ---- Editing state ----
const method = shallowRef('GET')
const url = shallowRef('')
const reqHeaders = ref<KeyValuePair[]>([])
const reqBody = shallowRef('')
const reqBodyType = shallowRef<'none' | 'raw' | 'form' | 'form-data'>('none')
const reqFormData = ref<FormDataItem[]>([])
const reqContentType = shallowRef('application/json')
const reqName = shallowRef('')
const editorTab = shallowRef<'headers' | 'body' | 'auth' | 'settings'>('headers')
const loading = shallowRef(false)
const saveMessage = shallowRef('')
const authType = shallowRef<'none' | 'basic' | 'bearer'>('none')
const authUsername = shallowRef('')
const authPassword = shallowRef('')
const authToken = shallowRef('')

const bodyLanguage = computed(() => {
  const ct = reqContentType.value
  if (ct.includes('json')) return 'json' as const
  if (ct.includes('graphql')) return 'graphql' as const
  return 'text' as const
})

const timeout = shallowRef(30000)
const followRedirects = shallowRef(true)
const keepAlive = shallowRef(true)
const retryCount = shallowRef(0)
const retryDelay = shallowRef(1000)

function parseUrl(input: string): { protocol: string; domain: string; port: number; path: string } {
  try {
    const u = new URL(input.startsWith('http') ? input : 'https://' + input)
    return {
      protocol: u.protocol.replace(':', ''),
      domain: u.hostname,
      port: u.port ? parseInt(u.port) : u.protocol === 'https:' ? 443 : 80,
      path: u.pathname + u.search,
    }
  } catch {
    return { protocol: 'https', domain: '', port: 443, path: '/' }
  }
}

function loadSampler(id: string) {
  const node = findNodeById(testPlan.plan, id)
  if (!node || node.type !== 'HttpSampler') return
  const s = node as unknown as HttpSampler
  reqName.value = s.name
  method.value = s.method
  const scheme = s.protocol || 'https'
  const host = s.domain || 'localhost'
  const port = s.port ? `:${s.port}` : ''
  const path = s.path || '/'
  url.value = `${scheme}://${host}${port}${path}`
  reqHeaders.value = s.headers.map(h => ({ ...h }))
  if (s.body.mode === 'raw') {
    reqBodyType.value = 'raw'
    reqBody.value = s.body.raw || ''
    reqContentType.value = s.body.contentType || 'application/json'
  } else if (s.body.mode === 'x-www-form-urlencoded') {
    reqBodyType.value = 'form'
    reqBody.value = (s.body.urlEncoded || []).map(p => `${p.key}=${p.value}`).join('&')
  } else if (s.body.mode === 'form-data') {
    reqBodyType.value = 'form-data'
    reqFormData.value = (s.body.formData || []).map(f => ({ ...f }))
  } else {
    reqBodyType.value = 'none'
    reqBody.value = ''
  }
  // Auth
  const auth = (s as unknown as Record<string, unknown>).auth as Record<string, unknown> | undefined
  authType.value = (auth?.type as 'none' | 'basic' | 'bearer') || 'none'
  authUsername.value = (auth?.username as string) || ''
  authPassword.value = (auth?.password as string) || ''
  authToken.value = (auth?.token as string) || ''

  // Advanced settings
  timeout.value = ((s as unknown as Record<string, unknown>).timeout as number) || 30000
  followRedirects.value = (s as unknown as Record<string, unknown>).followRedirects !== false
  keepAlive.value = (s as unknown as Record<string, unknown>).useKeepAlive !== false
  retryCount.value = ((s as unknown as Record<string, unknown>).retryCount as number) || 0
  retryDelay.value = ((s as unknown as Record<string, unknown>).retryDelay as number) || 1000
}

watch(
  () => props.samplerId,
  id => {
    if (id) {
      loadSampler(id)
      emit('response', null)
    }
  },
)

// ---- Save to store ----
function saveRequest() {
  if (!props.samplerId) return
  const parsed = parseUrl(url.value)
  const body: HttpBody = {
    mode:
      reqBodyType.value === 'raw'
        ? 'raw'
        : reqBodyType.value === 'form'
          ? 'x-www-form-urlencoded'
          : reqBodyType.value === 'form-data'
            ? 'form-data'
            : 'none',
  }
  if (reqBodyType.value === 'raw') {
    body.raw = reqBody.value
    body.contentType = reqContentType.value
  } else if (reqBodyType.value === 'form') {
    body.urlEncoded = reqBody.value
      .split('&')
      .filter(Boolean)
      .map(pair => {
        const [k, ...v] = pair.split('=')
        return { key: decodeURIComponent(k), value: decodeURIComponent(v.join('=')) }
      })
  } else if (reqBodyType.value === 'form-data') {
    body.formData = reqFormData.value.filter(f => f.key)
  }

  testPlan.updateNode(props.samplerId, {
    name: reqName.value,
    protocol: parsed.protocol,
    domain: parsed.domain,
    port: parsed.port,
    path: parsed.path,
    method: method.value,
    headers: reqHeaders.value.filter(h => h.key),
    body,
    auth: {
      type: authType.value,
      username: authUsername.value,
      password: authPassword.value,
      token: authToken.value,
    },
    timeout: timeout.value,
    followRedirects: followRedirects.value,
    useKeepAlive: keepAlive.value,
    retryCount: retryCount.value,
    retryDelay: retryDelay.value,
  } as unknown as Record<string, unknown>)

  saveMessage.value = 'Saved'
  setTimeout(() => {
    saveMessage.value = ''
  }, 1500)
  emit('saved')
}

// ---- Send request ----
async function sendRequest() {
  loading.value = true
  emit('update:loading', true)
  emit('response', null)

  const controller = new AbortController()
  const timeoutId = setTimeout(() => controller.abort(), timeout.value)

  try {
    const headers: Record<string, string> = {}
    for (const h of reqHeaders.value) {
      if (h.key) headers[h.key] = h.value
    }
    const opts: RequestInit = { method: method.value, headers, signal: controller.signal }

    if (method.value !== 'GET' && method.value !== 'HEAD' && reqBodyType.value !== 'none') {
      if (reqBodyType.value === 'raw') {
        headers['Content-Type'] = reqContentType.value
        opts.body = reqBody.value
      } else if (reqBodyType.value === 'form-data') {
        const fd = new FormData()
        for (const item of reqFormData.value) {
          if (!item.key) continue
          if (item.type === 'file') {
            fd.append(
              item.key,
              new Blob([item.value], { type: item.mimeType || 'application/octet-stream' }),
              item.filename || 'file',
            )
          } else {
            fd.append(item.key, item.value)
          }
        }
        opts.body = fd
        // Don't set Content-Type — browser sets it with boundary
      } else {
        headers['Content-Type'] = 'application/x-www-form-urlencoded'
        opts.body = reqBody.value
      }
    }

    let doFetch: typeof fetch
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      const { fetch: tauriFetch } = await import('@tauri-apps/plugin-http')
      doFetch = tauriFetch
    } else {
      doFetch = window.fetch.bind(window)
    }

    const start = performance.now()
    const res = await doFetch(url.value, opts)
    const elapsed = Math.round(performance.now() - start)
    const resBody = await res.text()

    emit('response', {
      status: `${res.status} ${res.statusText}`,
      statusCode: res.status,
      body: tryFormatJson(resBody),
      headers: [...res.headers.entries()].map(([k, v]) => `${k}: ${v}`).join('\n'),
      time: elapsed,
      size: new Blob([resBody]).size,
    })
  } catch (e) {
    emit('response', { status: 'Error', statusCode: 0, body: String(e), headers: '', time: 0, size: 0 })
  } finally {
    clearTimeout(timeoutId)
    loading.value = false
    emit('update:loading', false)
  }
}

function tryFormatJson(text: string): string {
  try {
    return JSON.stringify(JSON.parse(text), null, 2)
  } catch {
    return text
  }
}
</script>

<template>
  <datalist id="http-headers-modern">
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

  <div class="flex flex-col overflow-hidden flex-1 min-w-300px bg-base">
    <div class="panel-header">
      <div class="flex items-center gap-2">
        <span>Request</span>
        <Input v-if="samplerId" v-model="reqName" class="w-[180px] h-7 text-xs font-semibold" @blur="saveRequest" />
      </div>
      <div class="flex items-center gap-2">
        <span v-if="saveMessage" class="text-accent-cool text-[11px] font-medium">{{ saveMessage }}</span>
        <Button v-if="samplerId" variant="outline" size="sm" class="h-7 gap-1 text-[11px]" @click="saveRequest">
          <Save class="size-3" /> Save
        </Button>
        <Button :disabled="loading || !url" size="sm" class="h-7 gap-1" @click="sendRequest">
          <Send class="size-3" /> {{ loading ? 'Sending...' : 'Send' }}
        </Button>
      </div>
    </div>
    <div class="flex-1 flex flex-col overflow-hidden">
      <div class="flex p-2.5 gap-2">
        <select
          v-model="method"
          class="w-[100px] px-2 py-1.5 border border-border rounded bg-surface text-foreground text-[13px] font-semibold cursor-pointer focus:border-primary outline-none"
        >
          <option>GET</option>
          <option>POST</option>
          <option>PUT</option>
          <option>DELETE</option>
          <option>PATCH</option>
          <option>HEAD</option>
          <option>OPTIONS</option>
        </select>
        <Input
          v-model="url"
          class="flex-1 font-mono text-[13px]"
          placeholder="https://api.example.com/endpoint"
          @keydown.enter="sendRequest"
        />
      </div>

      <div class="flex gap-0.5 px-2.5 border-b border-outline">
        <span :class="['tab', { active: editorTab === 'headers' }]" @click="editorTab = 'headers'">Headers</span>
        <span :class="['tab', { active: editorTab === 'body' }]" @click="editorTab = 'body'">Body</span>
        <span :class="['tab', { active: editorTab === 'auth' }]" @click="editorTab = 'auth'">Auth</span>
        <span :class="['tab', { active: editorTab === 'settings' }]" @click="editorTab = 'settings'">Settings</span>
      </div>

      <!-- Headers -->
      <div v-if="editorTab === 'headers'" class="flex-1 overflow-y-auto p-2 px-2.5">
        <div class="flex gap-1.5 mb-1" v-for="(h, i) in reqHeaders" :key="i">
          <input v-model="h.key" placeholder="Header name" class="kv-key" />
          <input v-model="h.value" placeholder="Header value" class="kv-value" />
          <button
            class="p-0.5 border-none bg-transparent text-danger cursor-pointer text-base opacity-50 hover:opacity-100 transition-opacity"
            @click="reqHeaders.splice(i, 1)"
          >
            <X class="size-3.5" />
          </button>
        </div>
        <Button
          variant="ghost"
          size="sm"
          class="mt-1.5 gap-1 text-muted-foreground border-dashed border"
          @click="reqHeaders.push({ key: '', value: '' })"
          ><Plus class="size-3.5" /> Add Header</Button
        >
      </div>

      <!-- Body -->
      <div v-if="editorTab === 'body'" class="flex-1 flex flex-col overflow-hidden">
        <div class="flex gap-0.5 pt-2 px-2.5">
          <span :class="['type-opt', { active: reqBodyType === 'none' }]" @click="reqBodyType = 'none'">None</span>
          <span :class="['type-opt', { active: reqBodyType === 'raw' }]" @click="reqBodyType = 'raw'">Raw</span>
          <span :class="['type-opt', { active: reqBodyType === 'form' }]" @click="reqBodyType = 'form'">Form</span>
          <span :class="['type-opt', { active: reqBodyType === 'form-data' }]" @click="reqBodyType = 'form-data'"
            >Form-Data</span
          >
        </div>
        <div v-if="reqBodyType === 'raw'" class="flex flex-col gap-1.5 p-2 px-2.5 flex-1">
          <Input v-model="reqContentType" class="text-xs" placeholder="Content-Type" />
          <CodeMirrorEditor :language="bodyLanguage" :model-value="reqBody" @update:model-value="reqBody = $event" />
        </div>
        <div v-else-if="reqBodyType === 'form'" class="p-2 px-2.5 flex-1">
          <textarea v-model="reqBody" class="body-textarea" rows="6" placeholder="key1=value1&key2=value2"></textarea>
        </div>
        <div v-else-if="reqBodyType === 'form-data'" class="flex-1 overflow-y-auto p-2 px-2.5">
          <div class="flex gap-1.5 mb-1" v-for="(item, i) in reqFormData" :key="i">
            <input v-model="item.key" placeholder="Name" class="kv-key" list="http-headers-modern" />
            <select v-model="item.type" class="kv-type">
              <option value="text">Text</option>
              <option value="file">File</option>
            </select>
            <input v-model="item.value" placeholder="Value/Path" class="kv-value" />
            <template v-if="item.type === 'file'">
              <input v-model="item.filename" placeholder="Filename" class="kv-key" />
              <input v-model="item.mimeType" placeholder="MIME" class="kv-key" />
            </template>
            <button
              class="p-0.5 border-none bg-transparent text-danger cursor-pointer text-base opacity-50 hover:opacity-100 transition-opacity"
              @click="reqFormData.splice(i, 1)"
            >
              <X class="size-3.5" />
            </button>
          </div>
          <Button
            variant="ghost"
            size="sm"
            class="mt-1.5 gap-1 text-muted-foreground border-dashed border"
            @click="reqFormData.push({ key: '', value: '', type: 'text' })"
            ><Plus class="size-3.5" /> Add Field</Button
          >
        </div>
        <div v-else class="py-8 px-2.5 text-center text-muted text-12px"><p>No body will be sent.</p></div>
      </div>

      <!-- Auth -->
      <div v-if="editorTab === 'auth'" class="p-4 px-2.5">
        <div class="prop-row">
          <label>Type</label>
          <select v-model="authType">
            <option value="none">None</option>
            <option value="basic">Basic Auth</option>
            <option value="bearer">Bearer Token</option>
          </select>
        </div>
        <template v-if="authType === 'basic'">
          <div class="prop-row">
            <label>Username</label>
            <input v-model="authUsername" type="text" placeholder="user" />
          </div>
          <div class="prop-row">
            <label>Password</label>
            <input v-model="authPassword" type="password" placeholder="***" />
          </div>
        </template>
        <template v-if="authType === 'bearer'">
          <div class="prop-row">
            <label>Token</label>
            <input v-model="authToken" type="text" placeholder="eyJ..." />
          </div>
        </template>
      </div>

      <!-- Settings -->
      <div v-if="editorTab === 'settings'" class="p-4 px-2.5">
        <div class="prop-row">
          <label>Timeout (ms)</label>
          <input v-model.number="timeout" type="number" min="0" step="1000" />
        </div>
        <div class="prop-row">
          <label>Follow Redirects</label>
          <input v-model="followRedirects" type="checkbox" />
        </div>
        <div class="prop-row">
          <label>Keep-Alive</label>
          <input v-model="keepAlive" type="checkbox" />
        </div>
        <div class="prop-row">
          <label>Retry Count</label>
          <input v-model.number="retryCount" type="number" min="0" max="10" />
        </div>
        <div class="prop-row">
          <label>Retry Delay (ms)</label>
          <input v-model.number="retryDelay" type="number" min="0" step="100" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.7px;
  color: var(--text-secondary);
}

.editor-tabs .tab {
  padding: 6px 14px;
  font-size: 11px;
  color: var(--text-muted);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  font-weight: 500;
}

.editor-tabs .tab.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

.kv-key,
.kv-value {
  flex: 1;
  padding: 5px 8px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-base);
  color: var(--text-primary);
  font-size: 12px;
  transition: border-color 0.12s;
}

.kv-type {
  width: 70px;
  padding: 5px 4px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-base);
  color: var(--text-primary);
  font-size: 11px;
  transition: border-color 0.12s;
}

.kv-key:focus,
.kv-value:focus,
.kv-type:focus {
  border-color: var(--accent);
  outline: none;
}

.type-opt {
  padding: 4px 12px;
  font-size: 11px;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 3px 3px 0 0;
  border: 1px solid transparent;
  font-weight: 500;
}

.type-opt.active {
  color: var(--accent);
  border-color: var(--border);
  border-bottom-color: var(--bg-base);
  background: var(--bg-base);
}

.body-textarea {
  flex: 1;
  padding: 8px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-base);
  color: var(--text-primary);
  font-size: 12px;
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  resize: none;
  transition: border-color 0.12s;
}

.body-textarea:focus {
  border-color: var(--accent);
  outline: none;
}

.auth-editor .prop-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.auth-editor label {
  width: 70px;
  font-size: 11px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.auth-editor select,
.auth-editor input {
  flex: 1;
  padding: 5px 8px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-base);
  color: var(--text-primary);
  font-size: 12px;
  transition: border-color 0.12s;
}

.auth-editor select:focus,
.auth-editor input:focus {
  border-color: var(--accent);
  outline: none;
}

/* Settings editor */
.settings-editor {
  padding: 12px;
}
.settings-editor .prop-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
}
.settings-editor label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
}
.settings-editor input[type='number'] {
  width: 100px;
  padding: 5px 8px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-base);
  color: var(--text-primary);
  font-size: 12px;
  text-align: right;
  transition: border-color 0.12s;
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}
.settings-editor input[type='number']:focus {
  border-color: var(--accent);
  outline: none;
}
.settings-editor input[type='checkbox'] {
  width: 16px;
  height: 16px;
  accent-color: var(--accent);
  cursor: pointer;
}
</style>
