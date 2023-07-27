use tree_sitter::{InputEdit, Node, Tree};

use crate::Fixer;
use crate::test_utilities::Edit;

pub struct FunctionArgumentsSpaceFixer {}

impl Fixer for FunctionArgumentsSpaceFixer {
    fn query(&self) -> &str {
        "(function_call_expression arguments: (arguments) @arguments)"
    }

    fn fix(&mut self, node: &Node, source_code: &Vec<u8>) -> Option<Edit> {
        let tokens: Vec<u8> = node
            .children(&mut node.walk())
            .map(|child| match child.kind() {
                "," => b", ",
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
    use crate::fixers::function_arguments_space_fixer::FunctionArgumentsSpaceFixer;
    use crate::test_utilities::run_fixer;

    pub fn assert_inputs(input: &str, output: &str) {
        assert_eq!(
            run_fixer(input.to_string().into(), FunctionArgumentsSpaceFixer {}), output.as_bytes().to_vec()
        );
    }

    #[test]
    fn it_add_spaces_between_each_argument() {
        let input = indoc! {"
        <?php
        global_function(1,2,3,   4, 5);
        "};

        let output = indoc! {"
        <?php
        global_function(1, 2, 3, 4, 5);
        "};

        assert_inputs(input, output);
    }
}
