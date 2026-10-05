<script setup lang="ts">
// Radar/spider chart "Pencapaian vs Referensi" — SEMUA data nyata:
//  - `subjects`  : daftar mata pelajaran nyata dari `mapel_syarat` katalog PTN (real,
//                  bukan daftar hardcode) untuk program studi yang sedang dilihat.
//  - `pencapaian`: rata-rata nilai rapor ASLI siswa per mata pelajaran tsb (dicocokkan
//                  dari RaporScoreItem siswa yang sesungguhnya).
//  - `referensi` : SATU angka nyata (pg_snbp/pg_snbt agregat program) diterapkan rata di
//                  semua sumbu sebagai garis pembanding keseluruhan — BUKAN nilai minimum
//                  per mata pelajaran (data itu tidak ada di katalog manapun), makanya
//                  labelnya "Referensi PG Keseluruhan", bukan "Minimum Prodi" per mapel.
//
// Dibangun ulang di atas Chart.js (sebelumnya SVG polygon manual): dapat tooltip asli yang
// tidak numpuk, label sumbu tidak lagi terpotong ke 10 karakter / saling tumpang tindih
// (Chart.js otomatis mengatur jarak & word-wrap label radar), dan transisi/animasi halus.
// Prop API sengaja dipertahankan identik dengan versi lama supaya SchoolRasionalisasi.vue
// dan RasionalisasiSnbpEditor.vue tidak perlu diubah.
import { Radar } from 'vue-chartjs'
import type { ChartData, ChartOptions } from 'chart.js'

const props = defineProps<{
  subjects: string[]
  pencapaian: (number | null)[]
  referensi: number
  maxValue?: number
  /** Lebar kotak chart dalam px (opsional) — dipakai untuk render lebih besar/lega di
   * kontainer yang lebar (mis. dialog detail sekolah). Default tetap 280 supaya caller
   * lama (RasionalisasiSnbpEditor.vue) tidak berubah tampilannya. */
  size?: number
}>()

const max = computed(() => props.maxValue ?? 100)
const boxSize = computed(() => props.size ?? 280)
const boxHeight = computed(() => Math.round(boxSize.value * (260 / 280)))

const chartData = computed<ChartData<'radar'>>(() => ({
  labels: props.subjects,
  datasets: [
    {
      label: 'Pencapaian (rapor asli)',
      data: props.subjects.map((_, i) => props.pencapaian[i] ?? 0),
      backgroundColor: 'rgba(99, 102, 241, 0.25)',
      borderColor: '#6366f1',
      borderWidth: 2,
      pointBackgroundColor: '#6366f1',
      pointBorderColor: '#fff',
      pointRadius: 3.5,
    },
    {
      label: `Referensi PG ${props.referensi}`,
      data: props.subjects.map(() => props.referensi),
      backgroundColor: 'rgba(245, 158, 11, 0.12)',
      borderColor: '#f59e0b',
      borderWidth: 1.5,
      borderDash: [4, 3],
      pointRadius: 0,
    },
  ],
}))

const chartOptions = computed<ChartOptions<'radar'>>(() => ({
  responsive: true,
  maintainAspectRatio: false,
  scales: {
    r: {
      min: 0,
      max: max.value,
      ticks: { display: false, stepSize: max.value / 4 },
      grid: { color: '#e2e8f0' },
      angleLines: { color: '#e2e8f0' },
      pointLabels: { font: { size: 10 }, color: '#64748b' },
    },
  },
  plugins: {
    legend: { display: false },
    tooltip: {
      backgroundColor: '#1e293b',
      padding: 10,
      cornerRadius: 8,
      callbacks: {
        label: (ctx) => {
          if (ctx.datasetIndex === 0 && props.pencapaian[ctx.dataIndex] == null) return `${ctx.label}: belum ada nilai`
          return `${ctx.dataset.label}: ${ctx.formattedValue}`
        },
      },
    },
  },
}))
</script>

<template>
  <div v-if="subjects.length >= 3" class="text-center">
    <div class="w-full mx-auto" :style="{ maxWidth: `${boxSize}px`, height: `${boxHeight}px` }">
      <Radar :data="chartData" :options="chartOptions" />
    </div>
    <div class="flex items-center justify-center gap-4 mt-1 text-[10px] text-muted-foreground">
      <span class="flex items-center gap-1.5"><span class="w-2.5 h-2.5 rounded-full bg-indigo-500 inline-block" /> Pencapaian (rapor asli)</span>
      <span class="flex items-center gap-1.5"><span class="w-2.5 h-2.5 rounded-full border-2 border-amber-500 inline-block" /> Referensi PG {{ referensi }}</span>
    </div>
  </div>
  <div v-else class="text-xs text-muted-foreground text-center py-4">Data mata pelajaran syarat program ini belum cukup untuk grafik.</div>
</template>
