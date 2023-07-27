use tree_sitter::{Language, Node, Parser, Query, QueryCursor, Tree};
use crate::test_utilities::{Edit, perform_edit};

pub trait Fixer {
    fn query(&self) -> &str;

    fn fix(&mut self, node: &Node, source_code: &Vec<u8>) -> Option<Edit>;

    fn execute(&mut self, mut tree: Tree, parser: &mut Parser, source_code: &mut Vec<u8>, language: &Language) -> anyhow::Result<Tree> {
        let mut cursor = QueryCursor::new();
        let query = Query::new(*language, self.query())?;

        loop {
            let mut nodes: Vec<Node> = cursor
                .matches(&query, tree.root_node(), source_code.as_slice())
                .flat_map(|item| item.captures)
                .map(|capture| capture.node)
                .collect();

            let mut should_break = true;

            for mut node in nodes {
                if let Some(edit) = self.fix(&node, source_code) {
                    if edit.inserted_text != source_code[node.byte_range()].to_vec() {
                        perform_edit(&mut tree, source_code, &edit);

                        tree = parser.parse(&source_code, Some(&tree)).expect("error re-parsing code.");

                        should_break = true;

                        break;
                    }
                }
            }

            if should_break {
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

    pub fn execute(&mut self, tree: Tree, parser: &mut Parser, source_code: &mut Vec<u8>, language: &Language) -> anyhow::Result<Tree> {
        let mut tree = tree;

        for fixer in &mut self.fixers {
            tree = fixer.execute(tree, parser, source_code, language)?;
        }

        Ok(tree)
    }
}
