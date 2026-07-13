<script setup lang="ts">
import type {
  LoopController,
  IfController,
  WhileController,
  TransactionController,
  ThroughputController,
  ResponseAssertion,
  JsonAssertion,
  DurationAssertion,
  ConstantTimer,
  UniformRandomTimer,
  GaussianRandomTimer,
  RegexExtractor,
  JsonExtractor,
  BoundaryExtractor,
  HttpDefaults,
  CsvDataSet,
  UserVariables,
  UserParameters,
} from '@/types'

defineProps<{
  node:
    | LoopController
    | IfController
    | WhileController
    | TransactionController
    | ThroughputController
    | ResponseAssertion
    | JsonAssertion
    | DurationAssertion
    | ConstantTimer
    | UniformRandomTimer
    | GaussianRandomTimer
    | RegexExtractor
    | JsonExtractor
    | BoundaryExtractor
    | HttpDefaults
    | CsvDataSet
    | UserVariables
    | UserParameters
}>()

const emit = defineEmits<{
  update: [key: string, value: unknown]
}>()

function update(key: string, value: unknown) {
  emit('update', key, value)
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

  <!-- LoopController -->
  <template v-if="node.type === 'LoopController'">
    <div class="prop-row">
      <label class="pp-label">Loops</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as LoopController).loops"
        @input="update('loops', parseInt(($event.target as HTMLInputElement).value))"
      />
      <span class="prop-hint">-1 = forever</span>
    </div>
  </template>

  <!-- IfController -->
  <template v-if="node.type === 'IfController'">
    <div class="prop-row col">
      <label class="pp-label">Condition</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as IfController).condition"
        @input="update('condition', ($event.target as HTMLInputElement).value)"
        placeholder='${varName} == "true"'
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Use Expression</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as IfController).useExpression"
        @change="update('useExpression', ($event.target as HTMLInputElement).checked)"
      />
      <label class="pp-label" style="margin-left: 12px">Evaluate All</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as IfController).evaluateAll"
        @change="update('evaluateAll', ($event.target as HTMLInputElement).checked)"
      />
    </div>
  </template>

  <!-- WhileController -->
  <template v-if="node.type === 'WhileController'">
    <div class="prop-row col">
      <label class="pp-label">Condition</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as WhileController).condition"
        @input="update('condition', ($event.target as HTMLInputElement).value)"
        placeholder="${__threadNum} < 10"
      />
    </div>
  </template>

  <!-- TransactionController -->
  <template v-if="node.type === 'TransactionController'">
    <div class="prop-row">
      <label class="pp-label">Include Duration</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as TransactionController).includeDuration"
        @change="update('includeDuration', ($event.target as HTMLInputElement).checked)"
      />
    </div>
  </template>

  <!-- ThroughputController -->
  <template v-if="node.type === 'ThroughputController'">
    <div class="prop-row">
      <label class="pp-label">Throughput (/min)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as ThroughputController).throughput"
        min="0"
        @input="update('throughput', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Per Thread</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as ThroughputController).perThread"
        @change="update('perThread', ($event.target as HTMLInputElement).checked)"
      />
    </div>
  </template>

  <!-- Timers -->
  <template v-if="node.type === 'ConstantTimer'">
    <div class="prop-row">
      <label class="pp-label">Delay (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as ConstantTimer).delay"
        min="0"
        @input="update('delay', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
    </div>
  </template>

  <template v-if="node.type === 'UniformRandomTimer'">
    <div class="prop-row">
      <label class="pp-label">Min Delay (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as UniformRandomTimer).minDelay"
        min="0"
        @input="update('minDelay', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Max Delay (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as UniformRandomTimer).maxDelay"
        min="0"
        @input="update('maxDelay', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
    </div>
  </template>

  <template v-if="node.type === 'GaussianRandomTimer'">
    <div class="prop-row">
      <label class="pp-label">Delay (mean ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as GaussianRandomTimer).delay"
        min="0"
        @input="update('delay', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Deviation (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as GaussianRandomTimer).deviation"
        min="0"
        @input="update('deviation', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
    </div>
  </template>

  <!-- Assertions -->
  <template v-if="node.type === 'ResponseAssertion'">
    <div class="prop-row col">
      <label class="pp-label">Test Field</label>
      <select
        class="pp-field"
        :value="(node as ResponseAssertion).testField"
        @change="update('testField', ($event.target as HTMLSelectElement).value)"
      >
        <option value="responseBody">Response Body</option>
        <option value="responseCode">Response Code</option>
        <option value="responseMessage">Response Message</option>
        <option value="responseHeaders">Response Headers</option>
      </select>
    </div>
    <div class="prop-row col">
      <label class="pp-label">Pattern Matching</label>
      <select
        class="pp-field"
        :value="(node as ResponseAssertion).patternMatching"
        @change="update('patternMatching', ($event.target as HTMLSelectElement).value)"
      >
        <option value="contains">Contains</option>
        <option value="matches">Matches</option>
        <option value="equals">Equals</option>
        <option value="notContains">Not Contains</option>
      </select>
    </div>
    <div class="prop-row">
      <label class="pp-label">Assume Success</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as ResponseAssertion).assumeSuccess"
        @change="update('assumeSuccess', ($event.target as HTMLInputElement).checked)"
      />
    </div>
    <div class="prop-section">
      <div class="section-title">Patterns</div>
      <div v-for="(p, i) in (node as ResponseAssertion).patterns" :key="i" class="kv-row">
        <input
          type="text"
          :value="p"
          :placeholder="'Pattern ' + (i + 1)"
          class="kv-key"
          style="flex: 1"
          @input="
            ;(node as ResponseAssertion).patterns[i] = ($event.target as HTMLInputElement).value
            update('patterns', [...(node as ResponseAssertion).patterns])
          "
        />
        <button
          class="kv-remove"
          @click="
            () => {
              const arr = [...(node as ResponseAssertion).patterns]
              arr.splice(i, 1)
              update('patterns', arr)
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
            const arr = [...(node as ResponseAssertion).patterns]
            arr.push('')
            update('patterns', arr)
          }
        "
      >
        + Add Pattern
      </button>
    </div>
  </template>

  <template v-if="node.type === 'JsonAssertion'">
    <div class="prop-row col">
      <label class="pp-label">JSON Path</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as JsonAssertion).jsonPath"
        @input="update('jsonPath', ($event.target as HTMLInputElement).value)"
        placeholder="$.data.id"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Comparison Mode</label>
      <select
        class="pp-field"
        :value="(node as JsonAssertion).comparisonMode"
        @change="update('comparisonMode', ($event.target as HTMLSelectElement).value)"
      >
        <option value="exists">Exists</option>
        <option value="notExists">Not Exists</option>
        <option value="equals">Equals</option>
      </select>
    </div>
    <div class="prop-row col">
      <label class="pp-label">Expected Value</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as JsonAssertion).expectedValue"
        @input="update('expectedValue', ($event.target as HTMLInputElement).value)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Expect Null</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as JsonAssertion).expectNull"
        @change="update('expectNull', ($event.target as HTMLInputElement).checked)"
      />
    </div>
  </template>

  <template v-if="node.type === 'DurationAssertion'">
    <div class="prop-row">
      <label class="pp-label">Max Duration (ms)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as DurationAssertion).maxDuration"
        min="0"
        @input="update('maxDuration', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
    </div>
  </template>

  <!-- Extractors -->
  <template v-if="node.type === 'RegexExtractor'">
    <div class="prop-row col">
      <label class="pp-label">Reference Name</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as RegexExtractor).referenceName"
        @input="update('referenceName', ($event.target as HTMLInputElement).value)"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Regex</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as RegexExtractor).regex"
        @input="update('regex', ($event.target as HTMLInputElement).value)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Match No.</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as RegexExtractor).matchNo"
        min="1"
        @input="update('matchNo', parseInt(($event.target as HTMLInputElement).value) || 1)"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Template</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as RegexExtractor).template"
        @input="update('template', ($event.target as HTMLInputElement).value)"
        placeholder="$1$"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Default Value</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as RegexExtractor).defaultValue"
        @input="update('defaultValue', ($event.target as HTMLInputElement).value)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Use Headers</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as RegexExtractor).useHeaders"
        @change="update('useHeaders', ($event.target as HTMLInputElement).checked)"
      />
      <label class="pp-label" style="margin-left: 12px">Use Body</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as RegexExtractor).useBody"
        @change="update('useBody', ($event.target as HTMLInputElement).checked)"
      />
    </div>
  </template>

  <template v-if="node.type === 'JsonExtractor'">
    <div class="prop-row col">
      <label class="pp-label">Reference Name</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as JsonExtractor).referenceName"
        @input="update('referenceName', ($event.target as HTMLInputElement).value)"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">JSON Path</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as JsonExtractor).jsonPath"
        @input="update('jsonPath', ($event.target as HTMLInputElement).value)"
        placeholder="$.data.id"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Default Value</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as JsonExtractor).defaultValue"
        @input="update('defaultValue', ($event.target as HTMLInputElement).value)"
      />
    </div>
  </template>

  <template v-if="node.type === 'BoundaryExtractor'">
    <div class="prop-row col">
      <label class="pp-label">Reference Name</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as BoundaryExtractor).referenceName"
        @input="update('referenceName', ($event.target as HTMLInputElement).value)"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Left Boundary</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as BoundaryExtractor).leftBoundary"
        @input="update('leftBoundary', ($event.target as HTMLInputElement).value)"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Right Boundary</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as BoundaryExtractor).rightBoundary"
        @input="update('rightBoundary', ($event.target as HTMLInputElement).value)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Match No.</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as BoundaryExtractor).matchNo"
        min="1"
        @input="update('matchNo', parseInt(($event.target as HTMLInputElement).value) || 1)"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Default Value</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as BoundaryExtractor).defaultValue"
        @input="update('defaultValue', ($event.target as HTMLInputElement).value)"
      />
    </div>
  </template>

  <!-- Config Elements -->
  <template v-if="node.type === 'HttpDefaults'">
    <div class="prop-row">
      <label class="pp-label">Protocol</label>
      <select
        class="pp-field"
        :value="(node as HttpDefaults).protocol"
        @change="update('protocol', ($event.target as HTMLSelectElement).value)"
      >
        <option value="https">HTTPS</option>
        <option value="http">HTTP</option>
      </select>
    </div>
    <div class="prop-row col">
      <label class="pp-label">Domain</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as HttpDefaults).domain"
        @input="update('domain', ($event.target as HTMLInputElement).value)"
        placeholder="api.example.com"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Port</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as HttpDefaults).port"
        @input="update('port', parseInt(($event.target as HTMLInputElement).value) || 443)"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Path</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as HttpDefaults).path"
        @input="update('path', ($event.target as HTMLInputElement).value)"
        placeholder="/api"
      />
    </div>
    <div class="prop-section">
      <div class="section-title">Headers</div>
      <div v-for="(h, i) in (node as HttpDefaults).headers" :key="i" class="kv-row">
        <input
          type="text"
          :value="h.key"
          placeholder="Name"
          class="kv-key"
          list="http-headers"
          @input="
            h.key = ($event.target as HTMLInputElement).value
            update('headers', [...(node as HttpDefaults).headers])
          "
        />
        <input
          type="text"
          :value="h.value"
          placeholder="Value"
          class="kv-value"
          @input="
            h.value = ($event.target as HTMLInputElement).value
            update('headers', [...(node as HttpDefaults).headers])
          "
        />
        <button
          class="kv-remove"
          @click="
            () => {
              const hdrs = [...(node as HttpDefaults).headers]
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
            const hdrs = [...(node as HttpDefaults).headers]
            hdrs.push({ key: '', value: '' })
            update('headers', hdrs)
          }
        "
      >
        + Add Header
      </button>
    </div>
  </template>

  <template v-if="node.type === 'CsvDataSet'">
    <div class="prop-row col">
      <label class="pp-label">Filename</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as CsvDataSet).filename"
        @input="update('filename', ($event.target as HTMLInputElement).value)"
      />
    </div>
    <div class="prop-row col">
      <label class="pp-label">Variable Names</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as CsvDataSet).variableNames"
        @input="update('variableNames', ($event.target as HTMLInputElement).value)"
        placeholder="var1,var2"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Delimiter</label>
      <input
        class="pp-field"
        type="text"
        :value="(node as CsvDataSet).delimiter"
        @input="update('delimiter', ($event.target as HTMLInputElement).value)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Recycle on EOF</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as CsvDataSet).recycleOnEof"
        @change="update('recycleOnEof', ($event.target as HTMLInputElement).checked)"
      />
      <label class="pp-label" style="margin-left: 12px">Stop Thread on EOF</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as CsvDataSet).stopThreadOnEof"
        @change="update('stopThreadOnEof', ($event.target as HTMLInputElement).checked)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Ignore First Line</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as CsvDataSet).ignoreFirstLine"
        @change="update('ignoreFirstLine', ($event.target as HTMLInputElement).checked)"
      />
    </div>
  </template>

  <template v-if="node.type === 'UserVariables'">
    <div class="prop-section">
      <div class="section-title">Variables</div>
      <div v-for="(v, i) in (node as UserVariables).variables" :key="i" class="kv-row">
        <input
          type="text"
          :value="v.key"
          placeholder="Name"
          class="kv-key"
          @input="
            v.key = ($event.target as HTMLInputElement).value
            update('variables', [...(node as UserVariables).variables])
          "
        />
        <input
          type="text"
          :value="v.value"
          placeholder="Value"
          class="kv-value"
          @input="
            v.value = ($event.target as HTMLInputElement).value
            update('variables', [...(node as UserVariables).variables])
          "
        />
        <button
          class="kv-remove"
          @click="
            () => {
              const vars = [...(node as UserVariables).variables]
              vars.splice(i, 1)
              update('variables', vars)
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
            const vars = [...(node as UserVariables).variables]
            vars.push({ key: '', value: '' })
            update('variables', vars)
          }
        "
      >
        + Add Variable
      </button>
    </div>
  </template>

  <template v-if="node.type === 'UserParameters'">
    <div class="prop-section">
      <div class="section-title">Parameters</div>
      <div v-for="(v, i) in (node as UserParameters).parameters" :key="i" class="kv-row">
        <input
          type="text"
          :value="v.key"
          placeholder="Name"
          class="kv-key"
          @input="
            v.key = ($event.target as HTMLInputElement).value
            update('parameters', [...(node as UserParameters).parameters])
          "
        />
        <input
          type="text"
          :value="v.value"
          placeholder="Value"
          class="kv-value"
          @input="
            v.value = ($event.target as HTMLInputElement).value
            update('parameters', [...(node as UserParameters).parameters])
          "
        />
        <button
          class="kv-remove"
          @click="
            () => {
              const params = [...(node as UserParameters).parameters]
              params.splice(i, 1)
              update('parameters', params)
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
            const params = [...(node as UserParameters).parameters]
            params.push({ key: '', value: '' })
            update('parameters', params)
          }
        "
      >
        + Add Parameter
      </button>
    </div>
  </template>
</template>
