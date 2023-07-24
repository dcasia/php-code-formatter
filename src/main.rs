#![allow(dead_code)]
#![allow(warnings)]

use std::fs;

use tree_sitter::{InputEdit, Language, Node, Parser, Query, QueryCursor, Tree};

use crate::fixers::indent_fixer::IdentFixer;
use crate::test_utilities::{Edit, perform_edit};

mod fixers;
mod test_utilities;
mod constants;

extern "C" { fn tree_sitter_php() -> Language; }

const WHITE_SPACE: &str = " ";
const NEW_LINE: u8 = 10; // \n

#[cfg(target_family = "unix")]
pub const LINE_BREAK: &[u8; 1] = b"\n";

#[cfg(target_family = "windows")]
pub const LINE_BREAK: &[u8; 2] = b"\r\n";

pub trait Fixer {
    fn query(&self) -> &str;

    fn fix(&mut self, node: &Node, source_code: &mut Vec<u8>, tree: &Tree) -> Option<Edit>;

    fn exec(&mut self, mut tree: Tree, parser: &mut Parser, source_code: &mut Vec<u8>, language: &Language) -> anyhow::Result<Tree> {
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
                if let Some(edit) = self.fix(&node, source_code, &tree) {
                    if edit.inserted_text != source_code[node.byte_range()].to_vec() {
                        perform_edit(&mut tree, source_code, &edit);

                        tree = parser.parse(&source_code, Some(&tree)).expect("error re-parsing code.");

                        should_break = false;

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

    fn compute_edit(&self, node: &Node, tokens: &str) -> InputEdit {
        todo!()
    }

    fn remove_node(&self, node: &Node, source_code: &mut String) -> anyhow::Result<Option<InputEdit>> {
        source_code.replace_range(node.byte_range(), "");

        Ok(Some(self.compute_edit(node, "")))
    }

    fn build_sequence(&mut self, node: &Node, source_code: &mut String, callback: fn(token: &str) -> Vec<&str>) -> anyhow::Result<Option<InputEdit>> {
        let mut tokens = vec![];

        for child in node.children(&mut node.walk()) {
            if let Some(value) = source_code.get(child.byte_range()) {
                for item in callback(value) {
                    tokens.push(item)
                }
            }
        }

        let tokens = tokens.join("");
        let current_tokens = node.utf8_text(source_code.as_bytes())?;

        if tokens != current_tokens {
            source_code.replace_range(node.byte_range(), &tokens);

            return Ok(Some(self.compute_edit(node, &tokens)));
        }

        Ok(None)
    }
}


fn main() -> anyhow::Result<()> {
    let mut parser = Parser::new();
    let language = unsafe { tree_sitter_php() };

    parser.set_language(language)?;

    let mut source_code = fs::read_to_string("src/Sample.php")?.as_bytes().to_vec();
    let mut tree = parser.parse(&source_code, None).unwrap();

    let fixers: [fn() -> Box<dyn Fixer>; 1] = [
        // || Box::new(ArrayBracketSpaceFixer {}),
        // || Box::new(DeclareDirectiveSpaceFixer {}),
        // || Box::new(DeclareDirectiveExistenceFixer {}),
        // || Box::new(FunctionArgumentsSpaceFixer {}),
        || Box::new(IdentFixer {}),
        // || Box::new(HeaderLineFixer {}),
    ];

    for fixer in fixers {
        tree = fixer().exec(tree, &mut parser, &mut source_code, &language)?;
    }

    fs::write("src/Sample2.php", tree.root_node().utf8_text(source_code.as_slice())?)?;

    Ok(())
}
