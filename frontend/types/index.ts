// Mirrors the DTOs exposed by the Rust backend (interfaces/http/dto/*).
// Kept hand-in-sync deliberately (no codegen) so the contract stays easy to read.

export type Role = 'admin' | 'school' | 'student' | 'content'
export type UserStatus = 'active' | 'inactive' | 'pending'

export interface User {
  id: string
  name: string
  email: string
  role: Role
  school_id?: string | null
  school_name?: string | null
  status: UserStatus
  last_login?: string | null
  phone?: string | null
  /** Student-only. */
  nisn?: string | null
  /** Student-only, e.g. "Kelas 12". */
  grade?: string | null
  /** Student-only, nomor induk sekolah (beda dari nisn). Create-only, lihat CreateUserPayload. */
  nis?: string | null
  gender?: string | null
  birth_date?: string | null
  enrolled_at?: string | null
  rombel_code?: string | null
  username?: string | null
  /** URL path (e.g. `/uploads/avatars/xxx.jpg`) to a locally-stored profile photo. */
  avatar_url?: string | null
  /** Self-declared "Jurusan yang Diminati" — real, student-typed field of interest, distinct
   * from `PtnProgramItem.rumpun`. Only the student sets this (self-service profile update). */
  minat_jurusan?: string | null
  created_at: string
}

export interface AuthResponse {
  token: string
  user: User
}

// ─── Profil sendiri (semua role) — lihat backend auth_service::update_profile /
// change_password. Sengaja tidak termasuk role/status/school_id/email/nisn/grade — itu
// dikelola admin/sekolah, bukan self-service. ──────────────────────────────────────
export interface UpdateProfilePayload {
  name: string
  phone?: string | null
  /** "Jurusan yang Diminati" — like `phone`, the backend overwrites this column
   * unconditionally on every save, so every caller of `updateProfile()` must always resend
   * the current value for whichever of these optional fields it isn't actively changing
   * (see `ProfileDialog.vue` and `RasionalisasiSnbpEditor.vue`). */
  minat_jurusan?: string | null
}

export interface ChangePasswordPayload {
  current_password: string
  new_password: string
}

export type QuestionType =
  | 'multiple_choice'
  | 'complex_multiple'
  | 'short_answer'
  | 'true_false'
  | 'matching'
  | 'essay'
  | 'true_false_complex'
  | 'matching_image'
  | 'ordering'
export type Difficulty = 'easy' | 'medium' | 'hard'
export type QuestionStatus = 'draft' | 'review' | 'approved' | 'rejected' | 'revision'

export interface Question {
  id: string
  code: string
  question_type: QuestionType
  subject: string
  topic: string
  subtopic: string
  difficulty: Difficulty
  bloom_level: string
  stimulus?: string | null
  question_text: string
  options?: string[] | null
  correct_answer: string
  explanation: string
  tags: string[]
  status: QuestionStatus
  review_note?: string | null
  reviewed_at?: string | null
  created_by: string
  created_by_name: string
  usage_count: number
  average_score: number
  time_limit?: number | null
  created_at: string
  updated_at: string
}

// ─── Bulk "Import Soal" .docx (see backend application::question_import_service) ──────────

export interface ImportBatchMeta {
  subject: string
  topic?: string
  subtopic?: string
  bloom_level?: string
  time_limit?: number | null
  submit_for_review: boolean
  question_set_id?: string
}

export interface ImportIssue {
  question_index: number
  no_soal: string | null
  message: string
}

export interface PreviewedQuestion {
  question_index: number
  no_soal: string
  question_type: string
  question_text: string
  difficulty: string
  explanation: string
  options: string[] | null
  correct_answer: string
  stimulus: string | null
}

export interface ImportPreviewResponse {
  questions: PreviewedQuestion[]
  issues: ImportIssue[]
}

export interface ImportCommitResponse {
  created: Question[]
  issues: ImportIssue[]
}

export interface QuestionPayload {
  question_type: QuestionType
  subject: string
  topic: string
  subtopic: string
  difficulty: Difficulty
  bloom_level: string
  stimulus?: string | null
  question_text: string
  options?: string[] | null
  correct_answer: string
  explanation: string
  tags: string[]
  time_limit?: number | null
  submit_for_review?: boolean
  /** Optional "Set Soal" to attach this question to as part of the same save — see backend
   *  `QuestionPayload::question_set_id` doc comment. `null`/omitted = not attached to a set. */
  question_set_id?: string | null
}

export interface QuestionListResponse {
  items: Question[]
  total: number
  page: number
  page_size: number
}

export type SessionType = 'tryout' | 'drilling' | 'mini'

/** Which national exam track this content is built to prepare for. Formalizes what used to be
 *  an implicit heuristic (`elective_pick_count > 0` ⇒ "treat as TKA") into an explicit field on
 *  Package/TryoutSessionTemplate/SimulationTemplateItem. `snbt` = UTBK/SNBT prep, `tka` = TKA
 *  (mapel wajib + mapel pilihan) prep. */
export type ExamTrack = 'snbt' | 'tka'

export interface TryoutSessionTemplate {
  id: string
  title: string
  session_type: SessionType
  duration_minutes: number
  question_count: number
  subject_filter?: string | null
  topic_filter?: string | null
  difficulty_filter?: string | null
  is_premium: boolean
  created_at: string
  /** Non-null when the session is pinned to a curated "Set Soal" instead of a random draw
   *  from the bank soal filtered by subject/topic/difficulty — see SessionFormDialog.vue. */
  question_set_id?: string | null
  /** `true` = still being assembled (e.g. via the "Buat Paket + Subtes" wizard), hidden from
   *  the student/school catalogue until the owning Package is published. Never sent to
   *  Student/School accounts as `true` — backend filters those rows out entirely. */
  is_draft: boolean
  /** `true` = this session is one of its owning Package's "mapel pilihan" (elective) options
   *  rather than always-available — e.g. TKA's Fisika/Kimia/Ekonomi subtests, as opposed to its
   *  mandatory Bahasa Indonesia/Matematika/Bahasa Inggris ones. Only meaningful when the owning
   *  Package's `elective_pick_count > 0` — see PackageItem.elective_pick_count and
   *  `/mapel-pilihan` (student "Pilih Mapel Pilihan" panel). */
  is_elective: boolean
  /** See ExamTrack doc comment. */
  exam_track: ExamTrack
  /** `null`/`undefined` (default) = tampil ke siswa jenjang apapun. Set = katalog siswa
   *  (DrillingZone) hanya menampilkan sesi ini ke siswa yang sekolahnya berjenjang sama;
   *  siswa B2C tanpa sekolah tetap melihatnya (jenjangnya tidak bisa ditentukan). Filter
   *  sebenarnya berjalan di backend (`TryoutService::list_sessions`) — field ini di sini
   *  murni supaya form admin bisa menampilkan/mengubah nilainya. */
  school_type_scope?: SchoolType | null
}

