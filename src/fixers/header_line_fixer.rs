use tree_sitter::Node;

use crate::constants::LINE_BREAK;
use crate::fixer::Fixer;
use crate::test_utilities::Edit;

pub struct HeaderLineFixer {}

impl HeaderLineFixer {
    fn handle(&self, source_code: &Vec<u8>, current_node: &Node, next_node: &Node) -> Option<Vec<u8>> {
        let mut tokens = source_code[current_node.byte_range()].to_vec();

        let current_node_row = current_node.start_position().row;
        let next_node_row = next_node.start_position().row;

        let is_same_line = current_node_row == next_node_row;
        let is_distinct = current_node.kind() != next_node.kind();

        println!("{:?}", current_node.utf8_text(source_code).unwrap());

        tokens.extend_from_slice(LINE_BREAK);

        if is_distinct && is_same_line {
            tokens.extend_from_slice(LINE_BREAK);
        }

        if is_same_line {
            return Some(tokens);
        }

        if current_node_row + 1 == next_node_row && is_distinct {
            return Some(tokens);
        }

        return None;
    }
}

impl Fixer for HeaderLineFixer {
    fn query(&self) -> &str {
        "(php_tag) @tag (declare_statement) @declare (namespace_definition) @namespace (namespace_use_declaration) @use"
    }

    fn fix(&mut self, node: &Node, source_code: &Vec<u8>) -> Option<Edit> {
        if let Some(next_node) = node.next_named_sibling() {
            if let Some(text) = self.handle(source_code, node, &next_node) {
                return Some(
                    Edit {
                        deleted_length: node.end_byte() - node.start_byte(),
                        position: node.start_byte(),
                        inserted_text: text,
                    }
                );
            }
        }

        None
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

        use App\\One as Um;
        use App\\Two as Dois;
        "};

        assert_inputs(input, output);
    }
}
