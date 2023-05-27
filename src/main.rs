extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::fs;

use qahlvm::vm::*;

mod parser;
mod eval_parser;

fn main() {
    let program = fs::read_to_string("test.txt").unwrap();
    println!("--------------------- Source Code ---------------------");
    println!("{}", program);

    let res = parser::parse_data(&*program);

    println!("--------------------- AST ---------------------");
    for node in &res {
        println!("{:?}", node);
    }

    println!("--------------------- VM Output ---------------------");

    let mut vm = VirtualMachine::new(GcApproach::ReferenceCounting);
    vm.run(res);
}