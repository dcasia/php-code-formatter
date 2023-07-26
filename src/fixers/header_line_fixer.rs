use tree_sitter::{InputEdit, Node, Tree};

use crate::{Fixer, NEW_LINE};
use crate::test_utilities::Edit;

pub struct HeaderLineFixer {}

impl HeaderLineFixer {
    fn apply_fixer(&self, source_code: &mut Vec<u8>, current_node: &Node, next_node: &Node) -> anyhow::Result<Edit> {
        todo!();
        // let mut tokens = source_code[current_node.byte_range()].as_bytes().to_vec();
        // tokens.push(NEW_LINE);
        //
        // let current_node_row = current_node.start_position().row;
        // let next_node_row = next_node.start_position().row;
        //
        // if current_node_row == next_node_row {
        //     return Ok((Some(tokens), None));
        // }
        //
        // if current_node_row + 1 == next_node_row && current_node.kind() != next_node.kind() {
        //     return Ok((Some(tokens), None));
        // }
        //
        // return Ok((None, None));
    }
}

impl Fixer for HeaderLineFixer {
    fn query(&self) -> &str {
        "(php_tag) @tag (declare_statement) @declare (namespace_definition) @namespace (namespace_use_declaration) @use"
    }

    fn fix(&mut self, node: &Node, source_code: &Vec<u8>, tree: &Tree) -> Option<Edit> {
        todo!();
        // match node.next_sibling() {
        //     None => Ok((None, None)),
        //     Some(next_node) => self.apply_fixer(source_code, node, &next_node)
        // }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::fixers::header_line_fixer::HeaderLineFixer;
    use crate::test_utilities::run_fixer;

    pub fn assert_inputs(input: &str, output: &str) {
        assert_eq!(
            run_fixer(input.to_string().into(), HeaderLineFixer {}), output.as_bytes().to_vec()
        );
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
}
