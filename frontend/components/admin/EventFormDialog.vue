<script setup lang="ts">
import { Sparkles } from 'lucide-vue-next'
import type { EventItem, EventType, TryoutSessionTemplate } from '~/types'

const open = defineModel<boolean>({ default: false })
const props = defineProps<{ editEvent?: EventItem | null }>()
const emit = defineEmits<{ saved: [] }>()

const { eventService, tryoutService } = useApi()
const toast = useToast()

const TARGET_CLASSES = ['Kelas 10', 'Kelas 11', 'Kelas 12', 'Kelas 11-12', 'Kelas 10-12']

const sessions = ref<TryoutSessionTemplate[]>([])

const name = ref('')
const eventType = ref<EventType>('tryout')
const description = ref('')
const sessionId = ref('')
const startDate = ref('')
const endDate = ref('')
const maxParticipants = ref<number | null>(null)
const price = ref(0)
const prizes = ref('')
const targetClass = ref('Kelas 12')
const saving = ref(false)

const isEditing = computed(() => !!props.editEvent)

function resetForm(e?: EventItem | null) {
  name.value = e?.name || ''
  eventType.value = e?.event_type || 'tryout'
  description.value = e?.description || ''
  sessionId.value = e?.session_id || sessions.value[0]?.id || ''
  startDate.value = e?.start_date?.slice(0, 10) || ''
  endDate.value = e?.end_date?.slice(0, 10) || ''
  maxParticipants.value = e?.max_participants ?? null
  price.value = e?.price ?? 0
  prizes.value = e?.prizes || ''
  targetClass.value = e?.target_class || 'Kelas 12'
}

watch(open, async (isOpen) => {
  if (!isOpen) return
  if (sessions.value.length === 0) {
    try {
      sessions.value = await tryoutService.listSessions()
    } catch (e: any) {
      toast.error('Gagal memuat daftar paket soal', e?.message)
    }
  }
  resetForm(props.editEvent)
})

async function save() {
  if (!name.value.trim()) return toast.error('Nama event tidak boleh kosong!')
  if (!sessionId.value) return toast.error('Pilih paket soal terlebih dahulu!')
  if (!startDate.value) return toast.error('Tanggal mulai wajib diisi!')

  saving.value = true
  try {
    const payload = {
      name: name.value,
      event_type: eventType.value,
      description: description.value,
      session_id: sessionId.value,
      start_date: startDate.value,
      end_date: endDate.value || null,
      max_participants: maxParticipants.value,
      price: price.value,
      prizes: prizes.value,
      target_class: targetClass.value,
      status: props.editEvent?.status,
    }
    if (isEditing.value && props.editEvent) {
      await eventService.update(props.editEvent.id, payload)
      toast.success(`Event "${name.value}" berhasil diperbarui`)
    } else {
      await eventService.create(payload)
      toast.success(`Event "${name.value}" berhasil dibuat`)
    }
    open.value = false
    emit('saved')
  } catch (e: any) {
    toast.error('Gagal menyimpan event', e?.message)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <Dialog v-model="open" :title="isEditing ? 'Edit Event' : 'Buat Event Baru'" max-width="max-w-2xl">
    <div class="space-y-4">
      <div>
        <Label>Nama Event *</Label>
        <Input v-model="name" placeholder="Tryout Nasional SNBT #46" class="mt-1.5" />
      </div>

      <div class="grid sm:grid-cols-2 gap-4">
        <div>
          <Label>Tipe Event</Label>
          <select v-model="eventType" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
            <option value="tryout">Tryout Full</option>
            <option value="mini">Mini Tryout</option>
            <option value="drilling">Drilling</option>
          </select>
        </div>
        <div>
          <Label>Target Kelas</Label>
          <select v-model="targetClass" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
            <option v-for="c in TARGET_CLASSES" :key="c" :value="c">{{ c }}</option>
          </select>
        </div>
      </div>

      <div>
        <Label>Deskripsi</Label>
        <textarea
          v-model="description"
          class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm resize-none h-20 focus:outline-none focus:ring-2 focus:ring-indigo-300"
          placeholder="Deskripsi singkat event..."
        />
      </div>

      <div class="grid sm:grid-cols-2 gap-4">
        <div>
          <Label>Tanggal Mulai *</Label>
          <Input v-model="startDate" type="date" class="mt-1.5" />
        </div>
        <div>
          <Label>Tanggal Selesai</Label>
          <Input v-model="endDate" type="date" class="mt-1.5" />
        </div>
      </div>

      <div class="grid sm:grid-cols-2 gap-4">
        <div>
          <Label>Maks. Peserta</Label>
          <Input v-model.number="maxParticipants" type="number" min="0" placeholder="Tanpa batas" class="mt-1.5" />
        </div>
        <div>
          <Label>Harga (Rp)</Label>
          <Input v-model.number="price" type="number" min="0" class="mt-1.5" />
        </div>
      </div>

      <div>
        <Label>Paket Soal *</Label>
        <select v-model="sessionId" class="w-full mt-1.5 px-3 py-2 border rounded-md text-sm">
          <option v-if="sessions.length === 0" value="">Belum ada sesi tryout/drilling — buat dulu di tab Sesi Tryout</option>
          <option v-for="s in sessions" :key="s.id" :value="s.id">{{ s.title }} ({{ s.duration_minutes }} menit, {{ s.question_count }} soal)</option>
        </select>
      </div>

      <div>
        <Label>Hadiah (opsional)</Label>
        <Input v-model="prizes" placeholder="Laptop, Voucher Rp 1jt, ..." class="mt-1.5" />
      </div>

      <div class="flex items-center justify-between pt-4 border-t">
        <Button variant="outline" @click="open = false">Batal</Button>
        <Button variant="gradient" :disabled="saving" @click="save">
          <Sparkles class="w-4 h-4" />
          {{ saving ? 'Menyimpan...' : isEditing ? 'Update Event' : 'Buat Event' }}
        </Button>
      </div>
    </div>
  </Dialog>
</template>
