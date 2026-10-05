import type { ApiClient } from './ApiClient'
import type { IrtCalibrationStatus, IrtRecalibrationSummary, PlatformSettings, ScoreDisplayMode } from '~/types'

/** Admin-only platform-wide settings — B2C self-registration toggle + UTBK/SNBT "Skor Instan"
 * vs "Estimasi IRT" display mode. See backend `application::platform_settings_service` and
 * `application::irt_service` doc comments. */
export class SettingsService {
  constructor(private api: ApiClient) {}

  getB2cRegistration() {
    return this.api.get<PlatformSettings>('/settings/b2c-registration')
  }

  /** Same singleton settings row as `getB2cRegistration` (the backend has one settings endpoint
   *  returning every flag) — named generically for callers that only care about
   *  `score_display_mode`, e.g. the "Pengaturan Skor UTBK" panel. */
  getSettings() {
    return this.api.get<PlatformSettings>('/settings/b2c-registration')
  }

  setB2cRegistration(enabled: boolean) {
    return this.api.patch<PlatformSettings>('/settings/b2c-registration', { enabled })
  }

  setScoreDisplayMode(mode: ScoreDisplayMode) {
    return this.api.patch<PlatformSettings>('/settings/score-display-mode', { mode })
  }

  /** Global "Mode Terkunci" default applied to every Simulasi TO that doesn't set its own
   *  explicit per-template override — see `PlatformSettings.simulation_lockdown_default`. */
  setSimulationLockdownDefault(enabled: boolean) {
    return this.api.patch<PlatformSettings>('/settings/simulation-lockdown-default', { enabled })
  }

  getIrtStatus() {
    return this.api.get<IrtCalibrationStatus>('/settings/irt/status')
  }

  recalibrateIrt() {
    return this.api.post<IrtRecalibrationSummary>('/settings/irt/recalibrate')
  }
}
