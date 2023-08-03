use tree_sitter::Node;
use crate::constants::LINE_BREAK;

use crate::fixer::Fixer;
use crate::test_utilities::Edit;

enum Sequence {
    Parent,
    Next,
    NextNamed,
    NextIsNoneParent,
    PreviousIsNoneParent,
    Previous,
    PreviousNamed
}

pub struct NormalizerFixer {}

impl NormalizerFixer {
    fn get_node_sequence<'a>(&self, node: &'a Node, sequence: &[Sequence]) -> Option<Node<'a>> {
        sequence.iter().fold(Some(*node), |node, sequence| {
            if let Some(node) = node {
                match sequence {
                    Sequence::Parent => node.parent(),
                    Sequence::Next => node.next_sibling(),
                    Sequence::NextNamed => node.next_named_sibling(),
                    Sequence::Previous => node.prev_sibling(),
                    Sequence::PreviousNamed => node.prev_named_sibling(),
                    Sequence::NextIsNoneParent => match node.next_sibling() {
                        None => node.parent(),
                        Some(_) => None,
                    },
                    Sequence::PreviousIsNoneParent => match node.prev_sibling() {
                        None => node.parent(),
                        Some(_) => None,
                    },
                }
            } else {
                None
            }
        })
    }

    fn next_is(&self, node: &Node, kind: &str) -> bool {
        if let Some(next) = node.next_sibling() {
            if next.kind() == kind {
                return true;
            }
        }

        false
    }

    fn previous_is(&self, node: &Node, kind: &str) -> bool {
        if let Some(next) = node.prev_sibling() {
            if next.kind() == kind {
                return true;
            }
        }

        false
    }

    fn is_within(&self, node: &Node, kinds: &[&str]) -> bool {
        kinds.contains(&node.kind())
    }

    fn next_is_within(&self, node: &Node, kinds: &[&str]) -> bool {
        if let Some(next) = node.next_sibling() {
            if kinds.contains(&next.kind()) {
                return true;
            }
        }

        false
    }

    fn previous_is_within(&self, node: &Node, kinds: &[&str]) -> bool {
        if let Some(next) = node.prev_sibling() {
            if kinds.contains(&next.kind()) {
                return true;
            }
        }

        false
    }
}

impl NormalizerFixer {
    fn line_break_before_and_after(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        let mut line_break = LINE_BREAK.to_vec();
        line_break.extend_from_slice(&source_code[node.byte_range()]);
        line_break.extend_from_slice(LINE_BREAK);

        line_break
    }

    fn line_break_before(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        let mut line_break = LINE_BREAK.to_vec();
        line_break.extend_from_slice(&source_code[node.byte_range()]);

        line_break
    }

    fn space_before_and_after(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        let mut tokens = b" ".to_vec();
        tokens.extend_from_slice(&source_code[node.byte_range()]);
        tokens.extend_from_slice(b" ");

        tokens
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

        tokens.extend_from_slice(b" ");

        tokens
    }

    fn pass_through(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        source_code[node.byte_range()].to_vec()
    }
}

impl NormalizerFixer {
    fn handle_return(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(next) = node.next_sibling() {
            if next.kind() == ";" {
                return self.pass_through(&node, &source_code);
            }
        }

        self.space_after(&node, &source_code)
    }

    fn handle_semicolon(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(next) = node.next_sibling() {
            if next.kind() == ")" {
                return self.pass_through(&node, &source_code);
            }
        }

        self.line_break_after(&node, &source_code)
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
            // if previous.kind() == "argument" && node.parent().unwrap().kind() == "formal_parameters" {
            //     return self.pass_through(&node, &source_code);
            // }

            return match previous.kind() {
                "," | "(" => self.pass_through(&node, &source_code),
                _ => self.line_break_before(&node, &source_code),
            };
        }

