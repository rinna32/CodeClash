<script setup lang="ts">
// Редактор кода на CodeMirror 6. Только в браузере (.client): ему нужен DOM.
// CodeMirror, а не Monaco: собирается локально, без web workers и CDN.
import { indentWithTab } from '@codemirror/commands'
import { html } from '@codemirror/lang-html'
import { javascript } from '@codemirror/lang-javascript'
import { python } from '@codemirror/lang-python'
import { EditorState, Prec } from '@codemirror/state'
import { oneDark } from '@codemirror/theme-one-dark'
import { keymap } from '@codemirror/view'
import { basicSetup, EditorView } from 'codemirror'
import type { Mode } from '~/types/Task'

const props = defineProps<{
  mode: Mode
}>()

const code = defineModel<string>({ required: true })

const emit = defineEmits<{
  run: [] // Cmd/Ctrl + Enter
  submit: [] // Cmd/Ctrl + Shift + Enter
}>()

const EDITOR_BACKGROUND = '#282828' // редактор остаётся тёмным, как в Codewars: код на тёмном читать привычнее

const editorRef = ref<HTMLDivElement | null>(null)
let view: EditorView | null = null

function languageFor(mode: Mode) {
  if (mode === 'python') return python()
  if (mode === 'css') return html()
  return javascript()
}

onMounted(async () => {
  // .client-компонент на SSR-странице отрисовывается уже после mounted — ждём тик, иначе ref пустой
  await nextTick()
  if (!editorRef.value) return

  view = new EditorView({
    parent: editorRef.value,
    state: EditorState.create({
      doc: code.value,
      extensions: [
        basicSetup,
        // Prec.highest — иначе Mod-Enter перехватит стандартная «вставка пустой строки»
        Prec.highest(keymap.of([
          { key: 'Mod-Enter', run: () => { emit('run'); return true } },
          { key: 'Mod-Shift-Enter', run: () => { emit('submit'); return true } },
        ])),
        keymap.of([indentWithTab]),
        languageFor(props.mode),
        oneDark,
        // Фон и полосу номеров красим одним цветом — без полосы-разделителя редактор выглядит цельно
        EditorView.theme({
          '&': { height: '100%', fontSize: '14px', backgroundColor: EDITOR_BACKGROUND },
          '.cm-gutters': { backgroundColor: EDITOR_BACKGROUND, borderRight: 'none' },
          '.cm-scroller': { fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace' },
        }, { dark: true }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) code.value = update.state.doc.toString()
        }),
      ],
    }),
  })
  view.focus()
})

// Код поменяли снаружи (сброс к заготовке) — переносим в редактор
watch(code, (value) => {
  if (!view || value === view.state.doc.toString()) return
  view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: value } })
})

onUnmounted(() => {
  view?.destroy()
})
</script>

<template>
  <div ref="editorRef" class="h-full overflow-hidden bg-[#282828]" />
</template>
