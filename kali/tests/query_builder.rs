use kali::{
    column::{Column, ColumnExpr},
    query_builder::QueryBuilder,
};
use sqlx::SqlitePool;

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

#[derive(Debug, sqlx::FromRow)]
struct User {
    id: i64,
    username: String,
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users"))]
async fn fetch_one_test(pool: SqlitePool) -> anyhow::Result<()> {
    let user: User = QueryBuilder::new("users")
        .columns(&[UserColumn::Id, UserColumn::Username])
        .filter(UserColumn::Id.eq(1))
        .fetch_one(&pool)
        .await?;

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "alice");

    Ok(())
}
