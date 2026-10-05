import MarkdownIt from 'markdown-it'
import katex from 'katex'

// markdown-it instantiation isn't free and this renders on every question (list/preview/quiz
// views can render dozens at once) — build the instance once per module load, not per call.
let mdInstance: MarkdownIt | null = null
function getMd(): MarkdownIt {
  if (!mdInstance) {
    mdInstance = new MarkdownIt({
      html: false, // NEVER allow raw HTML through — this text can originate from free-typed
      // admin/content input as well as a bulk .docx import; html:false means markdown-it itself
      // escapes any literal `<`/`>` rather than passing them through as real tags.
      linkify: false, // question content has no reason to auto-link bare URLs
      breaks: true, // a single newline in a <Textarea> should render as a visible line break
    })
  }
  return mdInstance
}

let placeholderSeq = 0

/**
 * `useQuestionMarkdown` — shared "Markdown + LaTeX" renderer for all question content fields
 * (stimulus, question_text, options, explanation). Call once per component (it reads
 * `useRuntimeConfig()`, which must happen inside a Nuxt context) and reuse the returned
 * `renderQuestionMarkdown` function as many times as needed afterwards — e.g. inside a
 * `computed()` or directly in a `v-for`.
 *
 * Technique for combining KaTeX with markdown-it safely: LaTeX delimited by `$$...$$` (block)
 * or `$...$` (inline) is extracted FIRST and rendered to HTML via KaTeX, replaced with an
 * opaque placeholder token, THEN the remaining text runs through markdown-it — finally the
 * placeholders are substituted back with the pre-rendered KaTeX HTML. Order matters: if LaTeX
 * were left in the text while markdown-it ran, characters extremely common inside math
 * (`x_1`, `a^2`, `a*b`) would get mangled by markdown's emphasis/subscript-adjacent rules
 * before KaTeX ever saw them.
 */
export function useQuestionMarkdown() {
  const config = useRuntimeConfig()
  // Backend serves /uploads/... at its own origin, not the Nuxt server's — see
  // composables/useUploadUrl.ts (same derivation, duplicated here so this module has no
  // component-lifecycle dependency beyond the one runtimeConfig read above).
  const apiOrigin = (config.public.apiBase as string).replace(/\/api\/?$/, '')

  function stashKatex(placeholders: Map<string, string>, html: string): string {
    const token = `@@KATEX_${placeholderSeq++}@@`
    placeholders.set(token, html)
    return token
  }

  function renderQuestionMarkdown(source: string | null | undefined): string {
    const text = (source ?? '').trim()
    if (!text) return ''

    const placeholders = new Map<string, string>()

    // Block math $$...$$ must be extracted before inline $...$ so a doubled `$$` isn't
    // mistaken for two empty inline expressions.
    let withPlaceholders = text.replace(/\$\$([\s\S]+?)\$\$/g, (whole, expr: string) => {
      try {
        return stashKatex(placeholders, katex.renderToString(expr.trim(), { throwOnError: false, displayMode: true }))
      } catch {
        return whole
      }
    })

    // Inline math $...$ — excludes a `$` immediately touching whitespace on either side, so a
    // plain currency amount like "biaya sekitar $ 50" in prose text isn't mistaken for a
    // formula opener; also can't span a blank line, so it never accidentally eats a whole
    // paragraph when an author forgets a closing `$`.
    withPlaceholders = withPlaceholders.replace(/\$([^\s$](?:[^$\n]*[^\s$])?)\$/g, (whole, expr: string) => {
      try {
        return stashKatex(placeholders, katex.renderToString(expr.trim(), { throwOnError: false, displayMode: false }))
      } catch {
        return whole
      }
    })

    let html = getMd().render(withPlaceholders)

    for (const [token, katexHtml] of placeholders) {
      html = html.split(token).join(katexHtml)
    }

    // Rewrite relative /uploads/... image src to the backend's real origin.
    html = html.replace(/<img\s+([^>]*?)src="(\/uploads\/[^"]+)"/g, (_m, attrs: string, src: string) => {
      return `<img ${attrs}src="${apiOrigin}${src}"`
    })

    return html
  }

  return { renderQuestionMarkdown }
}
