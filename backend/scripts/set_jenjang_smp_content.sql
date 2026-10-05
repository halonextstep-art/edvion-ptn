-- ============================================================================
-- set_jenjang_smp_content.sql
-- Set otomatis school_type_scope = 'smp' untuk semua konten TKA SMP yang
-- sudah ada — tidak perlu klik manual satu-satu di Kelola Paket. Aman
-- dijalankan berkali-kali (idempotent).
-- ============================================================================

-- PART 1: sesi tryout asli yang sudah punya soal sungguhan (Matematika &
-- Bahasa Indonesia TKA SMP) — target langsung by ID, sudah dikonfirmasi
-- sebelumnya di sesi ini.
UPDATE tryout_sessions
SET school_type_scope = 'smp'
WHERE id IN (
  '3daadc49-f0c2-4bf7-a305-5cdbd1e2c3cb',  -- Matematika
  '756ef2c0-6821-4d92-88fb-dfd31766c4c1'   -- Bahasa Indonesia
)
RETURNING id, title, school_type_scope;

-- PART 2: semua sesi tryout LAIN yang terdaftar sebagai isi paket manapun
-- bernama "TKA SMP" — jaga-jaga ada subtes lain di luar 2 di atas.
UPDATE tryout_sessions ts
SET school_type_scope = 'smp'
WHERE ts.school_type_scope IS NULL
  AND EXISTS (
    SELECT 1 FROM package_content_items pci
    JOIN packages p ON p.id = pci.package_id
    WHERE pci.content_type = 'tryout_session'
      AND pci.content_id = ts.id
      AND p.name ILIKE '%TKA SMP%'
  )
RETURNING id, title, school_type_scope;

-- PART 3: semua simulation_template yang terdaftar sebagai isi paket
-- manapun bernama "TKA SMP" (mis. "TKA SMP #1 — Hari 1 (Wajib)").
UPDATE simulation_templates st
SET school_type_scope = 'smp'
WHERE st.school_type_scope IS NULL
  AND EXISTS (
    SELECT 1 FROM package_content_items pci
    JOIN packages p ON p.id = pci.package_id
    WHERE pci.content_type = 'simulation_template'
      AND pci.content_id = st.id
      AND p.name ILIKE '%TKA SMP%'
  )
RETURNING id, title, school_type_scope;

-- Kalau ketiga PART di atas 0 baris semua — kemungkinan backend belum
-- di-restart (migration 20250101000039 belum ke-load) atau nama paketnya
-- tidak mengandung "TKA SMP" persis. Cek dulu sebelum lanjut.
