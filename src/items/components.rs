use crate::format::{Format, Formatter, Indent, Newline};
use crate::items::keywords::WhenKeyword;
use crate::items::symbols::{
    CloseBraceSymbol, CloseParenSymbol, CloseSquareSymbol, CommaSymbol, DoubleLeftAngleSymbol,
    DoubleRightAngleSymbol, DoubleRightArrowSymbol, MapMatchSymbol, OpenBraceSymbol,
    OpenParenSymbol, OpenSquareSymbol, RightArrowSymbol, SemicolonSymbol, SharpSymbol,
};
use crate::parse::{self, Parse, ResumeParse, TokenStream};
use crate::span::{Position, Span};

pub use efmt_derive::Element;

#[derive(Debug, Clone, Span)]
pub struct Null {
    // Note that `next_token_start_position` can be larger than `prev_token_end_position`
    // because this behavior is required when using `Null` in other items
    // (please imagine the case where `Null` is at the front (or last) of an item and
    // `#[derive(Format)]` is specified on that item).
    next_token_start_position: Position,
    prev_token_end_position: Position,
}

impl Parse for Null {
    fn parse(ts: &mut parse::TokenStream) -> parse::Result<Self> {
        Ok(Self {
            next_token_start_position: ts.next_token_start_position()?,
            prev_token_end_position: ts.prev_token_end_position(),
        })
    }
}

impl Format for Null {
    fn format(&self, _: &mut Formatter) {}
}

#[derive(Debug, Clone, Span, Parse, Format)]
pub struct Maybe<T>(Either<T, Null>);

impl<T> Maybe<T> {
    pub fn some(item: T) -> Self {
        Self(Either::A(item))
    }

    pub fn parse_none(ts: &mut TokenStream) -> parse::Result<Self> {
        ts.parse().map(Either::B).map(Self)
    }

    pub fn none_from_position(position: Position) -> Self {
        Self(Either::B(Null {
            prev_token_end_position: position,
            next_token_start_position: position,
        }))
    }

    pub fn get(&self) -> Option<&T> {
        if let Either::A(x) = &self.0 {
            Some(x)
        } else {
            None
        }
    }
}

impl<T: Element> Element for Maybe<T> {
    fn is_packable(&self) -> bool {
        self.get().map_or(true, Element::is_packable)
    }
}

#[derive(Debug, Clone, Span, Parse, Format)]
pub enum Either<A, B> {
    A(A),
    B(B),
}

#[derive(Debug, Clone, Span, Parse)]
pub struct Parenthesized<T> {
    open: OpenParenSymbol,
    item: T,
    close: CloseParenSymbol,
}

impl<T> Parenthesized<T> {
    pub fn get(&self) -> &T {
        &self.item
    }
}

impl<T: Format> Format for Parenthesized<T> {
    fn format(&self, fmt: &mut Formatter) {
        self.open.format(fmt);
        fmt.subregion(Indent::CurrentColumn, Newline::Never, |fmt| {
            self.item.format(fmt);
        });
        self.close.format(fmt);
    }
}

#[derive(Debug, Clone, Span, Parse, Format)]
pub struct Params<T>(Parenthesized<Items<T>>);

impl<T> Params<T> {
    pub fn get(&self) -> &[T] {
        self.0.get().items()
    }
}

#[derive(Debug, Clone, Span, Parse, Format)]
pub struct Args<T>(Parenthesized<Items<T>>);

impl<T> Args<T> {
    pub fn get(&self) -> &[T] {
        self.0.get().items()
    }
}

#[derive(Debug, Clone, Span, Parse)]
pub struct CommaDelimiter(CommaSymbol);

impl Format for CommaDelimiter {
    fn format(&self, fmt: &mut Formatter) {
        self.0.format(fmt);
        fmt.add_space();
    }
}

#[derive(Debug, Clone)]
pub struct NonEmptyItems<T, D = CommaDelimiter> {
    items: Vec<T>,
    delimiters: Vec<D>,
}

impl<T, D> NonEmptyItems<T, D> {
    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn delimiters(&self) -> &[D] {
        &self.delimiters
    }
}

impl<T: Span, D> Span for NonEmptyItems<T, D> {
    fn start_position(&self) -> Position {
        self.items[0].start_position()
    }

