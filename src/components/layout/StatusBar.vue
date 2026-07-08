<script setup lang="ts">
import { useExecutionStore } from '@/stores'

const execution = useExecutionStore()
</script>

<template>
  <div class="statusbar">
    <div class="status-left">
      <span
        :class="['status-indicator', execution.status]"
      />
      <span class="status-text">{{ execution.status.toUpperCase() }}</span>
    </div>
    <div class="status-center">
      <span class="stat-item">
        Threads: <strong>{{ execution.threadsActive }}</strong>
      </span>
      <span class="stat-item">
        Samples: <strong>{{ execution.totalSamples }}</strong>
      </span>
      <span class="stat-item">
        Errors: <strong class="error">{{ execution.errorCount }}</strong>
      </span>
      <span class="stat-item" v-if="execution.status === 'running'">
        Elapsed: <strong>{{ execution.elapsedSeconds }}s</strong>
      </span>
    </div>
    <div class="status-right">
      <span class="stat-item">
        Error Rate:
        <strong :class="{ error: execution.errorCount > 0 }">
          {{ execution.totalSamples > 0
            ? ((execution.errorCount / execution.totalSamples) * 100).toFixed(1) + '%'
            : '0%' }}
        </strong>
      </span>
    </div>
  </div>
</template>

<style scoped>
.statusbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  height: 28px;
  min-height: 28px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border);
  font-size: 11px;
  user-select: none;
}

.status-left,
.status-center,
.status-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.status-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-muted);
}

.status-indicator.idle { background: var(--text-muted); }
.status-indicator.running { background: var(--success); animation: pulse 1s infinite; }
.status-indicator.stopping { background: var(--warning); animation: pulse 0.5s infinite; }
.status-indicator.completed { background: var(--accent); }

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.stat-item {
  color: var(--text-secondary);
}

.stat-item strong {
  color: var(--text-primary);
  font-weight: 600;
}

.stat-item strong.error {
  color: var(--danger);
}
</style>
