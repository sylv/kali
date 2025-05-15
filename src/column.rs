use crate::builder::{expr::Expr, ordering::ColumnOrdering, value::Value};

pub trait Column: Copy + Send + Sync {
    fn to_col_name(&self) -> &str;

    fn write(&self, f: &mut String) {
        f.push('"');
        f.push_str(self.to_col_name());
        f.push('"');
    }
}

pub trait ColumnExpr<'a, C: Column> {
    fn eq<V: Into<Value>>(self, value: V) -> Expr<'a, C>;
    fn gt<V: Into<Value>>(self, value: V) -> Expr<'a, C>;
    fn lt<V: Into<Value>>(self, value: V) -> Expr<'a, C>;
    fn like<V: Into<Value>>(self, value: V) -> Expr<'a, C>;
    fn in_list<V: Into<Value>>(self, values: Vec<V>) -> Expr<'a, C>;
    fn is_null(self) -> Expr<'a, C>;

    fn asc(self) -> ColumnOrdering<C>;
    fn desc(self) -> ColumnOrdering<C>;
    fn asc_nulls_first(self) -> ColumnOrdering<C>;
    fn asc_nulls_last(self) -> ColumnOrdering<C>;
    fn desc_nulls_first(self) -> ColumnOrdering<C>;
    fn desc_nulls_last(self) -> ColumnOrdering<C>;
}

impl<'a, C: Column> ColumnExpr<'a, C> for C {
    fn eq<V: Into<Value>>(self, value: V) -> Expr<'a, C> {
        Expr::Equal(self, value.into())
    }

    fn gt<V: Into<Value>>(self, value: V) -> Expr<'a, C> {
        Expr::Gt(self, value.into())
    }

    fn lt<V: Into<Value>>(self, value: V) -> Expr<'a, C> {
        Expr::Lt(self, value.into())
    }

    fn like<V: Into<Value>>(self, value: V) -> Expr<'a, C> {
        Expr::Like(self, value.into())
    }

    fn in_list<V: Into<Value>>(self, values: Vec<V>) -> Expr<'a, C> {
        let values = values.into_iter().map(|v| v.into()).collect();
        Expr::In(self, values)
    }

    fn is_null(self) -> Expr<'a, C> {
        Expr::Equal(self, Value::Null)
    }

    fn asc(self) -> ColumnOrdering<C> {
        ColumnOrdering::Asc(self)
    }

    fn desc(self) -> ColumnOrdering<C> {
        ColumnOrdering::Desc(self)
    }

    fn asc_nulls_first(self) -> ColumnOrdering<C> {
        ColumnOrdering::AscNullsFirst(self)
    }

    fn asc_nulls_last(self) -> ColumnOrdering<C> {
        ColumnOrdering::AscNullsLast(self)
    }

    fn desc_nulls_first(self) -> ColumnOrdering<C> {
        ColumnOrdering::DescNullsFirst(self)
    }

    fn desc_nulls_last(self) -> ColumnOrdering<C> {
        ColumnOrdering::DescNullsLast(self)
    }
}
