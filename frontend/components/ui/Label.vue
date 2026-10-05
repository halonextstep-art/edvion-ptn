<script setup lang="ts">
import { cn } from '~/lib/utils'
defineProps<{ for?: string }>()
defineOptions({ inheritAttrs: false })
</script>

<template>
  <label :for="for" :class="cn('text-sm font-medium leading-none', $attrs.class as string)">
    <slot />
  </label>
</template>
