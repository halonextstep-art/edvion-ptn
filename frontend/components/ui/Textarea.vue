<script setup lang="ts">
import { cn } from '~/lib/utils'

defineProps<{ modelValue?: string; placeholder?: string; rows?: number }>()
defineEmits<{ 'update:modelValue': [value: string] }>()
defineOptions({ inheritAttrs: false })
</script>

<template>
  <textarea
    :value="modelValue"
    :placeholder="placeholder"
    :rows="rows || 4"
    @input="$emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    :class="cn(
      'flex w-full rounded-lg border border-input bg-white px-3 py-2 text-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring resize-none dark:bg-white/5 dark:text-white dark:placeholder:text-white/30',
      $attrs.class as string,
    )"
    v-bind="{ ...$attrs, class: undefined }"
  />
</template>