export interface PlayableQuestion {
  id: string
  subject: string
  topic: string
  question_type: QuestionType
  difficulty: Difficulty
  stimulus?: string | null
  question_text: string
  options?: string[] | null
}

export type AttemptStatus = 'in_progress' | 'submitted'

export interface Attempt {
  id: string
  session_id: string
  session_title: string
  session_type: SessionType
  status: AttemptStatus
  duration_minutes: number
  started_at: string
  submitted_at?: string | null
  score?: number | null
  accuracy?: number | null
  correct_count?: number | null
  wrong_count?: number | null
  unanswered_count?: number | null
  time_used_seconds?: number | null
}

export interface AttemptWithQuestions {
  attempt: Attempt
  questions: PlayableQuestion[]
}

export interface SavedAnswer {
  question_id: string
  answer_text?: string | null
  flagged: boolean
}

/** Response of `GET /tryout/attempts/:id/resume` — used to rebuild the player page's state
 *  after a refresh/reopen (attemptSession store is in-memory only, see its doc comment). */
export interface AttemptResumeResponse {
  attempt: Attempt
  questions: PlayableQuestion[]
  answers: SavedAnswer[]
}

export interface SubjectBreakdown {
  subject: string
  correct: number
  total: number
}

export interface AttemptResult {
  attempt: Attempt
  subject_breakdown: SubjectBreakdown[]
  /** "Estimasi IRT" — see backend `domain::irt` doc comment. `null` when too few of this
   *  attempt's questions currently have a trusted item calibration ("belum tersedia"). */
  irt_score: number | null
  /** Which score(s) to actually display — set by the admin, see `ScoreDisplayMode`. */
  score_display_mode: ScoreDisplayMode
}

export interface ReviewItem {
  question_id: string
  code: string
  subject: string
  question_type: QuestionType
  stimulus?: string | null
  question_text: string
  options?: string[] | null
  correct_answer: string
  explanation: string
  user_answer?: string | null
  is_correct?: boolean | null
  flagged: boolean
}

// ─── Schools (Sekolah Mitra) ───────────────────────────────────────────────────
export type SchoolType = 'sma' | 'smk' | 'ma' | 'smp'
export type PackageType = 'basic' | 'premium' | 'enterprise'
export type SchoolStatus = 'active' | 'inactive' | 'pending'

export interface School {
  id: string
  name: string
  school_type: SchoolType
  city: string
  province: string
  email: string
  phone: string
  join_date: string
  package_type: PackageType
  contact_person: string
  status: SchoolStatus
  /** Percentage (0-50) of the contracted plan value paid back to the school. */
  revenue_share: number
  /** Contracted plan value in Rupiah, admin-recorded (no billing system yet). */
  monthly_revenue: number
  created_at: string
}

export interface SchoolPayload {
  name: string
  school_type: SchoolType
  city: string
  province: string
  email: string
  phone: string
  package_type: PackageType
  contact_person: string
  status?: SchoolStatus
  revenue_share?: number
  monthly_revenue?: number
}

export const PACKAGE_QUOTA: Record<PackageType, number> = { basic: 150, premium: 300, enterprise: 9999 }

// School-portal-only aggregate — computed live from the school's own real students'
// attempts/targets (see backend SchoolPortalService), never stored or fabricated.
export interface PtnDistributionItem {
  nama_ptn: string
  count: number
}

export interface StudentRosterStat {
  student_id: string
  best_score?: number | null
  avg_score?: number | null
  tryouts_completed: number
  target_ptn_count: number
  last_attempt_at?: string | null
}

export interface SchoolOverview {
  total_students: number
  avg_score: number | null
  tryouts_completed: number
  target_ptn_count: number
}

// School Overview "Aktivitas Terkini" — real submitted attempt dari salah satu siswa
// sekolah ini, terbaru dulu (lihat SchoolPortalService::recent_activity).
export interface RecentActivityItem {
  student_name: string
  session_title: string
  session_type: 'tryout' | 'drilling' | 'mini'
  score: number | null
  submitted_at: string
}

// ─── Users (admin-managed accounts) ────────────────────────────────────────────
export interface CreateUserPayload {
  name: string
  email: string
  password: string
  role: Role
  school_id?: string | null
  status?: UserStatus
  phone?: string | null
  nisn?: string | null
  grade?: string | null
  /** Student-only, dari format import Excel mitra sekolah. Create-only — sengaja tidak ada
   * di UpdateUserPayload supaya form "Edit User" biasa tidak bisa menghapus nilai ini. */
  nis?: string | null
  gender?: string | null
  birth_date?: string | null
  enrolled_at?: string | null
  rombel_code?: string | null
  username?: string | null
}

export interface UpdateUserPayload {
  name: string
  email: string
  role: Role
  school_id?: string | null
  status?: UserStatus
  phone?: string | null
  nisn?: string | null
  grade?: string | null
  /** Non-empty only when resetting the password. */
  password?: string
}

// ─── Analytics (read-only, aggregated from real data) ──────────────────────────
export interface AnalyticsSummary {
  total_students: number
  total_schools: number
  total_questions: number
  approved_questions: number
  total_attempts: number
  submitted_attempts: number
  average_score: number
}

export interface ScoreTrendPoint {
  day: string
  attempts: number
  average_score: number
}

/** "Perbandingan Tahun" — attempts dikelompokkan per tahun kalender `submitted_at`
 * (bukan angkatan/kohort siswa, karena belum ada field kohort yang andal di sistem). */
export interface YearlyPerformancePoint {
  year: number
  attempts: number
  distinct_students: number
  average_score: number
}

export interface SubjectAccuracy {
  subject: string
  correct: number
  total: number
}

export interface QuestionStatusCount {
  status: string
  count: number
}

export interface SchoolRanking {
  school_id: string
  school_name: string
  active_students: number
  average_score: number
}

export interface StudentRanking {
  student_id: string
  student_name: string
  attempts_count: number
  best_score: number
  average_score: number
}

/** School portal's "Analytics & Insights" drill-down — see backend
 *  `domain::analytics::StudentActivity` doc comment for why this INCLUDES students with zero
 *  submitted attempts (unlike `StudentRanking` above), and why `best_score`/`average_score`/
 *  `last_attempt_at` are `null` rather than `0` for those students. */
export interface StudentActivityItem {
  student_id: string
  student_name: string
  rombel_code: string | null
  attempts_count: number
  best_score: number | null
  average_score: number | null
  last_attempt_at: string | null
}

// ─── Events (Manajemen Event) ───────────────────────────────────────────────────
export type EventType = 'tryout' | 'drilling' | 'mini'
export type EventStatus = 'draft' | 'upcoming' | 'ongoing' | 'completed' | 'archived'

