import type { ApiClient } from './ApiClient'
import type { SubjectCategoryWithSubjects, SubjectItem } from '~/types'

export class TaxonomyService {
  constructor(private api: ApiClient) {}

  list() {
    return this.api.get<SubjectCategoryWithSubjects[]>('/taxonomy')
  }

  createCategory(payload: { name: string; description?: string; sort_order?: number }) {
    return this.api.post<SubjectCategoryWithSubjects>('/taxonomy/categories', payload)
  }

  updateCategory(id: string, payload: { name: string; description?: string; sort_order?: number }) {
    return this.api.put<SubjectCategoryWithSubjects>(`/taxonomy/categories/${id}`, payload)
  }

  deleteCategory(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/taxonomy/categories/${id}`)
  }

  createSubject(payload: { category_id: string; name: string; code?: string; sort_order?: number }) {
    return this.api.post<SubjectItem>('/taxonomy/subjects', payload)
  }

  updateSubject(id: string, payload: { name: string; code?: string; sort_order?: number }) {
    return this.api.put<SubjectItem>(`/taxonomy/subjects/${id}`, payload)
  }

  deleteSubject(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/taxonomy/subjects/${id}`)
  }
}
