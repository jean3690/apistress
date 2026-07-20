<script setup lang="ts">
import { shallowRef, computed, onMounted, onUnmounted } from 'vue'
import { useTestPlanStore, useExecutionStore } from '@/stores'
import { useUIStore } from '@/stores/uiStore'
import { useFileIO } from '@/composables/useFileIO'
import { importJmx } from '@/utils/jmx'
import { importPostman } from '@/utils/postman'
import { importOpenApi } from '@/utils/openapi'
import { importPlan, detectImportFormat } from '@/utils/importer'
import { exportPlan, formatLabel, getSupportedExportFormats } from '@/utils/exporter'
import type { ExportFormat } from '@/utils/exporter'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
} from '@/components/ui/dropdown-menu'
import {
  Plus,
  FolderOpen,
  Save,
  Play,
  Square,
  X,
  FileUp,
  FileDown,
  Circle,
  CircleDot,
  ChevronDown,
  History,
  FilePlus,
  SaveAll,
  Import,
  LogOut,
} from '@lucide/vue'

const testPlan = useTestPlanStore()
const execution = useExecutionStore()
const ui = useUIStore()
const { state: fileState, loadPlan, savePlan, saveAs, exportAs, closePlan, loadRecentFile } = useFileIO()
const statusMessage = shallowRef('')
let statusTimer: ReturnType<typeof setTimeout> | null = null

const SUPPORTED_EXPORT_FORMATS = getSupportedExportFormats()

const currentFileName = computed(() => {
  const cp = fileState.value.currentPath
  if (cp) return cp.split(/[/\\]/).pop()
  return testPlan.plan.name ? `${testPlan.plan.name}.json` : 'untitled'
})

onMounted(() => {
  window.addEventListener('app:save', onSave)
  window.addEventListener('app:saveas', onSaveAs)
  window.addEventListener('file:new', onNew)
  window.addEventListener('file:open', onOpen)
})

onUnmounted(() => {
  window.removeEventListener('app:save', onSave)
  window.removeEventListener('app:saveas', onSaveAs)
  window.removeEventListener('file:new', onNew)
  window.removeEventListener('file:open', onOpen)
})

function showStatus(msg: string, duration = 2000) {
  statusMessage.value = msg
  if (statusTimer) clearTimeout(statusTimer)
  statusTimer = setTimeout(() => {
    statusMessage.value = ''
  }, duration)
}

async function onNew() {
  if (testPlan.dirty && !confirm('Discard unsaved changes?')) return
  testPlan.newPlan()
  await closePlan()
  showStatus('New plan created')
}

async function onSave() {
  const json = testPlan.toJSON()
  const defaultName = `${(testPlan.plan.name || 'test-plan').replace(/[<>:"/\\|?*]/g, '_')}.json`
  const ok = await savePlan(json, defaultName)
  if (ok) {
    testPlan.dirty = false
    showStatus('Plan saved')
  }
}

async function onSaveAs() {
  const json = testPlan.toJSON()
  const defaultName = `${(testPlan.plan.name || 'test-plan').replace(/[<>:"/\\|?*]/g, '_')}.json`
  const ok = await saveAs(json, defaultName)
  if (ok) {
    testPlan.dirty = false
    showStatus('Plan saved as')
  }
}

async function onOpen() {
  if (testPlan.dirty && !confirm('Discard unsaved changes?')) return
  const json = await loadPlan()
  if (json) {
    testPlan.loadPlan(json)
    showStatus('Plan opened')
  }
}

async function onOpenRecent(filePath: string) {
  if (testPlan.dirty && !confirm('Discard unsaved changes?')) return
  const content = await loadRecentFile(filePath)
  if (!content) {
    showStatus('Failed to open file - it may have been moved or deleted', 3000)
    return
  }
  try {
    const format = detectImportFormat(content, filePath)
    if (format === 'jmx') {
      const plan = importJmx(content)
      testPlan.loadPlan(JSON.stringify(plan))
    } else {
      const result = importPlan(content, filePath)
      testPlan.loadPlan(JSON.stringify(result.plan))
    }
  } catch (e) {
    // If format detection failed, try loading as raw JSON test plan
    try {
      testPlan.loadPlan(content)
    } catch {
      showStatus('Failed to open file - unsupported format', 3000)
      return
    }
  }
  showStatus('Opened recent file')
}