export interface EventItem {
  id: string
  name: string
  event_type: EventType
  description: string
  session_id: string
  session_title: string
  duration_minutes: number
  start_date: string
  end_date?: string | null
  max_participants?: number | null
  /** Real distinct-student count derived from attempts — never hardcoded. */
  participants: number
  price: number
  status: EventStatus
  prizes: string
  target_class: string
  created_at: string
}

export interface EventPayload {
  name: string
  event_type: EventType
  description: string
  session_id: string
  start_date: string
  end_date?: string | null
  max_participants?: number | null
  price: number
  prizes: string
  target_class: string
  status?: EventStatus
}

// ─── Kalender Deadline SNBP/SNBT/UTBK ───────────────────────────────────────────
export type AdmissionTrack = 'snbp' | 'snbt' | 'utbk'

export interface AdmissionDeadlineItem {
  id: string
  track: AdmissionTrack
  year: number
  label: string
  description: string
  deadline_date: string
  /** Hari tersisa sampai deadline (negatif = sudah lewat), dihitung live oleh backend. */
  days_remaining: number
  created_at: string
  updated_at: string
}

export interface AdmissionDeadlinePayload {
  track: AdmissionTrack
  year: number
  label: string
  description: string
  deadline_date: string
}

/** Shared by Badge and Package: "preset" (a curated lucide-vue-next icon, named in
 * icon_name) or "custom" (an admin-uploaded image, at icon_url). See
 * components/shared/IconPicker.vue. */
export type IconType = 'preset' | 'custom'

// ─── Packages (Manajemen Paket) ──────────────────────────────────────────────────
export interface PackageItem {
  id: string
  name: string
  package_type: string
  original_price: number
  sale_price: number
  /** Always derived from original_price/sale_price on the backend, never sent by the client. */
  discount: number
  validity: string
  features: string[]
  badge: string
  /** Legacy raw-emoji field — superseded by icon_type/icon_name/icon_url, no longer
   * rendered anywhere. Kept only because the backend still returns it. */
  emoji: string
  icon_type: IconType
  icon_name: string | null
  icon_url: string | null
  /** Optional wide promotional image at the top of the package card, distinct from the small
   * square icon above. `null` (default) means the card keeps rendering its gradient + icon
   * exactly as before — purely additive. See components/shared/BannerPicker.vue. */
  banner_url: string | null
  gradient: string
  accent_color: string
  active: boolean
  sort_order: number
  /** `0` = this package imposes no "mapel pilihan" restriction — every session it contains is
   *  freely startable. A positive number is the exact count of `is_elective` sessions in this
   *  package a student must choose before any of them become startable (see ElectivesResponse /
   *  "Pilih Mapel Pilihan"). */
  elective_pick_count: number
  /** See ExamTrack doc comment. */
  exam_track: ExamTrack
  /** Always 0 — no purchase/checkout system exists yet, so there is no real sales figure. */
  sold_count: number
  /** What this package actually unlocks — e.g. "Simulasi UTBK Batch 1 (Simulasi TO)", "Sesi
   * Matematika Dasar (Sesi Tryout)" — derived live from `package_content_items`, distinct from
   * `features` (a free-text marketing bullet list the admin types by hand). Empty means the
   * package has no content assigned yet and won't unlock anything for whoever owns it. */
  content_titles: string[]
  created_at: string
  updated_at: string
}

export interface PackagePayload {
  name: string
  package_type: string
  original_price: number
  sale_price: number
  validity: string
  features: string[]
  badge: string
  icon_type: IconType
  icon_name: string | null
  icon_url: string | null
  banner_url?: string | null
  gradient: string
  accent_color: string
  active: boolean
  sort_order: number
  /** See PackageItem.elective_pick_count doc comment. */
  elective_pick_count?: number
  /** See ExamTrack doc comment. Defaults to 'snbt' server-side if omitted. */
  exam_track?: ExamTrack
}

// ─── School Package Entitlement (admin menyematkan paket ke sekolah — inheritance
// nyata untuk siswa sekolah tsb, dicek server-side saat start_attempt sesi premium) ────
export interface SchoolEntitlementItem {
  id: string
  school_id: string
  school_name: string
  package_id: string
  package_name: string
  starts_at: string
  expires_at: string | null
  is_active: boolean
  note: string | null
  granted_by: string
  granted_by_name: string
  created_at: string
}

export interface SchoolEntitlementPayload {
  school_id: string
  package_id: string
  starts_at?: string
  expires_at?: string | null
  note?: string | null
}

// ─── Taxonomy (Kategori & Mata Uji: SNBT / TKA-IPA / TKA-IPS / AKM) ──────────────
export interface SubjectItem {
  id: string
  category_id: string
  name: string
  code: string
  sort_order: number
  created_at: string
  updated_at: string
}

export interface SubjectCategoryWithSubjects {
  id: string
  name: string
  description: string
  sort_order: number
  created_at: string
  updated_at: string
  subjects: SubjectItem[]
}

// ─── Set Soal (deterministic, curated question sets) ────────────────────────────
// A fixed, ordered list of questions an admin/content author curates once, then attaches
// to a tryout/drilling session so every student who takes that session sees the exact same
// questions in the exact same order — instead of the usual random draw from the bank soal
// filtered by subject/topic/difficulty. See backend domain::question_set doc comment.
export interface QuestionSetItem {
  id: string
  name: string
  description: string
  created_by: string
  created_at: string
  updated_at: string
  /** Real, live count of member questions — never fabricated. */
  item_count: number
}

// ─── Vouchers (Manajemen Voucher) ────────────────────────────────────────────────
export type VoucherType = 'access' | 'bulk_school' | 'promo' | 'referral'
export type DiscountType = 'percent' | 'fixed' | 'full'
export type VoucherStatus = 'active' | 'inactive' | 'expired' | 'redeemed'

export interface VoucherItem {
  id: string
  code: string
  voucher_type: VoucherType
  package_id: string
  package_name: string
  discount_type: DiscountType
  discount_value: number
  max_uses: number
  /** Real, live count — incremented by the student redemption flow. */
  used_count: number
  active: boolean
  /** Derived from active + expiry + used_count, never set directly. */
  status: VoucherStatus
  expires_at: string
  note: string
  school_name?: string | null
  referrer_name?: string | null
  referrer_commission?: number | null
  created_at: string
}

export interface VoucherPayload {
  voucher_type: VoucherType
  package_id: string
  discount_type: DiscountType
  discount_value: number
  max_uses: number
  expires_at: string
  note: string
  school_name?: string | null
  referrer_name?: string | null
  referrer_commission?: number | null
  quantity: number
}

export interface RedemptionResult {
  voucher_code: string
  package_id: string
  package_name: string
  discount_type: DiscountType
  discount_value: number
}

// ─── Gamifikasi ───────────────────────────────────────────────────────────────────
export interface PointRuleItem {
  id: string
  action: string
  category: string
  base_points: number
  multiplier: number
  enabled: boolean
  icon: string
  sort_order: number
  created_at: string
}
export interface PointRulePayload {
  action: string
  category: string
  base_points: number
  multiplier: number
  enabled: boolean
  icon: string
  sort_order: number
}

