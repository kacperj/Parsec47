//! Arithmetic formulas embedded in BulletML element text (e.g. `60-$rank*60`).
//!
//! Port of `formula.h` (the lazy expression tree) plus the lexer/grammar from the
//! Bison-generated `calc.cpp`. The supported language: decimal numbers, the
//! variables `$rand`, `$rank`, and `$1`..`$9`, the binary operators `+ - * /`
//! (left-associative; `+`/`-` bind looser than `*`/`/`), unary minus, and
//! parentheses. Evaluation is deferred: an [`Expr`] is evaluated against an
//! [`EvalCtx`] each time a value is needed, so `$rand` re-samples on every eval.

use crate::AppRunner;

/// A binary arithmetic operator.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

/// A parsed formula expression tree (the analogue of C++ `Formula<double>`).
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// A literal number.
    Const(f64),
    /// `$rand` — sampled from [`AppRunner::get_rand`] at eval time.
    Rand,
    /// `$rank` — read from [`EvalCtx::rank`].
    Rank,
    /// `$N` — the Nth parameter (1-based; index 0 is unused, mirroring C++).
    Param(usize),
    /// Unary negation.
    Neg(Box<Expr>),
    /// A binary operation.
    Bin(Op, Box<Expr>, Box<Expr>),
}

/// Context supplied when evaluating an [`Expr`]: the current rank, the parameter
/// vector for the active `*Ref` scope, and the host (for `$rand`).
pub struct EvalCtx<'a> {
    pub rank: f64,
    /// Parameters for `$N`; index 0 is a placeholder, real params start at 1.
    /// An empty slice means "no parameters" (every `$N` then resolves to 1).
    pub params: &'a [f64],
    pub app: &'a mut dyn AppRunner,
}

impl Expr {
    /// Evaluate this expression against `ctx`.
    pub fn value(&self, ctx: &mut EvalCtx) -> f64 {
        match self {
            Expr::Const(v) => *v,
            Expr::Rand => ctx.app.get_rand(),
            Expr::Rank => ctx.rank,
            // C++ `Param::value`: in-range -> params[id]; otherwise 1.
            Expr::Param(id) => ctx.params.get(*id).copied().unwrap_or(1.0),
            Expr::Neg(e) => -e.value(ctx),
            Expr::Bin(op, l, r) => {
                let a = l.value(ctx);
                let b = r.value(ctx);
                match op {
                    Op::Add => a + b,
                    Op::Sub => a - b,
                    Op::Mul => a * b,
                    Op::Div => a / b,
                }
            }
        }
    }
}

/// Parse a formula string into an [`Expr`]. Mirrors `calc()`: malformed or empty
/// input yields a constant `0.0` rather than failing (the engine never feeds it
/// invalid formulas in practice).
pub fn parse(src: &str) -> Expr {
    let tokens = lex(src);
    let mut p = Parser { tokens: &tokens, pos: 0 };
    let expr = p.parse_expr();
    match expr {
        Some(e) if p.pos == p.tokens.len() => e,
        _ => Expr::Const(0.0),
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Rand,
    Rank,
    Param(usize),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

// Mirrors the `yylex` in calc.cpp: skip whitespace; read a run of digits/'.' as a
// number; `$rand`/`$rank` keywords; `$` + single digit as a parameter id.
fn lex(src: &str) -> Vec<Tok> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b' ' | b'\t' | b'\r' | b'\n' => i += 1,
            b'+' => {
                out.push(Tok::Plus);
                i += 1;
            }
            b'-' => {
                out.push(Tok::Minus);
                i += 1;
            }
            b'*' => {
                out.push(Tok::Star);
                i += 1;
            }
            b'/' => {
                out.push(Tok::Slash);
                i += 1;
            }
            b'(' => {
                out.push(Tok::LParen);
                i += 1;
            }
            b')' => {
                out.push(Tok::RParen);
                i += 1;
            }
            b'$' => {
                if src[i + 1..].starts_with("rand") {
                    out.push(Tok::Rand);
                    i += 5;
                } else if src[i + 1..].starts_with("rank") {
                    out.push(Tok::Rank);
                    i += 5;
                } else {
                    // `$` followed by a single digit (C++ reads substr(0,1) as int).
                    let id = bytes
                        .get(i + 1)
                        .filter(|d| d.is_ascii_digit())
                        .map(|d| (d - b'0') as usize)
                        .unwrap_or(0);
                    out.push(Tok::Param(id));
                    i += 2;
                }
            }
            _ if c == b'.' || c.is_ascii_digit() => {
                let start = i;
                while i < bytes.len() && (bytes[i] == b'.' || bytes[i].is_ascii_digit()) {
                    i += 1;
                }
                let num = src[start..i].parse::<f64>().unwrap_or(0.0);
                out.push(Tok::Num(num));
            }
            // Unknown character: skip it (the C++ lexer would return it as a token
            // and yyparse would error out, which `calc` swallows).
            _ => i += 1,
        }
    }
    out
}

struct Parser<'a> {
    tokens: &'a [Tok],
    pos: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.pos)
    }

    // expr := term (('+' | '-') term)*
    fn parse_expr(&mut self) -> Option<Expr> {
        let mut lhs = self.parse_term()?;
        while let Some(op) = match self.peek() {
            Some(Tok::Plus) => Some(Op::Add),
            Some(Tok::Minus) => Some(Op::Sub),
            _ => None,
        } {
            self.pos += 1;
            let rhs = self.parse_term()?;
            lhs = Expr::Bin(op, Box::new(lhs), Box::new(rhs));
        }
        Some(lhs)
    }

    // term := factor (('*' | '/') factor)*
    fn parse_term(&mut self) -> Option<Expr> {
        let mut lhs = self.parse_factor()?;
        while let Some(op) = match self.peek() {
            Some(Tok::Star) => Some(Op::Mul),
            Some(Tok::Slash) => Some(Op::Div),
            _ => None,
        } {
            self.pos += 1;
            let rhs = self.parse_factor()?;
            lhs = Expr::Bin(op, Box::new(lhs), Box::new(rhs));
        }
        Some(lhs)
    }

    // factor := '-' factor | '(' expr ')' | NUM | $rand | $rank | $N
    fn parse_factor(&mut self) -> Option<Expr> {
        match self.peek()? {
            Tok::Minus => {
                self.pos += 1;
                Some(Expr::Neg(Box::new(self.parse_factor()?)))
            }
            Tok::LParen => {
                self.pos += 1;
                let e = self.parse_expr()?;
                match self.peek() {
                    Some(Tok::RParen) => {
                        self.pos += 1;
                        Some(e)
                    }
                    _ => None,
                }
            }
            Tok::Num(v) => {
                let v = *v;
                self.pos += 1;
                Some(Expr::Const(v))
            }
            Tok::Rand => {
                self.pos += 1;
                Some(Expr::Rand)
            }
            Tok::Rank => {
                self.pos += 1;
                Some(Expr::Rank)
            }
            Tok::Param(id) => {
                let id = *id;
                self.pos += 1;
                Some(Expr::Param(id))
            }
            _ => None,
        }
    }
}
