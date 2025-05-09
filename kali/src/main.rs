use kali::{builder::QueryBuilder, column::ColumnExpr};
use kali_macros::entity;
use sqlx::prelude::FromRow;

#[entity]
#[derive(Debug, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
}

pub fn main() {
    let qb = QueryBuilder::select_from(User::TABLE_NAME)
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