export type Rarity = 'common' | 'rare' | 'epic' | 'legendary'
export type BadgeConditionType = 'score_gte' | 'streak_gte' | 'questions_gte' | 'tryout_gte' | 'rank_lte'

export interface BadgeItem {
  id: string
  /** Legacy raw-emoji field — superseded by icon_type/icon_name/icon_url, no longer
   * rendered anywhere. Kept only because the backend still returns it. */
  emoji: string
  icon_type: IconType
  icon_name: string | null
  icon_url: string | null
  name: string
  description: string
  rarity: Rarity
  condition_type: BadgeConditionType
  condition_value: number
  active: boolean
  /** Always computed live from real attempt data — never stored or fabricated. */
  earned_by: number
  created_at: string
}
export interface BadgePayload {
  icon_type: IconType
  icon_name: string | null
  icon_url: string | null
  name: string
  description: string
  rarity: Rarity
  condition_type: BadgeConditionType
  condition_value: number
  active: boolean
}

export type ChallengeType = 'most_solved' | 'highest_score' | 'longest_streak' | 'speed'
export type ChallengeStatus = 'upcoming' | 'active' | 'ended'

export interface ChallengeItem {
  id: string
  name: string
  challenge_type: ChallengeType
  status: ChallengeStatus
  start_date: string
  end_date: string
  target_value: number
  reward_points: number
  reward_badge?: string | null
  description: string
  /** Always computed live from real attempts in [start_date, end_date] — never stored. */
  participants: number
  completions: number
  created_at: string
}
export interface ChallengePayload {
  name: string
  challenge_type: ChallengeType
  start_date: string
  end_date: string
  target_value: number
  reward_points: number
  reward_badge?: string | null
  description: string
}

export interface LeaderboardEntryItem {
  rank: number
  student_id: string
  student_name: string
  school_name?: string | null
  /** Best score (national/school scope) or accuracy % (per-subject scope). */
  best_score: number
  streak: number
}

export interface StudentBadgeStatusItem {
  id: string
  /** Legacy raw-emoji field — superseded by icon_type/icon_name/icon_url. */
  emoji: string
  icon_type: IconType
  icon_name: string | null
  icon_url: string | null
  name: string
  description: string
  rarity: 'common' | 'rare' | 'epic' | 'legendary'
  earned: boolean
  progress_percent: number
}

export type LeaderboardScope = 'national' | 'school'
export type LeaderboardRange = 'today' | 'week' | 'all_time'
export interface LeaderboardQueryParams {
  limit?: number
  scope?: LeaderboardScope
  range?: LeaderboardRange
  subject?: string
}

// ─── Keuangan ─────────────────────────────────────────────────────────────────────
// Dashboard agregat dari data nyata (Event, Sekolah, Paket) — bukan buku besar transaksi
// fiktif seperti di referensi, karena belum ada sistem pembayaran/checkout di aplikasi ini.
export interface FinanceSummary {
  total_event_revenue_estimate: number
  paid_events_count: number
  total_paid_participants: number
  school_count: number
  avg_revenue_share_percent: number
  total_monthly_revenue: number
  active_package_count: number
  package_catalog_value: number
}

export interface EventRevenueItem {
  event_id: string
  event_name: string
  price: number
  participants: number
  revenue_estimate: number
}

export interface SchoolRevenueItem {
  school_id: string
  school_name: string
  package_type: string
  revenue_share_percent: number
  monthly_revenue: number
}

// ─── Rasionalisasi SNBT/SNBP ───────────────────────────────────────────────────────
// Katalog PTN nyata (daya tampung/peminat/passing grade, sumber: data resmi SNBP & SNBT
// yang dipublikasikan) + estimasi peluang dari skor nyata siswa (rapor untuk SNBP,
// rata-rata tryout untuk SNBT). Lihat catatan metodologi di backend
// (domain::rationalization) — ini estimasi yang disederhanakan dari data yang benar-benar
// ada, bukan replikasi penuh formula referensi yang butuh data historis 3-tahun per
// sekolah yang tidak tersedia.

export type PtnTrack = 'snbp' | 'snbt'
export type PtnPriority = 'utama' | 'cadangan' | 'aman'
export type ChanceTier = 'ketat' | 'moderat' | 'aman'

export interface PtnProgramItem {
  id: string
  kode: number
  nama_ptn: string
  nama_prodi: string
  provinsi: string
  kota: string
  singkatan: string
  rumpun: string
  mapel_syarat: string
  daya_tampung_snbp: number
  peminat_snbp: number
  daya_tampung_snbt: number
  peminat_snbt: number
  pg_snbt: number
  pg_snbp: number
  jenjang: string
  /** `false` = prodi ini hasil riset deskriptif saja, belum ada angka daya tampung/peminat/
   * passing-grade resmi yang terverifikasi — UI harus tampilkan "Data belum lengkap", bukan
   * persentase peluang (backend juga sudah menolak menghitung chance asli untuk kasus ini,
   * lihat `unavailable_chance` di domain::rationalization). */
  has_official_stats: boolean
  created_at: string
  updated_at: string
}

export interface PtnProgramPayload {
  nama_ptn: string
  nama_prodi: string
  provinsi: string
  kota: string
  singkatan: string
  rumpun: string
  mapel_syarat: string
  daya_tampung_snbp: number
  peminat_snbp: number
  daya_tampung_snbt: number
  peminat_snbt: number
  pg_snbt: number
  pg_snbp: number
  jenjang: string
  has_official_stats?: boolean
}

export interface ProgramListResponse {
  items: PtnProgramItem[]
  total: number
  page: number
  page_size: number
}

// ─── Institution master data (profil institusi, di luar ptn_programs) ──────────────
// Dilengkapi lewat riset eksternal per institusi (lihat backend/data/institutions.csv) —
// field yang tidak berhasil diverifikasi sengaja dibiarkan null, bukan ditebak.
export interface InstitutionItem {
  id: string
  nama_ptn: string
  singkatan: string
  website: string | null
  alamat: string | null
  kota: string | null
  provinsi: string | null
  tahun_berdiri: number | null
  status: string | null
  akreditasi: string | null
  logo_url: string | null
  sumber: string | null
  created_at: string
  updated_at: string
}

export interface InstitutionPayload {
  nama_ptn: string
  singkatan: string
  website?: string | null
  alamat?: string | null
  kota?: string | null
  provinsi?: string | null
  tahun_berdiri?: number | null
  status?: string | null
  akreditasi?: string | null
  logo_url?: string | null
  sumber?: string | null
}

export interface RaporScoreItem {
  id: string
  semester: number
  subject: string
  score: number
  is_minat: boolean
}

export interface RaporEntryPayload {
  semester: number
  subject: string
  score: number
  is_minat: boolean
}

