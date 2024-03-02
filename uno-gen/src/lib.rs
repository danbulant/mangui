use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{IdentFragment, quote, quote_spanned, ToTokens};
use quote::spanned::Spanned;

macro_rules! impl_enum_totokens {
    (
        $name:ident,
        $prefix:path
        $(, $( $variant:ident ),+)?
        $(; $( $qvariant:ident ( $( $qual:ident ),+ ) ),* )?
        $(| $( $qvariant2:ident ( $( $qual2:ident => $qtype:tt ),+ ) ),* )?
    ) => {
        impl ToTokens for $name {
            fn to_tokens(&self, stream: &mut TokenStream) {
                stream.extend(match self {
                    $(
                        $(
                            $name::$variant => {
                                quote! {
                                    $prefix::$variant
                                }
                            }
                        ),+
                    )?
                    $(
                        $(
                            $name::$qvariant($($qual),+) => {
                                quote! {
                                    $prefix::$qvariant($(#$qual),+)
                                }
                            }
                        )+
                    )?
                    $(
                        $(
                            $name::$qvariant2($($qual2),+) => {
                                quote! {
                                    $prefix::$qvariant2($( $qtype ),+)
                                }
                            }
                        )+
                    )?
                });
            }
        }        
    }
}

macro_rules! impl_struct_usersettable_totokens {
    (
        $name:ident,
        $prefix:path,
        $($variant:ident),+
        $(| $( $qvariant:ident => $qtype:tt ),* )?
    ) => {
        impl ToTokens for $name {
            fn to_tokens(&self, stream: &mut TokenStream) {
                let $name { $($variant),+, .. } = self;
                let mut substream = TokenStream::new();
                $(
                    if !$variant.is_empty() {
                        substream.extend(quote! { $variant: #$variant, });
                    }
                )+
                $(
                    $(
                        if !$qvariant.is_empty() {
                            substream.extend(quote! { $qvariant: #$qtype, });
                        }
                    ),*
                )?
                stream.extend(quote! {
                    $prefix {
                        #substream
                        ..Default::default()
                    }
                });
            }
        }        
    }
}

#[derive(Clone, Default, Debug)]
enum UserSettable<T> {
    Value(T),
    Arbitrary(TokenStream),
    #[default]
    None
}

impl<T> UserSettable<T> {
    fn require_value(&mut self) -> Result<&mut T, RuleParseError> {
        match self {
            UserSettable::Value(value) => Ok(value),
            UserSettable::Arbitrary(stream) => {
                Err(RuleParseError {
                    span: stream.__span(),
                    message: "Expected a value".to_owned()
                })
            },
            UserSettable::None => {
                Err(RuleParseError {
                    span: Span::call_site(),
                    message: "Expected a value".to_owned()
                })
            }
        }
    }
    
    fn is_empty(&self) -> bool {
        match self {
            UserSettable::Value(_) => false,
            UserSettable::Arbitrary(_) => false,
            UserSettable::None => true
        }
    }
}

impl<T: Default> UserSettable<T> {
    fn require_non_arbitrary(&mut self) -> Result<&mut T, RuleParseError> {
        match self {
            UserSettable::Value(value) => Ok(value),
            UserSettable::None => {
                *self = UserSettable::Value(Default::default());
                Ok(self.require_value()?)
            },
            UserSettable::Arbitrary(stream) => {
                Err(RuleParseError {
                    span: stream.__span(),
                    message: "Expected a value".to_owned()
                })
            }
        }
    }
}

impl<T: ToTokens> ToTokens for UserSettable<T> {
    fn to_tokens(&self, stream: &mut TokenStream) {
        stream.extend(match self {
            UserSettable::Value(value) => value.to_token_stream(),
            UserSettable::Arbitrary(stream) => stream.clone(),
            UserSettable::None => quote! { Default::default() }
        });
    }
}

#[derive(Clone, Default, Debug)]
struct Point<T> {
    x: UserSettable<T>,
    y: UserSettable<T>
}

impl<T: ToTokens> ToTokens for Point<T> {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let Point { x, y } = self;
        stream.extend(quote! {
            mangui::taffy::Point { x: #x, y: #y }
        });
    }
}

#[derive(Clone, Default, Debug)]
struct Size<T> {
    width: UserSettable<T>,
    height: UserSettable<T>
}

impl<T: ToTokens> ToTokens for Size<T> {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let Size { width, height } = self;
        stream.extend(quote! {
            mangui::taffy::Size { width: #width, height: #height }
        });
    }
}

#[derive(Clone, Default, Debug)]
struct Rect<T> {
    pub left: UserSettable<T>,
    pub right: UserSettable<T>,
    pub top: UserSettable<T>,
    pub bottom: UserSettable<T>,
}

impl<T: ToTokens> ToTokens for Rect<T> {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let Rect { left, right, top, bottom } = self;
        let mut substream = TokenStream::new();
        if !left.is_empty() {
            substream.extend(quote! { left: #left, });
        }
        if !right.is_empty() {
            substream.extend(quote! { right: #right, });
        }
        if !top.is_empty() {
            substream.extend(quote! { top: #top, });
        }
        if !bottom.is_empty() {
            substream.extend(quote! { bottom: #bottom, });
        }
        if left.is_empty() || right.is_empty() || top.is_empty() || bottom.is_empty() {
            substream.extend(quote! { ..Rect::zero() });
        }
        stream.extend(quote! {
            mangui::taffy::geometry::Rect { #substream }
        });
    }
}

#[derive(Clone, Default, Debug)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32
}

impl ToTokens for Color {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let Color { r, g, b, a } = self;
        stream.extend(quote! {
            mangui::femtovg::Color::rgba(#r, #g, #b, #a)
        });
    }
}

#[derive(Clone, Debug)]
enum Paint {
    Color(Color)
}

impl Default for Paint {
    fn default() -> Self {
        Paint::Color(Color {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 0.
        })
    }
}

impl ToTokens for Paint {
    fn to_tokens(&self, stream: &mut TokenStream) {
        stream.extend(match self {
            Paint::Color(color) => {
                quote! {
                    mangui::femtovg::Paint::color(#color)
                }
            }
        });
    }
}

#[derive(Copy, Clone, Default, Debug)]
enum Cursor {
    #[default]
    Default
}

impl_enum_totokens!(Cursor, mangui::femtovg::Cursor, Default);

#[derive(Clone, Default, Debug)]
struct Transform {
    pub position: UserSettable<Point<f32>>,
    pub scale: UserSettable<Size<f32>>,
    pub rotation: UserSettable<f32>
}

impl_struct_usersettable_totokens!(Transform, mangui::nodes::Transform, position, scale, rotation);

#[derive(Clone, Default, Debug)]
struct Style {
    pub layout: UserSettable<TaffyStyle>,
    pub cursor: UserSettable<Cursor>,
    pub background: UserSettable<Paint>,
    pub text_fill: UserSettable<Paint>,
    pub font_size: UserSettable<f32>,
    pub line_height: UserSettable<f32>,
    pub border_radius: UserSettable<f32>,
    pub transform: UserSettable<Transform>
}

impl_struct_usersettable_totokens!(
    Style,
    mangui::nodes::Style,
    layout, cursor, background, text_fill, font_size, line_height, border_radius, transform
);

#[derive(Clone, Default, Debug)]
enum Position {
    #[default]
    Relative,
    Absolute,
}

impl_enum_totokens!(Position, mangui::taffy::Position, Relative, Absolute);

#[derive(Clone, Default, Debug)]
enum Display {
    Block,
    #[default]
    Flex,
    Grid,
    None,
}

impl_enum_totokens!(Display, mangui::taffy::Display, Block, Flex, Grid, None);

#[derive(Clone, Default, Debug)]
enum Overflow {
    #[default]
    Visible,
    Clip,
    Hidden,
    Scroll,
}

impl_enum_totokens!(Overflow, mangui::taffy::Overflow, Visible, Clip, Hidden, Scroll);

#[derive(Clone, Default, Debug)]
enum LengthPercentageAuto {
    Length(f32),
    Percent(f32),
    #[default]
    Auto,
}

impl_enum_totokens!(LengthPercentageAuto, mangui::taffy::LengthPercentageAuto, Auto; Length(i), Percent(i));

#[derive(Clone, Default, Debug)]
enum Dimension {
    Length(f32),
    Percent(f32),
    #[default]
    Auto,
}

impl_enum_totokens!(Dimension, mangui::taffy::Dimension, Auto; Length(i), Percent(i));

#[derive(Clone, Debug)]
enum LengthPercentage {
    Length(f32),
    Percent(f32),
}

impl_enum_totokens!(LengthPercentage, mangui::taffy::LengthPercentage; Length(i), Percent(i));

impl Default for LengthPercentage {
    fn default() -> Self {
        LengthPercentage::Length(0.)
    }
}

#[derive(Clone, Debug)]
enum AlignItems {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

impl_enum_totokens!(AlignItems, mangui::taffy::AlignItems, Start, End, FlexStart, FlexEnd, Center, Baseline, Stretch);

type AlignSelf = AlignItems;

#[derive(Clone, Debug)]
enum AlignContent {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}
type JustifyContent = AlignContent;

impl_enum_totokens!(AlignContent, mangui::taffy::AlignContent, Start, End, FlexStart, FlexEnd, Center, Stretch, SpaceBetween, SpaceEvenly, SpaceAround);

#[derive(Clone, Default, Debug)]
enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl_enum_totokens!(FlexDirection, mangui::taffy::FlexDirection, Row, Column, RowReverse, ColumnReverse);

#[derive(Clone, Default, Debug)]
enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

impl_enum_totokens!(FlexWrap, mangui::taffy::FlexWrap, NoWrap, Wrap, WrapReverse);


#[derive(Clone, Default, Debug)]
struct Line<T> {
    pub start: T,
    pub end: T,
}

impl<T: ToTokens> ToTokens for Line<T> {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let Line { start, end } = self;
        stream.extend(quote! {
            mangui::taffy::Line::new(#start, #end)
        });
    }
}

#[derive(Clone, Debug)]
struct MinMax<Min, Max> {
    pub min: Min,
    pub max: Max,
}

impl<Min: ToTokens, Max: ToTokens> ToTokens for MinMax<Min, Max> {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let MinMax { min, max } = self;
        stream.extend(quote! {
            mangui::taffy::MinMax::new(#min, #max)
        });
    }
}

#[derive(Clone, Debug)]
enum MinTrackSizingFunction {
    Fixed(LengthPercentage),
    MinContent,
    MaxContent,
    Auto,
}

impl_enum_totokens!(MinTrackSizingFunction, mangui::taffy::MinTrackSizingFunction, MinContent, MaxContent, Auto; Fixed(i));

#[derive(Clone, Debug)]
enum MaxTrackSizingFunction {
    Fixed(LengthPercentage),
    MinContent,
    MaxContent,
    FitContent(LengthPercentage),
    Auto,
    Fraction(f32),
}
type NonRepeatedTrackSizingFunction = MinMax<MinTrackSizingFunction, MaxTrackSizingFunction>;

impl_enum_totokens!(MaxTrackSizingFunction, mangui::taffy::MaxTrackSizingFunction, MinContent, MaxContent, Auto; FitContent(i), Fraction(i), Fixed(i));

#[derive(Clone, Debug)]
enum GridTrackRepetition {
    AutoFill,
    AutoFit,
    Count(u16),
}

impl_enum_totokens!(GridTrackRepetition, mangui::taffy::GridTrackRepetition, AutoFill, AutoFit; Count(i));

#[derive(Clone, Debug)]
enum TrackSizingFunction {
    Single(NonRepeatedTrackSizingFunction),
    Repeat(GridTrackRepetition, Vec<NonRepeatedTrackSizingFunction>),
}

impl_enum_totokens!(TrackSizingFunction, mangui::taffy::TrackSizingFunction; Single(i) | Repeat(repeat => (#repeat), functions => (vec![#( #functions ),*])));

#[derive(Clone, Default, Debug)]
enum GridAutoFlow {
    #[default]
    Row,
    Column,
    RowDense,
    ColumnDense,
}

impl_enum_totokens!(GridAutoFlow, mangui::taffy::GridAutoFlow, Row, Column, RowDense, ColumnDense);

#[derive(Clone, Debug)]
struct GridLine(i16);

impl ToTokens for GridLine {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let GridLine(line) = self;
        stream.extend(quote! {
            mangui::taffy::GridLine::new(#line)
        });
    }
}

#[derive(Clone, Default, Debug)]
enum GridPlacement {
    #[default]
    Auto,
    Line(GridLine),
    Span(u16),
}

impl_enum_totokens!(GridPlacement, mangui::taffy::GridPlacement, Auto; Line(i), Span(i));

/// Styles for positioning. Note that grid template rows/columns and auto rows/columns are not supported yet (generated)
#[derive(Clone, Default, Debug)]
struct TaffyStyle {
    pub display: UserSettable<Display>,
    pub overflow: UserSettable<Point<Overflow>>,
    pub scrollbar_width: UserSettable<f32>,
    pub position: UserSettable<Position>,
    pub inset: UserSettable<Rect<LengthPercentageAuto>>,
    pub size: UserSettable<Size<Dimension>>,
    pub min_size: UserSettable<Size<Dimension>>,
    pub max_size: UserSettable<Size<Dimension>>,
    pub aspect_ratio: UserSettable<f32>,
    pub margin: UserSettable<Rect<LengthPercentageAuto>>,
    pub padding: UserSettable<Rect<LengthPercentage>>,
    pub border: UserSettable<Rect<LengthPercentage>>,
    pub align_items: UserSettable<AlignItems>,
    pub align_self: UserSettable<AlignSelf>,
    pub justify_items: UserSettable<AlignItems>,
    pub justify_self: UserSettable<AlignSelf>,
    pub align_content: UserSettable<AlignContent>,
    pub justify_content: UserSettable<JustifyContent>,
    pub gap: UserSettable<Size<LengthPercentage>>,
    pub flex_direction: UserSettable<FlexDirection>,
    pub flex_wrap: UserSettable<FlexWrap>,
    pub flex_basis: UserSettable<Dimension>,
    pub flex_grow: UserSettable<f32>,
    pub flex_shrink: UserSettable<f32>,
    pub grid_template_rows: UserSettable<Vec<TrackSizingFunction>>,
    pub grid_template_columns: UserSettable<Vec<TrackSizingFunction>>,
    pub grid_auto_rows: UserSettable<Vec<NonRepeatedTrackSizingFunction>>,
    pub grid_auto_columns: UserSettable<Vec<NonRepeatedTrackSizingFunction>>,
    pub grid_auto_flow: UserSettable<GridAutoFlow>,
    pub grid_row: UserSettable<Line<GridPlacement>>,
    pub grid_column: UserSettable<Line<GridPlacement>>,
}

impl_struct_usersettable_totokens!(
    TaffyStyle,
    mangui::taffy::Style,
    display, overflow, scrollbar_width, position, inset, size, min_size, max_size,
    aspect_ratio, margin, padding, border,
    align_items, align_self, justify_items, justify_self, align_content, justify_content,
    gap,
    flex_direction, flex_wrap, flex_basis, flex_grow, flex_shrink,
    grid_auto_flow, grid_row, grid_column
);

trait ValueToUserSettable<T> {
    fn to_user_settable(self, span: Span, inverse: bool) -> Result<UserSettable<T>, RuleParseError>;
}

impl ValueToUserSettable<f32> for TokenTree {
    fn to_user_settable(self, _span: Span, inverse: bool) -> Result<UserSettable<f32>, RuleParseError> {
        match self {
            TokenTree::Literal(lit) => {
                let text = lit.to_string();
                let mut lit = text.parse::<f32>().map_err(|_| RuleParseError {
                    span: lit.span(),
                    message: "Expected a float".to_owned()
                })?;
                if inverse {
                    lit = -lit;
                }
                Ok(UserSettable::Value(lit))
            },
            TokenTree::Group(group) => {
                if inverse {
                    return Err(RuleParseError {
                        span: group.span(),
                        message: "Groups are not invertible".to_owned()
                    })
                }
                Ok(UserSettable::Arbitrary(group.stream()))
            },
            _ => {
                Err(RuleParseError {
                    span: self.span(),
                    message: "Expected a literal or a group".to_owned()
                })
            }
        }
    }
}

impl ValueToUserSettable<LengthPercentage> for TokenTree {
    fn to_user_settable(self, span: Span, inverse: bool) -> Result<UserSettable<LengthPercentage>, RuleParseError> {
        match self.to_user_settable(span, inverse)? {
            UserSettable::Value(value) => Ok(UserSettable::Value(LengthPercentage::Length(value))),
            UserSettable::Arbitrary(stream) => {
                Ok(UserSettable::Arbitrary(stream))
            },
            UserSettable::None => {
                Err(RuleParseError {
                    span,
                    message: "Expected a value".to_owned()
                })
            }

        }
    }
}

impl ValueToUserSettable<LengthPercentageAuto> for TokenTree {
    fn to_user_settable(self, span: Span, inverse: bool) -> Result<UserSettable<LengthPercentageAuto>, RuleParseError> {
        match &self {
            TokenTree::Literal(lit) if lit.to_string() == "auto" => {
                if inverse {
                    return Err(RuleParseError {
                        span: lit.span(),
                        message: "auto is not invertible".to_owned()
                    })
                }
                return Ok(UserSettable::Value(LengthPercentageAuto::Auto))
            },
            _ => {}
        }
        match self.to_user_settable(span, inverse)? {
            UserSettable::Value(value) => Ok(UserSettable::Value(LengthPercentageAuto::Length(value))),
            UserSettable::Arbitrary(stream) => {
                Ok(UserSettable::Arbitrary(stream))
            },
            UserSettable::None => {
                Err(RuleParseError {
                    span,
                    message: "Expected a value".to_owned()
                })
            }

        }
    }
}

impl ValueToUserSettable<Overflow> for TokenTree {
    fn to_user_settable(self, span: Span, inverse: bool) -> Result<UserSettable<Overflow>, RuleParseError> {
        if inverse {
            return Err(RuleParseError {
                span,
                message: "overflow is not invertible".to_owned()
            })
        }
        match self {
            TokenTree::Ident(ident) => {
                match ident.to_string().as_str() {
                    "visible" => Ok(UserSettable::Value(Overflow::Visible)),
                    "hidden" => Ok(UserSettable::Value(Overflow::Hidden)),
                    "scroll" => Ok(UserSettable::Value(Overflow::Scroll)),
                    "clip" => Ok(UserSettable::Value(Overflow::Clip)),
                    _ => {
                        Err(RuleParseError {
                            span: ident.span(),
                            message: "Expected a valid value (one of visible, hidden, scroll or clip)".to_owned()
                        })
                    }
                }
            },
            _ => {
                Err(RuleParseError {
                    span: self.span(),
                    message: "Expected a valid value (one of visible, hidden, scroll or clip)".to_owned()
                })
            }
        }
    }
}

impl ValueToUserSettable<TaffyStyle> for TokenTree {
    fn to_user_settable(self, _span: Span, inverse: bool) -> Result<UserSettable<TaffyStyle>, RuleParseError> {
        match self {
            TokenTree::Group(group) => {
                if inverse {
                    return Err(RuleParseError {
                        span: group.span(),
                        message: "Groups are not invertible".to_owned()
                    })
                }
                Ok(UserSettable::Arbitrary(group.stream()))
            },
            _ => {
                Err(RuleParseError {
                    span: self.span(),
                    message: "Expected a group - layout doesn't support literal values".to_owned()
                })
            }
        }
    }
}

impl<T: ValueToUserSettable<Y>, Y> ValueToUserSettable<Y> for Option<T> {
    fn to_user_settable(self, span: Span, inverse: bool) -> Result<UserSettable<Y>, RuleParseError> {
        match self {
            Some(value) => value.to_user_settable(span, inverse),
            None => Err(RuleParseError {
                span,
                message: "Expected a value".to_owned()
            })
        }
    }
}

#[derive(Clone, Debug)]
struct Rule {
    prefix: Option<String>,
    inverse: bool,
    prefix_span: Span,
    name: String,
    name_span: Span,
    value: Option<TokenTree>
}

struct RuleParseError {
    span: Span,
    message: String
}

#[proc_macro]
pub fn uno(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = TokenStream::from(item);
    dbg!(&item);
    let rules = parse_rules(item);

    let rules = match rules {
        Ok(rules) => rules,
        Err(err) => {
            let RuleParseError { span, message } = err;
            return quote_spanned!(span => {
                compile_error!(#message);
            }).into();
        }
    };

    let style = match process_rules(rules) {
        Ok(style) => style,
        Err(err) => {
            let RuleParseError { span, message } = err;
            return quote_spanned!(span => {
                compile_error!(#message);
            }).into();
        }
    };

    dbg!(&style);

    style.to_token_stream().into()
}

fn process_rules(rules: Vec<Rule>) -> Result<Style, RuleParseError> {
    let mut style = Style::default();
    dbg!(&rules);

    for rule in rules {
        let Rule {
            prefix,
            prefix_span,
            name,
            name_span,
            value,
            inverse
        } = rule;

        // todo: handle passing single objects to certain attributes (p, m, gap, overflow, etc)
        match name.as_str() {
            "flex" => {
                // todo: support certain identifiers like col, row, etc
                if let Some(value) = value {
                    let value = value.to_user_settable(name_span, inverse)?;
                    style.layout.require_non_arbitrary()?.flex_grow = value.clone();
                    style.layout.require_non_arbitrary()?.flex_shrink = value;
                } else {
                    style.layout.require_non_arbitrary()?.display = UserSettable::Value(Display::Flex);
                }
            },
            "p" => {
                let value = value.to_user_settable(name_span, inverse)?;
                style.layout.require_non_arbitrary()?.padding.require_non_arbitrary()?.top = value.clone();
                style.layout.require_non_arbitrary()?.padding.require_non_arbitrary()?.right = value.clone();
                style.layout.require_non_arbitrary()?.padding.require_non_arbitrary()?.left = value.clone();
                style.layout.require_non_arbitrary()?.padding.require_non_arbitrary()?.bottom = value;
            },
            "m" => {
                let value = value.to_user_settable(name_span, inverse)?;
                style.layout.require_non_arbitrary()?.margin.require_non_arbitrary()?.top = value.clone();
                style.layout.require_non_arbitrary()?.margin.require_non_arbitrary()?.right = value.clone();
                style.layout.require_non_arbitrary()?.margin.require_non_arbitrary()?.left = value.clone();
                style.layout.require_non_arbitrary()?.margin.require_non_arbitrary()?.bottom = value;
            },
            "pt" | "pr" | "pl" | "pb" => {
                let value = value.to_user_settable(name_span, inverse)?;
                match name.as_str() {
                    "pt" => style.layout.require_non_arbitrary()?.padding.require_non_arbitrary()?.top = value,
                    "pr" => style.layout.require_non_arbitrary()?.padding.require_non_arbitrary()?.right = value,
                    "pl" => style.layout.require_non_arbitrary()?.padding.require_non_arbitrary()?.left = value,
                    "pb" => style.layout.require_non_arbitrary()?.padding.require_non_arbitrary()?.bottom = value,
                    _ => {}
                }
            },
            "mt" | "ml" | "mr" | "mb" => {
                let value = value.to_user_settable(name_span, inverse)?;
                match name.as_str() {
                    "mt" => style.layout.require_non_arbitrary()?.margin.require_non_arbitrary()?.top = value,
                    "mr" => style.layout.require_non_arbitrary()?.margin.require_non_arbitrary()?.right = value,
                    "ml" => style.layout.require_non_arbitrary()?.margin.require_non_arbitrary()?.left = value,
                    "mb" => style.layout.require_non_arbitrary()?.margin.require_non_arbitrary()?.bottom = value,
                    _ => {}
                }
            },
            "gap" => {
                let value = value.to_user_settable(name_span, inverse)?;
                style.layout.require_non_arbitrary()?.gap.require_non_arbitrary()?.width = value.clone();
                style.layout.require_non_arbitrary()?.gap.require_non_arbitrary()?.height = value;
            },
            "overflow" => {
                let value = value.to_user_settable(name_span, inverse)?;
                style.layout.require_non_arbitrary()?.overflow.require_non_arbitrary()?.x = value.clone();
                style.layout.require_non_arbitrary()?.overflow.require_non_arbitrary()?.y = value;
            },
            "overflow_x" => {
                let value = value.to_user_settable(name_span, inverse)?;
                style.layout.require_non_arbitrary()?.overflow.require_non_arbitrary()?.x = value;
            },
            "overflow_y" => {
                let value = value.to_user_settable(name_span, inverse)?;
                style.layout.require_non_arbitrary()?.overflow.require_non_arbitrary()?.y = value;
            },
            "layout" => {
                if let Some(value) = value {
                    let value = value.to_user_settable(name_span, inverse)?;
                    style.layout = value;
                } else {
                    return Err(RuleParseError {
                        span: name_span,
                        message: "Expected a value as a reference to TaffyStyle object".to_owned()
                    });
                }
            },
            _ => {
                return Err(RuleParseError {
                    span: name_span,
                    message: "Unknown rule".to_owned()
                });
            }
        }
    }

    Ok(style)
}

/// Parse the rules from the input
///
/// Example rules:
///
/// flex p-5px mt-1 mb-2 ml-[i] hover:test overflow-hidden flex-col mb-1/2
fn parse_rules(item: TokenStream) -> Result<Vec<Rule>, RuleParseError> {
    let mut rules = vec![];
    let mut prefix = None;
    let mut prefix_span = Span::call_site();
    let mut name = String::new();
    let mut name_span = Span::call_site();
    let mut value = None;
    let mut should_parse_value = false;
    let mut inverse = false;

    for token in item {
        match token {
            TokenTree::Ident(ident) => {
                if should_parse_value {
                    value = Some(TokenTree::Ident(ident));
                    should_parse_value = false;

                    rules.push(Rule {
                        prefix: prefix.take(),
                        prefix_span,
                        name: std::mem::take(&mut name),
                        name_span,
                        value: value.take(),
                        inverse
                    });
                } else if prefix.is_some() && name.is_empty() {
                    name = ident.to_string();
                    name_span = ident.span();
                } else {
                    if !name.is_empty() {
                        rules.push(Rule {
                            prefix: prefix.take(),
                            prefix_span,
                            name,
                            name_span,
                            value: None,
                            inverse
                        });
                    }
                    name = ident.to_string();
                    name_span = ident.span();
                }
                inverse = false;
                prefix_span = Span::call_site();
            },
            TokenTree::Punct(punct) => {
                if punct.as_char() == '-' {
                    if name.is_empty() {
                        inverse = true;
                    } else {
                        should_parse_value = true;
                    }
                } else if punct.as_char() == ':' {
                    prefix = Some(name);
                    prefix_span = name_span;
                    name = String::new();
                } else if punct.as_char() == '/' {
                    return Err(RuleParseError {
                        span: punct.span(),
                        message: "Fractions are not supported yet".to_owned()
                    });
                } else {
                    return Err(RuleParseError {
                        span: punct.span(),
                        message: "Unexpected punctuation".to_owned()
                    });
                }
            },
            TokenTree::Literal(_) | TokenTree::Group(_) => {
                if !should_parse_value {
                    // error
                    return Err(RuleParseError {
                        span: token.span(),
                        message: "Unexpected value, expected literal (rule start)".to_owned()
                    });
                }
                value = Some(token);

                rules.push(Rule {
                    prefix: prefix.take(),
                    prefix_span,
                    name: std::mem::take(&mut name),
                    name_span,
                    value: value.take(),
                    inverse
                });
                should_parse_value = false;
                inverse = false;
                prefix_span = Span::call_site();
            },
        }
    }

    if !name.is_empty() {
        rules.push(Rule {
            prefix: prefix.take(),
            prefix_span,
            name,
            name_span,
            value: None,
            inverse
        });
    }

    Ok(rules)
}