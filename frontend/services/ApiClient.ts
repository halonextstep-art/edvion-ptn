// Thin OOP wrapper around Nuxt's `$fetch`. Every domain service (AuthService,
// QuestionService, TryoutService) is constructed with one of these instead of calling
// `$fetch` directly — centralizes base URL, auth header injection, and error shaping.

export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

export type TokenGetter = () => string | null

export class ApiClient {
  constructor(
    private baseURL: string,
    private getToken: TokenGetter,
  ) {}

  private headers(): Record<string, string> {
    const token = this.getToken()
    const headers: Record<string, string> = { 'Content-Type': 'application/json' }
    if (token) headers.Authorization = `Bearer ${token}`
    return headers
  }

  // 25s timeout on every request — without this, a request the server never responds to (a
  // hung/slow endpoint, a dropped connection, etc.) leaves whatever page called it stuck on
  // its loading spinner FOREVER with no error and no way out, since nothing here ever settles
  // the promise otherwise. ofetch (Nuxt's $fetch) aborts and throws once `timeout` elapses, so
  // the existing catch block below turns it into a normal, recoverable `ApiError` instead.
  private static readonly TIMEOUT_MS = 25_000

  private async request<T>(method: string, path: string, body?: unknown, query?: Record<string, unknown>): Promise<T> {
    try {
      return await $fetch<T>(path, {
        method: method as any,
        baseURL: this.baseURL,
        headers: this.headers(),
        body: body as any,
        query,
        timeout: ApiClient.TIMEOUT_MS,
      })
    } catch (err: any) {
      const status = err?.response?.status ?? err?.statusCode ?? 0
      const isTimeout = err?.name === 'AbortError' || /timeout/i.test(err?.message ?? '')
      const message = isTimeout
        ? 'Server tidak merespons — coba lagi dalam beberapa saat'
        : (err?.data?.error ?? err?.message ?? 'Terjadi kesalahan tidak dikenal')
      throw new ApiError(message, status)
    }
  }

  get<T>(path: string, query?: Record<string, unknown>) {
    return this.request<T>('GET', path, undefined, query)
  }
  post<T>(path: string, body?: unknown) {
    return this.request<T>('POST', path, body)
  }
  put<T>(path: string, body?: unknown) {
    return this.request<T>('PUT', path, body)
  }
  patch<T>(path: string, body?: unknown) {
    return this.request<T>('PATCH', path, body)
  }
  delete<T>(path: string) {
    return this.request<T>('DELETE', path)
  }

  /** Multipart file upload (avatar, achievement certificate, etc.) — deliberately bypasses
   * `request()`'s JSON `Content-Type` header so the browser can set the correct
   * `multipart/form-data; boundary=...` one itself. */
  async upload<T>(path: string, formData: FormData): Promise<T> {
    const token = this.getToken()
    const headers: Record<string, string> = {}
    if (token) headers.Authorization = `Bearer ${token}`
    try {
      // Longer timeout than request() — uploads (avatar, achievement certificate) legitimately
      // take longer than a JSON round-trip, but still shouldn't hang forever.
      return await $fetch<T>(path, { method: 'POST', baseURL: this.baseURL, headers, body: formData, timeout: 60_000 })
    } catch (err: any) {
      const status = err?.response?.status ?? err?.statusCode ?? 0
      const message = err?.data?.error ?? err?.message ?? 'Terjadi kesalahan tidak dikenal'
      throw new ApiError(message, status)
    }
  }
}
