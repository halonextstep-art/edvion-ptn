<script setup lang="ts">
import { inject, type Ref } from 'vue'
import { cn } from '~/lib/utils'

const props = defineProps<{ value: string }>()
const model = inject<Ref<string>>('tabsModel')!
defineOptions({ inheritAttrs: false })
</script>

<template>
  <button
    type="button"
    role="tab"
    :aria-selected="model === props.value"
    @click="model = props.value"
    :class="cn(
      'inline-flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm font-semibold transition-all',
      model === props.value ? 'bg-white shadow-sm text-slate-900' : 'text-slate-500 hover:text-slate-700',
      $attrs.class as string,
    )"
  >
    <slot />
  </button>
</template>
