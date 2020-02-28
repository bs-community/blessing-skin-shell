use crate::parser::ast::*;
use ansi_term::Color;
use std::collections::HashMap;

fn white_space(size: usize) -> String {
    " ".repeat(size)
}

fn template(template: &Template) -> String {
    match template {
        Template::Unquoted(body) => {
            body.parts.iter().enumerate().fold(
                String::with_capacity(body.parts.len() * 5),
                |output, (i, part)| {
                    output
                        + &match part {
                            TemplatePart::Raw(literal) => {
                                // User may input a switch later, so paint it with light blue.
                                if &literal.value == "-" && i == 0 {
                                    Color::Fixed(39).paint("-").to_string()
                                } else {
                                    literal.value.to_owned()
                                }
                            }
                            TemplatePart::Variable(var) => variable(var),
                        }
                },
            )
        }
        Template::Single(raw) => Color::Yellow.paint(format!("'{}'", raw.text)).to_string(),
        Template::Double(body) => {
            let middle = body.parts.iter().fold(
                String::with_capacity(body.parts.len() * 5),
                |output, part| {
                    output
                        + &match part {
                            TemplatePart::Raw(literal) => {
                                Color::Yellow.paint(&literal.value).to_string()
                            }
                            TemplatePart::Variable(var) => variable(var),
                        }
                },
            );

            let quote = Color::Yellow.paint("\"");
            format!("{}{}{}", quote, middle, quote)
        }
    }
}

fn variable(variable: &Variable) -> String {
    Color::Fixed(93)
        .paint(format!("${}", variable.id.name))
        .to_string()
}

fn switch(switch: &Switch) -> String {
    let mut output = String::with_capacity(3);

    output.push_str(&Color::Fixed(39).paint(&switch.name.name).to_string());
    if let Some(value) = &switch.value {
        output.push_str(&Color::Fixed(39).paint("=").to_string());
        output.push_str(&template(&value));
    }

    output
}

fn parameter(parameter: &Parameter) -> String {
    match &parameter.param {
        Param::Literal(literal) => template(&literal.literal),
        Param::LongSwitch(sw) => format!("{}{}", Color::Fixed(39).paint("--"), switch(&sw)),
        Param::ShortSwitch(sw) => format!("{}{}", Color::Fixed(39).paint("-"), switch(&sw)),
    }
}

fn parameters(parameters: &Parameters, prefix_idx: usize) -> String {
    parameters
        .params
        .iter()
        .fold((prefix_idx, String::new()), |(pos, output), param| {
            (
                param.span.end.index,
                output + &white_space(param.span.start.index - pos) + &parameter(param),
            )
        })
        .1
}

fn program<T>(program: &Program, executables: &HashMap<String, T>) -> String {
    if executables.keys().any(|exec| exec == &program.id.name) {
        Color::Green.paint(&program.id.name).to_string()
    } else {
        Color::Red.paint(&program.id.name).to_string()
    }
}

