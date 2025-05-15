use kali::{
    builder::{OnConflict, QueryBuilder},
    column::ColumnExpr,
};
use sqlx::SqlitePool;

#[kali::entity]
#[derive(Debug, sqlx::FromRow)]
struct User {
    id: i64,
    username: String,
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users"))]
async fn select_from(pool: SqlitePool) -> anyhow::Result<()> {
    let user: User = QueryBuilder::select_from("users")
        .columns(&[User::Id, User::Username])
        .filter(User::Id.eq(1))
        .fetch_one(&pool)
        .await?;

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "holden");

    Ok(())
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users"))]
async fn insert_into(pool: SqlitePool) -> anyhow::Result<()> {
    QueryBuilder::insert_into("users")
        .value(User::Username, "prax")
        .execute(&pool)
        .await?;

    let user: User = QueryBuilder::select_from("users")
        .columns(&[User::Id, User::Username])
        .filter(User::Username.eq("prax"))
        .fetch_one(&pool)
        .await?;

    assert_eq!(user.id, 8); // auto-generated
    assert_eq!(user.username, "prax");

    Ok(())
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users"))]
async fn update(pool: SqlitePool) -> anyhow::Result<()> {
    let user: User = QueryBuilder::select_from("users")
        .columns(&[User::Id, User::Username])
        .filter(User::Id.eq(1))
        .fetch_one(&pool)
        .await?;

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "holden");

    QueryBuilder::update("users")
        .set(User::Username, "james")
        .filter(User::Id.eq(1))
        .execute(&pool)
        .await?;

    let user: User = QueryBuilder::select_from("users")
        .columns(&[User::Id, User::Username])
        .filter(User::Id.eq(1))
        .fetch_one(&pool)
        .await?;

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "james");

    Ok(())
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users"))]
async fn delete(pool: SqlitePool) -> anyhow::Result<()> {
    let user: Option<User> = QueryBuilder::select_from("users")
        .columns(&[User::Id, User::Username])
        .filter(User::Id.eq(1))
        .fetch_optional(&pool)
        .await?;

    assert!(user.is_some());

    QueryBuilder::delete_from("users")
        .filter(User::Id.eq(1))
        .execute(&pool)
        .await?;

    let user: Option<User> = QueryBuilder::select_from("users")
        .columns(&[User::Id, User::Username])
        .filter(User::Id.eq(1))
        .fetch_optional(&pool)
        .await?;

    assert!(user.is_none());
    Ok(())
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users"))]
async fn returning(pool: SqlitePool) -> anyhow::Result<()> {
    let user: User = QueryBuilder::insert_into("users")
        .value(User::Username, "prax")
        .returning(&[User::Id, User::Username])
        .fetch_one(&pool)
        .await?;

    assert_eq!(user.id, 8); // auto-generated
    assert_eq!(user.username, "prax");

    Ok(())
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users"))]
async fn on_conflict(pool: SqlitePool) -> anyhow::Result<()> {
    let user: User = QueryBuilder::insert_into("users")
        .value(User::Id, 1)
        .value(User::Username, "prax")
        .on_conflict(User::Id, OnConflict::Update)
        .set(User::Username, "holden")
        .returning(&[User::Id, User::Username])
        .fetch_one(&pool)
        .await?;

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "holden");

    Ok(())
}
