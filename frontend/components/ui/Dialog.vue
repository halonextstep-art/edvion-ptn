<script setup lang="ts">
import { X } from 'lucide-vue-next'

const open = defineModel<boolean>({ default: false })
defineProps<{ title?: string; maxWidth?: string }>()
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div class="absolute inset-0 bg-black/40" @click="open = false" />
      <div
        class="relative w-full bg-white rounded-2xl shadow-2xl max-h-[90vh] overflow-y-auto"
        :class="maxWidth || 'max-w-2xl'"
      >
        <div class="flex items-center justify-between px-6 py-4 border-b sticky top-0 bg-white z-10">
          <slot name="header">
            <h2 class="text-lg font-bold text-slate-900">{{ title }}</h2>
          </slot>
          <button type="button" class="p-2 rounded-lg hover:bg-slate-100" @click="open = false">
            <X class="w-5 h-5" />
          </button>
        </div>
        <div class="p-6">
          <slot />
        </div>
      </div>
    </div>
  </Teleport>
</template>
