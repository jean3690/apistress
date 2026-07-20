import { defineStore } from 'pinia'
import { shallowRef, ref, watch } from 'vue'

const STORAGE_KEY = 'apistress-ui-prefs'
const MAX_RECENT_FILES = 10

export interface RecentFile {
  path: string
  name: string
  openedAt: number
}

interface UiPrefs {
  activeResultTab: string
  recentFiles: RecentFile[]
  autoSave: boolean
}

function loadPrefs(): UiPrefs {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) return JSON.parse(raw)
  } catch {
    /* ignore */
  }
  return { activeResultTab: 'response', recentFiles: [], autoSave: false }
}

function savePrefs(prefs: Record<string, unknown>) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs))
  } catch {
    /* ignore */
  }
}

export const useUIStore = defineStore('ui', () => {
  const saved = loadPrefs()
  const activeResultTab = shallowRef(saved.activeResultTab)
  const recentFiles = ref<RecentFile[]>(saved.recentFiles || [])
  const autoSave = shallowRef(saved.autoSave ?? false)

  watch(activeResultTab, tab => {
    savePrefs({ activeResultTab: tab, recentFiles: recentFiles.value, autoSave: autoSave.value })
  })

  watch(
    recentFiles,
    files => {
      savePrefs({ activeResultTab: activeResultTab.value, recentFiles: files, autoSave: autoSave.value })
    },
    { deep: true },
  )

  watch(autoSave, val => {
    savePrefs({ activeResultTab: activeResultTab.value, recentFiles: recentFiles.value, autoSave: val })
  })

  function setActiveResultTab(tab: string) {
    activeResultTab.value = tab
  }

  function addRecentFile(path: string) {
    if (!path || typeof path !== 'string') return
    const name = path.split(/[/\\]/).pop() || path
    const entry: RecentFile = { path, name, openedAt: Date.now() }
    const filtered = recentFiles.value.filter(f => f.path !== path)
    recentFiles.value = [entry, ...filtered].slice(0, MAX_RECENT_FILES)
  }

  function removeRecentFile(path: string) {
    recentFiles.value = recentFiles.value.filter(f => f.path !== path)
  }

  function clearRecentFiles() {
    recentFiles.value = []
  }

  function toggleAutoSave() {
    autoSave.value = !autoSave.value
  }

  return {
    activeResultTab,
    recentFiles,
    autoSave,
    setActiveResultTab,
    addRecentFile,
    removeRecentFile,
    clearRecentFiles,
    toggleAutoSave,
  }
})