async function onImportJmx() {
  if (testPlan.dirty && !confirm('Discard unsaved changes?')) return
  const xml = await loadPlan()
  if (!xml) return
  try {
    const plan = importJmx(xml)
    testPlan.loadPlan(JSON.stringify(plan))
    showStatus('JMX imported')
  } catch (e) {
    alert('Invalid JMX file: ' + (e as Error).message)
  }
}

async function onImportPostman() {
  if (testPlan.dirty && !confirm('Discard unsaved changes?')) return
  const json = await loadPlan()
  if (!json) return
  try {
    const plan = importPostman(json)
    testPlan.loadPlan(JSON.stringify(plan))
    showStatus('Postman collection imported')
  } catch (e) {
    alert('Invalid Postman collection: ' + (e as Error).message)
  }
}

async function onImportOpenApi() {
  if (testPlan.dirty && !confirm('Discard unsaved changes?')) return
  const content = await loadPlan()
  if (!content) return
  try {
    const plan = importOpenApi(content)
    testPlan.loadPlan(JSON.stringify(plan))
    showStatus('OpenAPI spec imported')
  } catch (e) {
    alert('Invalid OpenAPI spec: ' + (e as Error).message)
  }
}

async function onExport(format: ExportFormat) {
  // Save first if there are unsaved changes
  if (testPlan.dirty) {
    await onSave()
  }
  const result = exportPlan(testPlan.plan, format)
  const ok = await exportAs(result.content, format, result.defaultFileName)
  if (ok) showStatus(`${formatLabel(format)} exported`)
}

async function onClose() {
  if (testPlan.dirty && !confirm('Discard unsaved changes?')) return
  testPlan.newPlan()
  await closePlan()
  showStatus('Plan closed')
}

async function onRun() {
  await execution.startTest(testPlan.toJSON())
}

function onStop() {
  execution.stopTest()
}

const displayStatus = computed(() => statusMessage.value || fileState.value.lastStatus || '')
</script>

