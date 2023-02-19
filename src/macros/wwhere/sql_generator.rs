use std::collections::BTreeMap;

use crate::{
    error::SqloError, macros::sql_query::SqlQuery, relations::Relation, sqlo::Sqlo, sqlos::Sqlos,
};

use darling::util::IdentString;
use itertools::Itertools;
use syn::{spanned::Spanned, Expr, ExprField, Member};

use super::{
    tok::{Tok, Toks},
    tokenizer::WhereTokenizer,
    Like,
};

pub(crate) fn where_generate_sql<'a>(
    main: &str,
    sqlos: &'a Sqlos,
    wwhere: &WhereTokenizer,
) -> Result<SqlQuery, SqloError> {
    let mut gen = WhereSqlGenerator::new(main, sqlos);
    gen.dispatch(wwhere.into())?;
    let joins = if gen.joins.is_empty() {
        "".to_string()
    } else {
        format!("{} ", gen.joins.values().join(" "))
    };
    let query = format!("{joins}WHERE {}", gen.query());
    Ok(SqlQuery {
        query,
        params: gen.arguments,
    })
}

struct WhereSqlGenerator<'a> {
    sqlos: &'a Sqlos,
    main: &'a Sqlo,
    query: Vec<String>,
    joins: BTreeMap<&'a IdentString, String>,
    arguments: Vec<Expr>,
}

impl<'a> WhereSqlGenerator<'a> {
    fn new(main: &str, sqlos: &'a Sqlos) -> Self {
        let main = sqlos
            .get(main)
            .unwrap_or_else(|_| panic!("No derived struct named ;{main}"));
        WhereSqlGenerator {
            sqlos,
            main,
            query: vec![],
            arguments: vec![],
            joins: BTreeMap::new(),
        }
    }

    fn dispatch(&mut self, toks: Toks) -> Result<(), SqloError> {
        for tok in toks {
            // self.query.push(self.dispatch_tok(tok)?);
            let res = self.dispatch_tok(tok)?;
            self.query.push(res);
        }
        Ok(())
    }

    fn dispatch_toks(&mut self, toks: Toks) -> Result<String, SqloError> {
        let mut res = vec![];
        for tok in toks {
            res.push(self.dispatch_tok(tok)?)
        }
        Ok(res.join(" "))
    }

    fn dispatch_tok(&mut self, tok: Tok) -> Result<String, SqloError> {
        let res = match tok {
            Tok::Field(f) => self.field(f)?,
            Tok::ForeignKey(f) => self.foreign_key(f)?,
            Tok::Null(t) => self.null(t)?,
            // Tok::Between(t) => self.between(t)?,
            Tok::Like(t) => self.like(t)?,
            Tok::Paren(t) => self.parenthesis(t)?,
            Tok::Sign(s) => self.sign(s)?,
            Tok::Value(v) => self.value(v)?,
            Tok::Not(n) => self.not(n)?,
            Tok::Error(e) => self.error(e)?,
            Tok::In(t) => self.iin(t)?,
            // _ => unimplemented!("Not yet all Tok implemented"),
        };
        Ok(res)
    }

    fn query(&self) -> String {
        self.query.join(" ")
    }
}

impl<'a> WhereSqlGenerator<'a> {
    fn error(&mut self, e: syn::Error) -> Result<String, SqloError> {
        Err(e.into())
    }

    fn field(&mut self, v: IdentString) -> Result<String, SqloError> {
        if let Some(field) = self.main.field(&v) {
            Ok(field.column.to_string())
        } else {
            Err(SqloError::new(
                &format!("Can't find field `{v} in {}", self.main.ident),
                v.span(),
            ))
        }
    }

    fn foreign_key(&mut self, f: ExprField) -> Result<String, SqloError> {
        // get base
        let related = match *f.base {
            Expr::Path(p) => {
                if let Some(base) = p.path.get_ident() {
                    base.clone().into()
                } else {
                    return Err(SqloError::new(
                        "Foreign Key field must be single ident, can't use ::",
                        p.span(),
                    ));
                }
            }
            _ => {
                return Err(SqloError::new(
                    "Invalud Foreign Key accessor",
                    f.base.span(),
                ))
            }
        };

        // get related
        let from_field: IdentString = if let Member::Named(ref ident) = f.member {
            ident.clone().into()
        } else {
            return Err(SqloError::new(
                "Invalid, can't be numeric. Should be a field name.",
                f.member.span(),
            ));
        };

        // find a matching relation
        let Relation::ForeignKey(rel) = self.sqlos.relations.find(&self.main.ident, &related)?;

        let slave_sqlo = self.sqlos.get(&rel.from)?;
        let slave_field = slave_sqlo.field(&from_field).ok_or_else(|| {
            SqloError::new(
                &format!("This field does not exist in {} struct", slave_sqlo.ident),
                f.member.span(),
            )
        })?;

        // add join if not already added
        if self.joins.get(&rel.from).is_none() {
            self.joins.insert(&rel.from, rel.to_inner_join(self.sqlos));
        }

        Ok(format!("{}.{}", slave_sqlo.tablename, slave_field.column))
    }

