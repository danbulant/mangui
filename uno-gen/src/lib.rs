use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{IdentFragment, quote_spanned};
use quote::spanned::Spanned;

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

#[derive(Clone, Default, Debug)]
struct Point<T> {
    x: UserSettable<T>,
    y: UserSettable<T>
}

#[derive(Clone, Default, Debug)]
struct Size<T> {
    width: UserSettable<T>,
    height: UserSettable<T>
}

#[derive(Clone, Default, Debug)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32
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

#[derive(Copy, Clone, Default, Debug)]
enum Cursor {
    #[default]
    Default
}

#[derive(Clone, Default, Debug)]
struct Transform {
    pub position: UserSettable<Point<f32>>,
    pub scale: UserSettable<Size<f32>>,
    pub rotation: UserSettable<f32>
}

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

#[derive(Clone, Default, Debug)]
struct Rect<T> {
    pub left: UserSettable<T>,
    pub right: UserSettable<T>,
    pub top: UserSettable<T>,
    pub bottom: UserSettable<T>,
}

#[derive(Clone, Default, Debug)]
enum Position {
    #[default]
    Relative,
    Absolute,
}

#[derive(Clone, Default, Debug)]
enum Display {
    Block,
    #[default]
    Flex,
    Grid,
    None,
}

#[derive(Clone, Default, Debug)]
enum Overflow {
    #[default]
    Visible,
    Clip,
    Hidden,
    Scroll,
}

#[derive(Clone, Default, Debug)]
enum LengthPercentageAuto {
    Length(f32),
    Percent(f32),
    #[default]
    Auto,
}

#[derive(Clone, Default, Debug)]
enum Dimension {
    Length(f32),
    Percent(f32),
    #[default]
    Auto,
}

#[derive(Clone, Debug)]
enum LengthPercentage {
    Length(f32),
    Percent(f32),
}

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

#[derive(Clone, Default, Debug)]
enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Clone, Default, Debug)]
enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}


#[derive(Clone, Default, Debug)]
struct Line<T> {
    pub start: T,
    pub end: T,
}

#[derive(Clone, Debug)]
struct MinMax<Min, Max> {
    pub min: Min,
    pub max: Max,
}
#[derive(Clone, Debug)]
enum MinTrackSizingFunction {
    Fixed(LengthPercentage),
    MinContent,
    MaxContent,
    Auto,
}
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
#[derive(Clone, Debug)]
enum GridTrackRepetition {
    AutoFill,
    AutoFit,
    Count(u16),
}

#[derive(Clone, Debug)]
enum TrackSizingFunction {
    Single(NonRepeatedTrackSizingFunction),
    Repeat(GridTrackRepetition, Vec<NonRepeatedTrackSizingFunction>),
}

#[derive(Clone, Default, Debug)]
enum GridAutoFlow {
    #[default]
    Row,
    Column,
    RowDense,
    ColumnDense,
}

#[derive(Clone, Debug)]
struct GridLine(i16);
#[derive(Clone, Default, Debug)]
enum GridPlacement {
    #[default]
    Auto,
    Line(GridLine),
    Span(u16),
}
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
    let mut output = TokenStream::new();
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

    dbg!(style);

    output.into()
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