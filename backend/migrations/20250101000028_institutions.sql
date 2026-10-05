-- Master data institusi (universitas/institut/politeknik/ISI/ISBI PTN) — melengkapi
-- ptn_programs yang selama ini hanya menyimpan data program studi + statistik penerimaan,
-- tanpa profil institusi (alamat, akreditasi, website, tahun berdiri, status kelembagaan, logo).
-- Dikeyai oleh nama_ptn (unik di ptn_programs, terverifikasi 137 institusi berbeda), bukan
-- singkatan — beberapa baris ptn_programs punya singkatan yang salah ketik/duplikat.

CREATE TABLE institutions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nama_ptn        TEXT NOT NULL UNIQUE,
    singkatan       TEXT NOT NULL,
    website         TEXT,
    alamat          TEXT,
    kota            TEXT,
    provinsi        TEXT,
    tahun_berdiri   INT,
    status          TEXT,              -- mis. "PTN-BH", "Satker", "BLU"
    akreditasi      TEXT,              -- akreditasi institusi BAN-PT/LAMDIK, mis. "Unggul", "A", "Baik Sekali"
    logo_url        TEXT,
    sumber          TEXT,              -- catatan sumber data riset (url referensi utama), untuk audit
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_institutions_nama_ptn ON institutions(nama_ptn);

-- Link nullable dari ptn_programs -> institutions. Nullable karena backfill dilakukan lewat
-- import script terpisah (match by nama_ptn), bukan lewat migrasi ini.
ALTER TABLE ptn_programs ADD COLUMN institution_id UUID REFERENCES institutions(id);
CREATE INDEX idx_ptn_programs_institution_id ON ptn_programs(institution_id);
