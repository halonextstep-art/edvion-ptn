//! Package (Manajemen Paket) CRUD use-cases. Admin-only, mirroring the reference
//! `AdminPackageManagement`. Pricing/marketing content shown on the landing page —
//! `discount` is always derived from the two prices (see `domain::package::compute_discount`),
//! never accepted or stored directly, so it can't be set inconsistently by a client.

use std::sync::Arc;

use uuid::Uuid;

use crate::application::question_service::QuestionService;
use crate::application::simulation_service::SimulationService;
use crate::application::tryout_service::TryoutService;
use crate::domain::package::{ExamTrack, Package, PackageContentItem, PackageContentType};
use crate::domain::repository::{
    NewPackage, PackageContentRepository, PackageRepository, PackageUpdate, SimulationTemplateRepository,
    TryoutSessionRepository,
};
use crate::domain::user::Role;
use crate::error::{AppError, AppResult};
use crate::interfaces::http::middleware::AuthUser;

fn exam_track_label(t: ExamTrack) -> &'static str {
    match t {
        ExamTrack::Snbt => "SNBT/UTBK",
        ExamTrack::Tka => "TKA",
    }
}

pub struct CreatePackageInput {
    pub name: String,
    pub package_type: String,
    pub original_price: i32,
    pub sale_price: i32,
    pub validity: String,
    pub features: Vec<String>,
    pub badge: String,
    pub emoji: String,
    pub icon_type: String,
    pub icon_name: Option<String>,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub gradient: String,
    pub accent_color: String,
    pub active: bool,
    pub sort_order: i32,
    /// See `domain::package::Package::elective_pick_count` doc comment.
    pub elective_pick_count: i32,
    pub exam_track: ExamTrack,
}

pub struct UpdatePackageInput {
    pub name: String,
    pub package_type: String,
    pub original_price: i32,
    pub sale_price: i32,
    pub validity: String,
    pub features: Vec<String>,
    pub badge: String,
    pub emoji: String,
    pub icon_type: String,
    pub icon_name: Option<String>,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub gradient: String,
    pub accent_color: String,
    pub active: bool,
    pub sort_order: i32,
    pub elective_pick_count: i32,
    pub exam_track: ExamTrack,
}

pub struct PackageService {
    packages: Arc<dyn PackageRepository>,
    content: Arc<dyn PackageContentRepository>,
    sessions: Arc<dyn TryoutSessionRepository>,
    templates: Arc<dyn SimulationTemplateRepository>,
    tryout: Arc<TryoutService>,
    simulation: Arc<SimulationService>,
    questions: Arc<QuestionService>,
}

impl PackageService {
    pub fn new(
        packages: Arc<dyn PackageRepository>,
        content: Arc<dyn PackageContentRepository>,
        sessions: Arc<dyn TryoutSessionRepository>,
        templates: Arc<dyn SimulationTemplateRepository>,
        tryout: Arc<TryoutService>,
        simulation: Arc<SimulationService>,
        questions: Arc<QuestionService>,
    ) -> Self {
        Self { packages, content, sessions, templates, tryout, simulation, questions }
    }

    /// Admin sees any package's content; Content only their own (mirrors `get`/`list`).
    /// "Isi Paket": which TryoutSessions/SimulationTemplates does owning this Package unlock.
    /// See `domain::package::PackageContentItem` doc comment for the per-package-scoping
    /// rationale.
    pub async fn list_content(&self, actor: &AuthUser, package_id: Uuid) -> AppResult<Vec<PackageContentItem>> {
        self.require_visible(actor, package_id).await?;
        self.content.list_for_package(package_id).await
    }

