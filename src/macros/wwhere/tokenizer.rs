use syn::{BinOp, Expr, ExprBinary};

use super::tok::Toks;
use super::totok::ToTok;

#[derive(Debug, Clone)]
pub(crate) enum WhereTokenizer {
    Mono(syn::Expr),
    Binary(syn::ExprBinary),
}

impl syn::parse::Parse for WhereTokenizer {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let forked = input.fork();
        match input.parse::<ExprBinary>() {
            Ok(binary) => Ok(WhereTokenizer::Binary(binary)),
            Err(_) => match forked.parse::<syn::Expr>() {
                Ok(expr) => Ok(WhereTokenizer::Mono(expr)),
                Err(e) => return Err(syn::Error::new(e.span(), "Not a valid where expression")),
            },
        }
    }
}

pub(crate) fn parse_binary_eq(left: &Expr, op: &BinOp, right: &Expr, acc: &mut Toks) {
    // check for Null pattern
    if let Expr::Path(p) = right {
        if let Some(ident) = p.path.get_ident() {
            if ident == "None" {
                match left {
                    Expr::Field(_) | Expr::Path(_) => match op {
                        BinOp::Eq(_) | BinOp::Ne(_) => {
                            acc.null(&left, &op);
                            return;
                        }

                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                }
            }
        }
    }
    // regular
    left.as_param(acc);
    parse_operator(op, acc);
    right.as_value(acc);
}

pub(crate) fn parse_binary_comp(left: &Expr, op: &BinOp, right: &Expr, acc: &mut Toks) {
    // This seems ot be Between vi ternary comparison
    // if let Some(v) = parse_between(left, op, right, acc) {
    //     if v {
    //         // ternary parsed
    //         return;
    //     } else {
    //         // a usual
    left.as_param(acc);
    parse_operator(op, acc);
    right.as_value(acc);
    // }
    // }
}

pub(crate) fn parse_binary_bool(left: &Expr, op: &BinOp, right: &Expr, acc: &mut Toks) {
    parse_binary_bool_member(left, acc, "before");
    parse_operator(op, acc);
    parse_binary_bool_member(right, acc, "after");
}

pub(crate) fn parse_binary_bool_member(expr: &Expr, acc: &mut Toks, err_msg: &str) {
    match expr {
        Expr::Binary(b) => b.as_param(acc),
        // Expr::Call(_) | Expr::MethodCall(_) => acc.call(expr),
        Expr::Unary(u) => u.as_param(acc),
        Expr::Paren(p) => p.as_param(acc),
        _ => acc.error(expr, &format!("Expression not supported {err_msg} and/or")),
    }
}

pub(crate) fn op_to_str(op: &BinOp) -> &str {
    match op {
        BinOp::Eq(_) => "==",
        BinOp::Ne(_) => "!=",
        BinOp::Le(_) => "<=",
        BinOp::Lt(_) => "<",
        BinOp::Ge(_) => ">=",
        BinOp::Gt(_) => ">",
        BinOp::And(_) => "&&",
        BinOp::Or(_) => "||",
        _ => unimplemented!(),
    }
}

pub(crate) fn parse_operator(op: &BinOp, acc: &mut Toks) {
    acc.sign(op_to_str(op))
}

// return Some:true=succes, Some:false=not a ternary, None:error
// pub(crate) fn parse_between(left: &Expr, op: &BinOp, right: &Expr, acc: &mut Toks) -> Option<bool> {
//     let mut flag = Some(true);
//     match left {
//         // This seems ot be Between vi ternary comparison
//         Expr::Binary(lhs) =>         //_ => left.as_param(acc),
//         {
//             let mut toks = Toks::default();
//             lhs.left.as_value(& mut toks);
//             match lhs.op {
//                 BinOp::Lt(_) | BinOp::Le(_) | BinOp::Gt(_) | BinOp::Ge(_) => {
//                     parse_operator(&lhs.op, &mut toks)
//                 }

//                 _ => {
//                     acc.error(lhs.op, "Only <, >, <=,>= allowed in ternary comparison");
//                     flag = None;
//                 }
//             };
//             lhs.right.as_param(&mut toks);//midle in ternary is the param, others are value
//             parse_operator(op, &mut toks);
//             right.as_value(&mut toks);
//             acc.between(toks);
//         }
//         _=> {flag=Some(false);}
//     };

//     // if let Some(true) = flag {}
//     flag
// }

#[cfg(test)]
mod wwhere {
    use super::*;
    macro_rules! parse_wwhere {
        ($titre:ident,$a:literal) => {
            #[test]
            fn $titre() {
                let input: WhereTokenizer = syn::parse_str($a).unwrap();
                let toks: Toks = (&input).into();
                assert_eq!(toks.to_string(), $a)
            }
        };
        ($fz:ident,$i:literal,$o:literal) => {
            #[test]
            fn $fz() {
                let input: WhereTokenizer = syn::parse_str($i).unwrap();
                let toks: Toks = (&input).into();
                assert_eq!(toks.to_string(), $o)
            }
            // }
        };
    }
    //Eq Not Eq
    parse_wwhere!(eq_field, "a == 1", "a==1");
    parse_wwhere!(eq_simple_fk_field, "a.b == 1", "a.b==1");
    parse_wwhere!(eq_simple_array, "a.b == [1,2,3]", "a.b==[1,2,3]");
    parse_wwhere!(eq_simple_index, "a.b == bla[1]", "a.b==bla[1]");
    parse_wwhere!(eq_simple_reference, "a == &bla", "a==&bla");
    parse_wwhere!(eq_simple_path, "a == bla", "a==bla");
    parse_wwhere!(eq_long_path, "a == bla::bli", "a==bla::bli");
    parse_wwhere!(eq_simple_tuple, "a == (1,2,3)", "a==(1,2,3)");
    parse_wwhere!(neq_field, "a != 1", "a!=1");
    parse_wwhere!(neq_simple_fk_field, "a.b != 1", "a.b!=1");
    parse_wwhere!(
        eq_parenth_right,
        "a==1 && (b==2 || c==3)",
        "a==1&&(b==2||c==3)"
    );

    // None
    parse_wwhere!(none_field, "a == None", "a==None");
    parse_wwhere!(none_fk, "a.b == None", "a.b==None");
    parse_wwhere!(not_none, "a != None", "a!=None");
    parse_wwhere!(not_none_fk, "a.b != None", "a.b!=None");
    // parse_bin!(not_on_a_whole_binary, "a = 1 && !a == 2", "a==1&&!a==2");

    // Comparison
    parse_wwhere!(lt_field, "a < 1", "a<1");
    parse_wwhere!(gt_field, "a > 1", "a>1");
    parse_wwhere!(le_field, "a <= 1", "a<=1");
    parse_wwhere!(ge_field, "a >= 1", "a>=1");
    // parse_bin!(lt_ternary_field, "1 < a < 1", "1<a<1");
    // parse_bin!(lt_ternary_field_relation, "1 < a.b < 1", "1<a.b<1");
    // parse_bin!(ternary_field, "4 <= a >= 1", "4<=a>=1");
    // parse_bin!(reverse_ternary_field, "1 >= a <= 4", "1>=a<=4");
    // parse_bin!(
    //     bad_signe_in_ternary,
    //     "1 == E <= 2",
    //     "Only <, >, <=,>= allowed in ternary comparison1E"
    // );
    // parse_bin!(ternary_with_path_as_value, "a < b < c", "a<b<c");

    // Boolean

    parse_wwhere!(and_simple, "a == 1 && b == 2", "a==1&&b==2");
    parse_wwhere!(or_simple, "a == 1 || b == 2", "a==1||b==2");
    // parse_bin!(and_call_left, "bla(lol) && b == 2", "bla(lol)&&b==2");
    // parse_bin!(and_method_left, "a.c(lol) && b == 2", "a.c(lol)&&b==2");
    // parse_bin!(and_call_right, "b == 2 && bla(lol)", "b==2&&bla(lol)");
    parse_wwhere!(not_on_binary_with_paren_left, "!(a==1)&&a==1");
    parse_wwhere!(
        and_not_binary_with_paren_right,
        "a==1 && !(a==2)",
        "a==1&&!(a==2)"
    );

    //Call & MethodCall
    // parse_bin!(and_not_call_right, "a==1 && !b()", "a==1&&!b()");
    // parse_bin!(and_not_method_right, "a==1 && !a.b()", "a==1&&!a.b()");
    // parse_bin!(and_method_right, "b == 2 && r.bla(lol)", "b==2&&r.bla(lol)");
    // parse_bin!(and_not_call_left, "!b() && a ==1", "!b()&&a==1");
    // parse_bin!(and_not_method_left, "!a.b() && a ==1", "!a.b()&&a==1");
    // parse_bin!(call_equal_left, "bla(a)==22");
    //

    //Mono
    parse_wwhere!(mono_paren, "(a==1)");
    parse_wwhere!(mono_not, "!(a==1)");

    // Divers
    parse_wwhere!(bad_left, "a[1] == 1", "Not supported as parameter==1");
    parse_wwhere!(bad_sign, "a[1] << 1", "Operator not permitted");
    parse_wwhere!(
        eq_simple_methode,
        "a == i.bla()",
        "a==Not supported as rhs of comparison expression"
    );
    parse_wwhere!(
        eq_simple_call,
        "a == bla()",
        "a==Not supported as rhs of comparison expression"
    );
    parse_wwhere!(
        not_in_eq_right,
        "a == !b()",
        "a==Not supported as rhs of comparison expression"
    );
    // parse_bin!(
    //     something_complex,
    //     "!(a==b && c==2) && ((1<c.x<=4) || a.contains(\"bla\"))",
    //     "!(a==b&&c==2)&&((1<c.x<=4)||a.contains(bla))"
    // );
}
