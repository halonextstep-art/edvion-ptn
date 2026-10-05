<script setup lang="ts">
// Shared bar/line/stacked chart used across admin, school, and student portals —
// replaces the ~8 near-identical hand-rolled `<div>` bar charts that used to be
// copy-pasted per page (fixed pixel heights, hover-only tooltips invisible on touch
// devices, a `Math.max(pct, 3)` "zero floor" hack that made 0-value days look nonzero).
// Built on Chart.js via vue-chartjs: real responsive canvas, tooltips that work on both
// hover AND tap (mobile-friendly), a proper zero baseline, and no per-page duplication.
import { Bar } from 'vue-chartjs'
import type { ChartData, ChartOptions } from 'chart.js'
import { chartColors, withAlpha } from '~/utils/chartColors'

export interface ChartSeries {
  label: string
  data: (number | null)[]
  /** Explicit color; falls back to the shared palette (never repeats within one chart). */
  color?: string
  /** 'line' overlays this series as a line on top of the bars (e.g. a target/benchmark). */
  type?: 'bar' | 'line'
  /** Datasets sharing a `stack` id are stacked on top of each other. */
  stack?: string
}

const props = withDefaults(
  defineProps<{
    labels: string[]
    series: ChartSeries[]
    horizontal?: boolean
    /** Appended to every tooltip/axis value, e.g. '%' or ' siswa'. */
    suffix?: string
    height?: number
    yMax?: number
    emptyLabel?: string
    /** When true, the cursor becomes a pointer over bars and `bar-click` becomes emittable. */
    clickable?: boolean
  }>(),
  { horizontal: false, suffix: '', height: 220, emptyLabel: 'Belum ada data untuk ditampilkan.', clickable: false },
)

/** Emits the clicked category's label (e.g. a kelas/rombel name) — only meaningful when
 *  `clickable` is set; consumers use this for chart-driven drill-down (click a bar to filter
 *  a table below it), so this stays a plain label string rather than a numeric index. */
const emit = defineEmits<{ 'bar-click': [label: string] }>()
function handleClick(_event: unknown, elements: { index: number }[]) {
  if (!props.clickable || !elements.length) return
  const label = props.labels[elements[0].index]
  if (label != null) emit('bar-click', label)
}

const hasData = computed(() => props.labels.length > 0 && props.series.some((s) => s.data.some((v) => v != null)))

const palette = computed(() => chartColors(props.series.length))

const chartData = computed<ChartData<'bar' | 'line'>>(() => ({
  labels: props.labels,
  datasets: props.series.map((s, i) => {
    const color = s.color ?? palette.value[i]
    const isLine = s.type === 'line'
    return {
      type: s.type ?? 'bar',
      label: s.label,
      data: s.data,
      stack: s.stack,
      backgroundColor: isLine ? withAlpha(color, 0.15) : withAlpha(color, 0.85),
      borderColor: color,
      borderWidth: isLine ? 2 : 0,
      borderRadius: isLine ? 0 : 6,
      borderSkipped: false,
      maxBarThickness: 36,
      pointRadius: isLine ? 3 : 0,
      pointBackgroundColor: color,
      tension: isLine ? 0.3 : 0,
      fill: isLine ? false : undefined,
    }
  }),
}))

const chartOptions = computed<ChartOptions<'bar'>>(() => ({
  responsive: true,
  maintainAspectRatio: false,
  indexAxis: props.horizontal ? 'y' : 'x',
  interaction: { mode: 'index', intersect: false },
  onClick: props.clickable ? (handleClick as any) : undefined,
  onHover: props.clickable
    ? (event: any, elements: unknown[]) => {
        if (event.native?.target) (event.native.target as HTMLElement).style.cursor = elements.length ? 'pointer' : 'default'
      }
    : undefined,
  plugins: {
    legend: { display: props.series.length > 1, position: 'bottom', labels: { boxWidth: 10, padding: 12, font: { size: 11 } } },
    tooltip: {
      backgroundColor: '#1e293b',
      padding: 10,
      cornerRadius: 8,
      titleFont: { size: 12, weight: 'bold' },
      bodyFont: { size: 12 },
      callbacks: {
        label: (ctx) => `${ctx.dataset.label}: ${ctx.formattedValue}${props.suffix}`,
      },
    },
  },
  scales: {
    x: {
      stacked: props.series.some((s) => s.stack),
      grid: { display: props.horizontal, drawTicks: false },
      ticks: { font: { size: 11 } },
    },
    y: {
      stacked: props.series.some((s) => s.stack),
      beginAtZero: true,
      max: props.yMax,
      grid: { display: !props.horizontal, drawTicks: false },
      ticks: { font: { size: 11 }, callback: (v) => `${v}${props.suffix}` },
    },
  },
}))
</script>

<template>
  <!-- `position: relative` + `width: 100%` + `min-width: 0` are NOT cosmetic here — Chart.js's
       own docs call this out explicitly: with `maintainAspectRatio: false`, the canvas
       resizes off its immediate parent's computed box, and WITHOUT `position: relative` on
       that parent, the resize/ResizeObserver math can misfire and let the canvas grow past
       its actual container instead of shrinking to fit it. That silently blew out the whole
       page's horizontal width wherever this chart sits (visible as unrelated cards further
       down the page getting clipped on the right on mobile, where the overflow is
       proportionally huge vs. a narrow viewport) — this fixes the root cause for every chart
       usage across Admin/School/Student at once rather than patching each page separately.
       `min-width: 0` additionally guards against grid/flex ancestors that would otherwise let
       the item's intrinsic content size force the whole row wider (the "grid blowout" gotcha). -->
  <div v-if="hasData" :style="{ height: `${height}px`, position: 'relative', width: '100%', minWidth: 0 }">
    <Bar :data="chartData as ChartData<'bar'>" :options="chartOptions" />
  </div>
  <div v-else class="py-10 text-center text-sm text-muted-foreground" :style="{ minHeight: `${Math.min(height, 120)}px` }">
    {{ emptyLabel }}
  </div>
</template>
