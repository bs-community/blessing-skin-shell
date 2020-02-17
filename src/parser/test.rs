use super::ast::*;
use super::pos::Position;
use super::*;
use combine::stream::position::Stream;

fn source(input: &str) -> Stream<&str, Position> {
    Stream::with_positioner(input, Position::new())
}

#[test]
fn parse_identifier() {
    let result = identifier()
        .parse(source("kumiko_-"))
        .map(|x| (x.0, x.1.input));
    assert_eq!(
        result,
        Ok((
            Identifier {
                name: "kumiko_".to_string(),
                span: Span {
                    start: Position {
                        line: 1,
                        column: 1,
                        index: 0
                    },
                    end: Position {
                        line: 1,
                        column: 8,
                        index: 7
                    }
                }
            },
            "-"
        ))
    )
}

#[test]
fn parse_loose_identifier() {
    let result = loose_identifer()
        .parse(source("kumiko_-?.!"))
        .map(|x| (x.0, x.1.input));
    assert_eq!(
        result,
        Ok((
            Identifier {
                name: "kumiko_-?.!".to_string(),
                span: Span {
                    start: Position {
                        line: 1,
                        column: 1,
                        index: 0
                    },
                    end: Position {
                        line: 1,
                        column: 12,
                        index: 11
                    }
                }
            },
            ""
        ))
    )
}

#[test]
fn parse_variable() {
    let result = variable().parse(source("$kumiko")).map(|x| x.0);
    assert_eq!(
        result,
        Ok(Variable {
            id: Identifier {
                name: "kumiko".to_string(),
                span: Span {
                    start: Position {
                        line: 1,
                        column: 2,
                        index: 1
                    },
                    end: Position {
                        line: 1,
                        column: 8,
                        index: 7
                    }
                }
            },
            span: Span {
                start: Position {
                    line: 1,
                    column: 1,
                    index: 0
                },
                end: Position {
                    line: 1,
                    column: 8,
                    index: 7
                }
            }
        })
    );

    assert!(variable().parse(source("$-")).is_err());
}

#[test]
fn parse_comment() {
    let result = comment()
        .parse(source("# abc\ndef"))
        .map(|x| (x.0, x.1.input));
    assert_eq!(
        result,
        Ok((
            Comment {
                content: " abc".to_string(),
                span: Span {
                    start: Position {
                        line: 1,
                        column: 2,
                        index: 1
                    },
                    end: Position {
                        line: 1,
                        column: 6,
                        index: 5
                    },
                }
            },
            "\ndef"
        ))
    )
}
