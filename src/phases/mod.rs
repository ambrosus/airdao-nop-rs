pub mod check_docker;

use futures_util::future::BoxFuture;

use crate::error;

pub trait Phase {
    fn run<'a>() -> BoxFuture<'a, Result<(), error::AppError>>;
}
