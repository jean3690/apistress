<script setup lang="ts">
import { computed } from 'vue'
import { useTestPlanStore } from '@/stores'
import MonacoJsonEditor from '@/components/editors/MonacoJsonEditor.vue'
import type {
  TestPlan, ThreadGroup, HttpSampler, LoopController, IfController,
  WhileController, TransactionController, ThroughputController,
  ResponseAssertion, JsonAssertion, DurationAssertion,
  ConstantTimer, UniformRandomTimer, GaussianRandomTimer,
  RegexExtractor, JsonExtractor, BoundaryExtractor,
  HttpDefaults, CsvDataSet, UserVariables, UserParameters,
} from '@/types'

const testPlan = useTestPlanStore()

interface AnyElement {
  id: string; type: string; name: string; enabled: boolean;
  [key: string]: unknown;
}

const node = computed<AnyElement | null>(() => testPlan.selectedNode as unknown as AnyElement)

function update(prop: string, value: unknown) {
  if (node.value) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    testPlan.updateNode(node.value.id, { [prop]: value } as any)
  }
}

function updateNested(path: string[], value: unknown) {
  if (!node.value) return
  // Build nested object
  const patch: Record<string, unknown> = {}
  let cur: Record<string, unknown> = patch
  for (let i = 0; i < path.length - 1; i++) {
    const next: Record<string, unknown> = {}
    cur[path[i]] = next
    cur = next
  }
  cur[path[path.length - 1]] = value
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  testPlan.updateNode(node.value.id, patch as any)
}

function addHeader() {
  if (!node.value) return
  const headers = [...((node.value as unknown as HttpSampler).headers || []), { key: '', value: '' }]
  update('headers', headers)
}

function removeHeader(idx: number) {
  if (!node.value) return
  const headers = [...(node.value as unknown as HttpSampler).headers]
  headers.splice(idx, 1)
  update('headers', headers)
}

function addQueryParam() {
  if (!node.value) return
  const params = [...((node.value as unknown as HttpSampler).queryParams || []), { key: '', value: '' }]
  update('queryParams', params)
}

function removeQueryParam(idx: number) {
  if (!node.value) return
  const params = [...(node.value as unknown as HttpSampler).queryParams]
  params.splice(idx, 1)
  update('queryParams', params)
}

function addPattern() {
  if (!node.value) { return }
  const patterns = [...((node.value as unknown as ResponseAssertion).patterns || []), '']
  update('patterns', patterns)
}

function removePattern(idx: number) {
  if (!node.value) return
  const patterns = [...(node.value as unknown as ResponseAssertion).patterns]
  patterns.splice(idx, 1)
  update('patterns', patterns)
}
</script>

