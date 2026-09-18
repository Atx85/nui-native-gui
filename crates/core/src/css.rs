use crate::{
    Color, Error, Rect,
    decoration::{Appearance, Gradient, Icon, InsetShadow, Outline, Stop},
    html::Node,
    layout::{Align, Dimension, Direction, Display, Position},
};
use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserState,
    QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, StyleSheetParser, Token,
    parse_important,
};

type Parsed<T> = Result<T, ParseError<String>>;
#[derive(Clone, Debug)]
pub(crate) enum Value {
    Absolute,
    Relative,
    Static,
    Auto,
    Percent(f32),
    Block,
    Flex,
    Row,
    Column,
    Stretch,
    Start,
    Center,
    End,
    Length(f32),
    Color(Color),
    ContentBox,
    BorderBox,
    None,
    Gradient(Gradient),
    Shadow(InsetShadow),
    Solid,
    Dotted,
    Character(char),
    Icon(Icon),
    ObjectFit(crate::image::ObjectFit),
    Overflow(crate::scrolling::Overflow),
}
#[derive(Clone, Debug)]
pub(crate) struct Declaration {
    assignments: Vec<(usize, Value)>,
    important: bool,
}
const PROPERTIES: [&str; 46] = [
    "position",
    "left",
    "top",
    "width",
    "height",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "background-color",
    "color",
    "box-sizing",
    "background-image",
    "border-width",
    "border-top-color",
    "border-right-color",
    "border-bottom-color",
    "border-left-color",
    "border-radius",
    "font-size",
    "box-shadow",
    "outline-width",
    "outline-color",
    "outline-style",
    "outline-offset",
    "border-style",
    "content",
    "-native-ui-icon",
    "-native-ui-icon-size",
    "-native-ui-icon-stroke",
    "margin-top",
    "margin-right",
    "margin-bottom",
    "margin-left",
    "display",
    "flex-direction",
    "flex-grow",
    "gap",
    "align-items",
    "min-width",
    "min-height",
    "max-width",
    "max-height",
    "object-fit",
    "overflow-x",
    "overflow-y",
];
#[derive(Clone, Copy, Debug)]
pub(crate) struct ControlStyle {
    pub rect: Rect,
    pub layout: crate::layout::Style,
    pub overflow: [crate::scrolling::Overflow; 2],
    pub padding: [f32; 4],
    pub border_width: f32,
    pub font_size: Option<f32>,
    pub appearance: Appearance,
}
impl Default for ControlStyle {
    fn default() -> Self {
        Self {
            rect: Rect::default(),
            layout: crate::layout::Style::default(),
            overflow: [crate::scrolling::Overflow::Visible; 2],
            padding: [0.0; 4],
            border_width: 1.0,
            font_size: None,
            appearance: Appearance::default(),
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Part {
    #[default]
    Control,
    PickerIcon,
    Selection,
    SliderTrack,
    SliderFill,
    SliderThumb,
    Scrollbar,
    ScrollTrack,
    ScrollThumb,
    ScrollCorner,
}
#[derive(Debug)]
pub(crate) struct CompiledPart {
    pub base: ControlStyle,
    pub states: [Appearance; STATE_COUNT],
}
pub(crate) fn compile_part(
    node: &Node,
    rules: &[Rule],
    part: Part,
    flow: bool,
) -> Result<Option<Box<CompiledPart>>, Error> {
    if !rules
        .iter()
        .any(|r| r.selectors.iter().any(|s| s.matches(node, u8::MAX, part)))
    {
        return Ok(None);
    }
    if (part == Part::PickerIcon && node.tag() != "select")
        || (part == Part::Selection
            && (node.tag() != "input"
                || node
                    .attr("type")
                    .is_some_and(|t| !t.eq_ignore_ascii_case("text"))))
    {
        return Err(Error::new(
            "::picker-icon requires a select; ::selection requires a text input",
        ));
    }
    if matches!(
        part,
        Part::SliderTrack | Part::SliderFill | Part::SliderThumb
    ) && (node.tag() != "input"
        || !node
            .attr("type")
            .is_some_and(|t| t.eq_ignore_ascii_case("range")))
    {
        return Ok(None);
    }
    if matches!(
        part,
        Part::Scrollbar | Part::ScrollTrack | Part::ScrollThumb | Part::ScrollCorner
    ) && (!flow
        || !matches!(
            node.tag(),
            "body" | "div" | "main" | "aside" | "section" | "header" | "footer"
        ))
    {
        return Ok(None);
    }
    Ok(Some(Box::new(CompiledPart {
        base: compute_for_part(node, rules, 0, part, flow)?,
        states: state_styles(node, rules, part, flow)?,
    })))
}
pub(crate) const HOVER: u8 = 1;
pub(crate) const ACTIVE: u8 = 2;
pub(crate) const FOCUS: u8 = 4;
pub(crate) const DISABLED: u8 = 8;
pub(crate) const CHECKED: u8 = 16;
pub(crate) const STATE_COUNT: usize = 32;

// Compile visual combinations before painting; explicit option updates also use this path.
pub(crate) fn state_styles(
    node: &Node,
    rules: &[Rule],
    part: Part,
    flow: bool,
) -> Result<[Appearance; STATE_COUNT], Error> {
    let mut styles = [Appearance::default(); STATE_COUNT];
    for (state, style) in styles.iter_mut().enumerate() {
        *style = compute_for_part(node, rules, state as u8, part, flow)?.appearance;
    }
    Ok(styles)
}
#[derive(Default, Debug, Clone)]
pub(crate) struct Selector {
    tag: Option<String>,
    ids: Vec<String>,
    classes: Vec<String>,
    attributes: Vec<AttributeSelector>,
    excluded_attributes: Vec<AttributeSelector>,
    states: u8,
    pseudo_count: usize,
    part: Part,
}
impl Selector {
    fn matches(&self, node: &Node, state: u8, part: Part) -> bool {
        self.part == part
            && state & self.states == self.states
            && self.tag.as_deref().is_none_or(|tag| tag == node.tag())
            && self
                .attributes
                .iter()
                .all(|attribute| attribute.matches(node))
            && self
                .excluded_attributes
                .iter()
                .all(|attribute| !attribute.matches(node))
            && self
                .ids
                .iter()
                .all(|id| node.attr("id").as_ref() == Some(id))
            && self.classes.iter().all(|class| {
                node.attr("class")
                    .unwrap_or_default()
                    .split_ascii_whitespace()
                    .any(|c| c == class)
            })
    }
    fn specificity(&self) -> (usize, usize, usize) {
        (
            self.ids.len(),
            self.classes.len()
                + self.attributes.len()
                + self.excluded_attributes.len()
                + self.pseudo_count,
            usize::from(self.tag.is_some()) + usize::from(self.part != Part::Control),
        )
    }
}
#[derive(Debug, Clone)]
struct AttributeSelector {
    name: String,
    value: Option<String>,
    case_insensitive: Option<bool>,
}
impl AttributeSelector {
    fn matches(&self, node: &Node) -> bool {
        let Some(actual) = node.attr(&self.name) else {
            return false;
        };
        self.value.as_ref().is_none_or(|expected| {
            // HTML input/button type values are ASCII case-insensitive.
            if self
                .case_insensitive
                .unwrap_or(self.name == "type" && matches!(node.tag(), "input" | "button"))
            {
                actual.eq_ignore_ascii_case(expected)
            } else {
                actual == *expected
            }
        })
    }
}
fn attribute_selector(input: &mut Parser<'_>) -> Parsed<AttributeSelector> {
    let name = input.expect_ident()?.to_ascii_lowercase();
    let mut attribute = AttributeSelector {
        name,
        value: None,
        case_insensitive: None,
    };
    if !input.is_exhausted() {
        input.expect_delim('=')?;
        attribute.value = Some(input.expect_ident_or_string()?.to_string());
        if !input.is_exhausted() {
            attribute.case_insensitive =
                Some(match input.expect_ident()?.to_ascii_lowercase().as_str() {
                    "i" => true,
                    "s" => false,
                    _ => {
                        return Err(ParseError::custom(
                            "attribute selector flags must be i or s",
                        ));
                    }
                });
        }
    }
    input.expect_exhausted()?;
    Ok(attribute)
}
#[derive(Debug, Clone)]
pub(crate) struct Rule {
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
}
struct CssParser;
impl<'i> AtRuleParser<'i> for CssParser {
    type Prelude = ();
    type AtRule = Rule;
    type Error = String;
}
impl<'i> QualifiedRuleParser<'i> for CssParser {
    type Prelude = Vec<Selector>;
    type QualifiedRule = Rule;
    type Error = String;
    fn parse_prelude(&mut self, input: &mut Parser<'i>) -> Parsed<Vec<Selector>> {
        input.parse_comma_separated(|input| {
            let mut selector = Selector::default();
            let mut any = false;
            // Whitespace within a selector is a combinator, not a compound selector.
            while let Ok(token) = input.next_including_whitespace().cloned() {
                if selector.part != Part::Control && !matches!(token, Token::Colon | Token::WhiteSpace(_)) {
                    return Err(ParseError::custom("only state pseudo-classes may follow a pseudo-element"));
                }
                match token {
                    Token::WhiteSpace(_) if !any => continue,
                    Token::WhiteSpace(_) => {
                        if input.is_exhausted() {
                            break;
                        }
                        return Err(ParseError::custom("descendant selectors are not supported"));
                    }
                    Token::Ident(tag) if !any => selector.tag = Some(tag.to_ascii_lowercase()),
                    Token::Delim('*') if !any => {}
                    Token::IDHash(id) => selector.ids.push(id.to_string()),
                    Token::Delim('.') => selector.classes.push(input.expect_ident()?.to_string()),
                    Token::SquareBracketBlock => {
                        selector
                            .attributes
                            .push(input.parse_nested_block(attribute_selector)?);
                    }
                    Token::Colon => {
                        if input.try_parse(|p| p.expect_colon()).is_ok() {
                            let name = input.expect_ident()?;
                            if selector.part != Part::Control {return Err(ParseError::custom("only one pseudo-element is supported"));}
                            selector.part = match name.to_ascii_lowercase().as_str() {
                                "picker-icon" => Part::PickerIcon,
                                "selection" => Part::Selection,
                                "slider-track" | "-webkit-slider-runnable-track" => Part::SliderTrack,
                                "slider-fill" => Part::SliderFill,
                                "slider-thumb" | "-webkit-slider-thumb" => Part::SliderThumb,
                                "scrollbar" | "-webkit-scrollbar" => Part::Scrollbar,
                                "scrollbar-track" | "-webkit-scrollbar-track" => Part::ScrollTrack,
                                "scrollbar-thumb" | "-webkit-scrollbar-thumb" => Part::ScrollThumb,
                                "scrollbar-corner" | "-webkit-scrollbar-corner" => Part::ScrollCorner,
                                _ => {
                                    return Err(ParseError::custom(format!(
                                        "unsupported pseudo-element ::{name}"
                                    )));
                                }
                            };
                            any = true;
                            continue;
                        }
                        if input
                            .try_parse(|p| p.expect_function_matching("not"))
                            .is_ok()
                        {
                            if selector.part != Part::Control {
                                return Err(ParseError::custom(":not must precede the pseudo-element"));
                            }
                            let attribute = input.parse_nested_block(|p| {
                                p.expect_square_bracket_block()?;
                                let attribute = p.parse_nested_block(attribute_selector)?;
                                p.expect_exhausted()?;
                                Ok(attribute)
                            })?;
                            selector.excluded_attributes.push(attribute);
                            any = true;
                            continue;
                        }
                        let name = input.expect_ident()?;
                        let state = match name.to_ascii_lowercase().as_str() {
                            "hover" => HOVER,
                            "active" => ACTIVE,
                            "focus" => FOCUS,
                            "disabled" => DISABLED,
                            "checked" => CHECKED,
                            _ => {
                                return Err(ParseError::custom(format!(
                                    "unsupported pseudo-class :{name}"
                                )));
                            }
                        };
                        selector.states |= state;
                        selector.pseudo_count += 1;
                    }
                    _ => {
                        return Err(ParseError::custom(
                            "supported selectors: tag, #id, .class, attributes, compounds and comma lists",
                        ));
                    }
                }
                any = true;
            }
            if !any {
                return Err(ParseError::custom("empty selector"));
            }
            Ok(selector)
        })
    }
    fn parse_block(
        &mut self,
        selectors: Vec<Selector>,
        _: &ParserState,
        input: &mut Parser<'i>,
    ) -> Parsed<Rule> {
        let declarations = declarations(input)?;
        if selectors.iter().any(|selector| selector.states != 0)
            && declarations.iter().any(|declaration| {
                declaration.assignments.iter().any(
                    |(property, _)| !matches!(property, 9 | 10 | 12 | 14..=18 | 20..=24 | 26..=29 | 43),
                )
            })
        {
            return Err(ParseError::custom(
                "state rules support visual decoration only; layout, border width and font size must stay fixed",
            ));
        }
        Ok(Rule {
            selectors,
            declarations,
        })
    }
}
struct Declarations;
impl<'i> DeclarationParser<'i> for Declarations {
    type Declaration = Declaration;
    type Error = String;
    fn parse_value(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i>,
        _: &ParserState,
    ) -> Parsed<Declaration> {
        let name = name.to_ascii_lowercase();
        let assignments = match name.as_str() {
            "overflow" => {
                let x = overflow(input)?;
                let y = if input.is_exhausted() {
                    x
                } else {
                    overflow(input)?
                };
                vec![(44, Value::Overflow(x)), (45, Value::Overflow(y))]
            }
            "margin" => four_values(input, |p| Ok(Value::Length(length(p, false)?)))?
                .into_iter()
                .enumerate()
                .map(|(i, v)| (i + 30, v))
                .collect(),
            "padding" => four_values(input, |p| Ok(Value::Length(length(p, true)?)))?
                .into_iter()
                .enumerate()
                .map(|(i, v)| (i + 5, v))
                .collect(),
            "border-color" => four_values(input, |p| Ok(Value::Color(color(p)?)))?
                .into_iter()
                .enumerate()
                .map(|(i, v)| (i + 14, v))
                .collect(),
            "background" => {
                if let Ok(image) = input.try_parse(gradient) {
                    vec![
                        (9, Value::Color(Color(0, 0, 0, 0))),
                        (12, Value::Gradient(image)),
                    ]
                } else if input.try_parse(|p| p.expect_ident_matching("none")).is_ok() {
                    vec![(9, Value::Color(Color(0, 0, 0, 0))), (12, Value::None)]
                } else {
                    vec![(9, Value::Color(color(input)?)), (12, Value::None)]
                }
            }
            "border" => {
                if input.try_parse(|p| p.expect_ident_matching("none")).is_ok() {
                    vec![
                        (13, Value::Length(0.0)),
                        (25, Value::None),
                        (14, Value::None),
                        (15, Value::None),
                        (16, Value::None),
                        (17, Value::None),
                    ]
                } else {
                    let width = decoration_length(input, true)?;
                    input.expect_ident_matching("solid")?;
                    let c = color(input)?;
                    vec![
                        (13, Value::Length(width)),
                        (25, Value::Solid),
                        (14, Value::Color(c)),
                        (15, Value::Color(c)),
                        (16, Value::Color(c)),
                        (17, Value::Color(c)),
                    ]
                }
            }
            "outline" => {
                if input.try_parse(|p| p.expect_ident_matching("none")).is_ok() {
                    vec![
                        (21, Value::Length(0.0)),
                        (22, Value::None),
                        (23, Value::None),
                    ]
                } else {
                    let width = decoration_length(input, true)?;
                    let style = line_style(input, true)?;
                    vec![
                        (21, Value::Length(width)),
                        (23, style),
                        (22, Value::Color(color(input)?)),
                    ]
                }
            }
            _ => {
                let property = PROPERTIES
                    .iter()
                    .position(|p| name == *p)
                    .ok_or_else(|| ParseError::custom(format!("unsupported property '{name}'")))?;
                let value = match property {
                    0 => match input.expect_ident()?.to_ascii_lowercase().as_str() {
                        "absolute" => Value::Absolute,
                        "relative" => Value::Relative,
                        "static" => Value::Static,
                        _ => {
                            return Err(ParseError::custom(
                                "position supports static, relative and absolute",
                            ));
                        }
                    },
                    3 | 4 => {
                        if input.try_parse(|p| p.expect_ident_matching("auto")).is_ok() {
                            Value::Auto
                        } else if let Ok(value) = input.try_parse(|p| p.expect_percentage()) {
                            if !value.is_finite() || value < 0.0 {
                                return Err(ParseError::custom(
                                    "size percentages must be finite and nonnegative",
                                ));
                            }
                            Value::Percent(value)
                        } else {
                            Value::Length(length(input, true)?)
                        }
                    }
                    44 | 45 => Value::Overflow(overflow(input)?),
                    43 => Value::ObjectFit(
                        match input.expect_ident()?.to_ascii_lowercase().as_str() {
                            "fill" => crate::image::ObjectFit::Fill,
                            "contain" => crate::image::ObjectFit::Contain,
                            "cover" => crate::image::ObjectFit::Cover,
                            _ => {
                                return Err(ParseError::custom(
                                    "object-fit supports fill, contain or cover",
                                ));
                            }
                        },
                    ),
                    30..=33 => Value::Length(length(input, false)?),
                    34 => match input.expect_ident()?.to_ascii_lowercase().as_str() {
                        "block" => Value::Block,
                        "flex" => Value::Flex,
                        _ => return Err(ParseError::custom("display supports block and flex")),
                    },
                    35 => match input.expect_ident()?.to_ascii_lowercase().as_str() {
                        "row" => Value::Row,
                        "column" => Value::Column,
                        _ => {
                            return Err(ParseError::custom(
                                "flex-direction supports row and column",
                            ));
                        }
                    },
                    36 => {
                        let grow = input.expect_number()?;
                        if !grow.is_finite() || !(0.0..=10000.0).contains(&grow) {
                            return Err(ParseError::custom(
                                "flex-grow must be between 0 and 10000",
                            ));
                        }
                        Value::Length(grow)
                    }
                    37 | 39..=42 => Value::Length(length(input, true)?),
                    38 => match input.expect_ident()?.to_ascii_lowercase().as_str() {
                        "stretch" => Value::Stretch,
                        "start" | "flex-start" => Value::Start,
                        "center" => Value::Center,
                        "end" | "flex-end" => Value::End,
                        _ => {
                            return Err(ParseError::custom(
                                "align-items supports stretch, start, center and end",
                            ));
                        }
                    },
                    9 | 10 | 14..=17 | 22 => Value::Color(color(input)?),
                    11 => {
                        let value = input.expect_ident()?;
                        if value.eq_ignore_ascii_case("border-box") {
                            Value::BorderBox
                        } else if value.eq_ignore_ascii_case("content-box") {
                            Value::ContentBox
                        } else {
                            return Err(ParseError::custom(
                                "box-sizing must be border-box or content-box",
                            ));
                        }
                    }
                    12 => {
                        if input.try_parse(|p| p.expect_ident_matching("none")).is_ok() {
                            Value::None
                        } else {
                            Value::Gradient(gradient(input)?)
                        }
                    }
                    20 => {
                        if input.try_parse(|p| p.expect_ident_matching("none")).is_ok() {
                            Value::None
                        } else {
                            input.expect_ident_matching("inset")?;
                            let x = decoration_length(input, false)?;
                            let y = decoration_length(input, false)?;
                            let blur = length(input, true)?;
                            if blur != 0.0 {
                                return Err(ParseError::custom(
                                    "box-shadow supports inset <x> <y> 0 <spread> <color> only",
                                ));
                            }
                            Value::Shadow(InsetShadow {
                                x,
                                y,
                                spread: decoration_length(input, true)?,
                                color: color(input)?,
                            })
                        }
                    }
                    23 => line_style(input, true)?,
                    25 => line_style(input, false)?,
                    27 => {
                        let name = input.expect_ident_cloned()?;
                        Value::Icon(match name.as_ref() {
                            "chevron-down" => Icon::ChevronDown,
                            "chevron-up-down" => Icon::ChevronUpDown,
                            _ => {
                                return Err(ParseError::custom(
                                    "unsupported -native-ui-icon shape",
                                ));
                            }
                        })
                    }
                    28 | 29 => {
                        let size = length(input, true)?;
                        if size <= 0.0 || size > 64.0 {
                            return Err(ParseError::custom(
                                "icon dimensions must be greater than 0 and at most 64px",
                            ));
                        }
                        Value::Length(size)
                    }
                    26 => {
                        let text = input.expect_string()?;
                        let mut chars = text.chars();
                        let ch = chars.next().filter(|c| !c.is_control()).ok_or_else(|| {
                            ParseError::custom("content requires one printable character")
                        })?;
                        if chars.next().is_some() {
                            return Err(ParseError::custom("content supports one character only"));
                        }
                        Value::Character(ch)
                    }
                    13 | 18 | 21 | 24 => Value::Length(decoration_length(input, property != 24)?),
                    19 => {
                        let size = length(input, true)?;
                        if size <= 0.0 || size > 256.0 {
                            return Err(ParseError::custom(
                                "font-size must be greater than 0 and at most 256px",
                            ));
                        }
                        Value::Length(size)
                    }
                    _ => Value::Length(length(input, property >= 3)?),
                };
                vec![(property, value)]
            }
        };
        let important = input.try_parse(parse_important).is_ok();
        input.expect_exhausted()?;
        Ok(Declaration {
            assignments,
            important,
        })
    }
}
impl<'i> AtRuleParser<'i> for Declarations {
    type Prelude = ();
    type AtRule = Declaration;
    type Error = String;
}
impl<'i> QualifiedRuleParser<'i> for Declarations {
    type Prelude = ();
    type QualifiedRule = Declaration;
    type Error = String;
}
impl<'i> RuleBodyItemParser<'i, Declaration, String> for Declarations {
    fn parse_declarations(&self) -> bool {
        true
    }
    fn parse_qualified(&self) -> bool {
        false
    }
}
fn declarations(input: &mut Parser<'_>) -> Parsed<Vec<Declaration>> {
    RuleBodyParser::new(input, &mut Declarations)
        .map(|r| r.map_err(|(e, _, _)| e))
        .collect()
}
pub(crate) fn parse(css: &str) -> Result<Vec<Rule>, Error> {
    StyleSheetParser::new(&mut Parser::new(css), &mut CssParser)
        .map(|r| {
            r.map_err(|(e, source, location)| {
                Error::new(format!(
                    "CSS line {}, column {} ({source:?}): {e}",
                    location.line + 1,
                    location.column
                ))
            })
        })
        .collect()
}
pub(crate) fn compute(node: &Node, rules: &[Rule], flow: bool) -> Result<ControlStyle, Error> {
    compute_for_part(node, rules, 0, Part::Control, flow)
}
fn compute_for_part(
    node: &Node,
    rules: &[Rule],
    state: u8,
    part: Part,
    flow: bool,
) -> Result<ControlStyle, Error> {
    if node.name.is_none() {
        return Ok(ControlStyle::default());
    }
    // Compare importance, inline origin, specificity, then source order.
    type Priority = (bool, bool, (usize, usize, usize), usize);
    let mut values: [Option<(Priority, Value)>; 46] = std::array::from_fn(|_| None);
    let mut order = 0;
    let mut apply = |decls: &[Declaration], inline, specificity| {
        for decl in decls {
            order += 1;
            let priority = (decl.important, inline, specificity, order);
            for (property, value) in &decl.assignments {
                let slot = &mut values[*property];
                if slot.as_ref().is_none_or(|(old, _)| priority >= *old) {
                    *slot = Some((priority, value.clone()));
                }
            }
        }
    };
    for rule in rules {
        if let Some(specificity) = rule
            .selectors
            .iter()
            .filter(|s| s.matches(node, state, part))
            .map(Selector::specificity)
            .max()
        {
            apply(&rule.declarations, false, specificity);
        }
    }
    if part == Part::Control
        && let Some(style) = node.attr("style")
    {
        let decls = declarations(&mut Parser::new(&style))
            .map_err(|e| Error::new(format!("inline CSS: {e:?}")))?;
        apply(&decls, true, (0, 0, 0));
    }
    if matches!(
        part,
        Part::SliderFill | Part::ScrollTrack | Part::ScrollCorner
    ) && values[3..5].iter().any(Option::is_some)
    {
        return Err(Error::new(
            "this part follows its track or scrollbar dimensions; size ::slider-track or ::scrollbar instead",
        ));
    }
    if matches!(
        part,
        Part::SliderTrack | Part::SliderThumb | Part::Scrollbar | Part::ScrollThumb
    ) && values[3..5].iter().any(|v| {
        v.as_ref()
            .is_some_and(|(_, v)| !matches!(v, Value::Length(_)))
    }) {
        return Err(Error::new(
            "slider and scrollbar part dimensions require px or zero",
        ));
    }
    let auxiliary = part != Part::Control || node.tag() == "option";
    if values[44..46].iter().any(Option::is_some)
        && (!flow
            || part != Part::Control
            || !matches!(
                node.tag(),
                "body" | "div" | "main" | "aside" | "section" | "header" | "footer"
            ))
    {
        return Err(Error::new("overflow requires a layout container"));
    }
    if part == Part::Selection
        && values.iter().enumerate().any(|(i, v)| {
            v.is_some() && !matches!(i, 9 | 10) && !(i == 12 && matches!(v, Some((_, Value::None))))
        })
    {
        return Err(Error::new(
            "::selection supports solid background and color only",
        ));
    }
    if part != Part::PickerIcon && values[26].is_some() {
        return Err(Error::new("content is supported on ::picker-icon only"));
    }
    if part != Part::PickerIcon && values[27..30].iter().any(Option::is_some) {
        return Err(Error::new(
            "icon properties are supported on ::picker-icon only",
        ));
    }
    if values[26].is_some() && values[27].is_some() {
        return Err(Error::new(
            "picker content and vector icon cannot be combined",
        ));
    }
    if values[43].is_some() && (node.tag() != "img" || part != Part::Control) {
        return Err(Error::new("object-fit is supported on img only"));
    }
    if auxiliary && values[30..43].iter().any(Option::is_some) {
        return Err(Error::new(
            "layout properties are not supported on options or pseudo-elements",
        ));
    }
    if !flow
        && (values[30..43].iter().any(Option::is_some)
            || values.iter().any(|v| {
                matches!(
                    v,
                    Some((
                        _,
                        Value::Relative | Value::Static | Value::Auto | Value::Percent(_)
                    ))
                )
            }))
    {
        return Err(Error::new(
            "relative layout requires from_html_css_with_viewport",
        ));
    }
    if auxiliary
        && values.iter().enumerate().any(|(i, v)| {
            v.is_some()
                && matches!(i, 0..=8 | 11)
                && !(part != Part::Control && part != Part::Selection && matches!(i, 3 | 4))
        })
    {
        return Err(Error::new(
            "options and pseudo-elements support styling, not layout or padding",
        ));
    }
    if !flow && node.tag() == "body" && part == Part::Control {
        if values.iter().enumerate().any(|(i, v)| {
            v.is_some() && i != 9 && !(i == 12 && matches!(v, Some((_, Value::None))))
        }) {
            return Err(Error::new(
                "body currently supports a solid background only",
            ));
        }
        let background = match values[9] {
            Some((_, Value::Color(c))) => Some(c),
            _ => None,
        };
        return Ok(ControlStyle {
            appearance: Appearance {
                background,
                ..Appearance::default()
            },
            ..ControlStyle::default()
        });
    }
    let container = flow
        && matches!(
            node.tag(),
            "body" | "div" | "main" | "aside" | "section" | "header" | "footer"
        );
    if !auxiliary
        && !container
        && !matches!(
            node.tag(),
            "button" | "input" | "select" | "label" | "canvas" | "img"
        )
    {
        if values.iter().any(Option::is_some) {
            return Err(Error::new(format!(
                "styling on <{}> is not supported yet",
                node.tag()
            )));
        }
        return Ok(ControlStyle::default());
    }
    if !flow && !auxiliary && values[..5].iter().any(Option::is_none) {
        let missing: Vec<_> = PROPERTIES[..5]
            .iter()
            .enumerate()
            .filter(|(i, _)| values[*i].is_none())
            .map(|(_, p)| *p)
            .collect();
        return Err(Error::new(format!(
            "<{}> {:?} is missing CSS: {}",
            node.tag(),
            node.attr("id"),
            missing.join(", ")
        )));
    }
    let length = |i| match &values[i] {
        Some((_, Value::Length(v))) => *v,
        _ => 0.0,
    };
    let padding = [length(5), length(6), length(7), length(8)];
    let mut rect = Rect {
        x: length(1),
        y: length(2),
        w: length(3),
        h: length(4),
    };
    let border_width = if matches!(values[25], Some((_, Value::None))) {
        0.0
    } else if values[13].is_some() {
        length(13)
    } else if part != Part::Control
        || container
        || (node.tag() == "input"
            && node
                .attr("type")
                .is_some_and(|t| t.eq_ignore_ascii_case("range")))
        || matches!(node.tag(), "label" | "canvas" | "img")
    {
        0.0
    } else {
        1.0
    };
    let borders = border_width * 2.0;
    if matches!(values[11], Some((_, Value::ContentBox))) {
        rect.w += padding[1] + padding[3] + borders;
        rect.h += padding[0] + padding[2] + borders;
    } else {
        // Border-box cannot be smaller than its padding and border; content clamps to zero.
        rect.w = rect.w.max(padding[1] + padding[3] + borders);
        rect.h = rect.h.max(padding[0] + padding[2] + borders);
    }
    rect.validate()?;
    // Unspecified picker dimensions fill the reserved area; border metrics must
    // not turn that into an implicitly specified two-pixel box.
    if part == Part::PickerIcon {
        for (index, dimension) in [(3, &mut rect.w), (4, &mut rect.h)] {
            if values[index].is_none() {
                *dimension = 0.0;
            } else if length(index) <= 0.0 {
                return Err(Error::new(
                    "picker width and height must be greater than zero",
                ));
            }
        }
    }
    let color = |i| match values[i] {
        Some((_, Value::Color(color))) => Some(color),
        _ => None,
    };
    let overflow = crate::scrolling::normalize([44, 45].map(|i| match values[i] {
        Some((_, Value::Overflow(v))) => v,
        _ => crate::scrolling::Overflow::Visible,
    }));
    let appearance = Appearance {
        object_fit: match values[43] {
            Some((_, Value::ObjectFit(fit))) => fit,
            _ => crate::image::ObjectFit::default(),
        },
        icon: match values[27] {
            Some((_, Value::Icon(icon))) => Some(icon),
            _ => None,
        },
        icon_size: if values[28].is_some() {
            length(28)
        } else {
            10.0
        },
        icon_stroke: if values[29].is_some() {
            length(29)
        } else {
            1.5
        },
        content: match values[26] {
            Some((_, Value::Character(c))) => Some(c),
            _ => None,
        },
        background: color(9),
        text: color(10),
        image: match values[12] {
            Some((_, Value::Gradient(g))) => Some(g),
            _ => None,
        },
        borders: [color(14), color(15), color(16), color(17)],
        radius: length(18),
        shadow: match values[20] {
            Some((_, Value::Shadow(s))) => Some(s),
            _ => None,
        },
        outline: Outline {
            width: if matches!(values[23], Some((_, Value::Solid | Value::Dotted))) {
                if values[21].is_some() {
                    length(21)
                } else {
                    3.0
                }
            } else {
                0.0
            },
            color: color(22),
            dotted: matches!(values[23], Some((_, Value::Dotted))),
            offset: length(24),
        },
    };
    let dimension = |i| match values[i] {
        Some((_, Value::Length(v))) => Dimension::Px(v),
        Some((_, Value::Percent(v))) => Dimension::Percent(v),
        _ => Dimension::Auto,
    };
    let min = [length(39), length(40)];
    let max = [
        if values[41].is_some() {
            length(41)
        } else {
            f32::INFINITY
        },
        if values[42].is_some() {
            length(42)
        } else {
            f32::INFINITY
        },
    ];
    if min[0] > max[0] || min[1] > max[1] {
        return Err(Error::new("minimum size cannot exceed maximum size"));
    }
    if flow
        && matches!(values[0], Some((_, Value::Static)))
        && (values[1].is_some() || values[2].is_some())
    {
        return Err(Error::new(
            "left/top offsets require position:relative or absolute",
        ));
    }
    Ok(ControlStyle {
        overflow,
        rect,
        layout: crate::layout::Style {
            position: match values[0] {
                Some((_, Value::Absolute)) => Position::Absolute,
                Some((_, Value::Relative)) => Position::Relative,
                _ => Position::Static,
            },
            width: dimension(3),
            height: dimension(4),
            margin: [length(30), length(31), length(32), length(33)],
            display: if matches!(values[34], Some((_, Value::Flex))) {
                Display::Flex
            } else {
                Display::Block
            },
            direction: if matches!(values[35], Some((_, Value::Column))) {
                Direction::Column
            } else {
                Direction::Row
            },
            grow: length(36),
            gap: length(37),
            min,
            max,
            align: match values[38] {
                Some((_, Value::Start)) => Align::Start,
                Some((_, Value::Center)) => Align::Center,
                Some((_, Value::End)) => Align::End,
                _ => Align::Stretch,
            },
            content_box: matches!(values[11], Some((_, Value::ContentBox))),
        },
        padding,
        border_width,
        font_size: values[19].as_ref().map(|_| length(19)),
        appearance,
    })
}

fn four_values(
    input: &mut Parser<'_>,
    mut parse: impl FnMut(&mut Parser<'_>) -> Parsed<Value>,
) -> Parsed<[Value; 4]> {
    let mut sides = vec![parse(input)?];
    while sides.len() < 4 {
        match input.try_parse(&mut parse) {
            Ok(value) => sides.push(value),
            Err(_) => break,
        }
    }
    Ok([
        sides[0].clone(),
        sides.get(1).unwrap_or(&sides[0]).clone(),
        sides.get(2).unwrap_or(&sides[0]).clone(),
        sides.get(3).or(sides.get(1)).unwrap_or(&sides[0]).clone(),
    ])
}
fn decoration_length(input: &mut Parser<'_>, positive: bool) -> Parsed<f32> {
    let value = length(input, positive)?;
    if value.abs() > 4096.0 {
        return Err(ParseError::custom(
            "decoration lengths must not exceed 4096px",
        ));
    }
    Ok(value)
}
fn line_style(input: &mut Parser<'_>, dotted: bool) -> Parsed<Value> {
    match input.expect_ident()?.to_ascii_lowercase().as_str() {
        "solid" => Ok(Value::Solid),
        "none" => Ok(Value::None),
        "dotted" if dotted => Ok(Value::Dotted),
        _ => Err(ParseError::custom(
            "supported styles are solid/none, and dotted for outlines",
        )),
    }
}
fn gradient(input: &mut Parser<'_>) -> Parsed<Gradient> {
    input.expect_function_matching("linear-gradient")?;
    input.parse_nested_block(|input| {
        let mut horizontal = false;
        let mut reverse = false;
        if let Ok((h, r)) = input.try_parse(|input| -> Parsed<(bool, bool)> {
            let result = match input.next()?.clone() {
                Token::Ident(to) if to.eq_ignore_ascii_case("to") => {
                    match input.expect_ident()?.to_ascii_lowercase().as_str() {
                        "bottom" => (false, false),
                        "top" => (false, true),
                        "right" => (true, false),
                        "left" => (true, true),
                        _ => {
                            return Err(ParseError::custom(
                                "gradient direction must be top, bottom, left or right",
                            ));
                        }
                    }
                }
                Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("deg") => {
                    match value {
                        0.0 => (false, true),
                        90.0 => (true, false),
                        180.0 => (false, false),
                        270.0 => (true, true),
                        _ => {
                            return Err(ParseError::custom(
                                "only axis-aligned gradients are supported",
                            ));
                        }
                    }
                }
                _ => return Err(ParseError::custom("expected gradient direction")),
            };
            input.expect_comma()?;
            Ok(result)
        }) {
            horizontal = h;
            reverse = r;
        }
        let mut stops = input.parse_comma_separated(|input| {
            let c = color(input)?;
            let position = input
                .try_parse(|input| -> Parsed<f32> {
                    match input.next()?.clone() {
                        Token::Percentage { unit_value, .. }
                            if unit_value.is_finite() && (0.0..=1.0).contains(&unit_value) =>
                        {
                            Ok(unit_value)
                        }
                        _ => Err(ParseError::custom(
                            "gradient stops require percentages between 0% and 100%",
                        )),
                    }
                })
                .ok();
            input.expect_exhausted()?;
            Ok((c, position))
        })?;
        if !(2..=8).contains(&stops.len()) {
            return Err(ParseError::custom("gradients require two to eight stops"));
        }
        let last = stops.len() - 1;
        stops[0].1.get_or_insert(0.0);
        stops[last].1.get_or_insert(1.0);
        let mut previous = 0.0_f32;
        for (_, position) in &mut stops {
            if let Some(p) = position {
                *p = p.max(previous);
                previous = *p;
            }
        }
        let mut left = 0;
        while left < last {
            let right = (left + 1..=last).find(|&i| stops[i].1.is_some()).unwrap();
            let from = stops[left].1.unwrap();
            let to = stops[right].1.unwrap();
            for (i, (_, position)) in stops.iter_mut().enumerate().take(right).skip(left + 1) {
                *position = Some(from + (to - from) * (i - left) as f32 / (right - left) as f32);
            }
            left = right;
        }
        let mut result = Gradient {
            stops: [Stop::default(); 8],
            count: stops.len(),
            horizontal,
            reverse,
        };
        for (slot, (color, position)) in result.stops.iter_mut().zip(stops) {
            *slot = Stop {
                color,
                position: position.unwrap(),
            };
        }
        Ok(result)
    })
}

fn length(input: &mut Parser<'_>, nonnegative: bool) -> Parsed<f32> {
    let value = match input.next()?.clone() {
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("px") => value,
        Token::Number { value: 0.0, .. } => 0.0,
        _ => return Err(ParseError::custom("length requires px (or unitless zero)")),
    };
    if !value.is_finite() || (nonnegative && value < 0.0) {
        return Err(ParseError::custom(
            "length must be finite; size and padding cannot be negative",
        ));
    }
    Ok(value)
}
fn color(input: &mut Parser<'_>) -> Parsed<Color> {
    match input.next()?.clone() {
        Token::Hash(hex) | Token::IDHash(hex) => {
            if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(ParseError::custom("invalid hex color"));
            }
            let byte = |s: &str| u8::from_str_radix(s, 16).unwrap();
            Ok(match hex.len() {
                3 | 4 => Color(
                    byte(&hex[0..1]) * 17,
                    byte(&hex[1..2]) * 17,
                    byte(&hex[2..3]) * 17,
                    if hex.len() == 4 {
                        byte(&hex[3..4]) * 17
                    } else {
                        255
                    },
                ),
                6 | 8 => Color(
                    byte(&hex[0..2]),
                    byte(&hex[2..4]),
                    byte(&hex[4..6]),
                    if hex.len() == 8 {
                        byte(&hex[6..8])
                    } else {
                        255
                    },
                ),
                _ => return Err(ParseError::custom("hex colors require 3, 4, 6 or 8 digits")),
            })
        }
        Token::Ident(name) => {
            let (r, g, b, a) = match name.to_ascii_lowercase().as_str() {
                "transparent" => (0, 0, 0, 0),
                "black" => (0, 0, 0, 255),
                "white" => (255, 255, 255, 255),
                "red" => (255, 0, 0, 255),
                "green" => (0, 128, 0, 255),
                "blue" => (0, 0, 255, 255),
                "yellow" => (255, 255, 0, 255),
                "gray" | "grey" => (128, 128, 128, 255),
                "silver" => (192, 192, 192, 255),
                "maroon" => (128, 0, 0, 255),
                "purple" => (128, 0, 128, 255),
                "fuchsia" | "magenta" => (255, 0, 255, 255),
                "lime" => (0, 255, 0, 255),
                "olive" => (128, 128, 0, 255),
                "navy" => (0, 0, 128, 255),
                "teal" => (0, 128, 128, 255),
                "aqua" | "cyan" => (0, 255, 255, 255),
                "orange" => (255, 165, 0, 255),
                "rebeccapurple" => (102, 51, 153, 255),
                _ => {
                    return Err(ParseError::custom(format!(
                        "unsupported named color '{name}'; use hex or rgb()"
                    )));
                }
            };
            Ok(Color(r, g, b, a))
        }
        Token::Function(name)
            if name.eq_ignore_ascii_case("rgb") || name.eq_ignore_ascii_case("rgba") =>
        {
            input.parse_nested_block(|input| {
                let component = |input: &mut Parser<'_>, alpha| -> Parsed<u8> {
                    let n = match input.next()?.clone() {
                        Token::Number { value, .. } => {
                            if alpha {
                                value * 255.0
                            } else {
                                value
                            }
                        }
                        Token::Percentage { unit_value, .. } => unit_value * 255.0,
                        _ => return Err(ParseError::custom("expected color component")),
                    };
                    if !n.is_finite() {
                        return Err(ParseError::custom("nonfinite color component"));
                    }
                    Ok(n.clamp(0.0, 255.0).round() as u8)
                };
                let r = component(input, false)?;
                let commas = input.try_parse(|p| p.expect_comma()).is_ok();
                let g = component(input, false)?;
                if commas {
                    input.expect_comma()?;
                }
                let b = component(input, false)?;
                let alpha = if commas {
                    input.try_parse(|p| p.expect_comma()).is_ok()
                } else {
                    input.try_parse(|p| p.expect_delim('/')).is_ok()
                };
                let a = if alpha { component(input, true)? } else { 255 };
                input.expect_exhausted()?;
                Ok(Color(r, g, b, a))
            })
        }
        _ => Err(ParseError::custom(
            "expected a solid hex, named or rgb()/rgba() color",
        )),
    }
}

fn overflow(input: &mut Parser<'_>) -> Parsed<crate::scrolling::Overflow> {
    use crate::scrolling::Overflow::*;
    match input.expect_ident()?.to_ascii_lowercase().as_str() {
        "visible" => Ok(Visible),
        "hidden" => Ok(Hidden),
        "clip" => Ok(Clip),
        "auto" => Ok(Auto),
        "scroll" => Ok(Scroll),
        _ => Err(ParseError::custom(
            "overflow supports visible, hidden, clip, auto and scroll",
        )),
    }
}

// Retain only selectors that may style future options. Normal rendering and reflow
// still use precomputed appearances; these rules run only on explicit list updates.
pub(crate) fn option_rules(rules: &[Rule]) -> Vec<Rule> {
    rules
        .iter()
        .filter_map(|r| {
            let selectors: Vec<_> = r
                .selectors
                .iter()
                .filter(|s| {
                    s.part == Part::Control && s.tag.as_deref().is_none_or(|t| t == "option")
                })
                .cloned()
                .collect();
            (!selectors.is_empty()).then(|| Rule {
                selectors,
                declarations: r.declarations.clone(),
            })
        })
        .collect()
}
