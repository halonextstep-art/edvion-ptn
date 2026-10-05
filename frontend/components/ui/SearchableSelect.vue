<script setup lang="ts">
// Combobox generik: tombol yang membuka panel berisi input pencarian + daftar opsi
// (opsional dikelompokkan, dengan header grup). Dibuat untuk mengganti <select> polos di
// tempat-tempat yang daftar opsinya bisa panjang (mis. "Mata Uji" untuk kategori TKA yang
// mata pelajaran pilihannya banyak) — <select> native tidak punya cara cari-ketik built-in
// di semua browser, combobox ini memberi itu tanpa mengubah bentuk data (masih emit satu
// string value seperti v-model di <select> biasa).
import { ChevronDown, Search, Check } from 'lucide-vue-next'
import { cn } from '~/lib/utils'

export interface SearchableSelectOption {
  value: string
  label: string
}
export interface SearchableSelectGroup {
  label: string
  options: SearchableSelectOption[]
}

const props = defineProps<{
  modelValue: string
  groups: SearchableSelectGroup[]
  placeholder?: string
  id?: string
  required?: boolean
}>()
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

const open = ref(false)
const query = ref('')
const root = ref<HTMLElement | null>(null)
const searchInput = ref<HTMLInputElement | null>(null)

const selectedLabel = computed(() => {
  for (const g of props.groups) {
    const found = g.options.find((o) => o.value === props.modelValue)
    if (found) return found.label
  }
  return ''
})

const filteredGroups = computed(() => {
  const q = query.value.trim().toLowerCase()
  if (!q) return props.groups
  return props.groups
    .map((g) => ({ ...g, options: g.options.filter((o) => o.label.toLowerCase().includes(q)) }))
    .filter((g) => g.options.length > 0)
})

const flatFiltered = computed(() => filteredGroups.value.flatMap((g) => g.options))
const highlightIndex = ref(0)
watch(filteredGroups, () => { highlightIndex.value = 0 })

function toggle() {
  open.value = !open.value
  if (open.value) {
    query.value = ''
    highlightIndex.value = 0
    nextTick(() => searchInput.value?.focus())
  }
}

function select(value: string) {
  emit('update:modelValue', value)
  open.value = false
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    open.value = false
    return
  }
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    highlightIndex.value = Math.min(highlightIndex.value + 1, flatFiltered.value.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    highlightIndex.value = Math.max(highlightIndex.value - 1, 0)
  } else if (e.key === 'Enter') {
    e.preventDefault()
    const opt = flatFiltered.value[highlightIndex.value]
    if (opt) select(opt.value)
  }
}

function onClickOutside(e: MouseEvent) {
  if (open.value && root.value && !root.value.contains(e.target as Node)) open.value = false
}
onMounted(() => document.addEventListener('mousedown', onClickOutside))
onUnmounted(() => document.removeEventListener('mousedown', onClickOutside))
</script>

<template>
  <div ref="root" class="relative">
    <!-- Placeholder murni buat validasi HTML "required" bawaan form, sinkron ke modelValue, tidak pernah terlihat pengguna. -->
    <input :value="modelValue" :required="required" tabindex="-1" class="sr-only" aria-hidden="true" @focus="searchInput?.focus()" />
    <button
      :id="id"
      type="button"
      class="flex h-10 w-full items-center justify-between rounded-lg border border-input bg-white px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring dark:bg-white/5 dark:text-white"
      @click="toggle"
    >
      <span :class="cn('truncate text-left', !selectedLabel && 'text-muted-foreground')">{{ selectedLabel || placeholder || 'Pilih...' }}</span>
      <ChevronDown class="w-4 h-4 shrink-0 text-muted-foreground" />
    </button>

    <div
      v-if="open"
      class="absolute z-20 mt-1 w-full rounded-lg border bg-white shadow-lg dark:bg-slate-900"
    >
      <div class="flex items-center gap-2 px-2.5 py-2 border-b">
        <Search class="w-3.5 h-3.5 text-muted-foreground shrink-0" />
        <input
          ref="searchInput"
          v-model="query"
          type="text"
          placeholder="Cari..."
          class="w-full text-sm outline-none bg-transparent"
          @keydown="onKeydown"
        />
      </div>
      <div class="max-h-64 overflow-y-auto py-1">
        <template v-if="flatFiltered.length > 0">
          <div v-for="g in filteredGroups" :key="g.label">
            <p class="px-3 pt-2 pb-1 text-[10px] font-bold uppercase tracking-wider text-muted-foreground">{{ g.label }}</p>
            <button
              v-for="o in g.options"
              :key="o.value"
              type="button"
              class="flex w-full items-center justify-between gap-2 px-3 py-1.5 text-sm text-left hover:bg-slate-50 dark:hover:bg-white/5"
              :class="flatFiltered[highlightIndex]?.value === o.value ? 'bg-slate-50 dark:bg-white/5' : ''"
              @click="select(o.value)"
            >
              <span class="truncate">{{ o.label }}</span>
              <Check v-if="o.value === modelValue" class="w-3.5 h-3.5 text-indigo-600 shrink-0" />
            </button>
          </div>
        </template>
        <p v-else class="px-3 py-4 text-sm text-center text-muted-foreground">Tidak ditemukan</p>
      </div>
    </div>
  </div>
</template>
