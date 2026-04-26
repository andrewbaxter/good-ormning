use sqlparser::ast as sql;
use std::rc::Rc;
use crate::GoodQueryInput;

#[cfg(feature = "pg")]
pub mod pg {
    use super::*;
    use good_ormning::pg::{
        query::{
            expr::{
                Expr,
                BinOp,
                ComputeType,
                ExprType,
                ExprValName,
            },
            select::{
                Select,
                NamedSelectSource,
                Join,
                JoinSource,
                JoinType,
                Order,
            },
            select_body::{
                SelectBody,
                SelectJunction,
                SelectJunctionOperator,
            },
            insert::{
                Insert,
                InsertConflict,
            },
            update::Update,
            delete::Delete,
            utils::{
                Returning,
                With,
                CteBuilder,
                QueryBody,
            },
            helpers::*,
        },
        schema::{
            table::TableRef,
            field::FieldRef,
        },
        types::{
            Type,
            SimpleType,
            SimpleSimpleType,
        },
        Query,
        QueryResCount,
    };

    pub fn convert_query(input: &GoodQueryInput, statement: &sql::Statement) -> Query {
        unimplemented!()
    }
}

#[cfg(feature = "sqlite")]
pub mod sqlite {
    use super::*;
    use good_ormning::sqlite::{
        query::{
            expr::{
                Expr,
                BinOp,
                ComputeType,
                ExprType,
                Binding,
            },
            select::{
                Select,
                NamedSelectSource,
                Join,
                JoinSource,
                JoinType,
                Order,
            },
            select_body::{
                SelectBody,
                SelectJunction,
                SelectJunctionOperator,
            },
            insert::{
                Insert,
                InsertConflict,
            },
            update::Update,
            delete::Delete,
            utils::{
                Returning,
                With,
                CteBuilder,
                QueryBody,
            },
            helpers::*,
        },
        schema::{
            table::TableRef,
            field::FieldRef,
        },
        types::{
            Type,
            SimpleType,
            SimpleSimpleType,
        },
        Query,
        QueryResCount,
    };

    pub fn convert_query(input: &GoodQueryInput, statement: &sql::Statement) -> Query {
        unimplemented!()
    }
}
