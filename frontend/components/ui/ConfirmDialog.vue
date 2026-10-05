<script setup lang="ts">
import { AlertTriangle } from 'lucide-vue-next'
import Button from './Button.vue'

const open = defineModel<boolean>({ default: false })
withDefaults(
  defineProps<{
    title?: string
    description?: string
    confirmLabel?: string
    cancelLabel?: string
    danger?: boolean
  }>(),
  {
    title: 'Apakah kamu yakin?',
    description: 'Tindakan ini tidak dapat dibatalkan.',
    confirmLabel: 'Konfirmasi',
    cancelLabel: 'Batal',
    danger: true,
  },
)
const emit = defineEmits<{ confirm: [] }>()

function confirm() {
  emit('confirm')
  open.value = false
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="fixed inset-0 z-[60] flex items-center justify-center p-4">
      <div class="absolute inset-0 bg-black/40" @click="open = false" />
      <div class="relative bg-white rounded-2xl shadow-2xl w-full max-w-sm p-6 text-center">
        <div class="w-12 h-12 rounded-full bg-red-100 flex items-center justify-center mx-auto mb-4">
          <AlertTriangle class="w-6 h-6 text-red-500" />
        </div>
        <h3 class="font-bold text-slate-900 mb-1.5">{{ title }}</h3>
        <p class="text-sm text-muted-foreground mb-5">{{ description }}</p>
        <div class="flex gap-2">
          <Button variant="outline" class="flex-1" @click="open = false">{{ cancelLabel }}</Button>
          <Button :class="danger ? 'flex-1 bg-red-600 hover:bg-red-700 text-white' : 'flex-1'" @click="confirm">
            {{ confirmLabel }}
          </Button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
