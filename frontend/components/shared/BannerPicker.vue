<script setup lang="ts">
// Shared Package banner form control — a single optional wide promotional image, distinct
// from the small square icon (see IconPicker.vue). No preset tab (a banner is, by nature,
// custom artwork) — just upload / preview / remove. Uploads to POST /api/uploads/banner,
// Admin-only (see backend upload_handler::upload_banner), then v-model carries the returned
// URL (or null) so PackageManager.vue can spread it straight into the package payload's
// `banner_url` field. `null` (the default) means the package card keeps rendering its
// gradient + icon exactly as before — this control is purely additive.
import { Upload, Loader2, X } from 'lucide-vue-next'

const model = defineModel<string | null>({ required: true })

const { client } = useApi()
const toast = useToast()
const { resolve: resolveUploadUrl } = useUploadUrl()
const previewUrl = computed(() => resolveUploadUrl(model.value))

const uploading = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

const MAX_BYTES = 3 * 1024 * 1024
const ALLOWED_MIME = ['image/png', 'image/jpeg', 'image/webp']

function pickFile() {
  fileInput.value?.click()
}

function removeBanner() {
  model.value = null
}

async function onFileSelected(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  if (!ALLOWED_MIME.includes(file.type)) {
    toast.error('Format tidak didukung', 'Gunakan PNG, JPEG, atau WEBP.')
    return
  }
  if (file.size > MAX_BYTES) {
    toast.error('File terlalu besar', 'Maksimum 3 MB.')
    return
  }
  uploading.value = true
  try {
    const form = new FormData()
    form.append('file', file)
    const res = await client.upload<{ url: string }>('/uploads/banner', form)
    model.value = res.url
    toast.success('Banner berhasil diunggah')
  } catch (err: any) {
    toast.error('Gagal mengunggah banner', err?.message)
  } finally {
    uploading.value = false
    if (fileInput.value) fileInput.value.value = ''
  }
}
</script>

<template>
  <div class="space-y-2">
    <input ref="fileInput" type="file" accept="image/png,image/jpeg,image/webp" class="hidden" @change="onFileSelected" />

    <div v-if="model" class="relative rounded-xl overflow-hidden border bg-slate-100 aspect-[3/1]">
      <img :src="previewUrl ?? undefined" alt="Banner paket" class="w-full h-full object-cover" />
      <button
        type="button"
        class="absolute top-2 right-2 w-7 h-7 rounded-full bg-black/60 hover:bg-black/80 text-white flex items-center justify-center transition-colors"
        title="Hapus banner"
        @click="removeBanner"
      >
        <X class="w-4 h-4" />
      </button>
    </div>

    <button
      v-else
      type="button" :disabled="uploading"
      class="w-full flex items-center justify-center gap-2 py-4 rounded-xl border-2 border-dashed text-xs font-semibold text-slate-500 hover:border-indigo-400 hover:text-indigo-600 transition-colors disabled:opacity-60"
      @click="pickFile"
    >
      <Loader2 v-if="uploading" class="w-4 h-4 animate-spin" />
      <Upload v-else class="w-4 h-4" />
      {{ uploading ? 'Mengunggah...' : 'Upload banner (opsional, landscape, PNG/JPEG/WEBP, maks 3MB)' }}
    </button>

    <button v-if="model" type="button" :disabled="uploading" class="text-xs font-semibold text-indigo-600 hover:text-indigo-700 disabled:opacity-60" @click="pickFile">
      Ganti banner
    </button>
  </div>
</template>
