import { defineStore } from 'pinia'
import type { AttemptWithQuestions } from '~/types'

// Bridges the question payload from "start attempt" (DrillingZone) to the player page
// (/tryout/[attemptId]) without a second network round-trip — the backend intentionally
// only returns questions at start time (never again, so the answer key can't leak via a
// refetch mid-attempt).
export const useAttemptSessionStore = defineStore('attemptSession', {
  state: () => ({ current: null as AttemptWithQuestions | null }),
  actions: {
    set(data: AttemptWithQuestions) {
      this.current = data
    },
    clear() {
      this.current = null
    },
  },
})
