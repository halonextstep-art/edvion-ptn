import type { LoginPayload, RegisterPayload } from '~/services/AuthService'
import type { ChangePasswordPayload, UpdateProfilePayload } from '~/types'

// Thin composable wrapping AuthService + the Pinia auth store, so pages/components
// call one thing (`useAuth()`) instead of juggling the service and the store.
export function useAuth() {
  const auth = useAuthStore()
  const { authService } = useApi()

  async function login(payload: LoginPayload) {
    const res = await authService.login(payload)
    auth.setSession(res.token, res.user)
    return res.user
  }

  async function register(payload: RegisterPayload) {
    const res = await authService.register(payload)
    auth.setSession(res.token, res.user)
    return res.user
  }

  function logout() {
    auth.logout()
    navigateTo('/login')
  }

  // Dipakai ProfileDialog (semua role) — edit nama/telepon sendiri, lalu refresh cache lokal.
  async function updateProfile(payload: UpdateProfilePayload) {
    const user = await authService.updateProfile(payload)
    auth.setUser(user)
    return user
  }

  // Ganti password sendiri — backend memverifikasi current_password dulu.
  function changePassword(payload: ChangePasswordPayload) {
    return authService.changePassword(payload)
  }

  // Upload/ganti foto profil sendiri — disimpan di disk lokal backend, lihat
  // `infrastructure::storage` (backend) & AuthService.uploadAvatar (frontend).
  async function uploadAvatar(file: File) {
    const user = await authService.uploadAvatar(file)
    auth.setUser(user)
    return user
  }

  return {
    user: computed(() => auth.user),
    role: computed(() => auth.role),
    isAuthenticated: computed(() => auth.isAuthenticated),
    login,
    register,
    logout,
    updateProfile,
    changePassword,
    uploadAvatar,
  }
}
