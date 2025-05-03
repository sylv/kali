## goals

- focus on simplicity. a thin layer on top of sqlx that makes it easier to work with
- coexist with existing tools - integrate nicely with sqlx and sea-orm, support async_graphql and serde macros on entities
- only focus on sqlite and possibly postgres. im not being lazy, im being opinionated. its called a design choice and its classy.
- codegen
  - generate entities from an existing database
  - avoids the entities getting out of sync without having to do migrations ourselves
  - if this route, taking codegen style from tonic might be better than outputting to src
  - allow mapping types
    - option 1: `#[map_type(bool, optional_transformer)]` which is read by the codegen before overwriting the file
    - option 2: toml config file that specifies custom mappings

## usage

```rs
#[derive(Debug)]
#[kali::entity]
struct User {
    #[primary_key]
    id: i32,

    #[referenced_by(field = user)]
    profile: Ref<Profile>,

    #[referenced_by(field = author)]
    posts: Collection<Post>
}

#[derive(Debug)]
#[kali::entity]
struct Profile {
    #[primary_key]
    id: i32,

    nickname: Option<String>,
    
    #[foreign_key(field = id, references = id)]
    user: Ref<User>
}

#[derive(Debug)]
#[kali::entity]
struct Post {
    #[primary_key]
    id: i32,

    title: String,

    content: String,

    author_id: i32,

    #[foreign_key(field = author_id, references = id)]
    author: Ref<User>
}

// Finding an entity
// find_first_by_id, maybe_find_first_by_id, find_by_id, find_by_ids
let user = User::find_first_by_id(&db, 1).await?;

// Loading relations (Ref<T> or Ref<Option<T>>)
// load, unwrap, try_unwrap
let profile = user.profile().load(&db).await?;

// Streaming related entities
// stream, load_all
let mut posts = user.posts().stream(&db).await?;
while let Some(post) = posts.try_next().await? {
    println!("{:#?}", post);
}

// Inserting a new entity
let user = User::create()
    .id(1)
    .title("Hello World")
    .profile(new_profile)
    .insert(&db)
    .await?;

// Finding an entity with a filter
let found_user = User::query()
    .where(UserColumn::Username.eq("test"))
    .where(UserColumn::Id.gt(1))
    .order_by(UserColumn::Id, Order::Asc)
    .find_first(&db)

// Updating an entity
let mut profile = Profile::find_first_by_id(&db, 1).await?;
profile.nickname = Some("new_nickname".to_string());
profile.update(&db).await?;

// Deleting an entity by id
User::delete_by_id(&db, 1).await?;

// Deleting an entity by reference
let user = User::find_first_by_id(&db, 1).await?;
user.delete(&db).await?;

// Escape hatch
// query_as! might not be doable because of https://github.com/launchbadge/sqlx/issues/514
let user = query_as!(User, "SELECT * FROM users WHERE id = $1", 1)
    .fetch_one(&db)
    .await?;
```