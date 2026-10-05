// Mendaftarkan public/sw.js (service worker minimal, lihat komentar di file itu) — dibuat
// sebagai plugin `.client.ts` terpisah (bukan inline di app.vue) supaya Nuxt otomatis skip di
// SSR (registrasi service worker tidak punya arti di server) tanpa perlu guard `process.client`
// manual, sama pola dengan chartjs.client.ts.
export default defineNuxtPlugin(() => {
  if (!('serviceWorker' in navigator)) return
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/sw.js').catch(() => {
      // Non-kritis — seluruh aplikasi tetap berfungsi penuh tanpa service worker ini; yang
      // hilang cuma kemampuan "Add to Home Screen" di sebagian browser, bukan fitur inti.
    })
  })
})
