use tree_sitter::{InputEdit, Node, Tree};

use crate::{Fixer, WHITE_SPACE};

pub struct ArrayBracketSpaceFixer {}

impl Fixer for ArrayBracketSpaceFixer {
    fn query(&self) -> &str {
        "(array_creation_expression) @value"
    }

    fn fix(&mut self, node: &Node, source_code: &mut String, tree: &Tree) -> anyhow::Result<String> {
        let tokens: Vec<&str> = node
            .children(&mut node.walk())
            .map(|child| match child.kind() {
                "[" => "[ ",
                "]" => " ]",
                "," => ", ",
                _ => child.utf8_text(source_code.as_bytes()).unwrap()
            })
            .collect();

        Ok(tokens.join(""))
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::fixers::array_bracket_space_fixer::ArrayBracketSpaceFixer;
    use crate::test_utilities::run_fixer;

    pub fn assert_inputs(input: &str, output: &str) {
        assert_eq!(
            run_fixer(input.to_string(), ArrayBracketSpaceFixer {}), output
        );
    }

    #[test]
    fn it_add_spaces_around_brackets_and_inner_elements() {
        let input = indoc! {"
        <?php
        $value = [1,2,3];
        "};

        let output = indoc! {"
        <?php
        $value = [ 1, 2, 3 ];
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_add_space_within_nested_arrays() {
        let input = indoc! {"
        <?php
        $value = [1,2,[a,b,c],3];
        "};

        let output = indoc! {"
        <?php
        $value = [ 1, 2, [ a, b, c ], 3 ];
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_fix_array_that_contains_wierd_spaces_from_start() {
        let input = indoc! {"
        <?php
        $value = [    1,2  ,[a,  b, c
        ], 3    ];
        "};

        let output = indoc! {"
        <?php
        $value = [ 1, 2, [ a, b, c ], 3 ];
        "};

        assert_inputs(input, output);
    }
}