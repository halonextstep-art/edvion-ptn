import type { ApiClient } from './ApiClient'
import type { NotificationItem } from '~/types'

/** Real per-user notification feed — see backend `application::notification_service` doc
 * comment. Self-scoped: every method operates on the calling user's own notifications. */
export class NotificationService {
  constructor(private api: ApiClient) {}

  listMine(limit = 20) {
    return this.api.get<NotificationItem[]>('/notifications', { limit })
  }

  unreadCount() {
    return this.api.get<{ count: number }>('/notifications/unread-count')
  }

  markRead(id: string) {
    return this.api.post<{ ok: boolean }>(`/notifications/${id}/read`)
  }

  markAllRead() {
    return this.api.post<{ ok: boolean }>('/notifications/read-all')
  }
}
