<script setup lang="ts">
import { computed } from 'vue'
import { useExecutionStore } from '@/stores'
import { formatElapsed } from '@/utils/time'

const execution = useExecutionStore()

const errorRate = computed(() => {
  if (execution.totalSamples === 0) return '0.0'
  return ((execution.errorCount / execution.totalSamples) * 100).toFixed(1)
})
</script>

<template>
  <div
    class="flex items-center justify-between px-14px h-26px min-h-26px bg-deep border-t border-outline text-11px select-none"
  >
    <div class="flex items-center gap-2">
      <span
        class="w-6px h-6px rounded-full shrink-0"
        :class="{
          'bg-muted': execution.status === 'idle',
          'bg-accent animate-dot-pulse': execution.status === 'running',
          'bg-warning animate-dot-pulse-fast': execution.status === 'stopping',
          'bg-accent-cool': execution.status === 'completed',
        }"
      />
      <span class="text-10px font-semibold text-secondary tracking-wider">{{ execution.status.toUpperCase() }}</span>
    </div>

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
        <span class="metric-value" :class="{ 'text-danger': execution.errorCount > 0 }">{{
          execution.errorCount
        }}</span>
      </div>
      <div v-if="execution.status === 'running'" class="metric">
        <span class="metric-label">Elapsed</span>
        <span class="metric-value">{{ formatElapsed(execution.elapsedSeconds) }}</span>
      </div>
    </div>

    <div class="flex items-center gap-1">
      <div class="metric">
        <span class="metric-label">Error Rate</span>
        <span class="metric-value" :class="{ 'text-danger': execution.errorCount > 0 }">{{ errorRate }}%</span>
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
