<script setup lang="ts">
// Profil Saya — dipakai di ke-4 role (Admin, Sekolah, Siswa, Konten). Satu komponen
// bersama, bukan diduplikasi per role, karena field & alurnya sama persis (edit nama/
// telepon sendiri, ganti password sendiri) — hanya potongan info read-only (sekolah,
// NISN/kelas) yang tampil kondisional sesuai role. Backend: PUT /api/auth/me untuk
// profil, PUT /api/auth/me/password untuk password (lihat AuthService::update_profile /
// change_password) — sengaja tidak lewat UserService admin, supaya user biasa tidak
// bisa mengubah role/status/sekolah/email milik sendiri.
import { User as UserIcon, Phone, Mail, Building2, GraduationCap, ShieldCheck, KeyRound, Eye, EyeOff, Loader2, Camera } from 'lucide-vue-next'
import type { Role } from '~/types'

const open = defineModel<boolean>({ default: false })
const { user, updateProfile, changePassword, uploadAvatar } = useAuth()
const toast = useToast()
const { resolve: resolveUploadUrl } = useUploadUrl()

const ROLE_LABEL: Record<Role, string> = {
  admin: 'Admin Pusat', school: 'PIC Sekolah', student: 'Siswa', content: 'Tim Konten',
}

const initials = computed(() => (user.value?.name || '?').trim().charAt(0).toUpperCase())
const avatarUrl = computed(() => resolveUploadUrl(user.value?.avatar_url))

// ─── Upload foto profil ─────────────────────────────────────────────────────────────
const avatarInput = ref<HTMLInputElement | null>(null)
const uploadingAvatar = ref(false)
const AVATAR_MAX_BYTES = 2 * 1024 * 1024
const AVATAR_ALLOWED = ['image/png', 'image/jpeg', 'image/webp']

function pickAvatar() {
  avatarInput.value?.click()
}
async function onAvatarSelected(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  if (!AVATAR_ALLOWED.includes(file.type)) {
    toast.error('Format tidak didukung', 'Gunakan PNG, JPEG, atau WEBP.')
    return
  }
  if (file.size > AVATAR_MAX_BYTES) {
    toast.error('File terlalu besar', 'Maksimum 2 MB.')
    return
  }
  uploadingAvatar.value = true
  try {
    await uploadAvatar(file)
    toast.success('Foto profil diperbarui')
  } catch (err: any) {
    toast.error('Gagal mengunggah foto', err?.message)
  } finally {
    uploadingAvatar.value = false
    if (avatarInput.value) avatarInput.value.value = ''
  }
}

// ─── Edit profil (nama & telepon) ──────────────────────────────────────────────────
const name = ref('')
const phone = ref('')
const savingProfile = ref(false)

function resetProfileForm() {
  name.value = user.value?.name || ''
  phone.value = user.value?.phone || ''
}
watch(open, (isOpen) => { if (isOpen) resetProfileForm() })

async function saveProfile() {
  if (!name.value.trim()) return toast.error('Nama tidak boleh kosong!')
  savingProfile.value = true
  try {
    // minat_jurusan tidak diedit di dialog ini (diedit di step "Data Siswa" Rasionalisasi
    // SNBP) — tapi backend menimpa kolom ini tanpa syarat setiap kali disimpan, jadi nilai
    // saat ini harus selalu dikirim ulang di sini supaya tidak ke-NULL-kan tanpa sengaja.
    await updateProfile({ name: name.value.trim(), phone: phone.value.trim() || null, minat_jurusan: user.value?.minat_jurusan ?? null })
    toast.success('Profil berhasil diperbarui')
  } catch (e: any) {
    toast.error('Gagal memperbarui profil', e?.message)
  } finally {
    savingProfile.value = false
  }
}

// ─── Ganti password ─────────────────────────────────────────────────────────────────
const showPasswordForm = ref(false)
const currentPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const showCurrent = ref(false)
const showNew = ref(false)
const changingPassword = ref(false)

function resetPasswordForm() {
  currentPassword.value = ''
  newPassword.value = ''
  confirmPassword.value = ''
  showPasswordForm.value = false
}
watch(open, (isOpen) => { if (isOpen) resetPasswordForm() })

async function savePassword() {
  if (!currentPassword.value) return toast.error('Masukkan password saat ini!')
  if (newPassword.value.length < 6) return toast.error('Password baru minimal 6 karakter!')
  if (newPassword.value !== confirmPassword.value) return toast.error('Konfirmasi password baru tidak cocok!')
  changingPassword.value = true
  try {
    await changePassword({ current_password: currentPassword.value, new_password: newPassword.value })
    toast.success('Password berhasil diubah')
    resetPasswordForm()
  } catch (e: any) {
    toast.error('Gagal mengubah password', e?.message ?? 'Password saat ini mungkin salah')
  } finally {
    changingPassword.value = false
  }
}
</script>

