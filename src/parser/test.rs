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
fn parse_template_literal_with_quoted() {
    let result = template_literal(true)
        .parse(source("abc'def #$"))
        .map(|x| (x.0, x.1.input));
    assert_eq!(
        result,
        Ok((
            TemplateLiteral {
                value: "abc'def #".to_string(),
                span: Span {
                    start: Position {
                        line: 1,
                        column: 1,
                        index: 0
                    },
                    end: Position {
                        line: 1,
                        column: 10,
                        index: 9
                    }
                }
            },
            "$"
        ))
    )
}

#[test]
fn parse_template_literal_with_unquoted() {
    let result = template_literal(false)
        .parse(source("abc "))
        .map(|x| (x.0, x.1.input));
    assert_eq!(
        result,
        Ok((
            TemplateLiteral {
                value: "abc".to_string(),
                span: Span {
                    start: Position {
                        line: 1,
                        column: 1,
                        index: 0
                    },
                    end: Position {
                        line: 1,
                        column: 4,
                        index: 3
                    }
                }
            },
            " "
        ))
    )
}

#[test]
fn parse_single_dollar() {
    let result = single_dollar()
        .parse(source("$$"))
        .map(|x| (x.0, x.1.input));
    assert_eq!(
        result,
        Ok((
            TemplateLiteral {
                value: "$".to_string(),
                span: Span {
                    start: Position {
                        line: 1,
                        column: 1,
                        index: 0
                    },
                    end: Position {
                        line: 1,
                        column: 2,
                        index: 1
                    }
                }
            },
            "$"
        ))
    )
}

#[test]
fn parse_template_body_with_quoted() {
    let result = template_body(true).parse(source("")).map(|x| x.0);
    assert_eq!(
        result,
        Ok(TemplateBody {
            parts: vec![],
            span: Span {
                start: Position {
                    line: 1,
                    column: 1,
                    index: 0,
                },
                end: Position {
                    line: 1,
                    column: 1,
                    index: 0,
                }
            }
        })
    );

    let result = template_body(true).parse(source("ab$$what")).map(|x| x.0);
    assert_eq!(
        result,
        Ok(TemplateBody {
            parts: vec![
                TemplatePart::Raw(TemplateLiteral {
                    value: "ab".to_string(),
                    span: Span {
                        start: Position {
                            line: 1,
                            column: 1,
                            index: 0,
                        },
                        end: Position {
                            line: 1,
                            column: 3,
                            index: 2,
                        }
                    }
                }),
                TemplatePart::Raw(TemplateLiteral {
                    value: "$".to_string(),
                    span: Span {
                        start: Position {
                            line: 1,
                            column: 3,
                            index: 2,
                        },
                        end: Position {
                            line: 1,
                            column: 4,
                            index: 3,
                        }
                    }
                }),
                TemplatePart::Variable(Variable {
                    id: Identifier {
                        name: "what".to_string(),
                        span: Span {
                            start: Position {
                                line: 1,
                                column: 5,
                                index: 4,
                            },
                            end: Position {
                                line: 1,
                                column: 9,
                                index: 8,
                            }
                        }
                    },
                    span: Span {
                        start: Position {
                            line: 1,
                            column: 4,
                            index: 3,
                        },
                        end: Position {
                            line: 1,
                            column: 9,
                            index: 8,
                        }
                    }
                }),
            ],
            span: Span {
                start: Position {
                    line: 1,
                    column: 1,
                    index: 0,
                },
                end: Position {
                    line: 1,
                    column: 9,
                    index: 8,
                }
            }
        })
    );
}

#[test]
fn parse_template_body_with_unquoted() {
    let result = template_body(false).parse(source(""));
    assert!(result.is_err());
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