    fn iin(&mut self, toks: Toks) -> Result<String, SqloError> {
        let mut iter = toks.into_iter();
        if let Some(lhs) = iter.next() {
            let lhs = self.dispatch_tok(lhs)?;
            let mut res = vec![];
            for r in iter {
                res.push(self.dispatch_tok(r)?)
            }
            return Ok(format!("{lhs} IN ({})", res.join(",")));
        }
        Err(SqloError::new_lost("Sqlo API Error, In is invalid"))
    }

    fn like(&mut self, l: Like) -> Result<String, SqloError> {
        Ok(l.to_string())
    }

    fn not(&mut self, toks: Toks) -> Result<String, SqloError> {
        if let Tok::Paren(p) = toks
            .into_iter()
            .next()
            .expect("Sqlo API Error, Not should contain something")
        {
            return Ok(format!("NOT {}", self.dispatch_toks(p)?));
        }
        Err(SqloError::new_lost("Sqlo API Error, Not is invalid"))
    }

    fn null(&mut self, toks: Toks) -> Result<String, SqloError> {
        let mut iter = toks.into_iter();
        let lhs = self.dispatch_tok(
            iter.next()
                .expect("Sqlo API Error, Null should always have lhs"),
        )?;
        let sign = match iter
            .next()
            .expect("Sqlo API Error, Null should always have a sign")
            .to_string()
            .as_str()
        {
            "==" => "IS",
            "!=" => "IS NOT",
            _ => {
                return Err(SqloError::new_lost(
                    "Sqlo API Error, Null should only be used with != or ==",
                ))
            }
        };
        Ok(format!("{} {} NULL", lhs, sign))
    }
    fn parenthesis(&mut self, toks: Toks) -> Result<String, SqloError> {
        Ok(format!("({})", self.dispatch_toks(toks)?))
    }

    fn sign(&mut self, s: String) -> Result<String, SqloError> {
        let signe = match s.as_str() {
            "==" => "=",
            "!=" => "<>",
            "<" => "<",
            "<=" => "<=",
            ">" => ">",
            ">=" => ">=",
            "&&" => "AND",
            "||" => "OR",
            _ => return Err(SqloError::new_lost("Operator not supported")),
        };
        Ok(signe.to_string())
    }

    fn value(&mut self, v: Expr) -> Result<String, SqloError> {
        self.arguments.push(v);
        Ok("?".to_string())
    }
}

#[cfg(test)]
mod test_wwhere_sql_generator {

    use crate::{relations::Relations, virtual_file::VirtualFile};

    use super::*;

    use crate::macros::wwhere::tokenizer::WhereTokenizer;

    fn get_sqlos() -> Sqlos {
        let sqlos = VirtualFile::new().load().expect("cannot load Sqlos");
        let mut entities = vec![];
        let mut relations = vec![];
        // filter only thoose related
        for i in ["Aaa", "Bbb", "Ccc", "Ddd"] {
            let sqlo = sqlos.get(i).cloned().unwrap();
            entities.push(sqlo);
            let rela = sqlos.relations.filter_entity("from", i).relations;
            relations.extend(rela);
        }
        Sqlos {
            entities,
            relations: Relations { relations },
        }
    }

    macro_rules! test_where_sql_generator {
        ($title:ident, $main:literal, $content:literal, $res:literal , $arguments:expr) => {
            #[test]
            fn $title() {
                let sqlos = get_sqlos();
                let contt: WhereTokenizer = syn::parse_str($content).expect("test setup error");
                let sql_query =
                    where_generate_sql($main, &sqlos, &contt).expect("generate_where_sql failed");
                assert_eq!(sql_query.query, $res);
                assert_eq!(sql_query.params.len(), $arguments);
            }
        };
    }

