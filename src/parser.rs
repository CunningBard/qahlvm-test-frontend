use pest::iterators::Pair;
use pest::Parser;
use qahlvm::ast::Node;
use crate::eval_parser::eval_from_rule;

#[derive(Parser)]
#[grammar = "front_end.pest"]
struct BareParser {}


pub fn single_parse(pair: Pair<Rule>) -> Node {
    match pair.as_rule() {
        Rule::variable_assignment => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let value = eval_from_rule(inner.next().unwrap());
            Node::Assign(name, value)
        }
        Rule::function_call => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let mut args = vec![];
            for arg in inner {
                args.push(eval_from_rule(arg));
            }
            Node::FnCall(name, args)
        }
        Rule::while_loop => {
            let mut inner = pair.into_inner();
            let condition = eval_from_rule(inner.next().unwrap());
            let mut body = vec![];
            let block = inner.next().unwrap();
            for item in block.into_inner() {
                body.push(single_parse(item));
            }
            Node::WhileLoop(condition, body)
        }
        _ => unreachable!("{:?}", pair.as_rule())
    }
}


pub fn parse_data(data: &str) -> Vec<Node>{
    let res = BareParser::parse(Rule::program, data).unwrap_or_else(|e| panic!("{}", e));
    let mut nodes = vec![];
    for part in res {
        if let Rule::EOI = part.as_rule() { break }

        nodes.push(single_parse(part));
    }

    nodes
}