use sqlx::{Sqlite, query::Query, sqlite::SqliteArguments};

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    String(String),
    Integer(i64),
    Real(f64),
    Blob(Vec<u8>),
    Null,
}

impl Value {
    pub fn bind_to<'q, 'a>(
        self,
        query: Query<'q, Sqlite, SqliteArguments<'a>>,
    ) -> Query<'q, Sqlite, SqliteArguments<'a>>
    where
        'q: 'a,
        'a: 'q,
    {
        match self {
            Value::Bool(v) => query.bind(v),
            Value::String(v) => query.bind(v),
            Value::Integer(v) => query.bind(v),
            Value::Real(v) => query.bind(v),
            Value::Blob(v) => query.bind(v),
            Value::Null => query.bind::<Option<i32>>(None),
        }
    }
}

macro_rules! valuable {
    ($name:ident, $type:ty) => {
        impl Into<Value> for $type {
            fn into(self) -> Value {
                Value::$name(self)
            }
        }
    };
}

valuable!(Bool, bool);
valuable!(String, String);
valuable!(Integer, i64);
valuable!(Real, f64);
valuable!(Blob, Vec<u8>);

macro_rules! valuable_with_coerce {
    ($name:ident, $type:ty, $conv:ty) => {
        impl Into<Value> for $type {
            fn into(self) -> Value {
                Value::$name(self as $conv)
            }
        }
    };
}

// note: sqlx does not support u64
valuable_with_coerce!(Integer, i8, i64);
valuable_with_coerce!(Integer, i16, i64);
valuable_with_coerce!(Integer, i32, i64);
valuable_with_coerce!(Integer, u32, i64);
valuable_with_coerce!(Real, f32, f64);
valuable_with_coerce!(Integer, u8, i64);
valuable_with_coerce!(Integer, u16, i64);

impl Into<Value> for &str {
    fn into(self) -> Value {
        Value::String(self.to_string())
    }
}

impl<T: Into<Value>> Into<Value> for Option<T> {
    fn into(self) -> Value {
        match self {
            Some(value) => value.into(),
            None => Value::Null,
        }
    }
}

impl Into<Value> for () {
    fn into(self) -> Value {
        Value::Null
    }
}

impl Into<Value> for &[u8] {
    fn into(self) -> Value {
        Value::Blob(self.to_vec())
    }
}
