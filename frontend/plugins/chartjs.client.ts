// Registers Chart.js once, client-side only (canvas rendering has no meaning during SSR,
// and Chart.js touches `window`/`Image` at import time in some builds) — the `.client.ts`
// suffix makes Nuxt skip this plugin entirely on the server.
import { Chart, registerables } from 'chart.js'

export default defineNuxtPlugin(() => {
  Chart.register(...registerables)
  // Sensible app-wide defaults so every chart looks consistent without repeating options.
  Chart.defaults.font.family =
    "ui-sans-serif, system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif"
  Chart.defaults.color = '#64748b' // slate-500, matches muted-foreground used elsewhere
  Chart.defaults.borderColor = '#e2e8f0' // slate-200 gridlines
})
