<script setup lang="ts">
// Renders one Badge/Package icon consistently everywhere it's displayed — either the
// curated preset lucide icon (icon_type === 'preset') or the admin-uploaded custom image
// (icon_type === 'custom'). Pairs with IconPicker.vue (which sets these same 3 fields);
// see utils/presetIcons.ts for the shared preset catalogue both rely on.
import { resolveIconComponent } from '~/utils/presetIcons'
import type { IconType } from '~/types'

const props = withDefaults(
  defineProps<{
    iconType?: IconType | null
    iconName?: string | null
    iconUrl?: string | null
    /** Tailwind size classes applied to the icon/image itself, e.g. 'w-5 h-5'. */
    size?: string
  }>(),
  { size: 'w-5 h-5', iconType: 'preset', iconName: null, iconUrl: null },
)

const { resolve: resolveUploadUrl } = useUploadUrl()

const resolvedUrl = computed(() => (props.iconType === 'custom' ? resolveUploadUrl(props.iconUrl) : null))
const IconComponent = computed(() => resolveIconComponent(props.iconName))
</script>

<template>
  <img v-if="resolvedUrl" :src="resolvedUrl" alt="" class="object-cover rounded" :class="size" />
  <component :is="IconComponent" v-else :class="size" />
</template>
