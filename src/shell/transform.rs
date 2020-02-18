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

    fn parameter(&self, parameter: Parameter) -> Argument {
        let Parameter { param, .. } = parameter;

        match param {
            Param::Literal(literal) => Argument::Text(self.template(literal.literal)),
            Param::LongSwitch(switch) | Param::ShortSwitch(switch) => self.switch(switch),
        }
    }

    fn switch(&self, switch: Switch) -> Argument {
        if self.text_only {
            Argument::Text(self.switch_to_text(switch))
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
            "{}={}",
            switch.name.name,
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

        let mut parts = Vec::with_capacity(2);
        parts.push(TemplatePart::Variable(var_node));
        parts.push(TemplatePart::Raw(literal_node));
        let node = TemplateBody {
            parts,
            span: Span::default(),
        };

        assert_eq!(transformer.template_body(node), "reina&kumiko");
    }
}