    /// "Setujui Semua Soal" — bulk-approves every question that belongs to a "Set Soal" used
    /// by any `tryout_session` attached to this package, instead of an admin having to open
    /// and approve each question one by one in Bank Soal. Scoped ONLY to the Set Soal path
    /// (`TryoutSession::question_set_id`) — deliberately does NOT touch the random-filter path
    /// (subject/topic/difficulty), since that draws from the whole shared question bank rather
    /// than questions actually written for this package, and bulk-approving by a filter match
    /// could sweep in unrelated content the admin never intended to approve. This directly
    /// targets the exact workflow gap the "Buat Paket + Subtes" wizard creates: every question
    /// a Content-role author writes there starts as `draft` (see `QuestionService::create`),
    /// and a single package can easily bundle dozens of them across many subtes.
    pub async fn bulk_approve_content(&self, actor: &AuthUser, package_id: Uuid) -> AppResult<usize> {
        actor.require_role(&[Role::Admin])?;
        self.require_visible(actor, package_id).await?;

        let items = self.content.list_for_package(package_id).await?;
        let mut set_ids: Vec<Uuid> = Vec::new();
        for item in items {
            if item.content_type != PackageContentType::TryoutSession {
                continue;
            }
            if let Some(session) = self.sessions.find_by_id(item.content_id).await? {
                if let Some(set_id) = session.question_set_id {
                    set_ids.push(set_id);
                }
            }
        }
        self.questions.bulk_approve_by_sets(actor, &set_ids).await
    }

    /// Validates BOTH the package and the referenced content actually exist before linking
    /// them — `package_content_items.content_id` has no DB-level FK (it's polymorphic across
    /// two tables), so this is the only place that check happens. `Role::Content` may attach
    /// content to a package they own (the "Buat Paket + Subtes" wizard) — see `require_owned`.
    pub async fn add_content_item(
        &self,
        actor: &AuthUser,
        package_id: Uuid,
        content_type: PackageContentType,
        content_id: Uuid,
    ) -> AppResult<PackageContentItem> {
        let pkg = self.require_owned(actor, package_id).await?;
        // Guardrail added after the real "TKA SMP #2" mismatch (a TKA-content session ended up
        // tagged 'snbt', so it never got picked up by exam_track filtering — see
        // `backend/scripts/fix_tka_smp_exam_track.sql`). Attaching a session/template whose own
        // exam_track disagrees with its package's would silently recreate that exact class of
        // bug (content scored/reported under the wrong jalur ujian), so it's rejected outright
        // rather than allowed and left to be discovered later via a mismatched report.
        let content_track = match content_type {
            PackageContentType::TryoutSession => {
                self.sessions.find_by_id(content_id).await?
                    .ok_or_else(|| AppError::NotFound(format!("session {content_id} not found")))?
                    .exam_track
            }
            PackageContentType::SimulationTemplate => {
                self.templates.find_by_id(content_id).await?
                    .ok_or_else(|| AppError::NotFound(format!("template {content_id} not found")))?
                    .exam_track
            }
        };
        if content_track != pkg.exam_track {
            return Err(AppError::Validation(format!(
                "Jalur ujian tidak cocok: paket ini '{}' tapi konten yang dipilih '{}'. Pilih konten dengan jalur ujian yang sama, atau ubah jalur ujian paket terlebih dahulu.",
                exam_track_label(pkg.exam_track),
                exam_track_label(content_track),
            )));
        }
        self.content.add_item(package_id, content_type, content_id).await
    }

    pub async fn remove_content_item(&self, actor: &AuthUser, item_id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        if !self.content.remove_item(item_id).await? {
            return Err(AppError::NotFound(format!("content item {item_id} not found")));
        }
        Ok(())
    }

