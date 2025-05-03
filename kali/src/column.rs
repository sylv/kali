use crate::{bindable::Bindable, expr::Expr};

pub trait Column: Send + Sync {
    fn raw(&self) -> &str;

    fn write(&self, f: &mut String) {
        f.push('"');
        f.push_str(self.raw());
        f.push('"');
    }
}

pub trait ColumnExpr<'a, C: Column> {
    fn eq<V: 'a + Bindable>(self, value: V) -> Expr<'a, C>;
    fn gt<V: 'a + Bindable>(self, value: V) -> Expr<'a, C>;
    fn lt<V: 'a + Bindable>(self, value: V) -> Expr<'a, C>;
    fn like<V: 'a + Bindable>(self, value: V) -> Expr<'a, C>;
    fn in_list<V: 'a + Bindable>(self, values: Vec<V>) -> Expr<'a, C>;
    fn is_null(self) -> Expr<'a, C>;
}

impl<'a, C: Column> ColumnExpr<'a, C> for C {
    fn eq<V: 'a + Bindable>(self, value: V) -> Expr<'a, C> {
        Expr::Equal(self, Box::new(value))
    }

    fn gt<V: 'a + Bindable>(self, value: V) -> Expr<'a, C> {
        Expr::Gt(self, Box::new(value))
    }

    fn lt<V: 'a + Bindable>(self, value: V) -> Expr<'a, C> {
        Expr::Lt(self, Box::new(value))
    }

    fn is_null(self) -> Expr<'a, C> {
        Expr::Equal(self, Box::new(None::<i32>))
    }

    fn like<V: 'a + Bindable>(self, value: V) -> Expr<'a, C> {
        Expr::Like(self, Box::new(value))
    }

    fn in_list<V: 'a + Bindable>(self, values: Vec<V>) -> Expr<'a, C> {
        let values = values
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Bindable + 'a>)
            .collect();
        Expr::In(self, values)
    }
}
