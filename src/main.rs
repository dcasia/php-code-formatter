#![allow(dead_code)]
#![allow(warnings)]

use std::fs;

use crate::fixer::{Fixer, FixerRunner};
use crate::fixers::indent_bracket_body_fixer::IndentBracketBodyFixer;

mod fixers;
mod test_utilities;
mod constants;
mod fixer;

fn main() -> anyhow::Result<()> {
    let mut runner = FixerRunner::new();

    let fixers: [fn() -> Box<dyn Fixer>; 1] = [
        //|| Box::new(ArrayBracketSpaceFixer {}),
        //|| Box::new(DeclareDirectiveSpaceFixer {}),
        //|| Box::new(DeclareDirectiveExistenceFixer {}),
        //|| Box::new(FunctionArgumentsSpaceFixer {}),
        || Box::new(IndentBracketBodyFixer {}),
        //|| Box::new(HeaderLineFixer {}),
    ];

    fixers.iter().for_each(|fixer| runner.add_fixer(fixer()));

    let mut source_code = fs::read_to_string("src/Sample.php")?.as_bytes().to_vec();

    let tree = runner.execute(&mut source_code)?;

    fs::write("src/Sample2.php", tree.root_node().utf8_text(source_code.as_slice())?)?;

    Ok(())
}
