//! "Mapel Pilihan" — TKA-style subject-selection use-cases. A Package can mark some of its
//! `TryoutSession`s as `is_elective` (e.g. TKA SMA's Fisika/Kimia/Ekonomi/etc.) and set an
//! `elective_pick_count` (e.g. 2) — the exact number of those elective sessions a student must
//! choose before any of them become startable. The package's mandatory sessions (`is_elective =
//! false`) are completely unaffected and always startable, exactly as before this feature
//! existed. Per product decision, a student's choice is NOT locked once made — `set_choices` is
//! a full replace, so a student may freely add/remove/swap picks at any time (see
//! `domain::repository::PackageElectiveRepository::set_choices` doc comment).

use std::collections::HashSet;
use std::sync::Arc;

use uuid::Uuid;

use crate::domain::package::{Package, PackageContentType};
use crate::domain::repository::{PackageContentRepository, PackageElectiveRepository, PackageRepository, TryoutSessionRepository};
use crate::domain::tryout::TryoutSession;
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

/// One package's "mapel pilihan" group, for the student-facing overview — every package that
/// currently has at least one eligible elective session, regardless of whether this student owns
/// it yet (picking is free/preparatory; real access is still gated separately by `is_premium` at
/// start-attempt time).
pub struct PackageElectiveGroup {
    pub package: Package,
    pub sessions: Vec<TryoutSession>,
    pub my_choices: Vec<Uuid>,
}

pub struct ElectiveService {
    sessions: Arc<dyn TryoutSessionRepository>,
    packages: Arc<dyn PackageRepository>,
    content: Arc<dyn PackageContentRepository>,
    choices: Arc<dyn PackageElectiveRepository>,
}

impl ElectiveService {
    pub fn new(
        sessions: Arc<dyn TryoutSessionRepository>,
        packages: Arc<dyn PackageRepository>,
        content: Arc<dyn PackageContentRepository>,
        choices: Arc<dyn PackageElectiveRepository>,
    ) -> Self {
        Self { sessions, packages, content, choices }
    }

    /// This student's current picks for this package (empty list if they haven't chosen yet).
    pub async fn my_choices(&self, actor: &AuthUser, package_id: Uuid) -> AppResult<Vec<Uuid>> {
        actor.require_role(&[Role::Student])?;
        self.choices.list_choices(actor.user_id, package_id).await
    }

    /// Combined view for the student-facing "Pilih Mapel Pilihan" screen — one round trip
    /// instead of three. `(sessions, elective_pick_count, my_current_choices)`.
    pub async fn get_electives_view(&self, actor: &AuthUser, package_id: Uuid) -> AppResult<(Vec<TryoutSession>, i32, Vec<Uuid>)> {
        actor.require_role(&[Role::Student])?;
        let pkg = self.packages.find_by_id(package_id).await?
            .ok_or_else(|| AppError::NotFound(format!("package {package_id} not found")))?;
        let sessions = self.elective_sessions(package_id).await?;
        let my_choices = self.choices.list_choices(actor.user_id, package_id).await?;
        Ok((sessions, pkg.elective_pick_count, my_choices))
    }

    /// Full replace of this student's picks for this package. Validates the pick count against
    /// the package's `elective_pick_count` (when > 0 — a package with 0 has no restriction and
    /// this endpoint has nothing to enforce) and that every chosen id is actually one of this
    /// package's `is_elective` sessions (guards against a student picking an arbitrary/mandatory
    /// session id from elsewhere).
    pub async fn set_choices(&self, actor: &AuthUser, package_id: Uuid, session_ids: Vec<Uuid>) -> AppResult<Vec<Uuid>> {
        actor.require_role(&[Role::Student])?;
        let pkg = self.packages.find_by_id(package_id).await?
            .ok_or_else(|| AppError::NotFound(format!("package {package_id} not found")))?;

        // De-dupe defensively — a client resubmitting the same id twice shouldn't count as 2
        // picks against the limit.
        let mut deduped: Vec<Uuid> = Vec::new();
        let mut seen = HashSet::new();
        for id in session_ids {
            if seen.insert(id) {
                deduped.push(id);
            }
        }

        if pkg.elective_pick_count > 0 && deduped.len() > pkg.elective_pick_count as usize {
            return Err(AppError::Validation(format!(
                "maksimal {} mapel pilihan untuk paket \"{}\"",
                pkg.elective_pick_count, pkg.name
            )));
        }

        let valid_ids: HashSet<Uuid> = self.elective_sessions(package_id).await?.into_iter().map(|s| s.id).collect();
        for id in &deduped {
            if !valid_ids.contains(id) {
                return Err(AppError::Validation(
                    "salah satu sesi yang dipilih bukan mapel pilihan pada paket ini".to_string(),
                ));
            }
        }

        self.choices.set_choices(actor.user_id, package_id, deduped.clone()).await?;
        Ok(deduped)
    }

