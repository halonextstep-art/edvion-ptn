<script setup lang="ts">
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '~/lib/utils'

const badgeVariants = cva(
  'inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors',
  {
    variants: {
      variant: {
        default: 'border-transparent bg-primary text-primary-foreground',
        secondary: 'border-transparent bg-secondary text-secondary-foreground',
        outline: 'text-foreground',
        success: 'border-transparent bg-emerald-100 text-emerald-700 dark:bg-emerald-500/20 dark:text-emerald-300',
        warning: 'border-transparent bg-amber-100 text-amber-700 dark:bg-amber-500/20 dark:text-amber-300',
        destructive: 'border-transparent bg-red-100 text-red-700 dark:bg-red-500/20 dark:text-red-300',
      },
    },
    defaultVariants: { variant: 'default' },
  },
)

type Variant = VariantProps<typeof badgeVariants>
withDefaults(defineProps<{ variant?: Variant['variant'] }>(), { variant: 'default' })
defineOptions({ inheritAttrs: false })
</script>

<template>
  <span :class="cn(badgeVariants({ variant }), $attrs.class as string)">
    <slot />
  </span>
</template>
