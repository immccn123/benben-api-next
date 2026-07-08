use std::io::Cursor;

use rocket::{
    http::{ContentType, Status},
    response::Responder,
    Response,
};

use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(unused)]
pub enum AppError {
    #[error("DatabaseError")]
    DbErr(#[from] sea_orm::DbErr),
    #[error("TypeError: {0}")]
    TypeErr(&'static str),
    #[error("Param Validate Failed")]
    ParamValidateFailed(&'static str),
    #[error("InvalidParam: {0}")]
    InvalidParam(String),
    #[error("Network Error")]
    NetworkError(#[from] reqwest::Error),
    #[error("Unimplemented Feature")]
    UnimplementedYet,
    #[error("ModelError: {0}")]
    ModelError(String),
}

impl<'r> Responder<'r, 'r> for AppError {
    fn respond_to(self, _request: &'_ rocket::Request<'_>) -> rocket::response::Result<'r> {
        eprintln!(
            "AppError: {}",
            match &self {
                Self::DbErr(err) => err.to_string(),
                Self::TypeErr(err) => err.to_string(),
                Self::NetworkError(err) => err.to_string(),

                // Nothing to worry about
                Self::ParamValidateFailed(msg) => {
                    return Ok(Response::build()
                        .header(ContentType::JSON)
                        .status(Status::UnprocessableEntity)
                        .sized_body(msg.len(), Cursor::new(*msg))
                        .finalize());
                }
                Self::InvalidParam(msg) => {
                    return Ok(Response::build()
                        .header(ContentType::JSON)
                        .status(Status::UnprocessableEntity)
                        .sized_body(msg.len(), Cursor::new(msg.clone()))
                        .finalize());
                }
                Self::UnimplementedYet => {
                    return Ok(Response::build().status(Status::NotImplemented).finalize());
                }
                Self::ModelError(msg) => {
                    return Ok(Response::build()
                        .header(ContentType::JSON)
                        .status(Status::InternalServerError)
                        .sized_body(msg.len(), Cursor::new(msg.clone()))
                        .finalize());
                }
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
