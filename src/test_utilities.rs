use tree_sitter::{Node, Parser};
use crate::{Fixer, tree_sitter_php};

pub fn debug_node(node: &Node, source_code: &str) {
    println!("Start Position: {:?}", node.start_position());
    println!("End Position: {:?}", node.end_position());
    println!("{}", source_code.get(node.start_byte()..node.end_byte()).unwrap_or(""));
}

pub fn is_multiline(node: &Node) -> bool {
    node.start_position().row != node.end_position().row
}

pub fn run_fixer(source_code: String, mut fixer: impl Fixer) -> String {
    let mut parser = Parser::new();
    let language = unsafe { tree_sitter_php() };

    parser.set_language(language).unwrap();

    let mut source_code = source_code.to_string();

    let mut tree = parser.parse(&source_code, None).unwrap();

    fixer.exec(tree, &mut parser, &mut source_code, &language);

    source_code
}
