use std::collections::VecDeque;
use pest::iterators::Pair;
use qahlvm::ast::{Eval};
use crate::parser::Rule;


enum StackIntermediate {
    Value(Eval),
    Operator(String),
    Operation(String, Box<StackIntermediate>, Box<StackIntermediate>),
}

impl StackIntermediate {
    fn from_eval(val: Eval) -> Self {
        match val {
            Eval::Int(_)
            | Eval::Bool(_)
            | Eval::Float(_)
            | Eval::Array(_)
            | Eval::String(_)
            | Eval::Object(_)
            | Eval::VarRef(_)
            | Eval::GetMember(_, _)
            | Eval::FnCall(_, _) => Self::Value(val),

            Eval::Add(lhs, rhs) => { Self::Operation("+".to_string(), Box::new(Self::from_eval(*lhs)), Box::new(Self::from_eval(*rhs))) }
            Eval::Sub(lhs, rhs) => { Self::Operation("-".to_string(), Box::new(Self::from_eval(*lhs)), Box::new(Self::from_eval(*rhs))) }
            Eval::Mul(lhs, rhs) => { Self::Operation("*".to_string(), Box::new(Self::from_eval(*lhs)), Box::new(Self::from_eval(*rhs))) }
            Eval::Div(lhs, rhs) => { Self::Operation("/".to_string(), Box::new(Self::from_eval(*lhs)), Box::new(Self::from_eval(*rhs))) }
            Eval::Mod(lhs, rhs) => { Self::Operation("%".to_string(), Box::new(Self::from_eval(*lhs)), Box::new(Self::from_eval(*rhs))) }
            Eval::Pow(lhs, rhs) => { Self::Operation("^".to_string(), Box::new(Self::from_eval(*lhs)), Box::new(Self::from_eval(*rhs))) }
            Eval::Eq(lhs, rhs) => { Self::Operation("==".to_string(), Box::new(Self::from_eval(*lhs)), Box::new(Self::from_eval(*rhs))) }
            Eval::Ne(lhs, rhs) => { Self::Operation("!=".to_string(), Box::new(Self::from_eval(*lhs)), Box::new(Self::from_eval(*rhs))) }
            Eval::Gt(lhs, rhs) => { Self::Operation(">".to_string(), Box::new(Self::from_eval(*lhs)), Box::new(Self::from_eval(*rhs))) }
            Eval::Ge(lhs, rhs) => { Self::Operation(">=".to_string(), Box::new(Self::from_eval(*lhs)), Box::new(Self::from_eval(*rhs))) }
            Eval::Lt(lhs, rhs) => { Self::Operation("<".to_string(), Box::new(Self::from_eval(*lhs)), Box::new(Self::from_eval(*rhs))) }
            Eval::Le(lhs, rhs) => { Self::Operation("<=".to_string(), Box::new(Self::from_eval(*lhs)), Box::new(Self::from_eval(*rhs))) }
            Eval::And(lhs, rhs) => { Self::Operation("&&".to_string(), Box::new(Self::from_eval(*lhs)), Box::new(Self::from_eval(*rhs))) }
            Eval::Or(lhs, rhs) => { Self::Operation("||".to_string(), Box::new(Self::from_eval(*lhs)), Box::new(Self::from_eval(*rhs))) }
            Eval::Not(val) => { Self::Operation("!".to_string(), Box::new(Self::from_eval(*val)), Box::new(Self::from_eval(Eval::Bool(true)))) }
        }
    }
    fn to_eval(self) -> Eval {
        match self {
            StackIntermediate::Value(val) => {
                val
            }
            StackIntermediate::Operator(op) => {
                unreachable!("Operator left on stack: {}", op)
            }
            StackIntermediate::Operation(op, lhs, rhs) => {
                let lhs = Box::new(lhs.to_eval());
                let rhs = Box::new(rhs.to_eval());
                match &*op {
                    "+" => { Eval::Add(lhs, rhs) }
                    "-" => { Eval::Sub(lhs, rhs) }
                    "*" => { Eval::Mul(lhs, rhs) }
                    "/" => { Eval::Div(lhs, rhs) }
                    "%" => { Eval::Mod(lhs, rhs) }
                    "^" => { Eval::Pow(lhs, rhs) }
                    "==" => { Eval::Eq(lhs, rhs) }
                    "!=" => { Eval::Ne(lhs, rhs) }
                    ">" => { Eval::Gt(lhs, rhs) }
                    ">=" => { Eval::Ge(lhs, rhs) }
                    "<" => { Eval::Lt(lhs, rhs) }
                    "<=" => { Eval::Le(lhs, rhs) }
                    "&&" => { Eval::And(lhs, rhs) }
                    "||" => { Eval::Or(lhs, rhs) }
                    "!" => { Eval::Not(lhs) }
                    _ => { unreachable!("Unknown operator: {}", op) }
                }
            }
        }
    }