// ─── Import nilai rapor massal (Portal Sekolah, banyak siswa dari 1 file Excel) ────────
export interface RaporBulkImportEntryPayload {
  nis: string
  semester: number
  subject: string
  score: number
  is_minat: boolean
}

export interface RaporBulkImportErrorItem {
  nis: string
  semester: number
  subject: string
  reason: string
}

export interface RaporBulkImportResult {
  saved_count: number
  errors: RaporBulkImportErrorItem[]
}

export interface ChanceResponse {
  chance_percent: number
  tier: ChanceTier
  student_value: number
  program_threshold: number
  competition_ratio: number
  /** Poin persentase dari rasio persaingan program (real, daya_tampung/peminat). */
  competition_adjustment: number
  /** Poin persentase dari Index Prestasi siswa (real, 0 untuk jalur SNBT). */
  prestasi_contribution: number
}

export interface TargetWithChanceItem {
  id: string
  ptn_program_id: string
  track: PtnTrack
  priority: PtnPriority
  sort_order: number
  program: PtnProgramItem
  chance: ChanceResponse
}

// ─── School Rasionalisasi: Dashboard / Alumni ("Kaka Kelas") / Eligible ─────────
// Semua nyata: alumni & eligible diinput langsung oleh PIC sekolah (tidak ada dataset
// yang bisa menurunkannya otomatis); avg_chance_snbp dihitung live dari mesin
// compute_chance yang sama dengan tab Rasionalisasi siswa.

export interface UniversityCountItem {
  nama_ptn: string
  count: number
}

export interface StudentChanceSummaryItem {
  student_id: string
  student_name: string
  best_chance_percent: number
  tier: ChanceTier
}

export interface SchoolEligibilityItem {
  id: string
  year: number
  eligible_count: number
  updated_at: string
  /** Jumlah siswa yang benar-benar terdaftar aktif SNBP tahun ini (dari roster "Daftar
   * Siswa", bukan input manual) — dihitung live di backend, disandingkan ke eligible_count. */
  terdaftar_aktif_count: number
}

export interface SnbpRankingEntryItem {
  student_id: string
  student_name: string
  rapor_avg: number | null
  prestasi_index: number
  target_count: number
  best_chance_percent: number
  tier: ChanceTier
}

export interface SnbpDashboardItem {
  total_siswa_aktif: number
  terasionalisasi: number
  avg_chance_snbp: number | null
  eligible_terbaru: SchoolEligibilityItem | null
  distribusi_universitas: UniversityCountItem[]
  top_peluang: StudentChanceSummaryItem[]
}

export interface AlumniBenchmarkItem {
  id: string
  alumni_name: string
  graduation_year: number
  track: PtnTrack
  nama_ptn: string
  nama_prodi: string
  benchmark_score: number
  created_at: string
}

export interface AlumniPayload {
  alumni_name: string
  graduation_year: number
  track: PtnTrack
  nama_ptn: string
  nama_prodi: string
  benchmark_score: number
}

// ─── "Detail Rapor" alumni: snapshot nilai per mapel (bukan per-semester) ───────────────
export interface AlumniRaporScoreItem {
  id: string
  alumni_id: string
  subject: string
  score: number
}

export interface AlumniRaporEntryPayload {
  subject: string
  score: number
}

export interface EligibilityPayload {
  year: number
  eligible_count: number
}

// ─── Prestasi (student achievement) — feeds Index Prestasi di estimasi peluang SNBP ────
// Bobot per tingkat sama persis dengan sheet "Hitungan Rasionalisasi" (lihat
// backend domain::rationalization::AchievementLevel::bonus): sekolah 0.5, kab-kota 1,
// provinsi 2, nasional 4, internasional 8 — dijumlahkan lalu di-cap maksimal 10 poin.
export type AchievementLevelValue = 'sekolah' | 'kab-kota' | 'provinsi' | 'nasional' | 'internasional'

export interface AchievementItem {
  id: string
  nama: string
  tingkat: AchievementLevelValue
  tahun: number
  juara: string
  bonus: number
  certificate_url?: string | null
  created_at: string
}

// ─── Rekap SNBT (post-exam tracking, real: status/skor diisi manual oleh PIC sekolah
// setelah pengumuman resmi SNBT; est_score diprioritaskan dari Simulasi UTBK terakhir
// yang selesai (lebih realistis, 7 subtes), fallback ke rata-rata tryout per-subtes) ──

export type SnbtTrackingStatusValue = 'terdaftar' | 'sudah-ujian' | 'diterima' | 'tidak-diterima'
export type SnbtEstScoreSource = 'simulasi' | 'drilling'

export interface SnbtChoiceItem {
  nama_ptn: string
  nama_prodi: string
  priority: PtnPriority
  pg_snbt: number
  daya_tampung_snbt: number
  peminat_snbt: number
}

// Satu baris = satu siswa (bukan lagi per-target) — siswa cuma ikut 1 ujian UTBK/SNBT
// nyata yang menghasilkan 1 skor, dipakai bareng untuk pilihan 1 & 2 manapun yang
// diterima. Lihat SnbtTracking (backend) untuk penjelasan lengkap kenapa model lama
// (per-target) salah.
export interface SnbtRosterRowItem {
  student_id: string
  student_name: string
  pilihan_1: SnbtChoiceItem | null
  pilihan_2: SnbtChoiceItem | null
  est_score: number | null
  est_score_source: SnbtEstScoreSource | null
  actual_score: number | null
  exam_date: string | null
  status: SnbtTrackingStatusValue
  notes: string
}

export interface SnbtDashboardItem {
  terdaftar: number
  sudah_ujian: number
  diterima: number
  skor_perlu_diisi: number
}

// ─── Rekap TKA (School PIC) — see backend application::school_tka_service doc comment.
// A "TKA package" is any Package with exam_track = 'tka' (explicit admin signal). ───────
export interface TkaSubjectScoreItem {
  session_id: string
  session_title: string
  is_elective: boolean
  attempted: boolean
  /** Platform 0-1000 scale ("Skor Instan") — info pendukung transparansi, bukan skor resmi TKA. */
  raw_score: number | null
  /** Honest display-only rescale ke skala jenjang TKA (0-100 SD/SMP, 200-800 SMA/SMK/MA) —
   *  lihat `TkaPackageGroupItem.score_scale_label`. Never a real IRT score. */
  scaled_score: number | null
  /** Apakah skor ini mencapai ambang "Istimewa" untuk jenjang paket ini — lihat
   *  `TkaPackageGroupItem.istimewa_threshold`. */
  is_istimewa: boolean | null
  submitted_at: string | null
}

export interface TkaStudentRowItem {
  student_id: string
  student_name: string
  elective_chosen_count: number
  subjects: TkaSubjectScoreItem[]
}

