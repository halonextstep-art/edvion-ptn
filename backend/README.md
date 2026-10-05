# EdvionPTN Backend

Rust (Axum + SQLx) API server for EdvionPTN, following a layered/OOP-inspired
architecture:

```
src/
  domain/          entities + repository traits (ports) — no SQL, no HTTP
  application/      use-case services (AuthService, QuestionService, TryoutService)
  infrastructure/    Postgres repository implementations (adapters)
  interfaces/http/   Axum routes, DTOs, JWT auth middleware
```

Dependency direction: `interfaces -> application -> domain`, with `infrastructure`
implementing `domain::repository` traits and only being wired together in `main.rs`
(the composition root). This means `application` services never import `sqlx` or
`axum` directly, and can be unit tested with fake repositories if desired.

## Prerequisites

- Rust 1.75+ (`rustup install stable`)
- PostgreSQL 14+

## Setup

```bash
cp .env.example .env
# edit .env with your local Postgres connection string + a real JWT_SECRET

createdb gaspolptn   # or use your preferred Postgres client

cargo run            # applies migrations automatically on boot, then serves on :8080

# Local/dev/staging only — never against a production DATABASE_URL. Populates demo
# accounts, sample questions & tryout sessions with publicly-documented passwords
# (see backend/src/bin/seed.rs doc comment).
SEED_CONFIRM=yes-seed-demo-data cargo run --bin seed
```

Demo accounts (created by `cargo run --bin seed`, local/dev/staging only):

| Role    | Email                   | Password    |
|---------|--------------------------|-------------|
| Admin   | admin@edvionptn.id      | admin123    |
| School  | sekolah@sman1.sch.id     | sekolah123  |
| Student | siswa@student.com        | siswa123    |
| Content | konten@edvion.id         | konten123   |

To remove demo data from a database it was already seeded into (e.g. before going
live), run `backend/scripts/cleanup_demo_data.sql` — see that file's header comment
for what it does and doesn't remove, and how to dry-run it first.

## API surface

```
GET    /health

POST   /api/auth/register
POST   /api/auth/login
GET    /api/auth/me                          (auth)

GET    /api/questions                        (auth; ?search=&subject=&status=&difficulty=&mine=&page=&page_size=)
POST   /api/questions                        (auth: admin | content)
GET    /api/questions/:id                    (auth)
PUT    /api/questions/:id                    (auth: admin | owner-content)
DELETE /api/questions/:id                    (auth: admin | owner-content)
POST   /api/questions/:id/review             (auth: admin) { action: approve|reject|revision, note? }

GET    /api/tryout/sessions                  (auth; ?type=tryout|drilling|mini)
POST   /api/tryout/sessions                  (auth: admin)
POST   /api/tryout/sessions/:id/start        (auth: student) -> attempt + questions (no answer key)
PUT    /api/tryout/attempts/:id/answers      (auth: owner) autosave one answer
POST   /api/tryout/attempts/:id/submit       (auth: owner) grades server-side, returns score
GET    /api/tryout/attempts                  (auth: student) my attempt history
GET    /api/tryout/attempts/:id              (auth: owner)
GET    /api/tryout/attempts/:id/review       (auth: owner) full review incl. correct answers
```

All authenticated routes expect `Authorization: Bearer <jwt>`.

## Tests

```bash
cargo test   # domain::tryout::compute_score unit tests
```

## Notes / follow-ups for the next slice

- School CRUD (Partner Management), Events, Finance, Gamification, Voucher modules
  from the reference design are not implemented yet — only Auth, Question Bank, and
  Drilling/Tryout, per the agreed first milestone.
- `random_approved()` uses `ORDER BY random()`, which is fine at current scale but
  should move to a smarter sampling strategy once the question bank grows large.
- Short-answer grading (`normalize()` in `tryout_service.rs`) is intentionally simple
  (trim/lowercase/comma-to-dot); revisit if more forgiving grading is needed.
