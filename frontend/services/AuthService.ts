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
  email: string
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
