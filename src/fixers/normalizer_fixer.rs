use tree_sitter::Node;
use crate::constants::LINE_BREAK;

use crate::fixer::Fixer;
use crate::test_utilities::Edit;

pub struct NormalizerFixer {}

impl NormalizerFixer {
    fn line_break_before_and_after(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        let mut line_break = LINE_BREAK.to_vec();
        line_break.extend_from_slice(&source_code[node.byte_range()]);
        line_break.extend_from_slice(LINE_BREAK);

        line_break
    }

    fn space_before_and_after(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        let mut tokens = b" ".to_vec();
        tokens.extend_from_slice(&source_code[node.byte_range()]);
        tokens.extend_from_slice(b" ");

        tokens
    }

    fn line_break_before(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        let mut line_break = LINE_BREAK.to_vec();
        line_break.extend_from_slice(&source_code[node.byte_range()]);

        line_break
    }

    fn line_break_before_and_space_after(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        let mut line_break = LINE_BREAK.to_vec();
        line_break.extend_from_slice(&source_code[node.byte_range()]);
        line_break.extend_from_slice(b" ");

        line_break
    }

    fn line_break_after(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        let mut tokens = source_code[node.byte_range()].to_vec();
        tokens.extend_from_slice(LINE_BREAK);

        tokens
    }

    fn space_before(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        let mut tokens = b" ".to_vec();
        tokens.extend_from_slice(&source_code[node.byte_range()]);

        tokens
    }

    fn space_after(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        let mut tokens = source_code[node.byte_range()].to_vec();

        // Only if the next element is on the same row
        if let Some(next) = node.next_sibling() {
            if next.start_position().row == node.start_position().row {
                tokens.extend_from_slice(b" ");
            }
        }

        tokens
    }

    fn pass_through(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        source_code[node.byte_range()].to_vec()
    }

    fn handle_return(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(next) = node.next_sibling() {
            if next.kind() == ";" {
                return self.pass_through(&node, &source_code);
            }
        }

        self.space_after(&node, &source_code)
    }

    fn handle_class_kind(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(previous) = node.prev_sibling() {
            if previous.kind() == "abstract_modifier" {
                return self.space_before_and_after(&node, &source_code);
            }
        }

        self.space_after(&node, &source_code)
    }

    fn handle_open_parenthesis(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(next) = node.next_sibling() {
            if next.kind() == ")" {
                return self.pass_through(&node, &source_code)
            }
        }

        self.line_break_after(&node, &source_code)
    }

    fn handle_close_parenthesis(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(previous) = node.prev_sibling() {
            if previous.kind() == "(" {
                return self.pass_through(&node, &source_code);
            }
        }

        self.line_break_before(&node, &source_code)
    }

    fn handle_close_squiggly_bracket(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if node.next_sibling().is_none() {
            return self.line_break_after(&node, &source_code)
        }

        self.pass_through(&node, &source_code)
    }

    fn handle_open_array_bracket(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(next) = node.next_sibling() {
            if next.kind() == "]" {
                return self.pass_through(&node, &source_code);
            }
        }

        if let Some(previous) = node.prev_sibling() {
            if previous.kind() == "variable_name" {
                return self.space_after(&node, &source_code);
            }
        }

        self.line_break_after(&node, &source_code)
    }

    fn handle_close_array_bracket(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(previous) = node.prev_sibling() {
            if previous.kind() == "[" {
                return self.pass_through(&node, &source_code);
            }

            return match previous.kind() {
                "integer" |
                "string" |
                "variable_name" |
                "encapsed_string" |
                "binary_expression" |
                "member_access_expression" => self.space_before(&node, &source_code),
                "," => self.pass_through(&node, &source_code),
                _ => self.line_break_before(&node, &source_code),
            }
        }

        self.line_break_before(&node, &source_code)
    }

    fn handle_function(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(previous) = node.prev_sibling() {
            if previous.kind() == "visibility_modifier" {
                return self.space_before_and_after(&node, &source_code);
            }
        }

        self.space_after(&node, &source_code)
    }

