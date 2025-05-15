use crate::{
    builder::{QueryBuilder, Select},
    column::Column,
};

pub trait Entity {
    type C: Column + 'static;

    fn table_name() -> &'static str;
    fn columns() -> &'static [Self::C];
    fn primary_key() -> &'static Self::C;

    fn query<'a>() -> QueryBuilder<'a, Select, Self::C> {
        QueryBuilder::select_from(Self::table_name()).columns(Self::columns())
    }
}
