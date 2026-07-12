<script setup lang="ts">
import { ref, watch } from 'vue'
import { useTestPlanStore } from '@/stores'
import { findNodeById } from '@/utils/tree-utils'
import type { HttpSampler, HttpBody, KeyValuePair, FormDataItem } from '@/types'

const testPlan = useTestPlanStore()

const props = defineProps<{
  samplerId: string | null
}>()

const emit = defineEmits<{
  response: [data: { status: string; body: string; headers: string } | null]
  'update:loading': [value: boolean]
  saved: []
}>()

// ---- Editing state ----
const method = ref('GET')
const url = ref('')
const reqHeaders = ref<KeyValuePair[]>([])
const reqBody = ref('')
const reqBodyType = ref<'none' | 'raw' | 'form' | 'form-data'>('none')
const reqFormData = ref<FormDataItem[]>([])
const reqContentType = ref('application/json')
const reqName = ref('')
const editorTab = ref<'headers' | 'body' | 'auth'>('headers')
const loading = ref(false)
const saveMessage = ref('')
const authType = ref<'none' | 'basic' | 'bearer'>('none')
const authUsername = ref('')
const authPassword = ref('')
const authToken = ref('')

function parseUrl(input: string): { protocol: string; domain: string; port: number; path: string } {
  try {
    const u = new URL(input.startsWith('http') ? input : 'https://' + input)
    return {
      protocol: u.protocol.replace(':', ''),
      domain: u.hostname,
      port: u.port ? parseInt(u.port) : (u.protocol === 'https:' ? 443 : 80),
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
}

watch(() => props.samplerId, (id) => {
  if (id) {
    loadSampler(id)
    emit('response', null)
  }
})

// ---- Save to store ----
function saveRequest() {
  if (!props.samplerId) return
  const parsed = parseUrl(url.value)
  const body: HttpBody = {
    mode: reqBodyType.value === 'raw' ? 'raw'
      : reqBodyType.value === 'form' ? 'x-www-form-urlencoded'
      : reqBodyType.value === 'form-data' ? 'form-data'
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
  } as unknown as Record<string, unknown>)

  saveMessage.value = 'Saved'
  setTimeout(() => { saveMessage.value = '' }, 1500)
  emit('saved')
}

// ---- Send request ----
async function sendRequest() {
  loading.value = true
  emit('update:loading', true)
  emit('response', null)
  try {
    const headers: Record<string, string> = {}
    for (const h of reqHeaders.value) {
      if (h.key) headers[h.key] = h.value
    }
    const opts: RequestInit = { method: method.value, headers }

    if (method.value !== 'GET' && method.value !== 'HEAD' && reqBodyType.value !== 'none') {
      if (reqBodyType.value === 'raw') {
        headers['Content-Type'] = reqContentType.value
        opts.body = reqBody.value
      } else if (reqBodyType.value === 'form-data') {
        const fd = new FormData()
        for (const item of reqFormData.value) {
          if (!item.key) continue
          if (item.type === 'file') {
            fd.append(item.key, new Blob([item.value], { type: item.mimeType || 'application/octet-stream' }), item.filename || 'file')
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
      status: `${res.status} ${res.statusText} (${elapsed}ms)`,
      body: tryFormatJson(resBody),
      headers: [...res.headers.entries()].map(([k, v]) => `${k}: ${v}`).join('\n'),
    })
  } catch (e) {
    emit('response', { status: 'Error', body: String(e), headers: '' })
  } finally {
    loading.value = false
    emit('update:loading', false)
  }
}

function tryFormatJson(text: string): string {
  try { return JSON.stringify(JSON.parse(text), null, 2) } catch { return text }
}
</script>

<template>
  <div class="panel editor-panel">
    <div class="panel-header">
      <div class="header-left">
        <span>Request</span>
        <input
          v-if="samplerId"
          v-model="reqName"
          class="req-name-input"
          @blur="saveRequest"
        />
      </div>
      <div class="header-right">
        <span v-if="saveMessage" class="save-msg">{{ saveMessage }}</span>
        <button v-if="samplerId" class="save-btn" @click="saveRequest">Save</button>
        <button class="send-btn" :disabled="loading || !url" @click="sendRequest">
          {{ loading ? 'Sending...' : 'Send' }}
        </button>
      </div>
    </div>
    <div class="editor-content">
      <div class="url-bar">
        <select v-model="method" class="method-select">
          <option>GET</option><option>POST</option><option>PUT</option>
          <option>DELETE</option><option>PATCH</option><option>HEAD</option><option>OPTIONS</option>
        </select>
        <input v-model="url" class="url-input" placeholder="https://api.example.com/endpoint" @keydown.enter="sendRequest" />
      </div>

      <div class="editor-tabs">
        <span :class="['tab', { active: editorTab === 'headers' }]" @click="editorTab = 'headers'">Headers</span>
        <span :class="['tab', { active: editorTab === 'body' }]" @click="editorTab = 'body'">Body</span>
        <span :class="['tab', { active: editorTab === 'auth' }]" @click="editorTab = 'auth'">Auth</span>
      </div>

      <!-- Headers -->
      <div v-if="editorTab === 'headers'" class="kv-editor">
        <div class="kv-row" v-for="(h, i) in reqHeaders" :key="i">
          <input v-model="h.key" placeholder="Header name" class="kv-key" />
          <input v-model="h.value" placeholder="Header value" class="kv-value" />
          <button class="kv-remove" @click="reqHeaders.splice(i, 1)">x</button>
        </div>
        <button class="kv-add" @click="reqHeaders.push({ key: '', value: '' })">+ Add Header</button>
      </div>

      <!-- Body -->
      <div v-if="editorTab === 'body'" class="body-editor">
        <div class="body-type-bar">
          <span :class="['type-opt', { active: reqBodyType === 'none' }]" @click="reqBodyType = 'none'">None</span>
          <span :class="['type-opt', { active: reqBodyType === 'raw' }]" @click="reqBodyType = 'raw'">Raw</span>
          <span :class="['type-opt', { active: reqBodyType === 'form' }]" @click="reqBodyType = 'form'">Form</span>
          <span :class="['type-opt', { active: reqBodyType === 'form-data' }]" @click="reqBodyType = 'form-data'">Form-Data</span>
        </div>
        <div v-if="reqBodyType === 'raw'" class="body-raw">
          <input v-model="reqContentType" class="ct-input" placeholder="Content-Type" />
          <textarea v-model="reqBody" class="body-textarea" rows="12" placeholder='{"key": "value"}'></textarea>
        </div>
        <div v-else-if="reqBodyType === 'form'" class="body-form">
          <textarea v-model="reqBody" class="body-textarea" rows="6" placeholder="key1=value1&key2=value2"></textarea>
        </div>
        <div v-else-if="reqBodyType === 'form-data'" class="kv-editor">
          <div class="kv-row" v-for="(item, i) in reqFormData" :key="i">
            <input v-model="item.key" placeholder="Name" class="kv-key" />
            <select v-model="item.type" class="kv-type">
              <option value="text">Text</option>
              <option value="file">File</option>
            </select>
            <input v-model="item.value" placeholder="Value/Path" class="kv-value" />
            <template v-if="item.type === 'file'">
              <input v-model="item.filename" placeholder="Filename" class="kv-key" />
              <input v-model="item.mimeType" placeholder="MIME" class="kv-key" />
            </template>
            <button class="kv-remove" @click="reqFormData.splice(i, 1)">x</button>
          </div>
          <button class="kv-add" @click="reqFormData.push({ key: '', value: '', type: 'text' })">+ Add Field</button>
        </div>
        <div v-else class="body-none"><p>No body will be sent.</p></div>
      </div>

      <!-- Auth -->
      <div v-if="editorTab === 'auth'" class="auth-editor">
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
    </div>
  </div>
</template>

<style scoped>
.panel { display: flex; flex-direction: column; overflow: hidden; }

.editor-panel {
  flex: 1;
  min-width: 300px;
  background: var(--bg-primary);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.header-left, .header-right { display: flex; align-items: center; gap: 8px; }

.req-name-input {
  padding: 2px 6px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 12px;
  font-weight: 600;
  width: 180px;
}

.req-name-input:focus { border-color: var(--accent); outline: none; }

.save-msg { color: var(--success); font-size: 11px; }

.save-btn {
  padding: 4px 12px;
  border: 1px solid var(--accent);
  border-radius: 4px;
  background: transparent;
  color: var(--accent);
  font-size: 12px;
  cursor: pointer;
}

.save-btn:hover { background: rgba(137, 180, 250, 0.15); }

.send-btn {
  padding: 4px 16px;
  border: none;
  border-radius: 4px;
  background: var(--accent);
  color: var(--bg-primary);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.send-btn:disabled { opacity: 0.4; cursor: not-allowed; }
.send-btn:hover:not(:disabled) { background: var(--accent-hover); }

.editor-content { flex: 1; display: flex; flex-direction: column; overflow: hidden; }

.url-bar { display: flex; padding: 10px; gap: 8px; }

.method-select {
  width: 100px;
  padding: 6px 8px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg-surface);
  color: var(--text-primary);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}

.url-input {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
  font-family: 'SF Mono', 'Consolas', monospace;
}

.url-input:focus, .method-select:focus { border-color: var(--accent); outline: none; }

.editor-tabs { display: flex; gap: 2px; padding: 0 10px; border-bottom: 1px solid var(--border); }

.editor-tabs .tab {
  padding: 6px 14px;
  font-size: 11px;
  color: var(--text-muted);
  cursor: pointer;
  border-bottom: 2px solid transparent;
}

.editor-tabs .tab.active { color: var(--accent); border-bottom-color: var(--accent); }

.kv-editor { flex: 1; overflow-y: auto; padding: 8px 10px; }

.kv-row { display: flex; gap: 6px; margin-bottom: 4px; }

.kv-key, .kv-value {
  flex: 1;
  padding: 4px 8px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 12px;
}

.kv-type {
  width: 70px;
  padding: 4px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 11px;
}

.kv-key:focus, .kv-value:focus, .kv-type:focus { border-color: var(--accent); outline: none; }

.kv-remove {
  padding: 2px 6px;
  border: none;
  background: transparent;
  color: var(--danger);
  cursor: pointer;
  font-size: 16px;
}

.kv-add {
  margin-top: 6px;
  padding: 4px 12px;
  border: 1px dashed var(--border);
  border-radius: 4px;
  background: transparent;
  color: var(--text-muted);
  font-size: 11px;
  cursor: pointer;
}

.kv-add:hover { border-color: var(--accent); color: var(--accent); }

.body-editor { flex: 1; display: flex; flex-direction: column; overflow: hidden; }

.body-type-bar { display: flex; gap: 2px; padding: 8px 10px 0; }

.type-opt {
  padding: 4px 12px;
  font-size: 11px;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 4px 4px 0 0;
  border: 1px solid transparent;
}

.type-opt.active {
  color: var(--accent);
  border-color: var(--border);
  border-bottom-color: var(--bg-primary);
  background: var(--bg-primary);
}

.body-raw { display: flex; flex-direction: column; gap: 6px; padding: 8px 10px; flex: 1; }

.ct-input {
  padding: 4px 8px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 12px;
}

.ct-input:focus { border-color: var(--accent); outline: none; }

.body-form { padding: 8px 10px; flex: 1; }

.body-textarea {
  flex: 1;
  padding: 8px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 12px;
  font-family: 'SF Mono', 'Consolas', monospace;
  resize: none;
}

.body-textarea:focus { border-color: var(--accent); outline: none; }

.body-none { padding: 30px 10px; text-align: center; color: var(--text-muted); font-size: 12px; }

.auth-editor { padding: 16px 10px; }

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

.auth-editor select, .auth-editor input {
  flex: 1;
  padding: 4px 8px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 12px;
}

.auth-editor select:focus, .auth-editor input:focus { border-color: var(--accent); outline: none; }
</style>