export interface TkaPackageGroupItem {
  package_id: string
  package_name: string
  elective_pick_count: number
  /** Jenjang paket ini (dari `school_type_scope` sesi-sesinya) — `null` = tidak diset eksplisit,
   *  jatuh ke skala SMA/SMK/MA sebagai default. */
  school_type_scope: SchoolType | null
  /** Label skala tampilan, mis. "0–100" (SD/SMP) atau "200–800" (SMA/SMK/MA). */
  score_scale_label: string
  /** Ambang nilai kategori "Istimewa" untuk paket ini (95 atau 725). */
  istimewa_threshold: number
  students: TkaStudentRowItem[]
}

export interface SnbtTrackingPayload {
  actual_score: number | null
  exam_date: string | null
  status: SnbtTrackingStatusValue
  notes: string
}

// ─── SNBP roster ("Daftar Siswa", School portal) ───────────────────────────────
// pilihan_1/pilihan_2 selalu diturunkan live dari target PTN nyata siswa (bukan field
// yang bisa diedit terpisah) — hanya konsultan/status/aktif yang benar-benar diinput PIC.
export type SnbpStatusValue = 'belum' | 'diterima-snbp' | 'diterima-snbt' | 'tidak'

export interface SnbpRosterRowItem {
  student_id: string
  student_name: string
  kelas: string | null
  konsultan: string
  pilihan_1: string | null
  pilihan_2: string | null
  status: SnbpStatusValue
  aktif: boolean
  year: number
  /** Real, student-declared "Jurusan yang Diminati" — read-only on the school side (only the
   * student sets this). `null` means the student hasn't filled it in yet. */
  minat_jurusan: string | null
}

export interface SnbpParticipationPayload {
  year: number
  konsultan: string
  status: SnbpStatusValue
  aktif: boolean
}

// ─── Audit trail ("Riwayat Perubahan") — accountability log for edit-on-behalf actions.
// Only actions taken by a School PIC/Admin on a student's data get logged here; students
// editing their own data never appear.
export interface AuditLogEntryItem {
  id: string
  actor_name: string
  actor_role: string
  action: string
  entity_type: string
  summary: string
  created_at: string
}

// ─── Laporan (report-builder + saved reports, School portal) ──────────────────────
// Performance/Participation dihitung langsung dari roster+attempts nyata (bisa difilter
// per kelas nyata); PtnTarget memakai ulang agregat rasionalisasi yang sudah nyata
// (dashboard SNBP + dashboard SNBT) dan selalu mencakup seluruh sekolah (filter kelas
// diabaikan untuk jenis ini). Setiap laporan yang di-generate disimpan sebagai snapshot
// permanen di backend — membuka laporan lama menampilkan data persis saat dibuat.

export type ReportTypeValue = 'performance' | 'participation' | 'ptn-target' | 'tka' | 'akreditasi'

/** `null` = "Semua Jalur" (SNBT + TKA blended) — see backend
 *  `domain::report::Report::exam_track_filter` doc comment. */
export type ReportExamTrackFilter = 'snbt' | 'tka' | null

export interface GenerateReportPayload {
  report_type: ReportTypeValue
  period_label: string
  year_month: string | null
  class_filter: string | null
  exam_track_filter?: ReportExamTrackFilter
  /** Ambang nilai kelulusan ("KKM") untuk laporan Akreditasi — default 75 di backend jika
   *  tidak diisi. Diabaikan untuk jenis laporan lain. */
  threshold?: number | null
}

export interface ClassAvgRowItem {
  grade: string
  student_count: number
  avg_score: number | null
}

export interface MonthlyTrendPointItem {
  month_label: string
  avg_score: number | null
  attempt_count: number
}

export interface ReportRankingRowItem {
  student_id: string
  student_name: string
  grade: string | null
  attempt_count: number
  best_score: number | null
  avg_score: number | null
}

export interface PerformancePayloadItem {
  kind: 'performance'
  total_students: number
  avg_score: number | null
  per_class: ClassAvgRowItem[]
  monthly_trend: MonthlyTrendPointItem[]
  ranking: ReportRankingRowItem[]
}

export interface ParticipationRowItem {
  student_id: string
  student_name: string
  grade: string | null
  attempt_count: number
  last_attempt_at: string | null
}

export interface ParticipationPayloadItem {
  kind: 'participation'
  total_students: number
  active_students: number
  participation_rate: number
  total_attempts: number
  avg_attempts_per_active_student: number
  rows: ParticipationRowItem[]
}

export interface PtnTargetPayloadItem {
  kind: 'ptn-target'
  total_siswa_dengan_target: number
  distribusi_universitas: UniversityCountItem[]
  snbt_terdaftar: number
  snbt_sudah_ujian: number
  snbt_diterima: number
  avg_chance_snbp: number | null
}

/** Package-level rollup for the "Hasil TKA" report — see backend
 *  `domain::report::TkaPackageSummaryRow` doc comment. Scores are on the jenjang-appropriate
 *  honest display-only rescale (see `score_scale_label`), never a real IRT score. */
export interface TkaPackageSummaryItem {
  package_id: string
  package_name: string
  elective_pick_count: number
  student_count: number
  elective_ready_count: number
  school_type_scope: SchoolType | null
  score_scale_label: string
  istimewa_threshold: number
  avg_wajib_score: number | null
  avg_pilihan_score: number | null
  /** Rata-rata skor mentah platform (skala 0-1000, "Skor Instan") — info pendukung
   *  transparansi, bukan skor resmi TKA. */
  avg_raw_wajib_score: number | null
  avg_raw_pilihan_score: number | null
  avg_istimewa_count: number | null
}

export interface TkaStudentReportRowItem {
  student_id: string
  student_name: string
  grade: string | null
  package_name: string
  school_type_scope: SchoolType | null
  score_scale_label: string
  istimewa_threshold: number
  elective_chosen_count: number
  elective_pick_count: number
  avg_wajib_score: number | null
  avg_pilihan_score: number | null
  avg_raw_wajib_score: number | null
  avg_raw_pilihan_score: number | null
  istimewa_count: number
  subject_count: number
}

export interface TkaReportPayloadItem {
  kind: 'tka'
  total_students: number
  packages: TkaPackageSummaryItem[]
  students: TkaStudentReportRowItem[]
}

/** See backend `domain::report::AkreditasiReportPayload` doc comment: ringkasan prestasi
 *  akademik siswa untuk dokumentasi akreditasi (EDS) — bukan export Dapodik lengkap. */
export interface AkreditasiClassRowItem {
  grade: string
  student_count: number
  avg_score: number | null
  above_threshold_count: number
}

export interface AkreditasiStudentRowItem {
  student_id: string
  student_name: string
  grade: string | null
  attempt_count: number
  avg_score: number | null
  above_threshold: boolean
  has_ptn_target: boolean
}

