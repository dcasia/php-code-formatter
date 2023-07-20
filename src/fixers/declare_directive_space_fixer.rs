use std::ops::Deref;
use tree_sitter::{InputEdit, Node, Tree};

use crate::{Fixer};

pub struct DeclareDirectiveSpaceFixer {}

impl Fixer for DeclareDirectiveSpaceFixer {
    fn query(&self) -> &str {
        "(declare_statement (declare_directive) @fix-equal) @fix-parenthesis"
    }

    fn fix(&mut self, node: &Node, source_code: &mut String, tree: &Tree) -> anyhow::Result<(Option<Vec<u8>>, Option<InputEdit>)> {
        let tokens: Vec<u8> = node
            .children(&mut node.walk())
            .map(|child| match child.kind() {
                "=" => " = ",
                _ => &source_code[child.byte_range()]
            })
            .flat_map(|token| token.as_bytes().to_owned())
            .collect();

        Ok((Some(tokens), None))
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::fixers::declare_directive_space_fixer::DeclareDirectiveSpaceFixer;
    use crate::test_utilities::run_fixer;

    pub fn assert_inputs(input: &str, output: &str) {
        assert_eq!(
            run_fixer(input.to_string(), DeclareDirectiveSpaceFixer {}), output
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
