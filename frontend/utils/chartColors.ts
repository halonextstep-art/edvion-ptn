// Shared color helpers for all Chart.js-based charts — used so a donut/bar chart with
// more series than our curated palette never silently repeats a color (the bug found in
// the old hand-rolled SVG donut, which only had 6 fixed colors and recycled them).

// Curated palette (brand-consistent indigo/purple/emerald family) used first, in order,
// for the common case of a handful of series/segments.
export const CHART_PALETTE = [
  '#6366f1', // indigo-500
  '#8b5cf6', // violet-500
  '#ec4899', // pink-500
  '#f59e0b', // amber-500
  '#10b981', // emerald-500
  '#06b6d4', // cyan-500
  '#f43f5e', // rose-500
  '#84cc16', // lime-500
]

/**
 * Returns `count` visually distinct hex colors. Uses the curated palette for the first
 * `CHART_PALETTE.length` entries, then generates additional evenly-spaced hues around the
 * color wheel so a chart with e.g. 12 universities never reuses a color for two segments.
 */
export function chartColors(count: number): string[] {
  if (count <= CHART_PALETTE.length) return CHART_PALETTE.slice(0, count)
  const extra = count - CHART_PALETTE.length
  const generated = Array.from({ length: extra }, (_, i) => {
    const hue = Math.round((360 / extra) * i)
    return `hsl(${hue}, 70%, 55%)`
  })
  return [...CHART_PALETTE, ...generated]
}

export function withAlpha(hex: string, alpha: number): string {
  if (hex.startsWith('hsl')) return hex.replace('hsl(', 'hsla(').replace(')', `, ${alpha})`)
  const m = /^#([0-9a-f]{6})$/i.exec(hex)
  if (!m) return hex
  const n = parseInt(m[1], 16)
  const r = (n >> 16) & 255
  const g = (n >> 8) & 255
  const b = n & 255
  return `rgba(${r}, ${g}, ${b}, ${alpha})`
}
