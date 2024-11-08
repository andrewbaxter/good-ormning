use {
    super::{
        expr::{
            ExprType,
        },
        select_body::{
            build_select_junction,
            SelectBody,
            SelectJunction,
        },
        utils::{
            build_with,
            QueryBody,
            With,
        },
    },
    crate::{
        sqlite::{
            QueryResCount,
        },
        utils::Tokens,
    },
    std::collections::HashMap,
};

#[derive(Clone, Debug)]
pub struct Select {
    pub with: Option<With>,
    pub body: SelectBody,
    pub body_junctions: Vec<SelectJunction>,
}

impl QueryBody for Select {
    fn build(
        &self,
        ctx: &mut super::utils::SqliteQueryCtx,
        path: &rpds::Vector<String>,
        res_count: QueryResCount,
    ) -> (ExprType, Tokens) {
        let mut out = Tokens::new();
        if let Some(w) = &self.with {
            out.s(&build_with(ctx, path, w).to_string());
        }
        let body: (ExprType, Tokens) = self.body.build(ctx, &HashMap::new(), path, res_count);
        out.s(&body.1.to_string());
        out.s(&build_select_junction(ctx, path, &body.0, &self.body_junctions).to_string());
        return (body.0, out);
    }
}
