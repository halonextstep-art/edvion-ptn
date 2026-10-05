-- Local-disk file uploads (avatar foto profil + lampiran sertifikat prestasi). Files are
-- saved to a local `uploads/` directory on the backend server (see
-- `infrastructure::storage::local_storage`) and served back via a static `/uploads/*`
-- route — no cloud storage dependency, sufficient for current scale and easy to swap for
-- S3-compatible storage later without touching these columns (they just hold a URL path).
ALTER TABLE users ADD COLUMN avatar_url TEXT;
ALTER TABLE student_achievements ADD COLUMN certificate_url TEXT;
