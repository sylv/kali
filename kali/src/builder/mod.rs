use super::{builder::expr::Expr, builder::ordering::ColumnOrdering};
use crate::column::Column;
use std::marker::PhantomData;
use value::Value;

pub mod expr;
pub mod ordering;
pub mod value;

pub struct Select;
pub struct Insert;
pub struct Update;
pub struct Delete;
pub struct OnConflicted;

#[derive(PartialEq, Eq)]
enum QueryKind {
    Select,
    Insert,
    Update,
    Delete,
}

macro_rules! generic_builder {
    ($name:ident, $kind:expr) => {
        pub fn $name(table: &'a str) -> Self {
            Self {
                table,
                kind: $kind,
                columns: None,
                filter: None,
                limit: None,
                offset: None,
                values: None,
                set: None,
                on_conflict: None,
                returning: None,
                order_by: Vec::new(),
                _type: PhantomData,
            }
        }
    };
}
pub trait Veccable<T> {
    fn to_vec(self) -> Vec<T>;
}

impl<T> Veccable<T> for T {
    fn to_vec(self) -> Vec<T> {
        vec![self]
    }
}

impl<T: Clone> Veccable<T> for (T, T) {
    fn to_vec(self) -> Vec<T> {
        vec![self.0.clone(), self.1.clone()]
    }
}

impl<T: Clone> Veccable<T> for (T, T, T) {
    fn to_vec(self) -> Vec<T> {
        vec![self.0.clone(), self.1.clone(), self.2.clone()]
    }
}

pub enum OnConflict {
    Ignore,
    Update,
}

pub struct QueryBuilder<'a, S, C: Column> {
    table: &'a str,
    kind: QueryKind,
    columns: Option<&'a [C]>,
    values: Option<Vec<(C, Value)>>,
    set: Option<Vec<(C, Value)>>,
    on_conflict: Option<(Vec<C>, OnConflict)>,
    returning: Option<&'a [C]>,
    filter: Option<Expr<'a, C>>,
    limit: Option<i64>,
    offset: Option<i64>,
    order_by: Vec<ColumnOrdering<C>>,
    _type: PhantomData<S>,
}

impl<'a, C: Column> QueryBuilder<'a, Select, C> {
    generic_builder!(select_from, QueryKind::Select);

    pub fn columns(mut self, columns: &'a [C]) -> Self {
        self.columns = Some(columns);
        self
    }

    pub fn order_by(mut self, ordering: ColumnOrdering<C>) -> Self {
        self.order_by.push(ordering);
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }
}

impl<'a, C: Column> QueryBuilder<'a, Insert, C> {
    generic_builder!(insert_into, QueryKind::Insert);

    pub fn value<V: Into<Value> + 'a>(mut self, col: C, value: V) -> Self {
        if let Some(values) = &mut self.values {
            values.push((col, value.into()));
        } else {
            self.values = Some(vec![(col, value.into())]);
        }

        self
    }

    pub fn on_conflict<CV: Veccable<C> + 'a>(
        mut self,
        columns: CV,
        on_conflict: OnConflict,
    ) -> QueryBuilder<'a, OnConflicted, C> {
        self.on_conflict = Some((columns.to_vec(), on_conflict));
        QueryBuilder {
            table: self.table,
            kind: self.kind,
            columns: self.columns,
            filter: self.filter,
            limit: self.limit,
            offset: self.offset,
            values: self.values,
            set: self.set,
            on_conflict: self.on_conflict,
            returning: self.returning,
            order_by: self.order_by,
            _type: PhantomData,
        }
    }

    pub fn returning(mut self, columns: &'a [C]) -> Self {
        self.returning = Some(columns);
        self
    }
}

impl<'a, C: Column> QueryBuilder<'a, OnConflicted, C> {
    /// Only valid after `on_conflict`
    pub fn set<V: Into<Value>>(mut self, column: C, value: V) -> Self {
        if !self.on_conflict.is_some() {
            // todo: should use typestate pattern to prevent this at compile time
            panic!("Cannot set value without an ON CONFLICT clause");
        }

        let value = value.into();
        if let Some(set) = &mut self.set {
            set.push((column, value));
        } else {
            self.set = Some(vec![(column, value)]);
        }
        self
    }

