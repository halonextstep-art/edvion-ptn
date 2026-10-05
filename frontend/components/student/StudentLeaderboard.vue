<script setup lang="ts">
// Leaderboard — data asli, dihitung langsung dari attempts (skor) atau attempt_answers
// (akurasi per mata uji) di backend (lihat LeaderboardRepository::top_students /
// top_students_by_subject). Scope Nasional/Sekolahku, filter waktu, dan filter mata uji
// semuanya diteruskan sebagai query nyata ke backend — tidak ada data yang dipalsukan
// atau dihitung ulang di client.
import { Trophy, Medal, Loader2, Users, School as SchoolIcon, BarChart3, Flame, Crown, ArrowRight } from 'lucide-vue-next'
import type { LeaderboardEntryItem, LeaderboardScope, LeaderboardRange, SubjectCategoryWithSubjects } from '~/types'

const emit = defineEmits<{ navigate: [tab: string] }>()

const { gamificationService, taxonomyService } = useApi()
const { user } = useAuth()

// Katalog mata uji live dari taksonomi — lihat TaxonomyManager.vue admin.
const categories = ref<SubjectCategoryWithSubjects[]>([])
const firstSubjectName = computed(() => categories.value[0]?.subjects[0]?.name || '')
async function loadCategories() {
  try {
    categories.value = await taxonomyService.list()
    if (!subject.value) subject.value = firstSubjectName.value
  } catch {
    // non-critical — the "Per Subtes" filter just stays empty until this resolves
  }
}

type Mode = 'national' | 'school' | 'subject'
const mode = ref<Mode>('national')
const range = ref<LeaderboardRange>('all_time')
const subject = ref('')

const entries = ref<LeaderboardEntryItem[]>([])
const loading = ref(true)
const errorMsg = ref('')

async function load() {
  loading.value = true
  errorMsg.value = ''
  try {
    const scope: LeaderboardScope = mode.value === 'school' ? 'school' : 'national'
    entries.value = await gamificationService.leaderboard({
      limit: 50,
      scope,
      range: range.value,
      subject: mode.value === 'subject' ? subject.value : undefined,
    })
  } catch (e: any) {
    errorMsg.value = e?.message ?? 'Gagal memuat leaderboard'
  } finally {
    loading.value = false
  }
}
onMounted(() => {
  load()
  loadCategories()
})
watch([mode, range, subject], load)

const myRank = computed(() => entries.value.find((e) => e.student_id === user.value?.id))
const podium = computed(() => entries.value.slice(0, 3))
const rest = computed(() => entries.value.slice(3))
const scoreLabel = computed(() => (mode.value === 'subject' ? '%' : ''))

// Tinggi kolom podium dulu di-hardcode per rank (#1 selalu 96px, #2 64px, #3 48px) tanpa
// melihat skor asli — jadi #1 dengan skor 650 vs #2 dengan skor 649 (beda tipis) kelihatan
// seperti beda jauh, padahal tidak. Sekarang tinggi proporsional terhadap skor #1 (skor
// tertinggi = 100% dari rentang), jadi selisih visual antar kolom mencerminkan selisih
// skor yang sesungguhnya.
const PODIUM_MIN_H = 48
const PODIUM_MAX_EXTRA = 56
function podiumHeight(score: number) {
  const top = podium.value[0]?.best_score || 0
  if (top <= 0) return PODIUM_MIN_H
  return PODIUM_MIN_H + Math.round((Math.max(score, 0) / top) * PODIUM_MAX_EXTRA)
}

function medalColor(rank: number) {
  if (rank === 1) return 'text-yellow-500'
  if (rank === 2) return 'text-slate-400'
  if (rank === 3) return 'text-orange-500'
  return 'text-slate-300'
}
function initials(name: string) {
  return name.split(' ').map((p) => p[0]).slice(0, 2).join('').toUpperCase()
}

const MODE_TABS: { key: Mode; label: string; icon: any }[] = [
  { key: 'national', label: 'Nasional', icon: Users },
  { key: 'school', label: 'Sekolahku', icon: SchoolIcon },
  { key: 'subject', label: 'Per Subtes', icon: BarChart3 },
]
const RANGE_TABS: { key: LeaderboardRange; label: string }[] = [
  { key: 'today', label: 'Hari Ini' },
  { key: 'week', label: 'Minggu Ini' },
  { key: 'all_time', label: 'All Time' },
]

