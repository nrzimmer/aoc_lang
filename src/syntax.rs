#![allow(dead_code)]
use crate::lexer::Rule;
use pest::iterators::{Pair, Pairs};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Syntax<'a> {
    content: Pair<'a, Rule>,
    pub(crate) externs: HashMap<String, ExternFunction>,
    pub(crate) functions: HashMap<String, Function>,
    variables: VarTree,
    pub(crate) strings: Vec<String>,
    next_vartree: usize,
}

impl<'a> Syntax<'a> {
    pub fn new(content: Pair<'a, Rule>) -> Self {
        let externs = HashMap::new();
        let functions = HashMap::new();
        let strings = Vec::new();
        let variables = VarTree {
            father: None,
            variables: Vec::new(),
            children: HashMap::new(),
        };
        Self {
            content,
            externs,
            functions,
            variables,
            strings,
            next_vartree: 0,
        }
    }

    fn gen_id(&mut self) -> usize {
        let id = self.next_vartree;
        self.next_vartree += 1;
        id
    }

    pub fn analyze(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let parsed = self.content.clone().into_inner();
        for pair in parsed {
            self.parse(pair)?;
        }
        Ok(())
    }

    fn parse(&mut self, pair: Pair<Rule>) -> Result<(), Box<dyn std::error::Error>> {
        let rule = pair.as_rule();
        match rule {
            Rule::extern_function => self.parse_extern_function(pair),
            Rule::function => self.parse_function(pair),
            Rule::declaration => {
                todo!()
            }
            Rule::EOI => Ok(()),
            _ => {
                todo!("{:?}", pair)
            }
        }
    }

    fn parse_function(&mut self, pair: Pair<Rule>) -> Result<(), Box<dyn std::error::Error>> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_span().as_str().to_string();
        let mut parameters = HashMap::new();
        if let Some(params) = Syntax::expect(&inner, Rule::parameter_list) {
            inner.next();
            for param in params.into_inner() {
                let mut param = param.into_inner();
                let var_type = param.next().unwrap();
                let name = param.next().unwrap();
                let param_type = VarType::from_str(var_type.as_span().as_str());
                let name = name.as_span().as_str().to_string();
                parameters.insert(name, param_type);
            }
        }
        let mut return_type = VarType::Void;
        if let Some(rt) = Syntax::expect(&inner, Rule::return_type) {
            inner.next();
            let rt = rt.into_inner().next().unwrap();
            return_type = VarType::from_str(rt.as_span().as_str());
        }
        let id = self.gen_id();
        let mut function = Function {
            name: name.clone(),
            id,
            parameters,
            return_type,
            code: Block {
                id,
                statements: Vec::new(),
            },
        };

        if let Some(block) = Syntax::expect(&inner, Rule::block) {
            inner.next();
            for pair in block.into_inner() {
                self.parse_statement(pair, &mut function.code)?;
            }
        } else {
            return Err(format!("No code block for function: {}", name).into());
        }

