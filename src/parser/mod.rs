pub(crate) mod ast;
mod pos;

#[cfg(test)]
mod tests;

use ast::*;
use combine::{
    parser::{
        char::{alpha_num, space, spaces, string},
        choice::{choice, optional},
        combinator::attempt,
        repeat::{many, many1, sep_end_by1, skip_many1},
        sequence::between,
        token::{one_of, position, satisfy, token},
        EasyParser,
    },
    stream::{self, easy, Positioned, Stream},
    ParseError, Parser,
};
use pos::Position;

fn identifier<Input>() -> impl Parser<Input, Output = Identifier>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
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

fn loose_identifer<Input>() -> impl Parser<Input, Output = Identifier>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
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

fn variable<Input>() -> impl Parser<Input, Output = Variable>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
    (position(), token('$'), identifier(), position()).map(|(start, _, id, end)| {
        let span = Span { start, end };
        Variable { id, span }
    })
}

fn template_literal<Input>(quoted: bool) -> impl Parser<Input, Output = TemplateLiteral>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
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

fn single_dollar<Input>() -> impl Parser<Input, Output = TemplateLiteral>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
    (position(), token('$'), position()).map(|(start, value, end): (_, char, _)| {
        let span = Span { start, end };
        let value = value.to_string();
        TemplateLiteral { value, span }
    })
}

fn template_part<Input>(quoted: bool) -> impl Parser<Input, Output = TemplatePart>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
    let template_literal = template_literal(quoted).map(TemplatePart::Raw);
    let variable = variable().map(TemplatePart::Variable);
    let dollar = single_dollar().map(TemplatePart::Raw);
    choice((
        attempt(template_literal),
        attempt(variable),
        attempt(dollar),
    ))
}

fn template_body<Input>(quoted: bool) -> impl Parser<Input, Output = TemplateBody>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
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

fn unquoted<Input>() -> impl Parser<Input, Output = Template>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
    template_body(false).map(Template::Unquoted)
}

fn double_quoted<Input>() -> impl Parser<Input, Output = Template>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
    between(token('"'), token('"'), template_body(true)).map(Template::Double)
}

fn raw_text<Input>() -> impl Parser<Input, Output = RawText>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
    (position(), many(satisfy(|c| c != '\'')), position()).map(|(start, text, end)| {
        let span = Span { start, end };
        RawText { text, span }
    })
}

fn single_quoted<Input>() -> impl Parser<Input, Output = Template>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
    between(token('\''), token('\''), raw_text()).map(Template::Single)
}

fn template<Input>() -> impl Parser<Input, Output = Template>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
    choice((single_quoted(), double_quoted(), unquoted()))
}

fn switch<Input>() -> impl Parser<Input, Output = Switch>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
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

fn param_literal<Input>() -> impl Parser<Input, Output = ParamLiteral>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
    (position(), template(), position()).map(|(start, literal, end)| {
        let span = Span { start, end };
        ParamLiteral { literal, span }
    })
}

fn literal<Input>() -> impl Parser<Input, Output = Param>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
    param_literal().map(Param::Literal)
}

fn long_switch<Input>() -> impl Parser<Input, Output = Param>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
    (string("--"), switch()).map(|(_, switch)| Param::LongSwitch(switch))
}

fn short_switch<Input>() -> impl Parser<Input, Output = Param>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
    (token('-'), switch()).map(|(_, switch)| Param::ShortSwitch(switch))
}

fn param<Input>() -> impl Parser<Input, Output = Param>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
    choice((
        attempt(short_switch()),
        attempt(long_switch()),
        attempt(literal()),
    ))
}

fn parameter<Input>() -> impl Parser<Input, Output = Parameter>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
    (position(), param(), position()).map(|(start, param, end)| {
        let span = Span { start, end };
        Parameter { param, span }
    })
}

fn parameters<Input>() -> impl Parser<Input, Output = Parameters>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
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

fn program<Input>() -> impl Parser<Input, Output = Program>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
    (position(), loose_identifer(), position()).map(|(start, id, end)| {
        let span = Span { start, end };
        Program { id, span }
    })
}

fn command<Input>() -> impl Parser<Input, Output = Command>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
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

fn comment<Input>() -> impl Parser<Input, Output = Comment>
where
    Input: Stream<Token = char, Position = Position>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    Input: Positioned,
{
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

pub fn parse_interactive(
    input: &str,
) -> Result<(Command, &str), easy::Errors<char, &str, Position>> {
    spaces()
        .with(command())
        .skip(comment())
        .easy_parse(stream::position::Stream::with_positioner(
            input,
            Position::new(),
        ))
        .map(|x| (x.0, x.1.input))
}
