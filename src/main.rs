#![feature(impl_trait_in_bindings)]
pub mod tiles;
pub mod solve;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
mod cli;

#[cfg(target_arch = "wasm32")]
fn main() {
    web::main()
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), tiles::TilesError> {
    cli::main()
}
