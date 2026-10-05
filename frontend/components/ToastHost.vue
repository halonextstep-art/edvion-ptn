<script setup lang="ts">
const { toasts } = useToast()

const variantClasses: Record<string, string> = {
  success: 'border-emerald-200 bg-emerald-50 text-emerald-800',
  error: 'border-red-200 bg-red-50 text-red-800',
  info: 'border-blue-200 bg-blue-50 text-blue-800',
}
</script>

<template>
  <div class="fixed top-4 right-4 z-[100] flex flex-col gap-2 w-80 max-w-[90vw]">
    <TransitionGroup name="toast">
      <div
        v-for="t in toasts"
        :key="t.id"
        class="rounded-xl border shadow-lg px-4 py-3 text-sm"
        :class="variantClasses[t.variant]"
      >
        <p class="font-semibold">{{ t.title }}</p>
        <p v-if="t.description" class="text-xs opacity-80 mt-0.5">{{ t.description }}</p>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: all 0.2s ease;
}
.toast-enter-from {
  opacity: 0;
  transform: translateY(-8px);
}
.toast-leave-to {
  opacity: 0;
  transform: translateX(20px);
}
</style>
