import { ref } from 'vue'

export interface FileIoState {
  currentPath: string | null
  saving: boolean
  loading: boolean
  lastStatus: string
}

/**
 * Composable for test plan file open/save via Tauri dialog + fs, with browser fallback.
 * Call `resolvePlugins()` first to avoid runtime errors in browser-only dev.
 */
export function useFileIO() {
  const state = ref<FileIoState>({
    currentPath: null,
    saving: false,
    loading: false,
    lastStatus: '',
  })

  let dialogModule: typeof import('@tauri-apps/plugin-dialog') | null = null
  let fsModule: typeof import('@tauri-apps/plugin-fs') | null = null
  let pluginsResolved = false

  function isTauri(): boolean {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
  }

  async function resolvePlugins(): Promise<boolean> {
    if (pluginsResolved) return dialogModule != null && fsModule != null
    if (!isTauri()) return false
    try {
      dialogModule = await import('@tauri-apps/plugin-dialog')
      fsModule = await import('@tauri-apps/plugin-fs')
      pluginsResolved = true
      return true
    } catch {
      return false
    }
  }

  /** Load a test plan from file. Returns JSON string or null if cancelled/error. */
  async function loadPlan(): Promise<string | null> {
    state.value.loading = true
    state.value.lastStatus = ''
    try {
      if (await resolvePlugins() && dialogModule && fsModule) {
        const filePath = await dialogModule.open({
          multiple: false,
          filters: [{ name: 'Test Plan', extensions: ['json'] }],
        })
        if (!filePath) return null

        const content = await fsModule.readTextFile(filePath as string)
        state.value.currentPath = filePath as string
        state.value.lastStatus = 'Loaded'
        return content
      } else {
        // Browser fallback
        return new Promise((resolve) => {
          const input = document.createElement('input')
          input.type = 'file'
          input.accept = '.json'
          input.onchange = () => {
            const file = input.files?.[0]
            if (!file) { resolve(null); return }
            const reader = new FileReader()
            reader.onload = () => resolve(reader.result as string)
            reader.readAsText(file)
          }
          input.click()
        })
      }
    } catch (e) {
      console.error('Failed to load file:', e)
      state.value.lastStatus = 'Load failed'
      return null
    } finally {
      state.value.loading = false
    }
  }

  /** Save a JSON string to file. Returns true on success. */
  async function savePlan(json: string, defaultName?: string): Promise<boolean> {
    state.value.saving = true
    state.value.lastStatus = ''
    try {
      if (await resolvePlugins() && dialogModule && fsModule) {
        const filePath = await dialogModule.save({
          defaultPath: state.value.currentPath || (defaultName || 'test-plan.json'),
          filters: [{ name: 'Test Plan', extensions: ['json'] }],
        })
        if (!filePath) return false

        await fsModule.writeTextFile(filePath, json)
        state.value.currentPath = filePath
        state.value.lastStatus = 'Saved'
        return true
      } else {
        // Browser fallback: download as file
        const blob = new Blob([json], { type: 'application/json' })
        const url = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = url
        a.download = defaultName || 'test-plan.json'
        a.click()
        URL.revokeObjectURL(url)
        state.value.lastStatus = 'Saved'
        return true
      }
    } catch (e) {
      console.error('Failed to save file:', e)
      state.value.lastStatus = 'Save failed'
      return false
    } finally {
      state.value.saving = false
    }
  }

  return { state, loadPlan, savePlan }
}