const title = computed(() => {
  if (mode.value === 'school') return 'Leaderboard Sekolahku'
  if (mode.value === 'subject') return `Leaderboard — ${subject.value}`
  return 'Leaderboard Nasional'
})
</script>

<template>
  <div class="space-y-4">
    <Card class="p-5 bg-gradient-to-br from-indigo-600 via-purple-600 to-pink-600 text-white border-0">
      <div class="flex items-center gap-3 mb-4">
        <div class="w-11 h-11 rounded-xl bg-white/20 flex items-center justify-center shrink-0">
          <Trophy class="w-6 h-6" />
        </div>
        <div>
          <h3 class="font-bold text-lg">{{ title }}</h3>
          <p class="text-sm text-indigo-100">Peringkat dihitung langsung dari data asli — diperbarui otomatis, tidak ada yang dipalsukan.</p>
        </div>
        <div v-if="myRank" class="ml-auto text-right shrink-0 hidden sm:block">
          <div class="text-2xl font-black">#{{ myRank.rank }}</div>
          <div class="text-[11px] text-indigo-100">peringkat kamu</div>
        </div>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <div class="inline-flex rounded-lg bg-white/15 p-1 gap-1">
          <button
            v-for="t in MODE_TABS" :key="t.key"
            class="px-3 py-1.5 rounded-md text-xs font-semibold flex items-center gap-1.5 transition-colors"
            :class="mode === t.key ? 'bg-white text-indigo-700' : 'text-white/80 hover:bg-white/10'"
            @click="mode = t.key"
          >
            <component :is="t.icon" class="w-3.5 h-3.5" />{{ t.label }}
          </button>
        </div>
        <div class="inline-flex rounded-lg bg-white/15 p-1 gap-1">
          <button
            v-for="t in RANGE_TABS" :key="t.key"
            class="px-3 py-1.5 rounded-md text-xs font-semibold transition-colors"
            :class="range === t.key ? 'bg-white text-indigo-700' : 'text-white/80 hover:bg-white/10'"
            @click="range = t.key"
          >
            {{ t.label }}
          </button>
        </div>
        <select
          v-if="mode === 'subject'" v-model="subject"
          class="ml-auto text-xs font-semibold rounded-md bg-white/15 border-0 text-white px-2 py-1.5 focus:outline-none focus:ring-2 focus:ring-white/50"
        >
          <optgroup v-for="cat in categories" :key="cat.id" :label="cat.name" class="text-slate-900">
            <option v-for="s in cat.subjects" :key="s.id" :value="s.name">{{ s.name }}</option>
          </optgroup>
        </select>
      </div>
    </Card>

    <div v-if="loading" class="p-10 text-center text-muted-foreground">
      <Loader2 class="w-6 h-6 animate-spin mx-auto mb-2" /> Memuat leaderboard...
    </div>
    <Card v-else-if="errorMsg" class="p-6 text-center text-sm text-red-600">{{ errorMsg }}</Card>
    <Card v-else-if="entries.length === 0" class="p-10 text-center text-muted-foreground text-sm">
      Belum ada data untuk tampilan ini. Selesaikan tryout/drilling untuk masuk leaderboard.
    </Card>

    <template v-else>
      <!-- Podium top 3 — lebar kolom dikecilkan lagi di bawah `sm` (w-20/w-24 dulu, bukan
           langsung w-24/w-28) supaya total 3 kolom + gap tidak mepet/overflow di layar
           ~360-375px (sebelumnya total lebarnya nyaris pas dengan lebar konten Card, gampang
           kepotong 1-2px di device yang lebih sempit dari iPhone standar). -->
      <Card class="p-4 sm:p-6 pt-8 sm:pt-10 bg-gradient-to-b from-indigo-50 to-white overflow-hidden">
        <div class="flex items-end justify-center gap-2 sm:gap-6">
          <!-- #2 -->
          <div v-if="podium[1]" class="flex flex-col items-center w-20 sm:w-28">
            <div class="w-11 h-11 sm:w-14 sm:h-14 rounded-full bg-gradient-to-br from-slate-300 to-slate-400 flex items-center justify-center text-white font-bold mb-2 ring-4 ring-slate-200 text-sm sm:text-base">
              {{ initials(podium[1].student_name) }}
            </div>
            <div class="text-[11px] sm:text-xs font-semibold text-slate-900 text-center truncate w-full">{{ podium[1].student_name }}</div>
            <div class="text-[9px] sm:text-[10px] text-muted-foreground text-center truncate w-full">{{ podium[1].school_name || '-' }}</div>
            <div class="mt-2 w-full rounded-t-lg bg-slate-300 flex flex-col items-center justify-center text-slate-700" :style="{ height: `${podiumHeight(podium[1].best_score)}px` }">
              <span class="font-black text-sm sm:text-base">#2</span>
              <span class="text-[11px] sm:text-xs font-bold">{{ podium[1].best_score }}{{ scoreLabel }}</span>
            </div>
          </div>
          <!-- #1 -->
          <div v-if="podium[0]" class="flex flex-col items-center w-24 sm:w-32">
            <Crown class="w-5 h-5 sm:w-6 sm:h-6 text-yellow-500 mb-1" />
            <div class="w-12 h-12 sm:w-16 sm:h-16 rounded-full bg-gradient-to-br from-yellow-400 to-amber-500 flex items-center justify-center text-white font-bold mb-2 ring-4 ring-yellow-200 text-base">
              {{ initials(podium[0].student_name) }}
            </div>
            <div class="text-xs sm:text-sm font-bold text-slate-900 text-center truncate w-full">{{ podium[0].student_name }}</div>
            <div class="text-[9px] sm:text-[10px] text-muted-foreground text-center truncate w-full">{{ podium[0].school_name || '-' }}</div>
            <div class="mt-2 w-full rounded-t-lg bg-gradient-to-b from-yellow-400 to-amber-500 flex flex-col items-center justify-center text-white" :style="{ height: `${podiumHeight(podium[0].best_score)}px` }">
              <span class="font-black text-base sm:text-lg">#1</span>
              <span class="text-xs sm:text-sm font-bold">{{ podium[0].best_score }}{{ scoreLabel }}</span>
            </div>
          </div>
          <!-- #3 -->
          <div v-if="podium[2]" class="flex flex-col items-center w-20 sm:w-28">
            <div class="w-11 h-11 sm:w-14 sm:h-14 rounded-full bg-gradient-to-br from-orange-300 to-orange-500 flex items-center justify-center text-white font-bold mb-2 ring-4 ring-orange-200 text-sm sm:text-base">
              {{ initials(podium[2].student_name) }}
            </div>
            <div class="text-[11px] sm:text-xs font-semibold text-slate-900 text-center truncate w-full">{{ podium[2].student_name }}</div>
            <div class="text-[9px] sm:text-[10px] text-muted-foreground text-center truncate w-full">{{ podium[2].school_name || '-' }}</div>
            <div class="mt-2 w-full rounded-t-lg bg-orange-400 flex flex-col items-center justify-center text-white" :style="{ height: `${podiumHeight(podium[2].best_score)}px` }">
              <span class="font-black text-sm sm:text-base">#3</span>
              <span class="text-[11px] sm:text-xs font-bold">{{ podium[2].best_score }}{{ scoreLabel }}</span>
            </div>
          </div>
        </div>
      </Card>

      <!-- Rest of table — tabel 5 kolom aslinya cuma ada `overflow-hidden` di Card (bukan
           `overflow-x-auto` pada tabelnya sendiri), jadi di layar <640px kolom "Sekolah" bikin
           tabel kepotong/tidak terbaca. Sekarang: `hidden sm:block` tabel biasa di layar besar,
           diganti daftar kartu ringkas (`sm:hidden`) yang menyusun info yang sama secara
           vertikal per siswa di mobile. -->
      <Card v-if="rest.length > 0" class="overflow-hidden">
        <table class="w-full text-sm hidden sm:table">
          <thead class="bg-slate-50 text-xs uppercase text-slate-500">
            <tr>
              <th class="text-left px-4 py-3 w-16">Rank</th>
              <th class="text-left px-4 py-3">Siswa</th>
              <th class="text-left px-4 py-3">Sekolah</th>
              <th class="text-right px-4 py-3 w-24">Streak</th>
              <th class="text-right px-4 py-3">{{ mode === 'subject' ? 'Akurasi' : 'Skor Terbaik' }}</th>
            </tr>
          </thead>
          <tbody class="divide-y">
            <tr
              v-for="e in rest" :key="e.student_id"
              :class="e.student_id === user?.id ? 'bg-indigo-50' : 'hover:bg-slate-50'"
            >
              <td class="px-4 py-3">
                <div class="flex items-center gap-1.5">
                  <Medal v-if="e.rank <= 3" class="w-4 h-4" :class="medalColor(e.rank)" />
                  <span class="font-bold text-slate-700">#{{ e.rank }}</span>
                </div>
              </td>
              <td class="px-4 py-3 font-medium text-slate-900">
                {{ e.student_name }}
                <Badge v-if="e.student_id === user?.id" variant="secondary" class="ml-1.5">Kamu</Badge>
              </td>
              <td class="px-4 py-3 text-muted-foreground">{{ e.school_name || '-' }}</td>
              <td class="px-4 py-3 text-right text-muted-foreground">
                <span v-if="e.streak > 0" class="inline-flex items-center gap-1"><Flame class="w-3.5 h-3.5 text-orange-500" />{{ e.streak }}d</span>
                <span v-else>-</span>
              </td>
              <td class="px-4 py-3 text-right font-bold text-slate-900">{{ e.best_score }}{{ scoreLabel }}</td>
            </tr>
          </tbody>
        </table>

        <!-- Mobile card-list -->
        <div class="sm:hidden divide-y">
          <div
            v-for="e in rest" :key="e.student_id"
            class="flex items-center gap-3 px-4 py-3"
            :class="e.student_id === user?.id ? 'bg-indigo-50' : ''"
          >
            <div class="flex items-center gap-1 shrink-0 w-10">
              <Medal v-if="e.rank <= 3" class="w-3.5 h-3.5" :class="medalColor(e.rank)" />
              <span class="font-bold text-slate-700 text-sm">#{{ e.rank }}</span>
            </div>
            <div class="min-w-0 flex-1">
              <div class="font-medium text-slate-900 text-sm truncate flex items-center gap-1.5">
                {{ e.student_name }}
                <Badge v-if="e.student_id === user?.id" variant="secondary" class="shrink-0">Kamu</Badge>
              </div>
              <div class="text-xs text-muted-foreground truncate">{{ e.school_name || '-' }}</div>
            </div>
            <div class="text-right shrink-0">
              <div class="font-bold text-slate-900 text-sm">{{ e.best_score }}{{ scoreLabel }}</div>
              <div v-if="e.streak > 0" class="text-[11px] text-muted-foreground inline-flex items-center gap-1 justify-end w-full"><Flame class="w-3 h-3 text-orange-500" />{{ e.streak }}d</div>
            </div>
          </div>
        </div>
      </Card>

      <!-- My rank CTA footer -->
      <Card v-if="myRank" class="p-4 border-0 bg-gradient-to-r from-indigo-600 to-purple-600 text-white flex items-center justify-between gap-3 flex-wrap">
        <div class="flex items-center gap-3">
          <span class="w-10 h-10 rounded-full bg-white/20 font-bold flex items-center justify-center text-sm shrink-0">#{{ myRank.rank }}</span>
          <div>
            <div class="text-sm font-semibold">{{ myRank.student_name }} (Kamu)</div>
            <div class="text-xs text-indigo-100">{{ myRank.school_name || 'Tanpa sekolah' }} · Skor {{ myRank.best_score }}{{ scoreLabel }}</div>
          </div>
        </div>
        <Button size="sm" class="bg-white text-indigo-700 hover:bg-indigo-50 gap-1.5" @click="emit('navigate', 'drilling')">
          Naikkan Rank <ArrowRight class="w-3.5 h-3.5" />
        </Button>
      </Card>
    </template>
  </div>
</template>
