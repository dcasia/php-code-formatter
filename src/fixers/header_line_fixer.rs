use tree_sitter::{InputEdit, Node};

use crate::Fixer;

pub struct HeaderLineFixer {}

impl HeaderLineFixer {
    fn apply_fixer(&self, source_code: &mut String, current_node: &Node, next_node: &Node) -> anyhow::Result<Option<InputEdit>> {
        let mut tokens = current_node.utf8_text(&source_code.as_bytes())?;
        let tokens = vec![tokens, "\n"];
        let tokens = tokens.join("");
        let tokens = tokens.as_str();

        let current_node_row = current_node.start_position().row;
        let next_node_row = next_node.start_position().row;

        if current_node_row == next_node_row || (current_node_row + 1 == next_node_row && current_node.kind() != next_node.kind()) {
            source_code.replace_range(current_node.byte_range(), tokens);

            return Ok(Some(self.compute_edit(current_node, &tokens)));
        }

        return Ok(None);
    }
}

impl Fixer for HeaderLineFixer {
    fn query(&self) -> &str {
        "(php_tag) @tag (declare_statement) @declare (namespace_definition) @namespace (namespace_use_declaration)+ @use"
    }

    fn fix(&mut self, node: &Node, source_code: &mut String) -> anyhow::Result<Option<InputEdit>> {
        match node.next_sibling() {
            None => Ok(None),
            Some(next_node) => self.apply_fixer(source_code, node, &next_node)
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use crate::fixers::header_line_fixer::HeaderLineFixer;

    use crate::test_utilities::run_fixer;

    pub fn assert_inputs(input: &str, output: &str) {
        assert_eq!(
            run_fixer(input.to_string(), HeaderLineFixer {}), output
        );
    }

    #[test]
    fn it_adds_new_line_where_necessary() {
        let mut input = indoc! {"
        <?php
        declare(strict_types = 1);
        namespace App\\Console\\Commands\\Laravel;
        use Illuminate\\Console\\One;
        use Illuminate\\Console\\Two;
        class EnvironmentEncryptCommand {}
        "};

        let mut output = indoc! {"
        <?php

        declare(strict_types = 1);

        namespace App\\Console\\Commands\\Laravel;

        use Illuminate\\Console\\One;
        use Illuminate\\Console\\Two;

        class EnvironmentEncryptCommand {}
        "};

        assert_inputs(input, output);
    }
}