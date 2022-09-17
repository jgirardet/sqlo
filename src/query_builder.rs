use itertools::Itertools;

use crate::sqlo::DatabaseType;

pub fn commma_sep_with_parenthes_literal_list(list: &[&str]) -> String {
    if list.is_empty() {
        return "".to_string();
    }
    let sep_comad = list.into_iter().join(",");
    format!("({sep_comad})")
}

pub fn qmarks(nb: usize, db: &DatabaseType) -> String {
    (0..nb).into_iter().map(|_| db.get_qmark()).join(",")
}

pub fn qmarks_with_col(cols: &[&str], db: &DatabaseType) -> String {
    cols.into_iter().map(|c| format!("{c}={}", db.get_qmark())).join(",")
}

// pub fn qmarks_parenthes(nb: usize, db: &DatabaseType) -> String {
//     if nb == 0 {
//         "()".to_string()
//     } else {
//         let qmarks = qmarks(nb, db);
//         format!("({qmarks})")
//     }
// }

#[cfg(test)]
mod test_query_builder {
    use super::{commma_sep_with_parenthes_literal_list, qmarks, qmarks_with_col};
    use crate::sqlo::DatabaseType;

    #[test]
    fn is_empty() {
        assert_eq!(commma_sep_with_parenthes_literal_list(&[]), "")
    }
    #[test]
    fn is_not_empty() {
        assert_eq!(
            commma_sep_with_parenthes_literal_list(&[&"bla", &"bli"]),
            "(bla,bli)"
        )
    }

    macro_rules! test_qmarks {
        ($nb:literal  $res:literal $db:expr) => {
            paste::paste! {

                #[test]
                fn [<qmarks_  $nb>]() {
                    assert_eq!(qmarks($nb, $db),$res);
                }
            }
        };
    }

    const SQLITE:&DatabaseType = &DatabaseType::Sqlite;

    test_qmarks!(1 "?" &DatabaseType::Sqlite);
    test_qmarks!(2 "?,?" &DatabaseType::Sqlite);
    test_qmarks!(0 "" &DatabaseType::Sqlite);

    macro_rules! test_qmarks_with_col {
        ($($col:literal),*; $db:expr, $res:literal) => {
            paste::paste!{

                #[test]
                fn [<qmarks_with_col _ $($col)*>]() {
                    assert_eq!(
                        qmarks_with_col(&[$($col),*], $db)
                        ,$res.to_string()
                    );
                }
            }
            };
    }
    
    test_qmarks_with_col!("bla","bli"; SQLITE, "bla=?,bli=?");
    test_qmarks_with_col!("bla"; SQLITE, "bla=?");
    test_qmarks_with_col!(; SQLITE, "");

    // macro_rules! test_qmarks_parenthes {
    //     ($nb:literal  $res:literal $db:expr) => {
    //         paste::paste! {

    //             #[test]
    //             fn [<qmarks_parenth_  $nb>]() {
    //                 assert_eq!(qmarks_parenthes($nb, $db), $res);
    //             }
    //         }
    //     };
    // }
    // test_qmarks_parenthes!(1 "(?)" &DatabaseType::Sqlite);
    // test_qmarks_parenthes!(2 "(?,?)" &DatabaseType::Sqlite);
    // test_qmarks_parenthes!(0 "()" &DatabaseType::Sqlite);
}
