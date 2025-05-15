# kali

> [!CAUTION]
> This is a work in progress and is not ready for even alpha use yet.

a query builder and orm-ish crate for rust, built on sqlx

## todo

- updating entities
- optional relations
- support for joins and preloading collections/relations with them
- support for streaming collections
- support for caching relations
  - `try_unwrap()` and `unwrap()`
  - `load()` and `load_all()` use cached data when available, otherwise loads and caches
- codegen cli
  - generate entities from an existing database
  - avoids the entities getting out of sync with the database
  - allow mapping types
    - option 1: `#[map_type(bool, optional_transformer)]` which is read by the codegen before overwriting the file
    - option 2: toml config file that specifies custom mappings

## example

```rs
#[kali::entity]
#[derive(Debug, sqlx::FromRow)]
struct User {
    // optional, defaults to `id`
    #[primary_key]
    id: i32,

    #[relation(referenced_by = user)]
    profile: Reference<Profile>,

    #[relation(referenced_by = user)]
    posts: Collection<Post>
}

#[kali::entity]
#[derive(Debug, sqlx::FromRow)]
struct Profile {
    #[primary_key]
    id: i32,

    nickname: Option<String>,
    
    // references is optional, defaults to the primary key of the referenced entity
    #[relation(foreign_key = id, references = id)]
    user: Reference<User>
}

#[kali::entity]
#[derive(Debug, sqlx::FromRow)]
struct Post {
    #[primary_key]
    id: i32,

    title: String,

    content: String,

    author_id: i32,

    #[relation(foreign_key = author_id, references = id)]
    author: Reference<User>
}

// Finding an entity
let user = User::fetch_one(&db, 1).await?;
let profile = user.profile().load(&db).await?;

// Finding an entity with a filter
let found_user = User::query()
    .where(User::Username.eq("test"))
    .where(User::Id.gt(1))
    .order_by(User::Id.asc())
    .fetch_one(&db)
    .await?;

// Deleting an entity by id
User::delete_one(&db, 1).await?;

// Escape hatch
let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
    .bind(1)
    .fetch_one(&db)
    .await?;
```