    /// `Role::Content` may create a package too, via the "Buat Paket + Subtes" wizard — it is
    /// always forced inactive regardless of what the caller sends: per product decision, Konten
    /// only assembles up to a draft, Admin alone finalizes pricing and publishes (`set_active`).
    pub async fn create(&self, actor: &AuthUser, input: CreatePackageInput) -> AppResult<Package> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        validate(&input.name, input.original_price, input.sale_price)?;
        if input.elective_pick_count < 0 {
            return Err(AppError::Validation("jumlah mapel pilihan tidak boleh negatif".to_string()));
        }
        let active = if actor.role == Role::Content { false } else { input.active };
        self.packages
            .create(NewPackage {
                name: input.name,
                package_type: input.package_type,
                original_price: input.original_price,
                sale_price: input.sale_price,
                validity: input.validity,
                features: input.features,
                badge: input.badge,
                emoji: input.emoji,
                icon_type: input.icon_type,
                icon_name: input.icon_name,
                icon_url: input.icon_url,
                banner_url: input.banner_url,
                gradient: input.gradient,
                accent_color: input.accent_color,
                active,
                sort_order: input.sort_order,
                created_by: actor.user_id,
                elective_pick_count: input.elective_pick_count,
                exam_track: input.exam_track,
            })
            .await
    }

    /// Admin sees every package; Content only the ones they created themselves (their drafts
    /// awaiting Admin review, plus anything of theirs already published).
    pub async fn list(&self, actor: &AuthUser) -> AppResult<Vec<Package>> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        let mut all = self.packages.list().await?;
        if actor.role == Role::Content {
            all.retain(|p| p.created_by == actor.user_id);
        }
        Ok(all)
    }

    async fn require_visible(&self, actor: &AuthUser, package_id: Uuid) -> AppResult<Package> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        let pkg = self.packages.find_by_id(package_id).await?
            .ok_or_else(|| AppError::NotFound(format!("package {package_id} not found")))?;
        if actor.role != Role::Admin && pkg.created_by != actor.user_id {
            return Err(AppError::Forbidden("Anda hanya bisa melihat paket yang Anda buat sendiri".to_string()));
        }
        Ok(pkg)
    }

    async fn require_owned(&self, actor: &AuthUser, package_id: Uuid) -> AppResult<Package> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        let pkg = self.packages.find_by_id(package_id).await?
            .ok_or_else(|| AppError::NotFound(format!("package {package_id} not found")))?;
        if actor.role != Role::Admin && pkg.created_by != actor.user_id {
            return Err(AppError::Forbidden("Anda hanya bisa mengubah paket yang Anda buat sendiri".to_string()));
        }
        Ok(pkg)
    }

    /// No-auth read for the public marketing landing page — only active packages, in the
    /// same display order admins configured, so the page never diverges from what's
    /// actually for sale (previously the landing page had its own separate hardcoded
    /// catalog that could silently drift from the real one).
    pub async fn list_public(&self) -> AppResult<Vec<Package>> {
        let mut all = self.packages.list().await?;
        all.retain(|p| p.active);
        all.sort_by_key(|p| p.sort_order);
        Ok(all)
    }

    /// No-auth — used only by the public marketing landing page to show what a package
    /// actually includes (e.g. "Simulasi UTBK Batch 1 (Simulasi TO)"), so a prospective buyer
    /// isn't going off the admin's free-text `features` bullets alone. Safe to expose without
    /// auth: content titles are already public-facing marketing information (session/template
    /// titles), nothing sensitive. Returns an empty list (not an error) for an unknown/kosong
    /// package_id — the landing page should degrade gracefully, not break.
    pub async fn public_content_titles(&self, package_id: Uuid) -> AppResult<Vec<String>> {
        let items = self.content.list_for_package(package_id).await?;
        Ok(items
            .into_iter()
            .map(|i| {
                let kind = match i.content_type {
                    PackageContentType::TryoutSession => "Sesi Tryout",
                    PackageContentType::SimulationTemplate => "Simulasi TO",
                };
                format!("{} ({kind})", i.content_title)
            })
            .collect())
    }

    pub async fn get(&self, actor: &AuthUser, id: Uuid) -> AppResult<Package> {
        self.require_visible(actor, id).await
    }

    pub async fn update(&self, actor: &AuthUser, id: Uuid, input: UpdatePackageInput) -> AppResult<Package> {
        actor.require_role(&[Role::Admin])?;
        validate(&input.name, input.original_price, input.sale_price)?;
        if input.elective_pick_count < 0 {
            return Err(AppError::Validation("jumlah mapel pilihan tidak boleh negatif".to_string()));
        }
        self.packages
            .update(
                id,
                PackageUpdate {
                    name: input.name,
                    package_type: input.package_type,
                    original_price: input.original_price,
                    sale_price: input.sale_price,
                    validity: input.validity,
                    features: input.features,
                    badge: input.badge,
                    emoji: input.emoji,
                    icon_type: input.icon_type,
                    icon_name: input.icon_name,
                    icon_url: input.icon_url,
                    banner_url: input.banner_url,
                    gradient: input.gradient,
                    accent_color: input.accent_color,
                    active: input.active,
                    sort_order: input.sort_order,
                    elective_pick_count: input.elective_pick_count,
                    exam_track: input.exam_track,
                },
            )
            .await?
            .ok_or_else(|| AppError::NotFound(format!("package {id} not found")))
    }

    /// Admin-only — this is the single official "Publish Paket" action. Turning a package
    /// active ALSO clears `is_draft` on every TryoutSession/SimulationTemplate it contains, so
    /// a package built via the "Buat Paket + Subtes" wizard (Admin or Content) can never end up
    /// active while its own content is still hidden from the catalogue as a draft — publishing
    /// the package is what publishes everything inside it, in one action. Best-effort: a single
    /// content item failing to un-draft (e.g. deleted out from under the package) does not abort
    /// the publish, since the package itself is still valid and the admin can retry that item.
    pub async fn set_active(&self, actor: &AuthUser, id: Uuid, active: bool) -> AppResult<Package> {
        actor.require_role(&[Role::Admin])?;
        let pkg = self.packages
            .set_active(id, active)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("package {id} not found")))?;
        if active {
            let items = self.content.list_for_package(id).await.unwrap_or_default();
            for item in items {
                let _ = match item.content_type {
                    PackageContentType::TryoutSession => self.tryout.set_draft(item.content_id, false).await.map(|_| ()),
                    PackageContentType::SimulationTemplate => {
                        self.simulation.set_template_draft(item.content_id, false).await.map(|_| ())
                    }
                };
            }
        }
        Ok(pkg)
    }

    /// Admin may delete any package. `Role::Content` may delete only a package they created
    /// themselves AND that is still a draft (`!active`) — once Admin publishes it (`set_active`),
    /// it may be live on the landing page or already purchased, so only Admin can remove it from
    /// there on. This mirrors `require_owned`'s ownership check but adds the draft-only gate.
    pub async fn delete(&self, actor: &AuthUser, id: Uuid) -> AppResult<()> {
        actor.require_role(&[Role::Admin, Role::Content])?;
        if actor.role == Role::Content {
            let pkg = self.packages.find_by_id(id).await?
                .ok_or_else(|| AppError::NotFound(format!("package {id} not found")))?;
            if pkg.created_by != actor.user_id {
                return Err(AppError::Forbidden("Anda hanya bisa menghapus paket yang Anda buat sendiri".to_string()));
            }
            if pkg.active {
                return Err(AppError::Forbidden("Paket ini sudah live — hubungi Admin untuk menghapusnya".to_string()));
            }
        }
        let deleted = self.packages.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("package {id} not found")));
        }
        Ok(())
    }
}

fn validate(name: &str, original_price: i32, sale_price: i32) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("nama paket wajib diisi".to_string()));
    }
    if original_price <= 0 {
        return Err(AppError::Validation("harga asli harus lebih dari 0".to_string()));
    }
    if sale_price <= 0 {
        return Err(AppError::Validation("harga jual harus lebih dari 0".to_string()));
    }
    if sale_price > original_price {
        return Err(AppError::Validation("harga jual tidak boleh lebih besar dari harga asli".to_string()));
    }
    Ok(())
}
