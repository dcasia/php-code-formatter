use std::ops::Deref;
use tree_sitter::{InputEdit, Node, Tree};

use crate::{Fixer};
use crate::test_utilities::Edit;

pub struct DeclareDirectiveSpaceFixer {}

impl Fixer for DeclareDirectiveSpaceFixer {
    fn query(&self) -> &str {
        "(declare_statement (declare_directive) @fix-equal) @fix-parenthesis"
    }

    fn fix(&mut self, node: &Node, source_code: &Vec<u8>, tree: &Tree) -> Option<Edit> {
        let tokens: Vec<u8> = node
            .children(&mut node.walk())
            .map(|child| match child.kind() {
                "=" => b" = ",
                _ => &source_code[child.byte_range()]
            })
            .flat_map(|token| token.to_owned())
            .collect();

        Some(Edit {
            deleted_length: node.end_byte() - node.start_byte(),
            position: node.start_byte(),
            inserted_text: tokens,
        })
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::fixers::declare_directive_space_fixer::DeclareDirectiveSpaceFixer;
    use crate::test_utilities::run_fixer;

    pub fn assert_inputs(input: &str, output: &str) {
        assert_eq!(
            run_fixer(input.to_string().into(), DeclareDirectiveSpaceFixer {}), output.as_bytes().to_vec()
        );
    }

    #[test]
    fn it_add_space_between_equal_token() {
        let input = indoc! {"
        <?php
        declare(strict_types=1);
        "};

        let output = indoc! {"
        <?php
        declare(strict_types = 1);
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_keeps_the_original_integer_value() {
        let input = indoc! {"
        <?php
        declare(strict_types=0);
        "};

        let output = indoc! {"
        <?php
        declare(strict_types = 0);
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_removes_spaces_between_open_close_parenthesis() {
        let input = indoc! {"
        <?php
        declare( strict_types=1 );
        "};

        let output = indoc! {"
        <?php
        declare(strict_types = 1);
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_removes_excessive_spaces() {
        let input = indoc! {"
        <?php
        declare(     strict_types      =        1      );
        "};

        let output = indoc! {"
        <?php
        declare(strict_types = 1);
        "};

        assert_inputs(input, output);
    }
}
