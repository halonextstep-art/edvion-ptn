<script setup lang="ts">
import { Download, RefreshCw, CheckCircle2, AlertCircle, Copy, ShieldCheck, Send, Key, ChevronRight, ChevronLeft, Upload, FileSpreadsheet } from 'lucide-vue-next'
import type { School } from '~/types'
import { PACKAGE_QUOTA } from '~/types'
import { parseStudentImportWorkbook, downloadStudentImportTemplate, importedRowToCreatePayload, type ImportedStudentRow } from '~/utils/studentImportXlsx'

const open = defineModel<boolean>({ default: false })
const props = defineProps<{ school: School; mode: 'approve' | 'import' }>()
const emit = defineEmits<{ saved: [] }>()

const { userService, schoolService } = useApi()
const toast = useToast()

type Step = 1 | 2 | 3

const step = ref<Step>(props.mode === 'approve' ? 1 : 2)
const schoolEmail = ref(props.school.email)
const fileInput = ref<HTMLInputElement | null>(null)
const selectedFile = ref<File | null>(null)
const parsing = ref(false)
const parseError = ref('')
const rows = ref<ImportedStudentRow[]>([])
const creating = ref(false)
const done = ref(false)
const createdCount = ref(0)
const failedRows = ref<{ name: string; reason: string }[]>([])

watch(open, (isOpen) => {
  if (isOpen) {
    step.value = props.mode === 'approve' ? 1 : 2
    schoolEmail.value = props.school.email
    selectedFile.value = null
    parseError.value = ''
    rows.value = []
    done.value = false
    createdCount.value = 0
    failedRows.value = []
    if (fileInput.value) fileInput.value.value = ''
  }
})

const quota = computed(() => PACKAGE_QUOTA[props.school.package_type])
const readyRows = computed(() => rows.value.filter((r) => r.status === 'ready'))
const errorRows = computed(() => rows.value.filter((r) => r.status === 'error'))

function pickFile() {
  fileInput.value?.click()
}

async function onFileSelected(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  if (!/\.(xlsx|xls)$/i.test(file.name)) {
    toast.error('Format tidak didukung', 'Unggah file .xlsx atau .xls.')
    return
  }
  selectedFile.value = file
  rows.value = []
  parseError.value = ''
  parsing.value = true
  try {
    rows.value = await parseStudentImportWorkbook(file)
    if (rows.value.length === 0) parseError.value = 'Tidak ada baris data ditemukan di file ini.'
  } catch (err: any) {
    parseError.value = err?.message || 'Gagal membaca file.'
  } finally {
    parsing.value = false
  }
}

function downloadTemplate() {
  downloadStudentImportTemplate()
  toast.success('Template .xlsx diunduh')
}

async function copyCredentials() {
  const text = readyRows.value
    .map((r) => `${r.name} | Email: ${r.email} | Password: ${r.password}`)
    .join('\n')
  try {
    await navigator.clipboard?.writeText(text)
    toast.success('Daftar kredensial disalin ke clipboard')
  } catch {
    // clipboard permission denied — the table on screen still has everything
  }
}

async function handleActivate() {
  creating.value = true
  createdCount.value = 0
  failedRows.value = []
  for (const r of readyRows.value) {
    try {
      await userService.create(importedRowToCreatePayload(r, props.school.id))
      createdCount.value += 1
    } catch (e: any) {
      failedRows.value.push({ name: r.name, reason: e?.message || 'gagal dibuat' })
    }
  }

  if (props.mode === 'approve') {
    try {
      await schoolService.setStatus(props.school.id, 'active')
    } catch (e: any) {
      toast.error('Siswa berhasil diimport, tapi gagal mengaktifkan status mitra', e?.message)
    }
  }

  creating.value = false
  done.value = true
  emit('saved')
}
</script>

