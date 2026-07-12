<script setup lang="ts">
import { ref, watch, onBeforeUnmount, shallowRef, type Ref } from 'vue'
import type { editor } from 'monaco-editor'

const props = defineProps<{
  modelValue: string
  readOnly?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const containerRef = ref<HTMLElement>()
const monacoEditor = shallowRef<editor.IStandaloneCodeEditor | null>(null) as Ref<editor.IStandaloneCodeEditor | null>
const loaded = ref(false)

async function initEditor() {
  if (!containerRef.value || loaded.value) return
  try {
    const monaco = await import('monaco-editor')
    loaded.value = true

    const ed = monaco.editor.create(containerRef.value, {
      value: props.modelValue,
      language: 'json',
      theme: 'vs-dark',
      minimap: { enabled: false },
      lineNumbers: 'on',
      scrollBeyondLastLine: false,
      automaticLayout: true,
      readOnly: props.readOnly ?? false,
      fontSize: 12,
      fontFamily: "'SF Mono', 'Consolas', 'Cascadia Code', monospace",
      tabSize: 2,
      wordWrap: 'on',
      folding: true,
      glyphMargin: false,
      lineDecorationsWidth: 0,
      lineNumbersMinChars: 3,
      padding: { top: 8 },
    })

    ed.onDidChangeModelContent(() => {
      emit('update:modelValue', ed.getValue())
    })

    monacoEditor.value = ed
  } catch (e) {
    console.warn('Monaco editor failed to load:', e)
  }
}

watch(() => props.modelValue, (val) => {
  const ed = monacoEditor.value
  if (ed && ed.getValue() !== val) {
    ed.setValue(val)
  }
})

onBeforeUnmount(() => {
  monacoEditor.value?.dispose()
})
</script>

<template>
  <div ref="containerRef" class="monaco-container" @vue:mounted="initEditor()"></div>
</template>

<style scoped>
.monaco-container {
  width: 100%;
  min-height: 150px;
  height: 200px;
  border: 1px solid var(--border);
  border-radius: 4px;
  overflow: hidden;
}
</style>
