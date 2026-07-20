<script setup lang="ts">
import { computed } from 'vue'
import { useExecutionStore, useTestPlanStore } from '@/stores'
import { useFileIO } from '@/composables/useFileIO'
import { formatElapsed } from '@/utils/time'

const execution = useExecutionStore()
const testPlan = useTestPlanStore()
const { state: fileState } = useFileIO()

const errorRate = computed(() => {
  if (execution.totalSamples === 0) return '0.0'
  return ((execution.errorCount / execution.totalSamples) * 100).toFixed(1)
})

const fileName = computed(() => {
  const cp = fileState.value.currentPath
  if (cp) {
    const name = cp.split(/[/\\]/).pop()
    return testPlan.dirty ? `${name} *` : name
  }
  return testPlan.dirty ? 'untitled *' : ''
})
</script>

<template>
  <div
    class="flex items-center justify-between px-3.5 h-[26px] min-h-[26px] bg-sidebar border-t border-border text-[11px] select-none"
  >
    <!-- Left: Status indicator + file name -->
    <div class="flex items-center gap-2 min-w-0">
      <span
        class="w-[6px] h-[6px] rounded-full shrink-0"
        :class="{
          'bg-muted-foreground': execution.status === 'idle',
          'bg-primary animate-dot-pulse': execution.status === 'running',
          'bg-warning animate-dot-pulse-fast': execution.status === 'stopping',
          'bg-success': execution.status === 'completed',
        }"
      />
      <span class="text-[10px] font-semibold text-muted-foreground tracking-wider shrink-0">{{
        execution.status.toUpperCase()
      }}</span>

      <span
        v-if="fileName"
        class="text-[10px] text-muted-foreground truncate max-w-[200px] border-l border-border pl-2 ml-0.5"
        :class="{ 'text-warning': testPlan.dirty }"
        :title="fileState.currentPath || ''"
      >
        {{ fileName }}
      </span>
    </div>

    <!-- Center: Execution metrics -->
    <div class="flex items-center gap-0.5">
      <div class="metric">
        <span class="metric-label">Threads</span>
        <span class="metric-value">{{ execution.threadsActive }}</span>
      </div>
      <div class="metric">
        <span class="metric-label">Samples</span>
        <span class="metric-value">{{ execution.totalSamples.toLocaleString() }}</span>
      </div>
      <div class="metric">
        <span class="metric-label">Errors</span>
        <span class="metric-value" :class="{ 'text-destructive': execution.errorCount > 0 }">{{
          execution.errorCount
        }}</span>
      </div>
      <div v-if="execution.status === 'running'" class="metric">
        <span class="metric-label">Elapsed</span>
        <span class="metric-value">{{ formatElapsed(execution.elapsedSeconds) }}</span>
      </div>
    </div>

    <!-- Right: Error rate -->
    <div class="flex items-center gap-1">
      <div class="metric">
        <span class="metric-label">Error Rate</span>
        <span class="metric-value" :class="{ 'text-destructive': execution.errorCount > 0 }">{{ errorRate }}%</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.metric {
  display: flex;
  align-items: baseline;
  gap: 5px;
  padding: 0 8px;
  border-left: 1px solid var(--border);
}
.metric:first-child {
  border-left: none;
}
.metric-label {
  font-size: 9px;
  font-weight: 500;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.4px;
}
.metric-value {
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--text-primary);
  min-width: 32px;
  text-align: right;
}

@keyframes dot-pulse {
  0%,
  100% {
    opacity: 1;
    box-shadow: 0 0 0 0 var(--accent-glow);
  }
  50% {
    opacity: 0.5;
    box-shadow: 0 0 0 3px transparent;
  }
}
.animate-dot-pulse {
  animation: dot-pulse 1.5s ease-in-out infinite;
}
.animate-dot-pulse-fast {
  animation: dot-pulse 0.6s ease-in-out infinite;
}
</style>