    fn vec_deque_as_eval(mut vd: VecDeque<Self>) -> Eval {
        let mut left = vd.pop_front().expect("err no items");
        loop {
            let op = match vd.pop_front().expect("err no items"){
                StackIntermediate::Operator(op) => { op },
                _ => unreachable!()
            };
            let right = vd.pop_front().expect("err no items");
            left = StackIntermediate::Operation(
                op,
                Box::new(left),
                Box::new(right)
            );
            if vd.len() == 0 { break }
        }
        left.to_eval()
    }
}


pub fn eval_from_rule(rule: Pair<Rule>) -> Eval {
    match rule.as_rule() {
        Rule::identifier => { Eval::VarRef(rule.as_span().as_str().to_string()) }
        Rule::string => {
            let val_str = rule.as_span().as_str().to_string();
            Eval::String(val_str[1..val_str.len()-1].to_string())
        }
        Rule::float => { Eval::Float(rule.as_str().parse().unwrap()) }
        Rule::integer => { Eval::Int(rule.as_str().parse().unwrap()) }
        Rule::boolean => { Eval::Bool(rule.as_str().parse().unwrap()) }
        Rule::function_call => {
            let mut inner = rule.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let mut args = vec![];
            for arg in inner {
                args.push(eval_from_rule(arg));
            }
            Eval::FnCall(name, args)
        }
        Rule::list => {
            let mut items = vec![];
            for item in rule.into_inner() {
                items.push(eval_from_rule(item));
            }
            Eval::Array(items)
        }
        Rule::expr => {
            eval_from_rule(rule.into_inner().next().unwrap())
        }
        Rule::bare_expr => {
            let mut pairs = rule.into_inner();
            if pairs.len() == 1 {
                return eval_from_rule(pairs.next().unwrap());
            }

            let mut items = VecDeque::new();
            for pair in pairs {
                match pair.as_rule() {
                    Rule::eq_ops => {
                        items.push_back(StackIntermediate::Operator(pair.as_str().to_string()))
                    }
                    _ => {
                        let res = eval_from_rule(pair);
                        items.push_back(StackIntermediate::Value(res));
                    }
                }
            }
            StackIntermediate::vec_deque_as_eval(items)
        }
        Rule::sum => {
            let mut pairs = rule.into_inner();
            if pairs.len() == 1 {
                return eval_from_rule(pairs.next().unwrap());
            }

            let mut items = VecDeque::new();
            for pair in pairs {
                match pair.as_rule() {
                    Rule::sum_ops => {
                        items.push_back(StackIntermediate::Operator(pair.as_str().to_string()))
                    }
                    _ => {
                        let res = eval_from_rule(pair);
                        items.push_back(StackIntermediate::Value(res));
                    }
                }
            }
            StackIntermediate::vec_deque_as_eval(items)
        }
        Rule::product => {
            let mut pairs = rule.into_inner();
            if pairs.len() == 1 {
                return eval_from_rule(pairs.next().unwrap());
            }

            let mut items = VecDeque::new();
            for pair in pairs {
                match pair.as_rule() {
                    Rule::prod_ops => {
                        items.push_back(StackIntermediate::Operator(pair.as_str().to_string()))
                    }
                    _ => {
                        let res = eval_from_rule(pair);
                        items.push_back(StackIntermediate::Value(res));
                    }
                }
            }
            StackIntermediate::vec_deque_as_eval(items)
        }
        Rule::term => {
            eval_from_rule(rule.into_inner().next().unwrap())
        }
        _ => { unreachable!() }
    }
}