use kali::{column::ColumnExpr, entity::Entity};
use sqlx::SqlitePool;

#[kali::entity("users")]
#[derive(Debug, sqlx::FromRow)]
struct User {
    id: i64,
    username: String,

    #[relation(referenced_by = user)]
    profile: kali::reference::Reference<UserProfile>,

    #[relation(referenced_by = user)]
    posts: kali::collection::Collection<Post>,
}

#[kali::entity("user_profiles")]
#[derive(Debug, sqlx::FromRow)]
struct UserProfile {
    #[primary_key]
    user_id: i64,
    bio: String,

    #[relation(foreign_key = user_id)]
    user: kali::reference::Reference<User>,
}

#[kali::entity("posts")]
#[derive(Debug, sqlx::FromRow)]
struct Post {
    id: i64,
    content: String,
    user_id: i64,
    title: String,

    #[relation(foreign_key = user_id, references = id)]
    user: kali::reference::Reference<User>,
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users", "profiles"))]
async fn test_user_profile_relation(pool: SqlitePool) -> anyhow::Result<()> {
    // Test fetching a user and then getting their profile
    let user = User::fetch_one(&pool, 1).await?;
    assert_eq!(user.id, 1);
    assert_eq!(user.username, "holden");

    let profile = user.profile().load(&pool).await?;
    assert_eq!(profile.user_id, 1);
    assert_eq!(profile.bio, "can't park there bud");

    Ok(())
}

#[sqlx::test(migrations = "tests/migrations", fixtures("users", "profiles"))]
async fn test_profile_user_relation(pool: SqlitePool) -> anyhow::Result<()> {
    // Test fetching a profile and then getting the related user
    let profile = UserProfile::fetch_one(&pool, 1).await?;
    assert_eq!(profile.user_id, 1);
    assert_eq!(profile.bio, "can't park there bud");

    let user = profile.user().load(&pool).await?;
    assert_eq!(user.id, 1);
    assert_eq!(user.username, "holden");

    Ok(())
}

#[sqlx::test(
    migrations = "tests/migrations",
    fixtures("users", "profiles", "posts")
)]
async fn test_user_posts_collection(pool: SqlitePool) -> anyhow::Result<()> {
    // Test fetching a user and then getting their posts
    let user = User::fetch_one(&pool, 7).await?;
    assert_eq!(user.id, 7);
    assert_eq!(user.username, "naomi");

    let posts = user.posts().load_all(&pool).await?;
    assert_eq!(posts.len(), 3); // User 7 has 3 posts

    // Check content of one of the posts
    let post = posts.iter().find(|p| p.id == 5).unwrap();
    assert_eq!(post.title, "Foundation");
    assert!(post.content.contains("Asimov"));

    Ok(())
}

#[sqlx::test(
    migrations = "tests/migrations",
    fixtures("users", "profiles", "posts")
)]
async fn test_post_user_relation(pool: SqlitePool) -> anyhow::Result<()> {
    // Test fetching a post and then getting the related user
    let post: Post = Post::query()
        .filter(Post::Id.eq(1))
        .fetch_one(&pool)
        .await?;

    assert_eq!(post.id, 1);

    let user = post.user().load(&pool).await?;
    assert_eq!(user.id, 1);
    assert_eq!(user.username, "holden");

    Ok(())
}

#[sqlx::test(
    migrations = "tests/migrations",
    fixtures("users", "profiles", "posts")
)]
async fn test_query_with_relation_filter(pool: SqlitePool) -> anyhow::Result<()> {
    // Get a user
    let user = User::fetch_one(&pool, 7).await?;

    // Use the relation's query method to get filtered posts
    let posts: Vec<Post> = user
        .posts()
        .query()
        .filter(Post::Title.eq("Foundation"))
        .fetch_all(&pool)
        .await?;

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, 5);
    assert_eq!(posts[0].title, "Foundation");
    assert!(posts[0].content.contains("Asimov"));

    Ok(())
}
