<script setup lang="ts">
// "Isi Paket" — assign/detach which TryoutSessions (drilling/tryout tunggal) and/or
// SimulationTemplates (Simulasi UTBK / TO) a Package unlocks. Replaces the old all-or-nothing
// model where owning ANY package unlocked every premium session platform-wide (see
// backend domain::package::PackageContentItem doc comment). The join is edit-anytime from
// either direction in principle, but this dialog is the ONE place admins manage it — from the
// Package side — so authoring order (soal -> sesi -> TO -> paket, or paket dulu baru isi) never
// matters: assign whatever already exists to this package whenever it's ready.
import { X, Plus, Trash2, ListChecks, Timer, FileText, Lock } from 'lucide-vue-next'
import type { PackageContentItem, PackageItem, SimulationTemplateItem, TryoutSessionTemplate } from '~/types'

const open = defineModel<boolean>({ default: false })
const props = defineProps<{ pkg: PackageItem | null }>()

const { packageService, tryoutService, simulationService } = useApi()
const toast = useToast()

const items = ref<PackageContentItem[]>([])
const loading = ref(false)
const sessions = ref<TryoutSessionTemplate[]>([])
const templates = ref<SimulationTemplateItem[]>([])
const optionsLoading = ref(false)

const pickType = ref<'tryout_session' | 'simulation_template'>('tryout_session')
const pickId = ref('')
const adding = ref(false)
const removingId = ref<string | null>(null)

async function loadItems() {
  if (!props.pkg) return
  loading.value = true
  try {
    items.value = await packageService.listContent(props.pkg.id)
  } catch (e: any) {
    toast.error('Gagal memuat isi paket', e?.message)
  } finally {
    loading.value = false
  }
}

async function loadOptions() {
  optionsLoading.value = true
  try {
    const [s, t] = await Promise.all([tryoutService.listSessions(), simulationService.listTemplates()])
    sessions.value = s
    templates.value = t
  } catch (e: any) {
    toast.error('Gagal memuat daftar sesi/template', e?.message)
  } finally {
    optionsLoading.value = false
  }
}

watch(open, (isOpen) => {
  if (isOpen) {
    pickType.value = 'tryout_session'
    pickId.value = ''
    loadItems()
    loadOptions()
  }
})

// Options not yet assigned to this package — avoids offering a duplicate that would just
// no-op server-side. Also filtered to the SAME exam_track as this package — backend now rejects
// a mismatch outright (see PackageService::add_content_item), so hiding mismatched options here
// avoids the admin picking one just to see it error out.
const assignedIds = computed(() => new Set(items.value.map((i) => i.content_id)))
const availableSessionOptions = computed(() => sessions.value.filter((s) => !assignedIds.value.has(s.id) && s.exam_track === props.pkg?.exam_track))
const availableTemplateOptions = computed(() => templates.value.filter((t) => !assignedIds.value.has(t.id) && t.exam_track === props.pkg?.exam_track))

watch(pickType, () => { pickId.value = '' })

async function addItem() {
  if (!props.pkg || !pickId.value) return
  adding.value = true
  try {
    await packageService.addContentItem(props.pkg.id, pickType.value, pickId.value)
    pickId.value = ''
    toast.success('Konten ditambahkan ke paket')
    await loadItems()
  } catch (e: any) {
    toast.error('Gagal menambahkan konten', e?.message)
  } finally {
    adding.value = false
  }
}

async function removeItem(item: PackageContentItem) {
  if (!props.pkg) return
  removingId.value = item.id
  try {
    await packageService.removeContentItem(props.pkg.id, item.id)
    toast.success('Konten dilepas dari paket')
    await loadItems()
  } catch (e: any) {
    toast.error('Gagal melepas konten', e?.message)
  } finally {
    removingId.value = null
  }
}

function typeLabel(t: string) {
  return t === 'simulation_template' ? 'Simulasi UTBK (TO)' : 'Sesi Tryout/Drilling'
}
</script>

<template>
  <Dialog v-model="open" :title="`Isi Paket — ${pkg?.name ?? ''}`" max-width="max-w-lg">
    <div class="space-y-4">
      <p class="text-xs text-muted-foreground flex items-start gap-1.5">
        <Lock class="w-3.5 h-3.5 shrink-0 mt-0.5" />
        Siswa yang memiliki paket ini (lewat voucher atau paket sekolah) hanya bisa mengakses konten premium yang terdaftar di sini — bukan semua konten premium di platform.
      </p>

      <div class="flex items-end gap-2 p-3 bg-slate-50 border rounded-lg">
        <div class="flex-1 min-w-0">
          <label class="text-xs font-medium text-slate-600 mb-1 block">Tambah Konten</label>
          <select v-model="pickType" class="w-full px-2.5 py-1.5 border rounded-lg text-xs bg-white mb-1.5">
            <option value="tryout_session">Sesi Tryout/Drilling</option>
            <option value="simulation_template">Simulasi UTBK (TO)</option>
          </select>
          <select v-model="pickId" class="w-full px-2.5 py-1.5 border rounded-lg text-xs bg-white" :disabled="optionsLoading">
            <option value="" disabled>{{ optionsLoading ? 'Memuat...' : 'Pilih konten...' }}</option>
            <template v-if="pickType === 'tryout_session'">
              <option v-for="s in availableSessionOptions" :key="s.id" :value="s.id">{{ s.title }}</option>
            </template>
            <template v-else>
              <option v-for="t in availableTemplateOptions" :key="t.id" :value="t.id">{{ t.title }}</option>
            </template>
          </select>
        </div>
        <Button size="sm" variant="gradient" class="shrink-0" :disabled="!pickId || adding" @click="addItem">
          <Plus class="w-3.5 h-3.5" />Tambah
        </Button>
      </div>

      <div v-if="loading" class="p-6 text-center text-xs text-muted-foreground">Memuat isi paket...</div>
      <div v-else-if="items.length === 0" class="p-6 text-center text-xs text-muted-foreground border border-dashed rounded-lg flex flex-col items-center gap-1.5">
        <ListChecks class="w-5 h-5 opacity-40" />
        Belum ada konten di paket ini — siswa pemilik paket ini belum bisa mengakses apa pun.
      </div>
      <div v-else class="space-y-1.5">
        <div v-for="item in items" :key="item.id" class="flex items-center gap-2 p-2.5 border rounded-lg bg-white">
          <component :is="item.content_type === 'simulation_template' ? Timer : FileText" class="w-4 h-4 text-indigo-600 shrink-0" />
          <div class="flex-1 min-w-0">
            <p class="text-xs font-semibold text-slate-900 truncate">{{ item.content_title }}</p>
            <p class="text-[10px] text-muted-foreground">{{ typeLabel(item.content_type) }}</p>
          </div>
          <button
            class="p-1.5 rounded hover:bg-red-50 disabled:opacity-40 shrink-0"
            title="Lepas dari paket"
            :disabled="removingId === item.id"
            @click="removeItem(item)"
          >
            <Trash2 class="w-3.5 h-3.5 text-red-500" />
          </button>
        </div>
      </div>

      <Button variant="outline" class="w-full" @click="open = false"><X class="w-4 h-4" />Tutup</Button>
    </div>
  </Dialog>
</template>
