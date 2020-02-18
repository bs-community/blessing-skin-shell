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

fn parameters(parameters: &Parameters, prefix_spaces: usize) -> String {
    parameters
        .params
        .iter()
        .fold((prefix_spaces, String::new()), |(pos, output), param| {
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
    let mut output = String::new();

    output.push_str(&white_space(command.span.start.index));
    output.push_str(&program(&command.program, executables));
    if let Some(params) = &command.parameters {
        output.push_str(&parameters(&params, command.program.span.end.index));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
