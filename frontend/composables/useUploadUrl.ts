// Backend returns upload paths as relative URLs (e.g. `/uploads/avatars/xxx.jpg`), served
// by the Rust backend itself (not the Nuxt frontend) at its origin — distinct from
// `apiBase` which already has a trailing `/api`. This resolves a relative upload path
// into the full URL the browser can actually load the file from.
export function useUploadUrl() {
  const config = useRuntimeConfig()

  function resolve(path: string | null | undefined): string | null {
    if (!path) return null
    if (/^https?:\/\//i.test(path)) return path
    const origin = (config.public.apiBase as string).replace(/\/api\/?$/, '')
    return `${origin}${path}`
  }

  return { resolve }
}
