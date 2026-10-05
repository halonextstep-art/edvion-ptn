-- ============================================================================
-- cleanup_empty_tka_smp_packages.sql
-- Read-only dulu (PART 1) baru hapus (PART 2) — jangan jalankan PART 2 sebelum
-- baca hasil PART 1 dan yakin baris yang mau dihapus memang aman.
--
-- Menghapus paket "TKA SMP" yang KOSONG (tidak ada content item) DAN tidak
-- punya entitlement sekolah manapun — sisa dari percobaan wizard berulang.
-- schools_package_entitlements.package_id punya ON DELETE RESTRICT, jadi
-- DELETE otomatis GAGAL (bukan diam-diam menghapus data penting) kalau
-- ternyata ada sekolah yang masih terhubung ke paket itu.
-- ============================================================================

-- ── PART 1: paket "TKA SMP" mana saja yang kosong (tidak ada isi & tidak
-- ada entitlement) — aman dihapus.
SELECT p.id, p.name, p.exam_track, p.created_at,
       (SELECT count(*) FROM package_content_items pci WHERE pci.package_id = p.id) AS jumlah_isi,
       (SELECT count(*) FROM school_package_entitlements e WHERE e.package_id = p.id) AS jumlah_entitlement
FROM packages p
WHERE p.name ILIKE '%TKA SMP%'
ORDER BY p.created_at;

-- Baca hasilnya: paket dengan jumlah_isi = 0 DAN jumlah_entitlement = 0 itu
-- aman dihapus (kosong total, tidak dipakai siapapun). JANGAN hapus paket
-- yang jumlah_isi > 0 atau jumlah_entitlement > 0.

-- ── PART 2: hapus paket yang benar-benar kosong (jalankan setelah cek PART 1)
DELETE FROM packages p
WHERE p.name ILIKE '%TKA SMP%'
  AND NOT EXISTS (SELECT 1 FROM package_content_items pci WHERE pci.package_id = p.id)
  AND NOT EXISTS (SELECT 1 FROM school_package_entitlements e WHERE e.package_id = p.id)
RETURNING id, name;
