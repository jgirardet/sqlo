use indexmap::IndexSet;
use std::ops::Add;
use syn::Expr;

#[derive(Debug, Default, Clone)]
pub struct Arguments(IndexSet<Expr>); // IndexSet because order matters

impl Arguments {
    // Insert and/or return  1-based index.
    pub fn insert(&mut self, expr: &Expr) -> usize {
        if let Some(idx) = self.0.get_index_of(expr) {
            idx + 1
        } else {
            self.0.insert(expr.clone());
            self.0.len()
        }
    }

    // Return arguments as a sequence matching query pattern
    #[cfg(feature = "postgres")]
    pub fn as_result(&self, query: &str) -> Vec<&Expr> {
        IndexSet::<usize>::from_iter(get_indexes_form_query(query)) // remove double with pg
            .iter()
            .map(|idx| &self.0[idx - 1])
            .collect()
    }

    // Return arguments as a sequence matching query pattern
    #[cfg(not(feature = "postgres"))]
    pub fn as_result(&self, query: &str) -> Vec<&Expr> {
        get_indexes_form_query(query)
            .iter()
            .map(|idx| &self.0[idx - 1])
            .collect()
    }
}

impl IntoIterator for Arguments {
    type Item = Expr;

    type IntoIter = indexmap::set::IntoIter<Expr>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Extend<Expr> for Arguments {
    fn extend<T: IntoIterator<Item = Expr>>(&mut self, iter: T) {
        for i in iter {
            self.insert(&i);
        }
    }
}

impl From<Expr> for Arguments {
    fn from(value: Expr) -> Self {
        Arguments(IndexSet::from([value]))
    }
}

impl Add<Arguments> for Arguments {
    type Output = Arguments;

    fn add(self, rhs: Arguments) -> Self::Output {
        let mut res = Arguments::default();
        res.extend(self.0.into_iter().chain(rhs.0.into_iter()));
        res
    }
}

fn get_indexes_form_query(query: &str) -> Vec<usize> {
    let re = regex_macro::regex!(r"\$(\d+)");
    re.captures_iter(query)
        .map(|x| {
            x.get(1)
                .unwrap()
                .as_str()
                .parse::<usize>()
                .expect("Sqlo Internal Error: failed parsing usize argument")
        })
        .collect()
}
