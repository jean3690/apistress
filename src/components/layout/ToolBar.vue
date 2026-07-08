<script setup lang="ts">
import { ref } from 'vue'
import { useUIStore, useTestPlanStore, useExecutionStore } from '@/stores'
import { useFileIO } from '@/composables/useFileIO'
import { importJmx, exportJmx } from '@/utils/jmx'

const ui = useUIStore()
const testPlan = useTestPlanStore()
const execution = useExecutionStore()
const { state: fileState, loadPlan, savePlan } = useFileIO()
const importMessage = ref('')

function onNew() {
  if (testPlan.dirty && !confirm('Discard unsaved changes?')) return
  testPlan.newPlan()
}

async function onSave() {
  const json = testPlan.toJSON()
  const defaultName = `${(testPlan.plan.name || 'test-plan').replace(/[<>:"/\\|?*]/g, '_')}.json`
  const ok = await savePlan(json, defaultName)
  if (ok) testPlan.dirty = false
}

async function onLoad() {
  if (testPlan.dirty && !confirm('Discard unsaved changes?')) return
  const json = await loadPlan()
  if (json) testPlan.loadPlan(json)
}

async function onImportJmx() {
  if (testPlan.dirty && !confirm('Discard unsaved changes?')) return
  const xml = await loadPlan() // reuse file dialog — user picks .jmx file
  if (!xml) return
  try {
    const plan = importJmx(xml)
    testPlan.loadPlan(JSON.stringify(plan))
    importMessage.value = 'JMX imported'
    setTimeout(() => { importMessage.value = '' }, 2000)
  } catch (e) {
    alert('Invalid JMX file: ' + (e as Error).message)
  }
}

async function onExportJmx() {
  const xml = exportJmx(testPlan.plan)
  const defaultName = `${testPlan.plan.name.replace(/[<>:"/\\|?*]/g, '_')}.jmx`
  const ok = await savePlan(xml, defaultName)
  if (ok) {
    importMessage.value = 'JMX exported'
    setTimeout(() => { importMessage.value = '' }, 2000)
  }
}

async function onRun() {
  await execution.startTest(testPlan.toJSON())
}

function onStop() {
  execution.stopTest()
}
</script>

<template>
  <div class="toolbar">
    <div class="toolbar-left">
      <span class="app-title">ApiStress</span>
      <span class="separator">|</span>

      <button class="tb-btn" title="New Test Plan" @click="onNew">
        <span class="icon">📄</span> New
      </button>
      <button class="tb-btn" title="Open Test Plan" @click="onLoad">
        <span class="icon">📂</span> Open
      </button>
      <button class="tb-btn" title="Save Test Plan" @click="onSave">
        <span class="icon">💾</span> Save
      </button>
      <span v-if="fileState.lastStatus || importMessage" class="save-status">
        {{ fileState.lastStatus || importMessage }}
      </span>
      <span
        v-else-if="fileState.currentPath"
        class="file-path"
        :title="fileState.currentPath"
      >{{ fileState.currentPath.split(/[/\\]/).pop() }}</span>

      <span class="separator">|</span>

      <button class="tb-btn" title="Import JMeter .jmx" @click="onImportJmx">
        <span class="icon">📥</span> JMX
      </button>
      <button class="tb-btn" title="Export as JMeter .jmx" @click="onExportJmx">
        <span class="icon">📤</span> JMX
      </button>

      <span class="separator">|</span>

      <button class="tb-btn run" :disabled="execution.isRunning" title="Start Test" @click="onRun">
        <span class="icon">▶</span> Run
      </button>
      <button class="tb-btn stop" :disabled="!execution.isRunning" title="Stop Test" @click="onStop">
        <span class="icon">⏹</span> Stop
      </button>
      <button class="tb-btn" title="Clear Results" @click="execution.clear()">
        <span class="icon">🗑</span> Clear
      </button>

      <span class="separator">|</span>

      <button class="tb-btn" :class="{ dirty: testPlan.dirty }"
        :title="testPlan.dirty ? 'Unsaved changes' : 'No unsaved changes'">
        {{ testPlan.dirty ? '●' : '○' }}
      </button>
    </div>

    <div class="toolbar-right">
      <div class="mode-switcher">
        <button :class="['mode-btn', { active: ui.mode === 'classic' }]" @click="ui.setMode('classic')">Classic</button>
        <button :class="['mode-btn', { active: ui.mode === 'modern' }]" @click="ui.setMode('modern')">Modern</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  height: 40px;
  min-height: 40px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
  user-select: none;
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.app-title {
  font-weight: 700;
  font-size: 14px;
  color: var(--accent);
  margin-right: 4px;
}

.separator {
  color: var(--border);
  margin: 0 6px;
}

.tb-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s;
}

.tb-btn:hover { background: var(--bg-hover); }
.tb-btn:disabled { opacity: 0.4; cursor: not-allowed; }
.tb-btn .icon { font-size: 14px; }

.tb-btn.run { color: var(--success); }
.tb-btn.stop { color: var(--danger); }
.tb-btn.dirty { color: var(--warning); }

.save-status {
  color: var(--success);
  font-size: 11px;
  margin-left: 2px;
}

.file-path {
  color: var(--text-muted);
  font-size: 10px;
  margin-left: 4px;
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mode-switcher {
  display: flex;
  background: var(--bg-primary);
  border-radius: 6px;
  overflow: hidden;
  border: 1px solid var(--border);
}

.mode-btn {
  padding: 3px 14px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.mode-btn.active {
  background: var(--accent);
  color: var(--bg-primary);
  font-weight: 600;
}

.mode-btn:hover:not(.active) { color: var(--text-primary); }
</style>