    fn end_position(&self) -> Position {
        self.items[self.items.len() - 1].end_position()
    }
}

impl<T: Parse, D: Parse> Parse for NonEmptyItems<T, D> {
    fn parse(ts: &mut TokenStream) -> parse::Result<Self> {
        let mut items = vec![ts.parse()?];
        let mut delimiters = Vec::new();
        while let Ok(delimiter) = ts.parse() {
            delimiters.push(delimiter);
            items.push(ts.parse()?);
        }
        Ok(Self { items, delimiters })
    }
}

impl<T: Format, D: Format> Format for NonEmptyItems<T, D> {
    fn format(&self, fmt: &mut Formatter) {
        fmt.subregion(Indent::CurrentColumn, Newline::Never, |fmt| {
            let item = self.items().first().expect("unreachable");
            fmt.subregion(Indent::Inherit, Newline::Never, |fmt| item.format(fmt));
            for (item, delimiter) in self.items.iter().skip(1).zip(self.delimiters.iter()) {
                delimiter.format(fmt);
                fmt.subregion(
                    Indent::Inherit,
                    Newline::if_too_long_or_multi_line_parent(),
                    |fmt| item.format(fmt),
                );
            }
        });
    }
}

impl<T: Format, D: Format> NonEmptyItems<T, D> {
    pub fn format_multi_line(&self, fmt: &mut Formatter) {
        fmt.subregion(Indent::CurrentColumn, Newline::Never, |fmt| {
            let item = self.items().first().expect("unreachable");
            fmt.subregion(Indent::Inherit, Newline::Never, |fmt| item.format(fmt));
            for (item, delimiter) in self.items.iter().skip(1).zip(self.delimiters.iter()) {
                delimiter.format(fmt);
                fmt.subregion(Indent::Inherit, Newline::Always, |fmt| item.format(fmt));
            }
        });
    }
}

#[derive(Debug, Clone, Span, Parse, Format)]
pub struct Items<T, D = CommaDelimiter>(Maybe<NonEmptyItems<T, D>>);

impl<T, D> Items<T, D> {
    pub fn items(&self) -> &[T] {
        if let Some(x) = self.0.get() {
            x.items()
        } else {
            &[]
        }
    }

    pub fn delimiters(&self) -> &[D] {
        if let Some(x) = self.0.get() {
            x.delimiters()
        } else {
            &[]
        }
    }
}

#[derive(Debug, Clone, Span, Parse)]
pub struct MaybePackedItems<T, D = CommaDelimiter>(Items<T, D>);

impl<T: Format, D: Format> MaybePackedItems<T, D> {
    fn packed_format(&self, fmt: &mut Formatter) {
        fmt.subregion(Indent::CurrentColumn, Newline::Never, |fmt| {
            let item = self.0.items().first().expect("unreachable");
            fmt.subregion(Indent::Inherit, Newline::Never, |fmt| item.format(fmt));
            for (item, delimiter) in self
                .0
                .items()
                .iter()
                .skip(1)
                .zip(self.0.delimiters().iter())
            {
                delimiter.format(fmt);
                fmt.subregion(Indent::Inherit, Newline::if_too_long(), |fmt| {
                    item.format(fmt)
                });
            }
        });
    }
}

impl<T: Format + Element, D: Format> Format for MaybePackedItems<T, D> {
    fn format(&self, fmt: &mut Formatter) {
        if self.0.items().is_empty() {
        } else if self.0.items().iter().all(Element::is_packable) {
            fmt.subregion(Indent::CurrentColumn, Newline::Never, |fmt| {
                self.packed_format(fmt)
            });
        } else {
            self.0.format(fmt);
        }
    }
}

pub trait Element {
    fn is_packable(&self) -> bool;
}

#[derive(Debug, Clone, Span, Parse, Format)]
pub struct ListLike<T: Element, D = CommaDelimiter> {
    open: OpenSquareSymbol,
    items: MaybePackedItems<T, D>,
    close: CloseSquareSymbol,
}

#[derive(Debug, Clone, Span, Parse, Format)]
pub struct TupleLike<T: Element> {
    open: OpenBraceSymbol,
    items: MaybePackedItems<T>,
    close: CloseBraceSymbol,
}

