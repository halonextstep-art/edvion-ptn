-- ============================================================================
-- diagnose_package_mismatch.sql
-- Read-only. Cross-check: paket mana yang aktual berisi sesi Matematika/Bahasa
-- Indonesia, vs paket mana yang aktual di-assign (entitlement aktif) ke
-- sekolah "SMP Almasih".
-- ============================================================================

-- PART A: Entitlement aktif sekolah Almasih — paket_id mana yang SEBENARNYA
-- di-assign (nama sekolah dibetulkan, tanpa spasi di "Almasih").
SELECT s.id AS school_id, s.name AS school_name,
       e.package_id, p.name AS package_name,
       e.starts_at, e.expires_at,
       CASE
         WHEN e.starts_at > CURRENT_DATE THEN 'BELUM AKTIF'
         WHEN e.expires_at IS NOT NULL AND e.expires_at < CURRENT_DATE THEN 'KEDALUWARSA'
         ELSE 'AKTIF'
       END AS status
FROM schools s
LEFT JOIN school_package_entitlements e ON e.school_id = s.id
LEFT JOIN packages p ON p.id = e.package_id
WHERE s.name ILIKE '%Almasih%'
ORDER BY e.created_at DESC;

-- PART B: Dari 3 paket "TKA SMP" yang ada, MANA yang benar-benar berisi
-- sesi Matematika (3daadc49-f0c2-4bf7-a305-5cdbd1e2c3cb) dan
-- Bahasa Indonesia (756ef2c0-6821-4d92-88fb-dfd31766c4c1) sebagai content item.
SELECT pci.package_id, p.name AS package_name, ts.title AS session_title, pci.content_id
FROM package_content_items pci
JOIN packages p ON p.id = pci.package_id
JOIN tryout_sessions ts ON ts.id = pci.content_id
WHERE pci.package_id IN ('08d25fcf-df42-0000-0000-000000000000', '08bff647-7f60-0000-0000-000000000000', '9273fa73-3b00-0000-0000-000000000000')
  AND pci.content_id IN ('3daadc49-f0c2-4bf7-a305-5cdbd1e2c3cb', '756ef2c0-6821-4d92-88fb-dfd31766c4c1');

-- CATATAN: id 3 paket di atas saya potong dari screenshot (kolom package_id
-- ke-cut di tampilan tabel). Kalau query PART B error "invalid input syntax
-- for type uuid" karena ID tidak lengkap, jalankan versi ini sebagai gantinya
-- (tidak butuh ID paket sama sekali, otomatis cek ke semua paket):

-- PART B ALTERNATIF (pakai ini saja, lebih aman):
SELECT pci.package_id, p.name AS package_name, ts.title AS session_title
FROM package_content_items pci
JOIN packages p ON p.id = pci.package_id
JOIN tryout_sessions ts ON ts.id = pci.content_id
WHERE pci.content_id IN ('3daadc49-f0c2-4bf7-a305-5cdbd1e2c3cb', '756ef2c0-6821-4d92-88fb-dfd31766c4c1')
ORDER BY p.name;
