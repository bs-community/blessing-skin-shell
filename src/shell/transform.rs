use super::Vars;
use crate::parser::ast::*;

pub enum Argument {
    Text(String),
    Switch(String, Option<String>),
}

pub struct Transformer<'a> {
    variables: &'a Vars,
    text_only: bool,
}

impl<'a> Transformer<'a> {
    pub fn new(variables: &Vars, text_only: bool) -> Transformer {
        Transformer {
            variables,
            text_only,
        }
    }

    pub fn transform(&self, parameters: Parameters) -> Vec<Argument> {
        parameters
            .params
            .into_iter()
            .map(|param| self.parameter(param))
            .collect()
    }

    pub fn to_texts(&self, arguments: Vec<Argument>) -> Vec<String> {
        arguments
            .into_iter()
            .map(|argument| match argument {
                Argument::Text(t) => t,
                Argument::Switch(_, _) => unreachable!(),
            })
            .collect()
    }

    fn parameter(&self, parameter: Parameter) -> Argument {
        let Parameter { param, .. } = parameter;

        match param {
            Param::Literal(literal) => Argument::Text(self.template(literal.literal)),
            Param::LongSwitch(switch) => self.switch(switch, true),
            Param::ShortSwitch(switch) => self.switch(switch, false),
        }
    }

    fn switch(&self, switch: Switch, long: bool) -> Argument {
        if self.text_only {
            Argument::Text(format!(
                "{}{}",
                if long { "--" } else { "-" },
                self.switch_to_text(switch),
            ))
        } else {
            let pair = self.switch_to_pair(switch);
            Argument::Switch(pair.0, pair.1)
        }
    }

    fn switch_to_pair(&self, switch: Switch) -> (String, Option<String>) {
        let key = switch.name.name;
        let value = switch.value.map(|tpl| self.template(tpl));

        (key, value)
    }

    fn switch_to_text(&self, switch: Switch) -> String {
        format!(
            "{}{}{}",
            switch.name.name,
            if switch.value.is_some() { "=" } else { "" },
            switch
                .value
                .map(|tpl| self.template(tpl))
                .unwrap_or_default()
        )
    }

    fn template(&self, template: Template) -> String {
        match template {
            Template::Unquoted(body) => self.template_body(body),
            Template::Single(raw) => self.raw_text(raw),
            Template::Double(body) => self.template_body(body),
        }
    }

    fn raw_text(&self, raw_text: RawText) -> String {
        raw_text.text
    }

    fn template_body(&self, body: TemplateBody) -> String {
        body.parts.into_iter().fold(String::new(), |text, part| {
            text + &match part {
                TemplatePart::Raw(raw) => self.template_literal(raw),
                TemplatePart::Variable(var) => self.variable(var),
            }
        })
    }

    fn template_literal(&self, literal: TemplateLiteral) -> String {
        literal.value
    }

