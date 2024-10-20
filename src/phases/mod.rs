pub mod actions_menu;
pub mod check_docker;
pub mod check_status;
pub mod select_network;
pub mod select_node_ip;
pub mod select_private_key;

use futures_util::future::BoxFuture;

use crate::error;

pub trait Phase {
    fn run(&mut self) -> BoxFuture<'_, Result<(), error::AppError>>;
}