pub(super) fn command<T>(command: &Command, executables: &HashMap<String, T>) -> String {
    let mut output = format!(
        "{}{}",
        white_space(command.span.start.index),
        program(&command.program, executables)
    );

    if let Some(params) = &command.parameters {
        output.push_str(&parameters(&params, command.program.span.end.index));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn spaces() {
        assert_eq!("  ".to_string(), white_space(2));
    }

    #[test]
    fn render_template_unquoted() {
        let unquoted = Template::Unquoted(TemplateBody {
            parts: vec![TemplatePart::Raw(TemplateLiteral {
                value: "-".to_string(),
                span: Span::default(),
            })],
            span: Span::default(),
        });
        let output = template(&unquoted);
        assert_eq!(output, Color::Fixed(39).paint("-").to_string());

        let unquoted = Template::Unquoted(TemplateBody {
            parts: vec![TemplatePart::Raw(TemplateLiteral {
                value: "text".to_string(),
                span: Span::default(),
            })],
            span: Span::default(),
        });
        let output = template(&unquoted);
        assert_eq!(output, "text".to_string());

        let unquoted = Template::Unquoted(TemplateBody {
            parts: vec![
                TemplatePart::Raw(TemplateLiteral {
                    value: "text".to_string(),
                    span: Span::default(),
                }),
                TemplatePart::Variable(Variable {
                    id: Identifier {
                        name: "var".to_string(),
                        span: Span::default(),
                    },
                    span: Span::default(),
                }),
            ],
            span: Span::default(),
        });
        let output = template(&unquoted);
        assert_eq!(output, format!("text{}", Color::Fixed(93).paint("$var")));
    }

    #[test]
    fn render_template_single() {
        let single = Template::Single(RawText {
            text: "raw".to_string(),
            span: Span::default(),
        });

        let output = template(&single);
        assert_eq!(output, Color::Yellow.paint("'raw'").to_string())
    }

    #[test]
    fn render_template_double() {
        let double = Template::Double(TemplateBody {
            parts: vec![
                TemplatePart::Raw(TemplateLiteral {
                    value: "-".to_string(),
                    span: Span::default(),
                }),
                TemplatePart::Variable(Variable {
                    id: Identifier {
                        name: "var".to_string(),
                        span: Span::default(),
                    },
                    span: Span::default(),
                }),
            ],
            span: Span::default(),
        });
        let output = template(&double);

        let content = format!(
            "{}{}",
            Color::Yellow.paint("-"),
            Color::Fixed(93).paint("$var")
        );
        let quote = Color::Yellow.paint("\"");
        assert_eq!(output, format!("{}{}{}", quote, content, quote));
    }

    #[test]
    fn render_variable() {
        let var = Variable {
            id: Identifier {
                name: "v".to_string(),
                span: Span::default(),
            },
            span: Span::default(),
        };

        let output = variable(&var);
        assert_eq!(output, Color::Fixed(93).paint("$v").to_string());
    }

    #[test]
    fn render_switch() {
        let sw = Switch {
            name: Identifier {
                name: "key".to_string(),
                span: Span::default(),
            },
            value: None,
            span: Span::default(),
        };

        let output = switch(&sw);
        assert_eq!(output, Color::Fixed(39).paint("key").to_string());

        let sw = Switch {
            name: Identifier {
                name: "key".to_string(),
                span: Span::default(),
            },
            value: Some(Template::Single(RawText {
                text: "value".to_string(),
                span: Span::default(),
            })),
            span: Span::default(),
        };
        let output = switch(&sw);
        assert_eq!(
            output,
            vec![
                Color::Fixed(39).paint("key"),
                Color::Fixed(39).paint("="),
                Color::Yellow.paint("'value'")
            ]
            .iter()
            .map(|s| s.to_string())
            .join("")
        );
    }

    #[test]
    fn render_parameter_literal() {
        let param = Parameter {
            param: Param::Literal(ParamLiteral {
                literal: Template::Unquoted(TemplateBody {
                    parts: vec![TemplatePart::Raw(TemplateLiteral {
                        value: "text".to_string(),
                        span: Span::default(),
                    })],
                    span: Span::default(),
                }),
                span: Span::default(),
            }),
            span: Span::default(),
        };

        let output = parameter(&param);
        assert_eq!(output, "text".to_string());
    }

    #[test]
    fn render_parameter_short_switch() {
        let param = Parameter {
            param: Param::ShortSwitch(Switch {
                name: Identifier {
                    name: "key".to_string(),
                    span: Span::default(),
                },
                value: None,
                span: Span::default(),
            }),
            span: Span::default(),
        };

        let output = parameter(&param);
        assert_eq!(
            output,
            format!(
                "{}{}",
                Color::Fixed(39).paint("-"),
                Color::Fixed(39).paint("key")
            )
        );
    }

    #[test]
    fn render_parameter_long_switch() {
        let param = Parameter {
            param: Param::LongSwitch(Switch {
                name: Identifier {
                    name: "key".to_string(),
                    span: Span::default(),
                },
                value: None,
                span: Span::default(),
            }),
            span: Span::default(),
        };

        let output = parameter(&param);
        assert_eq!(
            output,
            format!(
                "{}{}",
                Color::Fixed(39).paint("--"),
                Color::Fixed(39).paint("key")
            )
        );
    }

    #[test]
    fn render_parameters() {
        let params = Parameters {
            params: vec![
                Parameter {
                    param: Param::Literal(ParamLiteral {
                        literal: Template::Unquoted(TemplateBody {
                            parts: vec![TemplatePart::Raw(TemplateLiteral {
                                value: "ab".to_string(),
                                span: Span {
                                    start: Position {
                                        line: 1,
                                        column: 3,
                                        index: 2,
                                    },
                                    end: Position {
                                        line: 1,
                                        column: 5,
                                        index: 4,
                                    },
                                },
                            })],
                            span: Span {
                                start: Position {
                                    line: 1,
                                    column: 3,
                                    index: 2,
                                },
                                end: Position {
                                    line: 1,
                                    column: 5,
                                    index: 4,
                                },
                            },
                        }),
                        span: Span {
                            start: Position {
                                line: 1,
                                column: 3,
                                index: 2,
                            },
                            end: Position {
                                line: 1,
                                column: 5,
                                index: 4,
                            },
                        },
                    }),
                    span: Span {
                        start: Position {
                            line: 1,
                            column: 3,
                            index: 2,
                        },
                        end: Position {
                            line: 1,
                            column: 5,
                            index: 4,
                        },
                    },
                },
                Parameter {
                    param: Param::Literal(ParamLiteral {
                        literal: Template::Unquoted(TemplateBody {
                            parts: vec![TemplatePart::Raw(TemplateLiteral {
                                value: "cd".to_string(),
                                span: Span {
                                    start: Position {
                                        line: 1,
                                        column: 7,
                                        index: 6,
                                    },
                                    end: Position {
                                        line: 1,
                                        column: 9,
                                        index: 8,
                                    },
                                },
                            })],
                            span: Span {
                                start: Position {
                                    line: 1,
                                    column: 7,
                                    index: 6,
                                },
                                end: Position {
                                    line: 1,
                                    column: 9,
                                    index: 8,
                                },
                            },
                        }),
                        span: Span {
                            start: Position {
                                line: 1,
                                column: 7,
                                index: 6,
                            },
                            end: Position {
                                line: 1,
                                column: 9,
                                index: 8,
                            },
                        },
                    }),
                    span: Span {
                        start: Position {
                            line: 1,
                            column: 7,
                            index: 6,
                        },
                        end: Position {
                            line: 1,
                            column: 9,
                            index: 8,
                        },
                    },
                },
            ],
            span: Span::default(),
        };

        let output = parameters(&params, 0);
        assert_eq!(&output, "  ab  cd");
    }

    #[test]
    fn render_program() {
        use std::collections::HashMap;

        let mut executables = HashMap::new();
        executables.insert("clear".to_string(), ());

        let output = program(
            &Program {
                id: Identifier {
                    name: "nope".to_string(),
                    span: Span::default(),
                },
                span: Span::default(),
            },
            &executables,
        );
        assert_eq!(Color::Red.paint("nope").to_string(), output);

        let output = program(
            &Program {
                id: Identifier {
                    name: "clear".to_string(),
                    span: Span::default(),
                },
                span: Span::default(),
            },
            &executables,
        );
        assert_eq!(Color::Green.paint("clear").to_string(), output);
    }

    #[test]
    fn render_command() {
        let c = Command {
            program: Program {
                id: Identifier {
                    name: "test".to_string(),
                    span: Span {
                        start: Position {
                            line: 1,
                            column: 2,
                            index: 1,
                        },
                        end: Position {
                            line: 1,
                            column: 6,
                            index: 5,
                        },
                    },
                },
                span: Span {
                    start: Position {
                        line: 1,
                        column: 2,
                        index: 1,
                    },
                    end: Position {
                        line: 1,
                        column: 6,
                        index: 5,
                    },
                },
            },
            parameters: None,
            span: Span {
                start: Position {
                    line: 1,
                    column: 2,
                    index: 1,
                },
                end: Position {
                    line: 1,
                    column: 6,
                    index: 5,
                },
            },
        };
        let output = command::<()>(&c, &HashMap::new());
        assert_eq!(output, format!(" {}", Color::Red.paint("test")));

        let c = Command {
            program: Program {
                id: Identifier {
                    name: "test".to_string(),
                    span: Span {
                        start: Position {
                            line: 1,
                            column: 2,
                            index: 1,
                        },
                        end: Position {
                            line: 1,
                            column: 6,
                            index: 5,
                        },
                    },
                },
                span: Span {
                    start: Position {
                        line: 1,
                        column: 2,
                        index: 1,
                    },
                    end: Position {
                        line: 1,
                        column: 6,
                        index: 5,
                    },
                },
            },
            parameters: Some(Parameters {
                params: vec![
                    Parameter {
                        param: Param::Literal(ParamLiteral {
                            literal: Template::Unquoted(TemplateBody {
                                parts: vec![TemplatePart::Raw(TemplateLiteral {
                                    value: "ab".to_string(),
                                    span: Span {
                                        start: Position {
                                            line: 1,
                                            column: 9,
                                            index: 8,
                                        },
                                        end: Position {
                                            line: 1,
                                            column: 11,
                                            index: 10,
                                        },
                                    },
                                })],
                                span: Span {
                                    start: Position {
                                        line: 1,
                                        column: 9,
                                        index: 8,
                                    },
                                    end: Position {
                                        line: 1,
                                        column: 11,
                                        index: 10,
                                    },
                                },
                            }),
                            span: Span {
                                start: Position {
                                    line: 1,
                                    column: 9,
                                    index: 8,
                                },
                                end: Position {
                                    line: 1,
                                    column: 11,
                                    index: 10,
                                },
                            },
                        }),
                        span: Span {
                            start: Position {
                                line: 1,
                                column: 9,
                                index: 8,
                            },
                            end: Position {
                                line: 1,
                                column: 11,
                                index: 10,
                            },
                        },
                    },
                    Parameter {
                        param: Param::Literal(ParamLiteral {
                            literal: Template::Unquoted(TemplateBody {
                                parts: vec![TemplatePart::Raw(TemplateLiteral {
                                    value: "cd".to_string(),
                                    span: Span {
                                        start: Position {
                                            line: 1,
                                            column: 12,
                                            index: 11,
                                        },
                                        end: Position {
                                            line: 1,
                                            column: 14,
                                            index: 13,
                                        },
                                    },
                                })],
                                span: Span {
                                    start: Position {
                                        line: 1,
                                        column: 12,
                                        index: 11,
                                    },
                                    end: Position {
                                        line: 1,
                                        column: 14,
                                        index: 13,
                                    },
                                },
                            }),
                            span: Span {
                                start: Position {
                                    line: 1,
                                    column: 12,
                                    index: 11,
                                },
                                end: Position {
                                    line: 1,
                                    column: 14,
                                    index: 13,
                                },
                            },
                        }),
                        span: Span {
                            start: Position {
                                line: 1,
                                column: 12,
                                index: 11,
                            },
                            end: Position {
                                line: 1,
                                column: 14,
                                index: 13,
                            },
                        },
                    },
                ],
                span: Span {
                    start: Position {
                        line: 1,
                        column: 9,
                        index: 8,
                    },
                    end: Position {
                        line: 1,
                        column: 14,
                        index: 13,
                    },
                },
            }),
            span: Span {
                start: Position {
                    line: 1,
                    column: 2,
                    index: 1,
                },
                end: Position {
                    line: 1,
                    column: 14,
                    index: 13,
                },
            },
        };
        let mut executables = HashMap::new();
        executables.insert("test".to_string(), ());
        let output = command(&c, &executables);
        assert_eq!(output, format!(" {}   ab cd", Color::Green.paint("test")));
    }
}
