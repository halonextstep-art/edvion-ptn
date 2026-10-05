// Usage: definePageMeta({ middleware: 'auth', roles: ['admin'] })
export default defineNuxtRouteMiddleware((to) => {
  if (import.meta.server) return // auth lives in localStorage; only enforce client-side

  const auth = useAuthStore()
  auth.hydrate()

  if (!auth.isAuthenticated) {
    return navigateTo('/login')
  }

  const allowedRoles = to.meta.roles as string[] | undefined
  if (allowedRoles && auth.role && !allowedRoles.includes(auth.role)) {
    return navigateTo('/')
  }
})