export interface AkreditasiPayloadItem {
  kind: 'akreditasi'
  total_students: number
  threshold_used: number
  avg_score_overall: number | null
  students_above_threshold: number
  students_with_ptn_target: number
  per_subject: SubjectAccuracy[]
  per_class: AkreditasiClassRowItem[]
  students: AkreditasiStudentRowItem[]
}

export type ReportPayloadItem = PerformancePayloadItem | ParticipationPayloadItem | PtnTargetPayloadItem | TkaReportPayloadItem | AkreditasiPayloadItem

export interface ReportItem {
  id: string
  school_id: string
  report_type: ReportTypeValue
  title: string
  period_label: string
  class_filter: string | null
  exam_track_filter: ReportExamTrackFilter
  payload: ReportPayloadItem
  created_at: string
}

export interface ReportSummaryItem {
  id: string
  report_type: ReportTypeValue
  title: string
  period_label: string
  class_filter: string | null
  exam_track_filter: ReportExamTrackFilter
  created_at: string
}

export interface AchievementPayload {
  nama: string
  tingkat: AchievementLevelValue
  tahun: number
  juara: string
}

// ─── Payment gateway (scaffolding only — belum ada akun/API key Midtrans/Xendit yang
// terhubung, jadi setiap checkout akan gagal dengan pesan jujur, bukan pura-pura sukses.
// Lihat backend domain::payment untuk detail lengkap.) ─────────────────────────────────

export type PaymentStatusValue = 'pending' | 'paid' | 'failed' | 'cancelled' | 'expired'

export interface PaymentTransactionItem {
  id: string
  package_id: string
  buyer_user_id: string
  amount: number
  currency: string
  provider: string | null
  provider_ref: string | null
  status: PaymentStatusValue
  failure_reason: string | null
  created_at: string
  updated_at: string
  paid_at: string | null
}

export interface CheckoutSessionItem {
  transaction_id: string
  redirect_url: string
  provider: string
}

// Tagged union on `mode` — mirrors backend `InitiatePurchaseResponse`. `checkout` only
// happens once a real gateway is ever configured; today `POST /payments/checkout` always
// returns `manual_pending` (see PaymentService::provider_name / NotConfiguredProvider).
export type InitiatePurchaseResult =
  | { mode: 'checkout'; transaction: PaymentTransactionItem; checkout: CheckoutSessionItem }
  | { mode: 'manual_pending'; transaction: PaymentTransactionItem }

export interface PaymentProviderStatusItem {
  provider: string
  configured: boolean
}

// Admin manual purchase-request queue row — GET /api/payments/pending.
export interface PendingPurchaseItem {
  transaction: PaymentTransactionItem
  buyer_name: string
  buyer_email: string
  package_name: string
}

// POST /api/payments/:id/fulfill response — transaction now Paid + the real, auto-redeemed
// Access voucher code the admin relays to the buyer manually.
export interface FulfillResult {
  transaction: PaymentTransactionItem
  voucher_code: string
}

// ─── Public (no-auth) — backs the marketing landing page. Real system-wide numbers and
// the real package catalog instead of a hardcoded, independently-maintained duplicate. ──

export interface PublicStats {
  total_students: number
  total_schools: number
  total_questions: number
  tryouts_completed: number
  average_score: number
}

export interface PublicVoucherCheck {
  valid: boolean
  reason: string | null
  package_name: string | null
  discount_type: DiscountType | null
  discount_value: number | null
  expires_at: string | null
}

export interface RegistrationStatus {
  b2c_registration_enabled: boolean
}

// ─── Platform-wide settings (Admin-only) — B2C self-registration on/off toggle + UTBK/SNBT
// "Skor Instan" vs "Estimasi IRT" display mode. See backend `domain::platform_settings` doc
// comment. ───────────────────────────────────────────────────────────────────────

/** Which UTBK/SNBT score(s) are shown to students & school reports. 'instant' (default) =
 *  perilaku sebelum fitur ini ada. See backend `domain::platform_settings::ScoreDisplayMode`
 *  and `domain::irt` doc comments — "Estimasi IRT" adalah estimasi kalibrasi platform sendiri,
 *  BUKAN replikasi skor UTBK resmi SNPMB. */
export type ScoreDisplayMode = 'instant' | 'irt' | 'both'

export interface PlatformSettings {
  b2c_registration_enabled: boolean
  score_display_mode: ScoreDisplayMode
  /** Global "Mode Terkunci" (focus/lockdown mode) default applied to every Simulasi TO that
   *  doesn't set its own explicit per-template override — see
   *  `SimulationTemplateItem.lockdown_override`. `false` by default. */
  simulation_lockdown_default: boolean
  updated_by: string | null
  updated_at: string
}

/** Honest current IRT calibration state — `null` fields mean "belum pernah dikalibrasi". */
export interface IrtCalibrationStatus {
  calibrated_item_count: number
  last_calibrated_at: string | null
}

/** Result of one "Kalibrasi Ulang IRT" run. */
export interface IrtRecalibrationSummary {
  calibrated_count: number
  skipped_insufficient_data_count: number
}

// ─── Notifications — real per-user feed, see backend `domain::notification` doc comment. ──

export interface NotificationItem {
  id: string
  kind: string
  title: string
  message: string
  link_tab: string | null
  read: boolean
  created_at: string
}

// ─── Simulasi UTBK (admin authoring: chains existing tryout_sessions into one ordered,
// server-timed template — see backend domain::simulation). Student player is a separate,
// not-yet-built feature; this only covers the admin template CRUD contract. ────────────
/** `"utbk_combined"` (default): the run's `combined_estimate_score` is a real equal-weighted
 *  estimate to show. `"per_subject"` (e.g. Simulasi TKA): real TKA reports a score PER MATA UJI
 *  and never combines them — `combined_estimate_score` is always `null` for these; read each
 *  slot's own `score` instead. */
export type SimulationTemplateKind = 'utbk_combined' | 'per_subject'

export interface SimulationTemplateSlotItem {
  sequence_index: number
  /** `null` when `is_elective` — no single fixed session, it depends on which student is
   *  taking it (resolved per-student from `elective_package_id`'s current picks at run start). */
  session_id: string | null
  session_title: string | null
  duration_minutes: number | null
  question_count: number | null
  subject_filter: string | null
  break_seconds: number
  /** See `SimulationTemplateItem.elective_package_id` doc comment. */
  is_elective: boolean
}

