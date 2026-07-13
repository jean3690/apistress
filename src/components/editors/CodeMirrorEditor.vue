<script setup lang="ts">
import { watch, onMounted, onBeforeUnmount, shallowRef, useTemplateRef } from 'vue'
import {
  EditorView,
  keymap,
  lineNumbers,
  highlightSpecialChars,
  drawSelection,
  highlightActiveLine,
  rectangularSelection,
  crosshairCursor,
  dropCursor,
} from '@codemirror/view'
import { EditorState, Compartment } from '@codemirror/state'
import { defaultKeymap, history, historyKeymap, indentMore, indentLess } from '@codemirror/commands'
import { json } from '@codemirror/lang-json'
import { graphqlLanguageSupport } from 'cm6-graphql'
import {
  bracketMatching,
  indentOnInput,
  syntaxHighlighting,
  defaultHighlightStyle,
  foldGutter,
  foldKeymap,
} from '@codemirror/language'
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search'
import {
  autocompletion,
  completionKeymap,
  closeBracketsKeymap,
  completeAnyWord,
  type CompletionContext,
} from '@codemirror/autocomplete'

const props = withDefaults(
  defineProps<{
    modelValue: string
    readOnly?: boolean
    language?: 'json' | 'graphql' | 'text'
  }>(),
  {
    readOnly: false,
    language: 'json',
  },
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const containerRef = useTemplateRef<HTMLElement>('container')
const editorView = shallowRef<EditorView | null>(null)
let inited = false

const languageConf = new Compartment()
const readOnlyConf = new Compartment()

function langExtension(lang: string) {
  if (lang === 'json') return json()
  if (lang === 'graphql') return graphqlLanguageSupport()
  return []
}

// ---- Completion providers ----

function jsonCompletions(context: CompletionContext) {
  const pos = context.pos
  const options = [
    { label: '"key": "value"', type: 'property' as const, apply: '\n\t"key": "value",\n}', detail: 'String value' },
    { label: '"key": 0', type: 'property' as const, apply: '\n\t"key": 0,\n}', detail: 'Number value' },
    { label: '"key": true', type: 'property' as const, apply: '\n\t"key": true,\n}', detail: 'Boolean value' },
    { label: '"key": null', type: 'property' as const, apply: '\n\t"key": null,\n}', detail: 'Null value' },
    { label: '"key": {}', type: 'property' as const, apply: '\n\t"key": {},\n}', detail: 'Object value' },
    { label: '"key": []', type: 'property' as const, apply: '"key": []', detail: 'Array value' },
    { label: '{}', type: 'keyword' as const, apply: '}', detail: 'Empty object' },
    { label: '[]', type: 'keyword' as const, apply: ']', detail: 'Empty array' },
  ]

  const word = context.matchBefore(/\w*/)
  if (word && word.from !== word.to) {
    const prefix = context.state.sliceDoc(word.from, pos).toLowerCase()
    const filtered = options.filter(o => o.label.toLowerCase().startsWith(prefix))
    if (filtered.length) return { from: word.from, options: filtered }
  }

  return { from: pos, options }
}

function variableCompletions(context: CompletionContext) {
  const pos = context.pos
  const line = context.state.doc.lineAt(pos)
  const textBefore = line.text.slice(0, pos - line.from)

  if (textBefore.endsWith('${')) {
    return {
      from: pos,
      options: [
        { label: '__threadNum', type: 'function' as const, detail: 'Current thread number (1-indexed)' },
        { label: '__time()', type: 'function' as const, detail: 'Current timestamp in milliseconds' },
      ],
    }
  }

  if (textBefore.endsWith('{{')) {
    return {
      from: pos,
      options: [
        { label: 'variableName', type: 'variable' as const, detail: 'Variable Interpolation', apply: 'variableName}}' },
      ],
    }
  }

  if (textBefore.endsWith('{')) {
    return {
      from: pos,
      options: [
        { label: '{{variable}}', type: 'variable' as const, detail: 'Variable Interpolation', apply: '{variable}}' },
      ],
    }
  }

  if (textBefore.includes('{{') && !textBefore.includes('}}', textBefore.lastIndexOf('{{'))) {
    return {
      from: pos,
      options: [{ label: '}}', type: 'variable' as const, detail: 'Close variable reference', apply: '}}' }],
    }
  }

  return null
}

function graphqlCompletions(context: CompletionContext) {
  const word = context.matchBefore(/\w*/)
  const from = word ? word.from : context.pos

  const options = [
    { label: 'query', type: 'keyword' as const, detail: 'GraphQL Query', apply: 'query name {\n  \n}' },
    { label: 'mutation', type: 'keyword' as const, detail: 'GraphQL Mutation', apply: 'mutation name {\n  \n}' },
    {
      label: 'subscription',
      type: 'keyword' as const,
      detail: 'GraphQL Subscription',
      apply: 'subscription name {\n  \n}',
    },
    {
      label: 'fragment',
      type: 'keyword' as const,
      detail: 'GraphQL Fragment',
      apply: 'fragment name on Type {\n  \n}',
    },
    { label: '{ }', type: 'keyword' as const, detail: 'Shorthand query', apply: '{\n  \n}' },
  ]
  return { from, options }
}

function completionSource(context: CompletionContext) {
  const lang = props.language
  let custom = null
  if (lang === 'json') custom = jsonCompletions(context) || variableCompletions(context)
  else if (lang === 'graphql') custom = graphqlCompletions(context) || variableCompletions(context)
  else custom = variableCompletions(context)
  return custom || completeAnyWord(context)
}

// ---- Theme ----

const appTheme = EditorView.theme(
  {
    '&': { backgroundColor: '#1e1e2e', color: '#cdd6f4', fontSize: '12px' },
    '&.cm-focused': { outline: 'none' },
    '.cm-scroller': { fontFamily: "'Cascadia Code', 'JetBrains Mono', 'Fira Code', 'Consolas', monospace" },
    '.cm-content': { caretColor: '#cdd6f4', padding: '8px 0' },
    '.cm-gutters': { backgroundColor: '#181825', color: '#6c7086', border: 'none', fontSize: '11px' },
    '.cm-lineNumbers .cm-gutterElement': { padding: '0 8px 0 4px', minWidth: '28px' },
    '.cm-activeLineGutter': { backgroundColor: '#313244', color: '#a6adc8' },
    '.cm-activeLine': { backgroundColor: 'rgba(255,255,255,0.03)' },
    '.cm-selectionBackground, ::selection': { backgroundColor: 'rgba(240,160,64,0.25)' },
    '.cm-cursor': { borderLeftColor: '#cdd6f4' },
    '.cm-matchingBracket': { backgroundColor: 'rgba(240,160,64,0.12)', outline: '1px solid #585b70' },
    '.cm-tooltip': { backgroundColor: '#313244', border: '1px solid #45475a', color: '#cdd6f4', fontSize: '11px' },
    '.cm-tooltip-autocomplete li': { padding: '3px 8px' },
    '.cm-tooltip-autocomplete li[aria-selected]': { backgroundColor: '#45475a' },
    '.cm-completionDetail': { color: '#6c7086', fontStyle: 'normal' },
    '.cm-completionMatchedText': { color: '#f0a040', fontWeight: '600' },
    '.cm-foldPlaceholder': { backgroundColor: '#313244', color: '#6c7086', border: '1px solid #45475a' },
  },
  { dark: true },
)

// ---- Init ----

function initEditor() {
  if (!containerRef.value || inited) return
  inited = true

  const view = new EditorView({
    doc: props.modelValue,
    extensions: [
      // Editor view
      lineNumbers(),
      highlightSpecialChars(),
      highlightActiveLine(),
      highlightSelectionMatches(),
      drawSelection(),
      dropCursor(),
      rectangularSelection(),
      crosshairCursor(),
      // Editing
      history(),
      indentOnInput(),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      bracketMatching(),
      foldGutter(),
      // Keymaps
      keymap.of([
        ...defaultKeymap,
        ...historyKeymap,
        ...foldKeymap,
        ...completionKeymap,
        ...closeBracketsKeymap,
        ...searchKeymap,
        { key: 'Tab', run: indentMore, shift: indentLess },
        { key: 'Shift-Tab', run: indentLess },
      ]),
      // Autocomplete: custom sources with word fallback
      autocompletion({
        override: [completionSource],
        defaultKeymap: true,
        activateOnTyping: true,
      }),
      // Language
      languageConf.of(langExtension(props.language)),
      // Read-only
      readOnlyConf.of(props.readOnly ? [EditorState.readOnly.of(true), EditorView.editable.of(false)] : []),
      // Theme
      appTheme,
      // v-model
      EditorView.updateListener.of(update => {
        if (update.docChanged) {
          emit('update:modelValue', update.state.doc.toString())
        }
      }),
    ],
    parent: containerRef.value,
  })

  editorView.value = view
}

// ---- Reactive updates ----

watch(
  () => props.language,
  lang => {
    const view = editorView.value
    if (!view) return
    view.dispatch({ effects: languageConf.reconfigure(langExtension(lang)) })
  },
)

watch(
  () => props.modelValue,
  val => {
    const view = editorView.value
    if (!view) return
    const current = view.state.doc.toString()
    if (current !== val) {
      view.dispatch({ changes: { from: 0, to: current.length, insert: val } })
    }
  },
)

watch(
  () => props.readOnly,
  ro => {
    const view = editorView.value
    if (!view) return
    view.dispatch({
      effects: readOnlyConf.reconfigure(ro ? [EditorState.readOnly.of(true), EditorView.editable.of(false)] : []),
    })
  },
)

onBeforeUnmount(() => {
  editorView.value?.destroy()
  editorView.value = null
})

onMounted(() => {
  initEditor()
})
</script>

<template>
  <div ref="container" class="cm-container" />
</template>

<style scoped>
.cm-container {
  width: 100%;
  min-height: 150px;
  height: 100%;
  border: 1px solid var(--border);
  border-radius: 3px;
}
.cm-container .cm-editor {
  border-radius: 3px;
  height: 100%;
  outline: none;
}
</style>
