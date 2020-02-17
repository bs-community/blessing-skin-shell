pub use super::pos::Position;

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub struct Identifier {
    pub name: String,
    pub span: Span,
}

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub enum Template {
    Unquoted(TemplateBody),
    Single(RawText),
    Double(TemplateBody),
}

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub struct RawText {
    pub text: String,
    pub span: Span,
}

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub struct TemplateBody {
    pub parts: Vec<TemplatePart>,
    pub span: Span,
}

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub enum TemplatePart {
    Raw(TemplateLiteral),
    Variable(Variable),
}

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub struct TemplateLiteral {
    pub value: String,
    pub span: Span,
}

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub struct Variable {
    pub id: Identifier,
    pub span: Span,
}

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub struct ParamLiteral {
    pub literal: Template,
    pub span: Span,
}

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub struct Command {
    pub program: Program,
    pub parameters: Option<Parameters>,
    pub span: Span,
}

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub struct Program {
    pub id: Identifier,
    pub span: Span,
}

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub enum Param {
    Literal(ParamLiteral),
    ShortSwitch(Switch),
    LongSwitch(Switch),
}

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub struct Parameter {
    pub param: Param,
    pub span: Span,
}

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub struct Parameters {
    pub params: Vec<Parameter>,
    pub span: Span,
}

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub struct Switch {
    pub name: Identifier,
    pub value: Option<Template>,
    pub span: Span,
}

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub struct Comment {
    pub content: String,
    pub span: Span,
}

impl Default for Span {
    fn default() -> Span {
        Span {
            start: Position::default(),
            end: Position::default(),
        }
    }
}
