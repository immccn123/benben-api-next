use std::io::Cursor;

use rocket::{
    error,
    http::{ContentType, Status},
    response::Responder,
    Response,
};

use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(unused)]
pub enum AppError {
    #[error("Database error")]
    DbErr(#[from] sea_orm::DbErr),
    #[error("Type error")]
    TypeErr(&'static str),
    #[error("XMLRPC error")]
    XmlRpcErr(#[from] dxr_client::ClientError),
    #[error("Param Validate Failed")]
    ParamValidateFailed(&'static str),
}

impl<'r> Responder<'r, 'r> for AppError {
    fn respond_to(self, _request: &'_ rocket::Request<'_>) -> rocket::response::Result<'r> {
        if let Self::ParamValidateFailed(msg) = self {
            return Ok(Response::build()
                .header(ContentType::JSON)
                .status(Status::ImATeapot)
                .sized_body(msg.len(), Cursor::new(msg))
                .finalize());
        }

        eprintln!(
            "AppError: {}",
            match &self {
                Self::DbErr(err) => err.to_string(),
                Self::TypeErr(err) => err.to_string(),
                Self::XmlRpcErr(err) => err.to_string(),
                _ => panic!("never goes here"),
            }
        );

        let json = json!({ "msg": self.to_string() }).to_string();

        Ok(Response::build()
            .header(ContentType::JSON)
            .status(Status::InternalServerError)
            .sized_body(json.len(), Cursor::new(json))
            .finalize())
    }
}
