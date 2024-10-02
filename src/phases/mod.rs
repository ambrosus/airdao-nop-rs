pub mod check_docker;
pub mod select_network;

use futures_util::future::BoxFuture;

use crate::error;

pub trait Phase {
    fn run<'a>(&'a mut self) -> BoxFuture<'a, Result<(), error::AppError>>;
}
