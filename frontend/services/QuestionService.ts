import type { ApiClient } from './ApiClient'
import type { ImportBatchMeta, ImportCommitResponse, ImportPreviewResponse, Question, QuestionListResponse, QuestionPayload } from '~/types'

export interface QuestionListFilter {
  search?: string
  subject?: string
  status?: string
  difficulty?: string
  question_type?: string
  mine?: boolean
  page?: number
  page_size?: number
}

export type ReviewAction = 'approve' | 'reject' | 'revision'

export class QuestionService {
  constructor(private api: ApiClient) {}

  list(filter: QuestionListFilter = {}) {
    return this.api.get<QuestionListResponse>('/questions', filter as Record<string, unknown>)
  }

  get(id: string) {
    return this.api.get<Question>(`/questions/${id}`)
  }

  create(payload: QuestionPayload) {
    return this.api.post<Question>('/questions', payload)
  }

  update(id: string, payload: QuestionPayload) {
    return this.api.put<Question>(`/questions/${id}`, payload)
  }

  remove(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/questions/${id}`)
  }

  review(id: string, action: ReviewAction, note?: string) {
    return this.api.post<Question>(`/questions/${id}/review`, { action, note })
  }

  /** Send a draft/rejected/revision question into the review queue (content: own only). */
  submitForReview(id: string) {
    return this.api.post<Question>(`/questions/${id}/submit-review`, {})
  }

  /** Read-only: parses an uploaded "Import Soal" .docx and returns every question it could
   * make sense of (plus a reason for every one it couldn't) — nothing is saved yet. See
   * backend `application::question_import_service` doc comment. */
  importPreview(file: File) {
    const form = new FormData()
    form.append('file', file)
    return this.api.upload<ImportPreviewResponse>('/questions/import/preview', form)
  }

  /** Re-parses the SAME file (no server-side caching between preview and commit — see backend
   * doc comment) and actually creates every successfully-parsed question, applying `meta`'s
   * subject/topic/etc. to all of them (the .docx format itself carries no per-question
   * subject/topic). */
  importCommit(file: File, meta: ImportBatchMeta) {
    const form = new FormData()
    form.append('file', file)
    form.append('subject', meta.subject)
    if (meta.topic) form.append('topic', meta.topic)
    if (meta.subtopic) form.append('subtopic', meta.subtopic)
    if (meta.bloom_level) form.append('bloom_level', meta.bloom_level)
    if (meta.time_limit != null) form.append('time_limit', String(meta.time_limit))
    form.append('submit_for_review', meta.submit_for_review ? 'true' : 'false')
    if (meta.question_set_id) form.append('question_set_id', meta.question_set_id)
    return this.api.upload<ImportCommitResponse>('/questions/import/commit', form)
  }
}
