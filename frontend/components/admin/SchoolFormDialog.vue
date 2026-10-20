<script setup lang="ts">
import { Sparkles } from 'lucide-vue-next'
import type { LinkableTenant, PackageType, School, SchoolStatus, SchoolType } from '~/types'

const open = defineModel<boolean>({ default: false })
const props = defineProps<{ editSchool?: School | null }>()
const emit = defineEmits<{ saved: [] }>()

const { schoolService } = useApi()
const toast = useToast()

const name = ref('')
const schoolType = ref<SchoolType>('sma')
const city = ref('')
const province = ref('')
const email = ref('')
const phone = ref('')
const packageType = ref<PackageType>('basic')
const contactPerson = ref('')
const status = ref<SchoolStatus>('active')
const revenueShare = ref(10)
const monthlyRevenue = ref(0)
const saving = ref(false)

// Create-only — adopt an existing `core.tenant` (already onboarded by another module,
// with its own real students) instead of creating a brand-new one. Can't be changed once
// a school is created, so this is hidden entirely while editing.
const linkMode = ref<'new' | 'existing'>('new')
const linkedTenantId = ref('')
const unlinkedTenants = ref<LinkableTenant[]>([])
const loadingTenants = ref(false)

const isEditing = computed(() => !!props.editSchool)

async function loadUnlinkedTenants() {
  loadingTenants.value = true
  try {
    unlinkedTenants.value = await schoolService.unlinkedTenants()
  } catch (e: any) {
    toast.error('Gagal memuat daftar tenant', e?.message)
  } finally {
    loadingTenants.value = false
  }
}

function resetForm(s?: School | null) {
  name.value = s?.name || ''
  schoolType.value = s?.school_type || 'sma'
  city.value = s?.city || ''
  province.value = s?.province || ''
  email.value = s?.email || ''
  phone.value = s?.phone || ''
  packageType.value = s?.package_type || 'basic'
  contactPerson.value = s?.contact_person || ''
  status.value = s?.status || (s ? 'active' : 'pending')
  revenueShare.value = s?.revenue_share ?? 10
  monthlyRevenue.value = s?.monthly_revenue ?? 0
  linkMode.value = 'new'
  linkedTenantId.value = ''
}

watch(open, (isOpen) => {
  if (isOpen) {
    resetForm(props.editSchool)
    if (!isEditing.value) loadUnlinkedTenants()
  }
})

// Prefill nama/slug when a tenant is picked — admin can still edit the rest, but the
// school name defaulting to the tenant's own name avoids an easy-to-miss mismatch.
watch(linkedTenantId, (id) => {
  const t = unlinkedTenants.value.find((t) => t.id === id)
  if (t && !name.value.trim()) name.value = t.name
})

