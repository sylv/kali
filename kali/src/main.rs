use kali::{
    column::{Column, ColumnExpr},
    query_builder::QueryBuilder,
};
use sqlx::prelude::FromRow;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
}

#[allow(non_upper_case_globals)]
impl User {
    const TABLE_NAME: &'static str = "users";
    const COLUMNS: &'static [UserColumn] = &[UserColumn::Id, UserColumn::Username];

    const Id: UserColumn = UserColumn::Id;
    const Username: UserColumn = UserColumn::Username;

    pub async fn fetch_one<'e, 'c: 'e, E>(id: i64, executor: E) -> Result<Self, sqlx::Error>
    where
        E: 'e + sqlx::Executor<'c, Database = sqlx::Sqlite>,
    {
        QueryBuilder::select_from(User::TABLE_NAME)
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
        QueryBuilder::select_from(User::TABLE_NAME)
            .columns(User::COLUMNS)
            .filter(UserColumn::Id.eq(id))
            .fetch_optional(executor)
            .await
    }

    pub async fn fetch_all<'e, 'c: 'e, E>(executor: E) -> Result<Vec<Self>, sqlx::Error>
    where
        E: 'e + sqlx::Executor<'c, Database = sqlx::Sqlite>,
    {
        QueryBuilder::select_from(User::TABLE_NAME)
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
    let qb = QueryBuilder::select_from("users")
        .columns(&[User::Id, User::Username])
        .filter(User::Id.eq(1))
        .filter(User::Username.eq("admin"))
        .order_by(User::Username.asc())
        .order_by(User::Id.desc_nulls_first())
        .limit(10)
        .offset(5);

    let (sql, values) = qb.to_sql();
    println!("{}", sql);
    println!("{:?}", values);
}
