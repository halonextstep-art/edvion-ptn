-- "Jurusan yang Diminati" — a real, student-filled major/field-of-interest signal,
-- distinct from `ptn_programs.rumpun` (which is a catalog attribute of a PROGRAM, not a
-- personal declaration by the STUDENT). Previously this app had no such signal, so the
-- "Alternatif Sesuai Minat Jurusan" resume category was deliberately left unbuilt (see
-- RasionalisasiSnbpEditor.vue) rather than faking it. Purely additive/nullable — no
-- backfill needed since it's a brand-new personal field with no prior data anywhere.
ALTER TABLE users ADD COLUMN minat_jurusan TEXT;
