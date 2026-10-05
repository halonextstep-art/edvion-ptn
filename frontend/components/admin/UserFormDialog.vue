<script setup lang="ts">
import { Eye, EyeOff, Copy, RefreshCw, Sparkles, GraduationCap, School as SchoolIcon, Layers, ShieldCheck } from 'lucide-vue-next'
import type { Role, School, User, UserStatus } from '~/types'

const open = defineModel<boolean>({ default: false })
const props = defineProps<{ editUser?: User | null }>()
const emit = defineEmits<{ saved: [] }>()

const { userService, schoolService } = useApi()
const toast = useToast()

const ROLE_CONFIG: Record<Role, { label: string; icon: any; color: string }> = {
  student: { label: 'Siswa', icon: GraduationCap, color: 'text-blue-600' },
  school: { label: 'Sekolah', icon: SchoolIcon, color: 'text-emerald-600' },
  content: { label: 'Tim Konten', icon: Layers, color: 'text-violet-600' },
  admin: { label: 'Admin', icon: ShieldCheck, color: 'text-rose-600' },
}
const GRADES = ['Kelas 10', 'Kelas 11', 'Kelas 12']

const name = ref('')
const email = ref('')
const password = ref('')
const showPassword = ref(false)
const role = ref<Role>('student')
const schoolId = ref<string>('')
const status = ref<UserStatus>('active')
const phone = ref('')
const nisn = ref('')
const grade = ref('Kelas 12')
const saving = ref(false)
const schools = ref<School[]>([])

const isEditing = computed(() => !!props.editUser)
// A School-PIC account must always belong to one real school. A Student may optionally have
// one (B2B, enrolled via a partner school) or none (B2C/mandiri) — so the school picker shows
// for both roles, but is only *required* for `school`.
const showSchoolPicker = computed(() => role.value === 'school' || role.value === 'student')
const schoolRequired = computed(() => role.value === 'school')

function genPassword(seed: string) {
  const part = (seed || 'user').replace(/\s+/g, '').slice(0, 4).toLowerCase() || 'user'
  return `Gspl${part}${Math.floor(1000 + Math.random() * 9000)}!`
}

function resetForm(u?: User | null) {
  name.value = u?.name || ''
  email.value = u?.email || ''
  password.value = ''
  role.value = u?.role || 'student'
  schoolId.value = u?.school_id || ''
  status.value = u?.status || 'active'
  phone.value = u?.phone || ''
  nisn.value = u?.nisn || ''
  grade.value = u?.grade || 'Kelas 12'
  if (!u) password.value = genPassword('user')
}

watch(open, async (isOpen) => {
  if (isOpen) {
    resetForm(props.editUser)
    try {
      schools.value = await schoolService.list()
    } catch {
      // school dropdown just stays empty; the rest of the form still works
    }
  }
})

function regeneratePassword() {
  password.value = genPassword(name.value)
}

async function copyPassword() {
  try {
    await navigator.clipboard?.writeText(password.value)
    toast.success('Password disalin ke clipboard')
  } catch {
    // clipboard permission denied — silently ignore, the value is still visible on screen
  }
}

