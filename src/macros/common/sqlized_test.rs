#[cfg(test)]
mod test_sqlized {

    #[test]
    #[ignore = "editor_thing"]
    fn dummy_for_rust_analyzer() {}

    macro_rules! sqlize_success {
        ($name:ident, $input:literal, $res: literal $(, $nb:literal)?) => {
            paste::paste! {
                #[test]
                fn [<sqlize_ $name>]() {
                    let sqlos = crate::virtual_file::VirtualFile::new().load().unwrap();
                    let phrase:crate::macros::common::Phrase = syn::parse_str($input).unwrap();
                    let sqlized = phrase.sqlize(&sqlos).unwrap();
                    assert_eq!(sqlized.to_string(), $res);
                    $(assert_eq!(sqlized.params().len(), $nb);)?

                }
            }
        };
    }

    macro_rules! sqlize_fail {
        ($name:ident, $input:literal, $res: literal) => {
            paste::paste! {
                #[test]
                fn [<sqlize_ $name>]() {
                    let sqlos = crate::virtual_file::VirtualFile::new().load().unwrap();
                    let phrase:crate::macros::common::Phrase = syn::parse_str($input).unwrap();
                    let err = phrase.sqlize(&sqlos).err().unwrap();
                    assert_eq!(err.to_string(), $res);

                }
            }
        };
    }

    #[cfg(test)]
    mod select {
        sqlize_success!(one_colonne, "SELECT id FROM Aaa", "SELECT id FROM aaa");
        sqlize_fail!(
            colone_unkownn,
            "SELECT fze FROM Aaa",
            "Field not found in [Aaa]"
        );
        sqlize_success!(
            column_has_other_name,
            "SELECT fi32 FROM Aaa",
            "SELECT fi32col FROM aaa"
        );
        sqlize_success!(two_cols, "SELECT id,id FROM Aaa", "SELECT id,id FROM aaa");
        sqlize_success!(
            with_cast,
            "SELECT id AS bla FROM Aaa",
            "SELECT id AS bla FROM aaa"
        );
        sqlize_success!(
            literal,
            r#"SELECT "string",1,true,false,1.2 FROM Aaa"#,
            r#"SELECT 'string',1,TRUE,FALSE,1.2 FROM aaa"#
        );
        sqlize_fail!(
            literal_not_supported,
            r#"SELECT b"bytestr" FROM Aaa"#,
            "Literal not supported"
        );
        sqlize_success!(
            struct_field_exists,
            "SELECT Aaa.id FROM Aaa",
            "SELECT aaa.id FROM aaa"
        );
        sqlize_success!(
            struct_alias_exists,
            "SELECT a.id FROM Aaa a",
            "SELECT a.id FROM aaa a"
        );
        sqlize_fail!(
            no_struct_for_field,
            "SELECT Bbb.id FROM Aaa",
            "No Sqlo struct or alias found in FROM clause"
        );
        sqlize_fail!(
            no_alias_for_field,
            "SELECT b.id FROM Aaa a",
            "No Sqlo struct or alias found in FROM clause"
        );
        sqlize_fail!(
            no_field_for_struct_with_field,
            "SELECT Aaa.fake FROM Aaa",
            "Field not found in [Aaa]"
        );
        sqlize_fail!(
            no_field_for_alias_with_field,
            "SELECT a.fake FROM Aaa a",
            "Field not found in [Aaa]"
        );
        sqlize_success!(parenthes, "SELECT (id) FROM Aaa", "SELECT (id) FROM aaa");
        sqlize_success!(
            parenthes_two_items,
            "SELECT (id,id) FROM Aaa",
            "SELECT (id,id) FROM aaa"
        );
        sqlize_success!(
            one_arg_call,
            "SELECT COUNT(id) FROM Aaa",
            "SELECT COUNT(id) FROM aaa"
        );
        sqlize_success!(
            two_arg_call,
            "SELECT CONCAT(id,fi32) FROM Aaa",
            "SELECT CONCAT(id,fi32col) FROM aaa"
        );
        sqlize_success!(
            call_with_cast,
            "SELECT COUNT(id) AS e FROM Aaa",
            "SELECT COUNT(id) AS e FROM aaa"
        );
        sqlize_success!(
            binary_same_col,
            "SELECT id+id AS e FROM Aaa",
            "SELECT id + id AS e FROM aaa"
        );

        sqlize_success!(
            binary_arythm_op,
            "SELECT id+id-id*id/id && 1 || 0 AS e FROM Aaa",
            "SELECT id + id - id * id / id AND 1 OR 0 AS e FROM aaa"
        );
        sqlize_success!(
            with_distinct,
            "SELECT DISTINCT id FROM Aaa",
            "SELECT DISTINCT id FROM aaa"
        );
    }

    #[cfg(test)]
    mod from {

        sqlize_success!(
            two_structs,
            "SELECT id FROM Aaa,Bbb",
            "SELECT id FROM aaa,bbb"
        );
        sqlize_success!(
            with_cast,
            "SELECT id FROM Aaa a,Bbb",
            "SELECT id FROM aaa a,bbb"
        );

        sqlize_fail!(
            struct_unknown,
            "SELECT id FROM AZERTY",
            "Can't find Sqlo struct AZERTY"
        );
    }
}
