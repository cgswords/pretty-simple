// Copyright 2025 Cameron Swords
// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;

use once_cell::unsync::Lazy;

mod tests;

// -------------------------------------------------------------------------------------------------
// Main Trait
// -------------------------------------------------------------------------------------------------

pub trait ToDoc {
    fn to_doc(&self) -> Doc;
    fn render(&self, width: i16) -> String {
        self.to_doc().render(width)
    }
}

// -------------------------------------------------------------------------------------------------
// Helpers
// -------------------------------------------------------------------------------------------------

/// Convert an iterator of items to a `Doc` by rendering each item with `ToDoc` and
/// interspersing `separator` between them.
///
/// Returns [`Doc::nil()`] if the iterator is empty.
///
/// # Example
/// ```rust
///use pretty_simple::*;
///
/// #[derive(Debug)]
/// struct Item(&'static str);
///
/// impl ToDoc for Item {
///     fn to_doc(&self) -> Doc { Doc::text(self.0) }
/// }
///
/// let xs = [Item("a"), Item("b"), Item("c")];
///
/// let doc = to_list(xs.iter(), Doc::text(","));
///
/// assert_eq!(doc.render(80), "a,b,c");
/// ```
pub fn to_list<'a, T>(docs: impl IntoIterator<Item = &'a T>, separator: Doc) -> Doc
where
    T: ToDoc + 'a,
{
    let mut iter = docs.into_iter();
    if let Some(first) = iter.next() {
        let mut output = first.to_doc();
        while let Some(next) = iter.next() {
            output = output.concat(separator.clone()).concat(next.to_doc());
        }
        output
    } else {
        Doc::nil()
    }
}

// -------------------------------------------------------------------------------------------------
// Documents
// -------------------------------------------------------------------------------------------------

pub struct Doc(Rc<DocInner>);

type DocFn = Rc<dyn Fn(i16) -> Doc + 'static>;

enum DocInner {
    Empty,
    Text(String),
    Line, // soft line break
    Concat(Doc, Doc),
    Nest(i16, Doc),
    Alt(Doc, Doc),
    Nesting(DocFn),
    Column(DocFn),
}

// This is a bit of an absue of notation, but it will make our lives a touch simpler.
impl DocInner {
    fn to_doc(self) -> Doc {
        Doc(Rc::new(self))
    }
}

impl Clone for Doc {
    fn clone(&self) -> Self {
        Doc(Rc::clone(&self.0))
    }
}
// -----------------------------------------------
// Thread Locals
// -----------------------------------------------

thread_local! {
    static NIL_INNER: Lazy<Rc<DocInner>> = Lazy::new(|| Rc::new(DocInner::Empty));
    static SPACE_INNER: Lazy<Rc<DocInner>> = Lazy::new(|| Rc::new(DocInner::Text(" ".to_string())));
    static COMMA_INNER: Lazy<Rc<DocInner>> = Lazy::new(|| Rc::new(DocInner::Text(",".to_string())));
    static LINE_INNER: Lazy<Rc<DocInner>> = Lazy::new(|| Rc::new(DocInner::Line));
    static SOFTLINE_INNER: Lazy<Rc<DocInner>> = Lazy::new(|| Rc::new(DocInner::Alt(Doc::space(), Doc::line())));
    static SOFTLINE_EMPTY_INNER: Lazy<Rc<DocInner>> = Lazy::new(|| Rc::new(DocInner::Alt(Doc::nil(), Doc::line())));
    static LPAREN_INNER: Lazy<Rc<DocInner>> = Lazy::new(|| Rc::new(DocInner::Text("(".to_string())));
    static RPAREN_INNER: Lazy<Rc<DocInner>> = Lazy::new(|| Rc::new(DocInner::Text(")".to_string())));
    static LANGLE_INNER: Lazy<Rc<DocInner>> = Lazy::new(|| Rc::new(DocInner::Text("<".to_string())));
    static RANGLE_INNER: Lazy<Rc<DocInner>> = Lazy::new(|| Rc::new(DocInner::Text(">".to_string())));
    static LBRACKET_INNER: Lazy<Rc<DocInner>> = Lazy::new(|| Rc::new(DocInner::Text("[".to_string())));
    static RBRACKET_INNER: Lazy<Rc<DocInner>> = Lazy::new(|| Rc::new(DocInner::Text("]".to_string())));
    static LBRACE_INNER: Lazy<Rc<DocInner>> = Lazy::new(|| Rc::new(DocInner::Text("{".to_string())));
    static RBRACE_INNER: Lazy<Rc<DocInner>> = Lazy::new(|| Rc::new(DocInner::Text("}".to_string())));
}

