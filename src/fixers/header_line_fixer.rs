use tree_sitter::Node;

use crate::constants::LINE_BREAK;
use crate::fixer::Fixer;
use crate::test_utilities::Edit;

pub struct HeaderLineFixer {}

impl HeaderLineFixer {
    fn handle_ungrouped(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        let mut tokens = source_code[node.byte_range()].to_vec();

        tokens.extend_from_slice(LINE_BREAK);

        if node.next_named_sibling().is_some() {
            tokens.extend_from_slice(LINE_BREAK);
        }

        tokens
    }

    fn handle_grouped(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        let mut tokens = source_code[node.byte_range()].to_vec();

        tokens.extend_from_slice(LINE_BREAK);

        // If the next node is different from the current one, we add an extra line break
        if node.next_named_sibling().filter(|next_node| next_node.kind() != node.kind()).is_some() {
            tokens.extend_from_slice(LINE_BREAK);
        }

        tokens
    }

    fn process(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        node.children(&mut node.walk())
            .map(|child| match child.kind() {
                "php_tag" |
                "declare_statement" |
                "namespace_definition" |
                "function_definition" |
                "class_declaration" => self.handle_ungrouped(&child, source_code),
                "namespace_use_declaration" |
                "expression_statement" => self.handle_grouped(&child, source_code),
                _ => source_code[child.byte_range()].to_vec()
            })
            .flat_map(|token| token.to_owned())
            .collect()
    }
}

impl Fixer for HeaderLineFixer {
    fn query(&self) -> &str {
        "(program) @program"
    }

    fn fix(&mut self, node: &Node, source_code: &Vec<u8>) -> Option<Edit> {
        Some(
            Edit {
                deleted_length: node.end_byte() - node.start_byte(),
                position: node.start_byte(),
                inserted_text: self.process(&node, source_code),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::fixer::FixerTestRunner;
    use crate::fixers::header_line_fixer::HeaderLineFixer;

    pub fn assert_inputs(input: &'static str, output: &'static str) {
        let mut runner = FixerTestRunner::new(input, output);
        runner.with_fixer(Box::new(HeaderLineFixer {}));
        runner.assert();
    }

    #[test]
    fn it_adds_new_line_where_necessary() {
        let input = indoc! {"
        <?php
        declare(strict_types = 1);
        namespace App\\Console;
        use App\\One;
        use App\\Two;
        class Example {}
        "};

        let output = indoc! {"
        <?php

        declare(strict_types = 1);

        namespace App\\Console;

        use App\\One;
        use App\\Two;

        class Example {}
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_correctly_fixes_statements_defined_on_the_same_line() {
        let input = indoc! {"
        <?phpdeclare(strict_types = 1);namespace App\\Console;use App\\One;use App\\Two;class Example {}
        "};

        let output = indoc! {"
        <?php

        declare(strict_types = 1);

        namespace App\\Console;

        use App\\One;
        use App\\Two;

        class Example {}
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_removes_leading_spaces_from_opening_php_tag() {
        let input = indoc! {"
            <?php
        "};
        let output = indoc! {"
        <?php
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_works_with_when_using_alias() {
        let input = indoc! {"
        <?phpuse App\\One as Um;use App\\Two as Dois;
        "};

        let output = indoc! {"
        <?php

        use App\\One as Um;
        use App\\Two as Dois;
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_removes_leading_white_spaces() {
        let input = indoc! {"
        <?php use App\\One;                    use App\\Two;
        "};

        let output = indoc! {"
        <?php

        use App\\One;
        use App\\Two;
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_separates_functions_and_expressions() {
        let input = indoc! {"
        <?php
        function a() {}
        function b() {}
        $a = 123;
        $b = 123;
        "};

        let output = indoc! {"
        <?php

        function a() {}

        function b() {}

        $a = 123;
        $b = 123;
        "};

        assert_inputs(input, output);
    }
}
