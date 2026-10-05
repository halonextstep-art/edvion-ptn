<script setup lang="ts">
import { cn } from '~/lib/utils'
defineOptions({ inheritAttrs: false })
</script>

<template>
  <div role="tablist" :class="cn('inline-flex items-center gap-1 rounded-xl bg-slate-100 p-1', $attrs.class as string)">
    <slot />
  </div>
</template>