    //test_generato_binary(name_of_the_test, main_struct, toks.fn, typeparam, field, res, nb_param)
    test_where_sql_generator!(
        field_change_column,
        "Aaa",
        "fi32 == 1",
        "WHERE fi32col = ?",
        1
    );
    test_where_sql_generator!(field_equal, "Aaa", "fstring == 1", "WHERE fstring = ?", 1);
    test_where_sql_generator!(
        field_different,
        "Aaa",
        "fstring != 1",
        "WHERE fstring <> ?",
        1
    );
    test_where_sql_generator!(field_inferior, "Aaa", "fstring < 1", "WHERE fstring < ?", 1);
    test_where_sql_generator!(
        field_inferior_eq,
        "Aaa",
        "fstring <= 1",
        "WHERE fstring <= ?",
        1
    );
    test_where_sql_generator!(field_superior, "Aaa", "fstring > 1", "WHERE fstring > ?", 1);
    test_where_sql_generator!(
        field_superior_eq,
        "Aaa",
        "fstring >= 1",
        "WHERE fstring >= ?",
        1
    );

    // Null
    test_where_sql_generator!(
        field_null,
        "Aaa",
        "fstring == None",
        "WHERE fstring IS NULL",
        0
    );
    test_where_sql_generator!(
        field_not_null,
        "Aaa",
        "fstring != None",
        "WHERE fstring IS NOT NULL",
        0
    );

    //Parenthes
    test_where_sql_generator!(parenthes, "Aaa", "(fstring == 1)", "WHERE (fstring = ?)", 1);

    //Not
    test_where_sql_generator!(
        not_field,
        "Aaa",
        "!(fstring==1)",
        "WHERE NOT fstring = ?",
        1
    );

    // Bool
    test_where_sql_generator!(
        field_and_field_change_col,
        "Aaa",
        "fstring == 2 && fi32 == 3",
        "WHERE fstring = ? AND fi32col = ?",
        2
    );
    test_where_sql_generator!(
        field_or_field_change_col,
        "Aaa",
        "fstring == 2 || fi32 == 3",
        "WHERE fstring = ? OR fi32col = ?",
        2
    );

    // Foreignkey
    test_where_sql_generator!(
        fk_same_table_same_column,
        "Aaa",
        "bbb.fi32>3",
        "INNER JOIN bbb ON aaa.id=bbb.aaa_fk WHERE bbb.fi32 > ?",
        1
    );
    test_where_sql_generator!(
        fk_same_table_other_column,
        "Aaa",
        "bbb.fstring>3",
        "INNER JOIN bbb ON aaa.id=bbb.aaa_fk WHERE bbb.fstringcol > ?",
        1
    );
    test_where_sql_generator!(
        fk_other_table_same_field_and_complex_type,
        "Bbb",
        "ccc.height>3",
        "INNER JOIN ccctable ON bbb.uu=ccctable.bbb_fk WHERE ccctable.height > ?",
        1
    );
    test_where_sql_generator!(
        fk_related,
        "Aaa",
        "the_ddds.size==1",
        "INNER JOIN ddd ON aaa.id=ddd.aaa_if WHERE ddd.size = ?",
        1
    );
    test_where_sql_generator!(
        fk_many_fk_field_query_only_one_join,
        "Aaa",
        r#"bbb.fstring == "bla" && bbb.fi32>3"#,
        "INNER JOIN bbb ON aaa.id=bbb.aaa_fk WHERE bbb.fstringcol = ? AND bbb.fi32 > ?",
        2
    );
    test_where_sql_generator!(
        fk_two_different_joins,
        "Aaa",
        "bbb.fi32==1 && the_ddds.size>3",
        "INNER JOIN bbb ON aaa.id=bbb.aaa_fk INNER JOIN ddd ON aaa.id=ddd.aaa_if WHERE bbb.fi32 = ? AND ddd.size > ?",
        2    );

    test_where_sql_generator!(
        fk_many_fk_for_same_join_and_related_and_two_different_joins,
        "Aaa",
        r#"bbb.fstring == "bla" && bbb.fi32==1 && the_ddds.size>3"#,
        "INNER JOIN bbb ON aaa.id=bbb.aaa_fk INNER JOIN ddd ON aaa.id=ddd.aaa_if WHERE bbb.fstringcol = ? AND bbb.fi32 = ? AND ddd.size > ?",
        3);

    // IN
    test_where_sql_generator!(
        in_field_array,
        "Aaa",
        "fi32..[1,2,3]",
        "WHERE fi32col IN (?,?,?)",
        3
    );
    test_where_sql_generator!(
        in_fk_array,
        "Aaa",
        "the_ddds.size..[1,2,3]",
        "INNER JOIN ddd ON aaa.id=ddd.aaa_if WHERE ddd.size IN (?,?,?)",
        3
    );
    test_where_sql_generator!(
        in_field_tupple,
        "Aaa",
        "fi32..(1,2,3)",
        "WHERE fi32col IN (?,?,?)",
        3
    );
}
