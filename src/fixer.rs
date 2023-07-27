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

        let mut tree = parser.parse(&source_code, None)
            .ok_or(anyhow::Error::msg("Failed to parse source code."))?;

        for fixer in &mut self.fixers {
            tree = fixer.execute(tree, &mut parser, source_code, &language)?;
        }

        Ok(tree)
    }
}

pub struct FixerTestRunner {
    fixers: Vec<Box<dyn Fixer>>,
    input: Vec<u8>,
    output: Vec<u8>,
}

impl FixerTestRunner {
    pub fn new(input: &'static str, output: &'static str) -> Self {
        Self {
            fixers: vec![],
            input: input.as_bytes().to_vec(),
            output: output.as_bytes().to_vec(),
        }
    }

    pub fn with_fixer(&mut self, fixer: Box<dyn Fixer>) {
        self.fixers.push(fixer);
    }

    pub fn assert(mut self) {
        let mut runner = FixerRunner {
            fixers: self.fixers
        };

        runner.execute(&mut self.input).expect("Failed to execute fixers.");

        let left = String::from_utf8(self.input).expect("Failed to convert input to string.");
        let right = String::from_utf8(self.output).expect("Failed to convert output to string.");

        assert_eq!(left, right);
    }
}