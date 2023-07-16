use tree_sitter::{InputEdit, Node};

use crate::{Fixer, WHITE_SPACE};

pub struct ArrayBracketSpaceFixer {}

impl Fixer for ArrayBracketSpaceFixer {
    fn query(&self) -> &str {
        "(array_creation_expression) @value"
    }

    fn fix(&mut self, node: &Node, source_code: &mut String) -> anyhow::Result<Option<InputEdit>> {
        self.build_sequence(node, source_code, |token| {
            match token {
                "[" => vec![token, WHITE_SPACE],
                "]" => vec![WHITE_SPACE, token],
                "," => vec![token, WHITE_SPACE],
                _ => vec![token]
            }
        })
    }
}
