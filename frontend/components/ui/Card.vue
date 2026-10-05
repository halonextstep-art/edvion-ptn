<script setup lang="ts">
import { cn } from '~/lib/utils'
defineOptions({ inheritAttrs: false })
</script>

<template>
  <div :class="cn('rounded-2xl border bg-card text-card-foreground shadow-sm', $attrs.class as string)" v-bind="{ ...$attrs, class: undefined }">
    <slot />
  </div>
</template>
