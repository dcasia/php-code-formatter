use std::fs;
use tree_sitter::{InputEdit, Language, Node, Parser, Point, Query, QueryCursor, Tree};
use crate::fixers::array_bracket_space_fixer::ArrayBracketSpaceFixer;
use crate::fixers::declare_directive_existence_fixer::DeclareDirectiveExistenceFixer;
use crate::fixers::declare_directive_space_fixer::DeclareDirectiveSpaceFixer;
use crate::fixers::header_line_fixer::HeaderLineFixer;

mod fixers;
mod test_utilities;

extern "C" { fn tree_sitter_php() -> Language; }

const WHITE_SPACE: &str = " ";
const NEW_LINE: &str = "\n\n";

pub trait Fixer {
    fn query(&self) -> &str;

    fn fix(&mut self, node: &Node, source_code: &mut String) -> anyhow::Result<Option<InputEdit>>;

    fn exec(&mut self, tree: &mut Tree, parser: &mut Parser, source_code: &mut String, language: &Language) -> anyhow::Result<()> {
        let mut cursor = QueryCursor::new();
        let query = Query::new(*language, self.query())?;

        loop {
            let string = String::new();
            let mut new_tree = tree.clone();
            let matches = cursor.matches(&query, new_tree.root_node(), string.as_bytes());

            let mut should_break = true;

            'top: for each_match in matches {
                for capture in each_match.captures {
                    if let Some(edit) = self.fix(&capture.node, source_code)? {
                        new_tree.edit(&edit);
                        should_break = false;
                        *tree = parser.parse(source_code.as_bytes(), Some(&new_tree)).unwrap();
                        break 'top;
                    }
                }
            }

            if should_break {
                break;
            }
        }

        Ok(())
    }

    fn compute_edit(&self, node: &Node, tokens: &str) -> InputEdit {
        InputEdit {
            start_byte: node.start_byte(),
            start_position: node.start_position(),
            old_end_byte: node.end_byte(),
            old_end_position: node.end_position(),
            new_end_byte: node.start_byte() + tokens.len(),
            new_end_position: Point::new(
                node.start_position().row,
                node.start_position().column + tokens.len(),
            ),
        }
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

    let mut source_code = fs::read_to_string("src/Sample.php")?;
    let mut tree = parser.parse(&source_code, None).unwrap();

    let fixers: [fn() -> Box<dyn Fixer>; 4] = [
        || Box::new(ArrayBracketSpaceFixer {}),
        || Box::new(DeclareDirectiveSpaceFixer {}),
        || Box::new(DeclareDirectiveExistenceFixer {}),
        || Box::new(HeaderLineFixer {}),
    ];

    for fixer in fixers {
        fixer().exec(&mut tree, &mut parser, &mut source_code, &language)?;
    }

    fs::write("src/Sample2.php", tree.root_node().utf8_text(source_code.as_bytes())?)?;

    Ok(())
}
