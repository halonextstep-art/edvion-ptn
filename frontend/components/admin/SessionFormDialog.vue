<script setup lang="ts">
import { Sparkles, Shuffle, ListChecks } from 'lucide-vue-next'
import type { ExamTrack, SchoolType, SessionType, TryoutSessionTemplate, SubjectCategoryWithSubjects, QuestionSetItem } from '~/types'

const open = defineModel<boolean>({ default: false })
const props = defineProps<{ editSession?: TryoutSessionTemplate | null }>()
const emit = defineEmits<{ saved: [] }>()

const { tryoutService, taxonomyService, questionSetService } = useApi()
const toast = useToast()

// Katalog mata uji live dari taksonomi — lihat TaxonomyManager.vue admin. Filter tetap
// mengirim nama mata uji (string polos), sama seperti sebelumnya.
const categories = ref<SubjectCategoryWithSubjects[]>([])
onMounted(async () => {
  try {
    categories.value = await taxonomyService.list()
  } catch (e: any) {
    toast.error('Gagal memuat daftar mata uji', e?.message)
  }
})

// Katalog Set Soal — lihat admin/QuestionSetManager.vue. Dipakai mode "Set Soal Tetap".
const sets = ref<QuestionSetItem[]>([])
onMounted(async () => {
  try {
    sets.value = await questionSetService.list()
  } catch (e: any) {
    toast.error('Gagal memuat daftar Set Soal', e?.message)
  }
})
const selectedSet = computed(() => sets.value.find((s) => s.id === selectedSetId.value) || null)

const title = ref('')
const sessionType = ref<SessionType>('drilling')
const durationMinutes = ref(30)
const questionCount = ref(10)
const subjectFilter = ref('')
const topicFilter = ref('')
const difficultyFilter = ref('')
const isPremium = ref(false)
// Dipertahankan (bukan field yang diedit lewat form ini) supaya menyimpan sesi draft/mapel-
// pilihan lewat dialog generik ini tidak diam-diam mereset keduanya ke false — lihat
// PackageWizard.vue untuk alur pembuatan aslinya (is_draft) dan checkbox "Mapel Pilihan" di
// bawah untuk is_elective.
const isDraft = ref(false)
const isElective = ref(false)
// Jalur ujian eksplisit — dipakai DrillingZone (siswa) untuk memisahkan tampilan SNBT/UTBK vs
// TKA. Sesi berdiri sendiri (dibuat lewat dialog ini, bukan lewat PackageWizard) tidak punya
// paket sebagai sumber sinyal ini, jadi harus dipilih langsung di sini.
const examTrack = ref<ExamTrack>('snbt')
// `null` = tampil ke siswa jenjang apapun (default, cocok untuk konten SNBT/UTBK umum). Isi
// kalau sesi ini memang khusus satu jenjang (mis. drilling/mini tryout TKA SMP) — siswa
// jenjang lain tidak akan melihat sesi ini sama sekali di katalognya. Lihat
// TryoutSessionTemplate.school_type_scope doc comment & migration
// 20250101000039_content_school_type_scope.sql.
const schoolTypeScope = ref<SchoolType | null>(null)
const saving = ref(false)

// Mode "random" = perilaku lama (undian dari bank soal via filter mata uji/topik/kesulitan/
// jumlah soal — TIDAK DIUBAH sama sekali). Mode "set" = soal tetap dari Set Soal terkurasi.
const mode = ref<'random' | 'set'>('random')
const selectedSetId = ref('')

const isEditing = computed(() => !!props.editSession)

function resetForm(s?: TryoutSessionTemplate | null) {
  title.value = s?.title || ''
  sessionType.value = s?.session_type || 'drilling'
  durationMinutes.value = s?.duration_minutes || 30
  questionCount.value = s?.question_count || 10
  subjectFilter.value = s?.subject_filter || ''
  topicFilter.value = s?.topic_filter || ''
  difficultyFilter.value = s?.difficulty_filter || ''
  isPremium.value = s?.is_premium || false
  isDraft.value = s?.is_draft || false
  isElective.value = s?.is_elective || false
  examTrack.value = s?.exam_track || 'snbt'
  schoolTypeScope.value = s?.school_type_scope || null
  // Sesi yang sudah punya question_set_id dibuka langsung dalam mode "Set Soal Tetap"
  // dengan set-nya terpilih; sesi baru atau sesi tanpa question_set_id selalu mode "random".
  if (s?.question_set_id) {
    mode.value = 'set'
    selectedSetId.value = s.question_set_id
  } else {
    mode.value = 'random'
    selectedSetId.value = ''
  }
}

