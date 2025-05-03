use kali::{
    column::{Column, ColumnExpr},
    query_builder::{Ordering, QueryBuilder},
};
use sqlx::prelude::FromRow;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
}

impl User {
    const TABLE_NAME: &'static str = "users";
    const COLUMNS: &'static [UserColumn] = &[UserColumn::Id, UserColumn::Username];

    pub async fn fetch_one<'e, 'c: 'e, E>(id: i64, executor: E) -> Result<Self, sqlx::Error>
    where
        E: 'e + sqlx::Executor<'c, Database = sqlx::Sqlite>,
    {
        QueryBuilder::new(User::TABLE_NAME)
            .columns(User::COLUMNS)
            .filter(UserColumn::Id.eq(id))
            .fetch_one(executor)
            .await
    }

    pub async fn fetch_optional<'e, 'c: 'e, E>(
        id: i64,
        executor: E,
    ) -> Result<Option<Self>, sqlx::Error>
    where
        E: 'e + sqlx::Executor<'c, Database = sqlx::Sqlite>,
    {
        QueryBuilder::new(User::TABLE_NAME)
            .columns(User::COLUMNS)
            .filter(UserColumn::Id.eq(id))
            .fetch_optional(executor)
            .await
    }

    pub async fn fetch_all<'e, 'c: 'e, E>(executor: E) -> Result<Vec<Self>, sqlx::Error>
    where
        E: 'e + sqlx::Executor<'c, Database = sqlx::Sqlite>,
    {
        QueryBuilder::new(User::TABLE_NAME)
            .columns(User::COLUMNS)
            .fetch_all(executor)
            .await
    }
}

pub enum UserColumn {
    Id,
    Username,
}

impl Column for UserColumn {
    fn raw(&self) -> &str {
        match self {
            UserColumn::Id => "id",
            UserColumn::Username => "username",
        }
    }
}

pub fn main() {
    let (sql, values) = QueryBuilder::new("users")
        .columns(&[UserColumn::Id, UserColumn::Username])
        .filter(UserColumn::Id.in_list(vec![1, 2, 3]))
        .filter(UserColumn::Username.eq("admin"))
        .order_by(UserColumn::Username, Ordering::Asc)
        .order_by(UserColumn::Id, Ordering::DescNullsFirst)
        .to_sql();

    println!("{}", sql);
    println!("{:?}", values);
    // let result: User = QueryBuilder::new("users")
    //     .filter(UserColumn::Id.in_list(vec![1, 2, 3]))
    //     .filter(UserColumn::Username.eq("admin"))
    //     .order_by(UserColumn::Username, Ordering::Asc)
    //     .order_by(UserColumn::Id, Ordering::DescNullsFirst)
    //     .fetch_one(&db)
    //     .await?;
}
