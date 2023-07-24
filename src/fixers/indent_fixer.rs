use tree_sitter::{InputEdit, Node, Point, Tree};

use crate::Fixer;
use crate::test_utilities::{is_multiline, debug_node, Edit};

pub struct IdentFixer {}

impl Fixer for IdentFixer {
    fn query(&self) -> &str {
        "(use_declaration) @use"
    }

    fn fix(&mut self, node: &Node, source_code: &mut Vec<u8>, tree: &Tree) -> Option<Edit> {
        let start_position = node.start_position();

        // it is already indented
        if start_position.column == 4 {
            return None;
        }

        if node.start_position().column == 0 {
            let mut tokens = source_code[node.byte_range()].to_vec();
            let mut ident = b"    ".to_vec();

            ident.append(&mut tokens);

            return Some(
                Edit {
                    deleted_length: node.end_byte() - node.start_byte(),
                    position: node.start_byte(),
                    inserted_text: ident,
                }
            )
        }

        let mut tokens = source_code[node.byte_range()].to_vec();
        let previous_token = node.prev_sibling().unwrap();

        // If the token starts on the same row as the previous token
        if previous_token.end_position().row == node.start_position().row {
            return None;
        }

        // pop the semicolon so there are difference between the previous token and the current one
        tokens.pop();

        let newlines = 1;
        let semicolon = 1;

        Some(
            Edit {
                deleted_length: node.end_byte() - previous_token.end_byte() - newlines - semicolon,
                position: previous_token.end_byte() + newlines,
                inserted_text: tokens,
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
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            use SomeTrait;
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
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            use SomeTrait;
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
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            use SomeTrait;
        }
        "};

        assert_inputs(input, output);
    }

     #[test]
    fn it_ignores_ident_if_defined_on_the_same_line_as_previous_token() {
        let input = indoc! {"
        <?php
        class Test { use SomeTrait; }
        "};

        let output = indoc! {"
        <?php
        class Test { use SomeTrait; }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_can_ident_chaotic_indentations() {
        let input = indoc! {"
        <?php
        class Test {
                use SomeTraitA;
        use SomeTraitB;
                            use SomeTraitC;
            use SomeTraitD;
        }
        "};

        let output = indoc! {"
        <?php
        class Test {
            use SomeTraitA;
            use SomeTraitB;
            use SomeTraitC;
            use SomeTraitD;
        }
        "};

        assert_inputs(input, output);
    }
}
