use crate::{bindable::Bindable, column::Column, expr::Expr};

pub enum Ordering {
    Asc,
    Desc,
    AscNullsFirst,
    AscNullsLast,
    DescNullsFirst,
    DescNullsLast,
}

impl Ordering {
    pub fn write(&self, f: &mut String) {
        match self {
            Ordering::Asc => f.push_str("ASC"),
            Ordering::Desc => f.push_str("DESC"),
            Ordering::AscNullsFirst => f.push_str("ASC NULLS FIRST"),
            Ordering::AscNullsLast => f.push_str("ASC NULLS LAST"),
            Ordering::DescNullsFirst => f.push_str("DESC NULLS FIRST"),
            Ordering::DescNullsLast => f.push_str("DESC NULLS LAST"),
        }
    }
}

pub enum QueryBuilderType<C: Column> {
    Select {
        columns: Vec<C>,
    },
    Insert {
        columns: Vec<C>,
        returning: Option<Vec<C>>,
    },
    Delete,
}

pub struct QueryBuilder<'a, C: Column> {
    table: String,
    columns: Option<&'a [C]>,
    filter: Option<Expr<'a, C>>,
    order_by: Vec<(C, Ordering)>,
    limit: Option<i64>,
    offset: Option<i64>,
}

impl<'a, C: Column> QueryBuilder<'a, C> {
    pub fn new(table: &str) -> Self {
        QueryBuilder {
            table: table.to_string(),
            columns: None,
            filter: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn columns(mut self, columns: &'a [C]) -> Self {
        self.columns = Some(columns);
        self
    }

    pub fn filter(mut self, expr: Expr<'a, C>) -> Self {
        if let Some(where_clause) = self.filter {
            self.filter = Some(where_clause.and(expr));
        } else {
            self.filter = Some(expr);
        }

        self
    }

    pub fn order_by(mut self, column: C, ordering: Ordering) -> Self {
        self.order_by.push((column, ordering));
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

    pub fn to_sql(self) -> (String, Vec<Box<dyn Bindable + 'a>>) {
        let mut query = "SELECT ".to_string();
        let mut values = Vec::new();

        if let Some(ref columns) = self.columns {
            let mut first = true;
            for column in columns.iter() {
                if !first {
                    query.push_str(", ");
                } else {
                    first = false;
                }

                column.write(&mut query);
            }
        } else {
            query.push('*');
        }

        query.push_str(&format!(" FROM \"{}\"", self.table));

        // WHERE
        if let Some(where_clause) = self.filter {
            query.push_str(" WHERE ");
            where_clause.write(&mut query, &mut values);
        }

        if !self.order_by.is_empty() {
            query.push_str(" ORDER BY ");
            let mut is_first = true;
            for (order, ordering) in &self.order_by {
                if !is_first {
                    query.push_str(", ");
                } else {
                    is_first = false;
                }
                order.write(&mut query);
                query.push(' ');
                ordering.write(&mut query);
            }
        }

        if let Some(limit) = self.limit {
            query.push_str(" LIMIT ");
            query.push_str(&limit.to_string());
        }

        if let Some(offset) = self.offset {
            query.push_str(" OFFSET ");
            query.push_str(&offset.to_string());
        }

        (query, values)
    }

    pub async fn fetch_one<'e, 'c: 'e, E, S>(self, executor: E) -> Result<S, sqlx::Error>
    where
        E: 'e + sqlx::Executor<'c, Database = sqlx::Sqlite>,
        S: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
    {
        let (query, values) = self.to_sql();
        let mut query = sqlx::query(&query);
        for value in values.iter() {
            query = value.bind_to(query);
        }

        query
            .fetch_one(executor)
            .await
            .and_then(|row| S::from_row(&row))
    }

    pub async fn fetch_optional<'e, 'c: 'e, E, S>(
        self,
        executor: E,
    ) -> Result<Option<S>, sqlx::Error>
    where
        E: 'e + sqlx::Executor<'c, Database = sqlx::Sqlite>,
        S: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
    {
        let (query, values) = self.to_sql();
        let mut query = sqlx::query(&query);
        for value in values.iter() {
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
        for value in values.iter() {
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
        for value in values.iter() {
            query = value.bind_to(query);
        }

        query.execute(executor).await
    }
}
