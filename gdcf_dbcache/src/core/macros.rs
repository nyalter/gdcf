macro_rules! table {
    ($model: ident => $table: ident {$($model_field: ident => $table_column: ident),*; $($unmapped_column: ident),*}) => {
        pub  mod $table {
            use super::$model;
            use core::backend::Error;
            use core::query::insert::Insertable;
            use core::query::select::{Queryable, Row};
            use core::table::{Field, Table, SetField};

            #[allow(non_upper_case_globals)]
            pub  const table_name: &str = stringify!($table);

            $(
                #[allow(non_upper_case_globals)]
                pub  static $table_column: Field = Field {
                    table: table_name,
                    name: stringify!($table_column)
                };
            )*

            $(
                #[allow(non_upper_case_globals)]
                pub  static $unmapped_column: Field = Field {
                    table: table_name,
                    name: stringify!($unmapped_column)
                };
            )*

            #[allow(non_upper_case_globals)]
            pub  static table: Table = Table {
                name: table_name,
                fields: &[
                    $(&$table_column,)*
                    $(&$unmapped_column,)*
                ]
            };

            #[cfg(feature = "pg")]
            mod pg {
                use core::backend::pg::Pg;
                use super::*;

                __insertable!(Pg, $model, $($model_field => $table_column,)*);
                __queryable!(Pg, $model, $($model_field,)*);
            }

            #[cfg(feature = "sqlite")]
            mod pg {
                use core::backend::sqlite::Sqlite;
                use super::*;

                __insertable!(Sqlite, $model, $($model_field => $table_column,)*);
                __queryable!(Sqlite, $model, $($model_field,)*);
            }

            #[cfg(feature = "mysql")]
            mod pg {
                use core::backend::mysql::MySql;
                use super::*;

                __insertable!(MySql, $model, $($model_field => $table_column,)*);
                __queryable!(MySql, $model, $($model_field,)*);
            }
        }
    };
}


macro_rules! __insertable {
    ($backend: ty, $model: ty, $($model_field: ident => $table_column: ident),*,) => {
        impl Insertable<$backend> for $model {
            fn table(&self) -> &Table {
                &table
            }

            fn values(&self) -> Vec<SetField<$backend>> {
                vec![
                    $(
                        $table_column.set(&self.$model_field)
                    ),*
                ]
            }
        }
    };
}

macro_rules! __queryable {
    (@[$(($idx: expr, $model_field: ident))*], $next_idx: expr, $backend: ty, $model: ident,) => {
        impl Queryable<$backend> for $model {
            fn from_row(row: &Row<$backend>, offset: isize) -> Result<Self, Error<$backend>> {
                Ok($model {
                    $(
                        $model_field: row.get(offset + $idx).expect("In code generated by table! macro. Did you define your schema correctly?")?,
                    )*
                })
            }
        }
    };

    (@[$(($idx: expr, $model_field: ident))*], $next_idx: expr, $backend: ty, $model: ident, $current: ident, $($rest: ident),*$(,)?) => {
        __queryable!(@[
            $(($idx, $model_field))*
            ($next_idx, $current)
        ], $next_idx + 1, $backend, $model, $($rest,)*);
    };

    ($backend: ty, $model: ident, $($rest: ident),*,) => {
        __queryable!(@[], 0, $backend, $model, $($rest,)*);
    };
}

macro_rules! create {
    ($model: ident, @[], [$($stack_tokens: tt)*], $column: ident => $sql_type: ty, $($rest: tt)*) => {
        create!($model, @[], [$($stack_tokens)*, ($column, $sql_type, [])], $($rest)*);
    };

    ($model: ident, @[$(($types: ty, $cons: expr)),*$(,)?], [$($stack_tokens: tt)*], $column: ident[] => $sql_type: ty, $($rest: tt)*) => {
        create!($model, @[], [$($stack_tokens)*, ($column, $sql_type, [$(($types, $cons),)*])], $($rest)*);
    };

    ($model: ident, @[$(($types: ty, $cons: expr)),*$(,)?], [$($stack_tokens: tt)*], $column: ident[NotNull $(,$($cons_tokens: tt)*)?] => $sql_type: ty, $($rest: tt)*) => {
        create!($model, @[(NotNullConstraint<'a>, NotNullConstraint::default()), $(($types, $cons),)*], [$($stack_tokens)*], $column[$($($cons_tokens)*)?] => $sql_type, $($rest)*);
    };

    ($model: ident, @[$(($types: ty, $cons: expr)),*$(,)?], [$($stack_tokens: tt)*], $column: ident[Unique $(,$($cons_tokens: tt)*)?] => $sql_type: ty, $($rest: tt)*) => {
        create!($model, @[(UniqueConstraint<'a>, UniqueConstraint::default()), $(($types, $cons),)*], [$($stack_tokens)*], $column[$($($cons_tokens)*)?] => $sql_type, $($rest)*);
    };

    ($model: ident, @[$(($types: ty, $cons: expr)),*$(,)?], [$($stack_tokens: tt)*], $column: ident[Primary $(,$($cons_tokens: tt)*)?] => $sql_type: ty, $($rest: tt)*) => {
        create!($model, @[(PrimaryKeyConstraint<'a>, PrimaryKeyConstraint::default()), $(($types, $cons),)*], [$($stack_tokens)*], $column[$($($cons_tokens)*)?] => $sql_type, $($rest)*);
    };

    ($model: ident, @[$(($types: ty, $cons: expr)),*$(,)?], [$($stack_tokens: tt)*], $column: ident[Default($value: expr) $(,$($cons_tokens: tt)*)?] => $sql_type: ty, $($rest: tt)*) => {
        create!($model: ident, @[(DefaultConstraint<'a>, DefaultConstraint::new($value)), $(($types, $cons),)*], [$($stack_tokens)*], $column[$($($cons_tokens)*)?] => $sql_type, $($rest)*);
    };

    ($model: ident, @[], [,$(($column: ident, $sql_type: ty, [$(($cons_type: ty, $constraint: expr)),* $(,)?])),*], ) => {
        use core::types::*;
        use core::query::create::*;
        use core::backend::Database;

        pub  fn create<'a, DB: Database + 'a>() -> Create<'a, DB>
            where
                $(
                    $(
                        $cons_type: Constraint<'a, DB> + 'static,
                    )*
                )*
                $(
                    $sql_type: Type<'a, DB>,
                )*
        {
            $model::table.create()
            $(
                .with_column(Column::new($model::$column.name(), {let ty: $sql_type = Default::default(); ty})
                    $(
                        .constraint($constraint)
                    )*
                )
            )*
        }
    };

    ($model: ident, @$($tokens: tt)*) => {
        compile_error!("Its broken!");
    };

    ($model: ident, $($tokens: tt)*) => {
        create!($model, @[], [], $($tokens)*,);
    };
}

macro_rules! if_query_part {
    ($t: ty, $tr: ty) => {
        impl<'a, DB: Database + 'a> $tr for $t
            where
                $t: QueryPart<'a, DB> {}
    };
}

macro_rules! simple_query_part {
    ($back: ty, $t: ty, $val: expr) => {
        impl<'a> QueryPart<'a, $back> for $t {
            fn to_sql_unprepared(&self) -> String {
                String::from($val)
            }
        }
    };
}