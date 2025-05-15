use crate::builder::expr::Expr;
use crate::builder::{QueryBuilder, Select};
use crate::entity::Entity;

pub struct Reference<RE: Entity> {
    filter: Expr<'static, RE::C>,
    _marker: std::marker::PhantomData<RE>,
}

impl<RE: Entity> Reference<RE> {
    pub fn new(filter: Expr<'static, RE::C>) -> Self {
        Self {
            filter,
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn load<'e, E>(&self, executor: E) -> sqlx::Result<RE>
    where
        E: 'e + sqlx::Executor<'e, Database = sqlx::Sqlite>,
        for<'r> RE: sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
    {
        RE::query()
            .filter(self.filter.clone())
            .fetch_one(executor)
            .await
    }

    pub fn query<'a>(&self) -> QueryBuilder<'a, Select, RE::C> {
        RE::query().filter(self.filter.clone())
    }
}
