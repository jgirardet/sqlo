macro_rules! test_qmarks {
    ($nb:literal  $res:literal) => {
        paste::paste! {

            #[test]
            fn [<qmarks_sqlite_  $nb>]() {
                assert_eq!($crate::database::qmarks($nb),$res);
            }
        }
    };
}

macro_rules! test_qmarks_with_col {
        ($($col:literal),*; $res:literal) => {
            paste::paste!{

                #[test]
                fn [<qmarks_with_col_sqlite _ $($col)*>]() {
                    assert_eq!(
                        $crate::database::qmarks_with_col(0,&[$($col),*])
                        ,$res.to_string()
                    );
                }
            }
            };
    }