impl Doc {
    // -------------------------------------------
    // Core Constructors
    // -------------------------------------------

    /// The empty document.
    ///
    /// Renders to nothing and acts as the identity element for [`Doc::concat`].
    pub fn nil() -> Doc {
        NIL_INNER.with(|lazy| Doc(Rc::clone(&*lazy)))
    }

    /// A single ASCII space as a document (`" "`).
    pub fn space() -> Doc {
        SPACE_INNER.with(|lazy| Doc(Rc::clone(&*lazy)))
    }

    /// A single ASCII comma as a document (`","`).
    pub fn comma() -> Doc {
        COMMA_INNER.with(|lazy| Doc(Rc::clone(&*lazy)))
    }

    /// A hard line break.
    ///
    /// When rendered, this always breaks the line and sets the cursor to the current
    /// indentation level tracked by nesting/indentation combinators.
    pub fn line() -> Doc {
        LINE_INNER.with(|lazy| Doc(Rc::clone(&*lazy)))
    }

    /// A soft line break that becomes a space if the layout fits the given width,
    /// or a newline otherwise.
    ///
    /// This is equivalent to `Alt(space, line)` in Wadler/Leijen pretty‑printing.
    pub fn softline() -> Doc {
        SOFTLINE_INNER.with(|lazy| Doc(Rc::clone(&*lazy)))
    }

    /// A soft line break that becomes empty if the layout fits, or a newline
    /// otherwise.
    ///
    /// Useful for optional separators (e.g., trailing commas off).
    pub fn softline_empty() -> Doc {
        SOFTLINE_EMPTY_INNER.with(|lazy| Doc(Rc::clone(&*lazy)))
    }

    /// Construct a document from raw text.
    ///
    /// The string is inserted verbatim; it will not contain line breaks unless
    /// they are present in the string itself (which generally should be avoided
    /// in pretty‑printing docs).
    pub fn text<S: Into<String>>(str: S) -> Doc {
        DocInner::Text(str.into()).to_doc()
    }

    /// Concatenate two documents without inserting any separator.
    pub fn concat(self, other: Doc) -> Doc {
        DocInner::Concat(self, other).to_doc()
    }

    /// Increase the nesting (indentation) level for all lines that follow a newline
    /// within the given document by `depth` columns.
    pub fn nest(self, depth: i16) -> Doc {
        DocInner::Nest(depth, self).to_doc()
    }

    // `<+>` from Haskell
    //
    // Concatenates the two documents with a space between them.
    pub fn concat_space(self, other: Doc) -> Doc {
        self.concat(Doc::space()).concat(other)
    }

    /// Creates an `alt` set, preferring the first one if it fits and devolving to the second if it
    /// does not.
    pub fn alt(self, other: Doc) -> Doc {
        DocInner::Alt(self, other).to_doc()
    }

    /// Try to render `self` on a single line by first flattening all soft breaks;
    /// if that does not fit within the current width, fall back to the original
    /// (multi‑line) layout.
    ///
    /// This is the standard `group` combinator from pretty‑printing literature.
    pub fn group(self) -> Doc {
        match &*self.0 {
            DocInner::Alt(_, _) => self,
            _ => DocInner::Alt(self.clone().flatten(), self).to_doc(),
        }
    }

    fn flatten(self) -> Doc {
        match &*self.0 {
            DocInner::Empty | DocInner::Text(_) => self,
            DocInner::Line => Doc::space(),
            DocInner::Concat(x, y) => {
                DocInner::Concat(x.clone().flatten(), y.clone().flatten()).to_doc()
            }
            DocInner::Nest(_, inner) => inner.clone().flatten(),
            DocInner::Alt(flat, _) => flat.clone().flatten(),
            DocInner::Column(f) => {
                let f = Rc::clone(f);
                let f = Rc::new(move |i| f(i).flatten());
                Doc(Rc::new(DocInner::Column(f)))
            }
            DocInner::Nesting(f) => {
                let f = Rc::clone(f);
                let f = Rc::new(move |i| f(i).flatten());
                Doc(Rc::new(DocInner::Nesting(f)))
            }
        }
    }

