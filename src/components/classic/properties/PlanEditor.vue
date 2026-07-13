<script setup lang="ts">
import type { TestPlan, ThreadGroup } from '@/types'

defineProps<{
  node: TestPlan | ThreadGroup
}>()

const emit = defineEmits<{
  update: [key: string, value: unknown]
}>()

function update(key: string, value: unknown) {
  emit('update', key, value)
}
</script>

<template>
  <!-- TestPlan -->
  <template v-if="node.type === 'TestPlan'">
    <div class="prop-row col">
      <label class="pp-label">Comments</label>
      <textarea
        class="pp-textarea"
        :value="(node as TestPlan).comments || ''"
        @input="update('comments', ($event.target as HTMLTextAreaElement).value)"
        rows="3"
        placeholder="Test plan description..."
      ></textarea>
    </div>
    <div class="prop-section">
      <div class="section-title">User Defined Variables</div>
      <div v-for="(v, i) in (node as TestPlan).variables" :key="i" class="kv-row">
        <input
          type="text"
          :value="v.key"
          placeholder="Name"
          class="kv-key"
          @input="
            v.key = ($event.target as HTMLInputElement).value
            update('variables', [...(node as TestPlan).variables])
          "
        />
        <input
          type="text"
          :value="v.value"
          placeholder="Value"
          class="kv-value"
          @input="
            v.value = ($event.target as HTMLInputElement).value
            update('variables', [...(node as TestPlan).variables])
          "
        />
        <button
          class="kv-remove"
          @click="
            () => {
              const vars = [...(node as TestPlan).variables]
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
            const vars = [...(node as TestPlan).variables]
            vars.push({ key: '', value: '' })
            update('variables', vars)
          }
        "
      >
        + Add Variable
      </button>
    </div>
    <div class="prop-section">
      <div class="section-title">Listeners</div>
      <div v-for="(l, i) in (node as TestPlan).listeners" :key="i" class="kv-row">
        <span class="kv-key" style="flex: 1">{{ l.type }}</span>
        <input
          type="text"
          :value="l.name"
          placeholder="Listener name"
          style="flex: 2"
          @input="
            l.name = ($event.target as HTMLInputElement).value
            update('listeners', [...(node as TestPlan).listeners])
          "
        />
        <button
          class="kv-remove"
          @click="
            () => {
              const arr = [...(node as TestPlan).listeners]
              arr.splice(i, 1)
              update('listeners', arr)
            }
          "
        >
          x
        </button>
      </div>
      <span class="prop-hint">Add listeners via the tree context menu</span>
    </div>
    <div class="prop-section">
      <div class="section-title">Test Assertions</div>
      <div v-for="(a, i) in (node as TestPlan).assertions" :key="i" class="kv-row">
        <select
          :value="a.metric"
          class="kv-key"
          @change="
            a.metric = ($event.target as HTMLSelectElement).value as 'errorRate'
            update('assertions', [...(node as TestPlan).assertions])
          "
        >
          <option value="errorRate">Error Rate %</option>
          <option value="avgResponseTime">Avg Response (ms)</option>
          <option value="throughput">Throughput (req/s)</option>
          <option value="p99">p99 (ms)</option>
        </select>
        <select
          :value="a.operator"
          class="kv-key"
          style="width: 60px"
          @change="
            a.operator = ($event.target as HTMLSelectElement).value as 'lt'
            update('assertions', [...(node as TestPlan).assertions])
          "
        >
          <option value="lt">&lt;</option>
          <option value="gt">&gt;</option>
        </select>
        <input
          type="number"
          :value="a.value"
          placeholder="Threshold"
          class="kv-value"
          style="width: 80px"
          @input="
            a.value = parseFloat(($event.target as HTMLInputElement).value) || 0
            update('assertions', [...(node as TestPlan).assertions])
          "
        />
        <button
          class="kv-remove"
          @click="
            () => {
              const arr = [...(node as TestPlan).assertions]
              arr.splice(i, 1)
              update('assertions', arr)
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
            const arr = [...(node as TestPlan).assertions]
            arr.push({ metric: 'errorRate', operator: 'lt', value: 5 })
            update('assertions', arr)
          }
        "
      >
        + Add Assertion
      </button>
    </div>
  </template>

  <!-- ThreadGroup -->
  <template v-if="node.type === 'ThreadGroup'">
    <div class="prop-row">
      <label class="pp-label">Threads</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as ThreadGroup).numThreads"
        min="1"
        @input="update('numThreads', parseInt(($event.target as HTMLInputElement).value) || 1)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Ramp-Up (s)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as ThreadGroup).rampUp"
        min="0"
        @input="update('rampUp', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">Warm-Up (s)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as ThreadGroup).warmUp"
        min="0"
        @input="update('warmUp', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
      <span class="prop-hint">0 = disabled</span>
    </div>
    <div class="prop-row">
      <label class="pp-label">Loops</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as ThreadGroup).loops"
        @input="update('loops', parseInt(($event.target as HTMLInputElement).value) || 1)"
      />
      <span class="prop-hint">-1 = forever</span>
    </div>
    <div class="prop-row">
      <label class="pp-label">Duration (s)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as ThreadGroup).duration"
        min="0"
        @input="update('duration', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
      <span class="prop-hint">0 = unlimited</span>
    </div>
    <div class="prop-row">
      <label class="pp-label">Delay (s)</label>
      <input
        class="pp-field"
        type="number"
        :value="(node as ThreadGroup).delay"
        min="0"
        @input="update('delay', parseInt(($event.target as HTMLInputElement).value) || 0)"
      />
    </div>
    <div class="prop-row">
      <label class="pp-label">On Error</label>
      <select
        class="pp-field"
        :value="(node as ThreadGroup).onErrorAction"
        @change="update('onErrorAction', ($event.target as HTMLSelectElement).value)"
      >
        <option value="continue">Continue</option>
        <option value="stopThread">Stop Thread</option>
        <option value="stopTest">Stop Test</option>
        <option value="startNextLoop">Start Next Loop</option>
      </select>
    </div>
    <div class="prop-row">
      <label class="pp-label">Scheduler</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as ThreadGroup).scheduler"
        @change="update('scheduler', ($event.target as HTMLInputElement).checked)"
      />
      <span class="prop-hint">Enable custom schedule</span>
    </div>
    <div class="prop-row">
      <label class="pp-label">Same User Each Iteration</label>
      <input
        class="pp-checkbox"
        type="checkbox"
        :checked="(node as ThreadGroup).sameUserOnEachIteration"
        @change="update('sameUserOnEachIteration', ($event.target as HTMLInputElement).checked)"
      />
    </div>
  </template>
</template>
