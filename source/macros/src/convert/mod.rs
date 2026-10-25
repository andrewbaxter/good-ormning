pub mod mariadb;
pub mod pg;
pub mod sqlite;

pub fn ident_part(p: &sqlparser::ast::ObjectNamePart) -> String {
    match p {
        sqlparser::ast::ObjectNamePart::Identifier(i) => return i.value.clone(),
        sqlparser::ast::ObjectNamePart::Function(_) => panic!("Unsupported: function in object name: {}", p),
    }
}

pub fn check_table_function_options(tf: &sqlparser::ast::TableFactor) {
    let sqlparser::ast::TableFactor::Table {
        name: _,
        alias: _,
        args,
        with_hints,
        version,
        with_ordinality: _,
        partitions,
        json_path,
        sample,
        index_hints,
    } =
        tf else {
            panic!("Expected a table: {}", tf);
        };
    if !with_hints.is_empty() || version.is_some() || !partitions.is_empty() || json_path.is_some() ||
        sample.is_some() ||
        !index_hints.is_empty() ||
        args.as_ref().is_some_and(|a| a.settings.is_some()) {
        panic!("Unsupported table syntax: {}", tf);
    }
}

pub fn check_table_options(tf: &sqlparser::ast::TableFactor) {
    check_table_function_options(tf);
    let sqlparser::ast::TableFactor::Table { alias, with_ordinality, .. } = tf else {
        unreachable!();
    };
    if *with_ordinality {
        panic!("Unsupported table syntax: {}", tf);
    }
    plain_alias(tf, alias);
}

pub fn plain_alias(tf: &sqlparser::ast::TableFactor, alias: &Option<sqlparser::ast::TableAlias>) -> Option<String> {
    let alias = alias.as_ref()?;
    if !alias.columns.is_empty() {
        panic!("Column aliases are only supported on table functions: {}", tf);
    }
    return Some(alias.name.value.clone());
}

pub fn check_wildcard_options(options: &sqlparser::ast::WildcardAdditionalOptions) {
    let sqlparser::ast::WildcardAdditionalOptions {
        wildcard_token: _,
        opt_ilike,
        opt_exclude,
        opt_except,
        opt_replace,
        opt_rename,
        opt_alias,
    } =
        options;
    if opt_ilike.is_some() || opt_exclude.is_some() || opt_except.is_some() || opt_replace.is_some() ||
        opt_rename.is_some() ||
        opt_alias.is_some() {
        panic!("Wildcard modifiers aren't supported: {}", options);
    }
}

pub fn check_function_options(f: &sqlparser::ast::Function) {
    if !f.within_group.is_empty() || f.null_treatment.is_some() ||
        !matches!(f.parameters, sqlparser::ast::FunctionArguments::None) ||
        matches!(f.args, sqlparser::ast::FunctionArguments::Subquery(_)) {
        panic!("Unsupported function syntax: {}", f);
    }
}
#[cfg(test)]
mod tests;
