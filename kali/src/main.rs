use kali::column::ColumnExpr;
use kali_macros::entity;
use sqlx::prelude::FromRow;

#[entity("users")]
#[derive(Debug, FromRow)]
pub struct User {
    pub id: i64,

    pub username: String,

    #[relation(referenced_by = user)]
    pub profile: Reference<UserProfile>,

    #[relation(referenced_by = user)]
    pub posts: Collection<Post>,
}

#[entity("user_profiles")]
#[derive(Debug, FromRow)]
pub struct UserProfile {
    #[primary_key]
    pub user_id: i64,

    pub bio: String,

    #[relation(foreign_key = user_id)]
    pub user: Reference<User>,
}

#[entity("posts")]
#[derive(Debug, FromRow)]
pub struct Post {
    #[primary_key]
    pub id: i64,

    pub user_id: i64,

    pub content: String,

    #[relation(foreign_key = user_id, references = id)]
    pub user: Reference<User>,
}

#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt::init();

    let profile = UserProfile {
        user_id: 1,
        bio: "Hello, world!".to_string(),
    };

    println!("{}", profile.__primary_key_value());

    // let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();

    // migrate!("tests/migrations")
    //     .run(&db)
    //     .await
    //     .expect("Failed to run migrations");

    // let user = profile.user().load(&db).await.expect("Failed to load user");
    // println!("Loaded user: {:?}", user);

    // let posts = user
    //     .posts()
    //     .load_all(&db)
    //     .await
    //     .expect("Failed to load posts");

    // println!("Loaded {} posts:", posts.len());
    // for post in posts {
    //     println!(" - Post {}: {}", post.id, post.content);
    // }
}
