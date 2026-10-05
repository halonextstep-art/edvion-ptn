<script setup lang="ts">
// Shared donut chart — replaces the hand-built SVG `stroke-dasharray` donut in
// SchoolRasionalisasi.vue, which only had 6 fixed colors and silently repeated them once
// a school had more than 6 target universities (two segments became indistinguishable
// without hovering). Chart.js gives real interactive tooltips (hover + tap) plus a
// generated color that never repeats (see utils/chartColors), and this component adds a
// center "total" label the old one lacked.
import { Doughnut } from 'vue-chartjs'
import type { ChartData, ChartOptions } from 'chart.js'
import { chartColors } from '~/utils/chartColors'

const props = withDefaults(
  defineProps<{
    labels: string[]
    values: number[]
    unitLabel?: string
    height?: number
    emptyLabel?: string
  }>(),
  { unitLabel: '', height: 200, emptyLabel: 'Belum ada data untuk ditampilkan.' },
)

const total = computed(() => props.values.reduce((s, v) => s + v, 0))
const hasData = computed(() => total.value > 0)
const palette = computed(() => chartColors(props.labels.length))

const chartData = computed<ChartData<'doughnut'>>(() => ({
  labels: props.labels,
  datasets: [{ data: props.values, backgroundColor: palette.value, borderColor: '#fff', borderWidth: 2, hoverOffset: 8 }],
}))

const chartOptions = computed<ChartOptions<'doughnut'>>(() => ({
  responsive: true,
  maintainAspectRatio: false,
  cutout: '68%',
  plugins: {
    legend: { display: false },
    tooltip: {
      backgroundColor: '#1e293b',
      padding: 10,
      cornerRadius: 8,
      callbacks: {
        label: (ctx) => {
          const value = ctx.parsed as number
          const pct = total.value ? Math.round((value / total.value) * 100) : 0
          return ` ${ctx.label}: ${value}${props.unitLabel} (${pct}%)`
        },
      },
    },
  },
}))
</script>

<template>
  <div v-if="hasData" class="flex flex-col sm:flex-row items-center gap-6">
    <div class="relative shrink-0" :style="{ width: `${height}px`, height: `${height}px` }">
      <Doughnut :data="chartData" :options="chartOptions" />
      <div class="absolute inset-0 flex flex-col items-center justify-center pointer-events-none">
        <span class="text-2xl font-black text-slate-900">{{ total }}</span>
        <span class="text-[10px] text-muted-foreground">Total{{ unitLabel ? ` ${unitLabel}` : '' }}</span>
      </div>
    </div>
    <div class="flex-1 w-full space-y-2 max-h-56 overflow-y-auto">
      <div v-for="(label, i) in labels" :key="label" class="flex items-center gap-2 text-sm rounded-lg px-1.5 py-0.5 -mx-1.5">
        <span class="w-2.5 h-2.5 rounded-full shrink-0" :style="{ background: palette[i] }" />
        <span class="flex-1 truncate text-slate-700">{{ label }}</span>
        <span class="text-muted-foreground font-semibold shrink-0">{{ values[i] }}{{ unitLabel }}</span>
      </div>
    </div>
  </div>
  <div v-else class="py-8 text-center text-sm text-muted-foreground">{{ emptyLabel }}</div>
</template>
