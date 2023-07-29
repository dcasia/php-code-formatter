use std::ops::Sub;

use anyhow::Context;
use tree_sitter::Node;

use crate::constants::{INDENT, INDENT_SIZE, INDENT_STR, LINE_BREAK, LINE_BREAK_STR};
use crate::fixer::Fixer;
use crate::test_utilities::Edit;

pub struct IndentBracketBodyFixer {}

impl IndentBracketBodyFixer {
    fn indent_compound_statement_node(
        &self,
        node: &Node,
        parent: &Node,
        current_indent: &mut Vec<u8>,
        source_code: &Vec<u8>,
        level: usize,
    )
    {
        let indent_size = INDENT.len();
        let line_break_size = LINE_BREAK.len();
        let node_start_byte = node.start_byte();
        let mut nesting = 0;

        if let Some(parent) = node.parent() {
            if parent.kind() == "function_call_expression" {
                nesting += 1;
            }
        }

        let mut inner_edit = self.process(&node, source_code, level, nesting);
        let mut indent_level = indent_size * level;
        let mut sub_indent_by = 0;

        if let Some(previous_node) = node.prev_sibling() {
            //--------------------------------------------------------------------------------------
            let previous_node_end_byte = previous_node.end_byte();
            let difference = node_start_byte - previous_node_end_byte;
            let is_over_indented = node_start_byte > previous_node_end_byte + indent_level;

            if is_over_indented == true {
                sub_indent_by = difference - indent_level - line_break_size;
            }

            if is_over_indented == false {
                //----------------------------------------------------------------------------------
                let repeat_by = (indent_level + line_break_size)
                    .checked_sub(difference)
                    .unwrap_or(0);

                let mut indent = b" ".repeat(repeat_by % indent_level);

                indent.append(&mut inner_edit);

                inner_edit = indent;
                //----------------------------------------------------------------------------------
            }
            //--------------------------------------------------------------------------------------
        }

        let start_offset = node_start_byte - parent.start_byte() + indent_level - sub_indent_by;
        let end_offset = start_offset + node.byte_range().count() - line_break_size + sub_indent_by;

        current_indent.splice(start_offset..=end_offset, inner_edit);
    }

    fn handle_switch_block<'a>(&self, node: Node<'a>) -> Option<Vec<Node<'a>>> {
        // todo
        // maybe is better to crash the software to teach a lesson to the users who uses switch statement
        None
    }

    fn handle_call_expression<'a>(&self, node: Node<'a>) -> Option<Vec<Node<'a>>> {
        if let Some(child) = node.child_by_field_name("arguments") {
            let collection: Vec<Node> = child.named_children(&mut child.walk())
                .flat_map(|child| child.named_children(&mut child.walk()).collect::<Vec<_>>())
                .filter(|child| child.kind() == "anonymous_function_creation_expression")
                .filter_map(|child| self.handle_default(child))
                .flatten()
                .collect();

            if collection.is_empty() == false {
                return Some(collection)
            }
        }

        None
    }

    fn handle_assigment_expression<'a>(&self, node: Node<'a>) -> Option<Vec<Node<'a>>> {
        node.child_by_field_name("right")
            .filter(|node| node.kind() == "anonymous_function_creation_expression")
            .map(|node| node.child_by_field_name("body"))
            .unwrap_or(None)
            .map(|node| vec![node])
    }

    fn handle_default<'a>(&self, node: Node<'a>) -> Option<Vec<Node<'a>>> {
        node.child_by_field_name("body")
            .filter(|node| match node.kind() {
                "compound_statement" => true,
                "match_block" => true,
                _ => false
            })
            .map(|node| vec![node])
    }

    fn handle_node(&self, child: &Node, source_code: &Vec<u8>, level: usize) -> Vec<u8> {
        let mut tokens = source_code[child.byte_range()].to_vec();
        let current_level = level + 1;

        let mut indent = INDENT.repeat(current_level).to_vec();
        indent.append(&mut tokens);

        if let Some(_) = child.next_sibling().filter(|node| node.kind() != ",") {
            indent.extend_from_slice(LINE_BREAK);
        }

        for inner_child in child.children(&mut child.walk()) {
            //--------------------------------------------------------------------------------------
            let node: Option<Vec<Node>> = match inner_child.kind() {
                "compound_statement" => Some(vec![inner_child]),
                "switch_block" => self.handle_switch_block(inner_child),
                "assignment_expression" => self.handle_assigment_expression(inner_child),
                "member_call_expression" |
                "function_call_expression" => self.handle_call_expression(inner_child),
                _ => self.handle_default(inner_child),
            };

            if let Some(inner_children) = node {
                //----------------------------------------------------------------------------------
                for inner_child in inner_children {
                    //------------------------------------------------------------------------------
                    self.indent_compound_statement_node(
                        &inner_child, &child, &mut indent, source_code, current_level
                    );
                    //------------------------------------------------------------------------------
                }
                //----------------------------------------------------------------------------------
            }
            //--------------------------------------------------------------------------------------
        }

        indent
    }

    fn process(&self, node: &Node, source_code: &Vec<u8>, level: usize, nesting: usize) -> Vec<u8> {
        node.children(&mut node.walk())
            .map(|child| match child.kind() {
                "{" => {
                    //------------------------------------------------------------------------------
                    let mut indent = INDENT_STR.repeat(level);

                    if child.start_position().column != 0 {
                        indent.clear();
                    }

                    format!("{}{{{}", indent, LINE_BREAK_STR).as_bytes().to_vec()
                    //------------------------------------------------------------------------------
                }
                "}" => format!("{}}}", INDENT_STR.repeat(level + nesting)).as_bytes().to_vec(),
                "," => format!(",{}", LINE_BREAK_STR).as_bytes().to_vec(),
                _ => self.handle_node(&child, source_code, level + nesting)
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
                inserted_text: self.process(&node, source_code, 0, 0),
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
    fn it_indents_if_not_indented() {
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
    fn it_removes_indents_when_it_is_over_indented() {
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
    fn it_adds_indents_when_it_is_under_indented() {
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
    fn it_adds_indents_when_it_is_under_indented_by_1() {
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
    fn it_fix_indents_when_it_is_crazy_indented() {
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
    fn it_indents_even_if_everything_is_inlined_in_a_single_line() {
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
    fn it_can_indent_chaotic_indentations() {
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
    fn it_can_indent_anonymous_function() {
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
    fn it_can_indent_anonymous_function_when_assigned_to_variables() {
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
    fn it_can_indent_for_if() {
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
    fn it_can_indent_match_block() {
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
    fn it_can_indent_nested_functions() {
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

    #[test]
    fn it_can_indent_global_function_call_argument() {
        let input = indoc! {"
        <?php
        class Test {
        function sample() {
        then(function () {
        $example = 1;
        });
        }
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            function sample() {
                then(function () {
                    $example = 1;
                });
            }
        }
        "};

        assert_inputs(input, output);
    }
}
