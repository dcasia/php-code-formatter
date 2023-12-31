use tree_sitter::Node;

use crate::fixer::Fixer;
use crate::test_utilities::Edit;

pub struct DeclareDirectiveExistenceFixer {}

impl DeclareDirectiveExistenceFixer {}

impl Fixer for DeclareDirectiveExistenceFixer {
    fn query(&self) -> &str {
        "(php_tag) @tag"
    }

    fn fix(&mut self, node: &Node, source_code: &Vec<u8>) -> Option<Edit> {
        let token = Vec::from("<?php declare(strict_types = 1);");

        let edit = Edit {
            deleted_length: node.end_byte() - node.start_byte(),
            position: node.start_byte(),
            inserted_text: token,
        };

        match node.next_sibling() {
            None => Some(edit),
            Some(next_node) => {
                if next_node.kind() != "declare_statement" {
                    return Some(edit);
                }

                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::fixer::FixerTestRunner;
    use crate::fixers::declare_directive_existence_fixer::DeclareDirectiveExistenceFixer;

    pub fn assert_inputs(input: &'static str, output: &'static str) {
        let mut runner = FixerTestRunner::new(input, output);
        runner.with_fixer(Box::new(DeclareDirectiveExistenceFixer {}));
        runner.assert();
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