async function save() {
  if (!name.value.trim()) return toast.error('Nama tidak boleh kosong!')
  if (!email.value.trim()) return toast.error('Email tidak boleh kosong!')
  if (!isEditing.value && password.value.length < 6) return toast.error('Password minimal 6 karakter!')
  if (schoolRequired.value && !schoolId.value) return toast.error('Pilih sekolah untuk role ini!')

  saving.value = true
  try {
    if (isEditing.value && props.editUser) {
      await userService.update(props.editUser.id, {
        name: name.value,
        email: email.value,
        role: role.value,
        school_id: showSchoolPicker.value && schoolId.value ? schoolId.value : null,
        status: status.value,
        phone: role.value === 'school' ? phone.value || null : null,
        nisn: role.value === 'student' ? nisn.value || null : null,
        grade: role.value === 'student' ? grade.value : null,
        password: password.value || undefined,
      })
      toast.success('Akun berhasil diperbarui!')
    } else {
      await userService.create({
        name: name.value,
        email: email.value,
        password: password.value,
        role: role.value,
        school_id: showSchoolPicker.value && schoolId.value ? schoolId.value : null,
        status: status.value,
        phone: role.value === 'school' ? phone.value || null : null,
        nisn: role.value === 'student' ? nisn.value || null : null,
        grade: role.value === 'student' ? grade.value : null,
      })
      toast.success(`Akun baru berhasil dibuat — password sementara: ${password.value}`)
    }
    open.value = false
    emit('saved')
  } catch (e: any) {
    toast.error('Gagal menyimpan akun', e?.message)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <Dialog v-model="open" :title="isEditing ? 'Edit Akun' : 'Tambah Akun Baru'" max-width="max-w-lg">
    <div class="space-y-4">
      <div>
        <Label class="mb-2 block">Role</Label>
        <div class="grid grid-cols-2 gap-2">
          <button
            v-for="(cfg, r) in ROLE_CONFIG"
            :key="r"
            type="button"
            class="flex items-center gap-2.5 p-3 rounded-xl border-2 text-left transition-all"
            :class="role === r ? 'border-indigo-500 bg-indigo-50' : 'border-slate-200 hover:border-indigo-200'"
            @click="role = r as Role"
          >
            <component :is="cfg.icon" class="w-4 h-4" :class="role === r ? 'text-indigo-600' : cfg.color" />
            <span class="text-sm font-semibold" :class="role === r ? 'text-indigo-700' : 'text-slate-600'">{{ cfg.label }}</span>
          </button>
        </div>
      </div>

      <div>
        <Label>Nama</Label>
        <Input v-model="name" placeholder="Nama lengkap" class="mt-1.5" />
      </div>
      <div>
        <Label>Email</Label>
        <Input v-model="email" type="email" placeholder="nama@email.com" class="mt-1.5" />
      </div>

      <div v-if="showSchoolPicker">
        <Label>Sekolah{{ schoolRequired ? '' : ' (opsional)' }}</Label>
        <select v-model="schoolId" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
          <option value="" :disabled="schoolRequired">
            {{ schoolRequired ? 'Pilih sekolah...' : 'Tanpa sekolah (Mandiri / B2C)' }}
          </option>
          <option v-for="s in schools" :key="s.id" :value="s.id">{{ s.name }}</option>
        </select>
        <p v-if="schools.length === 0 && schoolRequired" class="text-xs text-amber-600 mt-1.5">
          Belum ada sekolah terdaftar — tambahkan dulu lewat tab "Mitra".
        </p>
        <p v-else-if="role === 'student'" class="text-xs text-slate-400 mt-1.5">
          Kosongkan jika siswa mendaftar mandiri (B2C), tanpa terdaftar lewat sekolah mitra.
        </p>
      </div>

      <div v-if="role === 'student'" class="grid grid-cols-2 gap-3">
        <div>
          <Label>NISN</Label>
          <Input v-model="nisn" placeholder="0012345678" class="mt-1.5 font-mono" />
        </div>
        <div>
          <Label>Kelas</Label>
          <select v-model="grade" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
            <option v-for="g in GRADES" :key="g" :value="g">{{ g }}</option>
          </select>
        </div>
      </div>

      <div v-if="role === 'school'">
        <Label>No. Telepon</Label>
        <Input v-model="phone" placeholder="021-..." class="mt-1.5" />
      </div>

      <div>
        <Label>Status</Label>
        <select v-model="status" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
          <option value="active">Aktif</option>
          <option value="inactive">Nonaktif</option>
          <option value="pending">Pending (menunggu persetujuan)</option>
        </select>
      </div>

      <div v-if="!isEditing" class="p-4 bg-amber-50 border border-amber-200 rounded-xl">
        <p class="text-xs font-bold text-amber-700 mb-2">Password Sementara</p>
        <div class="flex items-center gap-2">
          <code class="flex-1 text-sm font-mono bg-white px-3 py-2 rounded-lg border border-amber-200 text-slate-800 truncate">
            {{ showPassword ? password : '••••••••••' }}
          </code>
          <button type="button" class="p-2 rounded-lg hover:bg-amber-100 text-amber-600 transition-colors" @click="showPassword = !showPassword">
            <EyeOff v-if="showPassword" class="w-4 h-4" />
            <Eye v-else class="w-4 h-4" />
          </button>
          <button type="button" class="p-2 rounded-lg hover:bg-amber-100 text-amber-600 transition-colors" title="Buat ulang" @click="regeneratePassword">
            <RefreshCw class="w-4 h-4" />
          </button>
          <button type="button" class="p-2 rounded-lg hover:bg-amber-100 text-amber-600 transition-colors" title="Salin" @click="copyPassword">
            <Copy class="w-4 h-4" />
          </button>
        </div>
        <p class="text-[10px] text-amber-600 mt-1.5">Password ini akan langsung aktif untuk akun baru — salin dan bagikan ke pemilik akun.</p>
      </div>

      <div v-else>
        <Label>Password Baru (opsional)</Label>
        <div class="relative mt-1.5">
          <Input v-model="password" :type="showPassword ? 'text' : 'password'" placeholder="Kosongkan jika tidak diubah" />
          <button
            type="button"
            class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
            @click="showPassword = !showPassword"
          >
            <EyeOff v-if="showPassword" class="w-4 h-4" />
            <Eye v-else class="w-4 h-4" />
          </button>
        </div>
      </div>

      <div class="flex items-center justify-between pt-4 border-t">
        <Button variant="outline" @click="open = false">Batal</Button>
        <Button variant="gradient" :disabled="saving" @click="save">
          <Sparkles class="w-4 h-4" />
          {{ saving ? 'Menyimpan...' : isEditing ? 'Update Akun' : 'Simpan Akun' }}
        </Button>
      </div>
    </div>
  </Dialog>
</template>
