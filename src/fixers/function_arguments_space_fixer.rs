use tree_sitter::{InputEdit, Node, Tree};

use crate::Fixer;

pub struct FunctionArgumentsSpaceFixer {}

impl Fixer for FunctionArgumentsSpaceFixer {
    fn query(&self) -> &str {
        "(function_call_expression arguments: (arguments) @arguments)"
    }

    fn fix(&mut self, node: &Node, source_code: &mut String, tree: &Tree) -> anyhow::Result<(Option<Vec<u8>>, Option<InputEdit>)>  {
        let tokens: Vec<u8> = node
            .children(&mut node.walk())
            .map(|child| match child.kind() {
                "," => ", ",
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

    use crate::fixers::array_bracket_space_fixer::ArrayBracketSpaceFixer;
    use crate::fixers::function_arguments_space_fixer::FunctionArgumentsSpaceFixer;
    use crate::test_utilities::run_fixer;

    pub fn assert_inputs(input: &str, output: &str) {
        assert_eq!(
            run_fixer(input.to_string(), FunctionArgumentsSpaceFixer {}), output
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
