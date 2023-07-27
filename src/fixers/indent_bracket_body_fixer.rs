use std::ops::Sub;

use anyhow::Context;
use tree_sitter::Node;

use crate::constants::{IDENT, IDENT_STR, LINE_BREAK, LINE_BREAK_STR};
use crate::fixer::Fixer;
use crate::test_utilities::Edit;

pub struct IndentBracketBodyFixer {}

impl IndentBracketBodyFixer {
    fn ident_compound_statement_node(
        &self,
        node: &Node,
        parent: &Node,
        current_ident: &mut Vec<u8>,
        source_code: &Vec<u8>,
        level: usize,
    )
    {
        let ident_size = IDENT.len();
        let line_break_size = LINE_BREAK.len();
        let node_start_byte = node.start_byte();

        let mut inner_edit = self.process(&node, source_code, level);
        let mut ident_level = ident_size * level;
        let mut sub_ident_by = 0;

        if let Some(previous_node) = node.prev_sibling() {
            //--------------------------------------------------------------------------------------
            let previous_node_end_byte = previous_node.end_byte();
            let difference = node_start_byte - previous_node_end_byte;
            let is_over_indented = node_start_byte > previous_node_end_byte + ident_level;

            if is_over_indented == true {
                sub_ident_by = difference - ident_level - line_break_size;
            }

            if is_over_indented == false {
                //----------------------------------------------------------------------------------
                let repeat_by = (ident_level + line_break_size)
                    .checked_sub(difference)
                    .unwrap_or(0);

                let mut ident = b" ".repeat(repeat_by % ident_level);

                ident.append(&mut inner_edit);

                inner_edit = ident;
                //----------------------------------------------------------------------------------
            }
            //--------------------------------------------------------------------------------------
        }

        let start_offset = node_start_byte - parent.start_byte() + ident_level - sub_ident_by;
        let end_offset = start_offset + node.byte_range().count() - line_break_size + sub_ident_by;

        current_ident.splice(start_offset..=end_offset, inner_edit);
    }

    fn handle_switch_block<'a>(&self, node: Node<'a>) -> Option<Node<'a>> {
        // todo
        // maybe is better to crash the software to teach a lesson to the users who uses switch statement
        Some(node)
    }

    fn handle_anonymous_function<'a>(&self, node: Node<'a>) -> Option<Node<'a>> {
        node.child_by_field_name("right")
            .filter(|node| node.kind() == "anonymous_function_creation_expression")
            .map(|node| node.child_by_field_name("body"))
            .unwrap_or(None)
    }

    fn handle_default<'a>(&self, node: Node<'a>) -> Option<Node<'a>> {
        node.child_by_field_name("body")
            .filter(|node| match node.kind() {
                "compound_statement" => true,
                "match_block" => true,
                _ => false
            })
    }

    fn handle_node(&self, child: &Node, source_code: &Vec<u8>, level: usize) -> Vec<u8> {
        let mut tokens = source_code[child.byte_range()].to_vec();
        let current_level = level + 1;

        let mut ident = IDENT.repeat(current_level).to_vec();
        ident.append(&mut tokens);

        if let Some(_) = child.next_sibling().filter(|node| node.kind() != ",") {
            ident.extend_from_slice(LINE_BREAK);
        }

        for inner_child in child.children(&mut child.walk()) {
            //--------------------------------------------------------------------------------------
            let node: Option<Node> = match inner_child.kind() {
                "compound_statement" => Some(inner_child),
                "switch_block" => self.handle_switch_block(inner_child),
                "assignment_expression" => self.handle_anonymous_function(inner_child),
                _ => self.handle_default(inner_child),
            };

            if let Some(inner_child) = node {
                //----------------------------------------------------------------------------------
                self.ident_compound_statement_node(
                    &inner_child, &child, &mut ident, source_code, current_level,
                );
                //----------------------------------------------------------------------------------
            }
            //--------------------------------------------------------------------------------------
        }

        ident
    }

    fn process(&self, node: &Node, source_code: &Vec<u8>, level: usize) -> Vec<u8> {
        node.children(&mut node.walk())
            .map(|child| match child.kind() {
                "{" => {
                    //------------------------------------------------------------------------------
                    let mut ident = IDENT_STR.repeat(level);

                    if child.start_position().column != 0 {
                        ident.clear();
                    }

                    format!("{}{{{}", ident, LINE_BREAK_STR).as_bytes().to_vec()
                    //------------------------------------------------------------------------------
                }
                "}" => format!("{}}}", IDENT_STR.repeat(level)).as_bytes().to_vec(),
                "," => format!(",{}", LINE_BREAK_STR).as_bytes().to_vec(),
                _ => self.handle_node(&child, source_code, level)
            })
            .flat_map(|token| token.to_owned())
            .collect()
    }
}

impl Fixer for IndentBracketBodyFixer {
    fn query(&self) -> &str {
        "(class_declaration body: (declaration_list) @brackets)"
    }