async function save() {
  if (!name.value.trim()) return toast.error('Nama sekolah tidak boleh kosong!')
  if (!email.value.trim()) return toast.error('Email tidak boleh kosong!')
  if (!contactPerson.value.trim()) return toast.error('Nama kontak person harus diisi!')
  if (!isEditing.value && linkMode.value === 'existing' && !linkedTenantId.value) {
    return toast.error('Pilih tenant yang akan di-link!')
  }

  saving.value = true
  try {
    const payload = {
      name: name.value,
      school_type: schoolType.value,
      city: city.value,
      province: province.value,
      email: email.value,
      phone: phone.value,
      package_type: packageType.value,
      contact_person: contactPerson.value,
      status: status.value,
      revenue_share: revenueShare.value,
      monthly_revenue: monthlyRevenue.value,
      ...(!isEditing.value && linkMode.value === 'existing' ? { linked_tenant_id: linkedTenantId.value } : {}),
    }
    if (isEditing.value && props.editSchool) {
      await schoolService.update(props.editSchool.id, payload)
      toast.success('Data sekolah berhasil diperbarui!')
    } else {
      await schoolService.create(payload)
      toast.success('Sekolah mitra baru berhasil ditambahkan!')
    }
    open.value = false
    emit('saved')
  } catch (e: any) {
    toast.error('Gagal menyimpan data sekolah', e?.message)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <Dialog v-model="open" :title="isEditing ? 'Edit Sekolah Mitra' : 'Tambah Sekolah Mitra'" max-width="max-w-lg">
    <div class="space-y-4">
      <div v-if="!isEditing" class="p-3 border rounded-lg bg-slate-50 space-y-2">
        <Label>Sumber Data Sekolah</Label>
        <div class="flex rounded-lg border bg-white p-0.5 w-fit">
          <button
            type="button" class="px-3 py-1 rounded-md text-xs font-semibold transition-colors"
            :class="linkMode === 'new' ? 'bg-indigo-600 text-white' : 'text-slate-500 hover:bg-slate-50'"
            @click="linkMode = 'new'"
          >Tenant Baru</button>
          <button
            type="button" class="px-3 py-1 rounded-md text-xs font-semibold transition-colors"
            :class="linkMode === 'existing' ? 'bg-indigo-600 text-white' : 'text-slate-500 hover:bg-slate-50'"
            @click="linkMode = 'existing'"
          >Link ke Tenant Existing</button>
        </div>
        <div v-if="linkMode === 'existing'">
          <select v-model="linkedTenantId" class="w-full mt-1 px-3 py-2 border rounded-md text-sm bg-white">
            <option value="" disabled>{{ loadingTenants ? 'Memuat...' : 'Pilih tenant...' }}</option>
            <option v-for="t in unlinkedTenants" :key="t.id" :value="t.id">{{ t.name }} ({{ t.slug }})</option>
          </select>
          <p class="text-xs text-muted-foreground mt-1.5">
            Sekolah ini sudah terdaftar di modul lain (akademik/tahfiz/dll) dengan siswa aslinya sendiri. PTN akan
            memakai data sekolah/siswa yang sudah ada — gunakan tombol "Import Siswa" nanti untuk memilih siswa
            mana saja yang boleh akses PTN, bukan membuat akun baru.
          </p>
        </div>
      </div>

      <div>
        <Label>Nama Sekolah</Label>
        <Input v-model="name" placeholder="Contoh: SMA Negeri 1 Jakarta" class="mt-1.5" />
      </div>
      <div class="grid sm:grid-cols-2 gap-4">
        <div>
          <Label>Jenjang</Label>
          <select v-model="schoolType" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
            <option value="smp">SMP</option>
            <option value="sma">SMA</option>
            <option value="smk">SMK</option>
            <option value="ma">MA</option>
          </select>
        </div>
        <div>
          <Label>Paket</Label>
          <select v-model="packageType" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
            <option value="basic">Basic</option>
            <option value="premium">Premium</option>
            <option value="enterprise">Enterprise</option>
          </select>
        </div>
        <div>
          <Label>Kota</Label>
          <Input v-model="city" placeholder="Jakarta Pusat" class="mt-1.5" />
        </div>
        <div>
          <Label>Provinsi</Label>
          <Input v-model="province" placeholder="DKI Jakarta" class="mt-1.5" />
        </div>
        <div>
          <Label>Email</Label>
          <Input v-model="email" type="email" placeholder="info@sekolah.sch.id" class="mt-1.5" />
        </div>
        <div>
          <Label>Telepon</Label>
          <Input v-model="phone" placeholder="021-xxxxxxx" class="mt-1.5" />
        </div>
      </div>
      <div>
        <Label>Contact Person</Label>
        <Input v-model="contactPerson" placeholder="Nama kepala sekolah / PIC" class="mt-1.5" />
      </div>
      <div class="grid sm:grid-cols-2 gap-4">
        <div>
          <Label>Revenue Share (%)</Label>
          <Input v-model.number="revenueShare" type="number" min="0" max="50" class="mt-1.5" />
        </div>
        <div>
          <Label>Nilai Kontrak / Bulan (Rp)</Label>
          <Input v-model.number="monthlyRevenue" type="number" min="0" class="mt-1.5" />
        </div>
      </div>

      <div>
        <Label>Status {{ isEditing ? '' : 'Awal' }}</Label>
        <select v-model="status" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
          <option value="pending">Pending (onboarding wizard)</option>
          <option value="active">Langsung Aktifkan</option>
          <option value="inactive">Tidak Aktif</option>
        </select>
        <p v-if="!isEditing && status === 'pending'" class="text-xs text-muted-foreground mt-1.5">
          Setelah disimpan, gunakan tombol "Setujui &amp; Onboarding" untuk import siswa dan mengaktifkan mitra.
        </p>
      </div>

      <div class="flex items-center justify-between pt-4 border-t">
        <Button variant="outline" @click="open = false">Batal</Button>
        <Button variant="gradient" :disabled="saving" @click="save">
          <Sparkles class="w-4 h-4" />
          {{ saving ? 'Menyimpan...' : isEditing ? 'Update Sekolah' : 'Simpan Sekolah' }}
        </Button>
      </div>
    </div>
  </Dialog>
</template>
