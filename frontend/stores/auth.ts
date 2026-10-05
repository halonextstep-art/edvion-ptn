import { defineStore } from 'pinia'
import type { User } from '~/types'

const TOKEN_KEY = 'edvionptn_token'
const USER_KEY = 'edvionptn_user'

export const useAuthStore = defineStore('auth', {
  state: () => ({
    token: null as string | null,
    user: null as User | null,
    hydrated: false,
  }),

  getters: {
    isAuthenticated: (state) => !!state.token && !!state.user,
    role: (state) => state.user?.role ?? null,
  },

  actions: {
    setSession(token: string, user: User) {
      this.token = token
      this.user = user
      if (import.meta.client) {
        localStorage.setItem(TOKEN_KEY, token)
        localStorage.setItem(USER_KEY, JSON.stringify(user))
      }
    },

    // Refreshes the cached user after a self-service profile edit — same persistence as
    // setSession but without touching the token (login/session stays the same).
    setUser(user: User) {
      this.user = user
      if (import.meta.client) {
        localStorage.setItem(USER_KEY, JSON.stringify(user))
      }
    },

    hydrate() {
      if (!import.meta.client || this.hydrated) return
      const token = localStorage.getItem(TOKEN_KEY)
      const raw = localStorage.getItem(USER_KEY)
      if (token && raw) {
        try {
          this.token = token
          this.user = JSON.parse(raw) as User
        } catch {
          this.logout()
        }
      }
      this.hydrated = true
    },

    logout() {
      this.token = null
      this.user = null
      if (import.meta.client) {
        localStorage.removeItem(TOKEN_KEY)
        localStorage.removeItem(USER_KEY)
      }
    },
  },
})
