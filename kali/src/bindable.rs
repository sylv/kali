use sqlx::{sqlite::SqliteArguments, Sqlite};
use std::fmt::Debug;

pub trait Bindable: Debug {
    fn bind_to<'q>(
        &'q self,
        query: sqlx::query::Query<'q, Sqlite, SqliteArguments<'q>>,
    ) -> sqlx::query::Query<'q, Sqlite, SqliteArguments<'q>>;
}

impl<'a, T: Bindable + 'a> Bindable for Option<T> {
    fn bind_to<'q>(
        &'q self,
        query: sqlx::query::Query<'q, Sqlite, SqliteArguments<'q>>,
    ) -> sqlx::query::Query<'q, Sqlite, SqliteArguments<'q>> {
        match self {
            Some(value) => value.bind_to(query),
            None => query.bind(None::<i32>),
        }
    }
}

impl<'a, T: Bindable + 'a> Bindable for Box<Option<T>> {
    fn bind_to<'q>(
        &'q self,
        query: sqlx::query::Query<'q, Sqlite, SqliteArguments<'q>>,
    ) -> sqlx::query::Query<'q, Sqlite, SqliteArguments<'q>> {
        (**self).bind_to(query)
    }
}

macro_rules! bindable {
    ($($t:ty),+) => {
        $(
            impl Bindable for $t {
                fn bind_to<'q>(
                    &'q self,
                    query: sqlx::query::Query<'q, Sqlite, SqliteArguments<'q>>,
                ) -> sqlx::query::Query<'q, Sqlite, SqliteArguments<'q>> {
                    query.bind(self)
                }
            }

            impl Bindable for &$t {
                fn bind_to<'q>(
                    &'q self,
                    query: sqlx::query::Query<'q, Sqlite, SqliteArguments<'q>>,
                ) -> sqlx::query::Query<'q, Sqlite, SqliteArguments<'q>> {
                    query.bind(*self)
                }
            }
        )+
    };
}

bindable!(bool);
bindable!(String, &str);
// u8, u16, u64 are not supported by sqlx::sqlite
bindable!(i8, i16, i32, i64, u32, f32, f64);