    fn variable(&self, variable: Variable) -> String {
        let name = &variable.id.name;
        self.variables.get(name).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn transform_raw_text() {
        let variables = HashMap::new();
        let transformer = Transformer::new(&variables, false);
        let node = RawText {
            text: "text".to_string(),
            span: Span::default(),
        };
        let result = transformer.raw_text(node);
        assert_eq!(result, "text");
    }

    #[test]
    fn transform_template_literal() {
        let variables = HashMap::new();
        let transformer = Transformer::new(&variables, false);
        let node = TemplateLiteral {
            value: "text".to_string(),
            span: Span::default(),
        };
        let result = transformer.template_literal(node);
        assert_eq!(result, "text");
    }

    #[test]
    fn transform_variable() {
        let mut variables = HashMap::with_capacity(1);
        variables.insert("kumiko".to_string(), "reina".to_string());
        let transformer = Transformer::new(&variables, false);

        let id_node = Identifier {
            name: "kumiko".to_string(),
            span: Span::default(),
        };
        let var_node = Variable {
            id: id_node,
            span: Span::default(),
        };
        assert_eq!(transformer.variable(var_node), "reina");

        let id_node = Identifier {
            name: "hazuki".to_string(),
            span: Span::default(),
        };
        let var_node = Variable {
            id: id_node,
            span: Span::default(),
        };
        assert_eq!(transformer.variable(var_node), "");
    }

    #[test]
    fn transform_template_body() {
        let mut variables = HashMap::with_capacity(1);
        variables.insert("kumiko".to_string(), "reina".to_string());
        let transformer = Transformer::new(&variables, false);

        let id_node = Identifier {
            name: "kumiko".to_string(),
            span: Span::default(),
        };
        let var_node = Variable {
            id: id_node,
            span: Span::default(),
        };
        let literal_node = TemplateLiteral {
            value: "&kumiko".to_string(),
            span: Span::default(),
        };

        let node = TemplateBody {
            parts: vec![
                TemplatePart::Variable(var_node),
                TemplatePart::Raw(literal_node),
            ],
            span: Span::default(),
        };

        assert_eq!(transformer.template_body(node), "reina&kumiko");
    }

    #[test]
    fn transform_template_unquoted() {
        let node = Template::Unquoted(TemplateBody {
            parts: vec![
                TemplatePart::Raw(TemplateLiteral {
                    value: "kumiko".to_string(),
                    span: Span::default(),
                }),
                TemplatePart::Variable(Variable {
                    id: Identifier {
                        name: "nope".to_string(),
                        span: Span::default(),
                    },
                    span: Span::default(),
                }),
            ],
            span: Span::default(),
        });

        let variables = HashMap::new();
        let transformer = Transformer::new(&variables, false);
        assert_eq!(transformer.template(node), "kumiko");
    }

    #[test]
    fn transform_template_single() {
        let node = Template::Single(RawText {
            text: "t".to_string(),
            span: Span::default(),
        });

        let variables = HashMap::new();
        let transformer = Transformer::new(&variables, false);
        assert_eq!(transformer.template(node), "t");
    }

    #[test]
    fn transform_template_double() {
        let node = Template::Double(TemplateBody {
            parts: vec![
                TemplatePart::Raw(TemplateLiteral {
                    value: "kumiko".to_string(),
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

        let mut variables = HashMap::new();
        variables.insert("var".to_string(), "-".to_string());
        let transformer = Transformer::new(&variables, false);
        assert_eq!(transformer.template(node), "kumiko-");
    }

    #[test]
    fn transform_switch_to_pair() {
        let sw = Switch {
            name: Identifier {
                name: "key".to_string(),
                span: Span::default(),
            },
            value: None,
            span: Span::default(),
        };
        let variables = HashMap::new();
        let transformer = Transformer::new(&variables, false);

        if let Argument::Switch(key, value) = transformer.switch(sw, true) {
            assert_eq!(key, "key".to_string());
            assert_eq!(value, None);
        } else {
            unreachable!();
        }

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
        let variables = HashMap::new();
        let transformer = Transformer::new(&variables, false);

        if let Argument::Switch(key, value) = transformer.switch(sw, false) {
            assert_eq!(key, "key".to_string());
            assert_eq!(value, Some("value".to_string()));
        } else {
            unreachable!();
        }
    }

    #[test]
    fn transform_switch_to_text() {
        let sw = Switch {
            name: Identifier {
                name: "key".to_string(),
                span: Span::default(),
            },
            value: None,
            span: Span::default(),
        };
        let variables = HashMap::new();
        let transformer = Transformer::new(&variables, true);
        if let Argument::Text(text) = transformer.switch(sw, true) {
            assert_eq!(text, "--key".to_string());
        } else {
            unreachable!();
        }

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
        let variables = HashMap::new();
        let transformer = Transformer::new(&variables, true);
        if let Argument::Text(text) = transformer.switch(sw, false) {
            assert_eq!(text, "-key=value".to_string());
        } else {
            unreachable!();
        }
    }

    #[test]
    fn transform_parameter() {
        let param = Parameter {
            param: Param::Literal(ParamLiteral {
                literal: Template::Single(RawText {
                    text: "t".to_string(),
                    span: Span::default(),
                }),
                span: Span::default(),
            }),
            span: Span::default(),
        };
        let variables = HashMap::new();
        let transformer = Transformer::new(&variables, true);

        if let Argument::Text(text) = transformer.parameter(param) {
            assert_eq!(text, "t".to_string());
        } else {
            unreachable!();
        }

        let param = Parameter {
            param: Param::LongSwitch(Switch {
                name: Identifier {
                    name: "t".to_string(),
                    span: Span::default(),
                },
                value: None,
                span: Span::default(),
            }),
            span: Span::default(),
        };
        let variables = HashMap::new();
        let transformer = Transformer::new(&variables, true);

        if let Argument::Text(text) = transformer.parameter(param) {
            assert_eq!(text, "--t".to_string());
        } else {
            unreachable!();
        }

        let param = Parameter {
            param: Param::ShortSwitch(Switch {
                name: Identifier {
                    name: "t".to_string(),
                    span: Span::default(),
                },
                value: None,
                span: Span::default(),
            }),
            span: Span::default(),
        };
        let variables = HashMap::new();
        let transformer = Transformer::new(&variables, true);

        if let Argument::Text(text) = transformer.parameter(param) {
            assert_eq!(text, "-t".to_string());
        } else {
            unreachable!();
        }
    }

    #[test]
    fn from_parameters_to_text() {
        let params = Parameters {
            params: vec![
                Parameter {
                    param: Param::Literal(ParamLiteral {
                        literal: Template::Single(RawText {
                            text: "1".to_string(),
                            span: Span::default(),
                        }),
                        span: Span::default(),
                    }),
                    span: Span::default(),
                },
                Parameter {
                    param: Param::Literal(ParamLiteral {
                        literal: Template::Single(RawText {
                            text: "2".to_string(),
                            span: Span::default(),
                        }),
                        span: Span::default(),
                    }),
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };
        let variables = HashMap::new();
        let transformer = Transformer::new(&variables, true);

        let text = transformer.to_texts(transformer.transform(params));
        assert_eq!("12", &text.join(""));
    }
}
