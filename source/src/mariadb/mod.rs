#[cfg(test)]
mod test {
    use {
        good_ormning_core::QueryResCount,
        super::{
            GenerateArgs,
            Version,
            generate,
            new_insert,
            new_select,
            new_update,
            query::{
                expr::{
                    BinOp,
                    Expr,
                },
                helpers::{
                    FnOpt,
                    fn_fixed,
                },
            },
            schema::field::{
                field_f32,
                field_f64,
                field_i16,
                field_i32,
                field_i64,
                field_str,
                field_u32,
            },
            types::{
                SimpleSimpleType,
                type_i64,
            },
        },
    };

    fn assert_generate_err(args: GenerateArgs, err_text: &str) {
        std::env::set_var("OUT_DIR", std::env::temp_dir().join("good_ormning_unit_tests"));
        let errs = generate(args).unwrap_err();
        assert!(errs.iter().any(|e| e.contains(err_text)), "{:?}", errs);
    }

    #[test]
    #[should_panic(expected = "already exists on table")]
    fn test_add_field_dup_bad() {
        generate(GenerateArgs {
            db_name: None,
            versions: vec![(0usize, {
                let v = Version::new();
                v.table("zPAO2PJU4").field("z437INV6D", field_str().build());
                v.build()
            }), (1usize, {
                let v = Version::new();
                let bananna = v.table("zQZQ8E2WD");
                bananna.field("z437INV6D", field_str().build());
                bananna.field("z437INV6D", field_i32().build());
                v.build()
            })],
            ..Default::default()
        }).unwrap();
    }

    #[test]
    fn test_auto_increment_non_int_bad() {
        assert_generate_err(GenerateArgs {
            db_name: Some("test_auto_increment_non_int_bad".to_string()),
            versions: vec![(0usize, {
                let v = Version::new();
                let bananna = v.table("zMOY9YMCK");
                let id = bananna.field("zPREUVAOD", field_str().max_len(10).auto_increment().build());
                bananna.primary_key(&[&id]);
                v.build()
            })],
            ..Default::default()
        }, "is auto increment but isn't an integer field");
    }

    #[test]
    fn test_assign_numeric_good() {
        std::env::set_var("OUT_DIR", std::env::temp_dir().join("good_ormning_unit_tests"));
        let v = Version::new();
        let bananna = v.table("bananna");
        let n = bananna.field("n", field_i32().build());
        let small = bananna.field("small", field_i16().build());
        let u = bananna.field("u", field_u32().build());
        let f = bananna.field("f", field_f32().build());
        let d = bananna.field("d", field_f64().build());
        generate(GenerateArgs {
            db_name: Some("test_assign_numeric_good".to_string()),
            default_time_library: None,
            versions: vec![(0usize, v.build())],
            queries: vec![
                new_update(&bananna, vec![(n.clone(), Expr::BinOp {
                    left: Box::new(Expr::Field(n.to_ref())),
                    op: BinOp::Plus,
                    right: Box::new(Expr::LitAnyInt(1)),
                }), (u.clone(), Expr::BinOp {
                    left: Box::new(Expr::Field(n.to_ref())),
                    op: BinOp::BitwiseOr,
                    right: Box::new(Expr::LitAnyInt(4)),
                }), (f.clone(), Expr::BinOp {
                    left: Box::new(Expr::Field(f.to_ref())),
                    op: BinOp::Multiply,
                    right: Box::new(Expr::LitAnyInt(2)),
                })]).build_query("computed"),
                new_update(
                    &bananna,
                    vec![
                        (n.clone(), Expr::Field(small.to_ref())),
                        (d.clone(), Expr::Field(n.to_ref())),
                        (u.clone(), Expr::LitAnyInt(4000000000)),
                    ],
                ).build_query("widen"),
            ],
        }).unwrap();
    }

