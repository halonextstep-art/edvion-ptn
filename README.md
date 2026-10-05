# EdvionPTN — Full-Stack Implementation

Full-stack implementation of the EdvionPTN exam-prep platform (referensi/ Figma/React
blueprint), built with:

- **Backend**: Rust (Axum + SQLx) + PostgreSQL, layered/OOP architecture
  (`domain -> application -> infrastructure/interfaces`).
- **Frontend**: Nuxt 3 (Vue 3, TypeScript, Tailwind), OOP service-layer
  (`ApiClient -> AuthService/QuestionService/TryoutService`) + Pinia stores.

See `backend/README.md` and `frontend/README.md` for setup instructions specific to
each app.

## Scope of this milestone

Per the agreed first-slice plan, this implementation covers **Auth (4 roles) + Question
Bank (Admin/Content CRUD + review workflow) + Drilling/Tryout (Siswa)** end-to-end,
fully wired frontend-to-backend-to-database. Other modules from the reference design
(Event Management, Partner Management, Analytics, Finance, Gamification, Voucher,
Rasionalisasi SNBT, Leaderboard, School portal) are intentionally out of scope for this
pass — the architecture (layered backend, service-based frontend) is built to extend
into those cleanly in a follow-up.

## Quick start

```bash
# 1. Backend
cd backend
cp .env.example .env   # edit DATABASE_URL / JWT_SECRET
createdb gaspolptn
cargo run               # runs migrations automatically, serves on :8080
cargo run --bin seed    # demo accounts + sample questions + sample sessions

# 2. Frontend (separate terminal)
cd frontend
cp .env.example .env
npm install
npm run dev              # serves on :3000, talks to the API at :8080/api
```

Demo logins (created by `cargo run --bin seed`):

| Role    | Email                 | Password   |
|---------|------------------------|------------|
| Admin   | admin@edvionptn.id    | admin123   |
| School  | sekolah@sman1.sch.id   | sekolah123 |
| Student | siswa@student.com      | siswa123   |
| Content | konten@edvion.id       | konten123  |

## Architecture notes

**Backend** (`backend/src/`):
- `domain/` — entities (`User`, `Question`, `TryoutSession`, `Attempt`, ...) and
  repository *traits* (ports). No SQL, no HTTP.
- `application/` — use-case services (`AuthService`, `QuestionService`,
  `TryoutService`) that hold business rules and depend only on the domain traits.
- `infrastructure/` — Postgres repository implementations (adapters) fulfilling the
  domain traits.
- `interfaces/http/` — Axum routes, DTOs, JWT auth middleware. The only layer that
  knows Axum exists.

**Frontend** (`frontend/`):
- `services/` — plain TypeScript classes (`ApiClient`, `AuthService`,
  `QuestionService`, `TryoutService`) mirroring the backend's service layer.
- `stores/` — Pinia stores for cross-page state (`auth`, `attemptSession`).
- `composables/` — `useApi()`, `useAuth()`, `useToast()` wire the services/stores into
  components idiomatically.
- `components/ui/` — hand-built shadcn-style primitives (Button, Card, Input, Tabs,
  Dialog, ...).
- `pages/` — landing (`/`), login (`/login`, 4 role tabs), admin/content dashboard
  shell (`/admin`, tabbed nav matching the reference — only "Bank Soal"/"Review Soal"
  is fully functional, other tabs show scoped-out placeholders), school portal shell
  (`/sekolah`, same pattern), student dashboard (`/siswa`, Drilling Zone),
  `/tryout/[attemptId]` (full player: intro → quiz → results → review).
- Visual fidelity: landing, login, student dashboard, admin Bank Soal (review
  queue/history + detail panel), admin shell, and school shell were all rebuilt as
  pixel-close ports of the corresponding `referensi/` React components (same Tailwind
  classes, copy, layout, and interactions where the underlying feature is in scope).

## Verified working

- **Backend**: compiles and runs locally via `cargo build` / `cargo run` — confirmed
  end-to-end (auth, question CRUD + review workflow, drilling/tryout attempts) after a
  few rounds of local compiler-error fixes (all resolved; see git history if curious).
  `cargo run --bin seed` populates demo accounts, sample questions, and session
  templates.
- **Frontend**: `npm run build` completes cleanly (client + server bundles, 0
  TypeScript/Vue errors) after every round of changes, most recently after the
  pixel-close visual rework of all main pages.
- **API contract**: cross-checked field-by-field — Rust DTOs
  (`interfaces/http/dto/*.rs`) and routes (`interfaces/http/router.rs`) match the
  frontend's `types/index.ts` and `services/*.ts` exactly, no drift found.

## Known limitations / next steps

- School and Content portals only get placeholder/reduced views for modules outside
  the agreed scope (Content shares the Admin question-bank UI with a trimmed tab set;
  School gets a full shell with tab navigation but most tabs are "coming soon") —
  intentional scope cut, not an oversight.
- Modules from the reference design listed under "Scope of this milestone" as out of
  scope are not implemented at all (no backend tables/endpoints for them yet).
