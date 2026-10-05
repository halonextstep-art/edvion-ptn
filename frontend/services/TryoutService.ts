import type { ApiClient } from './ApiClient'
import type {
  Attempt,
  AttemptResult,
  AttemptResumeResponse,
  AttemptWithQuestions,
  ExamTrack,
  ReviewItem,
  SchoolType,
  SessionType,
  TryoutSessionTemplate,
} from '~/types'

export interface CreateSessionPayload {
  title: string
  session_type: SessionType
  duration_minutes: number
  question_count: number
  subject_filter?: string | null
  topic_filter?: string | null
  difficulty_filter?: string | null
  is_premium?: boolean
  /** Non-null pins this session to a curated "Set Soal" instead of a random draw from the
   *  bank soal filtered by subject/topic/difficulty. Must be sent as explicit `null` (not
   *  omitted) to clear a previously-assigned set — see SessionFormDialog.vue. */
  question_set_id?: string | null
  /** `true` hides this session from the student/school catalogue — used by the "Buat Paket +
   *  Subtes" wizard while a subtes is still being assembled. Omitted/false = published
   *  immediately, matching the pre-existing SessionFormDialog behavior. */
  is_draft?: boolean
  /** See `TryoutSessionTemplate.is_elective` doc comment. */
  is_elective?: boolean
  /** See `ExamTrack` doc comment. Defaults to 'snbt' server-side if omitted. */
  exam_track?: ExamTrack
  /** See `TryoutSessionTemplate.school_type_scope` doc comment. Omitted/`null` = semua jenjang. */
  school_type_scope?: SchoolType | null
}

export class TryoutService {
  constructor(private api: ApiClient) {}

  listSessions(type?: SessionType) {
    return this.api.get<TryoutSessionTemplate[]>('/tryout/sessions', type ? { type } : undefined)
  }

  getSession(id: string) {
    return this.api.get<TryoutSessionTemplate>(`/tryout/sessions/${id}`)
  }

  createSession(payload: CreateSessionPayload) {
    return this.api.post<TryoutSessionTemplate>('/tryout/sessions', payload)
  }

  updateSession(id: string, payload: CreateSessionPayload) {
    return this.api.put<TryoutSessionTemplate>(`/tryout/sessions/${id}`, payload)
  }

  deleteSession(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/tryout/sessions/${id}`)
  }

  startAttempt(sessionId: string) {
    return this.api.post<AttemptWithQuestions>(`/tryout/sessions/${sessionId}/start`)
  }

  saveAnswer(attemptId: string, questionId: string, answerText: string | null, flagged: boolean) {
    return this.api.put<{ saved: boolean }>(`/tryout/attempts/${attemptId}/answers`, {
      question_id: questionId,
      answer_text: answerText,
      flagged,
    })
  }

  submitAttempt(attemptId: string, timeUsedSeconds: number) {
    return this.api.post<AttemptResult>(`/tryout/attempts/${attemptId}/submit`, {
      time_used_seconds: timeUsedSeconds,
    })
  }

  myAttempts() {
    return this.api.get<Attempt[]>('/tryout/attempts')
  }

  getAttempt(attemptId: string) {
    return this.api.get<Attempt>(`/tryout/attempts/${attemptId}`)
  }

  /** Rebuilds player state (questions + saved answers) for an existing in_progress attempt —
   *  used as a fallback when the player page's in-memory store is empty (e.g. after refresh). */
  resumeAttempt(attemptId: string) {
    return this.api.get<AttemptResumeResponse>(`/tryout/attempts/${attemptId}/resume`)
  }

  /** Admin-only — lihat semua attempt (termasuk in_progress) milik satu siswa tertentu. */
  adminListAttempts(studentId: string) {
    return this.api.get<Attempt[]>(`/tryout/admin/students/${studentId}/attempts`)
  }

  getReview(attemptId: string) {
    return this.api.get<ReviewItem[]>(`/tryout/attempts/${attemptId}/review`)
  }

  /** "Lihat Hasil" — skor + rincian per mata uji untuk attempt milik sendiri yang sudah
   *  disubmit, bisa dipanggil kapan saja setelahnya (bukan cuma sekali seperti hasil submit
   *  langsung) — dipakai halaman /hasil/[attemptId] untuk membuka kembali TO yang sudah selesai. */
  getResult(attemptId: string) {
    return this.api.get<AttemptResult>(`/tryout/attempts/${attemptId}/result`)
  }
}
