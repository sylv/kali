use crate::column::Column;

pub enum ColumnOrdering<T: Column> {
    Asc(T),
    Desc(T),
    AscNullsFirst(T),
    AscNullsLast(T),
    DescNullsFirst(T),
    DescNullsLast(T),
}

impl<T: Column> ColumnOrdering<T> {
    pub fn write(&self, f: &mut String) {
        match self {
            ColumnOrdering::Asc(column) => {
                column.write(f);
                f.push_str(" ASC");
            }
            ColumnOrdering::Desc(column) => {
                column.write(f);
                f.push_str(" DESC");
            }
            ColumnOrdering::AscNullsFirst(column) => {
                column.write(f);
                f.push_str(" ASC NULLS FIRST");
            }
            ColumnOrdering::AscNullsLast(column) => {
                column.write(f);
                f.push_str(" ASC NULLS LAST");
            }
            ColumnOrdering::DescNullsFirst(column) => {
                column.write(f);
                f.push_str(" DESC NULLS FIRST");
            }
            ColumnOrdering::DescNullsLast(column) => {
                column.write(f);
                f.push_str(" DESC NULLS LAST");
            }
        }
    }
}
