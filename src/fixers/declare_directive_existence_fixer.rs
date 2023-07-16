use tree_sitter::{InputEdit, Node, Tree};

use crate::Fixer;

pub struct DeclareDirectiveExistenceFixer {}

impl DeclareDirectiveExistenceFixer {
    fn insert_token(&self, node: &Node, source_code: &mut String) -> anyhow::Result<Option<InputEdit>> {
        let tokens = "<?php declare(strict_types = 1);";

        source_code.replace_range(node.byte_range(), tokens);

        return Ok(Some(self.compute_edit(node, &tokens)));
    }
}

impl Fixer for DeclareDirectiveExistenceFixer {
    fn query(&self) -> &str {
        "(php_tag) @tag"
    }

    fn fix(&mut self, node: &Node, source_code: &mut String, tree: &Tree) -> anyhow::Result<Option<InputEdit>> {
        match node.next_sibling() {
            None => self.insert_token(node, source_code),
            Some(next_node) => {
                if next_node.kind() != "declare_statement" {
                    return self.insert_token(node, source_code);
                }

                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::fixers::declare_directive_existence_fixer::DeclareDirectiveExistenceFixer;
    use crate::test_utilities::run_fixer;

    pub fn assert_inputs(input: &str, output: &str) {
        assert_eq!(
            run_fixer(input.to_string(), DeclareDirectiveExistenceFixer {}), output
        );
    }

    #[test]
    fn it_append_the_statement_if_missing() {
        let input = indoc! {"<?php"};
        let output = indoc! {"<?php declare(strict_types = 1);"};

        assert_inputs(input, output);
    }

    #[test]
    fn it_does_nothing_if_directive_is_already_defined() {
        let input = indoc! {"<?php declare(strict_types = 0);"};
        let output = indoc! {"<?php declare(strict_types = 0);"};

        assert_inputs(input, output);
    }

    #[test]
    fn it_add_the_directive_if_first_token_is_not_declare() {
        let input = indoc! {"
        <?php
        namespace App;
        "};

        let output = indoc! {"
        <?php declare(strict_types = 1);
        namespace App;
        "};

        assert_inputs(input, output);
    }
}