<template>
  <Dialog v-model="open" title="Profil Saya" max-width="max-w-md">
    <div class="space-y-6">
      <!-- Header identitas -->
      <div class="flex items-center gap-3">
        <button
          type="button" class="relative w-14 h-14 rounded-2xl shrink-0 group"
          title="Ganti foto profil" @click="pickAvatar"
        >
          <img v-if="avatarUrl" :src="avatarUrl" alt="" class="w-14 h-14 rounded-2xl object-cover" />
          <div v-else class="w-14 h-14 rounded-2xl bg-gradient-to-br from-violet-500 to-purple-600 flex items-center justify-center text-white text-xl font-black">
            {{ initials }}
          </div>
          <div class="absolute inset-0 rounded-2xl bg-black/0 group-hover:bg-black/40 transition-colors flex items-center justify-center">
            <Loader2 v-if="uploadingAvatar" class="w-4 h-4 text-white animate-spin" />
            <Camera v-else class="w-4 h-4 text-white opacity-0 group-hover:opacity-100 transition-opacity" />
          </div>
        </button>
        <input ref="avatarInput" type="file" accept="image/png,image/jpeg,image/webp" class="hidden" @change="onAvatarSelected" />
        <div class="min-w-0">
          <p class="font-bold text-slate-900 truncate">{{ user?.name }}</p>
          <p class="text-xs text-muted-foreground truncate">{{ user ? ROLE_LABEL[user.role] : '' }}</p>
        </div>
      </div>

      <!-- Info read-only -->
      <div class="space-y-2 p-3 rounded-xl bg-slate-50 border text-sm">
        <div class="flex items-center gap-2 text-slate-600"><Mail class="w-3.5 h-3.5 shrink-0" /> {{ user?.email }}</div>
        <div v-if="user?.school_name" class="flex items-center gap-2 text-slate-600"><Building2 class="w-3.5 h-3.5 shrink-0" /> {{ user.school_name }}</div>
        <div v-if="user?.role === 'student' && user?.grade" class="flex items-center gap-2 text-slate-600"><GraduationCap class="w-3.5 h-3.5 shrink-0" /> {{ user.grade }}<span v-if="user?.nisn"> · NISN {{ user.nisn }}</span></div>
        <div v-if="user?.role === 'admin'" class="flex items-center gap-2 text-slate-600"><ShieldCheck class="w-3.5 h-3.5 shrink-0" /> Akses penuh sistem</div>
        <p class="text-[10px] text-muted-foreground pt-1">Email, role, sekolah, NISN, dan kelas dikelola oleh admin/sekolah — hubungi mereka untuk mengubahnya.</p>
      </div>

      <!-- Edit nama & telepon -->
      <div class="space-y-3">
        <div>
          <Label class="flex items-center gap-1.5"><UserIcon class="w-3.5 h-3.5" /> Nama Lengkap</Label>
          <Input v-model="name" placeholder="Nama lengkap" class="mt-1.5" />
        </div>
        <div>
          <Label class="flex items-center gap-1.5"><Phone class="w-3.5 h-3.5" /> No. Telepon</Label>
          <Input v-model="phone" placeholder="08xx-xxxx-xxxx" class="mt-1.5" />
        </div>
        <Button variant="gradient" class="w-full" :disabled="savingProfile" @click="saveProfile">
          <Loader2 v-if="savingProfile" class="w-4 h-4 animate-spin" /> {{ savingProfile ? 'Menyimpan...' : 'Simpan Perubahan' }}
        </Button>
      </div>

      <!-- Ganti password -->
      <div class="pt-4 border-t">
        <button
          class="w-full flex items-center justify-between text-sm font-semibold text-slate-700 hover:text-slate-900"
          @click="showPasswordForm = !showPasswordForm"
        >
          <span class="flex items-center gap-1.5"><KeyRound class="w-3.5 h-3.5" /> Ubah Password</span>
          <span class="text-xs text-muted-foreground">{{ showPasswordForm ? 'Tutup' : 'Buka' }}</span>
        </button>

        <div v-if="showPasswordForm" class="space-y-3 mt-3">
          <div>
            <Label>Password Saat Ini</Label>
            <div class="relative mt-1.5">
              <Input v-model="currentPassword" :type="showCurrent ? 'text' : 'password'" placeholder="Password saat ini" />
              <button type="button" class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground" @click="showCurrent = !showCurrent">
                <EyeOff v-if="showCurrent" class="w-4 h-4" /><Eye v-else class="w-4 h-4" />
              </button>
            </div>
          </div>
          <div>
            <Label>Password Baru</Label>
            <div class="relative mt-1.5">
              <Input v-model="newPassword" :type="showNew ? 'text' : 'password'" placeholder="Minimal 6 karakter" />
              <button type="button" class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground" @click="showNew = !showNew">
                <EyeOff v-if="showNew" class="w-4 h-4" /><Eye v-else class="w-4 h-4" />
              </button>
            </div>
          </div>
          <div>
            <Label>Konfirmasi Password Baru</Label>
            <Input v-model="confirmPassword" :type="showNew ? 'text' : 'password'" placeholder="Ulangi password baru" class="mt-1.5" />
          </div>
          <Button variant="outline" class="w-full" :disabled="changingPassword" @click="savePassword">
            <Loader2 v-if="changingPassword" class="w-4 h-4 animate-spin" /> {{ changingPassword ? 'Mengubah...' : 'Ubah Password' }}
          </Button>
        </div>
      </div>
    </div>
  </Dialog>
</template>
