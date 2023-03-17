use std::{collections::HashSet, ops::Add};

use syn::Expr;

use super::{ColumnToSql, Generator};

#[derive(Debug, Default)]
pub struct Fragment {
    pub query: String,
    pub params: Vec<syn::Expr>,
    pub joins: HashSet<String>,
}

// Take  a string query
impl From<String> for Fragment {
    fn from(s: String) -> Self {
        Fragment {
            query: s,
            params: vec![],
            joins: HashSet::default(),
        }
    }
}

// Take  a string query
impl From<&str> for Fragment {
    fn from(s: &str) -> Self {
        Fragment {
            query: s.to_string(),
            params: vec![],
            joins: HashSet::default(),
        }
    }
}

// Take to tupple string (query, join)
impl From<(String, String)> for Fragment {
    fn from(s: (String, String)) -> Self {
        let mut h = HashSet::default();
        h.insert(s.1);
        Fragment {
            query: s.0,
            params: vec![],
            joins: h,
        }
    }
}

// take an Expr so its a argument
impl From<Expr> for Fragment {
    fn from(expr: Expr) -> Self {
        Fragment {
            query: "?".to_string(),
            params: vec![expr],
            joins: HashSet::default(),
        }
    }
}

impl Add<Fragment> for Fragment {
    type Output = Fragment;

    fn add(self, rhs: Fragment) -> Self::Output {
        let base_query = if self.query.is_empty() {
            "".to_string()
        } else {
            format!("{}, ", self.query)
        };
        Fragment {
            query: format!["{}{}", base_query, rhs.query],
            params: [self.params, rhs.params].concat(),
            joins: HashSet::from_iter(self.joins.into_iter().chain(rhs.joins)),
        }
    }
}

impl Fragment {
    pub fn add_no_comma(self, rhs: Fragment) -> Self {
        Fragment {
            query: format!["{} {}", self.query, rhs.query],
            params: [self.params, rhs.params].concat(),
            joins: HashSet::from_iter(self.joins.into_iter().chain(rhs.joins)),
        }
    }

    pub fn prepend_str(&mut self, text: &str) {
        self.query = format!("{}{}", text, self.query);
    }

    pub fn append_str(&mut self, text: &str) {
        self.query = format!("{}{}", self.query, text);
    }
}

impl Fragment {
    pub fn from_iterator<'a, T>(
        slice: T,
        ctx: &mut Generator,
    ) -> Result<Fragment, crate::error::SqloError>
    where
        T: std::iter::IntoIterator + 'a,
        T::Item: ColumnToSql + 'a,
    {
        let mut res = Fragment::default();
        for f in slice.into_iter() {
            res = res + f.column_to_sql(ctx)?
        }
        Ok(res)
    }
}