    /// Create a document whose contents are computed from the **current output column**.
    ///
    /// The closure receives the current cursor column (0‑based) and returns the
    /// document to splice in at that point. The closure is stored as a `'static`
    /// callable via `Rc`, so capture owned data in it.
    ///
    /// See also [`Doc::nesting`].
    pub fn column<F>(f: F) -> Doc
    where
        F: Fn(i16) -> Doc + 'static,
    {
        let f: DocFn = Rc::new(f);
        DocInner::Column(f).to_doc()
    }

    /// Create a document whose contents are computed from the **current nesting level**.
    ///
    /// The closure receives the current indentation level (the `i` tracked by the
    /// renderer) and returns the document to splice in. Use this to align content
    /// relative to the current indent.
    ///
    /// See also [`Doc::column`].
    pub fn nesting<F>(f: F) -> Doc
    where
        F: Fn(i16) -> Doc + 'static,
    {
        let f: DocFn = Rc::new(f);
        DocInner::Nesting(f).to_doc()
    }

    // -------------------------------------------
    // Helpers
    // -------------------------------------------

    /// Fold an iterator of documents by repeatedly combining adjacent items with
    /// `concat_f`.
    ///
    /// This is a generalized form of [`hcat`](Self::hcat), [`hsep`](Self::hsep),
    /// and [`vsep`](Self::vsep). Returns [`Doc::nil()`] for an empty iterator.
    pub fn concat_with<F>(docs: impl IntoIterator<Item = Doc>, concat_f: F) -> Doc
    where
        F: Fn(Doc, Doc) -> Doc,
    {
        let mut iter = docs.into_iter();
        if let Some(first) = iter.next() {
            let mut output = first;
            while let Some(next) = iter.next() {
                output = concat_f(output, next);
            }
            output
        } else {
            Doc::nil()
        }
    }

    /// A convenience for “hanging” indentation: `self.nest(i).align()`.
    ///
    /// Subsequent lines align under the first character after an `i`‑space indent.
    pub fn hang(self, i: i16) -> Doc {
        self.nest(i).align()
    }

    /// Indent `self` by `i` spaces, and use a hanging layout so subsequent lines
    /// align under the first non‑space character.
    ///
    /// Equivalent to `Doc::spaces(i).concat(self).hang(i)`.
    pub fn indent(self, i: i16) -> Doc {
        Doc::spaces(i).concat(self).hang(i)
    }

    /// Align subsequent lines to the current column.
    ///
    /// Useful for layouts like:
    /// ```text
    /// key: value that
    ///      wraps across lines
    /// ```
    /// Internally implemented via [`Doc::column`] and [`Doc::nesting`].
    pub fn align(self) -> Doc {
        // Move an owned clone into the closures so they’re 'static.
        Doc::column({
            let base = self.clone();
            move |k| {
                let base2 = base.clone();
                Doc::nesting(move |i| base2.clone().nest(k - i))
            }
        })
    }

    /// Produce `i` spaces as a document (`" ".repeat(i)`), with fast paths for 0 and 1.
    pub fn spaces(i: i16) -> Doc {
        match i {
            0 => Doc::nil(),
            1 => Doc::space(),
            n => Doc::text(" ".repeat(n as usize)),
        }
    }

    /// Horizontally separate an iterator of documents with single spaces.
    ///
    /// Equivalent to interspersing [`Doc::space()`] and concatenating.
    pub fn hsep(docs: impl IntoIterator<Item = Doc>) -> Doc {
        Doc::concat_with(docs, |x, y| x.concat_space(y))
    }

    /// Vertically separate an iterator of documents with hard newlines.
    ///
    /// Equivalent to interspersing [`Doc::line()`] and concatenating.
    pub fn vsep(docs: impl IntoIterator<Item = Doc>) -> Doc {
        Doc::concat_with(docs, |x, y| x.concat(Doc::line()).concat(y))
    }

    // Tries laying the elements out with spaces, or vertically if they do not fit.
    pub fn sep(docs: impl IntoIterator<Item = Doc>) -> Doc {
        Doc::vsep(docs).group()
    }

    /// Concatenate an iterator of documents without separators (left‑associative).
    pub fn hcat(docs: impl IntoIterator<Item = Doc>) -> Doc {
        Doc::concat_with(docs, |x, y| x.concat(y))
    }

