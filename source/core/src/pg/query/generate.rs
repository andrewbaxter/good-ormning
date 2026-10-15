use {
    crate::{
        QueryResCount,
        pg::{
            Query,
            query::{
                expr::ExprValName,
                utils::{
                    PgQueryCtx,
                    PgTableInfo,
                },
            },
            types::{
                Type,
                to_rust_types,
            },
        },
        utils::{
            Errs,
            sanitize_ident,
        },
    },
    proc_macro2::{
        Ident,
        TokenStream,
    },
    quote::{
        ToTokens,
        format_ident,
        quote,
    },
    std::collections::HashMap,
};

pub fn generate_query_functions(
    errs: &mut Errs,
    field_lookup: HashMap<crate::pg::schema::table::TableRef, PgTableInfo>,
    queries: Vec<Query>,
    _mod_name: &str,
    db_type: TokenStream,
) -> Vec<TokenStream> {
    let mut db_others = Vec::new();
    let mut res_type_idents: HashMap<String, Ident> = HashMap::new();
    for q in queries {
        let path = rpds::vector![format!("Query {}", q.name)];
        let mut ctx = PgQueryCtx::new(errs.clone(), field_lookup.clone());
        let res = q.body.build(&mut ctx, &path, q.res_count.clone());
        let ident = format_ident!("{}", q.name);
        let q_text = res.1.to_string();
        let args = ctx.rust_args.split_off(0);
        let args_forward = ctx.query_args.split_off(0);
        *errs = ctx.errs.clone();
        drop(ctx);
        let (res_ident, res_def, unforward_res) = {
            fn convert_one_res(
                errs: &mut Errs,
                path: &rpds::Vector<String>,
                i: usize,
                k: &ExprValName,
                v: &Type,
            ) -> Option<(Ident, TokenStream, TokenStream)> {
                if k.id.is_empty() {
                    errs.err(
                        path,
                        format!("Result element {} has no name; name it using `rename` if this is intentional", i),
                    );
                    return None;
                }
                let rust_types = to_rust_types(&v.type_.type_);
                let custom_trait_ident = rust_types.custom_trait;
                let mut ident = rust_types.ret_type;
                if v.opt {
                    ident = quote!(Option < #ident >);
                }
                let mut unforward = match &v.type_.type_ {
                    crate::pg::types::SimpleSimpleType::U32 => {
                        // Stored as bigint
                        if v.opt {
                            quote!{
                                let x: Option < i64 > = r.try_get(
                                    #i
                                ).to_good_error(|| format!("Reading result {}", #i)) ?;
                                let x: #ident = match x {
                                    Some(
                                        x
                                    ) => Some(
                                        u32::try_from(x).to_good_error(|| format!("Reading result {} as u32", #i))?
                                    ),
                                    None => None,
                                };
                            }
                        } else {
                            quote!{
                                let x: i64 = r.try_get(#i).to_good_error(|| format!("Reading result {}", #i)) ?;
                                let x: #ident = u32:: try_from(
                                    x
                                ).to_good_error(|| format!("Reading result {} as u32", #i)) ?;
                            }
                        }
                    },
                    _ => quote!{
                        let x: #ident = r.try_get(#i).to_good_error(|| format!("Reading result {}", #i)) ?;
                    },
                };
                if let Some(custom) = &v.type_.custom {
                    ident = match syn::parse_str::<syn::Path>(custom) {
                        Ok(i) => i.to_token_stream(),
                        Err(e) => {
                            errs.err(
                                path,
                                format!(
                                    "Couldn't parse provided custom type name [{}] as identifier path: {:?}",
                                    custom,
                                    e
                                ),
                            );
                            return None;
                        },
                    };
                    if v.opt {
                        unforward = quote!{
                            #unforward let x = if let Some(x) = x {
                                Some(
                                    < #ident as #custom_trait_ident < #ident >>:: from_sql(
                                        x
                                    ).to_good_error(|| format!("Parsing result {}", #i)) ?
                                )
                            }
                            else {
                                None
                            };
                        };
                        ident = quote!(Option < #ident >);
                    } else {
                        unforward = quote!{
                            #unforward let x =< #ident as #custom_trait_ident < #ident >>:: from_sql(
                                x
                            ).to_good_error(|| format!("Parsing result {}", #i)) ?;
                        };
                    }
                }
                return Some((format_ident!("{}", sanitize_ident(&k.id).1), ident, quote!({
                    #unforward x
                })));
            }

            if res.0.0.len() == 1 && q.res_name.is_none() {
                let e = &res.0.0[0];
                let (_, type_ident, unforward) = match convert_one_res(errs, &path, 0, &e.0, &e.1) {
                    None => {
                        continue;
                    },
                    Some(x) => x,
                };
                (type_ident, None, unforward)
            } else {
                let mut fields = vec![];
                let mut unforward_fields = vec![];
                let mut seen_names = std::collections::HashSet::new();
                for (i, (k, v)) in res.0.0.into_iter().enumerate() {
                    let (k_ident, type_ident, unforward) = match convert_one_res(errs, &path, i, &k, &v) {
                        Some(x) => x,
                        None => continue,
                    };
                    if !seen_names.insert(k_ident.to_string()) {
                        errs.err(
                            &path,
                            format!(
                                "Result element {} has name {} which is already used by another result element; rename it using `as`",
                                i,
                                k_ident
                            ),
                        );
                        continue;
                    }
                    fields.push(quote!{
                        pub #k_ident: #type_ident
                    });
                    unforward_fields.push(quote!{
                        #k_ident: #unforward
                    });
                }
                let body = quote!({
                    #(#fields,) *
                });
                let res_type_count = res_type_idents.len();
                let (res_ident, res_def) = match res_type_idents.entry(body.to_string()) {
                    std::collections::hash_map::Entry::Occupied(e) => {
                        (e.get().clone(), None)
                    },
                    std::collections::hash_map::Entry::Vacant(e) => {
                        let ident = if let Some(name) = q.res_name {
                            format_ident!("{}", name)
                        } else {
                            format_ident!("DbRes{}", res_type_count)
                        };
                        e.insert(ident.clone());
                        let res_def = quote!(pub struct #ident #body);
                        (ident, Some(res_def))
                    },
                };
                let unforward = quote!(#res_ident {
                    #(#unforward_fields,) *
                });
                (res_ident.to_token_stream(), res_def, unforward)
            }
        };
        let db_arg = quote!(db: & mut #db_type);
        match q.res_count {
            QueryResCount::None => {
                db_others.push(quote!{
                    pub async fn #ident(#db_arg, #(#args,) *) -> Result <(),
                    GoodError > {
                        let query = #q_text;
                        db.0.execute(query, &[#(& #args_forward,) *]).await.to_good_error_query(query) ?;
                        Ok(())
                    }
                });
            },
            QueryResCount::MaybeOne => {
                if let Some(res_def) = res_def {
                    db_others.push(res_def);
                }
                db_others.push(quote!{
                    pub async fn #ident(#db_arg, #(#args,) *) -> Result < Option < #res_ident >,
                    GoodError > {
                        let query = #q_text;
                        let res = db.0.query(query, &[#(& #args_forward,) *]).await.to_good_error_query(query) ?;
                        if let Some(r) = res.first() {
                            return Ok(Some(#unforward_res));
                        }
                        Ok(None)
                    }
                });
            },
            QueryResCount::One => {
                if let Some(res_def) = res_def {
                    db_others.push(res_def);
                }
                db_others.push(quote!{
                    pub async fn #ident(#db_arg, #(#args,) *) -> Result < #res_ident,
                    GoodError > {
                        let query = #q_text;
                        let res = db.0.query(query, &[#(& #args_forward,) *]).await.to_good_error_query(query) ?;
                        if res.len() != 1 {
                            return Err(
                                GoodError(
                                    format!(
                                        "Query {} returned {} results but exactly one was expected",
                                        #q_text,
                                        res.len()
                                    ),
                                ),
                            );
                        }
                        let r = &res[0];
                        Ok(#unforward_res)
                    }
                });
            },
            QueryResCount::Many => {
                if let Some(res_def) = res_def {
                    db_others.push(res_def);
                }
                db_others.push(quote!{
                    pub async fn #ident(#db_arg, #(#args,) *) -> Result < Vec < #res_ident >,
                    GoodError > {
                        let mut out = vec![];
                        let query = #q_text;
                        let res = db.0.query(query, &[#(& #args_forward,) *]).await.to_good_error_query(query) ?;
                        for r in res {
                            out.push(#unforward_res);
                        }
                        Ok(out)
                    }
                });
            },
        }
    }
    return db_others;
}