    fn handle_operators(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if node.kind() == ":" {
            if let Some(previous) = node.prev_sibling() {
                if previous.kind() == "?" {
                    return self.space_after(&node, &source_code);
                }
            }
        }

        if node.kind() == "?" {
            if let Some(next) = node.next_sibling() {
                if next.kind() == ":" {
                    return self.space_before(&node, &source_code);
                }
            }
        }

        self.space_before_and_after(&node, &source_code)
    }

    fn normalize_block(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        node.children(&mut node.walk())
            .map(|child| {
                if child.child_count() > 0 {
                    return self.normalize_block(&child, &source_code);
                }

                // println!("{}", child.kind());

                match child.kind() {
                    "=" | "+=" | "-=" | "*=" | "/=" | "%=" |                                        // Assignment Operators
                    "==" | "===" | "!=" | "<>" | "!==" | ">" | "<" | ">=" | "<=" | "<=>" |          // Comparison Operators
                    "+" | "-" | "*" | "**" | "/" | "%" |                                            // Arithmetic
                    "and" | "or" | "xor" | "&&" | "||" | "!" |                                      // Logical Operators
                    "." | ".=" |                                                                    // String Operators
                    "?:" | "??" | "?" | ":" |                                                       // Conditional Assignment Operators
                    ">>" | "<<" | "&" | "|" | "^" | ">>=" | "<<=" | "&=" | "|=" | "^="              // Bitwise Operators
                    => self.handle_operators(&child, &source_code),

                    // Class related tokens
                    "=>" |
                    "extends" |
                    "implements" => self.space_before_and_after(&child, &source_code),
                    "class" => self.handle_class_kind(&child, &source_code),

                    "return" => self.handle_return(&child, &source_code),
                    "," | ";" => self.line_break_after(&child, &source_code),
                    "function" => self.handle_function(&child, &source_code),
                    "->" => self.line_break_before(&child, &source_code),

                    // Brackets / Parenthesis
                    "[" => self.handle_open_array_bracket(&child, &source_code),
                    "]" => self.handle_close_array_bracket(&child, &source_code),
                    "{" => self.line_break_before_and_after(&child, &source_code),
                    "}" => self.handle_close_squiggly_bracket(&child, &source_code),
                    "(" => self.handle_open_parenthesis(&child, &source_code),
                    ")" => self.handle_close_parenthesis(&child, &source_code),

                    // Default
                    _ => self.pass_through(&child, &source_code)
                }
            })
            .flat_map(|token| token.to_owned())
            .collect()
    }
}

impl Fixer for NormalizerFixer {
    fn query(&self) -> &str {
        "(program) @program"
    }

