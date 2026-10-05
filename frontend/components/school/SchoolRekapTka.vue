<script setup lang="ts">
// Rekap TKA — monitoring progres "Pilih Mapel Pilihan" dan skor per-mata-uji TKA untuk
// setiap siswa. Sebuah "paket TKA" adalah Package dengan exam_track = 'tka' (sinyal eksplisit
// dari admin) — lihat backend application::school_tka_service doc comment. Skor per-mapel
// diambil langsung dari Attempt nyata siswa pada sesi-sesi paket tersebut (baik dikerjakan
// lewat Drilling biasa maupun lewat slot Simulasi TKA — keduanya sama-sama menghasilkan
// Attempt biasa di backend). Skala tampilan (0–100 utk SD/SMP, 200–800 utk SMA/SMK/MA) dan
// ambang kategori "Istimewa" mengikuti jenjang tiap paket (`score_scale_label`/
// `istimewa_threshold`, dihitung backend dari `school_type_scope`) — TIDAK di-hardcode di sini.
import { ListChecks, Loader2, AlertCircle, Search, ChevronRight, X, CheckCircle2, MinusCircle, Sparkles } from 'lucide-vue-next'
import type { TkaPackageGroupItem, TkaStudentRowItem } from '~/types'

const { rationalizationService } = useApi()
const toast = useToast()

