import { ref } from 'vue'
import { isTauri } from '@/utils/env'
import { getSupportedImportFormats } from '@/utils/importer'
import { getSupportedExportFormats, type ExportFormat } from '@/utils/exporter'
import { useUIStore } from '@/stores/uiStore'

export interface FileIoState {
  currentPath: string | null
  saving: boolean
  loading: boolean
  lastStatus: string
  statusType: 'info' | 'success' | 'error'
}

/** File filter descriptor for Tauri open/save dialogs */
export interface FileFilter {
  name: string
  extensions: string[]
}

let dialogModule: typeof import('@tauri-apps/plugin-dialog') | null = null
let fsModule: typeof import('@tauri-apps/plugin-fs') | null = null
let windowModule: typeof import('@tauri-apps/api/window') | null = null
let pluginsResolved = false

async function resolvePlugins(): Promise<boolean> {
  if (pluginsResolved) return dialogModule != null && fsModule != null
  if (!isTauri()) return false
  try {
    dialogModule = await import('@tauri-apps/plugin-dialog')
    fsModule = await import('@tauri-apps/plugin-fs')
    try {
      const wm = await import('@tauri-apps/api/window')
      windowModule = wm
    } catch {
      // window module optional
    }
    pluginsResolved = true
    return true
  } catch {
    return false
  }
}

function getImportFilters(): FileFilter[] {
  return [
    { name: 'All Supported', extensions: ['json', 'jmx', 'yaml', 'yml'] },
    ...getSupportedImportFormats().map(f => ({
      name: f.label,
      extensions: f.extensions.map(e => e.replace('.', '')),
    })),
    { name: 'All Files', extensions: ['*'] },
  ]
}

function getExportFilters(): FileFilter[] {
  return [
    ...getSupportedExportFormats().map(f => ({
      name: f.label,
      extensions: f.extensions.map(e => e.replace('.', '')),
    })),
  ]
}

/** Get filter for a specific export format */
function getFilterForFormat(format: ExportFormat): FileFilter {
  const info = getSupportedExportFormats().find(f => f.format === format)
  if (!info) return { name: 'JSON', extensions: ['json'] }
  return {
    name: info.label,
    extensions: info.extensions.map(e => e.replace('.', '')),
  }
}

