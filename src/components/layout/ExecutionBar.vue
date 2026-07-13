<script setup lang="ts">
import { shallowRef, computed, watch } from 'vue'
import { useExecutionStore, useTestPlanStore } from '@/stores'
import { formatElapsed } from '@/utils/time'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
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
        <Card>
          <CardContent class="flex flex-col items-center py-3">
            <span class="text-[22px] font-bold font-mono text-primary">{{ exec.totalSamples }}</span>
            <span class="text-[9px] text-muted-foreground uppercase mt-1 tracking-wider font-medium"
              >Total Samples</span
            >
          </CardContent>
        </Card>
        <Card :class="exec.errorCount > 0 ? 'border-destructive/25' : ''">
          <CardContent class="flex flex-col items-center py-3">
            <span
              class="text-[22px] font-bold font-mono"
              :class="exec.errorCount > 0 ? 'text-destructive' : 'text-primary'"
              >{{ exec.errorCount }}</span
            >
            <span class="text-[9px] text-muted-foreground uppercase mt-1 tracking-wider font-medium">Errors</span>
          </CardContent>
        </Card>
        <Card
          v-for="m in [
            { label: 'Median (p50)', value: tick?.p50, unit: 'ms' },
            { label: 'p99', value: tick?.p99, unit: 'ms' },
            { label: 'Throughput', value: tick?.throughput?.toFixed(1), unit: '/s' },
            { label: 'Avg Response', value: tick?.avgResponseTime, unit: 'ms' },
          ]"
          :key="m.label"
        >
          <CardContent class="flex flex-col items-center py-3">
            <span class="text-[22px] font-bold font-mono text-primary">
              {{ m.value ?? '-' }}<span class="text-xs font-normal text-muted-foreground ml-0.5">{{ m.unit }}</span>
            </span>
            <span class="text-[9px] text-muted-foreground uppercase mt-1 tracking-wider font-medium">{{
              m.label
            }}</span>
          </CardContent>
        </Card>
      </div>

      <div v-if="aggregateRows.length" class="px-3 pb-3">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead class="text-[9px]">Label</TableHead>
              <TableHead class="text-[9px]">Samples</TableHead>
              <TableHead class="text-[9px]">Avg</TableHead>
              <TableHead class="text-[9px]">Min</TableHead>
              <TableHead class="text-[9px]">Max</TableHead>
              <TableHead class="text-[9px]">p90</TableHead>
              <TableHead class="text-[9px]">p99</TableHead>
              <TableHead class="text-[9px]">Error %</TableHead>
              <TableHead class="text-[9px]">Req/s</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="row in aggregateRows" :key="row.label">
              <TableCell class="font-semibold">{{ row.label }}</TableCell>
              <TableCell class="font-mono">{{ row.count }}</TableCell>
              <TableCell class="font-mono">{{ row.avg }}ms</TableCell>
              <TableCell class="font-mono">{{ row.min }}ms</TableCell>
              <TableCell class="font-mono">{{ row.max }}ms</TableCell>
              <TableCell class="font-mono">{{ row.p90 }}ms</TableCell>
              <TableCell class="font-mono">{{ row.p99 }}ms</TableCell>
              <TableCell class="font-mono" :class="{ 'text-destructive font-semibold': row.errorRate > 0 }"
                >{{ row.errorRate }}%</TableCell
              >
              <TableCell class="font-mono">{{ row.throughput?.toFixed(1) }}</TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>

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