<template>
  <Dialog v-model="open" title="Onboarding Mitra Sekolah" max-width="max-w-2xl">
    <div v-if="done" class="text-center py-6">
      <div class="w-20 h-20 rounded-full bg-green-100 flex items-center justify-center mx-auto mb-5">
        <ShieldCheck class="w-10 h-10 text-green-600" />
      </div>
      <h3 class="text-2xl font-black text-slate-900 mb-2">Onboarding Selesai!</h3>
      <p class="text-sm text-muted-foreground mb-2">
        <span class="font-bold text-slate-900">{{ school.name }}</span> {{ mode === 'approve' ? 'kini aktif sebagai mitra.' : 'mendapat siswa baru.' }}
      </p>
      <p class="text-sm text-muted-foreground mb-6">
        <span class="font-bold text-green-600">{{ createdCount }} akun siswa</span> berhasil dibuat dan siap digunakan.
        <span v-if="failedRows.length" class="block text-red-500 mt-1">{{ failedRows.length }} baris gagal ({{ failedRows.map(f => f.name).join(', ') }}).</span>
      </p>
      <div class="p-4 bg-blue-50 border border-blue-200 rounded-xl text-left mb-6">
        <p class="text-xs font-bold text-blue-700 mb-1.5 flex items-center gap-1"><Send class="w-3.5 h-3.5" />Langkah Selanjutnya</p>
        <ul class="text-xs text-blue-800 space-y-1.5">
          <li>&bull; Kirim daftar kredensial ke PIC sekolah: <span class="font-bold">{{ schoolEmail }}</span></li>
          <li>&bull; Siswa login dengan <strong>email</strong> dan password dari file import</li>
          <li>&bull; Sarankan siswa mengganti password setelah login pertama</li>
        </ul>
      </div>
      <div class="flex gap-3">
        <Button variant="outline" class="flex-1 gap-2" @click="copyCredentials"><Copy class="w-4 h-4" />Salin Kredensial</Button>
        <Button class="flex-1 bg-green-600 hover:bg-green-700" @click="open = false">Selesai</Button>
      </div>
    </div>

    <div v-else class="space-y-5">
      <div class="flex items-center gap-0">
        <div v-for="(label, i) in ['Konfirmasi Mitra', 'Import Siswa', 'Aktivasi Akun']" :key="label" class="flex items-center flex-1">
          <div class="flex flex-col items-center flex-1">
            <div
              class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-black transition-colors"
              :class="step > (i + 1) ? 'bg-green-500 text-white' : step === (i + 1) ? 'bg-indigo-600 text-white' : 'bg-slate-100 text-slate-400'"
            >
              <CheckCircle2 v-if="step > (i + 1)" class="w-4 h-4" />
              <template v-else>{{ i + 1 }}</template>
            </div>
            <p class="text-[10px] font-semibold mt-1 text-center leading-tight" :class="step === (i + 1) ? 'text-indigo-600' : step > (i + 1) ? 'text-green-600' : 'text-slate-400'">{{ label }}</p>
          </div>
          <div v-if="i < 2" class="h-0.5 w-full mx-1 rounded-full mb-4" :class="step > (i + 1) ? 'bg-green-400' : 'bg-slate-200'" />
        </div>
      </div>

      <!-- Step 1: confirm -->
      <div v-if="step === 1" class="space-y-5">
        <div class="flex items-start gap-4 p-4 bg-blue-50 border border-blue-200 rounded-xl">
          <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-blue-500 to-cyan-500 flex items-center justify-center text-white shrink-0">
            <ShieldCheck class="w-6 h-6" />
          </div>
          <div>
            <h4 class="font-black text-slate-900">{{ school.name }}</h4>
            <p class="text-sm text-muted-foreground">{{ school.city }}, {{ school.province }}</p>
            <p class="text-xs text-muted-foreground mt-1">Maks. {{ quota === 9999 ? 'Unlimited' : quota }} siswa &middot; Revenue share {{ school.revenue_share }}%</p>
          </div>
        </div>
        <div>
          <Label>Email untuk terima kredensial siswa *</Label>
          <Input v-model="schoolEmail" class="mt-1.5" placeholder="email PIC sekolah" />
        </div>
        <div class="p-4 bg-amber-50 border border-amber-200 rounded-xl flex items-start gap-2">
          <AlertCircle class="w-4 h-4 text-amber-600 shrink-0 mt-0.5" />
          <p class="text-xs text-amber-800">Setelah dikonfirmasi, status mitra akan berubah dari <strong>Pending</strong> ke <strong>Aktif</strong> setelah import siswa selesai.</p>
        </div>
      </div>

      <!-- Step 2: import xlsx -->
      <div v-if="step === 2" class="space-y-5">
        <div class="flex items-center justify-between flex-wrap gap-2">
          <div>
            <h4 class="font-bold text-slate-900">Import Data Siswa</h4>
            <p class="text-sm text-muted-foreground">Kuota: <span class="font-bold text-indigo-600">{{ quota === 9999 ? 'Unlimited' : quota }} siswa</span></p>
          </div>
          <Button variant="outline" size="sm" class="gap-2" @click="downloadTemplate"><Download class="w-4 h-4" />Template .xlsx</Button>
        </div>
        <div class="p-3 bg-slate-50 border rounded-xl">
          <p class="text-xs font-mono text-slate-500">Kolom: nis, nisn, nama, jenis kelamin, tanggal lahir, tanggal masuk, kode rombel, nama rombel, username, email, telepon, password</p>
          <p class="text-[11px] text-muted-foreground mt-1">Wajib diisi: nisn, nama, email, password. Kolom lain boleh kosong. Password diambil langsung dari file (bukan dibuat otomatis).</p>
        </div>

        <div
          class="border-2 border-dashed rounded-xl p-6 text-center cursor-pointer transition-colors"
          :class="selectedFile ? 'border-emerald-300 bg-emerald-50' : 'border-slate-200 hover:border-indigo-300 hover:bg-slate-50'"
          @click="pickFile"
        >
          <input ref="fileInput" type="file" accept=".xlsx,.xls" class="hidden" @change="onFileSelected" />
          <FileSpreadsheet v-if="selectedFile" class="w-8 h-8 mx-auto mb-2 text-emerald-500" />
          <Upload v-else class="w-8 h-8 mx-auto mb-2 text-slate-400" />
          <p class="text-sm font-semibold text-slate-700">{{ selectedFile ? selectedFile.name : 'Klik untuk pilih file .xlsx' }}</p>
          <p v-if="parsing" class="text-xs text-indigo-600 mt-1 flex items-center justify-center gap-1"><RefreshCw class="w-3 h-3 animate-spin" />Membaca file...</p>
        </div>

        <div v-if="parseError" class="p-3 bg-red-50 border border-red-200 rounded-xl text-xs text-red-600 flex items-start gap-2">
          <AlertCircle class="w-3.5 h-3.5 shrink-0 mt-0.5" />{{ parseError }}
        </div>

        <div v-if="rows.length > 0">
          <p class="text-sm font-semibold text-slate-700 mb-2">
            Hasil validasi: <span class="text-green-600">{{ readyRows.length }} siap</span>
            <span v-if="errorRows.length">, <span class="text-red-500">{{ errorRows.length }} error</span></span>
          </p>
          <div class="border rounded-xl overflow-hidden max-h-52 overflow-y-auto">
            <table class="w-full text-xs">
              <thead class="sticky top-0 bg-slate-50">
                <tr class="border-b">
                  <th class="text-left px-3 py-2 font-semibold text-slate-600">NISN</th>
                  <th class="text-left px-3 py-2 font-semibold text-slate-600">Nama</th>
                  <th class="text-left px-3 py-2 font-semibold text-slate-600">Rombel</th>
                  <th class="text-left px-3 py-2 font-semibold text-slate-600">Status</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(r, i) in rows" :key="i" class="border-b" :class="r.status === 'error' ? 'bg-red-50' : ''">
                  <td class="px-3 py-2 font-mono">{{ r.nisn || '—' }}</td>
                  <td class="px-3 py-2 font-semibold">{{ r.name || '—' }}</td>
                  <td class="px-3 py-2 text-slate-600">{{ r.grade || '—' }}</td>
                  <td class="px-3 py-2">
                    <span v-if="r.status === 'ready'" class="text-green-600 font-semibold flex items-center gap-1"><CheckCircle2 class="w-3 h-3" />Siap</span>
                    <span v-else class="text-red-500 font-semibold flex items-center gap-1"><AlertCircle class="w-3 h-3" />{{ r.error }}</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <p v-if="readyRows.length > quota" class="text-xs text-red-600 mt-2 flex items-center gap-1"><AlertCircle class="w-3 h-3" />Melebihi kuota paket ({{ quota }} siswa).</p>
        </div>
      </div>

      <!-- Step 3: activate -->
      <div v-if="step === 3" class="space-y-5">
        <div>
          <h4 class="font-bold text-slate-900 mb-1">Preview Akun yang Akan Dibuat</h4>
          <p class="text-sm text-muted-foreground"><span class="font-bold text-green-600">{{ readyRows.length }} akun siswa</span> akan dibuat lewat backend — bukan simulasi.</p>
        </div>
        <div class="p-4 bg-indigo-50 border border-indigo-200 rounded-xl">
          <p class="text-xs font-bold text-indigo-700 mb-2 flex items-center gap-1.5"><Key class="w-3.5 h-3.5" />Aturan Akun Siswa</p>
          <ul class="text-xs text-indigo-800 space-y-1">
            <li>&bull; <strong>Username login:</strong> email siswa</li>
            <li>&bull; <strong>Password:</strong> persis seperti di kolom "password" pada file yang diunggah</li>
            <li>&bull; Sarankan siswa mengganti password setelah login pertama</li>
          </ul>
        </div>
        <div class="border rounded-xl overflow-hidden max-h-56 overflow-y-auto">
          <table class="w-full text-xs">
            <thead class="sticky top-0 bg-slate-50">
              <tr class="border-b">
                <th class="text-left px-3 py-2 font-semibold text-slate-600">Nama</th>
                <th class="text-left px-3 py-2 font-semibold text-slate-600">Email</th>
                <th class="text-left px-3 py-2 font-semibold text-slate-600">Password</th>
                <th class="text-left px-3 py-2 font-semibold text-slate-600">Rombel</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(r, i) in readyRows" :key="i" class="border-b hover:bg-slate-50">
                <td class="px-3 py-2 font-semibold">{{ r.name }}</td>
                <td class="px-3 py-2 font-mono text-indigo-600">{{ r.email }}</td>
                <td class="px-3 py-2 font-mono text-purple-600">{{ r.password }}</td>
                <td class="px-3 py-2 text-slate-600">{{ r.grade || '—' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div class="p-3 bg-emerald-50 border border-emerald-200 rounded-xl text-center">
            <p class="text-2xl font-black text-emerald-600">{{ readyRows.length }}</p>
            <p class="text-xs text-emerald-700 font-semibold">Akun siap dibuat</p>
          </div>
          <div class="p-3 bg-slate-50 border rounded-xl text-center">
            <p class="text-2xl font-black text-slate-700">{{ quota === 9999 ? '∞' : Math.max(quota - readyRows.length, 0) }}</p>
            <p class="text-xs text-slate-500 font-semibold">Sisa kuota</p>
          </div>
        </div>
      </div>

      <div class="flex items-center justify-between pt-4 border-t">
        <Button variant="outline" class="gap-1" @click="step === 1 || (step === 2 && mode === 'import') ? (open = false) : (step = (step - 1) as Step)">
          <ChevronLeft v-if="!(step === 1 || (step === 2 && mode === 'import'))" class="w-4 h-4" />{{ step === 1 || (step === 2 && mode === 'import') ? 'Batal' : 'Kembali' }}
        </Button>
        <Button v-if="step < 3" variant="gradient" :disabled="step === 2 && (rows.length === 0 || readyRows.length === 0 || readyRows.length > quota)" @click="step = (step + 1) as Step">
          Lanjut<ChevronRight class="w-4 h-4" />
        </Button>
        <Button v-else class="bg-green-600 hover:bg-green-700 gap-2" :disabled="creating" @click="handleActivate">
          <ShieldCheck class="w-4 h-4" />{{ creating ? 'Memproses...' : 'Konfirmasi & Aktifkan' }}
        </Button>
      </div>
    </div>
  </Dialog>
</template>