    /// Concatenate `docs`, inserting `separator` between each adjacent pair.
    ///
    /// Returns [`Doc::nil()`] if `docs` is empty.
    pub fn intersperse(docs: impl IntoIterator<Item = Doc>, separator: Doc) -> Doc {
        let mut iter = docs.into_iter();
        if let Some(first) = iter.next() {
            let mut output = first;
            while let Some(next) = iter.next() {
                output = output.concat(separator.clone()).concat(next);
            }
            output
        } else {
            Doc::nil()
        }
    }

    /// Surround `self` with `(` and `)` (parentheses).
    pub fn parens(self) -> Doc {
        Self::lparen().concat(self).concat(Self::rparen())
    }

    /// Surround `self` with `<` and `>` (angle brackets).
    pub fn angles(self) -> Doc {
        Self::langle().concat(self).concat(Self::rangle())
    }

    /// Surround `self` with `[` and `]` (square brackets).
    pub fn brackets(self) -> Doc {
        Self::lbracket().concat(self).concat(Self::rbracket())
    }

    /// Surround `self` with `{` and `}` (curly braces).
    pub fn braces(self) -> Doc {
        Self::lbrace().concat(self).concat(Self::rbrace())
    }

    /// Render `self` as a typical block:
    ///
    /// ```text
    /// {start}
    ///     {self (indented, grouped)}
    /// {end}
    /// ```
    ///
    /// Uses a 4‑space indent and inserts newlines before and after the block body.
    pub fn block(self, start: Doc, end: Doc) -> Doc {
        start
            .concat(Doc::line())
            .concat(self.indent(4).group())
            .concat(Doc::line())
            .concat(end)
    }

    /// Fill a la Wadler
    /// This
    pub fn fill(xs: &[Doc]) -> Doc {
        Self::fill_core(xs, 0, false)
    }

    /// `head_flat` means: treat xs[i] as already flattened (because caller passed `flatten y : zs`)
    fn fill_core(xs: &[Doc], i: usize, head_flat: bool) -> Doc {
        if i >= xs.len() {
            return Doc::nil();
        }
        let n = xs.len() - i;
        if n == 1 {
            let mut d = xs[i].clone();
            if head_flat {
                d = d.flatten();
            }
            return d;
        }

        // We have at least two: x = xs[i], y = xs[i+1]
        let x = xs[i].clone();
        let y_is_head = i + 1; // head of the recursive tail

        // Left branch: (flatten x <+> fill (flatten y : zs))
        // If the current head is already flattened, don't double-flatten.
        let x_flat = if head_flat {
            x.clone()
        } else {
            x.clone().flatten()
        };
        let left = x_flat
            .concat(Doc::space())
            // Next level's head (y) must be treated as already flattened
            .concat(Self::fill_core(xs, y_is_head, true));

        // Right branch: (x </> fill (y : zs))
        // If head_flat is true, x is already flattened; use it as-is.
        let x_for_right = if head_flat { x } else { xs[i].clone() };
        let right = x_for_right
            .concat(Doc::line())
            .concat(Self::fill_core(xs, y_is_head, false));

        left.alt(right)
    }

    // -------------------------------------------
    // Constant Constructors
    // -------------------------------------------

    /// The `<` document.
    pub fn lparen() -> Doc {
        LPAREN_INNER.with(|lazy| Doc(Rc::clone(&*lazy)))
    }

    /// The `>` document.
    pub fn rparen() -> Doc {
        RPAREN_INNER.with(|lazy| Doc(Rc::clone(&*lazy)))
    }

    /// The `<` document.
    pub fn langle() -> Doc {
        LANGLE_INNER.with(|lazy| Doc(Rc::clone(&*lazy)))
    }

    /// The `>` document.
    pub fn rangle() -> Doc {
        RANGLE_INNER.with(|lazy| Doc(Rc::clone(&*lazy)))
    }

    /// The `[` document.
    pub fn lbracket() -> Doc {
        LBRACKET_INNER.with(|lazy| Doc(Rc::clone(&*lazy)))
    }

    /// The `]` document.
    pub fn rbracket() -> Doc {
        RBRACKET_INNER.with(|lazy| Doc(Rc::clone(&*lazy)))
    }

    /// The `{` document.
    pub fn lbrace() -> Doc {
        LBRACE_INNER.with(|lazy| Doc(Rc::clone(&*lazy)))
    }

    /// The `}` document.
    pub fn rbrace() -> Doc {
        RBRACE_INNER.with(|lazy| Doc(Rc::clone(&*lazy)))
    }

