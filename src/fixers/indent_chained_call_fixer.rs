use std::ops::Sub;

use anyhow::Context;
use tree_sitter::{Node, Point};

use crate::constants::{IDENT, IDENT_STR, LINE_BREAK, LINE_BREAK_STR};
use crate::fixer::Fixer;
use crate::test_utilities::Edit;

pub struct IndentChainedCallFixer {}

#[derive(Debug)]
struct MemberExpression {
    start_byte: usize,
    end_byte: usize,
    start_position: Point,
    end_position: Point,
}

impl MemberExpression {
    fn new(node: &Node) -> Self {
        Self {
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            start_position: node.start_position(),
            end_position: node.end_position(),
        }
    }
}

impl IndentChainedCallFixer {
    fn handle_node(&self, child: &Node, source_code: &Vec<u8>, level: usize) -> Vec<u8> {
        let mut tokens = source_code[child.byte_range()].to_vec();

        let is_multiline = child.start_position().row != child.end_position().row;

        // ensure everything is multiline
        if is_multiline {}

        for child in child.named_children(&mut child.walk()) {
            if child.kind() == "member_call_expression" {
                // self.handle_node(&child, source_code, level);
            }
        }

        println!("{:?}", String::from_utf8(tokens.clone()).unwrap());

        // let current_level = level + 1;
        //
        // let mut ident = IDENT.repeat(current_level).to_vec();
        // ident.append(&mut tokens);
        //
        // if let Some(_) = child.next_sibling().filter(|node| node.kind() != ",") {
        //     ident.extend_from_slice(LINE_BREAK);
        // }

        tokens
    }

    fn count_chain(&self, node: &Node) -> usize {
        node.children(&mut node.walk())
            .fold(1, |count, child| match child.kind() {
                "member_call_expression" => self.count_chain(&child) + 1,
                _ => count
            })
    }

    fn get_expressions(&self, node: &Node) -> Vec<MemberExpression> {
        node.children(&mut node.walk())
            .fold(vec![MemberExpression::new(&node)], |count, child| match child.kind() {
                "member_call_expression" => {
                    let mut response = self.get_expressions(&child);
                    response.push(MemberExpression::new(&child));
                    response
                },
                _ => count
            })
    }

    fn process(&self, node: &Node, source_code: &Vec<u8>, is_root: bool, member_count: usize, child_id: usize) -> Vec<u8> {
        if member_count < 3 {
            return node.children(&mut node.walk())
                .map(|child| match child.kind() {
                    "member_call_expression" => self.process(&child, source_code, false, member_count, child_id),
                    _ => source_code[child.byte_range()].to_vec()
                })
                .flat_map(|token| token.to_owned())
                .collect()
        }

        // println!("ROOT: {:?}", node.utf8_text(source_code).unwrap());

        // for child in node.children(&mut node.walk()) {
        //     println!("CHILD: {:?}", child.utf8_text(source_code).unwrap());
        // }

        let start = node.start_position().column;
        let current_level = (IDENT.len() % start).checked_sub(1).unwrap_or(0);

        let mut response: Vec<u8> = node.children(&mut node.walk())
            .map(|child| match child.kind() {
                "->" => {
                    let mut ident = LINE_BREAK.as_slice().to_vec();

                    ident.append(&mut IDENT.repeat(current_level).to_vec());
                    ident.extend_from_slice(&source_code[child.byte_range()]);

                    let start = child.prev_sibling().unwrap().start_byte();
                    let root_start = node.start_byte();

                    if child.next_named_sibling().is_none() {
                        ident.extend_from_slice(LINE_BREAK);
                    }

                    ident
                }
                "member_call_expression" => self.process(&child, source_code, false, member_count, child_id - 1),
                _ => {
                    let mut tokens = source_code[child.byte_range()].to_vec();

                    if is_root {
                        // println!("PREVI {:?}", node.parent())
                        // tokens.extend_from_slice(b";");
                    }


                    tokens
                }
            })
            .flat_map(|token| token.to_owned())
            .collect();

        if child_id == 1 {
            let mut current = node.to_owned();
            let parent = loop {
                if let Some(parent) = current.parent() {
                    if current.kind() != node.kind() {
                        break current;
                    }

                    current = parent;
                }
            };

            if parent.kind() == "argument" {

                let mut ident = LINE_BREAK.as_slice().to_vec();

                ident.extend_from_slice(&IDENT.repeat(4).to_vec());
                ident.extend_from_slice(&response);

                return ident;

            }

        }

        response
    }
}

impl Fixer for IndentChainedCallFixer {
    fn query(&self) -> &str {
        "
        (expression_statement (member_call_expression) @chain)
        (argument (member_call_expression) @chain)
        "
    }

    fn fix(&mut self, node: &Node, source_code: &Vec<u8>) -> Option<Edit> {
        let member_count = self.count_chain(&node);
        let inserted_text = self.process(&node, source_code, true, member_count, member_count);

        Some(
            Edit {
                deleted_length: node.end_byte() - node.start_byte(),
                position: node.start_byte(),
                inserted_text: inserted_text,
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::fixer::FixerTestRunner;
    use crate::fixers::indent_bracket_body_fixer::IndentBracketBodyFixer;
    use crate::fixers::indent_chained_call_fixer::IndentChainedCallFixer;

    pub fn assert_inputs(input: &'static str, output: &'static str) {
        let mut runner = FixerTestRunner::new(input, output);
        runner.with_fixer(Box::new(IndentBracketBodyFixer {}));
        runner.with_fixer(Box::new(IndentChainedCallFixer {}));
        runner.assert();
    }

    #[test]
    fn it_does_nothing_if_there_are_less_than_3_members() {
        let input = indoc! {"
        <?php
        class Test {
        function sample() {
        static::string()->a()->b();
        }
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            function sample() {
                static::string()->a()->b();
            }
        }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_does_retracts_if_less_than_3_and_multiline() {
        let input = indoc! {"
        <?php
        class Test {
        function sample() {
        static::string()
        ->a()
        ->b();
        }
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            function sample() {
                static::string()->a()->b();
            }
        }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_breaks_chain_when_there_are_more_than_3_members_in_it() {
        let input = indoc! {"
        <?php
        class Test {
        function sample() {
        static::string()->a()->b()->c()->d()->e();
        }
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            function sample() {
                static::string()
                    ->a()
                    ->b()
                    ->c()
                    ->d()
                    ->e();
            }
        }
        "};

        assert_inputs(input, output);
    }
}
