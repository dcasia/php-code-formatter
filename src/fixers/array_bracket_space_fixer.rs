use tree_sitter::{InputEdit, Node, Tree};

use crate::Fixer;
use crate::test_utilities::{is_multiline, debug_node, Edit};

pub struct ArrayBracketSpaceFixer {}

impl Fixer for ArrayBracketSpaceFixer {
    fn query(&self) -> &str {
        "(array_creation_expression) @value"
    }

    fn fix(&mut self, node: &Node, source_code: &mut Vec<u8>, tree: &Tree) -> Option<Edit> {
        let tokens: Vec<u8> = node
            .children(&mut node.walk())
            .map(|child| match child.kind() {
                "[" => {
                    if let Some(next) = child.next_sibling() {
                        if next.kind() == "]" { return "[".as_bytes(); }
                    }
                    "[ ".as_bytes()
                }
                "]" => {
                    if let Some(next) = child.prev_sibling() {
                        if next.kind() == "[" { return "]".as_bytes(); }
                    }
                    " ]".as_bytes()
                }
                "," => ", ".as_bytes(),
                _ => &source_code[child.byte_range()]
            })
            .flat_map(|token| token.to_owned())
            .collect();

        Some(
            Edit {
                deleted_length: node.end_byte() - node.start_byte(),
                position: node.start_byte(),
                inserted_text: tokens,
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::fixers::array_bracket_space_fixer::ArrayBracketSpaceFixer;
    use crate::test_utilities::run_fixer;

    pub fn assert_inputs(input: &str, output: &str) {
        assert_eq!(
            run_fixer(input.into(), ArrayBracketSpaceFixer {}), output.as_bytes().to_vec()
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

    #[test]
    fn it_does_not_add_spaces_within_blank_arrays() {
        let input_output = indoc! {"<?php $value = [];"};

        assert_inputs(input_output, input_output);
    }
}