export function useFileIO() {
  const state = ref<FileIoState>({
    currentPath: null,
    saving: false,
    loading: false,
    lastStatus: '',
    statusType: 'info',
  })

  /** Update Tauri window title */
  async function updateWindowTitle(filename?: string) {
    try {
      if (windowModule?.getCurrentWindow) {
        const win = windowModule.getCurrentWindow()
        const base = 'ApiStress'
        if (filename) {
          await win.setTitle(`${filename} - ${base}`)
        } else {
          await win.setTitle(base)
        }
      }
    } catch {
      // non-Tauri or window API not available
    }
  }

  function setStatus(msg: string, type: 'info' | 'success' | 'error' = 'info') {
    state.value.lastStatus = msg
    state.value.statusType = type
  }

  /** Load a file's raw content. Returns { content, filePath } or null if cancelled/error. */
  async function loadRawContent(): Promise<{ content: string; filePath: string } | null> {
    state.value.loading = true
    state.value.lastStatus = ''
    try {
      if ((await resolvePlugins()) && dialogModule && fsModule) {
        const filePath = await dialogModule.open({
          multiple: false,
          filters: getImportFilters(),
        })
        if (!filePath) return null

        const content = await fsModule.readTextFile(filePath as string)
        const fp = filePath as string
        state.value.currentPath = fp
        setStatus('Loaded', 'success')

        try {
          const ui = useUIStore()
          ui.addRecentFile(fp)
        } catch {
          /* store may not be initialized */
        }

        await updateWindowTitle(fp.split(/[/\\]/).pop())
        return { content, filePath: fp }
      } else {
        return new Promise(resolve => {
          const input = document.createElement('input')
          input.type = 'file'
          input.accept = '.json,.jmx,.yaml,.yml'
          input.onchange = () => {
            const file = input.files?.[0]
            if (!file) {
              resolve(null)
              return
            }
            state.value.currentPath = file.name
            setStatus('Loaded', 'success')
            const reader = new FileReader()
            reader.onload = () => resolve({ content: reader.result as string, filePath: file.name })
            reader.readAsText(file)
          }
          input.click()
        })
      }
    } catch (e) {
      console.error('Failed to load file:', e)
      setStatus('Load failed', 'error')
      return null
    } finally {
      state.value.loading = false
    }
  }

  /** Save a string to file. Returns true on success. */
  async function saveContent(
    content: string,
    filters: FileFilter[],
    defaultName?: string,
    forceNewPath?: boolean,
  ): Promise<boolean> {
    state.value.saving = true
    state.value.lastStatus = ''
    try {
      if ((await resolvePlugins()) && dialogModule && fsModule) {
        const filePath = await dialogModule.save({
          defaultPath: forceNewPath
            ? defaultName || 'test-plan.json'
            : state.value.currentPath || defaultName || 'test-plan.json',
          filters,
        })
        if (!filePath) return false

        await fsModule.writeTextFile(filePath, content)
        state.value.currentPath = filePath
        setStatus('Saved', 'success')

        try {
          const ui = useUIStore()
          ui.addRecentFile(filePath)
        } catch {
          /* store may not be initialized */
        }

        await updateWindowTitle(filePath.split(/[/\\]/).pop())
        return true
      } else {
        const mimeTypes: Record<string, string> = {
          jmx: 'application/xml',
          json: 'application/json',
        }
        const ext = (defaultName || '').split('.').pop() || 'json'
        const blob = new Blob([content], { type: mimeTypes[ext] || 'text/plain' })
        const url = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = url
        a.download = defaultName || 'export.json'
        a.click()
        URL.revokeObjectURL(url)
        setStatus('Saved', 'success')
        return true
      }
    } catch (e) {
      console.error('Failed to save file:', e)
      setStatus('Save failed', 'error')
      return false
    } finally {
      state.value.saving = false
    }
  }

  // ---- High-level wrappers ----

  /** Load raw file content (opens with all import filters). Returns string content or null. */
  async function loadPlan(): Promise<string | null> {
    const result = await loadRawContent()
    return result?.content ?? null
  }

  /** Save raw content with export-format filters. */
  async function savePlan(content: string, defaultName?: string): Promise<boolean> {
    return saveContent(content, getExportFilters(), defaultName)
  }

  /** Save As — always prompts for a new path */
  async function saveAs(content: string, defaultName?: string): Promise<boolean> {
    return saveContent(content, getExportFilters(), defaultName, true)
  }

  /** Save to specific export format */
  async function exportAs(content: string, format: ExportFormat, defaultName: string): Promise<boolean> {
    return saveContent(content, [getFilterForFormat(format)], defaultName, true)
  }

  /** Close the current plan (reset without creating new) */
  async function closePlan(): Promise<void> {
    state.value.currentPath = null
    state.value.lastStatus = ''
    state.value.statusType = 'info'
    await updateWindowTitle()
  }

  /** Load a recent file directly by path */
  async function loadRecentFile(filePath: string): Promise<string | null> {
    state.value.loading = true
    state.value.lastStatus = ''
    try {
      if ((await resolvePlugins()) && fsModule) {
        const content = await fsModule.readTextFile(filePath)
        state.value.currentPath = filePath
        setStatus('Loaded', 'success')
        await updateWindowTitle(filePath.split(/[/\\]/).pop())
        return content
      }
      return null
    } catch (e) {
      console.error('Failed to load recent file:', e)
      setStatus('Failed to open file', 'error')
      return null
    } finally {
      state.value.loading = false
    }
  }

  return {
    state,
    loadRawContent,
    saveContent,
    loadPlan,
    savePlan,
    saveAs,
    exportAs,
    closePlan,
    loadRecentFile,
    updateWindowTitle,
  }
}
