import type { ApiClient } from './ApiClient'
import type { GenerateReportPayload, ReportItem, ReportSummaryItem } from '~/types'

export class ReportService {
  constructor(private api: ApiClient) {}

  generate(payload: GenerateReportPayload) {
    return this.api.post<ReportItem>('/reports', payload)
  }
  list() {
    return this.api.get<ReportSummaryItem[]>('/reports')
  }
  get(id: string) {
    return this.api.get<ReportItem>(`/reports/${id}`)
  }
  delete(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/reports/${id}`)
  }
}
