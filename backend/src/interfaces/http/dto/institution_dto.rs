use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::institution::Institution;
use crate::domain::repository::{InstitutionUpdate, NewInstitution};

#[derive(Debug, Serialize)]
pub struct InstitutionResponse {
    pub id: Uuid,
    pub nama_ptn: String,
    pub singkatan: String,
    pub website: Option<String>,
    pub alamat: Option<String>,
    pub kota: Option<String>,
    pub provinsi: Option<String>,
    pub tahun_berdiri: Option<i32>,
    pub status: Option<String>,
    pub akreditasi: Option<String>,
    pub logo_url: Option<String>,
    pub sumber: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Institution> for InstitutionResponse {
    fn from(i: Institution) -> Self {
        Self {
            id: i.id,
            nama_ptn: i.nama_ptn,
            singkatan: i.singkatan,
            website: i.website,
            alamat: i.alamat,
            kota: i.kota,
            provinsi: i.provinsi,
            tahun_berdiri: i.tahun_berdiri,
            status: i.status,
            akreditasi: i.akreditasi,
            logo_url: i.logo_url,
            sumber: i.sumber,
            created_at: i.created_at,
            updated_at: i.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct InstitutionPayload {
    #[validate(length(min = 1, message = "nama PTN wajib diisi"))]
    pub nama_ptn: String,
    #[validate(length(min = 1, message = "singkatan wajib diisi"))]
    pub singkatan: String,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub alamat: Option<String>,
    #[serde(default)]
    pub kota: Option<String>,
    #[serde(default)]
    pub provinsi: Option<String>,
    #[serde(default)]
    pub tahun_berdiri: Option<i32>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub akreditasi: Option<String>,
    #[serde(default)]
    pub logo_url: Option<String>,
    #[serde(default)]
    pub sumber: Option<String>,
}

impl InstitutionPayload {
    pub fn into_new(self) -> NewInstitution {
        NewInstitution {
            nama_ptn: self.nama_ptn,
            singkatan: self.singkatan,
            website: self.website,
            alamat: self.alamat,
            kota: self.kota,
            provinsi: self.provinsi,
            tahun_berdiri: self.tahun_berdiri,
            status: self.status,
            akreditasi: self.akreditasi,
            logo_url: self.logo_url,
            sumber: self.sumber,
        }
    }

    pub fn into_update(self) -> InstitutionUpdate {
        InstitutionUpdate {
            singkatan: self.singkatan,
            website: self.website,
            alamat: self.alamat,
            kota: self.kota,
            provinsi: self.provinsi,
            tahun_berdiri: self.tahun_berdiri,
            status: self.status,
            akreditasi: self.akreditasi,
            logo_url: self.logo_url,
            sumber: self.sumber,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct InstitutionListQuery {
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InstitutionLookupQuery {
    pub nama_ptn: String,
}
