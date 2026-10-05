<script setup lang="ts">
// Shared renderer for question content fields (stimulus/question_text/options/explanation) —
// interprets the "Markdown + LaTeX" authoring format (see composables/useQuestionMarkdown.ts)
// into safe, styled HTML. Used identically on the student side (tryout & simulasi players) and
// the admin side (Bank Soal, Set Soal preview, PackageWizard soal preview) so a question always
// renders exactly the same way no matter where it's viewed.
import { useQuestionMarkdown } from '~/composables/useQuestionMarkdown'

const props = defineProps<{
  text: string | null | undefined
  /** 'sm' for compact contexts (option rows, list previews); default is the full-size body. */
  size?: 'sm' | 'base'
}>()

const { renderQuestionMarkdown } = useQuestionMarkdown()
const html = computed(() => renderQuestionMarkdown(props.text))
</script>

<template>
  <div v-if="html" class="question-content" :class="size === 'sm' ? 'question-content--sm' : ''" v-html="html" />
</template>

<style scoped>
.question-content {
  line-height: 1.6;
  word-break: break-word;
}
.question-content :deep(p) {
  margin: 0 0 0.6em;
}
.question-content :deep(p:last-child) {
  margin-bottom: 0;
}
.question-content :deep(strong) {
  font-weight: 700;
}
.question-content :deep(ul),
.question-content :deep(ol) {
  margin: 0.4em 0 0.6em;
  padding-left: 1.4em;
}
.question-content :deep(ul) {
  list-style: disc;
}
.question-content :deep(ol) {
  list-style: decimal;
}
.question-content :deep(li) {
  margin: 0.15em 0;
}
.question-content :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: 0.5rem;
  margin: 0.5em 0;
  display: block;
}
.question-content :deep(table) {
  border-collapse: collapse;
  margin: 0.6em 0;
  font-size: 0.92em;
  max-width: 100%;
  display: block;
  overflow-x: auto;
}
.question-content :deep(th),
.question-content :deep(td) {
  border: 1px solid rgba(100, 100, 130, 0.25);
  padding: 0.4em 0.7em;
  text-align: left;
}
.question-content :deep(th) {
  background: rgba(99, 102, 241, 0.1);
  font-weight: 700;
}
.question-content :deep(code) {
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.9em;
  background: rgba(100, 100, 130, 0.12);
  padding: 0.1em 0.35em;
  border-radius: 0.3em;
}
.question-content :deep(pre) {
  background: rgba(100, 100, 130, 0.1);
  padding: 0.7em 0.9em;
  border-radius: 0.5em;
  overflow-x: auto;
  margin: 0.6em 0;
}
.question-content :deep(pre code) {
  background: none;
  padding: 0;
}
.question-content :deep(blockquote) {
  border-left: 3px solid rgba(99, 102, 241, 0.4);
  padding-left: 0.8em;
  margin: 0.6em 0;
  opacity: 0.85;
}
.question-content :deep(.katex-display) {
  margin: 0.6em 0;
  overflow-x: auto;
  overflow-y: hidden;
}
.question-content--sm {
  font-size: 0.85em;
}
</style>
