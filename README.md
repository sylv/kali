# kali

a query builder and orm-ish crate for rust, built on sqlx

**this is not ready for production use yet**

## todo

- support for loading relations
  - load, unwrap, try_unwrap
  - stream, load_all
- codegen
  - generate entities from an existing database
  - avoids the entities getting out of sync without having to do migrations ourselves
  - if this route, taking codegen style from tonic might be better than outputting to src
  - allow mapping types
    - option 1: `#[map_type(bool, optional_transformer)]` which is read by the codegen before overwriting the file
    - option 2: toml config file that specifies custom mappings

## target

this is a scratch pad for what i'm working toward, not what is supported.

```rs
#[derive(Debug)]
#[kali::entity]
struct User {
    #[primary_key]
    id: i32,

    // #[referenced_by(field = user)]
    // profile: Ref<Profile>,

    // #[referenced_by(field = author)]
    // posts: Collection<Post>
}

#[derive(Debug)]
#[kali::entity]
struct Profile {
    #[primary_key]
    id: i32,

    nickname: Option<String>,
    
    // #[foreign_key(field = id, references = id)]
    // user: Ref<User>
}

#[derive(Debug)]
#[kali::entity]
struct Post {
    #[primary_key]
    id: i32,

    title: String,

    content: String,

    author_id: i32,

    // #[foreign_key(field = author_id, references = id)]
    // author: Ref<User>
}

// Finding an entity
let user = User::fetch_one(&db, 1).await?;

// // Loading relations (Ref<T> or Ref<Option<T>>)
// // load, unwrap, try_unwrap
// let profile = user.profile().load(&db).await?;

// // Streaming related entities
// // stream, load_all
// let mut posts = user.posts().stream(&db).await?;
// while let Some(post) = posts.try_next().await? {
//     println!("{:#?}", post);
// }

// Inserting a new entity
let user = User::create()
    .id(1)
    .title("Hello World")
    .profile(new_profile)
    .insert(&db)
    .await?;

// Finding an entity with a filter
let found_user = User::query()
    .where(User::Username.eq("test"))
    .where(User::Id.gt(1))
    .order_by(User::Id.asc())
    .fetch_one(&db)

// // Updating an entity
// let mut profile = Profile::fetch_one(&db, 1).await?;
// profile.nickname = Some("new_nickname".to_string());
// profile.update(&db).await?;

// Deleting an entity by id
User::delete_one(&db, 1).await?;

// // Deleting an entity by reference
// let user = User::fetch_one(&db, 1).await?;
// user.delete(&db).await?;

// Escape hatch
// query_as! might not be doable because of https://github.com/launchbadge/sqlx/issues/514
let user = query_as!(User, "SELECT * FROM users WHERE id = $1", 1)
    .fetch_one(&db)
    .await?;
```