#[derive(Debug, Clone, Span, Parse, Format)]
pub struct BitstringLike<T: Element> {
    open: DoubleLeftAngleSymbol,
    items: MaybePackedItems<T>,
    close: DoubleRightAngleSymbol,
}

#[derive(Debug, Clone, Span, Parse, Format)]
pub struct MapLike<T> {
    sharp: SharpSymbol,
    items: TupleLike<MapItem<T>>,
}

#[derive(Debug, Clone, Span, Parse, Format)]
struct MapItem<T>(BinaryOpLike<T, MapDelimiter, T>);

impl<T> Element for MapItem<T> {
    fn is_packable(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Span, Parse, Format)]
struct MapDelimiter(Either<DoubleRightArrowSymbol, MapMatchSymbol>);

impl BinaryOpStyle for MapDelimiter {
    fn indent(&self) -> Indent {
        Indent::Offset(4)
    }

    fn newline(&self) -> Newline {
        Newline::if_too_long_or_multi_line()
    }
}

#[derive(Debug, Clone, Span, Parse, Format)]
pub struct Clauses<T>(NonEmptyItems<T, SemicolonDelimiter>);

impl<T> Clauses<T> {
    pub fn items(&self) -> &[T] {
        self.0.items()
    }
}

#[derive(Debug, Clone, Span, Parse)]
pub struct SemicolonDelimiter(SemicolonSymbol);

impl Format for SemicolonDelimiter {
    fn format(&self, fmt: &mut Formatter) {
        self.0.format(fmt);
        fmt.add_newline();
    }
}

#[derive(Debug, Clone, Span, Parse, Format)]
pub struct UnaryOpLike<O, T> {
    op: O,
    item: T,
}

impl<O, T> UnaryOpLike<O, T> {
    pub fn item(&self) -> &T {
        &self.item
    }
}

pub trait BinaryOpStyle {
    fn indent(&self) -> Indent;

    fn newline(&self) -> Newline;
}

#[derive(Debug, Clone, Span, Parse)]
pub struct BinaryOpLike<L, O, R> {
    pub left: L,
    pub op: O,
    pub right: R,
}

impl<L, O, R> Element for BinaryOpLike<L, O, R> {
    fn is_packable(&self) -> bool {
        false
    }
}

impl<L: Parse, O: Parse, R: Parse> ResumeParse<L> for BinaryOpLike<L, O, R> {
    fn resume_parse(ts: &mut parse::TokenStream, left: L) -> parse::Result<Self> {
        Ok(Self {
            left,
            op: ts.parse()?,
            right: ts.parse()?,
        })
    }
}

impl<L: Format, O: Format + BinaryOpStyle, R: Format> Format for BinaryOpLike<L, O, R> {
    fn format(&self, fmt: &mut Formatter) {
        self.left.format(fmt);

        fmt.add_space();
        self.op.format(fmt);
        fmt.add_space();

        let indent = self.op.indent();
        let newline = self.op.newline();
        fmt.subregion(indent, newline, |fmt| self.right.format(fmt));
    }
}

#[derive(Debug, Clone, Span, Parse)]
pub struct WithArrow<T> {
    item: T,
    arrow: RightArrowSymbol,
}

impl<T: Format> Format for WithArrow<T> {
    fn format(&self, fmt: &mut Formatter) {
        self.item.format(fmt);
        fmt.add_space();
        self.arrow.format(fmt);
        fmt.add_space();
    }
}

#[derive(Debug, Clone, Span, Parse, Format)]
pub struct WithGuard<T, U, D = Either<CommaSymbol, SemicolonSymbol>> {
    item: T,
    guard: Maybe<Guard<U, D>>,
}

#[derive(Debug, Clone, Span, Parse)]
struct Guard<T, D> {
    when: WhenKeyword,
    conditions: NonEmptyItems<T, D>,
}

impl<T: Format, D: Format> Format for Guard<T, D> {
    fn format(&self, fmt: &mut Formatter) {
        fmt.subregion(
            Indent::Offset(2),
            Newline::if_too_long_or_multi_line(),
            |fmt| {
                fmt.add_space();
                self.when.format(fmt);
                fmt.add_space();
                self.conditions.format(fmt);
            },
        );
    }
}