    #[test]
    fn test_untyped_time_needs_library_bad() {
        let v = Version::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_i32().build());
        assert_generate_err(GenerateArgs {
            db_name: Some("test_untyped_time_needs_library_bad".to_string()),
            default_time_library: None,
            versions: vec![(0usize, v.build())],
            queries: vec![
                new_select(&bananna)
                    .return_named("t", fn_fixed("now", vec![], SimpleSimpleType::AnyDatetime, FnOpt::Never))
                    .build_query("x", QueryResCount::One),
            ],
        }, "set `default_time_library`");
    }

    #[test]
    fn test_assign_narrow_column_bad() {
        let v = Version::new();
        let bananna = v.table("bananna");
        let n = bananna.field("n", field_i32().build());
        let big = bananna.field("big", field_i64().build());
        assert_generate_err(GenerateArgs {
            db_name: Some("test_assign_narrow_column_bad".to_string()),
            default_time_library: None,
            versions: vec![(0usize, v.build())],
            queries: vec![new_update(&bananna, vec![(n.clone(), Expr::Field(big.to_ref()))]).build_query("x")],
        }, "not assignable to I32 without losing values");
    }

    #[test]
    fn test_assign_narrow_param_bad() {
        let v = Version::new();
        let bananna = v.table("bananna");
        let n = bananna.field("n", field_i32().build());
        assert_generate_err(GenerateArgs {
            db_name: Some("test_assign_narrow_param_bad".to_string()),
            default_time_library: None,
            versions: vec![(0usize, v.build())],
            queries: vec![new_update(&bananna, vec![(n.clone(), Expr::Param {
                name: "n".to_string(),
                type_: type_i64().build(),
            })]).build_query("x")],
        }, "not assignable to I32 without losing values");
    }

    #[test]
    fn test_assign_literal_range_bad() {
        let v = Version::new();
        let bananna = v.table("bananna");
        let n = bananna.field("n", field_i32().build());
        assert_generate_err(GenerateArgs {
            db_name: Some("test_assign_literal_range_bad".to_string()),
            default_time_library: None,
            versions: vec![(0usize, v.build())],
            queries: vec![
                new_update(&bananna, vec![(n.clone(), Expr::LitAnyInt(3000000000))]).build_query("x"),
            ],
        }, "not assignable to I32 without losing values");
    }

    #[test]
    fn test_assign_float_to_int_bad() {
        let v = Version::new();
        let bananna = v.table("bananna");
        let n = bananna.field("n", field_i32().build());
        let f = bananna.field("f", field_f64().build());
        assert_generate_err(GenerateArgs {
            db_name: Some("test_assign_float_to_int_bad".to_string()),
            default_time_library: None,
            versions: vec![(0usize, v.build())],
            queries: vec![new_update(&bananna, vec![(n.clone(), Expr::BinOp {
                left: Box::new(Expr::Field(f.to_ref())),
                op: BinOp::Multiply,
                right: Box::new(Expr::LitAnyInt(2)),
            })]).build_query("x")],
        }, "use `cast` to convert it");
    }

    #[test]
    #[should_panic(expected = "already exists")]
    fn test_add_table_dup_bad() {
        generate(GenerateArgs {
            db_name: None,
            versions: vec![(0usize, {
                let v = Version::new();
                v.table("zSNS34DYI").field("z437INV6D", field_str().build());
                v.build()
            }), (1usize, {
                let v = Version::new();
                v.table("zSNS34DYI").field("z437INV6D", field_str().build());
                v.table("zSNS34DYI").field("z437INV6D", field_str().build());
                v.build()
            })],
            ..Default::default()
        }).unwrap();
    }

    #[test]
    fn test_res_count_none_bad() {
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        assert_generate_err(GenerateArgs {
            db_name: Some("test_res_count_none_bad".to_string()),
            default_time_library: None,
            versions: vec![(0usize, v.build())],
            queries: vec![new_select(&bananna).return_field(&hizat).build_query("x", QueryResCount::None)],
        }, "Query has returning values but result count is None");
    }

    #[test]
    fn test_returning_none_bad() {
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        assert_generate_err(GenerateArgs {
            db_name: Some("test_returning_none_bad".to_string()),
            default_time_library: None,
            versions: vec![(0usize, v.build())],
            queries: vec![
                new_insert(&bananna, vec![(hizat.clone(), Expr::LitString("hoy".into()))])
                    .return_field(&hizat)
                    .build_query("x", QueryResCount::None),
            ],
        }, "Query has returning values but result count is None");
    }

    #[test]
    fn test_auto_unkeyed_bad() {
        let v = Version::new();
        v.table("bananna").field("id", field_i64().auto_increment().build());
        assert_generate_err(GenerateArgs {
            db_name: Some("test_auto_unkeyed_bad".to_string()),
            default_time_library: None,
            versions: vec![(0usize, v.build())],
            queries: vec![],
        }, "requires auto increment fields to be the first field");
    }

    #[test]
    fn test_unbounded_key_bad() {
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.primary_key(&[&hizat]);
        assert_generate_err(GenerateArgs {
            db_name: Some("test_unbounded_key_bad".to_string()),
            default_time_library: None,
            versions: vec![(0usize, v.build())],
            queries: vec![],
        }, "has no maximum length");
    }

    #[test]
    fn test_unbounded_unique_good() {
        std::env::set_var("OUT_DIR", std::env::temp_dir().join("good_ormning_unit_tests"));
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        bananna.unique_index("bananna_hizat", &[&hizat]);
        generate(GenerateArgs {
            db_name: Some("test_unbounded_unique_good".to_string()),
            default_time_library: None,
            versions: vec![(0usize, v.build())],
            queries: vec![],
        }).unwrap();
    }

    #[test]
    fn test_update_res_count_bad() {
        let v = Version::new();
        let bananna = v.table("bananna");
        let hizat = bananna.field("hizat", field_str().build());
        let mut q = new_update(&bananna, vec![(hizat.clone(), Expr::LitString("x".into()))]).build_query("x");
        q.res_count = QueryResCount::One;
        assert_generate_err(GenerateArgs {
            db_name: Some("test_update_res_count_bad".to_string()),
            default_time_library: None,
            versions: vec![(0usize, v.build())],
            queries: vec![q],
        }, "doesn't support returning values from updates");
    }

    #[test]
    fn test_unsafe_expressions_bad() {
        use {
            super::query::{
                expr::PrefixOp,
                helpers::{
                    fn_coalesce,
                    fn_sum,
                    fn_values,
                },
                select::{
                    Join,
                    JoinSource,
                    JoinType,
                    NamedSelectSource,
                },
            },
            good_ormning_core::mariadb::types::{
                Type,
                type_decimal,
            },
        };

        let v = Version::new();
        let bananna = v.table("bananna");
        let n = bananna.field("n", field_i32().build());
        let s = bananna.field("s", field_str().build());
        let version = v.build();
        let field = |f: &super::FieldHandle| Box::new(Expr::Field(f.to_ref()));
        let today = || Box::new(fn_fixed("curdate", vec![], SimpleSimpleType::AnyDate, FnOpt::Never));
        let interval = || Box::new(Expr::Interval {
            value: Box::new(Expr::LitAnyInt(1)),
            unit: "day".to_string(),
        });
        let select = |e: Expr| new_select(&bananna).return_named("x", e).build_query("x", QueryResCount::Many);
        let cases: Vec<(&str, super::Query, &str)> = vec![
            ("date_minus_date", select(Expr::BinOp {
                left: today(),
                op: BinOp::Minus,
                right: today(),
            }), "needs numbers"),
            ("string_plus_string", select(Expr::BinOp {
                left: field(&s),
                op: BinOp::Plus,
                right: field(&s),
            }), "needs numbers"),
            ("negate_string", select(Expr::PrefixOp {
                op: PrefixOp::Minus,
                right: field(&s),
            }), "Negation needs a number"),
            ("sum_string", select(fn_sum(Expr::Field(s.to_ref()))), "sum needs a number"),
            ("cast_decimal_default", select(Expr::Cast {
                expr: field(&n),
                type_: type_decimal().build(),
                type_args: vec![],
            }), "needs a precision"),
            ("is_param", select(Expr::BinOp {
                left: field(&n),
                op: BinOp::Is,
                right: Box::new(Expr::Param {
                    name: "p".to_string(),
                    type_: Type {
                        opt: true,
                        type_: good_ormning_core::mariadb::types::SimpleType {
                            custom: None,
                            type_: SimpleSimpleType::Bool,
                        },
                    },
                }),
            }), "after `is`"),
            ("in_empty", select(Expr::BinOp {
                left: field(&n),
                op: BinOp::In,
                right: Box::new(Expr::LitArray(vec![])),
            }), "empty lists"),
            ("list_in_call", select(Expr::BinOp {
                left: field(&n),
                op: BinOp::Equals,
                right: Box::new(
                    fn_coalesce(vec![Expr::LitArray(vec![Expr::LitAnyInt(1), Expr::LitAnyInt(2)])]),
                ),
            }), "doesn't have arrays"),
            ("interval_left_of_minus", select(Expr::BinOp {
                left: interval(),
                op: BinOp::Minus,
                right: today(),
            }), "only allows `interval`"),
            ("interval_multiply", select(Expr::BinOp {
                left: today(),
                op: BinOp::Multiply,
                right: interval(),
            }), "only allows `interval`"),
            ("interval_on_int", select(Expr::BinOp {
                left: field(&n),
                op: BinOp::Plus,
                right: interval(),
            }), "Interval arithmetic needs a date"),
            ("values_in_select", select(fn_values(Expr::Field(n.to_ref()))), "on duplicate key update"),
            ("float_nan", select(Expr::LitF64(f64::NAN)), "NaN"),
            ("list_param_outside_in", select(Expr::BinOp {
                left: field(&n),
                op: BinOp::Equals,
                right: Box::new(Expr::ParamList {
                    name: "ids".to_string(),
                    types: vec![type_i64().build()],
                }),
            }), "List parameters can only be used"),
            ("list_param_tuple", select(Expr::BinOp {
                left: Box::new(Expr::LitArray(vec![Expr::Field(n.to_ref()), Expr::Field(n.to_ref())])),
                op: BinOp::In,
                right: Box::new(Expr::ParamList {
                    name: "ids".to_string(),
                    types: vec![type_i64().build()],
                }),
            }), "values per element"),
            ("list_param_tuple_types", select(Expr::BinOp {
                left: Box::new(Expr::LitArray(vec![Expr::Field(n.to_ref()), Expr::Field(s.to_ref())])),
                op: BinOp::In,
                right: Box::new(Expr::ParamList {
                    name: "pairs".to_string(),
                    types: vec![type_i64().build(), type_i64().build()],
                }),
            }), "types don't match"),
            ("list_param_as_scalar", select(Expr::BinOp {
                left: Box::new(Expr::BinOp {
                    left: field(&n),
                    op: BinOp::In,
                    right: Box::new(Expr::ParamList {
                        name: "ids".to_string(),
                        types: vec![type_i64().build()],
                    }),
                }),
                op: BinOp::And,
                right: Box::new(Expr::BinOp {
                    left: field(&n),
                    op: BinOp::Equals,
                    right: Box::new(Expr::Param {
                        name: "ids".to_string(),
                        type_: type_i64().build(),
                    }),
                }),
            }), "both as a list and as a single value"),
            (
                "insert_ignore_returning_one",
                new_insert(
                    &bananna,
                    vec![(n.clone(), Expr::LitAnyInt(1)), (s.clone(), Expr::LitString("a".into()))],
                )
                    .ignore()
                    .return_field(&n)
                    .build_query("x", QueryResCount::One),
                "must be optional",
            ),
            ("left_join_without_on", new_select(&bananna).join(Join {
                condition: None,
                source: Box::new(NamedSelectSource {
                    alias: Some("other".to_string()),
                    source: JoinSource::Table(bananna.to_ref()),
                }),
                type_: JoinType::Left,
            }).return_field(&n).build_query("x", QueryResCount::Many), "requires `on` or `using`"),
        ];
        for (name, query, err_text) in cases {
            assert_generate_err(GenerateArgs {
                db_name: Some(format!("test_unsafe_expressions_bad_{}", name)),
                default_time_library: None,
                versions: vec![(0usize, version.clone())],
                queries: vec![query],
            }, err_text);
        }
    }

    #[test]
    fn test_select_nothing_bad() {
        let v = Version::new();
        let bananna = v.table("bananna");
        bananna.field("hizat", field_str().build());
        assert_generate_err(GenerateArgs {
            db_name: Some("test_select_nothing_bad".to_string()),
            default_time_library: None,
            versions: vec![(0usize, v.build())],
            queries: vec![new_select(&bananna).build_query("x", QueryResCount::None)],
        }, "Select must have at least one output");
    }
}

