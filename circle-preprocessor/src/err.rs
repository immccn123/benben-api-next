use thiserror::Error;

macro_rules! from_err {
    ($raw_errname:path, $errname:ident) => {
        impl From<$raw_errname> for Error {
            fn from(err: $raw_errname) -> Self {
                Error::$errname(err)
            }
        }
    };
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("database error")]
    DbErr(#[from] sea_orm::DbErr),
    #[error("IO error")]
    IoErr(#[from] std::io::Error),
    #[error("parse int error")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("unknown error")]
    UndefiendError(UndefiendError),
}

from_err!(UndefiendError, UndefiendError);

pub type UndefiendError = ();
