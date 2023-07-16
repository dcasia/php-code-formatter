use tree_sitter::Parser;
use crate::{Fixer, tree_sitter_php};

pub fn run_fixer(source_code: String, mut fixer: impl Fixer) -> String {
    let mut parser = Parser::new();
    let language = unsafe { tree_sitter_php() };

    parser.set_language(language).unwrap();

    let mut source_code = source_code.to_string();

    let mut tree = parser.parse(&source_code, None).unwrap();

    fixer.exec(&mut tree, &mut parser, &mut source_code, &language);

    source_code
}
