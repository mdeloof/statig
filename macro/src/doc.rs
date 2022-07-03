use crate::analyze;
use crate::analyze::Node;
use serde::Serialize;
use serde_json::Value;
use std::fmt::Write;
use syn::parse_quote;
use syn::Attribute;
use tinytemplate::TinyTemplate;

static STATE_TEMPLATE: &str = include_str!("./doc_templates/state_template.html");
static STYLE_CSS: &str = include_str!("./doc_templates/style.css");
static SCRIPT_JS: &str = include_str!("./doc_templates/min_script.js");

/// A state that is part of the state machine tree.
#[derive(Debug, Serialize)]
pub struct State {
    name: String,
    level: usize,
    entry_action: Option<String>,
    exit_action: Option<String>,
    sub_states: Vec<State>,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub fn document(model: &analyze::Model) -> Attribute {
    let mut state_machine_tree = vec![];

    // Recursively iterate over the states in the state machine tree from the model
    // and retrieve the necessary documentation.
    for state in model.tree.clone() {
        state_machine_tree.push(document_state(state, model, 1));
    }

    let mut html_template = TinyTemplate::new();

    html_template
        .add_template("state_template", STATE_TEMPLATE)
        .unwrap();

    html_template.add_formatter("entry_action_formatter", |value, fmt| {
        if let Value::String(entry_action) = value {
            write!(
                fmt,
                r##"<span>⦿ <a href="#method.{entry_action}"><code>{entry_action}</code></a></span>&emsp;"##
            )
            .unwrap();
        }
        Ok(())
    });

    html_template.add_formatter("exit_action_formatter", |value, fmt| {
        if let Value::String(exit_action) = value {
            write!(
                fmt,
                r##"<span>⦻ <a href="#method.{exit_action}"><code>{exit_action}</code></a></span>"##
            )
            .unwrap();
        }
        Ok(())
    });

    let mut states_html = String::new();

    for state in state_machine_tree {
        states_html.push_str(&html_template.render("state_template", &state).unwrap())
    }

    let rendered = format!(
        "<style>\n{}\n</style>\n<div class=\"state-machine\" style=\"overflow: visible;\">\n{}\n</div>\n<script>\n{}\n</script>",
        STYLE_CSS, states_html, SCRIPT_JS.chars().filter(|&c| c != '\n').collect::<String>()
    );

    parse_quote!(#[doc=#rendered])
}

pub fn document_state(state: Node, model: &analyze::Model, level: usize) -> State {
    match state {
        Node::State { name } => {
            let entry_action = model
                .states
                .get(&name)
                .and_then(|state| state.entry_action.as_ref())
                .map(|action| action.to_string());
            let exit_action = model
                .states
                .get(&name)
                .and_then(|state| state.exit_action.as_ref())
                .map(|action| action.to_string());
            let sub_states = Vec::with_capacity(0);

            State {
                name: name.to_string(),
                level,
                entry_action,
                exit_action,
                sub_states,
            }
        }

        Node::Superstate { name, sub_state } => {
            let entry_action = model
                .states
                .get(&name)
                .and_then(|state| state.entry_action.as_ref())
                .map(|action| action.to_string());
            let exit_action = model
                .states
                .get(&name)
                .and_then(|state| state.exit_action.as_ref())
                .map(|action| action.to_string());
            let sub_states = sub_state
                .iter()
                .map(|state| document_state(state.clone(), model, level + 1))
                .collect();

            State {
                name: name.to_string(),
                level,
                entry_action,
                exit_action,
                sub_states,
            }
        }
    }
}
