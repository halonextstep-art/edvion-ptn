<script setup lang="ts">
// Shared authoring widget for every rich-content question field (stimulus, question_text,
// explanation, essay rubric, and per-option text) — a plain Textarea plus a small toolbar for
// inserting Markdown+LaTeX snippets (image / table / formula) at the cursor, and an optional
// live preview using the exact same renderer the student sees (QuestionContent.vue /
// composables/useQuestionMarkdown.ts). Reuses the generic `/uploads/question-image` endpoint —
// previously wired only to the `matching_image` question type's paired options (see
// QuestionFormPanel.vue's uploadMatchingImage) — for inline images inside free-text fields.
import { ImagePlus, Table2, Sigma, Eye, EyeOff, Loader2 } from 'lucide-vue-next'

const props = withDefaults(
  defineProps<{
    modelValue: string
    placeholder?: string
    rows?: number
    /** Some fields (e.g. a single option row) are too narrow/short-lived for a full toolbar+preview. */
    compact?: boolean
  }>(),
  { rows: 4, compact: false },
)
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

const { client } = useApi()
const toast = useToast()

const textareaEl = ref<HTMLTextAreaElement | null>(null)
const uploading = ref(false)
const showPreview = ref(false)

const inputClass =
  'flex w-full rounded-lg border border-input bg-white px-3 py-2 text-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring resize-none dark:bg-white/5 dark:text-white dark:placeholder:text-white/30'

// Inserts `snippet` at the current cursor (replacing any active selection), then re-focuses
// the textarea with the cursor placed right after the inserted text — unless `selectLen` is
// given, in which case the last `selectLen` inserted characters are selected instead (used for
// the formula template, so typing immediately replaces the placeholder expression).
function insertAtCursor(snippet: string, selectLen = 0) {
  const el = textareaEl.value
  const value = props.modelValue || ''
  const start = el?.selectionStart ?? value.length
  const end = el?.selectionEnd ?? value.length
  const next = value.slice(0, start) + snippet + value.slice(end)
  emit('update:modelValue', next)
  nextTick(() => {
    if (!el) return
    el.focus()
    const caretEnd = start + snippet.length
    const caretStart = selectLen > 0 ? caretEnd - selectLen : caretEnd
    el.setSelectionRange(caretStart, caretEnd)
  })
}

function insertTable() {
  insertAtCursor('\n\n| Kolom 1 | Kolom 2 | Kolom 3 |\n| --- | --- | --- |\n| Isi 1 | Isi 2 | Isi 3 |\n\n')
}
function insertFormula() {
  insertAtCursor('$rumus$', 'rumus'.length)
}

const IMAGE_MAX_BYTES = 4 * 1024 * 1024
const IMAGE_ALLOWED_MIME = ['image/png', 'image/jpeg', 'image/webp']

async function uploadImage(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  if (!IMAGE_ALLOWED_MIME.includes(file.type)) {
    toast.error('Format tidak didukung', 'Gunakan PNG, JPEG, atau WEBP.')
    ;(e.target as HTMLInputElement).value = ''
    return
  }
  if (file.size > IMAGE_MAX_BYTES) {
    toast.error('File terlalu besar', 'Maksimum 4 MB.')
    ;(e.target as HTMLInputElement).value = ''
    return
  }
  uploading.value = true
  try {
    const form = new FormData()
    form.append('file', file)
    const res = await client.upload<{ url: string }>('/uploads/question-image', form)
    insertAtCursor(`\n![gambar](${res.url})\n`)
    toast.success('Gambar berhasil disisipkan')
  } catch (err: any) {
    toast.error('Gagal mengunggah gambar', err?.message)
  } finally {
    uploading.value = false
    ;(e.target as HTMLInputElement).value = ''
  }
}
</script>

<template>
  <div class="space-y-1.5">
    <div class="flex items-center gap-1">
      <label class="p-1.5 rounded-lg border border-slate-200 hover:bg-slate-50 cursor-pointer text-slate-500 hover:text-slate-800 dark:border-white/10 dark:hover:bg-white/10 dark:text-white/50 dark:hover:text-white" title="Sisipkan gambar">
        <Loader2 v-if="uploading" class="w-3.5 h-3.5 animate-spin" />
        <ImagePlus v-else class="w-3.5 h-3.5" />
        <input type="file" accept="image/png,image/jpeg,image/webp" class="hidden" :disabled="uploading" @change="uploadImage" />
      </label>
      <button type="button" class="p-1.5 rounded-lg border border-slate-200 hover:bg-slate-50 text-slate-500 hover:text-slate-800 dark:border-white/10 dark:hover:bg-white/10 dark:text-white/50 dark:hover:text-white" title="Sisipkan tabel" @click="insertTable">
        <Table2 class="w-3.5 h-3.5" />
      </button>
      <button type="button" class="p-1.5 rounded-lg border border-slate-200 hover:bg-slate-50 text-slate-500 hover:text-slate-800 dark:border-white/10 dark:hover:bg-white/10 dark:text-white/50 dark:hover:text-white" title="Sisipkan rumus (LaTeX)" @click="insertFormula">
        <Sigma class="w-3.5 h-3.5" />
      </button>
      <button
        v-if="!compact" type="button"
        class="ml-auto flex items-center gap-1 px-2 py-1.5 rounded-lg border text-[10px] font-semibold transition-colors"
        :class="showPreview ? 'border-indigo-300 bg-indigo-50 text-indigo-700 dark:border-violet-500/40 dark:bg-violet-500/20 dark:text-violet-300' : 'border-slate-200 text-slate-500 hover:text-slate-800 dark:border-white/10 dark:text-white/50 dark:hover:text-white'"
        @click="showPreview = !showPreview"
      >
        <EyeOff v-if="showPreview" class="w-3 h-3" /><Eye v-else class="w-3 h-3" />{{ showPreview ? 'Tutup Preview' : 'Preview' }}
      </button>
    </div>

    <textarea
      ref="textareaEl"
      :value="modelValue"
      :placeholder="placeholder"
      :rows="rows"
      :class="inputClass"
      @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    />
    <p class="text-[10px] text-muted-foreground">Mendukung Markdown (**tebal**, tabel, gambar) dan LaTeX (<code>$rumus$</code> / <code>$$rumus$$</code>)</p>

    <div v-if="showPreview && modelValue.trim()" class="p-3 rounded-lg border border-dashed border-indigo-200 bg-indigo-50/40 dark:border-violet-500/20 dark:bg-violet-500/5">
      <p class="text-[10px] font-bold text-indigo-500 dark:text-violet-400 uppercase tracking-wider mb-1.5">Pratinjau</p>
      <QuestionContent :text="modelValue" size="sm" />
    </div>
  </div>
</template>
