pub mod build_pg;
pub mod build_sqlite;

pub fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    build_pg::build();
    build_sqlite::build();
}
