use super::{AsSql, Database};
use super::query::condition::{EqField, EqValue};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Table {
    pub(crate) name: &'static str,
    pub(crate) fields: &'static [&'static Field],
}

impl Table {
    pub(crate) fn fields(&self) -> &'static [&'static Field] {
        &self.fields
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Field {
    pub(crate) table: &'static str,
    pub(crate) name: &'static str,
}

impl Field {
    pub(crate) fn eq<'a, DB: Database + 'a>(&'a self, value: &'a AsSql<DB>) -> EqValue<'a, DB> {
        EqValue::new(&self, value)
    }

    pub(crate) fn same_as<'a>(&'a self, other: &'a Field) -> EqField<'a> {
        EqField::new(&self, other)
    }

    pub(crate) fn set<'a, DB: Database + 'a>(&'a self, value: &'a AsSql<DB>) -> SetField<'a, DB> {
        SetField {
            field: &self,
            value: FieldValue::Value(value),
        }
    }

    pub(crate) fn set_default<'a, DB: Database + 'a>(&'a self) -> SetField<'a, DB> {
        SetField {
            field: &self,
            value: FieldValue::Default,
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.table, self.name)
    }
}

pub(crate) enum FieldValue<'a, DB: Database + 'a> {
    Default,
    Value(&'a AsSql<DB>),
}

impl<'a, DB: Database> FieldValue<'a, DB> {
    pub(crate) fn is_default(&self) -> bool {
        match self {
            FieldValue::Default => true,
            _ => false
        }
    }

    pub(crate) fn is_value(&self) -> bool {
        match self {
            FieldValue::Value(_) => true,
            _ => false
        }
    }
}

impl<'a, DB: Database, T: 'a> From<&'a T> for FieldValue<'a, DB>
    where
        T: AsSql<DB>
{
    fn from(t: &'a T) -> Self {
        FieldValue::Value(t)
    }
}

pub(crate) struct SetField<'a, DB: Database + 'a> {
    pub(crate) field: &'a Field,
    pub(crate) value: FieldValue<'a, DB>,
}