<template>
  <div class="props-content">
    <!-- No selection -->
    <div v-if="!node" class="empty-props">
      <p>Select an element from the tree</p>
    </div>

    <div v-else class="prop-editor">
      <!-- Common properties -->
      <div class="prop-section">
        <div class="prop-row">
          <label class="pp-label">Name</label>
          <input
            type="text"
            :value="node.name"
            @input="update('name', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="prop-row">
          <label class="pp-label">Type</label>
          <span class="prop-value">{{ node.type }}</span>
        </div>
        <div class="prop-row">
          <label class="pp-label">Enabled</label>
          <input
            type="checkbox"
            :checked="node.enabled"
            @change="update('enabled', ($event.target as HTMLInputElement).checked)"
          />
        </div>
      </div>

      <!-- Type-specific properties -->
      <div class="prop-section">

        <!-- TestPlan -->
        <template v-if="node.type === 'TestPlan'">
          <div class="prop-row col">
            <label class="pp-label">Comments</label>
            <textarea class="pp-textarea" :value="(node as unknown as TestPlan).comments || ''"
              @input="update('comments', ($event.target as HTMLTextAreaElement).value)"
              rows="3" placeholder="Test plan description..."></textarea>
          </div>
          <div class="prop-section">
            <div class="section-title">User Defined Variables</div>
            <div v-for="(v, i) in (node as unknown as TestPlan).variables" :key="i" class="kv-row">
              <input type="text" :value="v.key" placeholder="Name" class="kv-key"
                @input="v.key = ($event.target as HTMLInputElement).value; update('variables', [...(node as unknown as TestPlan).variables])" />
              <input type="text" :value="v.value" placeholder="Value" class="kv-value"
                @input="v.value = ($event.target as HTMLInputElement).value; update('variables', [...(node as unknown as TestPlan).variables])" />
              <button class="kv-remove"
                @click="() => { const vars = [...(node as unknown as TestPlan).variables]; vars.splice(i, 1); update('variables', vars) }">x</button>
            </div>
            <button class="kv-add"
              @click="() => { const vars = [...(node as unknown as TestPlan).variables]; vars.push({ key: '', value: '' }); update('variables', vars) }">+ Add Variable</button>
          </div>
        </template>

        <!-- ThreadGroup -->
        <template v-if="node.type === 'ThreadGroup'">
          <div class="prop-row">
            <label class="pp-label">Threads</label>
            <input class="pp-field" type="number" :value="(node as unknown as ThreadGroup).numThreads" min="1"
              @input="update('numThreads', parseInt(($event.target as HTMLInputElement).value) || 1)" />
          </div>
          <div class="prop-row">
            <label class="pp-label">Ramp-Up (s)</label>
            <input class="pp-field" type="number" :value="(node as unknown as ThreadGroup).rampUp" min="0"
              @input="update('rampUp', parseInt(($event.target as HTMLInputElement).value) || 0)" />
          </div>
          <div class="prop-row">
            <label class="pp-label">Loops</label>
            <input class="pp-field" type="number" :value="(node as unknown as ThreadGroup).loops"
              @input="update('loops', parseInt(($event.target as HTMLInputElement).value))" />
            <span class="prop-hint">-1 = forever</span>
          </div>
          <div class="prop-row">
            <label class="pp-label">Duration (s)</label>
            <input class="pp-field" type="number" :value="(node as unknown as ThreadGroup).duration" min="0"
              @input="update('duration', parseInt(($event.target as HTMLInputElement).value) || 0)" />
          </div>
          <div class="prop-row">
            <label class="pp-label">Delay (s)</label>
            <input class="pp-field" type="number" :value="(node as unknown as ThreadGroup).delay" min="0"
              @input="update('delay', parseInt(($event.target as HTMLInputElement).value) || 0)" />
          </div>
          <div class="prop-row">
            <label class="pp-label">On Error</label>
            <select class="pp-field" :value="(node as unknown as ThreadGroup).onErrorAction"
              @change="update('onErrorAction', ($event.target as HTMLSelectElement).value)">
              <option value="continue">Continue</option>
              <option value="startNextLoop">Start Next Loop</option>
              <option value="stopThread">Stop Thread</option>
              <option value="stopTest">Stop Test</option>
            </select>
          </div>
        </template>

        <!-- HttpSampler -->
        <template v-if="node.type === 'HttpSampler'">
          <div class="prop-row">
            <label class="pp-label">Protocol</label>
            <select class="pp-field" :value="(node as unknown as HttpSampler).protocol"
              @change="update('protocol', ($event.target as HTMLSelectElement).value)">
              <option value="https">https</option>
              <option value="http">http</option>
            </select>
          </div>
          <div class="prop-row">
            <label class="pp-label">Method</label>
            <select class="pp-field" :value="(node as unknown as HttpSampler).method"
              @change="update('method', ($event.target as HTMLSelectElement).value)">
              <option>GET</option><option>POST</option><option>PUT</option>
              <option>DELETE</option><option>PATCH</option><option>HEAD</option><option>OPTIONS</option>
            </select>
          </div>
          <div class="prop-row">
            <label class="pp-label">Domain</label>
            <input class="pp-field" type="text" :value="(node as unknown as HttpSampler).domain"
              @input="update('domain', ($event.target as HTMLInputElement).value)" placeholder="api.example.com" />
          </div>
          <div class="prop-row">
            <label class="pp-label">Port</label>
            <input class="pp-field" type="number" :value="(node as unknown as HttpSampler).port"
              @input="update('port', parseInt(($event.target as HTMLInputElement).value) || 443)" />
          </div>
          <div class="prop-row">
            <label class="pp-label">Path</label>
            <input class="pp-field" type="text" :value="(node as unknown as HttpSampler).path"
              @input="update('path', ($event.target as HTMLInputElement).value)" placeholder="/api/endpoint" />
          </div>
          <div class="prop-row">
            <label class="pp-label">Timeout (ms)</label>
            <input class="pp-field" type="number" :value="(node as unknown as HttpSampler).timeout" min="0"
              @input="update('timeout', parseInt(($event.target as HTMLInputElement).value) || 30000)" />
          </div>
          <div class="prop-row">
            <label class="pp-label">Follow Redirects</label>
            <input class="pp-checkbox" type="checkbox" :checked="(node as unknown as HttpSampler).followRedirects"
              @change="update('followRedirects', ($event.target as HTMLInputElement).checked)" />
          </div>

          <!-- Auth -->
          <div class="prop-section">
            <div class="section-title">Authentication</div>
            <div class="prop-row">
              <label class="pp-label">Type</label>
              <select class="pp-field" :value="(node as unknown as HttpSampler).auth?.type || 'none'"
                @change="update('auth', {
                  type: ($event.target as HTMLSelectElement).value,
                  username: (node as unknown as HttpSampler).auth?.username || '',
                  password: (node as unknown as HttpSampler).auth?.password || '',
                  token: (node as unknown as HttpSampler).auth?.token || '',
                })">
                <option value="none">None</option>
                <option value="basic">Basic Auth</option>
                <option value="bearer">Bearer Token</option>
              </select>
            </div>
            <template v-if="(node as unknown as HttpSampler).auth?.type === 'basic'">
              <div class="prop-row">
                <label class="pp-label">Username</label>
                <input class="pp-field" type="text" :value="(node as unknown as HttpSampler).auth?.username || ''"
                  @input="update('auth', { ...(node as unknown as HttpSampler).auth, username: ($event.target as HTMLInputElement).value })" />
              </div>
              <div class="prop-row">
                <label class="pp-label">Password</label>
                <input class="pp-field" type="password" :value="(node as unknown as HttpSampler).auth?.password || ''"
                  @input="update('auth', { ...(node as unknown as HttpSampler).auth, password: ($event.target as HTMLInputElement).value })" />
              </div>
            </template>
            <template v-if="(node as unknown as HttpSampler).auth?.type === 'bearer'">
              <div class="prop-row col">
                <label class="pp-label">Token</label>
                <input class="pp-field" type="text" :value="(node as unknown as HttpSampler).auth?.token || ''"
                  @input="update('auth', { ...(node as unknown as HttpSampler).auth, token: ($event.target as HTMLInputElement).value })" />
              </div>
            </template>
          </div>

          <!-- Body -->
          <div class="prop-section">
            <div class="section-title">Body</div>
            <div class="prop-row">
              <label class="pp-label">Mode</label>
              <select class="pp-field" :value="(node as unknown as HttpSampler).body.mode"
                @change="updateNested(['body', 'mode'], ($event.target as HTMLSelectElement).value)">
                <option value="none">none</option>
                <option value="raw">raw</option>
                <option value="form-data">form-data</option>
                <option value="x-www-form-urlencoded">x-www-form-urlencoded</option>
              </select>
            </div>
            <template v-if="(node as unknown as HttpSampler).body.mode === 'raw'">
              <div class="prop-row">
                <label class="pp-label">Content-Type</label>
                <input class="pp-field" type="text" :value="(node as unknown as HttpSampler).body.contentType || ''"
                  @input="updateNested(['body', 'contentType'], ($event.target as HTMLInputElement).value)"
                  placeholder="application/json" />
              </div>
              <div class="prop-row col">
                <label class="pp-label">Body</label>
                <MonacoJsonEditor
                  :model-value="(node as unknown as HttpSampler).body.raw || ''"
                  @update:model-value="(v: string) => updateNested(['body', 'raw'], v)"
                />
              </div>
            </template>
            <template v-if="(node as unknown as HttpSampler).body.mode === 'form-data'">
              <div class="section-title">Form Data</div>
              <div v-for="(item, i) in ((node as unknown as HttpSampler).body.formData || [])" :key="i" class="kv-row">
                <input type="text" :value="item.key" placeholder="Name" class="kv-key"
                  @input="item.key = ($event.target as HTMLInputElement).value; updateNested(['body', 'formData'], [...((node as unknown as HttpSampler).body.formData || [])])" />
                <select :value="item.type || 'text'" class="pp-field fd-type"
                  @change="item.type = ($event.target as HTMLSelectElement).value as 'text' | 'file'; updateNested(['body', 'formData'], [...((node as unknown as HttpSampler).body.formData || [])])">
                  <option value="text">Text</option>
                  <option value="file">File</option>
                </select>
                <input type="text" :value="item.value" placeholder="Value/Path" class="kv-value"
                  @input="item.value = ($event.target as HTMLInputElement).value; updateNested(['body', 'formData'], [...((node as unknown as HttpSampler).body.formData || [])])" />
                <template v-if="item.type === 'file'">
                  <input type="text" :value="item.filename || ''" placeholder="Filename" class="kv-key"
                    @input="item.filename = ($event.target as HTMLInputElement).value; updateNested(['body', 'formData'], [...((node as unknown as HttpSampler).body.formData || [])])" />
                  <input type="text" :value="item.mimeType || ''" placeholder="MIME" class="kv-key"
                    @input="item.mimeType = ($event.target as HTMLInputElement).value; updateNested(['body', 'formData'], [...((node as unknown as HttpSampler).body.formData || [])])" />
                </template>
                <button class="kv-remove"
                  @click="() => { const fd = [...((node as unknown as HttpSampler).body.formData || [])]; fd.splice(i, 1); updateNested(['body', 'formData'], fd) }">x</button>
              </div>
              <button class="kv-add"
                @click="() => { const fd = [...((node as unknown as HttpSampler).body.formData || [])]; fd.push({ key: '', value: '', type: 'text' }); updateNested(['body', 'formData'], fd) }">+ Add Field</button>
            </template>
            <template v-if="(node as unknown as HttpSampler).body.mode === 'x-www-form-urlencoded'">
              <div class="section-title">URL Encoded Parameters</div>
              <div v-for="(p, i) in ((node as unknown as HttpSampler).body.urlEncoded || [])" :key="i" class="kv-row">
                <input type="text" :value="p.key" placeholder="Name" class="kv-key"
                  @input="p.key = ($event.target as HTMLInputElement).value; updateNested(['body', 'urlEncoded'], [...((node as unknown as HttpSampler).body.urlEncoded || [])])" />
                <input type="text" :value="p.value" placeholder="Value" class="kv-value"
                  @input="p.value = ($event.target as HTMLInputElement).value; updateNested(['body', 'urlEncoded'], [...((node as unknown as HttpSampler).body.urlEncoded || [])])" />
                <button class="kv-remove"
                  @click="() => { const ue = [...((node as unknown as HttpSampler).body.urlEncoded || [])]; ue.splice(i, 1); updateNested(['body', 'urlEncoded'], ue) }">x</button>
              </div>
              <button class="kv-add"
                @click="() => { const ue = [...((node as unknown as HttpSampler).body.urlEncoded || [])]; ue.push({ key: '', value: '' }); updateNested(['body', 'urlEncoded'], ue) }">+ Add Param</button>
            </template>
          </div>

          <!-- Headers -->
          <div class="prop-section">
            <div class="section-title">Headers</div>
            <div v-for="(h, i) in (node as unknown as HttpSampler).headers" :key="i" class="kv-row">
              <input type="text" :value="h.key" placeholder="Name" class="kv-key"
                @input="h.key = ($event.target as HTMLInputElement).value; update('headers', [...(node as unknown as HttpSampler).headers])" />
              <input type="text" :value="h.value" placeholder="Value" class="kv-value"
                @input="h.value = ($event.target as HTMLInputElement).value; update('headers', [...(node as unknown as HttpSampler).headers])" />
              <button class="kv-remove" @click="removeHeader(i)">x</button>
            </div>
            <button class="kv-add" @click="addHeader()">+ Add Header</button>
          </div>

          <!-- Query Params -->
          <div class="prop-section">
            <div class="section-title">Query Parameters</div>
            <div v-for="(p, i) in (node as unknown as HttpSampler).queryParams" :key="i" class="kv-row">
              <input type="text" :value="p.key" placeholder="Name" class="kv-key"
                @input="p.key = ($event.target as HTMLInputElement).value; update('queryParams', [...(node as unknown as HttpSampler).queryParams])" />
              <input type="text" :value="p.value" placeholder="Value" class="kv-value"
                @input="p.value = ($event.target as HTMLInputElement).value; update('queryParams', [...(node as unknown as HttpSampler).queryParams])" />
              <button class="kv-remove" @click="removeQueryParam(i)">x</button>
            </div>
            <button class="kv-add" @click="addQueryParam()">+ Add Query Param</button>
          </div>
        </template>

        <!-- LoopController -->
        <template v-if="node.type === 'LoopController'">
          <div class="prop-row">
            <label class="pp-label">Loops</label>
            <input class="pp-field" type="number" :value="(node as unknown as LoopController).loops"
              @input="update('loops', parseInt(($event.target as HTMLInputElement).value))" />
            <span class="prop-hint">-1 = forever</span>
          </div>
        </template>

        <!-- IfController -->
        <template v-if="node.type === 'IfController'">
          <div class="prop-row col">
            <label class="pp-label">Condition</label>
            <input class="pp-field" type="text" :value="(node as unknown as IfController).condition"
              @input="update('condition', ($event.target as HTMLInputElement).value)"
              placeholder='${varName} == "true"' />
          </div>
        </template>

        <!-- WhileController -->
        <template v-if="node.type === 'WhileController'">
          <div class="prop-row col">
            <label class="pp-label">Condition</label>
            <input class="pp-field" type="text" :value="(node as unknown as WhileController).condition"
              @input="update('condition', ($event.target as HTMLInputElement).value)"
              placeholder='${__threadNum} < 10' />
          </div>
        </template>

        <!-- TransactionController -->
        <template v-if="node.type === 'TransactionController'">
          <div class="prop-row">
            <label class="pp-label">Include Duration</label>
            <input class="pp-checkbox" type="checkbox" :checked="(node as unknown as TransactionController).includeDuration"
              @change="update('includeDuration', ($event.target as HTMLInputElement).checked)" />
          </div>
        </template>

        <!-- ThroughputController -->
        <template v-if="node.type === 'ThroughputController'">
          <div class="prop-row">
            <label class="pp-label">Throughput (/min)</label>
            <input class="pp-field" type="number" :value="(node as unknown as ThroughputController).throughput" min="0"
              @input="update('throughput', parseInt(($event.target as HTMLInputElement).value) || 0)" />
          </div>
          <div class="prop-row">
            <label class="pp-label">Per Thread</label>
            <input class="pp-checkbox" type="checkbox" :checked="(node as unknown as ThroughputController).perThread"
              @change="update('perThread', ($event.target as HTMLInputElement).checked)" />
          </div>
        </template>

        <!-- UniformRandomTimer -->
        <template v-if="node.type === 'UniformRandomTimer'">
          <div class="prop-row">
            <label class="pp-label">Min Delay (ms)</label>
            <input class="pp-field" type="number" :value="(node as unknown as UniformRandomTimer).minDelay" min="0"
              @input="update('minDelay', parseInt(($event.target as HTMLInputElement).value) || 0)" />
          </div>
          <div class="prop-row">
            <label class="pp-label">Max Delay (ms)</label>
            <input class="pp-field" type="number" :value="(node as unknown as UniformRandomTimer).maxDelay" min="0"
              @input="update('maxDelay', parseInt(($event.target as HTMLInputElement).value) || 0)" />
          </div>
        </template>

        <!-- GaussianRandomTimer -->
        <template v-if="node.type === 'GaussianRandomTimer'">
          <div class="prop-row">
            <label class="pp-label">Mean (ms)</label>
            <input class="pp-field" type="number" :value="(node as unknown as GaussianRandomTimer).delay" min="0"
              @input="update('delay', parseInt(($event.target as HTMLInputElement).value) || 0)" />
          </div>
          <div class="prop-row">
            <label class="pp-label">Deviation (ms)</label>
            <input class="pp-field" type="number" :value="(node as unknown as GaussianRandomTimer).deviation" min="0"
              @input="update('deviation', parseInt(($event.target as HTMLInputElement).value) || 0)" />
          </div>
        </template>

        <!-- ResponseAssertion -->
        <template v-if="node.type === 'ResponseAssertion'">
          <div class="prop-row">
            <label class="pp-label">Test Field</label>
            <select class="pp-field" :value="(node as unknown as ResponseAssertion).testField"
              @change="update('testField', ($event.target as HTMLSelectElement).value)">
              <option value="responseCode">Response Code</option>
              <option value="responseMessage">Response Message</option>
              <option value="responseBody">Response Body</option>
              <option value="responseHeaders">Response Headers</option>
              <option value="url">URL</option>
            </select>
          </div>
          <div class="prop-row">
            <label class="pp-label">Match Rule</label>
            <select class="pp-field" :value="(node as unknown as ResponseAssertion).patternMatching"
              @change="update('patternMatching', ($event.target as HTMLSelectElement).value)">
              <option value="contains">Contains</option>
              <option value="notContains">Not Contains</option>
              <option value="matches">Matches</option>
              <option value="equals">Equals</option>
              <option value="substring">Substring</option>
            </select>
          </div>
          <div class="prop-section">
            <div class="section-title">Patterns</div>
            <div v-for="(p, i) in (node as unknown as ResponseAssertion).patterns" :key="i" class="kv-row">
              <input type="text" :value="p" placeholder="Pattern" class="kv-full"
                @input="(node as unknown as ResponseAssertion).patterns[i] = ($event.target as HTMLInputElement).value; update('patterns', [...(node as unknown as ResponseAssertion).patterns])" />
              <button class="kv-remove" @click="removePattern(i)">x</button>
            </div>
            <button class="kv-add" @click="addPattern()">+ Add Pattern</button>
          </div>
        </template>

        <!-- JsonAssertion -->
        <template v-if="node.type === 'JsonAssertion'">
          <div class="prop-row col">
            <label class="pp-label">JSON Path</label>
            <input class="pp-field" type="text" :value="(node as unknown as JsonAssertion).jsonPath"
              @input="update('jsonPath', ($event.target as HTMLInputElement).value)" placeholder="$.data.id" />
          </div>
          <div class="prop-row">
            <label class="pp-label">Comparison</label>
            <select class="pp-field" :value="(node as unknown as JsonAssertion).comparisonMode"
              @change="update('comparisonMode', ($event.target as HTMLSelectElement).value)">
              <option value="exists">Exists</option>
              <option value="notExists">Not Exists</option>
              <option value="equals">Equals</option>
            </select>
          </div>
          <div v-if="(node as unknown as JsonAssertion).comparisonMode === 'equals'" class="prop-row col">
            <label class="pp-label">Expected</label>
            <input class="pp-field" type="text" :value="(node as unknown as JsonAssertion).expectedValue"
              @input="update('expectedValue', ($event.target as HTMLInputElement).value)" />
          </div>
        </template>

        <!-- DurationAssertion -->
        <template v-if="node.type === 'DurationAssertion'">
          <div class="prop-row">
            <label class="pp-label">Max (ms)</label>
            <input class="pp-field" type="number" :value="(node as unknown as DurationAssertion).maxDuration" min="0"
              @input="update('maxDuration', parseInt(($event.target as HTMLInputElement).value) || 3000)" />
          </div>
        </template>

        <!-- ConstantTimer -->
        <template v-if="node.type === 'ConstantTimer'">
          <div class="prop-row">
            <label class="pp-label">Delay (ms)</label>
            <input class="pp-field" type="number" :value="(node as unknown as ConstantTimer).delay" min="0"
              @input="update('delay', parseInt(($event.target as HTMLInputElement).value) || 0)" />
          </div>
        </template>

        <!-- RegexExtractor -->
        <template v-if="node.type === 'RegexExtractor'">
          <div class="prop-row col">
            <label class="pp-label">Ref Name</label>
            <input class="pp-field" type="text" :value="(node as unknown as RegexExtractor).referenceName"
              @input="update('referenceName', ($event.target as HTMLInputElement).value)" />
          </div>
          <div class="prop-row col">
            <label class="pp-label">Regex</label>
            <input class="pp-field" type="text" :value="(node as unknown as RegexExtractor).regex"
              @input="update('regex', ($event.target as HTMLInputElement).value)" placeholder="value=&quot;(.+?)&quot;" />
          </div>
          <div class="prop-row">
            <label class="pp-label">Template</label>
            <input class="pp-field" type="text" :value="(node as unknown as RegexExtractor).template"
              @input="update('template', ($event.target as HTMLInputElement).value)" placeholder="$1" />
          </div>
          <div class="prop-row">
            <label class="pp-label">Match No.</label>
            <input class="pp-field" type="number" :value="(node as unknown as RegexExtractor).matchNo" min="1"
              @input="update('matchNo', parseInt(($event.target as HTMLInputElement).value) || 1)" />
          </div>
          <div class="prop-row col">
            <label class="pp-label">Default</label>
            <input class="pp-field" type="text" :value="(node as unknown as RegexExtractor).defaultValue"
              @input="update('defaultValue', ($event.target as HTMLInputElement).value)" />
          </div>
        </template>

        <!-- JsonExtractor -->
        <template v-if="node.type === 'JsonExtractor'">
          <div class="prop-row col">
            <label class="pp-label">Ref Name</label>
            <input class="pp-field" type="text" :value="(node as unknown as JsonExtractor).referenceName"
              @input="update('referenceName', ($event.target as HTMLInputElement).value)" />
          </div>
          <div class="prop-row col">
            <label class="pp-label">JSON Path</label>
            <input class="pp-field" type="text" :value="(node as unknown as JsonExtractor).jsonPath"
              @input="update('jsonPath', ($event.target as HTMLInputElement).value)" placeholder="$.data.id" />
          </div>
          <div class="prop-row col">
            <label class="pp-label">Default</label>
            <input class="pp-field" type="text" :value="(node as unknown as JsonExtractor).defaultValue"
              @input="update('defaultValue', ($event.target as HTMLInputElement).value)" />
          </div>
        </template>

        <!-- BoundaryExtractor -->
        <template v-if="node.type === 'BoundaryExtractor'">
          <div class="prop-row col">
            <label class="pp-label">Ref Name</label>
            <input class="pp-field" type="text" :value="(node as unknown as BoundaryExtractor).referenceName"
              @input="update('referenceName', ($event.target as HTMLInputElement).value)" />
          </div>
          <div class="prop-row col">
            <label class="pp-label">Left Boundary</label>
            <input class="pp-field" type="text" :value="(node as unknown as BoundaryExtractor).leftBoundary"
              @input="update('leftBoundary', ($event.target as HTMLInputElement).value)" />
          </div>
          <div class="prop-row col">
            <label class="pp-label">Right Boundary</label>
            <input class="pp-field" type="text" :value="(node as unknown as BoundaryExtractor).rightBoundary"
              @input="update('rightBoundary', ($event.target as HTMLInputElement).value)" />
          </div>
        </template>

        <!-- HttpDefaults -->
        <template v-if="node.type === 'HttpDefaults'">
          <div class="prop-row">
            <label class="pp-label">Protocol</label>
            <select class="pp-field" :value="(node as unknown as HttpDefaults).protocol"
              @change="update('protocol', ($event.target as HTMLSelectElement).value)">
              <option value="https">https</option>
              <option value="http">http</option>
            </select>
          </div>
          <div class="prop-row col">
            <label class="pp-label">Domain</label>
            <input class="pp-field" type="text" :value="(node as unknown as HttpDefaults).domain"
              @input="update('domain', ($event.target as HTMLInputElement).value)" />
          </div>
          <div class="prop-row">
            <label class="pp-label">Port</label>
            <input class="pp-field" type="number" :value="(node as unknown as HttpDefaults).port"
              @input="update('port', parseInt(($event.target as HTMLInputElement).value) || 443)" />
          </div>
          <div class="prop-row col">
            <label class="pp-label">Path</label>
            <input class="pp-field" type="text" :value="(node as unknown as HttpDefaults).path"
              @input="update('path', ($event.target as HTMLInputElement).value)" placeholder="/" />
          </div>
          <div class="prop-section">
            <div class="section-title">Default Headers</div>
            <div v-for="(h, i) in (node as unknown as HttpDefaults).headers" :key="i" class="kv-row">
              <input type="text" :value="h.key" placeholder="Name" class="kv-key"
                @input="h.key = ($event.target as HTMLInputElement).value; update('headers', [...(node as unknown as HttpDefaults).headers])" />
              <input type="text" :value="h.value" placeholder="Value" class="kv-value"
                @input="h.value = ($event.target as HTMLInputElement).value; update('headers', [...(node as unknown as HttpDefaults).headers])" />
              <button class="kv-remove"
                @click="() => { const hdrs = [...(node as unknown as HttpDefaults).headers]; hdrs.splice(i, 1); update('headers', hdrs) }">x</button>
            </div>
            <button class="kv-add"
              @click="() => { const hdrs = [...(node as unknown as HttpDefaults).headers]; hdrs.push({ key: '', value: '' }); update('headers', hdrs) }">+ Add Header</button>
          </div>
        </template>

        <!-- CsvDataSet -->
        <template v-if="node.type === 'CsvDataSet'">
          <div class="prop-row col">
            <label class="pp-label">Filename</label>
            <input class="pp-field" type="text" :value="(node as unknown as CsvDataSet).filename"
              @input="update('filename', ($event.target as HTMLInputElement).value)" />
          </div>
          <div class="prop-row col">
            <label class="pp-label">Variables</label>
            <input class="pp-field" type="text" :value="(node as unknown as CsvDataSet).variableNames"
              @input="update('variableNames', ($event.target as HTMLInputElement).value)" placeholder="var1,var2" />
          </div>
          <div class="prop-row">
            <label class="pp-label">Delimiter</label>
            <input class="pp-field" type="text" :value="(node as unknown as CsvDataSet).delimiter"
              @input="update('delimiter', ($event.target as HTMLInputElement).value)" />
          </div>
        </template>

        <!-- UserVariables -->
        <template v-if="node.type === 'UserVariables'">
          <div class="prop-section">
            <div class="section-title">Variables</div>
            <div v-for="(v, i) in (node as unknown as UserVariables).variables" :key="i" class="kv-row">
              <input type="text" :value="v.key" placeholder="Name" class="kv-key"
                @input="v.key = ($event.target as HTMLInputElement).value; update('variables', [...(node as unknown as UserVariables).variables])" />
              <input type="text" :value="v.value" placeholder="Value" class="kv-value"
                @input="v.value = ($event.target as HTMLInputElement).value; update('variables', [...(node as unknown as UserVariables).variables])" />
              <button class="kv-remove" @click="(node as unknown as UserVariables).variables.splice(i, 1); update('variables', [...(node as unknown as UserVariables).variables])">x</button>
            </div>
            <button class="kv-add" @click="(node as unknown as UserVariables).variables.push({ key: '', value: '' }); update('variables', [...(node as unknown as UserVariables).variables])">+ Add Variable</button>
          </div>
        </template>

        <!-- UserParameters -->
        <template v-if="node.type === 'UserParameters'">
          <div class="prop-section">
            <div class="section-title">Parameters</div>
            <div v-for="(p, i) in (node as unknown as UserParameters).parameters" :key="i" class="kv-row">
              <input type="text" :value="p.key" placeholder="Name" class="kv-key"
                @input="p.key = ($event.target as HTMLInputElement).value; update('parameters', [...(node as unknown as UserParameters).parameters])" />
              <input type="text" :value="p.value" placeholder="Value" class="kv-value"
                @input="p.value = ($event.target as HTMLInputElement).value; update('parameters', [...(node as unknown as UserParameters).parameters])" />
              <button class="kv-remove" @click="(node as unknown as UserParameters).parameters.splice(i, 1); update('parameters', [...(node as unknown as UserParameters).parameters])">x</button>
            </div>
            <button class="kv-add" @click="(node as unknown as UserParameters).parameters.push({ key: '', value: '' }); update('parameters', [...(node as unknown as UserParameters).parameters])">+ Add Parameter</button>
          </div>
        </template>
      </div>

      <!-- Delete button -->
      <div v-if="node.type !== 'TestPlan'" class="prop-section">
        <button class="danger-btn" @click="testPlan.removeNode(node.id)">Delete Element</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.props-content {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.prop-editor {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.prop-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.3px;
  padding-bottom: 4px;
  border-bottom: 1px solid var(--border);
}

.prop-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.prop-row.col {
  flex-direction: column;
  align-items: stretch;
}

.pp-label {
  width: 80px;
  font-size: 11px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.prop-row.col .pp-label {
  width: auto;
}

.pp-field {
  flex: 1;
  padding: 4px 8px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 12px;
}

.pp-textarea {
  flex: 1;
  padding: 6px 8px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 12px;
  font-family: 'SF Mono', 'Consolas', monospace;
  resize: vertical;
}

.pp-field:focus,
.pp-textarea:focus {
  border-color: var(--accent);
  outline: none;
}

.pp-checkbox {
  width: 16px;
  height: 16px;
  accent-color: var(--accent);
}

.prop-value {
  color: var(--text-muted);
  font-size: 12px;
}

.prop-hint {
  font-size: 10px;
  color: var(--text-muted);
}

.kv-row {
  display: flex;
  gap: 4px;
  margin-bottom: 4px;
}

.kv-key, .kv-value {
  flex: 1;
  padding: 4px 6px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 12px;
}

.kv-full {
  flex: 1;
  padding: 4px 6px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 12px;
}

.fd-type {
  width: 70px;
  padding: 4px 4px;
  border: 1px solid var(--border);
  border-radius: 3px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 11px;
}

.kv-key:focus, .kv-value:focus, .kv-full:focus, .fd-type:focus {
  border-color: var(--accent);
  outline: none;
}

.kv-remove {
  padding: 2px 6px;
  border: none;
  background: transparent;
  color: var(--danger);
  cursor: pointer;
  font-size: 14px;
}

.kv-add {
  padding: 3px 10px;
  border: 1px dashed var(--border);
  border-radius: 4px;
  background: transparent;
  color: var(--text-muted);
  font-size: 11px;
  cursor: pointer;
}

.kv-add:hover { border-color: var(--accent); color: var(--accent); }

.danger-btn {
  padding: 6px 16px;
  border: 1px solid var(--danger);
  border-radius: 4px;
  background: transparent;
  color: var(--danger);
  cursor: pointer;
  font-size: 12px;
  width: 100%;
}

.danger-btn:hover {
  background: var(--danger);
  color: var(--bg-primary);
}

.empty-props {
  padding: 30px 20px;
  text-align: center;
  color: var(--text-muted);
}
</style>
