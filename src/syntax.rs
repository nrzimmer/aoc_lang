use crate::lexer::Rule;
use pest::iterators::{Pair, Pairs};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Syntax {
    externs: HashMap<String, ExternFunction>,
    functions: HashMap<String, Function>,
    variables: VarTree,
    strings: Vec<String>,
    next_vartree: usize,
}

impl Syntax {
    pub fn new() -> Self {
        let externs = HashMap::new();
        let functions = HashMap::new();
        let strings = Vec::new();
        let variables = VarTree {
            father: None,
            variables: Vec::new(),
            children: HashMap::new(),
        };
        Self {
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

    pub fn analize(&mut self, parsed: Pairs<Rule>) {
        for pair in parsed {
            self.parse(pair);
        }
    }

    pub fn assemble(&mut self, parsed: Pairs<Rule>) {
        if !self.functions.contains_key("main") {
            panic!("No main function");
        }

        println!(".text");
        println!(".section	.rodata");
        println!(".align 8");
        for id in 0..self.strings.len() {
            let str = &self.strings[id];
            println!(".STR{}:", id);
            println!("  .string {}", str);
        }
        println!();

        let mut main = self.functions.remove("main").unwrap();
        println!(".text\n.globl main");
        self.asm_function(&mut main);

        // Collect all function names first to avoid the borrow checker error
        let function_names: Vec<String> = self.functions.keys().cloned().collect();
        println!();

        // Process each function by name
        for name in function_names {
            let f = self.functions.get(&name).unwrap().clone();
            self.asm_function(&f);
            println!();
        }
    }

    fn asm_function(&mut self, function: &Function) {
        println!("{}:", function.name);
        let mut add_return = true;
        for stmt in &function.code.statements {
            match stmt {
                Statement::Block(blk) => {
                    todo!()
                }
                Statement::FunctionCall(call) => {
                    self.asm_pass_parameters(call.parameters.clone());
                    println!("  call {}", call.name);
                }
                Statement::ExternFunctionCall(call) => {
                    self.asm_pass_parameters(call.parameters.clone());
                    println!("  xor %eax, %eax");
                    println!("  call {}", call.name);
                }
                Statement::Return(ret) => {
                    add_return = false;
                    todo!()
                }
            }
        }
        if add_return {
            println!("  xor  %eax, %eax\n  ret")
        }
    }

    fn asm_pass_parameters(&self, params: Vec<Parameter>) {
        for idx in (0..params.len()).rev() {
            let param = &params[idx];
            if idx > 5 {
                self.asm_push_parameter(param);
            }
            match idx {
                5 => {
                    println!("  movl $4, %r9d")
                }
                4 => {
                    println!("  movl $3, %r8d")
                }
                3 => {
                    println!("  movl $2, %ecx")
                }
                2 => {
                    println!("  movl $1, %edx")
                }
                1 => {
                    self.asm_pass_param(param, "%esi");
                }
                0 => {
                    self.asm_pass_param(param, "%rdi");
                }
                _ => {
                    panic!("Should not happen!!! parameter index: {}", idx)
                }
            }
        }
    }

    fn asm_pass_param(&self, param: &Parameter, dest: &str) {
        match param.var_type {
            VarType::Int => {
                todo!()
            }
            VarType::Char => {
                todo!()
            }
            VarType::String => {
                println!("  leaq .STR{}(%rip), {}", param.id.unwrap(), dest);
            }
            VarType::Bool => {
                todo!()
            }
            VarType::Void => {
                panic!("Cannot pass void type")
            }
            VarType::VarArgs => {
                panic!("Cannot pass varargs")
            }
        }
    }

    fn asm_push_parameter(&self, param: &Parameter) {
        match param.var_type {
            VarType::Int => {
                todo!()
            }
            VarType::Char => {
                todo!()
            }
            VarType::String => {
                todo!()
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
    }

    fn parse(&mut self, pair: Pair<Rule>) {
        let rule = pair.as_rule();
        let value = pair.as_span().as_str().to_string();
        match rule {
            Rule::extern_function => self.extern_function(pair),
            Rule::function => self.function(pair),
            Rule::declaration => {}
            Rule::EOI => {}
            _ => {
                todo!("{:?}", pair)
            }
        }
    }

    fn function(&mut self, pair: Pair<Rule>) {
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
                self.statement(pair, &mut function.code);
            }
        } else {
            panic!("No code block for function: {}", name)
        }

        self.functions.insert(name, function);
    }

    fn statement(&mut self, pair: Pair<Rule>, code: &mut Block) {
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
                    let mut args = args.into_inner();
                    for arg in args {
                        let arg = arg.into_inner().next().unwrap();
                        let rule = arg.as_rule();
                        match rule {
                            Rule::literal => {
                                let arg = arg.into_inner().next().unwrap();
                                let var_type = VarType::from_rule(&arg.as_rule());
                                let value = arg.as_span().as_str().to_string();
                                let mut id: Option<usize> = None;
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
                                let name = arg.as_span().as_str().to_string();

                                let argument = Parameter {
                                    name,
                                    value: "".to_string(),
                                    id: None,
                                    var_type: VarType::VarArgs,
                                    is_literal: false,
                                };
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
                    return;
                }
                if self.functions.contains_key(&name) {
                    code.statements.push(Statement::FunctionCall(fn_call));
                    return;
                }
                panic!("Unknown function: {}", name);
            }
            Rule::return_statement => {
                let mut inner = pair.into_inner();
                println!("{:?}", inner);
            }
            _ => {
                todo!("{:?}", pair)
            }
        }
    }

    fn extern_function(&mut self, pair: Pair<Rule>) {
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
    }

    fn expect<'a>(rules: &Pairs<'a, Rule>, rule: Rule) -> Option<Pair<'a, Rule>> {
        if let Some(next) = rules.peek() {
            if next.as_rule() == rule {
                return Some(next);
            }
        }
        None
    }

    pub fn optimize(&mut self) {
        let mut remove = Vec::new();
        for func in self.functions.values() {
            if func.code.statements.is_empty() {
                remove.push(func.name.clone());
            }
        }

        for func in remove {
            self.functions.remove(&func);
        }
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
    id: Option<usize>,
    var_type: VarType,
    is_literal: bool,
}

#[derive(Debug, Clone)]
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
    name: String,
    id: usize,
    parameters: HashMap<String, VarType>,
    return_type: VarType,
    code: Block,
}

#[derive(Debug, Clone)]
pub struct Block {
    id: usize,
    statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct ExternFunction {
    name: String,
    parameters: Vec<VarType>,
    return_type: VarType,
}

#[derive(Debug, Clone)]
pub struct FnCall {
    name: String,
    parameters: Vec<Parameter>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Block(Block),
    FunctionCall(FnCall),
    ExternFunctionCall(FnCall),
    Return(VarType),
}
