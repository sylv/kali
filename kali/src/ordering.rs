use crate::column::Column;

pub enum Ordering<T: Column> {
    Asc(T),
    Desc(T),
    AscNullsFirst(T),
    AscNullsLast(T),
    DescNullsFirst(T),
    DescNullsLast(T),
}

impl<T: Column> Ordering<T> {
    pub fn write(&self, f: &mut String) {
        match self {
            Ordering::Asc(column) => {
                column.write(f);
                f.push_str(" ASC");
            }
            Ordering::Desc(column) => {
                column.write(f);
                f.push_str(" DESC");
            }
            Ordering::AscNullsFirst(column) => {
                column.write(f);
                f.push_str(" ASC NULLS FIRST");
            }
            Ordering::AscNullsLast(column) => {
                column.write(f);
                f.push_str(" ASC NULLS LAST");
            }
            Ordering::DescNullsFirst(column) => {
                column.write(f);
                f.push_str(" DESC NULLS FIRST");
            }
            Ordering::DescNullsLast(column) => {
                column.write(f);
                f.push_str(" DESC NULLS LAST");
            }
        }
    }
}