const groups = ref<TkaPackageGroupItem[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const search = ref('')
const activePackageId = ref<string | null>(null)

async function loadAll() {
  loading.value = true
  error.value = null
  try {
    groups.value = await rationalizationService.schoolTkaRoster()
    if (groups.value.length && !activePackageId.value) activePackageId.value = groups.value[0].package_id
  } catch (e: any) {
    error.value = e?.message || 'Gagal memuat Rekap TKA.'
    toast.error('Gagal memuat Rekap TKA', error.value)
  } finally {
    loading.value = false
  }
}
onMounted(loadAll)

const activeGroup = computed(() => groups.value.find((g) => g.package_id === activePackageId.value) ?? null)

const filteredStudents = computed(() => {
  if (!activeGroup.value) return []
  const q = search.value.trim().toLowerCase()
  const list = [...activeGroup.value.students]
  return q ? list.filter((s) => s.student_name.toLowerCase().includes(q)) : list
})

function wajibSubjects(row: TkaStudentRowItem) {
  return row.subjects.filter((s) => !s.is_elective)
}
function pilihanSubjects(row: TkaStudentRowItem) {
  return row.subjects.filter((s) => s.is_elective)
}
function avgScaled(subjects: TkaStudentRowItem['subjects']) {
  const scored = subjects.filter((s) => s.scaled_score != null).map((s) => s.scaled_score as number)
  if (!scored.length) return null
  return Math.round(scored.reduce((a, b) => a + b, 0) / scored.length)
}
/** Rata-rata skor mentah platform (0-1000, "Skor Instan") — info pendukung transparansi kecil,
 *  bukan skor resmi TKA. */
function avgRaw(subjects: TkaStudentRowItem['subjects']) {
  const scored = subjects.filter((s) => s.raw_score != null).map((s) => s.raw_score as number)
  if (!scored.length) return null
  return Math.round(scored.reduce((a, b) => a + b, 0) / scored.length)
}
function istimewaCount(row: TkaStudentRowItem) {
  return row.subjects.filter((s) => s.is_istimewa).length
}
function attemptedCount(row: TkaStudentRowItem) {
  return row.subjects.filter((s) => s.attempted).length
}
function electiveReady(row: TkaStudentRowItem) {
  if (!activeGroup.value) return true
  return row.elective_chosen_count >= activeGroup.value.elective_pick_count
}

// ── Drill-down per siswa ──
const detailRow = ref<TkaStudentRowItem | null>(null)
function openDetail(row: TkaStudentRowItem) {
  detailRow.value = row
}
function closeDetail() {
  detailRow.value = null
}
function formatDate(iso: string | null) {
  if (!iso) return '—'
  return new Date(iso).toLocaleDateString('id-ID', { day: 'numeric', month: 'short', year: 'numeric' })
}
</script>

<template>
  <div class="space-y-5">
    <!-- Hero -->
    <div class="relative overflow-hidden rounded-2xl bg-gradient-to-br from-violet-600 via-purple-600 to-indigo-700 p-6 text-white">
      <div class="absolute inset-0 opacity-10" style="background-image: radial-gradient(circle, white 1px, transparent 1px); background-size: 16px 16px;" />
      <div class="relative flex items-center justify-between flex-wrap gap-4">
        <div>
          <p class="text-xs font-semibold uppercase tracking-wide text-violet-100 mb-1">Monitoring TKA</p>
          <h3 class="text-xl font-black flex items-center gap-2"><ListChecks class="w-5 h-5" /> Rekap TKA</h3>
          <p class="text-sm text-violet-100 mt-1">Pantau progres pilih mata uji pilihan dan hasil TKA setiap siswa, per paket (jenjang).</p>
        </div>
      </div>
    </div>

    <div v-if="loading" class="py-16 text-center text-muted-foreground"><Loader2 class="w-6 h-6 animate-spin mx-auto mb-2" /> Memuat Rekap TKA...</div>
    <Card v-else-if="error" class="p-8 text-center border-red-200 bg-red-50">
      <AlertCircle class="w-8 h-8 mx-auto mb-2 text-red-500" />
      <p class="text-sm text-red-700 mb-3">{{ error }}</p>
      <Button variant="outline" size="sm" @click="loadAll">Coba Lagi</Button>
    </Card>
    <Card v-else-if="groups.length === 0" class="p-10 text-center text-sm text-muted-foreground">
      <ListChecks class="w-8 h-8 mx-auto mb-2 text-slate-300" />
      Belum ada paket TKA (dengan mapel pilihan) yang dikonfigurasi di sistem.
    </Card>

    <template v-else>
      <!-- Package tabs (biasanya "TKA SMA" / "TKA SMP") -->
      <div class="flex items-center gap-2 flex-wrap">
        <button
          v-for="g in groups" :key="g.package_id"
          class="px-3.5 py-1.5 rounded-lg text-xs font-semibold border transition-colors"
          :class="activePackageId === g.package_id ? 'bg-violet-600 text-white border-violet-600' : 'bg-white text-slate-600 hover:bg-slate-50'"
          @click="activePackageId = g.package_id"
        >
          {{ g.package_name }} <span class="opacity-70">({{ g.students.length }} siswa)</span>
        </button>
      </div>

      <template v-if="activeGroup">
        <div class="flex flex-col sm:flex-row gap-2">
          <div class="relative flex-1">
            <Search class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
            <Input v-model="search" placeholder="Cari nama siswa..." class="pl-9" />
          </div>
          <span v-if="activeGroup.elective_pick_count > 0" class="flex items-center gap-1.5 text-xs text-muted-foreground px-2">
            Wajib memilih <strong class="text-slate-700">{{ activeGroup.elective_pick_count }}</strong> mata uji pilihan
          </span>
          <span class="flex items-center gap-1.5 text-xs text-muted-foreground px-2">
            Skala nilai: <strong class="text-slate-700">{{ activeGroup.score_scale_label }}</strong> · Istimewa ≥ <strong class="text-slate-700">{{ activeGroup.istimewa_threshold }}</strong>
          </span>
        </div>

        <Card class="overflow-hidden">
          <div v-if="activeGroup.students.length === 0" class="p-10 text-center text-sm text-muted-foreground">
            Belum ada siswa untuk paket ini.
          </div>
          <div v-else-if="filteredStudents.length === 0" class="p-10 text-center text-sm text-muted-foreground">Tidak ada siswa yang cocok dengan pencarian.</div>
          <div v-else class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead class="bg-slate-50 text-xs uppercase text-slate-500">
                <tr>
                  <th class="text-left px-4 py-3">Siswa</th>
                  <th class="text-center px-4 py-3">Mapel Pilihan</th>
                  <th class="text-center px-4 py-3">Mapel Dikerjakan</th>
                  <th class="text-left px-4 py-3">Rata² Wajib</th>
                  <th class="text-left px-4 py-3">Rata² Pilihan</th>
                  <th class="text-center px-4 py-3">Istimewa</th>
                  <th class="text-right px-4 py-3">Detail</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-50">
                <tr v-for="r in filteredStudents" :key="r.student_id" class="hover:bg-slate-50">
                  <td class="px-4 py-3">
                    <div class="flex items-center gap-2.5">
                      <div class="w-8 h-8 rounded-full bg-violet-100 text-violet-700 flex items-center justify-center text-xs font-bold shrink-0">
                        {{ r.student_name.split(' ').map((w) => w[0]).slice(0, 2).join('').toUpperCase() }}
                      </div>
                      <div class="font-semibold text-slate-900">{{ r.student_name }}</div>
                    </div>
                  </td>
                  <td class="px-4 py-3 text-center">
                    <span
                      class="inline-flex items-center gap-1 px-2 py-1 rounded-full text-[11px] font-semibold"
                      :class="electiveReady(r) ? 'bg-emerald-100 text-emerald-700' : 'bg-amber-100 text-amber-700'"
                    >
                      <CheckCircle2 v-if="electiveReady(r)" class="w-3 h-3" /><MinusCircle v-else class="w-3 h-3" />
                      {{ r.elective_chosen_count }}/{{ activeGroup.elective_pick_count }}
                    </span>
                  </td>
                  <td class="px-4 py-3 text-center text-slate-700">{{ attemptedCount(r) }}/{{ r.subjects.length }}</td>
                  <td class="px-4 py-3">
                    <span v-if="avgScaled(wajibSubjects(r)) != null" class="font-semibold text-slate-900">{{ avgScaled(wajibSubjects(r)) }}</span>
                    <span v-else class="text-xs text-muted-foreground italic">Belum ada</span>
                    <span v-if="avgRaw(wajibSubjects(r)) != null" class="block text-[10px] text-muted-foreground">skor mentah {{ avgRaw(wajibSubjects(r)) }}/1000</span>
                  </td>
                  <td class="px-4 py-3">
                    <span v-if="avgScaled(pilihanSubjects(r)) != null" class="font-semibold text-slate-900">{{ avgScaled(pilihanSubjects(r)) }}</span>
                    <span v-else class="text-xs text-muted-foreground italic">Belum ada</span>
                    <span v-if="avgRaw(pilihanSubjects(r)) != null" class="block text-[10px] text-muted-foreground">skor mentah {{ avgRaw(pilihanSubjects(r)) }}/1000</span>
                  </td>
                  <td class="px-4 py-3 text-center">
                    <span v-if="istimewaCount(r) > 0" class="inline-flex items-center gap-1 px-2 py-1 rounded-full text-[11px] font-semibold bg-amber-100 text-amber-700">
                      <Sparkles class="w-3 h-3" /> {{ istimewaCount(r) }}/{{ r.subjects.length }}
                    </span>
                    <span v-else class="text-xs text-muted-foreground">—</span>
                  </td>
                  <td class="px-4 py-3 text-right">
                    <button class="p-1.5 rounded-lg hover:bg-violet-50 text-slate-400 hover:text-violet-600" title="Detail per mata uji" @click="openDetail(r)">
                      <ChevronRight class="w-4 h-4" />
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <div v-if="activeGroup.students.length > 0" class="px-4 py-2.5 text-xs text-muted-foreground border-t bg-slate-50/50">
            Menampilkan {{ filteredStudents.length }} dari {{ activeGroup.students.length }} siswa. Nilai ditampilkan pada skala {{ activeGroup.score_scale_label }}
            (kategori "Istimewa" mulai {{ activeGroup.istimewa_threshold }}) — bukan skor resmi TKA dari Kemendikdasmen, melainkan perkiraan platform berdasarkan hasil pengerjaan siswa.
            Angka "skor mentah" kecil di bawahnya menunjukkan skor asli platform (skala 0–1000) sebelum dikonversi.
          </div>
        </Card>
      </template>
    </template>

    <!-- Detail per mata uji -->
    <Dialog :model-value="!!detailRow" title="Detail Skor TKA" max-width="max-w-lg" @update:model-value="(v: boolean) => { if (!v) closeDetail() }">
      <div v-if="detailRow" class="space-y-4">
        <div class="flex items-center justify-between">
          <p class="text-sm font-semibold text-slate-900">{{ detailRow.student_name }}</p>
          <button class="p-1 rounded hover:bg-slate-100" @click="closeDetail"><X class="w-4 h-4 text-slate-400" /></button>
        </div>

        <div>
          <p class="text-xs font-bold text-slate-500 uppercase tracking-wide mb-2">Mata Uji Wajib</p>
          <div class="space-y-1.5">
            <div v-for="s in wajibSubjects(detailRow)" :key="s.session_id" class="flex items-center justify-between p-2.5 rounded-lg border text-sm" :class="s.is_istimewa ? 'border-amber-200 bg-amber-50/50' : ''">
              <span class="text-slate-700 truncate flex items-center gap-1.5">
                {{ s.session_title }}
                <Sparkles v-if="s.is_istimewa" class="w-3.5 h-3.5 text-amber-500 shrink-0" />
              </span>
              <span v-if="s.attempted" class="text-right shrink-0 ml-2">
                <span class="font-bold text-slate-900">{{ s.scaled_score }}</span>
                <span class="block text-[10px] text-muted-foreground">skor mentah {{ s.raw_score }}/1000 · {{ formatDate(s.submitted_at) }}</span>
              </span>
              <span v-else class="text-xs text-slate-400 italic shrink-0 ml-2">Belum dikerjakan</span>
            </div>
          </div>
        </div>

        <div>
          <p class="text-xs font-bold text-slate-500 uppercase tracking-wide mb-2">
            Mata Uji Pilihan ({{ detailRow.elective_chosen_count }}/{{ activeGroup?.elective_pick_count ?? '-' }} dipilih)
          </p>
          <div v-if="pilihanSubjects(detailRow).length === 0" class="text-xs text-muted-foreground italic p-2.5">Tidak ada mata uji pilihan pada paket ini.</div>
          <div v-else class="space-y-1.5 max-h-60 overflow-y-auto">
            <div v-for="s in pilihanSubjects(detailRow)" :key="s.session_id" class="flex items-center justify-between p-2.5 rounded-lg border text-sm" :class="s.is_istimewa ? 'border-amber-200 bg-amber-50/50' : ''">
              <span class="text-slate-700 truncate flex items-center gap-1.5">
                {{ s.session_title }}
                <Sparkles v-if="s.is_istimewa" class="w-3.5 h-3.5 text-amber-500 shrink-0" />
              </span>
              <span v-if="s.attempted" class="text-right shrink-0 ml-2">
                <span class="font-bold text-slate-900">{{ s.scaled_score }}</span>
                <span class="block text-[10px] text-muted-foreground">skor mentah {{ s.raw_score }}/1000 · {{ formatDate(s.submitted_at) }}</span>
              </span>
              <span v-else class="text-xs text-slate-400 italic shrink-0 ml-2">Belum dikerjakan</span>
            </div>
          </div>
        </div>

        <p class="text-[11px] text-muted-foreground border-t pt-3">
          Skor ditampilkan pada skala {{ activeGroup?.score_scale_label }} — perkiraan platform hasil rescale dari skor mentah 0–1000, bukan skor resmi TKA dari
          Kemendikdasmen. Ikon <Sparkles class="w-3 h-3 inline text-amber-500" /> menandai mata uji yang mencapai kategori "Istimewa" (≥ {{ activeGroup?.istimewa_threshold }}).
        </p>
      </div>
    </Dialog>
  </div>
</template>