    fn fix(&mut self, node: &Node, source_code: &Vec<u8>) -> Option<Edit> {
        Some(
            Edit {
                deleted_length: node.end_byte() - node.start_byte(),
                position: node.start_byte(),
                inserted_text: self.process(&node, source_code, 0),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::fixer::FixerTestRunner;
    use crate::fixers::indent_bracket_body_fixer::IndentBracketBodyFixer;

    pub fn assert_inputs(input: &'static str, output: &'static str) {
        let mut runner = FixerTestRunner::new(input, output);
        runner.with_fixer(Box::new(IndentBracketBodyFixer {}));
        runner.assert();
    }

    #[test]
    fn it_does_nothing_if_already_indented() {
        let input = indoc! {"
        <?php
        class Test {
            use SomeTrait;
            function sample()
            {
                return 1;
            }
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            use SomeTrait;
            function sample()
            {
                return 1;
            }
        }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_idents_if_not_indented() {
        let input = indoc! {"
        <?php
        class Test {
        use SomeTrait;
        function sample()
        {
        }
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            use SomeTrait;
            function sample()
            {
            }
        }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_removes_idents_when_it_is_over_indented() {
        let input = indoc! {"
        <?php
        class Test {
                function sample1()
                        {
                        function sample2()
                                    {
                                                  }
                  }
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            function sample1()
            {
                function sample2()
                {
                }
            }
        }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_adds_idents_when_it_is_under_indented() {
        let input = indoc! {"
        <?php
        class Test {
                       function sample1()
            {
                                 function sample2()
                 {
                    }
                      }
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            function sample1()
            {
                function sample2()
                {
                }
            }
        }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_adds_idents_when_it_is_under_indented_by_1() {
        let input = indoc! {"
        <?php
        class Test {
            function sample()
           {
            }
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            function sample()
            {
            }
        }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_fix_idents_when_it_is_crazy_indented() {
        let input = indoc! {"
        <?php
        class Test {
                use SomeTrait;
                        function sample()
                   {
        function sample2()
               {
                                                function sample3()
                   {
                                                                  }
                                                    }
            }
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            use SomeTrait;
            function sample()
            {
                function sample2()
                {
                    function sample3()
                    {
                    }
                }
            }
        }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_idents_even_if_everything_is_inlined_in_a_single_line() {
        let input = indoc! {"
        <?php
        class Test { use SomeTrait; function sample() {} }
        "};

        let output = indoc! {"
        <?php
        class Test {
            use SomeTrait;
            function sample() {
            }
        }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_can_ident_chaotic_indentations() {
        let input = indoc! {"
        <?php
        class Test { use SomeTraitA;
        use SomeTraitB;
                function sampleA() {}
        function sampleB() {}  function sampleC() {}
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            use SomeTraitA;
            use SomeTraitB;
            function sampleA() {
            }
            function sampleB() {
            }
            function sampleC() {
            }
        }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_can_ident_anonymous_function() {
        let input = indoc! {"
        <?php
        class Test
        {
        function sample()
        {
        function () {
        return function () {
        return 3;
        };
        };
        }
        }
        "};

        let output = indoc! {"
        <?php
        class Test
        {
            function sample()
            {
                function () {
                    return function () {
                        return 3;
                    };
                };
            }
        }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_can_ident_anonymous_function_when_assigned_to_variables() {
        let input = indoc! {"
        <?php
        class Test
        {
        function sample()
        {
        $test = function () {
        $test = 1;
        };
        }
        }
        "};

        let output = indoc! {"
        <?php
        class Test
        {
            function sample()
            {
                $test = function () {
                    $test = 1;
                };
            }
        }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_can_ident_for_if() {
        let input = indoc! {"
        <?php
        class Test {
        function sample() {
        for (;;) {
        if ($i % 2 === 0) {
        $sample = 1;
        }}}}
        "};

        let output = indoc! {"
        <?php
        class Test {
            function sample() {
                for (;;) {
                    if ($i % 2 === 0) {
                        $sample = 1;
                    }
                }
            }
        }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_can_ident_match_block() {
        let input = indoc! {"
        <?php
        class Test
        {
        function sample()
        {
        match (true) {
        true => 1,
        false => match (false) {
        true => 2,
        false => 3,
        },
        };
        }
        }
        "};

        let output = indoc! {"
        <?php
        class Test
        {
            function sample()
            {
                match (true) {
                    true => 1,
                    false => match (false) {
                        true => 2,
                        false => 3,
                    },
                };
            }
        }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_can_ident_nested_functions() {
        let input = indoc! {"
        <?php
        class Test {
        function sampleA()
        {
        $a = 1;
        function sampleB()
        {
        $b = 2;
        function sampleC()
        {
        $c = 3;
        }
        }
        }
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            function sampleA()
            {
                $a = 1;
                function sampleB()
                {
                    $b = 2;
                    function sampleC()
                    {
                        $c = 3;
                    }
                }
            }
        }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_does_not_destroy_lambda_functions() {
        let input = indoc! {"
        <?php
        class Test {
        function sample() {
        $example = fn () => true;
        }
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            function sample() {
                $example = fn () => true;
            }
        }
        "};

        assert_inputs(input, output);
    }
}
