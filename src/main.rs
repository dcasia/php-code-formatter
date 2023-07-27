#![allow(dead_code)]
#![allow(warnings)]

use std::fs;

use tree_sitter::{Language, Parser};

use crate::fixer::FixerRunner;
use crate::fixers::indent_bracket_body_fixer::IndentBracketBodyFixer;

mod fixers;
mod test_utilities;
mod constants;
mod fixer;

extern "C" { fn tree_sitter_php() -> Language; }

fn main() -> anyhow::Result<()> {
    let mut parser = Parser::new();
    let language = unsafe { tree_sitter_php() };

    parser.set_language(language)?;

    let mut source_code = fs::read_to_string("src/Sample.php")?.as_bytes().to_vec();
    let tree = parser.parse(&source_code, None).unwrap();
    let mut runner = FixerRunner::new();

    //runner.add_fixer(Box::new(ArrayBracketSpaceFixer {}));
    //runner.add_fixer(Box::new(DeclareDirectiveSpaceFixer {}));
    //runner.add_fixer(Box::new(DeclareDirectiveExistenceFixer {}));
    //runner.add_fixer(Box::new(FunctionArgumentsSpaceFixer {}));
    runner.add_fixer(Box::new(IndentBracketBodyFixer {}));
    //runner.add_fixer(Box::new(HeaderLineFixer {}));

    let tree = runner.execute(tree, &mut parser, &mut source_code, &language)?;

    fs::write("src/Sample2.php", tree.root_node().utf8_text(source_code.as_slice())?)?;

    Ok(())
}
