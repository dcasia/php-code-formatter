use tree_sitter::{InputEdit, Node, Point, Tree};

use crate::Fixer;
use crate::test_utilities::{is_multiline, debug_node};

pub struct IdentFixer {}

impl Fixer for IdentFixer {
    fn query(&self) -> &str {
        "(use_declaration) @use"
    }

    fn fix(&mut self, node: &Node, source_code: &mut String, tree: &Tree) -> anyhow::Result<(Option<Vec<u8>>, Option<InputEdit>)> {
        if node.start_position().column == 4 {
            return Ok((None, None));
        }

        if node.start_position().column == 0 {
            let mut tokens = source_code[node.byte_range()].as_bytes().to_vec();
            let mut ident = b"    ".to_vec();

            ident.append(&mut tokens);

            return Ok((Some(ident), None));
        }

        let current_node_start = node.start_position();
        let previous_node_end = node.prev_sibling().unwrap().end_position();

        // nodes start at different lines, then column can start from 0
        if current_node_start.row != previous_node_end.row {
            let start_byte = node.start_byte() - current_node_start.column;
            let difference = node.start_byte() - start_byte;

            println!("{:?}", &source_code[start_byte..node.end_byte()]);
            println!("{:?}", &source_code[node.start_byte()..node.end_byte()]);
            println!("{:?}", difference);

            let mut tokens = source_code[node.byte_range()].as_bytes().to_vec();

            let edit = InputEdit {
                start_byte,
                start_position: Point::new(node.start_position().row, 0),
                old_end_byte: node.end_byte(),
                old_end_position: node.end_position(),
                new_end_byte: node.end_byte() -2,
                new_end_position: Point::new(
                    node.start_position().row,
                    0 + tokens.len(),
                ),
            };

            return Ok((Some(tokens), Some(edit)));
        }

        println!("{:?}", node.start_position());
        println!("{:?}", node.prev_sibling().unwrap().end_position());

        Ok((None, None))
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use crate::fixers::indent_fixer::IdentFixer;

    use crate::test_utilities::run_fixer;

    pub fn assert_inputs(input: &str, output: &str) {
        assert_eq!(
            run_fixer(input.to_string(), IdentFixer {}), output
        );
    }

    #[test]
    fn it_correctly_ident_use_traits_declaration() {
        let input = indoc! {"
        <?php
        class Test {
        use SomeTrait;
            use SomeOtherTrait;
                use SomeOtherSuperTrait;
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            use SomeTrait;
            use SomeOtherTrait;
            use SomeOtherSuperTrait;
        }
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
