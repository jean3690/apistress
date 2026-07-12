import { defineStore } from 'pinia'
import { shallowRef, ref, watch } from 'vue'

export type UIMode = 'classic' | 'modern'

const STORAGE_KEY = 'apistress-ui-prefs'

function loadPrefs(): { mode: UIMode; activeResultTab: string } {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) return JSON.parse(raw)
  } catch { /* ignore */ }
  return { mode: 'classic', activeResultTab: 'tree' }
}

function savePrefs(prefs: Record<string, unknown>) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs))
  } catch { /* ignore */ }
}

export const useUIStore = defineStore('ui', () => {
  const saved = loadPrefs()
  const mode = shallowRef<UIMode>(saved.mode)
  const activeResultTab = shallowRef(saved.activeResultTab)
  const classicSplitSizes = ref([25, 50, 25]) // tree | props | results
  const modernSplitSizes = ref([20, 45, 35])  // list | editor | response

  watch([mode, activeResultTab], () => {
    savePrefs({ mode: mode.value, activeResultTab: activeResultTab.value })
  })

  function toggleMode() {
    mode.value = mode.value === 'classic' ? 'modern' : 'classic'
  }

  function setMode(m: UIMode) {
    mode.value = m
  }

  function setActiveResultTab(tab: string) {
    activeResultTab.value = tab
  }

  return {
    mode,
    activeResultTab,
    classicSplitSizes,
    modernSplitSizes,
    toggleMode,
    setMode,
    setActiveResultTab,
  }
})
