export type ToastVariant = 'success' | 'error' | 'info'

export interface ToastItem {
  id: number
  title: string
  description?: string
  variant: ToastVariant
}

const toasts = ref<ToastItem[]>([])
let counter = 0

function push(variant: ToastVariant, title: string, description?: string) {
  const id = ++counter
  toasts.value.push({ id, title, description, variant })
  setTimeout(() => {
    toasts.value = toasts.value.filter((t) => t.id !== id)
  }, 4000)
}

// Lightweight global toast bus (mirrors the reference's `sonner` toast.success/error/info).
export function useToast() {
  return {
    toasts,
    success: (title: string, description?: string) => push('success', title, description),
    error: (title: string, description?: string) => push('error', title, description),
    info: (title: string, description?: string) => push('info', title, description),
  }
}
