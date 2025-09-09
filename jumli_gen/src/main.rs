#![allow(dead_code)] // TODO: Remove

pub mod consts;
pub mod records;
pub mod sources;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
}
