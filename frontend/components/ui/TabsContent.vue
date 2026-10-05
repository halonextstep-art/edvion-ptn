<script setup lang="ts">
import { inject, type Ref } from 'vue'

const props = defineProps<{ value: string }>()
const model = inject<Ref<string>>('tabsModel')!
</script>

<template>
  <div v-if="model === props.value">
    <slot />
  </div>
</template>
