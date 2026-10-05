import type { ApiClient } from './ApiClient'
import type { Question, QuestionSetItem } from '~/types'

export class QuestionSetService {
  constructor(private api: ApiClient) {}

  list() {
    return this.api.get<QuestionSetItem[]>('/question-sets')
  }

  // Backend wraps the member list as `{ items: Question[] }` (SetItemsResponse) — reusing
  // the same QuestionResponse shape the Bank Soal endpoints return — so unwrap it here to
  // give callers the flat `Question[]` they actually want.
  async getItems(id: string) {
    const res = await this.api.get<{ items: Question[] }>(`/question-sets/${id}/items`)
    return res.items
  }

  create(payload: { name: string; description?: string }) {
    return this.api.post<QuestionSetItem>('/question-sets', payload)
  }

  update(id: string, payload: { name: string; description?: string }) {
    return this.api.put<QuestionSetItem>(`/question-sets/${id}`, payload)
  }

  remove(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/question-sets/${id}`)
  }

  addItem(setId: string, questionId: string) {
    return this.api.post<{ added: boolean }>(`/question-sets/${setId}/items`, { question_id: questionId })
  }

  removeItem(setId: string, questionId: string) {
    return this.api.delete<{ removed: boolean }>(`/question-sets/${setId}/items/${questionId}`)
  }
}
