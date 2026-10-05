<script setup lang="ts">
import { provide, type Ref } from 'vue'

const model = defineModel<string>({ required: true })
provide<Ref<string>>('tabsModel', model)
defineOptions({ inheritAttrs: false })
</script>

<template>
  <div :class="$attrs.class">
    <slot />
  </div>
</template>
