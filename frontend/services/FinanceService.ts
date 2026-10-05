import type { ApiClient } from './ApiClient'
import type { FinanceSummary, EventRevenueItem, SchoolRevenueItem } from '~/types'

export class FinanceService {
  constructor(private api: ApiClient) {}

  summary() {
    return this.api.get<FinanceSummary>('/finance/summary')
  }

  eventBreakdown() {
    return this.api.get<EventRevenueItem[]>('/finance/event-breakdown')
  }

  schoolBreakdown() {
    return this.api.get<SchoolRevenueItem[]>('/finance/school-breakdown')
  }
}