    pub fn returning(mut self, columns: &'a [C]) -> Self {
        self.returning = Some(columns);
        self
    }
}

impl<'a, C: Column> QueryBuilder<'a, Update, C> {
    generic_builder!(update, QueryKind::Update);

    pub fn set<V: Into<Value>>(mut self, column: C, value: V) -> Self {
        let value = value.into();
        if let Some(set) = &mut self.set {
            set.push((column, value));
        } else {
            self.set = Some(vec![(column, value)]);
        }
        self
    }

    pub fn returning(mut self, columns: &'a [C]) -> Self {
        self.returning = Some(columns);
        self
    }
}

impl<'a, C: Column> QueryBuilder<'a, Delete, C> {
    generic_builder!(delete_from, QueryKind::Delete);

    pub fn returning(mut self, columns: &'a [C]) -> Self {
        self.returning = Some(columns);
        self
    }
}

macro_rules! assert_kind {
    ($self:ident, $kind:pat) => {
        match $self.kind {
            $kind => {}
            _ => panic!("Wrong query kind, expected {}", stringify!($kind)),
        }
    };
}

impl<'a, T, C: Column> QueryBuilder<'a, T, C> {
    pub fn filter(mut self, expr: Expr<'a, C>) -> Self {
        if let Some(where_clause) = self.filter {
            self.filter = Some(where_clause.and(expr));
        } else {
            self.filter = Some(expr);
        }

        self
    }

    pub fn to_sql(self) -> (String, Vec<Value>) {
        let mut values = Vec::new();
        let mut query = match self.kind {
            QueryKind::Select => "SELECT ".to_string(),
            QueryKind::Insert => "INSERT INTO ".to_string(),
            QueryKind::Update => "UPDATE ".to_string(),
            QueryKind::Delete => "DELETE ".to_string(),
        };

        if let Some(columns) = self.columns {
            assert_kind!(self, QueryKind::Select | QueryKind::Insert);
            push_separated(&mut query, columns.iter(), |query, column| {
                column.write(query);
            });
        } else if matches!(self.kind, QueryKind::Select) {
            query.push_str("*");
        }

        match self.kind {
            QueryKind::Select => {
                query.push_str(" FROM ");
                query.push_str(self.table);
            }
            QueryKind::Insert => {
                query.push_str(self.table);
            }
            QueryKind::Update => {
                query.push_str(self.table);
            }
            QueryKind::Delete => {
                query.push_str(" FROM ");
                query.push_str(self.table);
            }
        }

        if let Some(sql_values) = self.values {
            assert_kind!(self, QueryKind::Insert);
            query.push_str(" (");
            push_separated(&mut query, sql_values.iter(), |query, (column, _)| {
                column.write(query);
            });
            query.push_str(") VALUES (");
            push_separated(&mut query, sql_values.into_iter(), |query, (_, value)| {
                query.push_str("?");
                values.push(value);
            });
            query.push_str(")");
        }

        // match is necessary to prevent .set from being consumed
        match self.kind {
            QueryKind::Update => {
                if let Some(set) = self.set {
                    assert_kind!(self, QueryKind::Update);
                    query.push_str(" SET ");
                    push_separated(&mut query, set.into_iter(), |query, (column, value)| {
                        column.write(query);
                        query.push_str(" = ?");
                        values.push(value);
                    });
                }
            }
            QueryKind::Insert => {
                if let Some(on_conflict) = self.on_conflict {
                    assert_kind!(self, QueryKind::Insert);
                    query.push_str(" ON CONFLICT (");
                    push_separated(&mut query, on_conflict.0.iter(), |query, column| {
                        column.write(query);
                    });
                    query.push_str(") ");
                    match on_conflict.1 {
                        OnConflict::Ignore => query.push_str("DO NOTHING"),
                        OnConflict::Update => {
                            query.push_str("DO UPDATE SET ");
                            // todo: should be enforced with typeset
                            let set = self.set.expect("SET clause is required for UPDATE");
                            push_separated(
                                &mut query,
                                set.into_iter(),
                                |query, (column, value)| {
                                    column.write(query);
                                    query.push_str(" = ?");
                                    values.push(value);
                                },
                            );
                        }
                    }
                }
            }
            _ => {
                assert!(
                    self.set.is_none(),
                    "SET clause is not allowed in this query kind"
                );
            }
        }

        if let Some(where_clause) = self.filter {
            assert_kind!(
                self,
                QueryKind::Select | QueryKind::Update | QueryKind::Delete
            );
            query.push_str(" WHERE ");
            where_clause.write(&mut query, &mut values);
        }

        if !self.order_by.is_empty() {
            assert_kind!(self, QueryKind::Select);
            query.push_str(" ORDER BY ");
            push_separated(&mut query, self.order_by.iter(), |query, ordering| {
                ordering.write(query);
            });
        }

        if let Some(limit) = self.limit {
            assert_kind!(self, QueryKind::Select);
            query.push_str(" LIMIT ");
            query.push_str(&limit.to_string());
        }

        if let Some(offset) = self.offset {
            assert_kind!(self, QueryKind::Select);
            query.push_str(" OFFSET ");
            query.push_str(&offset.to_string());
        }

        if let Some(returning) = self.returning {
            assert_kind!(
                self,
                QueryKind::Insert | QueryKind::Update | QueryKind::Delete
            );
            query.push_str(" RETURNING ");
            push_separated(&mut query, returning.iter(), |query, column| {
                column.write(query)
            });
        }

        (query, values)
    }