        self.line_break_before(&node, &source_code)
    }

    fn handle_close_squiggly_bracket(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(parent) = node.parent() {
            if let Some(parent) = parent.parent() {
                if let Some(next) = parent.next_sibling() {
                    if next.kind() != ";" {
                        return self.line_break_after(&node, &source_code);
                    }
                }

                if parent.kind() == "anonymous_function_creation_expression" {
                    return self.pass_through(&node, &source_code);
                }
            }

            if parent.kind() == "declaration_list" {
                return self.pass_through(&node, &source_code);
            }
        }

        match node.next_sibling() {
            None => self.line_break_after(&node, &source_code),
            Some(_) => self.pass_through(&node, &source_code)
        }
    }

    fn handle_open_array_bracket(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(next) = node.next_sibling() {
            if next.kind() == "]" {
                return self.pass_through(&node, &source_code);
            }
        }

        if let Some(previous) = node.prev_sibling() {
            if ["variable_name", "member_access_expression"].contains(&previous.kind()) {
                return self.space_after(&node, &source_code);
            }
        }

        self.line_break_after(&node, &source_code)
    }

    fn handle_close_array_bracket(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(previous) = node.prev_sibling() {
            if ["[", ","].contains(&previous.kind()) {
                return self.pass_through(&node, &source_code);
            }

            return match previous.kind() {
                "integer" |
                "string" |
                "variable_name" |
                "encapsed_string" |
                "binary_expression" |
                "member_access_expression" => self.space_before(&node, &source_code),
                _ => self.line_break_before(&node, &source_code),
            }
        }

        self.line_break_before(&node, &source_code)
    }

    fn handle_static_modifier(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(parent) = node.parent() {
            if self.next_is_within(&parent, &["property_element", "union_type"]) {
                 return self.space_after(&node, &source_code);
            }

            if let Some(previous) = parent.prev_sibling() {
                if previous.kind() == "visibility_modifier" {
                    return self.space_before(&node, &source_code);
                }
            }
        }

        self.pass_through(&node, &source_code)
    }

    fn handle_function(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if self.previous_is_within(node, &["static_modifier", "visibility_modifier"]) {
            return self.space_before_and_after(&node, &source_code);
        }

        if self.next_is(node, "name") {
            return self.space_after(&node, &source_code);
        }

        self.pass_through(&node, &source_code)
    }

    fn handle_primitive_parameters(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(parent) = node.parent() {
            if parent.kind() == "primitive_type" {
                if parent.next_sibling().is_none() {
                    // If it is at the tail of the function, we do nothing
                    if let Some(parent) = parent.parent() {
                        if self.next_is(&parent, "compound_statement") {
                            return self.pass_through(&node, &source_code);
                        }
                    }

                    return self.space_after(&node, &source_code);
                }
            }
        }

        self.pass_through(&node, &source_code)
    }

    fn handle_dollar_kind(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        self.pass_through(&node, &source_code)
    }

    fn handle_visibility_modifier(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(parent) = node.parent() {
            if self.next_is_within(&parent, &["property_element", "readonly_modifier", "union_type", "static_modifier"]) {
                return self.space_after(&node, &source_code);
            }

            if let Some(previous) = parent.prev_sibling() {
                if previous.kind() == "as" {
                    return self.space_after(&node, &source_code);
                }
            }
        }

        self.pass_through(&node, &source_code)
    }

    fn handle_use(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(parent) = node.parent() {
            if let Some(previous) = parent.prev_sibling() {
                if previous.kind() == "formal_parameters" {
                    return self.space_before_and_after(&node, &source_code);
                }
            }
        }

        self.space_after(&node, &source_code)
    }

    fn handle_name_kind(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if let Some(parent) = node.parent() {
            if self.next_is_within(&parent, &["|", "::", "use_list"]) {
                return self.pass_through(&node, &source_code);
            }

            if parent.kind() == "qualified_name" {
                let sequence = self.get_node_sequence(&parent, &[
                    Sequence::Parent,
                    Sequence::Next,
                ]);

                if let Some(parent) = sequence {
                    if self.is_within(&parent, &[";", "|"]) {
                        return self.pass_through(&node, &source_code);
                    }
                }

                if self.next_is(&parent, ";") {
                    return self.pass_through(&node, &source_code);
                }

                return self.space_after(&node, &source_code);
            }

            if parent.kind() == "named_type" {
                let sequence = self.get_node_sequence(&parent, &[
                    Sequence::Parent,
                    Sequence::NextIsNoneParent,
                    Sequence::Next,
                ]);

                if let Some(parent) = sequence {
                    if parent.kind() == "compound_statement" {
                        return self.pass_through(&node, &source_code);
                    }
                }

                return self.space_after(&node, &source_code);
            }
        }

        self.pass_through(&node, &source_code)
    }

    fn handle_operators(&self, node: &Node, source_code: &Vec<u8>) -> Vec<u8> {
        if node.kind() == ":" {
            if let Some(previous) = node.prev_sibling() {
                if ["name", "formal_parameters", "?"].contains(&previous.kind()) {
                    return self.space_after(&node, &source_code);
                }
            }
        }

        if ["+", "-"].contains(&node.kind()) {
            if let Some(parent) = node.parent() {
                if parent.kind() == "unary_op_expression" {
                    return self.pass_through(&node, &source_code);
                }
            }
        }

        if node.kind() == "?" {
            if let Some(next) = node.next_sibling() {
                if next.kind() == "named_type" {
                    return self.pass_through(&node, &source_code);
                }

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

                // println!("{:?} {:?}", child.kind(), child.utf8_text(&source_code).unwrap());

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
                    "as" |
                    "=>" |
                    "extends" |
                    "implements" => self.space_before_and_after(&child, &source_code),
                    "class" => self.handle_class_kind(&child, &source_code),
                    "$" => self.handle_dollar_kind(&child, &source_code),

                    "null" | "string" | "bool" | "boolean" | "float" | "int" |
                    "array" | "mixed" | "object" | "callable" | "resource"
                    => self.handle_primitive_parameters(&child, &source_code),

                    "private" | "public" | "protected" => self.handle_visibility_modifier(&child, &source_code),

                    "readonly" | "final" |
                    "const" | "echo" |
                    "namespace" | "interface" | "trait" |
                    "new" => self.space_after(&child, &source_code),
                    "use" => self.handle_use(&child, &source_code),

                    "name" => self.handle_name_kind(&child, &source_code),
                    "return" => self.handle_return(&child, &source_code),
                    ";" => self.handle_semicolon(&child, &source_code),
                    "," => self.line_break_after(&child, &source_code),
                    "function" => self.handle_function(&child, &source_code),
                    "static" => self.handle_static_modifier(&child, &source_code),
                    "->" | "?->" => self.line_break_before(&child, &source_code),

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
            $this->access [  'a'  ]
        "};

        let output = indoc! {"
            <?php
            $variable[ $a ] = [];
            $string[ 'a' ] = [];
            $escapedString[ \"a\" ] = [];
            $expression[ 1 + 2 ] = [];
            $this
            ->access[ 'a' ]
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn chained_calls_are_broken_into_new_lines() {
        let input = indoc! {"
            <?php
            $a->b($b->a()?->b())->c();
            $a->b($b->a->b)?->c;
        "};

        let output = indoc! {"
            <?php
            $a
            ->b(
            $b
            ->a()
            ?->b()
            )
            ->c();
            $a
            ->b(
            $b
            ->a
            ->b
            )
            ?->c;
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
                static  function    e() {}
                public static  function    f()  {  }
                private    static  function    g()  { }
                protected    static  function    h()    {}
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
            static function e()
            {
            }
            public static function f()
            {
            }
            private static function g()
            {
            }
            protected static function h()
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
            function()
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

    #[test]
    fn optional_return_type_is_correctly_normalized() {
        let input = indoc! {"
            <?php
            class Test
            {
                public function a()   : ?  A  { }
                public function b():?B{ }
                public function c(): ?C { }
            }
        "};

        let output = indoc! {"
            <?php
            class Test
            {
            public function a(): ?A
            {
            }
            public function b(): ?B
            {
            }
            public function c(): ?C
            {
            }
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn union_return_types_are_correctly_normalized() {
        let input = indoc! {"
            <?php
            class Test
            {
                public function a(): A    |B|  string |null|bool   { }
            }
        "};

        let output = indoc! {"
            <?php
            class Test
            {
            public function a(): A | B | string | null | bool
            {
            }
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn closing_parenthesis_does_not_add_unnecessary_new_lines() {
        let input = indoc! {"
            <?php
            wrapper($class->icon(function () {}));
        "};

        let output = indoc! {"
            <?php
            wrapper(
            $class
            ->icon(
            function()
            {
            }
            )
            );
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn closing_brackets_does_not_add_unnecessary_new_lines() {
        let input = indoc! {"
            <?php
            class Test {
            public function name() {}
            }
        "};

        let output = indoc! {"
            <?php
            class Test
            {
            public function name()
            {
            }
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn named_params_are_correctly_normalized() {
        let input = indoc! {"
            <?php
            function test(A $a, B|C $b, ?D $c = null) {}
        "};

        let output = indoc! {"
            <?php
            function test(
            A $a,
            B | C $b,
            ?D $c = null
            )
            {
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn union_primitive_arguments_are_correctly_normalized() {
        let input = indoc! {"
            <?php
            function test(
               null    $a,
            bool $b,
                int  $c,
              float  $d,
               string $e  ,
            array $f ,
              object  $g,
               callable $h,
             resource       $i,
              mixed       $h,
            ) {}
        "};

        let output = indoc! {"
            <?php
            function test(
            null $a,
            bool $b,
            int $c,
            float $d,
            string $e,
            array $f,
            object $g,
            callable $h,
            resource $i,
            mixed $h,
            )
            {
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn named_arguments_are_correctly_normalized() {
        let input = indoc! {"
            <?php
            test(  a:1, b   :  2, c: 3);
        "};

        let output = indoc! {"
            <?php
            test(
            a: 1,
            b: 2,
            c: 3
            );
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn new_keyword_is_spaced() {
        let input = indoc! {"
            <?php
            $object=new      MyObject();
            $class  =  new    class()   {    };
        "};

        let output = indoc! {"
            <?php
            $object = new MyObject();
            $class = new class ()
            {
            };
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn for_loops() {
        let input = indoc! {"
            <?php
            for($i   = 0 ; $i  <  10;   $i++ ) {
                    $test=1;
                            for (;;){}
            }
        "};

        let output = indoc! {"
            <?php
            for(
            $i = 0;
            $i < 10;
            $i++
            )
            {
            $test = 1;
            for(
            ;
            ;
            )
            {
            }
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn negative_and_floating_numbers() {
        let input = indoc! {"
            <?php
            $a=-1-+1;
            $b=-1.0+-1.-+1;
        "};

        let output = indoc! {"
            <?php
            $a = -1 - +1;
            $b = -1.0 + -1. - +1;
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn while_loops() {
        let input = indoc! {"
            <?php
            while  (true) {
                do{
                     $x++;
                }  while ( true );
            }
        "};

        let output = indoc! {"
            <?php
            while(
            true
            )
            {
            do
            {
            $x++;
            }
            while(
            true
            );
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn foreach() {
        let input = indoc! {"
            <?php
            foreach  ($a as $b=>$c) {}
        "};

        let output = indoc! {"
            <?php
            foreach(
            $a as $b => $c
            )
            {
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn qualified_namespace_named_parameter() {
        let input = indoc! {"
            <?php
            function(App\\Service\\A|\\Closure $a){};
        "};

        let output = indoc! {"
            <?php
            function(
            App\\Service\\A | \\Closure $a
            )
            {
            };
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn namespace_imports_and_alias() {
        let input = indoc! {"
            <?php
                namespace      App\\Service ;
               use  App\\Service\\A as B   ;
             use App\\Service\\C   ;
        "};

        let output = indoc! {"
            <?php
            namespace App\\Service;
            use App\\Service\\A as B;
            use App\\Service\\C;
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn traits() {
        let input = indoc! {"
            <?php
            class Test
            {
                use  A ;
                use A\\B ;
                use A\\C {
                    A\\C::test   as  test2;
                    test  as  test;
                    C::test  as  test;
                    \\App\\C::test   as  test;
                    \\App\\C::test   as private  test;
                    \\App\\C::test   as public  test;
                    \\App\\C::test   as protected  test;
                }
            }
        "};

        let output = indoc! {"
            <?php
            class Test
            {
            use A;
            use A\\B;
            use A\\C
            {
            A\\C::test as test2;
            test as test;
            C::test as test;
            \\App\\C::test as test;
            \\App\\C::test as private test;
            \\App\\C::test as public test;
            \\App\\C::test as protected test;
            }
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn consecutive_elements_are_spaced_after_brackets() {
        let input = indoc! {"
            <?php
            class A {}
            class B {}
            $a = new A();
            $b = new B();
            class C {}
        "};

        let output = indoc! {"
            <?php
            class A
            {
            }
            class B
            {
            }
            $a = new A();
            $b = new B();
            class C
            {
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn use_on_function() {
        let input = indoc! {"
            <?php
            function() use ($test){ };
        "};

        let output = indoc! {"
            <?php
            function() use (
            $test
            )
            {
            };
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn const_variables() {
        let input = indoc! {"
            <?php
            const   SAMPLE   =  1;
            class Test {   const    SAMPLE = 1; }
        "};

        let output = indoc! {"
            <?php
            const SAMPLE = 1;
            class Test
            {
            const SAMPLE = 1;
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn final_echo_readonly() {
        let input = indoc! {"
            <?php
               final   readonly  class  Test
            {
                public   readonly  string  $title;
                final   const  X  =  \"foo\";
                final   public  function  test()
                    {
                    echo        \"test\";
                }
            }
        "};

        let output = indoc! {"
            <?php
            final readonly class Test
            {
            public readonly string $title;
            final const X = \"foo\";
            final public function test()
            {
            echo \"test\";
            }
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn class_properties() {
        let input = indoc! {"
            <?php
            final class FooDecorator
            {
                private$foo;
                private  string  $foo;
                public  int  $foo;
                protected  bool| string  $foo=  1;
                private A | B $foo;
                private A\\B | A | null  $foo;
            }
        "};

        let output = indoc! {"
            <?php
            final class FooDecorator
            {
            private $foo;
            private string $foo;
            public int $foo;
            protected bool | string $foo = 1;
            private A | B $foo;
            private A\\B | A | null $foo;
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn interface_trait() {
        let input = indoc! {"
            <?php
            interface Foo {}
            trait Foo {}
        "};

        let output = indoc! {"
            <?php
            interface Foo
            {
            }
            trait Foo
            {
            }
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn static_properties() {
        let input = indoc! {"
            <?php
            class Foo
            {
                protected  static $var  =  1;
                static  $var  =  1;
                static  string  $var =  1;
                static  A $var  = 1;
                public  static  A\\B  $var  = 1;
                private  static  A\\B\\C  $var  = 1;
            }
        "};

        let output = indoc! {"
            <?php
            class Foo
            {
            protected static $var = 1;
            static $var = 1;
            static string $var = 1;
            static A $var = 1;
            public static A\\B $var = 1;
            private static A\\B\\C $var = 1;
            }
        "};

        assert_inputs(input, output);
    }
}