watch(open, (isOpen) => {
  if (isOpen) resetForm(props.editSession)
})

async function save() {
  if (!title.value.trim()) return toast.error('Judul sesi tidak boleh kosong!')
  if (mode.value === 'set' && !selectedSetId.value) return toast.error('Pilih Set Soal terlebih dahulu!')
  if (mode.value === 'random' && questionCount.value <= 0) return toast.error('Jumlah soal harus lebih dari 0!')
  if (durationMinutes.value <= 0) return toast.error('Durasi harus lebih dari 0!')

  saving.value = true
  try {
    const payload = {
      title: title.value,
      session_type: sessionType.value,
      duration_minutes: durationMinutes.value,
      question_count: mode.value === 'set' ? (selectedSet.value?.item_count ?? questionCount.value) : questionCount.value,
      subject_filter: subjectFilter.value || null,
      topic_filter: topicFilter.value || null,
      difficulty_filter: difficultyFilter.value || null,
      is_premium: isPremium.value,
      // Wajib eksplisit null di mode random, bukan sekadar dihilangkan — supaya sesi yang
      // sebelumnya dipasangi Set Soal lalu dikembalikan ke mode acak benar-benar terlepas
      // dari set lamanya di backend.
      question_set_id: mode.value === 'set' ? selectedSetId.value : null,
      is_draft: isDraft.value,
      is_elective: isElective.value,
      exam_track: examTrack.value,
      school_type_scope: schoolTypeScope.value,
    }
    if (isEditing.value && props.editSession) {
      await tryoutService.updateSession(props.editSession.id, payload)
      toast.success('Sesi berhasil diperbarui!')
    } else {
      await tryoutService.createSession(payload)
      toast.success('Sesi baru berhasil dibuat!')
    }
    open.value = false
    emit('saved')
  } catch (e: any) {
    toast.error('Gagal menyimpan sesi', e?.message)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <Dialog v-model="open" :title="isEditing ? 'Edit Sesi' : 'Buat Sesi Baru'" max-width="max-w-lg">
    <div class="space-y-4">
      <div>
        <Label>Judul Sesi</Label>
        <Input v-model="title" placeholder="Contoh: Drilling Matematika - Level Sedang" class="mt-1.5" />
      </div>
      <div class="grid sm:grid-cols-2 gap-4">
        <div>
          <Label>Tipe</Label>
          <select v-model="sessionType" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
            <option value="tryout">Tryout (Full Simulasi)</option>
            <option value="drilling">Drilling</option>
            <option value="mini">Mini Tryout</option>
          </select>
        </div>
        <div>
          <Label>Durasi (menit)</Label>
          <Input v-model.number="durationMinutes" type="number" min="1" class="mt-1.5" />
        </div>
      </div>

      <div class="grid sm:grid-cols-2 gap-4">
        <div>
          <Label>Jalur Ujian</Label>
          <select v-model="examTrack" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
            <option value="snbt">SNBT / UTBK</option>
            <option value="tka">TKA</option>
          </select>
        </div>
        <div>
          <Label>Jenjang (opsional)</Label>
          <select v-model="schoolTypeScope" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
            <option :value="null">Semua jenjang</option>
            <option value="sma">SMA</option>
            <option value="smk">SMK</option>
            <option value="ma">MA</option>
            <option value="smp">SMP</option>
          </select>
        </div>
      </div>
      <p class="text-xs text-muted-foreground -mt-2">
        Kosongkan ("Semua jenjang") untuk konten umum. Isi kalau sesi ini khusus satu jenjang (mis. drilling/mini tryout TKA SMP) — siswa jenjang lain tidak akan melihat sesi ini di katalognya sama sekali.
      </p>

      <!-- Mode: undian acak dari bank soal (perilaku lama, tidak diubah) vs Set Soal
           terkurasi yang tetap (baru) — lihat catatan `mode` di script. -->
      <div>
        <Label>Sumber Soal</Label>
        <div class="flex gap-2 mt-1.5">
          <button
            type="button"
            class="flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-lg border text-sm font-medium transition-colors"
            :class="mode === 'random' ? 'bg-indigo-50 border-indigo-300 text-indigo-700' : 'bg-white border-slate-200 text-slate-500 hover:bg-slate-50'"
            @click="mode = 'random'"
          ><Shuffle class="w-3.5 h-3.5" />Acak dari Bank Soal</button>
          <button
            type="button"
            class="flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-lg border text-sm font-medium transition-colors"
            :class="mode === 'set' ? 'bg-indigo-50 border-indigo-300 text-indigo-700' : 'bg-white border-slate-200 text-slate-500 hover:bg-slate-50'"
            @click="mode = 'set'"
          ><ListChecks class="w-3.5 h-3.5" />Set Soal Tetap</button>
        </div>
      </div>

      <div v-if="mode === 'random'" class="grid sm:grid-cols-2 gap-4">
        <div>
          <Label>Jumlah Soal</Label>
          <Input v-model.number="questionCount" type="number" min="1" class="mt-1.5" />
        </div>
        <div>
          <Label>Kesulitan</Label>
          <select v-model="difficultyFilter" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
            <option value="">Campuran</option>
            <option value="easy">Mudah</option>
            <option value="medium">Sedang</option>
            <option value="hard">Sulit</option>
          </select>
        </div>
        <div>
          <Label>Mata Uji (opsional)</Label>
          <select v-model="subjectFilter" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
            <option value="">Semua mata uji</option>
            <optgroup v-for="cat in categories" :key="cat.id" :label="cat.name">
              <option v-for="s in cat.subjects" :key="s.id" :value="s.name">{{ s.name }}</option>
            </optgroup>
          </select>
        </div>
        <div>
          <Label>Topik (opsional)</Label>
          <Input v-model="topicFilter" placeholder="Contoh: Aljabar" class="mt-1.5" />
        </div>
      </div>

      <div v-else class="space-y-2">
        <Label>Set Soal</Label>
        <select v-model="selectedSetId" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
          <option value="" disabled>Pilih Set Soal...</option>
          <option v-for="s in sets" :key="s.id" :value="s.id">{{ s.name }} ({{ s.item_count }} soal)</option>
        </select>
        <p v-if="selectedSet" class="text-xs text-muted-foreground">
          Set ini berisi <strong>{{ selectedSet.item_count }} soal</strong> — semua siswa yang mengambil sesi ini akan mendapat soal &amp; urutan yang persis sama (tidak diundi acak).
        </p>
      </div>

      <div class="flex items-center gap-3 p-4 bg-amber-50 rounded-lg border border-amber-200">
        <input id="isPremium" v-model="isPremium" type="checkbox" class="w-4 h-4" />
        <label for="isPremium" class="text-sm cursor-pointer">Sesi Premium (hanya untuk siswa dengan akses premium)</label>
      </div>

      <div class="flex items-center gap-3 p-4 bg-violet-50 rounded-lg border border-violet-200">
        <input id="isElective" v-model="isElective" type="checkbox" class="w-4 h-4" />
        <label for="isElective" class="text-sm cursor-pointer">
          Mapel Pilihan (bukan wajib) — siswa harus memilihnya dulu di "Pilih Mapel Pilihan" sebelum bisa mengerjakan, sesuai jumlah pilih paketnya
        </label>
      </div>

      <p v-if="mode === 'random'" class="text-xs text-muted-foreground">
        Sesi hanya bisa dimulai siswa jika ada soal berstatus "Disetujui" yang cocok dengan filter mata uji/kesulitan di atas.
      </p>

      <div class="flex items-center justify-between pt-4 border-t">
        <Button variant="outline" @click="open = false">Batal</Button>
        <Button variant="gradient" :disabled="saving" @click="save">
          <Sparkles class="w-4 h-4" />
          {{ saving ? 'Menyimpan...' : isEditing ? 'Update Sesi' : 'Simpan Sesi' }}
        </Button>
      </div>
    </div>
  </Dialog>
</template>