export interface SimulationTemplateItem {
  id: string
  title: string
  is_active: boolean
  /** Added alongside per-package content scoping — was previously ungated entirely. Defaults
   * `true` server-side; unlockable only once an admin assigns this template to a Package (see
   * PackageContentItem / `PackageService.addContentItem`). */
  is_premium: boolean
  /** Same "still being assembled, hidden from students" semantics as
   *  `TryoutSessionTemplate.is_draft` — see that field's doc comment. */
  is_draft: boolean
  /** Which Package's "mapel pilihan" choices resolve this template's `is_elective` slots.
   *  `null` when the template has no elective slot at all (every pre-existing Simulasi UTBK
   *  template, and a TKA "Hari 1 (Wajib)" template). */
  elective_package_id: string | null
  template_kind: SimulationTemplateKind
  /** See ExamTrack doc comment. */
  exam_track: ExamTrack
  /** See `TryoutSessionTemplate.school_type_scope` doc comment. */
  school_type_scope?: SchoolType | null
  /** "Mode Terkunci" (focus/lockdown mode) override for this template. `null`/`undefined` =
   *  inherit `PlatformSettings.simulation_lockdown_default`; `true`/`false` force it on/off
   *  regardless of the global default. The *effective* decision is only resolved (and
   *  immutably snapshotted) once a student starts a run — see `SimulationRunItem.lockdown_enabled`. */
  lockdown_override?: boolean | null
  /** Sum of every slot's KNOWN duration_minutes, server-computed. Under-counts (excludes
   *  `is_elective` slots, whose real duration is only known once a student's run resolves them)
   *  — show "≈" rather than presenting this as exact when the template has any elective slot. */
  total_duration_minutes: number
  slots: SimulationTemplateSlotItem[]
  created_at: string
}

export interface SimulationTemplateSlotPayload {
  sequence_index: number
  /** Omit/`null` iff `is_elective`. */
  session_id: string | null
  break_seconds: number
  is_elective?: boolean
}

export interface SimulationTemplatePayload {
  title: string
  is_premium: boolean
  slots: SimulationTemplateSlotPayload[]
  is_draft?: boolean
  /** Required iff at least one slot is `is_elective`. */
  elective_package_id?: string | null
  template_kind?: SimulationTemplateKind
  /** See ExamTrack doc comment. Defaults to 'snbt' server-side if omitted. */
  exam_track?: ExamTrack
  /** See `TryoutSessionTemplate.school_type_scope` doc comment. Omitted/`null` = semua jenjang. */
  school_type_scope?: SchoolType | null
  /** See `SimulationTemplateItem.lockdown_override` doc comment. Omitted/`null` = inherit the
   *  platform-wide default. */
  lockdown_override?: boolean | null
}

// ─── Per-package content scoping — a Package explicitly lists which TryoutSessions and/or
// SimulationTemplates it unlocks (replaces the old all-or-nothing "any package = everything"
// model). See backend domain::package::PackageContentItem doc comment. ──────────────────────
export type PackageContentType = 'tryout_session' | 'simulation_template'

export interface PackageContentItem {
  id: string
  package_id: string
  content_type: PackageContentType
  content_id: string
  content_title: string
  created_at: string
}

/** Response of `GET /school-entitlements/me/status` — everything the logged-in student can
 * currently play. `has_premium_access` is a coarse "unlocked ANYTHING at all" convenience flag
 * (derived server-side from the two id lists) for generic upsell banners; real per-item gating
 * of a specific Tryout/Simulasi card must check membership in the id lists instead. */
export interface AccessStatus {
  has_premium_access: boolean
  unlocked_tryout_session_ids: string[]
  unlocked_simulation_template_ids: string[]
}

// ─── "Mapel Pilihan" (TKA-style subject choice) — student-facing. See backend
// application::elective_service doc comment. ────────────────────────────────────────
export interface ElectivesResponse {
  /** Every `is_elective` session attached to this package (non-draft only). */
  sessions: TryoutSessionTemplate[]
  /** Exact count the student must pick. `0` = no restriction. */
  pick_count: number
  /** This student's current picks among `sessions` above. */
  my_choices: string[]
}

/** One package's "mapel pilihan" group — see `GET /api/electives` (my_elective_packages). */
export interface ElectivePackageGroup {
  package_id: string
  package_name: string
  sessions: TryoutSessionTemplate[]
  pick_count: number
  my_choices: string[]
}

// ─── Simulasi UTBK — student player (server-authoritative run state; see
// frontend/pages/simulasi/[runId].vue). Reuses PlayableQuestion above as-is. ──────────────
export interface SimulationRunSlotItem {
  sequence_index: number
  session_id: string
  session_title: string
  subject_filter: string | null
  duration_minutes: number
  break_seconds: number
  status: 'pending' | 'active' | 'submitted'
  attempt_id: string | null
  score: number | null
  /** Detail breakdown for the sertifikat/laporan PDF's per-subtest table + strength/weakness
   *  analysis — `null` until this slot's attempt is submitted. */
  accuracy: number | null
  correct_count: number | null
  wrong_count: number | null
  unanswered_count: number | null
  deadline_at: string | null
  break_ends_at: string | null
}

export interface SimulationRunItem {
  id: string
  /** Lets the catalog (DrillingZone) match a completed run back to its template — e.g. to swap
   *  "Mulai Simulasi" for "Lihat Hasil" once the student has finished it. */
  template_id: string
  template_title: string
  template_kind: SimulationTemplateKind
  /** Jenjang paket template ini — dipakai player + sertifikat utk pilih skala tampilan TKA yang
   *  benar (0-100 SD/SMP, 200-800 SMA/SMK/MA). `null` = tidak diset eksplisit, jatuh ke default
   *  SMA/SMK/MA. See backend `domain::simulation::SimulationRun::school_type_scope` doc comment. */
  school_type_scope: SchoolType | null
  status: 'in_progress' | 'completed' | 'abandoned'
  current_sequence_index: number
  /** Real estimate only when `template_kind === 'utbk_combined'` — always `null` for
   *  `'per_subject'` (e.g. Simulasi TKA); read each slot's own `score` instead. */
  combined_estimate_score: number | null
  started_at: string
  completed_at: string | null
  /** Immutable snapshot taken when this run started — see `SimulationTemplateItem.lockdown_override`
   *  and backend `domain::simulation::SimulationRun::lockdown_enabled` doc comments. When `true`,
   *  the player page (`pages/simulasi/[runId].vue`) must enforce "Mode Terkunci": fullscreen,
   *  hidden navigation, blocked copy/right-click/tab-switch, and violation reporting. */
  lockdown_enabled: boolean
  slots: SimulationRunSlotItem[]
  current_questions: PlayableQuestion[] | null
}

/** Honest, aggregate-only benchmark for a Simulasi template — see backend
 *  `domain::simulation::TemplateCompletionStats` doc comment. `avg_combined_score` is `null`
 *  whenever there's no completed run with a real combined score yet (new template, or every
 *  completed run so far is `per_subject`) — render "belum ada data pembanding", never a fake 0. */
export interface TemplateStatsItem {
  template_id: string
  participant_count: number
  avg_combined_score: number | null
}

/** One detected "Mode Terkunci" breach during a run (tab switch, fullscreen exit, devtools
 *  attempt, etc.) — see backend `domain::simulation::LockdownViolation` doc comment. */
export interface LockdownViolationItem {
  id: string
  event_type: string
  detail: string | null
  occurred_at: string
}
