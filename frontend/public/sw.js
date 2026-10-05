// Service worker minimal — sengaja TIDAK memakai module PWA Nuxt (@vite-pwa/nuxt dkk) karena
// butuh npm install dependency baru yang tidak reliable di sandbox pengembangan proyek ini
// (lihat catatan konstrain build di memori proyek). Ditulis manual, murni file statis, tanpa
// dependency tambahan apa pun.
//
// Tujuannya CUMA memenuhi syarat "installable" browser (Chrome/Edge mensyaratkan service
// worker terdaftar dengan fetch handler + manifest valid supaya prompt "Add to Home Screen"
// muncul) — BUKAN untuk membuat aplikasi bisa dipakai offline. Karena itu strategi cache-nya
// sangat konservatif: cuma aset yang BENAR-BENAR statis (ikon, manifest, favicon) yang di-cache
// cache-first. Semua halaman, bundle JS/CSS, dan panggilan API dibiarkan lewat apa adanya
// (network normal) supaya deploy baru selalu langsung terpakai — tidak ada risiko siswa
// "tersangkut" di versi lama aplikasi karena keliru men-cache bundle yang nama filenya
// berubah setiap build.
const CACHE_NAME = 'edvionptn-static-v1'
const STATIC_ASSETS = [
  '/pwa/icon-192.png',
  '/pwa/icon-512.png',
  '/pwa/icon-maskable-512.png',
  '/pwa/apple-touch-icon.png',
  '/favicon.ico',
  '/manifest.webmanifest',
]

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(STATIC_ASSETS)).catch(() => {
      // Non-fatal — kalau salah satu aset gagal di-precache, service worker tetap terpasang
      // (memenuhi syarat installability) dan sisanya tetap jalan lewat fetch handler di bawah.
    }),
  )
  self.skipWaiting()
})

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) => Promise.all(keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k)))),
  )
  self.clients.claim()
})

self.addEventListener('fetch', (event) => {
  const req = event.request
  if (req.method !== 'GET') return
  const url = new URL(req.url)
  if (url.origin !== self.location.origin) return
  if (!STATIC_ASSETS.includes(url.pathname)) return // segala sesuatu selain daftar di atas: biarkan lewat network normal

  event.respondWith(
    caches.match(req).then((cached) => cached || fetch(req)),
  )
})
