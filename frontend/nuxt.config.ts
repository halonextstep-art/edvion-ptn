// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-10-01',
  devtools: { enabled: true },

  modules: ['@nuxtjs/tailwindcss', '@pinia/nuxt'],

  css: ['~/assets/css/main.css'],

  // All components (ui/, questions/, student/, ...) register by filename only —
  // no directory-based prefix — so templates use <Button>, <QuestionFormDialog>, etc.
  components: [{ path: '~/components', pathPrefix: false }],

  typescript: {
    strict: true,
  },

  runtimeConfig: {
    public: {
      // Base URL of the Rust/Axum API. Override via NUXT_PUBLIC_API_BASE in .env.
      apiBase: process.env.NUXT_PUBLIC_API_BASE || 'http://localhost:8080/api',
    },
  },

  app: {
    // Fade halus antar-halaman (mis. /siswa -> /tryout/[id] -> /hasil/[id]) — bagian dari
    // polish "app-like": perpindahan rute jadi cross-fade, bukan potongan/flash mendadak
    // seperti reload halaman web biasa. Kelas .page-enter-active/.page-leave-active ada di
    // assets/css/main.css. Berlaku di seluruh portal (Admin/Sekolah/Siswa) — cosmetic-only,
    // tidak mengubah behavior apa pun.
    pageTransition: { name: 'page', mode: 'out-in' },
    head: {
      title: 'EdvionPTN — Sistem Persiapan SNBT',
      meta: [
        { name: 'description', content: 'Platform Drilling Tryout & Rasionalisasi SNBT berbasis AI.' },
        // `viewport-fit=cover` mengaktifkan env(safe-area-inset-*) supaya header/bottom-nav
        // bisa menghindari notch/home-indicator saat portal siswa dibuka sebagai PWA
        // terinstall (display: standalone) — tanpa ini env(safe-area-inset-*) selalu 0.
        { name: 'viewport', content: 'width=device-width, initial-scale=1, viewport-fit=cover' },
        { name: 'theme-color', content: '#4f46e5' },
        // iOS tidak membaca manifest.webmanifest untuk perilaku "Add to Home Screen" —
        // butuh meta Apple-nya sendiri supaya terbuka standalone (tanpa address bar Safari).
        { name: 'apple-mobile-web-app-capable', content: 'yes' },
        { name: 'apple-mobile-web-app-status-bar-style', content: 'black-translucent' },
        { name: 'apple-mobile-web-app-title', content: 'EdvionPTN' },
        { name: 'mobile-web-app-capable', content: 'yes' },
      ],
      link: [
        { rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' },
        { rel: 'manifest', href: '/manifest.webmanifest' },
        { rel: 'apple-touch-icon', href: '/pwa/apple-touch-icon.png' },
        {
          rel: 'stylesheet',
          href: 'https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:ital,wght@0,300;0,400;0,500;0,600;0,700;0,800;1,400&family=Outfit:wght@300;400;500;600;700;800;900&family=JetBrains+Mono:wght@400;500;600&display=swap',
        },
        // KaTeX stylesheet for rendering LaTeX equations in question content
        // (stimulus/question_text/options/explanation) — see composables/useQuestionMarkdown.ts.
        {
          rel: 'stylesheet',
          href: 'https://cdnjs.cloudflare.com/ajax/libs/KaTeX/0.16.11/katex.min.css',
        },
      ],
    },
  },
})
