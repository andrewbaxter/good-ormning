pub mod pg;
pub mod sqlite;

pub fn normalize_ident(ident: &sqlparser::ast::Ident) -> String {
    ident.value.clone()
}
