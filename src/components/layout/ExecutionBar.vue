<script setup lang="ts">
import { shallowRef, computed, watch } from 'vue'
import { useExecutionStore, useTestPlanStore } from '@/stores'
import { formatElapsed } from '@/utils/time'
import { Button } from '@/components/ui/button'
import { Play, Square } from '@lucide/vue'

const exec = useExecutionStore()
const testPlan = useTestPlanStore()

const showResults = shallowRef(false)

watch(
  () => exec.status,
  s => {
    if (s === 'running') showResults.value = true
  },
)

const elapsedDisplay = computed(() => formatElapsed(exec.elapsedSeconds))
const tick = computed(() => exec.statusTick)
const aggregateRows = computed(() => exec.aggregateByLabel.slice(0, 5))

async function startTest() {
  try {
    await exec.startTest(JSON.stringify(testPlan.plan))
  } catch (e) {
    console.error('Failed to start test:', e)
  }
}

function stopTest() {
  exec.stopTest()
}
</script>

<template>
  <div class="shrink-0">
    <div class="flex items-center justify-between h-[38px] px-3 bg-[#181825] border-t border-border text-xs">
      <div class="flex items-center gap-2">
        <Button v-if="!exec.isRunning" size="sm" class="h-7 gap-1" @click="startTest">
          <Play class="size-3.5 fill-current" /> Run Test
        </Button>
        <Button v-else variant="destructive" size="sm" class="h-7 gap-1" @click="stopTest">
          <Square class="size-3 fill-current" /> Stop
        </Button>

        <template v-if="exec.status !== 'idle'">
          <span
            :class="[
              'w-1.5 h-1.5 rounded-full shrink-0',
              exec.isRunning ? 'bg-primary animate-pulse' : 'bg-accent-cool',
            ]"
          />
          <span class="font-semibold text-[11px]">{{ exec.isRunning ? 'Running' : 'Completed' }}</span>
          <span class="text-border">|</span>
          <span class="text-muted-foreground font-mono text-[11px]">{{ elapsedDisplay }}</span>
          <span class="text-border">|</span>
          <span class="text-muted-foreground font-mono text-[11px]">Samples: {{ exec.totalSamples }}</span>
          <span class="text-border">|</span>
          <span
            class="font-mono text-[11px]"
            :class="exec.errorCount > 0 ? 'text-destructive font-semibold' : 'text-muted-foreground'"
            >Errors: {{ exec.errorCount }}</span
          >
          <span class="text-border">|</span>
          <span class="text-muted-foreground font-mono text-[11px]">Threads: {{ exec.threadsActive }}</span>
          <template v-if="tick?.p50 !== undefined">
            <span class="text-border">|</span>
            <span class="text-muted-foreground font-mono text-[11px]">p50: {{ tick.p50 }}ms</span>
            <span class="text-border">|</span>
            <span class="text-muted-foreground font-mono text-[11px]">p99: {{ tick.p99 }}ms</span>
            <span class="text-border">|</span>
            <span class="text-muted-foreground font-mono text-[11px]">{{ tick.throughput?.toFixed(1) }} req/s</span>
          </template>
        </template>
      </div>
      <Button
        v-if="exec.status !== 'idle'"
        variant="outline"
        size="sm"
        class="text-[11px]"
        @click="showResults = !showResults"
      >
        {{ showResults ? '▼' : '▲' }} {{ showResults ? 'Hide' : 'Show' }} Results
      </Button>
    </div>

    <div
      v-if="showResults && exec.status !== 'idle'"
      class="border-t border-border max-h-[260px] overflow-y-auto bg-background"
    >
      <div class="grid grid-cols-[repeat(auto-fit,minmax(130px,1fr))] gap-2 p-3">
        <div class="bg-surface border border-border px-3.5 py-3 text-center">
          <div class="text-[22px] font-bold font-mono text-primary">{{ exec.totalSamples }}</div>
          <div class="text-[9px] text-muted-foreground uppercase mt-1 tracking-wider font-medium">Total Samples</div>
        </div>
        <div
          class="bg-surface border px-3.5 py-3 text-center"
          :class="exec.errorCount > 0 ? 'border-destructive/25' : 'border-border'"
        >
          <div
            class="text-[22px] font-bold font-mono"
            :class="exec.errorCount > 0 ? 'text-destructive' : 'text-primary'"
          >
            {{ exec.errorCount }}
          </div>
          <div class="text-[9px] text-muted-foreground uppercase mt-1 tracking-wider font-medium">Errors</div>
        </div>
        <div class="bg-surface border border-border px-3.5 py-3 text-center">
          <div class="text-[22px] font-bold font-mono text-primary">
            {{ tick?.p50 ?? '-' }}<span class="text-xs font-normal text-muted-foreground ml-0.5">ms</span>
          </div>
          <div class="text-[9px] text-muted-foreground uppercase mt-1 tracking-wider font-medium">Median (p50)</div>
        </div>
        <div class="bg-surface border border-border px-3.5 py-3 text-center">
          <div class="text-[22px] font-bold font-mono text-primary">
            {{ tick?.p99 ?? '-' }}<span class="text-xs font-normal text-muted-foreground ml-0.5">ms</span>
          </div>
          <div class="text-[9px] text-muted-foreground uppercase mt-1 tracking-wider font-medium">p99</div>
        </div>
        <div class="bg-surface border border-border px-3.5 py-3 text-center">
          <div class="text-[22px] font-bold font-mono text-primary">
            {{ tick?.throughput?.toFixed(1) ?? '-'
            }}<span class="text-xs font-normal text-muted-foreground ml-0.5">/s</span>
          </div>
          <div class="text-[9px] text-muted-foreground uppercase mt-1 tracking-wider font-medium">Throughput</div>
        </div>
        <div class="bg-surface border border-border px-3.5 py-3 text-center">
          <div class="text-[22px] font-bold font-mono text-primary">
            {{ tick?.avgResponseTime ?? '-' }}<span class="text-xs font-normal text-muted-foreground ml-0.5">ms</span>
          </div>
          <div class="text-[9px] text-muted-foreground uppercase mt-1 tracking-wider font-medium">Avg Response</div>
        </div>
      </div>

      <table
        v-if="aggregateRows.length"
        class="w-full border-collapse text-[11px] mx-3 mb-3"
        style="width: calc(100% - 24px)"
      >
        <thead>
          <tr>
            <th
              class="text-left px-2 py-1 text-muted-foreground font-semibold border-b-2 border-border text-[9px] uppercase tracking-wider"
            >
              Label
            </th>
            <th
              class="text-left px-2 py-1 text-muted-foreground font-semibold border-b-2 border-border text-[9px] uppercase tracking-wider"
            >
              Samples
            </th>
            <th
              class="text-left px-2 py-1 text-muted-foreground font-semibold border-b-2 border-border text-[9px] uppercase tracking-wider"
            >
              Avg
            </th>
            <th
              class="text-left px-2 py-1 text-muted-foreground font-semibold border-b-2 border-border text-[9px] uppercase tracking-wider"
            >
              Min
            </th>
            <th
              class="text-left px-2 py-1 text-muted-foreground font-semibold border-b-2 border-border text-[9px] uppercase tracking-wider"
            >
              Max
            </th>
            <th
              class="text-left px-2 py-1 text-muted-foreground font-semibold border-b-2 border-border text-[9px] uppercase tracking-wider"
            >
              p90
            </th>
            <th
              class="text-left px-2 py-1 text-muted-foreground font-semibold border-b-2 border-border text-[9px] uppercase tracking-wider"
            >
              p99
            </th>
            <th
              class="text-left px-2 py-1 text-muted-foreground font-semibold border-b-2 border-border text-[9px] uppercase tracking-wider"
            >
              Error %
            </th>
            <th
              class="text-left px-2 py-1 text-muted-foreground font-semibold border-b-2 border-border text-[9px] uppercase tracking-wider"
            >
              Req/s
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in aggregateRows" :key="row.label">
            <td class="px-2 py-0.5 border-b border-white/3 font-semibold text-foreground">{{ row.label }}</td>
            <td class="px-2 py-0.5 border-b border-white/3 font-mono">{{ row.count }}</td>
            <td class="px-2 py-0.5 border-b border-white/3 font-mono">{{ row.avg }}ms</td>
            <td class="px-2 py-0.5 border-b border-white/3 font-mono">{{ row.min }}ms</td>
            <td class="px-2 py-0.5 border-b border-white/3 font-mono">{{ row.max }}ms</td>
            <td class="px-2 py-0.5 border-b border-white/3 font-mono">{{ row.p90 }}ms</td>
            <td class="px-2 py-0.5 border-b border-white/3 font-mono">{{ row.p99 }}ms</td>
            <td
              class="px-2 py-0.5 border-b border-white/3 font-mono"
              :class="{ 'text-destructive font-semibold': row.errorRate > 0 }"
            >
              {{ row.errorRate }}%
            </td>
            <td class="px-2 py-0.5 border-b border-white/3 font-mono">{{ row.throughput?.toFixed(1) }}</td>
          </tr>
        </tbody>
      </table>

      <div v-if="exec.assertionResults.length" class="px-3 pb-3">
        <div class="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider mb-1.5">
          Test Assertions
        </div>
        <div
          v-for="(a, i) in exec.assertionResults"
          :key="i"
          :class="[
            'flex items-center gap-2 px-2.5 py-1 rounded text-xs mb-1',
            a.passed ? 'bg-success/10 text-accent-cool' : 'bg-destructive/10 text-destructive',
          ]"
        >
          <span class="font-bold">{{ a.passed ? '✓' : '✗' }}</span>
          <span
            >{{ a.metric }} {{ a.operator }} {{ a.expected }} (actual:
            {{ typeof a.actual === 'number' ? a.actual.toFixed(1) : a.actual }})</span
          >
        </div>
      </div>
    </div>
  </div>
</template>
