import { defineStore } from 'pinia'
import { shallowRef, watch } from 'vue'

const STORAGE_KEY = 'apistress-ui-prefs'

function loadPrefs(): { activeResultTab: string } {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) return JSON.parse(raw)
  } catch {
    /* ignore */
  }
  return { activeResultTab: 'response' }
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

  watch(activeResultTab, tab => {
    savePrefs({ activeResultTab: tab })
  })

  function setActiveResultTab(tab: string) {
    activeResultTab.value = tab
  }

  return {
    activeResultTab,
    setActiveResultTab,
  }
})
