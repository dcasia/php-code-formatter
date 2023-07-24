use anyhow::Context;
use indoc::indoc;
use tree_sitter::{Node, Tree};

use crate::constants::{IDENT, IDENT_STR, LINE_BREAK_STR};
use crate::Fixer;
use crate::test_utilities::Edit;

pub struct IdentFixer {}

impl IdentFixer {
    fn ident(&self, node: &Node, source_code: &mut Vec<u8>, level: usize) -> Vec<u8> {
        node.children(&mut node.walk())
            .map(|child| match child.kind() {
                "{" => {
                    let mut ident = IDENT_STR.repeat(level);

                    if child.start_position().column != 0 {
                        ident.clear();
                    }

                    format!("{}{{{}", ident, LINE_BREAK_STR).as_bytes().to_vec()
                }
                "}" => format!("{}}}", IDENT_STR.repeat(level)).as_bytes().to_vec(),
                _ => {
                    let mut tokens = source_code[child.byte_range()].to_vec();
                    let current_level = level + 1;

                    let mut ident = IDENT.repeat(current_level).to_vec();
                    ident.append(&mut tokens);
                    ident.extend_from_slice(LINE_BREAK_STR.as_bytes());

                    if let Some(inner_child) = child.child_by_field_name("body") {

                        println!("PARENT_START: {:?}", inner_child.parent().unwrap().utf8_text(source_code).unwrap());
                        println!("CURRENT: {:?}", inner_child.utf8_text(source_code).unwrap());

                        let inner_edit = self.ident(&inner_child, source_code, current_level);

                        let start_offset = inner_child.start_byte() - child.start_byte() + (IDENT.len() * current_level);
                        let end_offset = start_offset + inner_child.byte_range().count() - LINE_BREAK_STR.len();

                        ident.splice(start_offset..=end_offset, inner_edit);

                        // println!("CHILD: {:?}", child.utf8_text(source_code).unwrap());
                        // println!("INNER: {:?}", inner_child.utf8_text(source_code).unwrap());
                        //
                        // println!("OFFSETS: {} {}", start_offset, end_offset);
                    }

                    ident
                }
            })
            .flat_map(|token| token.to_owned())
            .collect()
    }
}

impl Fixer for IdentFixer {
    fn query(&self) -> &str {
        indoc! {"
            (class_declaration body: (declaration_list) @brackets)
        "}
        //(function_definition body: (compound_statement) @function-brackets)
        //(class_declaration body: (declaration_list (method_declaration body: (compound_statement) @brackets)))
    }

    fn fix(&mut self, node: &Node, source_code: &mut Vec<u8>, tree: &Tree) -> Option<Edit> {
        Some(
            Edit {
                deleted_length: node.end_byte() - node.start_byte(),
                position: node.start_byte(),
                inserted_text: self.ident(&node, source_code, 0),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::fixers::indent_fixer::IdentFixer;
    use crate::test_utilities::run_fixer;

    pub fn assert_inputs(input: &str, output: &str) {
        let left = String::from_utf8(run_fixer(input.into(), IdentFixer {})).unwrap();
        let right = output.to_string();

        assert_eq!(left, right);
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
    fn it_removes_idents_if_over_indented() {
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
}
