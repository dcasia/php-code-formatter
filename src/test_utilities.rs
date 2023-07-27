use tree_sitter::{InputEdit, Node, Parser, Point, Tree};

use crate::tree_sitter_php;
use crate::fixer::Fixer;

#[derive(Debug)]
pub struct Edit {
    pub position: usize,
    pub deleted_length: usize,
    pub inserted_text: Vec<u8>,
}

pub fn debug_node(node: &Node, source_code: &str) {
    println!("Start Position: {:?}", node.start_position());
    println!("End Position: {:?}", node.end_position());
    println!("{}", source_code.get(node.start_byte()..node.end_byte()).unwrap_or(""));
}

pub fn is_multiline(node: &Node) -> bool {
    node.start_position().row != node.end_position().row
}

pub fn run_fixer(mut source_code: Vec<u8>, mut fixer: impl Fixer) -> Vec<u8> {
    let mut parser = Parser::new();
    let language = unsafe { tree_sitter_php() };

    parser.set_language(language).unwrap();

    let mut tree = parser.parse(&source_code, None).unwrap();

    fixer.execute(tree, &mut parser, &mut source_code, &language);

    source_code
}

pub fn perform_edit(tree: &mut Tree, input: &mut Vec<u8>, edit: &Edit) -> InputEdit {
    let start_byte = edit.position;
    let old_end_byte = edit.position + edit.deleted_length;
    let new_end_byte = edit.position + edit.inserted_text.len();
    let start_position = position_for_offset(input, start_byte);
    let old_end_position = position_for_offset(input, old_end_byte);

    input.splice(start_byte..old_end_byte, edit.inserted_text.iter().cloned());
    let new_end_position = position_for_offset(input, new_end_byte);

    let edit = InputEdit {
        start_byte,
        old_end_byte,
        new_end_byte,
        start_position,
        old_end_position,
        new_end_position,
    };

    tree.edit(&edit);

    edit
}

fn position_for_offset(input: &Vec<u8>, offset: usize) -> Point {
    let mut result = Point { row: 0, column: 0 };
    for c in &input[0..offset] {
        if *c as char == '\n' {
            result.row += 1;
            result.column = 0;
        } else {
            result.column += 1;
        }
    }
    result
}