    /// Every package that has `elective_pick_count > 0` AND at least one eligible (non-draft,
    /// `is_elective`) session — powers the student-facing "Pilih Mapel Pilihan" overview
    /// (`DrillingZone.vue`) without requiring the frontend to already know which package ids to
    /// ask about. Packages with no eligible electives yet (still drafts, or opted out via
    /// `elective_pick_count = 0`) are silently omitted rather than shown empty.
    pub async fn list_my_elective_packages(&self, actor: &AuthUser) -> AppResult<Vec<PackageElectiveGroup>> {
        actor.require_role(&[Role::Student])?;
        let all_packages = self.packages.list().await?;
        let mut groups = Vec::new();
        for pkg in all_packages {
            if pkg.elective_pick_count <= 0 {
                continue;
            }
            let sessions = self.elective_sessions(pkg.id).await?;
            if sessions.is_empty() {
                continue;
            }
            let my_choices = self.choices.list_choices(actor.user_id, pkg.id).await?;
            groups.push(PackageElectiveGroup { package: pkg, sessions, my_choices });
        }
        Ok(groups)
    }

    /// Called by `TryoutService::start_attempt_impl` right alongside the existing premium-access
    /// check. `Ok(())` (no gate) when: the session isn't `is_elective` at all, OR its owning
    /// package's `elective_pick_count` is 0 (package opted out of the restriction), OR the
    /// session isn't attached to any package (shouldn't happen via the wizard, but not
    /// DB-enforced — fails open here rather than blocking a session forever; the content-
    /// authoring side already gates real visibility via `is_draft`/`is_premium` separately).
    /// `Err(Forbidden)` when the session IS gated and this student hasn't currently chosen it.
    pub(crate) async fn assert_can_start(&self, student_id: Uuid, session: &TryoutSession) -> AppResult<()> {
        if !session.is_elective {
            return Ok(());
        }
        let package_ids = self.content.package_ids_for_content(PackageContentType::TryoutSession, session.id).await?;
        for package_id in package_ids {
            let Some(pkg) = self.packages.find_by_id(package_id).await? else { continue };
            if pkg.elective_pick_count <= 0 {
                continue;
            }
            let chosen = self.choices.list_choices(student_id, package_id).await?;
            if !chosen.contains(&session.id) {
                return Err(AppError::Forbidden(format!(
                    "\"{}\" termasuk mapel pilihan paket \"{}\" — pilih mapel ini dulu di halaman \"Pilih Mapel Pilihan\" (maks {} mapel) sebelum mengerjakan",
                    session.title, pkg.name, pkg.elective_pick_count
                )));
            }
        }
        Ok(())
    }

    /// Shared lookup: every non-draft `is_elective` `TryoutSession` attached to this package.
    async fn elective_sessions(&self, package_id: Uuid) -> AppResult<Vec<TryoutSession>> {
        let items = self.content.list_for_package(package_id).await?;
        let mut out = Vec::new();
        for item in items {
            if item.content_type != PackageContentType::TryoutSession {
                continue;
            }
            if let Some(session) = self.sessions.find_by_id(item.content_id).await? {
                if session.is_elective && !session.is_draft {
                    out.push(session);
                }
            }
        }
        Ok(out)
    }
}
