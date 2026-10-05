-- ============================================================================
-- diagnose_premium_access.sql
--
-- Read-only — tidak mengubah data apa pun. Untuk melacak kenapa siswa masih
-- lihat "Buka Premium" (terkunci) padahal paket sudah di-assign ke sekolahnya.
--
-- Cara kerja lock/unlock (lihat backend AccessService::has_access_to_content):
--   Siswa bisa buka TryoutSession X kalau DUA syarat ini SAMA-SAMA benar:
--   1. TryoutSession X terdaftar sebagai isi salah satu Package (package_content_items)
--   2. Sekolah siswa itu punya entitlement AKTIF (starts_at <= hari ini <= expires_at,
--      atau expires_at NULL = tanpa batas) ke Package YANG SAMA
--   Kalau salah satu saja tidak terpenuhi, kunci tetap muncul — jadi cek dua-duanya.
--
-- GANTI dulu 2 nilai di bawah sebelum menjalankan (baris "\set" di psql tidak
-- dipakai di sini supaya bisa langsung jalan di GUI DB apa pun — cukup
-- edit teks '%SMP Al Masih%' dan email siswa di PART 3):
-- ============================================================================

-- ── PART 1: Sekolah & entitlement paketnya ──────────────────────────────────
-- Ganti '%SMP Al Masih%' sesuai nama sekolah Anda.
SELECT
  s.id                              AS school_id,
  s.name                            AS school_name,
  p.id                              AS package_id,
  p.name                            AS package_name,
  e.starts_at,
  e.expires_at,
  CASE
    WHEN e.starts_at > CURRENT_DATE THEN 'BELUM AKTIF (starts_at di masa depan)'
    WHEN e.expires_at IS NOT NULL AND e.expires_at < CURRENT_DATE THEN 'SUDAH KEDALUWARSA'
    ELSE 'AKTIF'
  END AS status_entitlement
FROM schools s
LEFT JOIN school_package_entitlements e ON e.school_id = s.id
LEFT JOIN packages p ON p.id = e.package_id
WHERE s.name ILIKE '%SMP Al Masih%'
ORDER BY e.created_at DESC;

-- Kalau hasil di atas KOSONG (tidak ada baris sama sekali untuk sekolah ini) —
-- berarti entitlement-nya belum ke-assign sama sekali, atau ke-assign ke baris
-- sekolah yang berbeda (cek juga apakah ada 2 sekolah dengan nama mirip).


-- ── PART 2: Isi (content) dari paket yang di-assign di atas ────────────────
-- Ganti '<PACKAGE_ID>' dengan package_id dari hasil PART 1.
SELECT
  pci.content_type,
  pci.content_id,
  ts.title AS tryout_session_title,
  st.title AS simulation_template_title
FROM package_content_items pci
LEFT JOIN tryout_sessions ts ON pci.content_type = 'tryout_session' AND ts.id = pci.content_id
LEFT JOIN simulation_templates st ON pci.content_type = 'simulation_template' AND st.id = pci.content_id
WHERE pci.package_id = '<PACKAGE_ID>'::uuid;

-- Kalau tabel "TKA SMP - Matematika (Wajib)" / "TKA SMP - Bahasa Indonesia (Wajib)"
-- TIDAK muncul di hasil ini — itu penyebabnya: sesi tryout-nya belum terdaftar
-- sebagai isi paket ini, meski paketnya sendiri sudah di-assign ke sekolah.


-- ── PART 3: Sekolah siswa yang login benar-benar yang mana? ────────────────
-- Ganti email di bawah dengan email siswa yang login (Angelica Evelyn di screenshot).
SELECT u.id AS user_id, u.name, u.email, u.school_id, s.name AS school_name_siswa
FROM users u
LEFT JOIN schools s ON s.id = u.school_id
WHERE u.email = 'GANTI-EMAIL-SISWA@contoh.com';

-- Cocokkan school_id di sini dengan school_id di PART 1 — kalau BEDA (mis. siswa
-- ke-link ke baris sekolah duplikat yang berbeda dari yang di-assign paketnya),
-- itu penyebabnya.
