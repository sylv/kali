use kali::{column::ColumnExpr, entity::Entity};
use sqlx::SqlitePool;

#[kali::entity("users")]
#[derive(Debug, sqlx::FromRow)]
struct User {
    id: i64,
    username: String,
}

#[kali::entity("user_profiles")]
#[derive(Debug, sqlx::FromRow)]
struct UserProfile {
    #[primary_key]
    user_id: i64,
    bio: String,
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users"))]
async fn fetch_user_by_id(pool: SqlitePool) -> anyhow::Result<()> {
    let user = User::fetch_one(&pool, 1).await?;

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "holden");

    Ok(())
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users"))]
async fn query_user_by_id(pool: SqlitePool) -> anyhow::Result<()> {
    let user: User = User::query()
        .filter(User::Id.eq(1))
        .fetch_one(&pool)
        .await?;

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "holden");

    Ok(())
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users", "profiles"))]
async fn fetch_non_standard_pk(pool: SqlitePool) -> anyhow::Result<()> {
    let user_profile = UserProfile::fetch_one(&pool, 1).await?;

    assert_eq!(user_profile.user_id, 1);
    assert_eq!(user_profile.bio, "can't park there bud");

    Ok(())
}
