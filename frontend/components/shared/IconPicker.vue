<script setup lang="ts">
// Shared Badge/Package icon form control — two modes: "Pilih Ikon" (a grid of curated
// lucide-vue-next presets, see utils/presetIcons.ts) and "Upload Gambar" (uploads to
// POST /api/uploads/icon, Admin-only, see backend upload_handler::upload_icon, then uses
// the returned URL). v-model carries the full { icon_type, icon_name, icon_url } object so
// GamificationManager.vue / PackageManager.vue can spread it straight into their
// badge/package payload.
import { Upload, Loader2 } from 'lucide-vue-next'
import { PRESET_ICONS, PRESET_ICON_NAMES } from '~/utils/presetIcons'
import type { IconType } from '~/types'

export interface IconValue {
  icon_type: IconType
  icon_name: string | null
  icon_url: string | null
}

const model = defineModel<IconValue>({ required: true })

const { client } = useApi()
const toast = useToast()

const tab = ref<'preset' | 'custom'>(model.value.icon_type === 'custom' ? 'custom' : 'preset')
const uploading = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

const MAX_BYTES = 1024 * 1024
const ALLOWED_MIME = ['image/png', 'image/jpeg', 'image/webp', 'image/svg+xml']

function selectPreset(name: string) {
  model.value = { icon_type: 'preset', icon_name: name, icon_url: null }
}

function pickFile() {
  fileInput.value?.click()
}

async function onFileSelected(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  if (!ALLOWED_MIME.includes(file.type)) {
    toast.error('Format tidak didukung', 'Gunakan PNG, JPEG, WEBP, atau SVG.')
    return
  }
  if (file.size > MAX_BYTES) {
    toast.error('File terlalu besar', 'Maksimum 1 MB.')
    return
  }
  uploading.value = true
  try {
    const form = new FormData()
    form.append('file', file)
    const res = await client.upload<{ url: string }>('/uploads/icon', form)
    model.value = { icon_type: 'custom', icon_name: null, icon_url: res.url }
    toast.success('Ikon berhasil diunggah')
  } catch (err: any) {
    toast.error('Gagal mengunggah ikon', err?.message)
  } finally {
    uploading.value = false
    if (fileInput.value) fileInput.value.value = ''
  }
}
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center gap-3">
      <div class="w-14 h-14 rounded-2xl bg-slate-100 flex items-center justify-center overflow-hidden shrink-0 border">
        <IconDisplay :icon-type="model.icon_type" :icon-name="model.icon_name" :icon-url="model.icon_url" size="w-7 h-7 text-slate-600" />
      </div>
      <div class="flex gap-1 p-1 bg-slate-100 rounded-lg">
        <button
          type="button" class="px-3 py-1.5 rounded-md text-xs font-semibold transition-colors"
          :class="tab === 'preset' ? 'bg-white shadow-sm text-slate-900' : 'text-slate-500'"
          @click="tab = 'preset'"
        >
          Pilih Ikon
        </button>
        <button
          type="button" class="px-3 py-1.5 rounded-md text-xs font-semibold transition-colors"
          :class="tab === 'custom' ? 'bg-white shadow-sm text-slate-900' : 'text-slate-500'"
          @click="tab = 'custom'"
        >
          Upload Gambar
        </button>
      </div>
    </div>

    <div v-if="tab === 'preset'" class="flex flex-wrap gap-2 p-3 bg-slate-50 rounded-xl max-h-48 overflow-y-auto">
      <button
        v-for="name in PRESET_ICON_NAMES" :key="name" type="button" :title="name"
        class="w-9 h-9 rounded-lg flex items-center justify-center transition-colors"
        :class="model.icon_type === 'preset' && model.icon_name === name ? 'bg-indigo-100 ring-2 ring-indigo-400 text-indigo-600' : 'hover:bg-slate-200 text-slate-500'"
        @click="selectPreset(name)"
      >
        <component :is="PRESET_ICONS[name]" class="w-4 h-4" />
      </button>
    </div>

    <div v-else class="p-3 bg-slate-50 rounded-xl">
      <input ref="fileInput" type="file" accept="image/png,image/jpeg,image/webp,image/svg+xml" class="hidden" @change="onFileSelected" />
      <button
        type="button" :disabled="uploading"
        class="w-full flex items-center justify-center gap-2 py-3 rounded-lg border-2 border-dashed text-xs font-semibold text-slate-500 hover:border-indigo-400 hover:text-indigo-600 transition-colors disabled:opacity-60"
        @click="pickFile"
      >
        <Loader2 v-if="uploading" class="w-4 h-4 animate-spin" />
        <Upload v-else class="w-4 h-4" />
        {{ uploading ? 'Mengunggah...' : 'Pilih gambar (PNG/JPEG/WEBP/SVG, maks 1MB)' }}
      </button>
    </div>
  </div>
</template>
