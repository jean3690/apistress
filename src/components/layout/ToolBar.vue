<script setup lang="ts">
import { shallowRef, onMounted, onUnmounted } from 'vue'
import { useTestPlanStore, useExecutionStore } from '@/stores'
import { useFileIO } from '@/composables/useFileIO'
import { importJmx, exportJmx } from '@/utils/jmx'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { Plus, FolderOpen, Save, Play, Square, X, FileUp, FileDown, Circle, CircleDot } from '@lucide/vue'

const testPlan = useTestPlanStore()
const execution = useExecutionStore()
const { state: fileState, loadPlan, savePlan } = useFileIO()
const importMessage = shallowRef('')

onMounted(() => window.addEventListener('app:save', onSaveWrapper))
onUnmounted(() => window.removeEventListener('app:save', onSaveWrapper))

function onSaveWrapper() {
  onSave()
}

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
  const xml = await loadPlan()
  if (!xml) return
  try {
    const plan = importJmx(xml)
    testPlan.loadPlan(JSON.stringify(plan))
    importMessage.value = 'JMX imported'
    setTimeout(() => {
      importMessage.value = ''
    }, 2000)
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
    setTimeout(() => {
      importMessage.value = ''
    }, 2000)
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
  <TooltipProvider>
    <div
      class="flex items-center justify-between px-3.5 h-[42px] min-h-[42px] bg-[#181825] border-b border-border select-none"
    >
      <div class="flex items-center gap-1">
        <span class="font-bold text-[13px] tracking-wider text-foreground mr-1.5">
          <span class="text-primary">&#9670;</span> ApiStress
        </span>

        <Separator orientation="vertical" class="h-[18px] mx-1.5" />

        <Tooltip
          v-for="btn in [
            { fn: onNew, icon: Plus, label: 'New', tip: 'New Test Plan' },
            { fn: onLoad, icon: FolderOpen, label: 'Open', tip: 'Open Test Plan' },
            { fn: onSave, icon: Save, label: 'Save', tip: 'Save Test Plan' },
          ]"
          :key="btn.label"
        >
          <TooltipTrigger as-child>
            <Button variant="ghost" size="sm" class="h-7 gap-1 text-xs font-medium" @click="btn.fn">
              <component :is="btn.icon" class="size-3.5" /> {{ btn.label }}
            </Button>
          </TooltipTrigger>
          <TooltipContent>{{ btn.tip }}</TooltipContent>
        </Tooltip>

        <span v-if="fileState.lastStatus || importMessage" class="text-success text-[10px] ml-1 font-medium">
          {{ fileState.lastStatus || importMessage }}
        </span>
        <span
          v-else-if="fileState.currentPath"
          class="text-muted-foreground text-[10px] ml-1 max-w-[120px] truncate"
          :title="fileState.currentPath"
          >{{ fileState.currentPath.split(/[/\\]/).pop() }}</span
        >

        <Separator orientation="vertical" class="h-[18px] mx-1.5" />

        <Tooltip>
          <TooltipTrigger as-child>
            <Button variant="ghost" size="sm" class="h-7 gap-1 text-xs font-medium" @click="onImportJmx">
              <FileUp class="size-3.5" /> JMX
            </Button>
          </TooltipTrigger>
          <TooltipContent>Import JMX</TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger as-child>
            <Button variant="ghost" size="sm" class="h-7 gap-1 text-xs font-medium" @click="onExportJmx">
              <FileDown class="size-3.5" /> JMX
            </Button>
          </TooltipTrigger>
          <TooltipContent>Export JMX</TooltipContent>
        </Tooltip>

        <Separator orientation="vertical" class="h-[18px] mx-1.5" />

        <Tooltip>
          <TooltipTrigger as-child>
            <Button
              variant="ghost"
              size="sm"
              class="h-7 gap-1 text-xs font-semibold text-primary hover:bg-accent-glow"
              :disabled="execution.isRunning"
              @click="onRun"
            >
              <Play class="size-3 fill-primary" /> Run
            </Button>
          </TooltipTrigger>
          <TooltipContent>Run Test</TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger as-child>
            <Button
              variant="ghost"
              size="sm"
              class="h-7 gap-1 text-xs font-semibold text-danger hover:bg-danger-glow"
              :disabled="!execution.isRunning"
              @click="onStop"
            >
              <Square class="size-3 fill-danger" /> Stop
            </Button>
          </TooltipTrigger>
          <TooltipContent>Stop Test</TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger as-child>
            <Button variant="ghost" size="sm" class="h-7 gap-1 text-xs font-medium" @click="execution.clear()">
              <X class="size-3.5" /> Clear
            </Button>
          </TooltipTrigger>
          <TooltipContent>Clear Results</TooltipContent>
        </Tooltip>

        <Separator orientation="vertical" class="h-[18px] mx-1.5" />

        <Tooltip>
          <TooltipTrigger as-child>
            <Button variant="ghost" size="icon" class="h-7 w-7">
              <CircleDot v-if="testPlan.dirty" class="size-4 text-warning" />
              <Circle v-else class="size-4 text-muted-foreground" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>{{ testPlan.dirty ? 'Unsaved changes' : 'No unsaved changes' }}</TooltipContent>
        </Tooltip>
      </div>
    </div>
  </TooltipProvider>
</template>