<template>
  <TooltipProvider>
    <div
      class="flex items-center justify-between px-3.5 h-[42px] min-h-[42px] bg-[#181825] border-b border-border select-none"
    >
      <div class="flex items-center gap-0.5">
        <span class="font-bold text-[13px] tracking-wider text-foreground mr-1.5">
          <span class="text-primary">&#9670;</span> ApiStress
        </span>

        <Separator orientation="vertical" class="h-[18px] mx-1" />

        <!-- File Menu -->
        <DropdownMenu>
          <Tooltip>
            <TooltipTrigger as-child>
              <DropdownMenuTrigger as-child>
                <Button variant="ghost" size="sm" class="h-7 gap-1 text-xs font-medium">
                  <FilePlus class="size-3.5" /> File
                </Button>
              </DropdownMenuTrigger>
            </TooltipTrigger>
            <TooltipContent>File operations</TooltipContent>
          </Tooltip>
          <DropdownMenuContent class="min-w-[180px]">
            <DropdownMenuItem @click="onNew">
              <Plus class="size-3.5 mr-2" /> New
              <span class="shortcut">Ctrl+N</span>
            </DropdownMenuItem>

            <DropdownMenuSub>
              <DropdownMenuSubTrigger>
                <FolderOpen class="size-3.5 mr-2" /> Open
              </DropdownMenuSubTrigger>
              <DropdownMenuSubContent class="min-w-[200px]">
                <DropdownMenuItem @click="onOpen">
                  <FolderOpen class="size-3.5 mr-2" /> Browse...
                  <span class="shortcut">Ctrl+O</span>
                </DropdownMenuItem>
                <DropdownMenuSeparator v-if="ui.recentFiles.length > 0" />
                <DropdownMenuLabel v-if="ui.recentFiles.length > 0">
                  <History class="size-3 mr-1 inline" /> Recent
                </DropdownMenuLabel>
                <DropdownMenuItem
                  v-for="rf in ui.recentFiles"
                  :key="rf.path"
                  class="flex flex-col items-start gap-0 py-1.5"
                  @click="onOpenRecent(rf.path)"
                >
                  <span class="text-xs font-medium">{{ rf.name }}</span>
                  <span class="text-[10px] text-muted-foreground truncate max-w-[200px]">{{ rf.path }}</span>
                </DropdownMenuItem>
                <DropdownMenuSeparator v-if="ui.recentFiles.length > 0" />
                <DropdownMenuItem
                  v-if="ui.recentFiles.length > 0"
                  class="text-muted-foreground"
                  @click="ui.clearRecentFiles()"
                >
                  <X class="size-3 mr-2" /> Clear Recent
                </DropdownMenuItem>
              </DropdownMenuSubContent>
            </DropdownMenuSub>

            <DropdownMenuSeparator />

            <DropdownMenuItem @click="onSave">
              <Save class="size-3.5 mr-2" /> Save
              <span class="shortcut">Ctrl+S</span>
            </DropdownMenuItem>
            <DropdownMenuItem @click="onSaveAs">
              <SaveAll class="size-3.5 mr-2" /> Save As...
              <span class="shortcut">Ctrl+Shift+S</span>
            </DropdownMenuItem>

            <DropdownMenuSeparator />

            <DropdownMenuSub>
              <DropdownMenuSubTrigger>
                <FileDown class="size-3.5 mr-2" /> Export
              </DropdownMenuSubTrigger>
              <DropdownMenuSubContent>
                <DropdownMenuItem
                  v-for="fmt in SUPPORTED_EXPORT_FORMATS"
                  :key="fmt.format"
                  @click="onExport(fmt.format)"
                >
                  <FileDown class="size-3.5 mr-2" /> {{ fmt.label }}
                </DropdownMenuItem>
              </DropdownMenuSubContent>
            </DropdownMenuSub>

            <DropdownMenuSeparator />

            <DropdownMenuItem @click="onClose">
              <LogOut class="size-3.5 mr-2" /> Close
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>

        <!-- Import Dropdown -->
        <DropdownMenu>
          <Tooltip>
            <TooltipTrigger as-child>
              <DropdownMenuTrigger as-child>
                <Button variant="ghost" size="sm" class="h-7 gap-1 text-xs font-medium">
                  <Import class="size-3.5" /> Import <ChevronDown class="size-3" />
                </Button>
              </DropdownMenuTrigger>
            </TooltipTrigger>
            <TooltipContent>Import test plan</TooltipContent>
          </Tooltip>
          <DropdownMenuContent>
            <DropdownMenuItem @click="onImportJmx">
              <FileUp class="size-3.5 mr-2" /> JMeter (.jmx)
            </DropdownMenuItem>
            <DropdownMenuItem @click="onImportPostman">
              <FileUp class="size-3.5 mr-2" /> Postman Collection
            </DropdownMenuItem>
            <DropdownMenuItem @click="onImportOpenApi">
              <FileUp class="size-3.5 mr-2" /> OpenAPI / Swagger
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>

        <Separator orientation="vertical" class="h-[18px] mx-1.5" />

        <!-- Run / Stop / Clear -->
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
      </div>

      <!-- Right side: status & file info -->
      <div class="flex items-center gap-2">
        <span
          v-if="displayStatus"
          class="text-[10px] font-medium"
          :class="{
            'text-success': fileState.statusType === 'success',
            'text-danger': fileState.statusType === 'error',
            'text-muted-foreground': fileState.statusType === 'info',
          }"
        >
          {{ displayStatus }}
        </span>

        <span
          v-if="!displayStatus && fileState.currentPath && !testPlan.dirty"
          class="text-muted-foreground text-[10px] max-w-[160px] truncate"
          :title="fileState.currentPath"
        >
          {{ currentFileName }}
        </span>

        <Tooltip>
          <TooltipTrigger as-child>
            <Button variant="ghost" size="icon" class="h-7 w-7" @click="testPlan.dirty ? onSave() : undefined">
              <CircleDot v-if="testPlan.dirty" class="size-4 text-warning" />
              <Circle v-else class="size-4 text-muted-foreground" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            {{ testPlan.dirty ? 'Unsaved changes (click to save)' : 'No unsaved changes' }}
          </TooltipContent>
        </Tooltip>
      </div>
    </div>
  </TooltipProvider>
</template>

<style scoped>
.shortcut {
  margin-left: auto;
  padding-left: 12px;
  font-size: 10px;
  color: var(--muted-foreground);
  font-family: 'Cascadia Code', 'JetBrains Mono', 'Consolas', monospace;
}
</style>