    pub async fn fetch_one<'e, E, S>(mut self, executor: E) -> Result<S, sqlx::Error>
    where
        E: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
        S: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
    {
        if self.limit.is_none() && self.kind == QueryKind::Select {
            self.limit = Some(1);
        }

        let (query, values) = self.to_sql();
        let mut query = sqlx::query(&query);
        for value in values.into_iter() {
            query = value.bind_to(query);
        }

        query
            .fetch_one(executor)
            .await
            .and_then(|row| S::from_row(&row))
    }

    pub async fn fetch_optional<'e, 'c: 'e, E, S>(
        mut self,
        executor: E,
    ) -> Result<Option<S>, sqlx::Error>
    where
        E: 'e + sqlx::Executor<'c, Database = sqlx::Sqlite>,
        S: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
    {
        if self.limit.is_none() && self.kind == QueryKind::Select {
            self.limit = Some(1);
        }

        let (query, values) = self.to_sql();
        let mut query = sqlx::query(&query);
        for value in values.into_iter() {
            query = value.bind_to(query);
        }

        query
            .fetch_optional(executor)
            .await
            .and_then(|row| match row {
                Some(row) => S::from_row(&row).map(Some),
                None => Ok(None),
            })
    }

    pub async fn fetch_all<'e, 'c: 'e, E, S>(self, executor: E) -> Result<Vec<S>, sqlx::Error>
    where
        E: 'e + sqlx::Executor<'c, Database = sqlx::Sqlite>,
        S: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
    {
        let (query, values) = self.to_sql();
        let mut query = sqlx::query(&query);
        for value in values.into_iter() {
            query = value.bind_to(query);
        }

        query.fetch_all(executor).await.and_then(|rows| {
            rows.into_iter()
                .map(|row| S::from_row(&row))
                .collect::<Result<Vec<_>, _>>()
        })
    }

    pub async fn execute<'e, 'c: 'e, E>(
        self,
        executor: E,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error>
    where
        E: 'e + sqlx::Executor<'c, Database = sqlx::Sqlite>,
    {
        let (query, values) = self.to_sql();
        let mut query = sqlx::query(&query);
        for value in values.into_iter() {
            query = value.bind_to(query);
        }

        query.execute(executor).await
    }
}

fn push_separated<I, F>(query: &mut String, iter: I, mut cb: F)
where
    I: Iterator,
    F: FnMut(&mut String, I::Item),
{
    let mut first = true;
    for item in iter {
        if !first {
            query.push_str(", ");
        } else {
            first = false;
        }
        cb(query, item);
    }
}
