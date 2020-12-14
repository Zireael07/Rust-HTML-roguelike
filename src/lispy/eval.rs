use super::parser::*;
use super::types::*;
use nom::error::VerboseError;

/// And that's it!
/// We can now parse our entire lisp language.
///
/// But in order to make it a little more interesting, we can hack together
/// a little interpreter to take an Expr, which is really an
/// [Abstract Syntax Tree](https://en.wikipedia.org/wiki/Abstract_syntax_tree) (AST),
/// and give us something back

/// To start we define a couple of helper functions
fn get_num_from_expr(e: Expr) -> Option<i32> {
    if let Expr::Constant(Atom::Num(n)) = e {
      Some(n)
    } else {
      None
    }
}
  
fn get_bool_from_expr(e: Expr) -> Option<bool> {
    if let Expr::Constant(Atom::Boolean(b)) = e {
        Some(b)
    } else {
        None
    }
}

/// This function tries to reduce the AST.
/// This has to return an Expression rather than an Atom because quoted s_expressions
/// can't be reduced
fn eval_expression(e: Expr) -> Option<Expr> {
    match e {
        // Constants and quoted s-expressions are our base-case
        Expr::Constant(_) | Expr::Quote(_) => Some(e),
        // we then recursively `eval_expression` in the context of our special forms
        // and built-in operators
        Expr::If(pred, true_branch) => {
        let reduce_pred = eval_expression(*pred)?;
        if get_bool_from_expr(reduce_pred)? {
            eval_expression(*true_branch)
        } else {
            None
        }
        }
        Expr::IfElse(pred, true_branch, false_branch) => {
        let reduce_pred = eval_expression(*pred)?;
        if get_bool_from_expr(reduce_pred)? {
            eval_expression(*true_branch)
        } else {
            eval_expression(*false_branch)
        }
        }
        Expr::Application(head, tail) => {
        let reduced_head = eval_expression(*head)?;
        let reduced_tail = tail
            .into_iter()
            .map(|expr| eval_expression(expr))
            .collect::<Option<Vec<Expr>>>()?;
        if let Expr::Constant(Atom::BuiltIn(bi)) = reduced_head {
            Some(Expr::Constant(match bi {
            BuiltIn::Plus => Atom::Num(
                reduced_tail
                .into_iter()
                .map(get_num_from_expr)
                .collect::<Option<Vec<i32>>>()?
                .into_iter()
                .sum(),
            ),
            BuiltIn::Times => Atom::Num(
                reduced_tail
                .into_iter()
                .map(get_num_from_expr)
                .collect::<Option<Vec<i32>>>()?
                .into_iter()
                .product(),
            ),
            BuiltIn::Equal => Atom::Boolean(
                reduced_tail
                .iter()
                .zip(reduced_tail.iter().skip(1))
                .all(|(a, b)| a == b),
            ),
            BuiltIn::Not => {
                if reduced_tail.len() != 1 {
                return None;
                } else {
                Atom::Boolean(!get_bool_from_expr(reduced_tail.first().cloned().unwrap())?)
                }
            }
            BuiltIn::Minus => Atom::Num(if let Some(first_elem) = reduced_tail.first().cloned() {
                let fe = get_num_from_expr(first_elem)?;
                reduced_tail
                .into_iter()
                .map(get_num_from_expr)
                .collect::<Option<Vec<i32>>>()?
                .into_iter()
                .skip(1)
                .fold(fe, |a, b| a - b)
            } else {
                Default::default()
            }),
            BuiltIn::Divide => Atom::Num(if let Some(first_elem) = reduced_tail.first().cloned() {
                let fe = get_num_from_expr(first_elem)?;
                reduced_tail
                .into_iter()
                .map(get_num_from_expr)
                .collect::<Option<Vec<i32>>>()?
                .into_iter()
                .skip(1)
                .fold(fe, |a, b| a / b)
            } else {
                Default::default()
            }),
            }))
        } else {
            None
        }
        }
    }
}

/// And we add one more top-level function to tie everything together, letting
/// us call eval on a string directly
pub fn eval_from_str(src: &str) -> Result<Expr, String> {
parse_expr(src)
    .map_err(|e: nom::Err<VerboseError<&str>>| format!("{:#?}", e))
    .and_then(|(_, exp)| eval_expression(exp).ok_or("Eval failed".to_string()))
}