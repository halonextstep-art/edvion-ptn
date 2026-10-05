import type { ApiClient } from './ApiClient'
import type {
  LockdownViolationItem, SimulationRunItem, SimulationTemplateItem, SimulationTemplatePayload, TemplateStatsItem,
} from '~/types'

// Admin authoring for Simulasi UTBK templates (chains existing tryout_sessions into one
// ordered, server-timed run), plus the student-facing run player methods below. The server
// is authoritative for all timing/state transitions — getRun() always returns freshly
// reconciled state (auto-submits an expired active slot, auto-advances an elapsed break)
// before responding, which is what lets the player page derive 100% of its state from a
// single GET on every mount/refresh instead of client-side session storage.
export class SimulationService {
  constructor(private api: ApiClient) {}

  listTemplates() {
    return this.api.get<SimulationTemplateItem[]>('/simulations/templates')
  }

  createTemplate(payload: SimulationTemplatePayload) {
    return this.api.post<SimulationTemplateItem>('/simulations/templates', payload)
  }

  setTemplateActive(id: string, active: boolean) {
    return this.api.patch<SimulationTemplateItem>(`/simulations/templates/${id}/active`, { active })
  }

  setTemplatePremium(id: string, isPremium: boolean) {
    return this.api.patch<SimulationTemplateItem>(`/simulations/templates/${id}/premium`, { is_premium: isPremium })
  }

  deleteTemplate(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/simulations/templates/${id}`)
  }

  /** Aggregate-only benchmark used by the sertifikat/laporan PDF — see
   *  `TemplateStatsItem` doc comment. */
  templateStats(templateId: string) {
    return this.api.get<TemplateStatsItem>(`/simulations/templates/${templateId}/stats`)
  }

  /** Per-template "Mode Terkunci" override — `null` resets it to inherit the platform-wide
   *  default. See `SimulationTemplateItem.lockdown_override`. */
  setTemplateLockdown(id: string, lockdownOverride: boolean | null) {
    return this.api.patch<SimulationTemplateItem>(`/simulations/templates/${id}/lockdown`, {
      lockdown_override: lockdownOverride,
    })
  }

  // Creates the run but leaves slot 1 `pending` (no attempt, no deadline yet) — the exam clock
  // doesn't start until beginRun() is called from the pre-exam overview screen.
  startRun(templateId: string) {
    return this.api.post<SimulationRunItem>(`/simulations/templates/${templateId}/start`, {})
  }

  // Called when the student clicks "Mulai Sekarang" on the overview screen — this is what
  // actually activates slot 1 and starts the clock. Idempotent if called again.
  beginRun(runId: string) {
    return this.api.post<SimulationRunItem>(`/simulations/runs/${runId}/begin`, {})
  }

  getRun(runId: string) {
    return this.api.get<SimulationRunItem>(`/simulations/runs/${runId}`)
  }

  submitCurrent(runId: string) {
    return this.api.post<SimulationRunItem>(`/simulations/runs/${runId}/submit-current`, {})
  }

  advance(runId: string) {
    return this.api.post<SimulationRunItem>(`/simulations/runs/${runId}/advance`, {})
  }

  // Escape hatch untuk run yang macet (mis. tab ditutup di tengah simulasi dan tidak pernah
  // lanjut) — tanpa ini siswa terkunci permanen karena start_run menolak run baru selama masih
  // ada satu yang berstatus in_progress.
  abandon(runId: string) {
    return this.api.post<{ abandoned: boolean }>(`/simulations/runs/${runId}/abandon`, {})
  }

  listMyRuns() {
    return this.api.get<SimulationRunItem[]>('/simulations/runs')
  }

  /** Admin-only — lihat semua run (termasuk in_progress) milik satu siswa tertentu. Dipakai di
   *  UserManager.vue drawer detail untuk isu "siswa lapor macet". */
  adminListRuns(studentId: string) {
    return this.api.get<SimulationRunItem[]>(`/simulations/admin/students/${studentId}/runs`)
  }

  /** Called by the "Mode Terkunci" player enforcement every time it detects a breach (tab
   *  switch, fullscreen exit, devtools attempt, etc.) — fire-and-forget from the caller's
   *  perspective, never blocks the exam flow. */
  reportViolation(runId: string, eventType: string, detail?: string) {
    return this.api.post<LockdownViolationItem>(`/simulations/runs/${runId}/violations`, {
      event_type: eventType,
      detail: detail ?? null,
    })
  }

  /** The run owner (student, their own history) or Admin (any student's, for review). */
  listViolations(runId: string) {
    return this.api.get<LockdownViolationItem[]>(`/simulations/runs/${runId}/violations`)
  }
}
