use kali::{
    column::{Column, ColumnExpr},
    query_builder::{OnConflict, QueryBuilder},
};
use sqlx::SqlitePool;

pub enum UserCol {
    Id,
    Username,
}

impl Column for UserCol {
    fn raw(&self) -> &str {
        match self {
            UserCol::Id => "id",
            UserCol::Username => "username",
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct User {
    id: i64,
    username: String,
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users"))]
async fn select_from(pool: SqlitePool) -> anyhow::Result<()> {
    let user: User = QueryBuilder::select_from("users")
        .columns(&[UserCol::Id, UserCol::Username])
        .filter(UserCol::Id.eq(1))
        .fetch_one(&pool)
        .await?;

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "holden");

    Ok(())
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users"))]
async fn insert_into(pool: SqlitePool) -> anyhow::Result<()> {
    QueryBuilder::insert_into("users")
        .value(UserCol::Username, "prax")
        .execute(&pool)
        .await?;

    let user: User = QueryBuilder::select_from("users")
        .columns(&[UserCol::Id, UserCol::Username])
        .filter(UserCol::Username.eq("prax"))
        .fetch_one(&pool)
        .await?;

    assert_eq!(user.id, 8); // auto-generated
    assert_eq!(user.username, "prax");

    Ok(())
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users"))]
async fn update(pool: SqlitePool) -> anyhow::Result<()> {
    let user: User = QueryBuilder::select_from("users")
        .columns(&[UserCol::Id, UserCol::Username])
        .filter(UserCol::Id.eq(1))
        .fetch_one(&pool)
        .await?;

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "holden");

    QueryBuilder::update("users")
        .set(UserCol::Username, "james")
        .filter(UserCol::Id.eq(1))
        .execute(&pool)
        .await?;

    let user: User = QueryBuilder::select_from("users")
        .columns(&[UserCol::Id, UserCol::Username])
        .filter(UserCol::Id.eq(1))
        .fetch_one(&pool)
        .await?;

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "james");

    Ok(())
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users"))]
async fn delete(pool: SqlitePool) -> anyhow::Result<()> {
    let user: Option<User> = QueryBuilder::select_from("users")
        .columns(&[UserCol::Id, UserCol::Username])
        .filter(UserCol::Id.eq(1))
        .fetch_optional(&pool)
        .await?;

    assert!(user.is_some());

    QueryBuilder::delete_from("users")
        .filter(UserCol::Id.eq(1))
        .execute(&pool)
        .await?;

    let user: Option<User> = QueryBuilder::select_from("users")
        .columns(&[UserCol::Id, UserCol::Username])
        .filter(UserCol::Id.eq(1))
        .fetch_optional(&pool)
        .await?;

    assert!(user.is_none());
    Ok(())
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users"))]
async fn returning(pool: SqlitePool) -> anyhow::Result<()> {
    let user: User = QueryBuilder::insert_into("users")
        .value(UserCol::Username, "prax")
        .returning(&[UserCol::Id, UserCol::Username])
        .fetch_one(&pool)
        .await?;

    assert_eq!(user.id, 8); // auto-generated
    assert_eq!(user.username, "prax");

    Ok(())
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users"))]
async fn on_conflict(pool: SqlitePool) -> anyhow::Result<()> {
    let user: User = QueryBuilder::insert_into("users")
        .value(UserCol::Id, 1)
        .value(UserCol::Username, "prax")
        .on_conflict(UserCol::Id, OnConflict::Update)
        .set(UserCol::Username, "holden")
        .returning(&[UserCol::Id, UserCol::Username])
        .fetch_one(&pool)
        .await?;

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "holden");

    Ok(())
}
