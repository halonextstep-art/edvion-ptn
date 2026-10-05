<script setup lang="ts">
import { Sparkles } from 'lucide-vue-next'
import type { AdmissionDeadlineItem, AdmissionTrack } from '~/types'

const open = defineModel<boolean>({ default: false })
const props = defineProps<{ editDeadline?: AdmissionDeadlineItem | null }>()
const emit = defineEmits<{ saved: [] }>()

const { admissionDeadlineService } = useApi()
const toast = useToast()

const track = ref<AdmissionTrack>('snbp')
const year = ref(new Date().getFullYear())
const label = ref('')
const description = ref('')
const deadlineDate = ref('')
const saving = ref(false)

const isEditing = computed(() => !!props.editDeadline)

function resetForm(d?: AdmissionDeadlineItem | null) {
  track.value = d?.track || 'snbp'
  year.value = d?.year || new Date().getFullYear()
  label.value = d?.label || ''
  description.value = d?.description || ''
  deadlineDate.value = d?.deadline_date?.slice(0, 10) || ''
}

watch(open, (isOpen) => {
  if (isOpen) resetForm(props.editDeadline)
})

async function save() {
  if (!label.value.trim()) return toast.error('Label deadline tidak boleh kosong!')
  if (!deadlineDate.value) return toast.error('Tanggal deadline wajib diisi!')

  saving.value = true
  try {
    const payload = {
      track: track.value,
      year: year.value,
      label: label.value,
      description: description.value,
      deadline_date: deadlineDate.value,
    }
    if (isEditing.value && props.editDeadline) {
      await admissionDeadlineService.update(props.editDeadline.id, payload)
      toast.success(`Deadline "${label.value}" berhasil diperbarui`)
    } else {
      await admissionDeadlineService.create(payload)
      toast.success(`Deadline "${label.value}" berhasil ditambahkan`)
    }
    open.value = false
    emit('saved')
  } catch (e: any) {
    toast.error('Gagal menyimpan deadline', e?.message)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <Dialog v-model="open" :title="isEditing ? 'Edit Deadline' : 'Tambah Deadline Baru'" max-width="max-w-lg">
    <div class="space-y-4">
      <div class="grid sm:grid-cols-2 gap-4">
        <div>
          <Label>Jalur Seleksi *</Label>
          <select v-model="track" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
            <option value="snbp">SNBP</option>
            <option value="snbt">SNBT</option>
            <option value="utbk">UTBK</option>
          </select>
        </div>
        <div>
          <Label>Tahun *</Label>
          <Input v-model.number="year" type="number" min="2020" class="mt-1.5" />
        </div>
      </div>

      <div>
        <Label>Label *</Label>
        <Input v-model="label" placeholder="Penutupan Pendaftaran SNBP" class="mt-1.5" />
      </div>

      <div>
        <Label>Tanggal Deadline *</Label>
        <Input v-model="deadlineDate" type="date" class="mt-1.5" />
      </div>

      <div>
        <Label>Keterangan (opsional)</Label>
        <textarea
          v-model="description"
          class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm resize-none h-20 focus:outline-none focus:ring-2 focus:ring-indigo-300"
          placeholder="Detail tambahan, sumber resmi (SNPMB), dsb..."
        />
      </div>

      <div class="flex items-center justify-between pt-4 border-t">
        <Button variant="outline" @click="open = false">Batal</Button>
        <Button variant="gradient" :disabled="saving" @click="save">
          <Sparkles class="w-4 h-4" />
          {{ saving ? 'Menyimpan...' : isEditing ? 'Update Deadline' : 'Simpan Deadline' }}
        </Button>
      </div>
    </div>
  </Dialog>
</template>
