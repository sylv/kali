use super::value::Value;
use crate::column::Column;
use std::borrow::Cow;

#[derive(Clone)]
pub enum Expr<'a, C: Column> {
    Equal(C, Value),
    Gt(C, Value),
    Lt(C, Value),
    Like(C, Value),
    In(C, Vec<Value>),
    Raw(Cow<'a, str>),
    And(Box<Expr<'a, C>>, Box<Expr<'a, C>>),
    Or(Box<Expr<'a, C>>, Box<Expr<'a, C>>),
}

impl<'a, C: Column> Expr<'a, C> {
    pub fn and(self, other: Expr<'a, C>) -> Expr<'a, C> {
        Expr::And(Box::new(self), Box::new(other))
    }

    pub fn or(self, other: Expr<'a, C>) -> Expr<'a, C> {
        Expr::Or(Box::new(self), Box::new(other))
    }

    pub fn both(left: Expr<'a, C>, right: Expr<'a, C>) -> Expr<'a, C> {
        Expr::And(Box::new(left), Box::new(right))
    }

    pub fn either(left: Expr<'a, C>, right: Expr<'a, C>) -> Expr<'a, C> {
        Expr::Or(Box::new(left), Box::new(right))
    }

    pub(crate) fn write(self, f: &mut String, values: &mut Vec<Value>) {
        match self {
            Expr::Equal(column, value) => {
                column.write(f);
                match value {
                    Value::Null => f.push_str(" IS NULL"),
                    _ => {
                        f.push_str(" = ?");
                        values.push(value);
                    }
                }
            }
            Expr::Gt(column, value) => {
                column.write(f);
                f.push_str(" > ?");
                values.push(value);
            }
            Expr::Lt(column, value) => {
                column.write(f);
                f.push_str(" < ?");
                values.push(value);
            }
            Expr::And(left, right) => {
                f.push_str("(");
                left.write(f, values);
                f.push_str(") AND (");
                right.write(f, values);
                f.push_str(")");
            }
            Expr::Or(left, right) => {
                f.push_str("(");
                left.write(f, values);
                f.push_str(") OR (");
                right.write(f, values);
                f.push_str(")");
            }
            Expr::Raw(raw) => {
                f.push_str(&raw);
            }
            Expr::Like(column, value) => {
                column.write(f);
                f.push_str(" LIKE ?");
                values.push(value);
            }
            Expr::In(column, values_list) => {
                column.write(f);
                f.push_str(" IN (");
                let mut first = true;
                for value in values_list {
                    if !first {
                        f.push_str(", ");
                    } else {
                        first = false;
                    }
                    f.push('?');
                    values.push(value);
                }
                f.push(')');
            }
        }
    }
}
