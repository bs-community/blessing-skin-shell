pub(crate) mod ast;
mod pos;

#[cfg(test)]
mod tests;

use ast::*;
use combine::{
    error::StringStreamError,
    parser::{
        char::{alpha_num, space, spaces, string},
        choice::{choice, optional},
        combinator::attempt,
        repeat::{many, many1, sep_end_by1, skip_many1},
        sequence::between,
        token::{one_of, position, satisfy, token},
    },
    stream::position::Stream,
    Parser,
};
use pos::Position;

type Input<'a> = Stream<&'a str, Position>;

fn identifier<'a>() -> impl Parser<Input<'a>, Output = Identifier> {
    (
        position(),
        many1(choice((alpha_num(), one_of("_?!".chars())))),
        position(),
    )
        .map(|(start, name, end)| {
            let span = Span { start, end };
            Identifier { name, span }
        })
}

fn loose_identifer<'a>() -> impl Parser<Input<'a>, Output = Identifier> {
    (
        position(),
        many1(choice((alpha_num(), one_of("_?!-.".chars())))),
        position(),
    )
        .map(|(start, name, end)| {
            let span = Span { start, end };
            Identifier { name, span }
        })
}

fn variable<'a>() -> impl Parser<Input<'a>, Output = Variable> {
    (position(), token('$'), identifier(), position()).map(|(start, _, id, end)| {
        let span = Span { start, end };
        Variable { id, span }
    })
}

fn template_literal<'a>(quoted: bool) -> impl Parser<Input<'a>, Output = TemplateLiteral> {
    let value_parser = many1(satisfy(move |c: char| {
        let space = quoted || !c.is_whitespace();
        let quote = quoted || "'#".chars().all(|x| x != c);

        let forbidden = "$\n\"".chars().all(|x| x != c);

        space && quote && forbidden
    }));

    (position(), value_parser, position()).map(|(start, value, end)| {
        let span = Span { start, end };
        TemplateLiteral { value, span }
    })
}

fn single_dollar<'a>() -> impl Parser<Input<'a>, Output = TemplateLiteral> {
    (position(), token('$'), position()).map(|(start, value, end): (_, char, _)| {
        let span = Span { start, end };
        let value = value.to_string();
        TemplateLiteral { value, span }
    })
}

fn template_part<'a>(quoted: bool) -> impl Parser<Input<'a>, Output = TemplatePart> {
    let template_literal = template_literal(quoted).map(TemplatePart::Raw);
    let variable = variable().map(TemplatePart::Variable);
    let dollar = single_dollar().map(TemplatePart::Raw);
    choice((
        attempt(template_literal),
        attempt(variable),
        attempt(dollar),
    ))
}

fn template_body<'a>(quoted: bool) -> impl Parser<Input<'a>, Output = TemplateBody> {
    let parser = if quoted {
        // If we're in a quoted string, it can be an empty string.
        many(template_part(quoted)).left()
    } else {
        many1(template_part(quoted)).right()
    };
    (position(), parser, position()).map(|(start, parts, end)| {
        let span = Span { start, end };
        TemplateBody { parts, span }
    })
}

fn unquoted<'a>() -> impl Parser<Input<'a>, Output = Template> {
    template_body(false).map(Template::Unquoted)
}

fn double_quoted<'a>() -> impl Parser<Input<'a>, Output = Template> {
    between(token('"'), token('"'), template_body(true)).map(Template::Double)
}

fn raw_text<'a>() -> impl Parser<Input<'a>, Output = RawText> {
    (position(), many(satisfy(|c| c != '\'')), position()).map(|(start, text, end)| {
        let span = Span { start, end };
        RawText { text, span }
    })
}

fn single_quoted<'a>() -> impl Parser<Input<'a>, Output = Template> {
    between(token('\''), token('\''), raw_text()).map(Template::Single)
}

fn template<'a>() -> impl Parser<Input<'a>, Output = Template> {
    choice((single_quoted(), double_quoted(), unquoted()))
}

fn switch<'a>() -> impl Parser<Input<'a>, Output = Switch> {
    (
        position(),
        loose_identifer(),
        optional((token('='), template())),
        position(),
    )
        .map(|(start, name, value, end)| {
            let span = Span { start, end };
            let value = value.map(|x| x.1);
            Switch { name, value, span }
        })
}

fn param_literal<'a>() -> impl Parser<Input<'a>, Output = ParamLiteral> {
    (position(), template(), position()).map(|(start, literal, end)| {
        let span = Span { start, end };
        ParamLiteral { literal, span }
    })
}

fn literal<'a>() -> impl Parser<Input<'a>, Output = Param> {
    param_literal().map(Param::Literal)
}

fn long_switch<'a>() -> impl Parser<Input<'a>, Output = Param> {
    (string("--"), switch()).map(|(_, switch)| Param::LongSwitch(switch))
}

fn short_switch<'a>() -> impl Parser<Input<'a>, Output = Param> {
    (token('-'), switch()).map(|(_, switch)| Param::ShortSwitch(switch))
}

fn param<'a>() -> impl Parser<Input<'a>, Output = Param> {
    choice((
        attempt(short_switch()),
        attempt(long_switch()),
        attempt(literal()),
    ))
}

fn parameter<'a>() -> impl Parser<Input<'a>, Output = Parameter> {
    (position(), param(), position()).map(|(start, param, end)| {
        let span = Span { start, end };
        Parameter { param, span }
    })
}

fn parameters<'a>() -> impl Parser<Input<'a>, Output = Parameters> {
    (
        position(),
        sep_end_by1(parameter(), skip_many1(space())),
        position(),
    )
        .map(|(start, params, end)| {
            let span = Span { start, end };
            Parameters { params, span }
        })
}

fn program<'a>() -> impl Parser<Input<'a>, Output = Program> {
    (position(), loose_identifer(), position()).map(|(start, id, end)| {
        let span = Span { start, end };
        Program { id, span }
    })
}

fn command<'a>() -> impl Parser<Input<'a>, Output = Command> {
    (
        position(),
        program(),
        spaces(),
        optional(parameters()),
        position(),
    )
        .map(|(start, program, _, parameters, end)| {
            let span = Span { start, end };
            Command {
                program,
                parameters,
                span,
            }
        })
}

fn comment<'a>() -> impl Parser<Input<'a>, Output = Comment> {
    (
        token('#'),
        position(),
        many(satisfy(|c| c != '\n')),
        position(),
    )
        .map(|(_, start, content, end)| {
            let span = Span { start, end };
            Comment { content, span }
        })
}

pub fn parse(input: &str) -> Result<(Command, &str), StringStreamError> {
    spaces()
        .with(command())
        .skip(optional(comment()))
        .parse(Stream::with_positioner(input, Position::new()))
        .map(|x| (x.0, x.1.input))
}