        self.functions.insert(name, function);
        Ok(())
    }

    fn parse_statement(&mut self, pair: Pair<Rule>, code: &mut Block) -> Result<(), Box<dyn std::error::Error>> {
        let rule = pair.as_rule();
        if rule != Rule::statement {
            panic!("Expected statement, got: {:?}", rule)
        }
        let pair = pair.into_inner().next().unwrap();
        let rule = pair.as_rule();
        match rule {
            Rule::function_call => {
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_span().as_str().to_string();
                let mut arguments = Vec::new();
                if let Some(args) = Syntax::expect(&inner, Rule::argument_list) {
                    inner.next();
                    let args = args.into_inner();
                    for arg in args {
                        let arg = arg.into_inner().next().unwrap();
                        let rule = arg.as_rule();
                        match rule {
                            Rule::literal => {
                                let arg = arg.into_inner().next().unwrap();
                                let var_type = VarType::from_rule(&arg.as_rule());
                                let value = arg.as_span().as_str().to_string();
                                let id: Option<usize>; // = None;
                                match var_type {
                                    VarType::Int => {
                                        todo!()
                                    }
                                    VarType::Char => {
                                        todo!()
                                    }
                                    VarType::String => {
                                        id = Some(self.strings.len());
                                        self.strings.push(value.clone());
                                    }
                                    VarType::Bool => {
                                        todo!()
                                    }
                                    VarType::Void => {
                                        todo!()
                                    }
                                    VarType::VarArgs => {
                                        todo!()
                                    }
                                }
                                let argument = Parameter {
                                    name: "".to_string(),
                                    value,
                                    id,
                                    var_type,
                                    is_literal: true,
                                };
                                arguments.push(argument);
                            }
                            Rule::identifier => {
                                todo!("{:?}", arg);
                                // let name = arg.as_span().as_str().to_string();
                                //
                                // let argument = Parameter {
                                //     name,
                                //     value: "".to_string(),
                                //     id: None,
                                //     var_type: VarType::VarArgs,
                                //     is_literal: false,
                                // };
                            }
                            _ => {
                                panic!("Unknown argument: {:?}", arg);
                            }
                        }
                    }
                }
                let fn_call = FnCall {
                    name: name.clone(),
                    parameters: arguments,
                };
                if self.externs.contains_key(&name) {
                    code.statements.push(Statement::ExternFunctionCall(fn_call));
                    return Ok(());
                }
                if self.functions.contains_key(&name) {
                    code.statements.push(Statement::FunctionCall(fn_call));
                    return Ok(());
                }
                Err(format!("Unknown function: {}", name).into())
            }
            Rule::return_statement => {
                todo!()
            }
            _ => {
                todo!("{:?}", pair)
            }
        }
    }

    fn parse_extern_function(&mut self, pair: Pair<Rule>) -> Result<(), Box<dyn std::error::Error>> {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_span().as_str().to_string();
        let mut parameters = Vec::new();
        if let Some(params) = Syntax::expect(&inner, Rule::extern_parameter_list) {
            inner.next();
            for param in params.into_inner() {
                let param_type = param.as_span().as_str().to_string();
                parameters.push(VarType::from_str(&param_type));
            }
        }
        let mut return_type = VarType::Void;
        if let Some(rt) = Syntax::expect(&inner, Rule::return_type) {
            inner.next();
            let rt = rt.into_inner().next().unwrap();
            return_type = VarType::from_str(rt.as_span().as_str());
        }
        let function = ExternFunction {
            name: name.clone(),
            parameters,
            return_type,
        };
        self.externs.insert(name, function);

        Ok(())
    }

    fn expect(rules: &Pairs<'a, Rule>, rule: Rule) -> Option<Pair<'a, Rule>> {
        if let Some(next) = rules.peek() {
            if next.as_rule() == rule {
                return Some(next);
            }
        }
        None
    }

    pub fn optimize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut remove = Vec::new();
        for func in self.functions.values() {
            if func.code.statements.is_empty() {
                remove.push(func.name.clone());
            }
        }

        for func in remove {
            self.functions.remove(&func);
        }

        Ok(())
    }
}

#[derive(Debug)]
struct VarTree {
    father: Option<usize>,
    variables: Vec<Variable>,
    children: HashMap<usize, VarTree>,
}

#[derive(Debug)]
pub struct Variable {
    name: String,
    var_type: VarType,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    name: String,
    value: String,
    pub(crate) id: Option<usize>,
    pub(crate) var_type: VarType,
    is_literal: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarType {
    Int,
    Char,
    String,
    Bool,
    Void,
    VarArgs,
}

impl VarType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "int" => VarType::Int,
            "char" => VarType::Char,
            "string" => VarType::String,
            "bool" => VarType::Bool,
            "void" => VarType::Void,
            "..." => VarType::VarArgs,
            _ => panic!("Unknown type: {}", s),
        }
    }

    pub fn from_rule(r: &Rule) -> Self {
        match r {
            Rule::string => VarType::String,
            Rule::char => VarType::Char,
            Rule::integer => VarType::Int,
            _ => panic!("Unknown type: {:?}", r),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub(crate) name: String,
    id: usize,
    parameters: HashMap<String, VarType>,
    return_type: VarType,
    pub(crate) code: Block,
}

#[derive(Debug, Clone)]
pub struct Block {
    id: usize,
    pub(crate) statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct ExternFunction {
    name: String,
    pub(crate) parameters: Vec<VarType>,
    return_type: VarType,
}

#[derive(Debug, Clone)]
pub struct FnCall {
    pub(crate) name: String,
    pub(crate) parameters: Vec<Parameter>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Block(Block),
    FunctionCall(FnCall),
    ExternFunctionCall(FnCall),
    Return(VarType),
}
