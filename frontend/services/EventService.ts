import type { ApiClient } from './ApiClient'
import type { EventItem, EventPayload, EventStatus } from '~/types'

export class EventService {
  constructor(private api: ApiClient) {}

  list() {
    return this.api.get<EventItem[]>('/events')
  }

  /** Student/school-portal "Event Mendatang" widget — upcoming/ongoing only, sorted, capped. */
  upcoming(limit = 5) {
    return this.api.get<EventItem[]>('/events/upcoming', { limit })
  }

  get(id: string) {
    return this.api.get<EventItem>(`/events/${id}`)
  }

  create(payload: EventPayload) {
    return this.api.post<EventItem>('/events', payload)
  }

  update(id: string, payload: EventPayload) {
    return this.api.put<EventItem>(`/events/${id}`, payload)
  }

  setStatus(id: string, status: EventStatus) {
    return this.api.patch<EventItem>(`/events/${id}/status`, { status })
  }

  remove(id: string) {
    return this.api.delete<{ deleted: boolean }>(`/events/${id}`)
  }
}
