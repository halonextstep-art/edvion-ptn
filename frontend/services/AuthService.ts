import type { ApiClient } from './ApiClient'
import type { AuthResponse, ChangePasswordPayload, Role, UpdateProfilePayload, User } from '~/types'

export interface RegisterPayload {
  name: string
  email: string
  password: string
  role: Role
  school_id?: string | null
}

export interface LoginPayload {
  // Either a PTN account's own email, or a legacy cross-module account's username (shared
  // `core.akun` table) — see backend `AuthService::login` doc comment.
  username: string
  // Only needed to disambiguate a username login for a specific partner school
  // (`core.tenant.slug`) — omit for PTN's own email-based accounts, unambiguous without it.
  tenant_slug?: string | null
  password: string
}

export class AuthService {
  constructor(private api: ApiClient) {}

  register(payload: RegisterPayload) {
    return this.api.post<AuthResponse>('/auth/register', payload)
  }

  login(payload: LoginPayload) {
    return this.api.post<AuthResponse>('/auth/login', payload)
  }

  me() {
    return this.api.get<User>('/auth/me')
  }

  updateProfile(payload: UpdateProfilePayload) {
    return this.api.put<User>('/auth/me', payload)
  }

  changePassword(payload: ChangePasswordPayload) {
    return this.api.put<{ message: string }>('/auth/me/password', payload)
  }

  uploadAvatar(file: File) {
    const form = new FormData()
    form.append('file', file)
    return this.api.upload<User>('/auth/me/avatar', form)
  }
}
