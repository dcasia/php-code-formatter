use tree_sitter::Node;

use crate::fixer::Fixer;
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

    use crate::fixer::FixerTestRunner;
    use crate::fixers::function_arguments_space_fixer::FunctionArgumentsSpaceFixer;

    pub fn assert_inputs(input: &'static str, output: &'static str) {
        let mut runner = FixerTestRunner::new(input, output);
        runner.with_fixer(Box::new(FunctionArgumentsSpaceFixer {}));
        runner.assert();
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
