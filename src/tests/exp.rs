// Copyright 2025 Cameron Swords
// SPDX-License-Identifier: Apache-2.0

use crate::*;

// -------------------------------------------------------------------------------------------------
// Expr
// -------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Exp {
    Var(String),
    Lam {
        param: String,
        body: Box<Exp>,
    },
    App {
        fun: Box<Exp>,
        arg: Box<Exp>,
    },
    Let {
        name: String,
        value: Box<Exp>,
        body: Box<Exp>,
    },
}

impl Exp {
    pub fn var<S: Into<String>>(s: S) -> Self {
        Exp::Var(s.into())
    }
    pub fn lam<S: Into<String>>(param: S, body: Exp) -> Self {
        Exp::Lam {
            param: param.into(),
            body: Box::new(body),
        }
    }
    pub fn app(fun: Exp, arg: Exp) -> Self {
        Exp::App {
            fun: Box::new(fun),
            arg: Box::new(arg),
        }
    }
    pub fn let_in<S: Into<String>>(name: S, value: Exp, body: Exp) -> Self {
        Exp::Let {
            name: name.into(),
            value: Box::new(value),
            body: Box::new(body),
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Doc Helpers
// -------------------------------------------------------------------------------------------------

fn text<S: Into<String>>(s: S) -> Doc {
    Doc::text(s.into())
}

// ---- Precedence-aware pretty printer to Doc -------------------------

/// Render an expression to a `Doc` with minimal parentheses and layout hints.
/// - Precedence: `let` (0) < `λ` (1) < application (2) < atom (3)
pub fn expr_doc_pretty(e: &Exp) -> Doc {
    fn go(e: &Exp, ctx: u8) -> Doc {
        match e {
            Exp::Var(v) => text(v),

            Exp::Lam { param, body } => {
                let me = 1;
                let d = text("\\")
                    .concat(text(param))
                    .concat(text("."))
                    .concat(Doc::space())
                    .concat(go(body, me))
                    .group();
                if ctx > me {
                    Doc::parens(d)
                } else {
                    d
                }
            }

            Exp::App { fun, arg } => {
                let me = 2;
                // Application prints as grouped "fun <soft> arg", and we
                // indent the arg so line breaks align nicely.
                let d = Doc::hsep(vec![go(fun, me), go(arg, me + 1).nest(2)]).group();
                if ctx > me {
                    Doc::parens(d)
                } else {
                    d
                }
            }

            Exp::Let { name, value, body } => {
                let me = 0;
                // let x = <value>
                // in <body>
                // Both lines are in a single group so they flatten if they fit;
                // the value/body are allowed to break with indentation.
                let head = text("let")
                    .concat(Doc::space())
                    .concat(text(name))
                    .concat(Doc::space())
                    .concat(text("="));

                let line_in = Doc::line().concat(text("in")).concat(Doc::space());

                let d = head
                    .concat(Doc::space())
                    .concat(go(value, 0).nest(4).group())
                    .concat(line_in)
                    .concat(go(body, 0).nest(2))
                    .group();

                if ctx > me {
                    Doc::parens(d)
                } else {
                    d
                }
            }
        }
    }
    go(e, 0)
}

/// Compact, single-line Doc (just flattens the pretty form).
pub fn expr_doc_compact(e: &Exp) -> Doc {
    expr_doc_pretty(e).flatten()
}

// -------------------------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    // Small helper assuming your Doc has `render(width: i16) -> String`.
    // If your API differs, tweak here once.
    fn render(d: Doc, width: i16) -> String {
        d.render(width)
    }

    #[test]
    fn id_lambda() {
        let id = Exp::lam("x", Exp::var("x"));
        assert_snapshot!("id_compact", render(expr_doc_compact(&id), 120));
        assert_snapshot!("id_pretty", render(expr_doc_pretty(&id), 30));
    }

    #[test]
    fn app_associativity_left() {
        // (f x) y
        let e = Exp::app(Exp::app(Exp::var("f"), Exp::var("x")), Exp::var("y"));
        assert_snapshot!("app_left_compact", render(expr_doc_compact(&e), 120));
        assert_snapshot!("app_left_pretty", render(expr_doc_pretty(&e), 10));
    }

    #[test]
    fn app_associativity_right() {
        // f (x y)
        let e = Exp::app(Exp::var("f"), Exp::app(Exp::var("x"), Exp::var("y")));
        assert_snapshot!("app_right_compact", render(expr_doc_compact(&e), 120));
        assert_snapshot!("app_right_pretty", render(expr_doc_pretty(&e), 10));
    }

    #[test]
    fn let_simple() {
        let e = Exp::let_in("x", Exp::var("a"), Exp::app(Exp::var("f"), Exp::var("x")));
        assert_snapshot!("let_simple_compact", render(expr_doc_compact(&e), 120));
        assert_snapshot!("let_simple_pretty", render(expr_doc_pretty(&e), 20));
    }

    #[test]
    fn let_nested_lambda() {
        let e = Exp::let_in(
            "id",
            Exp::lam("x", Exp::var("x")),
            Exp::app(Exp::var("id"), Exp::var("y")),
        );
        assert_snapshot!(
            "let_nested_lambda_compact",
            render(expr_doc_compact(&e), 120)
        );
        assert_snapshot!("let_nested_lambda_pretty", render(expr_doc_pretty(&e), 12));
    }

    #[test]
    fn wide_breaking_case() {
        // Force breaks with long names
        let long = Exp::var("veryLongIdentifierThatForcesWrap");
        let e = Exp::let_in(
            "value",
            Exp::app(long.clone(), Exp::var("arg1")),
            Exp::app(Exp::app(long, Exp::var("arg2")), Exp::var("arg3")),
        );
        assert_snapshot!("wide_compact", render(expr_doc_compact(&e), 120));
        assert_snapshot!("wide_pretty", render(expr_doc_pretty(&e), 24));
    }
}
