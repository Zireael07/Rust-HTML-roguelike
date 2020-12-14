/// We start by defining the types that define the shape of data that we want.
/// In this case, we want something tree-like

/// Starting from the most basic, we define some built-in functions that our lisp has
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BuiltIn {
  Plus,
  Minus,
  Times,
  Divide,
  Equal,
  Not,
}

/// We now wrap this type and a few other primitives into our Atom type.
/// Remember from before that Atoms form one half of our language.

#[derive(Debug, PartialEq, Clone)]
pub enum Atom {
  Num(i32),
  Keyword(String),
  Boolean(bool),
  BuiltIn(BuiltIn),
}

/// The remaining half is Lists. We implement these as recursive Expressions.
/// For a list of numbers, we have `'(1 2 3)`, which we'll parse to:
/// ```
/// Expr::Quote(vec![Expr::Constant(Atom::Num(1)),
///                  Expr::Constant(Atom::Num(2)),
///                  Expr::Constant(Atom::Num(3))])
/// Quote takes an S-expression and prevents evaluation of it, making it a data
/// structure that we can deal with programmatically. Thus any valid expression
/// is also a valid data structure in Lisp itself.

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
  Constant(Atom),
  /// (func-name arg1 arg2)
  Application(Box<Expr>, Vec<Expr>),
  /// (if predicate do-this)
  If(Box<Expr>, Box<Expr>),
  /// (if predicate do-this otherwise-do-this)
  IfElse(Box<Expr>, Box<Expr>, Box<Expr>),
  /// '(3 (if (+ 3 3) 4 5) 7)
  Quote(Vec<Expr>),
}