    fn fix(&mut self, node: &Node, source_code: &Vec<u8>) -> Option<Edit> {
        let tokens: Vec<u8> = node
            .children(&mut node.walk())
            .map(|child| self.normalize_block(&child, &source_code))
            .flat_map(|token| token.to_owned())
            .collect();

        let mut opening = b"<?php".to_vec();
        opening.extend_from_slice(LINE_BREAK);
        opening.extend_from_slice(&tokens);

        // ensure there is only 1 line break at the end of the file
        while opening.ends_with(LINE_BREAK) {
            opening.pop();
        }

        opening.extend_from_slice(LINE_BREAK);

        Some(
            Edit {
                deleted_length: node.end_byte() - node.start_byte(),
                position: node.start_byte(),
                inserted_text: opening,
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use crate::fixer::FixerTestRunner;
    use crate::fixers::normalizer_fixer::NormalizerFixer;

    pub fn assert_inputs(input: &'static str, output: &'static str) {
        let mut runner = FixerTestRunner::new(input, output);
        runner.with_fixer(Box::new(NormalizerFixer {}));
        runner.assert();
    }

    #[test]
    fn operators_are_left_with_white_space_before_and_after() {
        let input = indoc! {"
            <?php
            $a=1;$a+=1;$a-=1;$a*=1;$a/=1;$a%=1;
            $b==1;$b===1;$b!=1;$b<>1;$b!==1;$b>1;$b<1;$b>=1;$b<=1;$b<=>1;
            $c+1;$c-1;$c*1;$c**1;$c/1;$c%1;
            $d    and $d  or  $d    xor   $d    && $d   ||$d   ! $d  ;
            $e.='1';$e.$e;
            $f?:1;$f??1;
            $g=1?1:2;
            $h++;$h--;++$h;--$h;
            $i>>1;$i<<1;$i&1;$i|1;$i^1;$i>>=1;$i<<=1;$i&=1;$i|=1;$i^=1;
        "};

        let output = indoc! {"
            <?php
            $a = 1;
            $a += 1;
            $a -= 1;
            $a *= 1;
            $a /= 1;
            $a %= 1;
            $b == 1;
            $b === 1;
            $b != 1;
            $b <> 1;
            $b !== 1;
            $b > 1;
            $b < 1;
            $b >= 1;
            $b <= 1;
            $b <=> 1;
            $c + 1;
            $c - 1;
            $c * 1;
            $c ** 1;
            $c / 1;
            $c % 1;
            $d and $d or $d xor $d && $d || $d ! $d;
            $e .= '1';
            $e . $e;
            $f ?: 1;
            $f ?? 1;
            $g = 1 ? 1 : 2;
            $h++;
            $h--;
            ++$h;
            --$h;
            $i >> 1;
            $i << 1;
            $i & 1;
            $i | 1;
            $i ^ 1;
            $i >>= 1;
            $i <<= 1;
            $i &= 1;
            $i |= 1;
            $i ^= 1;
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn function_name_space_is_normalized() {
        let input = indoc! {"
            <?php
            function     name() {}
        "};

        let output = indoc! {"
            <?php
            function name()
            {
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn class_name_space_is_normalized() {
        let input = indoc! {"
            <?php
            class    Test     {}
        "};

        let output = indoc! {"
            <?php
            class Test
            {
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn abstract_class_is_normalized() {
        let input = indoc! {"
            <?php
            abstract      class    Test {}
        "};

        let output = indoc! {"
            <?php
            abstract class Test
            {
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn class_extends_and_implements_space_is_normalized() {
        let input = indoc! {"
            <?php
            class    Test  extends      A    implements     B{}
        "};

        let output = indoc! {"
            <?php
            class Test extends A implements B
            {
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn array_brackets_are_normalized() {
        let input = indoc! {"
            <?php
            $array=[   1 ,2, 3    ];
        "};

        let output = indoc! {"
            <?php
            $array = [
            1,
            2,
            3
            ];
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn array_access_brackets_are_normalized() {
        let input = indoc! {"
            <?php
            $variable [$a] =[];
            $string ['a'] = [];
            $escapedString    [ \"a\"   ] =[];
            $expression  [1+2] =[     ];
        "};

        let output = indoc! {"
            <?php
            $variable[ $a ] = [];
            $string[ 'a' ] = [];
            $escapedString[ \"a\" ] = [];
            $expression[ 1 + 2 ] = [];
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn chained_calls_are_broken_into_new_lines() {
        let input = indoc! {"
            <?php
            $a->b($b->a()->b())->c();
            $a->b($b->a->b)->c;
        "};

        let output = indoc! {"
            <?php
            $a
            ->b(
            $b
            ->a()
            ->b()
            )
            ->c();
            $a
            ->b(
            $b
            ->a
            ->b
            )
            ->c;
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn function_modifier_is_normalized() {
        let input = indoc! {"
            <?php
            class Test {
                function    a() {}
                public   function    a() {}
                private    function    c() {}
                protected    function    d() {}
            }
        "};

        let output = indoc! {"
            <?php
            class Test
            {
            function a()
            {
            }
            public function a()
            {
            }
            private function c()
            {
            }
            protected function d()
            {
            }
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn returns_are_correctly_normalized() {
        let input = indoc! {"
            <?php
            function() {
                return    1;
                return 1;
                return[];
                return;
            }
        "};

        let output = indoc! {"
            <?php
            function ()
            {
            return 1;
            return 1;
            return [];
            return;
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn object_initializer_are_spaced() {
        let input = indoc! {"
            <?php
            $a=[
            1=>1,
                2=>    2,
            ];
            $b=[ 1=>1, 2=>    2];
        "};

        let output = indoc! {"
            <?php
            $a = [
            1 => 1,
            2 => 2,
            ];
            $b = [
            1 => 1,
            2 => 2
            ];
        "};

        assert_inputs(input, output);
    }
}
