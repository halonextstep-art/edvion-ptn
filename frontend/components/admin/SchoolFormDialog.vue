<script setup lang="ts">
import { Sparkles } from 'lucide-vue-next'
import type { PackageType, School, SchoolStatus, SchoolType } from '~/types'

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

const isEditing = computed(() => !!props.editSchool)

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
}

watch(open, (isOpen) => {
  if (isOpen) resetForm(props.editSchool)
})

async function save() {
  if (!name.value.trim()) return toast.error('Nama sekolah tidak boleh kosong!')
  if (!email.value.trim()) return toast.error('Email tidak boleh kosong!')
  if (!contactPerson.value.trim()) return toast.error('Nama kontak person harus diisi!')

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
