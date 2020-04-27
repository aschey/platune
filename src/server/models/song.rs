use serde::{Deserialize, Serialize};
use paperclip::actix::{
    // extension trait for actix_web::App and proc-macro attributes
    OpenApiExt, Apiv2Schema, api_v2_operation,
    // use this instead of actix_web::web
    web::{self, Json},
    api_v2_errors
};
#[derive(Queryable, Serialize, Apiv2Schema)]
pub struct Song {
    pub path: String,
    pub artist: String,
    pub name: String
}