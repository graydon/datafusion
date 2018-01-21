use std::string::String;

use super::sql::*;
use super::rel::*;

pub struct SqlToRel {
    default_schema: Option<String>
}

impl SqlToRel {

    pub fn new() -> Self {
        SqlToRel { default_schema: None }
    }

    pub fn sql_to_rel(&self, sql: &ASTNode, tt: &TupleType) -> Result<Box<Rel>, String> {
        match sql {
            &ASTNode::SQLSelect { ref projection, ref relation, ref selection, .. } => {

                let expr : Vec<Rex> = projection.iter()
                    .map(|e| self.sql_to_rex(&e, tt) )
                    .collect::<Result<Vec<Rex>,String>>()?;

                let input = match relation {
                    &Some(ref r) => Some(self.sql_to_rel(r, tt)?),
                    &None => None
                };

                match selection {
                    &Some(ref filter_expr) => {

                        let selection_rel = match input {
                            Some(x) => {
                                let schema = x.schema().clone();
                                Rel::Selection {
                                    expr: self.sql_to_rex(&filter_expr, tt)?,
                                    input: x,
                                    schema: schema
                                }
                            },
                            _ => unimplemented!() //TODO error handling
                        };

                        Ok(Box::new(Rel::Projection {
                            expr: expr,
                            input: Some(Box::new(selection_rel)),
                            schema: TupleType { columns: vec![] } //TODO
                        }))

                    },
                    _ => Ok(Box::new(Rel::Projection {
                            expr: expr,
                            input: input,
                            schema: TupleType { columns: vec![] } //TODO
                        }))
                }

            },

            &ASTNode::SQLIdentifier { ref id, .. } => {
                Ok(Box::new(Rel::TableScan {
                    schema_name: String::from("default"),
                    table_name: id.clone(),
                    schema: TupleType { columns: vec![] } //TODO need schema meta-data now
                }))
            },



            _ => Err(format!("sql_to_rel does not support this relation: {:?}", sql))
        }
    }

    pub fn sql_to_rex(&self, sql: &ASTNode, tt: &TupleType) -> Result<Rex, String> {
        match sql {

            &ASTNode::SQLLiteralInt(n) =>
                Ok(Rex::Literal(Value::UnsignedLong(n as u64))), //TODO

            &ASTNode::SQLIdentifier { ref id, .. } => {
                match tt.columns.iter().position(|c| c.name.eq(id) ) {
                    Some(index) => Ok(Rex::TupleValue(index)),
                    None => Err(String::from("Invalid identifier"))
                }
            },

            &ASTNode::SQLBinaryExpr { ref left, ref op, ref right } => {
                //TODO: we have this implemented somewhere else already
                let operator = match op {
                    &SQLOperator::GT => Operator::Gt,
                    _ => unimplemented!()
                };
                Ok(Rex::BinaryExpr {
                    left: Box::new(self.sql_to_rex(&left, &tt)?),
                    op: operator,
                    right: Box::new(self.sql_to_rex(&right, &tt)?),
                })

            },

            _ => Err(String::from(format!("Unsupported ast node {:?}", sql)))
        }
    }

}
