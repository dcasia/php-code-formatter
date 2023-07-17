#![allow(dead_code)]
#![allow(warnings)]

use std::fs;

use tree_sitter::{InputEdit, Language, Node, Parser, Point, Query, QueryCursor, Tree};

mod fixers;
mod test_utilities;

extern "C" { fn tree_sitter_php() -> Language; }

const WHITE_SPACE: &str = " ";
const NEW_LINE: &str = "\n\n";

pub trait Fixer {
    fn query(&self) -> &str;

    fn fix(&mut self, node: &Node, source_code: &mut String, tree: &Tree) -> anyhow::Result<String>;

    fn exec(&mut self, mut tree: Tree, parser: &mut Parser, source_code: &mut String, language: &Language) -> anyhow::Result<()> {
        let mut cursor = QueryCursor::new();
        let query = Query::new(*language, self.query())?;

        loop {
            let mut nodes: Vec<Node> = cursor
                .matches(&query, tree.root_node(), source_code.as_bytes())
                .flat_map(|item| item.nodes_for_capture_index(0).next())
                .collect();

            let mut should_break = true;

            for mut node in nodes {
                let tokens = self.fix(&node, source_code, &tree)?;

                if tokens != node.utf8_text(source_code.as_bytes())? {
                    source_code.replace_range(node.byte_range(), &tokens);

                    tree.edit(&InputEdit {
                        start_byte: node.start_byte(),
                        start_position: node.start_position(),
                        old_end_byte: node.end_byte(),
                        old_end_position: node.end_position(),
                        new_end_byte: node.start_byte() + tokens.len(),
                        new_end_position: Point::new(
                            node.start_position().row,
                            node.start_position().column + tokens.len(),
                        ),
                    });

                    tree = parser.parse(&source_code, Some(&tree)).unwrap();

                    should_break = false;

                    break;
                }
            }

            if should_break {
                break;
            }
        }

        Ok(())
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

    let mut source_code = fs::read_to_string("src/Sample.php")?;
    let mut tree = parser.parse(&source_code, None).unwrap();

    // let mut edits = vec![];

    //
    // for mut node in nodes.iter_mut() {
    //     let new_source_code = "declare( strict_types = 1 );".to_string();
    //
    //     let edit = InputEdit {
    //         start_byte: node.start_byte(),
    //         start_position: node.start_position(),
    //         old_end_byte: node.end_byte(),
    //         old_end_position: node.end_position(),
    //         new_end_byte: node.start_byte() + new_source_code.len(),
    //         new_end_position: Point::new(
    //             node.start_position().row,
    //             node.start_position().column + new_source_code.len(),
    //         ),
    //     };
    //
    //     // edits.push((edit, new_source_code));
    // }
    //
    // let mut previous: Option<InputEdit> = None;
    //
    // for (edit, new_code) in edits {
    //     let mut current_edit = edit;
    //
    //     println!("{:?}", current_edit);
    //
    //     if let Some(previous_edit) = previous {
    //         current_edit.start_byte += previous_edit.start_byte - previous_edit.old_end_byte;
    //         current_edit.old_end_byte += previous_edit.new_end_byte - previous_edit.old_end_byte;
    //         current_edit.new_end_byte += previous_edit.old_end_byte - previous_edit.new_end_byte;
    //         current_edit.old_end_position = Point {
    //             row: current_edit.old_end_position.row + previous_edit.old_end_position.row - previous_edit.old_end_position.row,
    //             column: current_edit.old_end_position.row + previous_edit.old_end_position.column - previous_edit.old_end_position.column,
    //         };
    //
    //         current_edit.new_end_position = Point {
    //             row: current_edit.new_end_position.row + previous_edit.new_end_position.row - previous_edit.old_end_position.row,
    //             column: current_edit.new_end_position.column + previous_edit.new_end_position.column - previous_edit.old_end_position.column,
    //         };
    //     }
    //
    //     tree.edit(&current_edit);
    //     source_code.replace_range(current_edit.start_byte..current_edit.old_end_byte, new_code.as_str());
    //     previous = Some(current_edit)
    // }

    // for each_match in matches {
    //     if let Some(node) = each_match.nodes_for_capture_index(0).next() {
    //
    //     }
    // }

    fs::write("src/Sample2.php", tree.root_node().utf8_text(source_code.as_bytes())?)?;

    Ok(())
}

// fn main() -> anyhow::Result<()> {
//     let mut parser = Parser::new();
//     let language = unsafe { tree_sitter_php() };
//
//     parser.set_language(language)?;
//
//     let fixers: [fn() -> Box<dyn Fixer>; 5] = [
//         || Box::new(ArrayBracketSpaceFixer {}),
//         || Box::new(DeclareDirectiveSpaceFixer {}),
//         || Box::new(DeclareDirectiveExistenceFixer {}),
//         || Box::new(RemoveUnusedImportsFixer {}),
//         || Box::new(HeaderLineFixer {}),
//     ];
//
//     let walker = WalkDir::new("/home/ziva-backend/app");
//
//     for entry in walker {
//         let entry = entry.unwrap();
//
//         if entry.file_type().is_dir() {
//             continue;
//         }
//
//         if entry.file_name().to_str().unwrap().ends_with(".php") {
//             let mut source_code = fs::read_to_string(entry.path())?;
//             let mut tree = parser.parse(&source_code, None).unwrap();
//
//             for fixer in fixers {
//                 fixer().exec(&mut tree, &mut parser, &mut source_code, &language)?;
//             }
//
//             fs::write(entry.path(), tree.root_node().utf8_text(source_code.as_bytes())?)?;
//
//             println!("{:?}", entry.path().display());
//         }
//     }
//
//     Ok(())
// }