use {
    convert_case::{
        Case,
        Casing,
    },
    good_ormning_core::{
        mariadb::{
            graph::utils::MariadbMigrateCtx,
            query::utils::{
                MariadbFieldInfo,
                MariadbTableInfo,
            },
            schema::{
                constraint::{
                    ConstraintType,
                    ForeignKeyAction,
                },
                field::{
                    Field,
                    FieldRef,
                    TypeVariant,
                },
                table::TableRef,
            },
            types::{
                SimpleSimpleType,
                Temporal,
                temporal,
            },
        },
        utils::Errs,
    },
    quote::{
        format_ident,
        quote,
    },
    std::{
        collections::{
            HashMap,
            HashSet,
        },
        env,
        fs,
        path::Path,
    },
};
pub use {
    good_ormning_core::mariadb::*,
    good_ormning_macros::{
        good_query_many_mariadb as good_query_many,
        good_query_one_mariadb as good_query_one,
        good_query_opt_mariadb as good_query_opt,
        good_query_mariadb as good_query,
    },
};

pub fn generate(args: GenerateArgs) -> Result<(), Vec<String>> {
    let db_name = args.db_name.as_deref().unwrap_or(good_ormning_core::utils::DEFAULT_DB_NAME);
    let out_dir = env::var("OUT_DIR").map_err(|e| vec![format!("OUT_DIR not set: {:?}", e)])?;
    let out_dir = Path::new(&out_dir);
    let output = out_dir.join(good_ormning_core::utils::rs_file_name(db_name));
    let json_dir = out_dir.join("good_ormning");
    if let Err(e) = fs::create_dir_all(&json_dir) {
        return Err(vec![format!("Error creating directory {:?}: {:?}", json_dir, e)]);
    }
    let json_path = json_dir.join(good_ormning_core::utils::json_file_name("mariadb", db_name));
    if args.versions.is_empty() {
        return Err(vec!["No versions provided".to_string()]);
    }
    {
        let mut versions_map: HashMap<usize, Version> = HashMap::new();
        for (version_i, version) in args.versions.iter() {
            if versions_map.insert(*version_i, version.clone()).is_some() {
                return Err(vec![format!("Version {} was provided multiple times", version_i)]);
            }
        }
        let schema_file = good_ormning_core::utils::SchemaFile {
            default_time_library: args.default_time_library,
            versions: versions_map,
        };
        if let Err(e) = fs::write(&json_path, serde_json::to_string(&schema_file).unwrap()) {
            return Err(vec![format!("Error writing schema file {:?}: {:?}", json_path, e)]);
        }
    }
    let mut errs = Errs::new();
    let mut migrations = vec![];
    let mut prev_version: Option<Version> = None;
    let mut prev_version_i: Option<i64> = None;
    let mut field_lookup: HashMap<TableRef, MariadbTableInfo> = HashMap::new();
    for (version_i, version) in &args.versions {
        let path = rpds::vector![format!("Migration to {}", version_i)];
        let mut migration = vec![];
        field_lookup.clear();
        for (table_id, table) in &version.tables {
            let mut fields: HashMap<FieldRef, MariadbFieldInfo> = HashMap::new();
            for (field_id, field) in &table.fields {
                fields.insert(FieldRef {
                    table_id: table_id.clone(),
                    field_id: field_id.clone(),
                }, MariadbFieldInfo {
                    sql_name: field.id.clone(),
                    type_: field.type_.type_.clone(),
                    has_default: field.type_.auto_increment || field.type_.default.is_some(),
                });
            }
            field_lookup.insert(TableRef(table_id.clone()), MariadbTableInfo {
                sql_name: table.id.clone(),
                fields: fields,
            });
        }
        let version_i = *version_i as i64;
        if let Some(i) = prev_version_i {
            if version_i != i as i64 + 1 {
                errs.err(
                    &path,
                    format!(
                        "Version numbers are not consecutive ({} to {}) - was an intermediate version deleted?",
                        i,
                        version_i
                    ),
                );
            }
        }
        for (table_id, table) in &version.tables {
            let path = path.push_back(format!("Table {}", table_id));
            let unbounded = |field: &Field| match field.type_.variant {
                Some(
                    TypeVariant::TinyText |
                    TypeVariant::Text |
                    TypeVariant::MediumText |
                    TypeVariant::TinyBlob |
                    TypeVariant::Blob |
                    TypeVariant::MediumBlob,
                ) => true,
                Some(
                    TypeVariant::Char |
                    TypeVariant::Binary |
                    TypeVariant::Mediumint |
                    TypeVariant::Timestamp |
                    TypeVariant::Year,
                ) => false,
                None => matches!(field.type_.type_.type_.type_, SimpleSimpleType::String | SimpleSimpleType::Bytes) &&
                    field.type_.max_len.is_none(),
            };
            let key_fields = |fields: Vec<&String>, what: &str| {
                for f in fields {
                    let Some(field) = table.fields.get(f) else {
                        continue;
                    };
                    if unbounded(field) {
                        errs.err(
                            &path,
                            format!(
                                "Field {} is used in {} but has no maximum length; MariaDB needs string and bytes fields to have a maximum length (`max_len`) to be used in keys",
                                f,
                                what
                            ),
                        );
                    }
                }
            };
            let mut auto_fields = vec![];
            for (field_id, field) in &table.fields {
                let is_time =
                    matches!(temporal(&field.type_.type_.type_.type_), Some(Temporal::Datetime | Temporal::Time));
                let is_decimal = field.type_.type_.type_.type_ == SimpleSimpleType::Decimal;
                if let Some(precision) = field.type_.precision {
                    if is_time && precision > 6 {
                        errs.err(
                            &path,
                            format!(
                                "Field {} has precision {}, but MariaDB's maximum for datetimes and times is 6",
                                field_id,
                                precision
                            ),
                        );
                    } else if is_decimal && (precision < 1 || precision > 65) {
                        errs.err(
                            &path,
                            format!(
                                "Field {} has precision {}, but MariaDB decimals have a precision from 1 to 65",
                                field_id,
                                precision
                            ),
                        );
                    } else if !is_time && !is_decimal {
                        errs.err(
                            &path,
                            format!("Field {} has a precision but isn't a datetime, time, or decimal field", field_id),
                        );
                    }
                }
                if let Some(scale) = field.type_.scale {
                    if !is_decimal {
                        errs.err(&path, format!("Field {} has a scale but isn't a decimal field", field_id));
                    } else {
                        match field.type_.precision {
                            Some(precision) => {
                                if scale > 38 || scale > precision {
                                    errs.err(
                                        &path,
                                        format!(
                                            "Field {} has scale {}, but MariaDB decimal scales must be at most 38 and at most the precision",
                                            field_id,
                                            scale
                                        ),
                                    );
                                }
                            },
                            None => {
                                errs.err(&path, format!("Field {} has a scale but no precision", field_id));
                            },
                        }
                    }
                }
                let base = &field.type_.type_.type_.type_;
                let is_int =
                    matches!(
                        base,
                        SimpleSimpleType::I8 | SimpleSimpleType::I16 | SimpleSimpleType::I32 | SimpleSimpleType::I64 |
                            SimpleSimpleType::U8 |
                            SimpleSimpleType::U16 |
                            SimpleSimpleType::U32 |
                            SimpleSimpleType::U64
                    );
                if field.type_.auto_increment {
                    if !is_int {
                        errs.err(&path, format!("Field {} is auto increment but isn't an integer field", field_id));
                    }
                    if field.type_.type_.opt {
                        errs.err(&path, format!("Field {} is auto increment but optional", field_id));
                    }
                    if field.type_.default.is_some() {
                        errs.err(&path, format!("Field {} is auto increment but has a default", field_id));
                    }
                    auto_fields.push(field_id.clone());
                }
                let variant_ok = match &field.type_.variant {
                    None => true,
                    Some(TypeVariant::Mediumint) => matches!(base, SimpleSimpleType::I32 | SimpleSimpleType::U32),
                    Some(TypeVariant::Year) => *base == SimpleSimpleType::U16,
                    Some(TypeVariant::Timestamp) => temporal(base) == Some(Temporal::Datetime),
                    Some(TypeVariant::Char | TypeVariant::TinyText | TypeVariant::Text | TypeVariant::MediumText) => {
                        *base == SimpleSimpleType::String
                    },
                    Some(
                        TypeVariant::Binary | TypeVariant::TinyBlob | TypeVariant::Blob | TypeVariant::MediumBlob,
                    ) => {
                        *base == SimpleSimpleType::Bytes
                    },
                };
                if !variant_ok {
                    errs.err(
                        &path,
                        format!(
                            "Field {} has variant {:?} which doesn't match its type",
                            field_id,
                            field.type_.variant
                        ),
                    );
                }
                match &field.type_.variant {
                    Some(TypeVariant::Char | TypeVariant::Binary) => {
                        if field.type_.max_len.map(|l| l > 255).unwrap_or(false) {
                            errs.err(
                                &path,
                                format!("Field {} is fixed length, which MariaDB limits to 255", field_id),
                            );
                        }
                    },
                    None => {
                        if field.type_.max_len.is_some() &&
                            !matches!(base, SimpleSimpleType::String | SimpleSimpleType::Bytes) {
                            errs.err(
                                &path,
                                format!("Field {} has a max length but isn't a string or bytes field", field_id),
                            );
                        }
                    },
                    _ => {
                        if field.type_.max_len.is_some() {
                            errs.err(
                                &path,
                                format!(
                                    "Field {} has a max length, but its type ({:?}) doesn't take one",
                                    field_id,
                                    field.type_.variant
                                ),
                            );
                        }
                    },
                }
            }
            if auto_fields.len() > 1 {
                errs.err(
                    &path,
                    format!("MariaDB only allows one auto increment field per table, but this has {:?}", auto_fields),
                );
            }
            let mut keyed_first = HashSet::new();
            for c in table.constraints.values() {
                match &c.type_ {
                    ConstraintType::PrimaryKey(x) => {
                        key_fields(x.fields.iter().collect(), &format!("primary key {}", c.id));
                        if let Some(f) = x.fields.first() {
                            keyed_first.insert(f.clone());
                        }
                    },
                    ConstraintType::ForeignKey(x) => {
                        key_fields(x.fields.iter().map(|(l, _)| l).collect(), &format!("foreign key {}", c.id));
                        if x.on_delete == Some(ForeignKeyAction::SetNull) ||
                            x.on_update == Some(ForeignKeyAction::SetNull) {
                            for (l, _) in &x.fields {
                                if table.fields.get(l).map(|f| !f.type_.type_.opt).unwrap_or(false) {
                                    errs.err(
                                        &path,
                                        format!(
                                            "Foreign key {} sets field {} to null, but the field isn't optional",
                                            c.id,
                                            l
                                        ),
                                    );
                                }
                            }
                        }
                        if let Some(remote_table) = version.tables.get(&x.remote_table) {
                            for (_, r) in &x.fields {
                                if let Some(field) = remote_table.fields.get(r) {
                                    if unbounded(field) {
                                        errs.err(
                                            &path,
                                            format!(
                                                "Foreign key {} references field {}.{} which has no maximum length; MariaDB needs string and bytes fields to have a maximum length (`max_len`) to be used in keys",
                                                c.id,
                                                x.remote_table,
                                                r
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                    },
                }
            }
            for i in table.indices.values() {
                if !i.unique {
                    key_fields(i.fields.iter().collect(), &format!("index {}", i.id));
                }
                if let Some(f) = i.fields.first() {
                    keyed_first.insert(f.clone());
                }
            }
            for f in &auto_fields {
                if !keyed_first.contains(f) {
                    errs.err(
                        &path,
                        format!(
                            "MariaDB requires auto increment fields to be the first field in the primary key or an index, but {} isn't",
                            f
                        ),
                    );
                }
            }
        }
        {
            let mut table_sql_names = HashMap::new();
            for (table_id, table) in &version.tables {
                table_sql_names.insert(table_id.clone(), table.id.clone());
            }
            let mut state = MariadbMigrateCtx::new(errs.clone(), table_sql_names, version.clone());
            let current_nodes = version.to_migrate_nodes();
            let prev_nodes = prev_version.take().map(|s| s.to_migrate_nodes());
            good_ormning_core::graphmigrate::migrate(&mut state, prev_nodes, &current_nodes);
            migration = state.statements.clone();
            errs = state.errs.clone();
        }
        let mut hash: u64 = 0xcbf29ce484222325;
        for statement in &migration {
            for b in statement.as_bytes().iter().chain([0u8].iter()) {
                hash ^= *b as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        let hash = format!("{:016x}", hash);
        let pascal_db_name: String = db_name.to_case(Case::Pascal);
        let enum_name = format_ident!("Db{}Versions", pascal_db_name);
        let newtype_name = format_ident!("Db{}{}", pascal_db_name, version_i as usize);
        let enum_variant = format_ident!("V{}", version_i as usize);
        migrations.push(quote!{
            if version < #version_i {
                const STATEMENTS:&[&str] = &[#(#migration,) *];
                const HASH: & str = #hash;
                if step > 0 && hash != HASH {
                    return Err(
                        GoodError(
                            format!(
                                "The migration to version {} was interrupted after {} statements, but the statements in the migration have changed since then so it can't be resumed",
                                #version_i,
                                step
                            ),
                        ),
                    );
                }
                for (i, query) in STATEMENTS.iter().enumerate().skip(step as usize) {
                    conn.query_drop(*query).await.to_good_error_query(query)?;
                    let query = "update `__good_version` set `step` = ?, `hash` = ? where `rid` = 0";
                    conn.exec_drop(query, (i as i64 + 1, HASH)).await.to_good_error_query(query)?;
                }
                step = 0;
                let mut txn =
                    conn
                        .start_transaction(mysql_async::TxOpts::default())
                        .await
                        .to_good_error(|| "Failed to start transaction".to_string())?;
                if let Some(callback) = & callback {
                    let wrapper = #newtype_name(txn);
                    let mut enum_val = #enum_name::#enum_variant(wrapper);
                    callback(&mut enum_val).await?;
                    txn = match enum_val {
                        #enum_name::#enum_variant(wrapper) => wrapper.0,
                        _ => panic !("Migration callback returned wrong version enum variant"),
                    };
                }
                let query = "update `__good_version` set `version` = ?, `step` = 0, `hash` = '' where `rid` = 0";
                txn.exec_drop(query, (#version_i,)).await.to_good_error_query(query) ?;
                txn.commit().await.to_good_error(|| "Failed to commit transaction".to_string())?;
            }
        });
        prev_version = Some(version.clone());
        prev_version_i = Some(version_i);
    }
    let last_version_i = prev_version_i.unwrap() as i64;
    let pascal_db_name: String = db_name.to_case(Case::Pascal);
    let enum_name = format_ident!("Db{}Versions", pascal_db_name);
    let mut enum_variants = vec![];
    let mut db_types = vec![];
    for (version_i, _) in &args.versions {
        let newtype_name = format_ident!("Db{}{}", pascal_db_name, version_i);
        let enum_variant = format_ident!("V{}", version_i);
        enum_variants.push(quote!(#enum_variant(#newtype_name < C >)));
        db_types.push(quote!{
            pub struct #newtype_name < C: good_ormning:: runtime:: mariadb:: MariadbConnection >(pub C);
        });
    }
    let latest_newtype_name = format_ident!("Db{}{}", pascal_db_name, last_version_i as usize);
    let db_alias_name = format_ident!("Db{}", pascal_db_name);
    let db_others =
        good_ormning_core::mariadb::query::generate::generate_query_functions(
            &mut errs,
            field_lookup,
            args.queries,
            "",
            quote!(#latest_newtype_name < impl good_ormning:: runtime:: mariadb:: MariadbConnection >),
            None,
            args.default_time_library,
        );
    let tokens = quote!{
        pub const DB_NAME: & str = #db_name;
        use good_ormning::runtime::GoodError;
        use good_ormning::runtime::ToGoodError;
        use good_ormning::runtime::mariadb::mysql_async;
        use good_ormning::runtime::mariadb::mysql_async::prelude::Queryable;
        use good_ormning::runtime::mariadb::MariadbConnection;
        #(#db_types) * pub enum #enum_name < C: good_ormning:: runtime:: mariadb:: MariadbConnection > {
            #(#enum_variants,) *
        }
        pub use #latest_newtype_name as #db_alias_name;
        async fn init_db(db: & mut impl good_ormning:: runtime:: mariadb:: MariadbConnection) -> Result <(),
        GoodError > {
            {
                let query =
                    "create table if not exists `__good_version` (`rid` int primary key, `version` bigint not null, `step` bigint not null, `hash` varchar(16) not null)";
                db.execute(query, vec![]).await.to_good_error_query(query)?;
            }
            {
                let query =
                    "insert into `__good_version` (`rid`, `version`, `step`, `hash`) values (0, -1, 0, '') on duplicate key update `rid` = `rid`";
                db.execute(query, vec![]).await.to_good_error_query(query)?;
            }
            Ok(())
        }
        async fn migrate_locked(
            conn: & mut mysql_async:: Conn,
            callback: Option <&(
                dyn for <'a,
                'c > Fn(
                    &'a mut #enum_name < mysql_async:: Transaction <'c >>
                ) -> std:: pin:: Pin < Box < dyn std:: future:: Future < Output = Result <(),
                GoodError >> + Send + 'a >> + Send + Sync
            ) >
        ) -> Result <(),
        GoodError > {
            let query = "select `version`, `step`, `hash` from `__good_version` where `rid` = 0";
            let Some((version, mut step, hash)) = conn.query_first::<(i64, i64, String),
            _ >(query).await.to_good_error_query(query) ? else {
                return Err(GoodError("Version table is missing its row".to_string()));
            };
            if version > #last_version_i {
                return Err(
                    GoodError(
                        format!(
                            "The latest known version is {}, but the schema is at unknown version {}",
                            #last_version_i,
                            version
                        ),
                    ),
                );
            }
            #(#migrations) * Ok(())
        }
        #[
            doc =
                "(Initialize and) migrate the database to the latest schema version. Optionally takes a callback which is run after each version, so custom post-schema change code can be run. Use `good_query!` macros with the version parameter to do migrations. MariaDB can't roll back schema changes, so if a migration is interrupted the next call continues it from where it stopped."
        ] pub async fn migrate < C: good_ormning:: runtime:: mariadb:: MariadbConnection + std:: borrow:: BorrowMut < mysql_async:: Conn > + Send,
        >(
            mut db: C,
            callback: Option <&(
                dyn for <'a,
                'c > Fn(
                    &'a mut #enum_name < mysql_async:: Transaction <'c >>
                ) -> std:: pin:: Pin < Box < dyn std:: future:: Future < Output = Result <(),
                GoodError >> + Send + 'a >> + Send + Sync
            ) >
        ) -> Result <#latest_newtype_name < C >,
        GoodError > {
            {
                let conn = std::borrow::BorrowMut::<mysql_async::Conn>::borrow_mut(&mut db);
                init_db(conn).await?;
                let query = "select get_lock(concat('good_ormning_', md5(database())), 31536000)";
                let locked: Option<Option<i64>> = conn.query_first(query).await.to_good_error_query(query)?;
                if locked != Some(Some(1)) {
                    return Err(GoodError(format!("Failed to acquire the migration lock, got {:?}", locked)));
                }
                let res = migrate_locked(conn, callback).await;
                let query = "select release_lock(concat('good_ormning_', md5(database())))";
                let release_res = conn.query_drop(query).await.to_good_error_query(query);
                res?;
                release_res?;
            }
            Ok(#latest_newtype_name(db))
        }
        pub async fn get_schema_version(
            db: & mut impl good_ormning:: runtime:: mariadb:: MariadbConnection
        ) -> Result < Option < i64 >,
        GoodError > {
            init_db(db).await?;
            let query = "select `version` from `__good_version` where `rid` = 0";
            let res = db.query(query, vec![]).await.to_good_error_query(query)?;
            if let Some(mut r) = res.into_iter().next() {
                let x: i64 = good_ormning::runtime::mariadb::get(&mut r, 0)?;
                if x == -1 {
                    return Ok(None);
                } else {
                    return Ok(Some(x));
                }
            }
            Ok(None)
        }
        #(#db_others) *
    };
    match genemichaels_lib::format_str(&tokens.to_string(), &genemichaels_lib::FormatConfig::default()) {
        Ok(src) => {
            match fs::write(&output, src.rendered.as_bytes()) {
                Ok(_) => { },
                Err(e) => errs.err(
                    &rpds::vector![],
                    format!("Failed to write generated code to {:?}: {:?}", output, e),
                ),
            };
        },
        Err(e) => {
            errs.err(&rpds::vector![], format!("Error formatting generated code: {:?}\n{}", e, tokens));
        },
    };
    errs.raise()?;
    return Ok(());
}

pub struct GenerateArgs {
    pub db_name: Option<String>,
    pub queries: Vec<Query>,
    pub versions: Vec<(usize, Version)>,
    pub default_time_library: Option<good_ormning_core::utils::TimeLibrary>,
}

impl Default for GenerateArgs {
    fn default() -> Self {
        return Self {
            db_name: None,
            default_time_library: None,
            versions: vec![],
            queries: vec![],
        };
    }
}
