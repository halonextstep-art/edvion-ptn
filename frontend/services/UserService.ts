import type { ApiClient } from './ApiClient'
import type { CreateUserPayload, Role, UpdateUserPayload, User, UserStatus } from '~/types'

export interface UserListFilter {
  role?: Role
  status?: UserStatus
  search?: string
  /** `true` = only B2B (has a school), `false` = only B2C (mandiri, no school). */
  has_school?: boolean
}

export class UserService {
  constructor(private api: ApiClient) {}

  list(filter: UserListFilter = {}) {
    return this.api.get<User[]>('/users', filter as Record<string, unknown>)
  }

  create(payload: CreateUserPayload) {
    return this.api.post<User>('/users', payload)
  }

  update(id: string, payload: UpdateUserPayload) {
    return this.api.put<User>(`/users/${id}`, payload)
  }

  remove(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/users/${id}`)
  }

  setStatus(id: string, status: UserStatus) {
    return this.api.patch<User>(`/users/${id}/status`, { status })
  }
}
