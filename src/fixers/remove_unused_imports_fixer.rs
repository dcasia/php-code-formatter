use indoc::indoc;
use tree_sitter::{InputEdit, Node, Query, QueryCursor, Tree};

use crate::Fixer;
use crate::test_utilities::Edit;

pub struct RemoveUnusedImportsFixer {}

impl Fixer for RemoveUnusedImportsFixer {
    fn query(&self) -> &str {
        "(namespace_use_declaration) @use"
    }

    fn fix(&mut self, node: &Node, source_code: &Vec<u8>) -> Option<Edit> {
        // Collect all static method calls Class::method()
        todo!();
        // let query = Query::new(node.language(), indoc! {"
        //     (scoped_call_expression scope: (name) @static-methods)
        //     (named_type (name) @function-arguments)
        //     (base_clause (name) @class-extends)
        // "})?;
        //
        // let mut cursor = QueryCursor::new();
        //
        // let captures = cursor.matches(&query, tree.root_node(), source_code.as_bytes());
        //
        // let static_calls: Vec<&str> = captures
        //     .flat_map(|captures| captures.captures)
        //     .map(|capture| capture.node.utf8_text(source_code.as_bytes()).expect("Failed to get text from capture"))
        //     .collect();
        //
        // let sub_query = Query::new(node.language(), "(namespace_use_clause (qualified_name (name) @name))")?;
        // let mut sub_cursor = QueryCursor::new();
        // let mut clone_source_code = source_code.clone();
        // let mut finding = sub_cursor.matches(&sub_query, *node, clone_source_code.as_bytes());
        //
        // if let Some(next_match) = finding.next() {
        //     if let Some(next_capture) = next_match.nodes_for_capture_index(0).next() {
        //         if static_calls.contains(&next_capture.utf8_text(source_code.as_bytes())?) == false {
        //             return Ok((Some("".as_bytes().to_vec()), None));
        //         }
        //     }
        // }
        //
        // Ok((None, None))
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::fixers::remove_unused_imports_fixer::RemoveUnusedImportsFixer;
    use crate::test_utilities::run_fixer;

    pub fn assert_inputs(input: &str, output: &str) {
        assert_eq!(
            run_fixer(input.to_string().into(), RemoveUnusedImportsFixer {}), output.as_bytes().to_vec()
        );
    }

    #[test]
    fn it_is_able_to_detect_static_method_calls() {
        let input = indoc! {"
        <?php
        use App\\One;
        use App\\Two;

        One::example();
        "};

        let output = indoc! {"
        <?php
        use App\\One;


        One::example();
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_is_able_to_detect_function_arguments() {
        let input = indoc! {"
        <?php
        use App\\One;
        use App\\Two;

        function example(One $one) {}
        "};

        let output = indoc! {"
        <?php
        use App\\One;


        function example(One $one) {}
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_is_able_to_detect_union_type_in_function_arguments() {
        let input = indoc! {"
        <?php
        use App\\One;
        use App\\Two;
        use App\\Three;

        function example(One|Two $oneOrTwo) {}
        "};

        let output = indoc! {"
        <?php
        use App\\One;
        use App\\Two;


        function example(One|Two $oneOrTwo) {}
        "};

        assert_inputs(input, output);
    }

    #[test]
    fn it_is_able_to_detect_class_extends() {
        let input = indoc! {"
        <?php
        use App\\One;
        use App\\Two;
        use App\\Three;
        use App\\Four;

        class Example extends One {}
        class Example extends Two, Four {}
        "};

        let output = indoc! {"
        <?php
        use App\\One;
        use App\\Two;

        use App\\Four;

        class Example extends One {}
        class Example extends Two, Four {}
        "};

        assert_inputs(input, output);
    }
}