    // -------------------------------------------
    // Rendering
    // -------------------------------------------

    /// Render the document to a `String` using the given maximum line `width`.
    ///
    /// Soft breaks choose between space/newline based on whether the flattened
    /// alternative fits within the remaining width; hard breaks always break.
    /// The algorithm is a variant of Wadler/Leijen pretty‑printing.
    pub fn render(self, width: i16) -> String {
        let rendered = self.best(width);
        let output = rendered.render();
        // std::mem::forget(rendered);
        output.unwrap()
    }

    fn best(self, width: i16) -> Render {
        use DocInner as DI;

        enum Cons {
            Cell { head: (i16, Doc), tail: Rc<Cons> },
            Nil,
        }

        fn cons(head: (i16, Doc), tail: Rc<Cons>) -> Rc<Cons> {
            Rc::new(Cons::Cell { head, tail })
        }

        // A non-allocating, non-recursive "does it fit?" that peeks ahead.
        // Returns false if we'd exceed `remaining` or hit a hard Line.
        fn fits(mut remaining: i16, mut cursor: i16, mut docs: Rc<Cons>) -> bool {
            while let Cons::Cell {
                head: (i, doc),
                tail,
            } = &*docs
            {
                match &*doc.0 {
                    DI::Line => return true,
                    DI::Empty => {
                        docs = tail.clone();
                    }
                    DI::Text(s) => {
                        let s_len = s.len() as i16;
                        if s_len > remaining {
                            return false;
                        };
                        remaining -= s_len;
                        cursor += s_len;
                        docs = tail.clone();
                    }
                    DI::Concat(x, y) => {
                        docs = cons((*i, x.clone()), cons((*i, y.clone()), tail.clone()));
                    }
                    DI::Nest(j, inner) => {
                        docs = cons((i + j, inner.clone()), tail.clone());
                    }
                    DI::Alt(flat, _doc2) => {
                        docs = cons((*i, flat.clone()), tail.clone());
                    }
                    DI::Column(f) => {
                        docs = cons((*i, f(cursor)), tail.clone());
                    }
                    DI::Nesting(f) => {
                        docs = cons((*i, f(*i)), tail.clone());
                    }
                }
            }
            true
        }

        let mut docs = cons((0, self), Rc::new(Cons::Nil));
        let mut cursor = 0i16;
        let mut out: Vec<RenderPart> = vec![];

        while let Cons::Cell { head, tail } = &*docs {
            let (indent, doc) = head;
            match &*doc.0 {
                DI::Empty => {
                    docs = tail.clone();
                }
                DI::Text(s) => {
                    out.push(RenderPart::Text(s.to_string()));
                    cursor = cursor + s.len() as i16;
                    docs = tail.clone();
                }
                DI::Concat(x, y) => {
                    docs = cons(
                        (*indent, x.clone()),
                        cons((*indent, y.clone()), tail.clone()),
                    );
                }
                DI::Nest(j, inner) => {
                    docs = cons((indent + j, inner.clone()), tail.clone());
                }
                DI::Line => {
                    out.push(RenderPart::Line(*indent));
                    cursor = *indent;
                    docs = tail.clone();
                }
                DI::Alt(flat, alt) => {
                    let flat = cons((*indent, flat.clone()), tail.clone());
                    if fits(width, cursor, flat.clone()) {
                        docs = flat;
                    } else {
                        docs = cons((*indent, alt.clone()), tail.clone());
                    }
                }
                DI::Column(f) => {
                    docs = cons((*indent, f(cursor)), tail.clone());
                }
                DI::Nesting(f) => {
                    docs = cons((*indent, f(*indent)), tail.clone());
                }
            }
        }

        Render(out)
    }
}

// -------------------------------------------------------------------------------------------------
// Rendering
// -------------------------------------------------------------------------------------------------

enum RenderPart {
    Line(i16),
    Text(String),
}

struct Render(Vec<RenderPart>);

impl Render {
    fn render(&self) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;
        let renders = &self.0;
        let mut output = String::new();
        for render in renders.iter() {
            match render {
                RenderPart::Line(i) => {
                    write!(&mut output, "\n")?;
                    for _n in 0..*i {
                        write!(&mut output, " ")?;
                    }
                }
                RenderPart::Text(s) => {
                    write!(&mut output, "{}", s)?;
                }
            }
        }
        Ok(output)
    }
}
