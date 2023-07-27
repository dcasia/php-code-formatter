use tree_sitter::{Language, Node, Parser, Query, QueryCursor, Tree};

use crate::test_utilities::{Edit, perform_edit};

extern "C" { pub fn tree_sitter_php() -> Language; }

pub trait Fixer {
    fn query(&self) -> &str;

    fn fix(&mut self, node: &Node, source_code: &Vec<u8>) -> Option<Edit>;

    fn execute(&mut self, mut tree: Tree, parser: &mut Parser, source_code: &mut Vec<u8>, language: &Language) -> anyhow::Result<Tree> {
        let mut cursor = QueryCursor::new();
        let query = Query::new(*language, self.query())?;
        let mut index = 0;

        loop {
            let mut nodes: Vec<Node> = cursor
                .matches(&query, tree.root_node(), source_code.as_slice())
                .flat_map(|item| item.captures)
                .map(|capture| capture.node)
                .collect();

            if let Some(node) = nodes.get(index) {
                index += 1;

                if let Some(edit) = self.fix(&node, source_code) {
                    if edit.inserted_text != source_code[node.byte_range()].to_vec() {
                        perform_edit(&mut tree, source_code, &edit);

                        tree = parser.parse(&source_code, Some(&tree)).expect("error re-parsing code.");
                    }
                }
            } else {
                break;
            }
        }

        Ok(tree)
    }
}

pub struct FixerRunner {
    fixers: Vec<Box<dyn Fixer>>,
}

impl FixerRunner {
    pub fn new() -> Self {
        Self { fixers: vec![] }
    }

    pub fn add_fixer(&mut self, fixer: Box<dyn Fixer>) {
        self.fixers.push(fixer);
    }

    pub fn execute(&mut self, source_code: &mut Vec<u8>) -> anyhow::Result<Tree> {
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_php() };

        parser.set_language(language)?;

        let mut tree = parser.parse(&source_code, None).unwrap();

        for fixer in &mut self.fixers {
            tree = fixer.execute(tree, &mut parser, source_code, &language)?;
        }

        Ok(tree)
    }
}

pub struct FixerTestRunner {
    fixers: Vec<Box<dyn Fixer>>,
    input: Option<Vec<u8>>,
    output: Option<Vec<u8>>,
}

impl FixerTestRunner {
    pub fn new() -> Self {
        Self {
            fixers: vec![],
            input: None,
            output: None,
        }
    }

    pub fn with_fixer(&mut self, fixer: Box<dyn Fixer>) -> &mut Self {
        self.fixers.push(fixer);

        self
    }

    pub fn with_input(&mut self, input: &'static str) -> &mut Self {
        self.input = Some(input.as_bytes().to_vec());

        self
    }

    pub fn with_expected_output(&mut self, output: &'static str) -> &mut Self {
        self.output = Some(output.as_bytes().to_vec());

        self
    }

    pub fn assert(self) {
        let mut runner = FixerRunner {
            fixers: self.fixers
        };

        let mut input = self.input.expect("Input is required, please call .with_input() method.");
        let expected_output = self.output.expect("Output is required, please call .with_expected_output() method.");

        runner.execute(&mut input).expect("Failed to execute fixers.");

        let left = String::from_utf8(input).expect("Failed to convert input to string.");
        let right = String::from_utf8(expected_output).expect("Failed to convert output to string.");

        assert_eq!(left, right);
    }
}