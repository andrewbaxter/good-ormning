use std::{
    path::PathBuf,
    env,
    str::FromStr,
};

pub mod build_pg;
pub mod build_sqlite;

pub fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let root = PathBuf::from_str(&env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();
    build_pg::build(&root);
    build_sqlite::build(&root);
}
