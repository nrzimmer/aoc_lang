#![allow(dead_code)]
use crate::lexer::Rule;
use pest::iterators::{Pair, Pairs};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Syntax<'a> {
    content: Pair<'a, Rule>,
    pub(crate) externs: HashMap<String, ExternFunction>,
    pub(crate) functions: HashMap<String, Function>,
    pub(crate) variables: VarTree,
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
            stack: 0,
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
        let mut vars = VarTree {
            father: Some(0),
            variables: Vec::new(),
            children: HashMap::new(),
            stack: 0,
        };
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
                self.parse_statement(pair, &mut function.code, &mut vars)?;
            }
        } else {
            return Err(format!("No code block for function: {}", name).into());
        }

        self.variables.children.insert(id, vars);
        self.functions.insert(name, function);
        Ok(())
    }

    fn parse_statement(&mut self, pair: Pair<Rule>, code: &mut Block, vars: &mut VarTree) -> Result<(), Box<dyn std::error::Error>> {
        let rule = pair.as_rule();
        if rule != Rule::statement {
            panic!("Expected statement, got: {:?}", rule)
        }
        let pair = pair.into_inner().next().unwrap();
        let rule = pair.as_rule();
        match rule {
            Rule::function_call => self.function_call(pair, code, vars),
            Rule::return_statement => {
                todo!()
            }
            Rule::declaration => self.declaration(pair, code, vars),
            Rule::assignment => self.assignment(pair, code, vars),
            _ => {
                todo!("{:?}", pair)
            }
        }
    }

    fn function_call(&mut self, pair: Pair<Rule>, code: &mut Block, vars: &mut VarTree) -> Result<(), Box<dyn std::error::Error>> {
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
                        let id: Option<usize>;
                        match var_type {
                            VarType::Int => {
                                id = None;
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
                            value: Some(value),
                            id,
                            var_type,
                            is_literal: true,
                        };
                        arguments.push(argument);
                    }
                    Rule::identifier => {
                        let name = arg.as_span().as_str().to_string();
                        let var_info = vars.variables.iter().find(|v| v.name == name.clone());
                        if var_info.is_none() {
                            return Err(format!("Unknown variable: {}", name).into());
                        }

                        let argument = Parameter {
                            name,
                            value: None,
                            id: None,
                            var_type: var_info.unwrap().var_type.clone(),
                            is_literal: false,
                        };
                        arguments.push(argument);
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

    fn declaration(&mut self, pair: Pair<Rule>, code: &mut Block, vars: &mut VarTree) -> Result<(), Box<dyn std::error::Error>> {
        let mut inner = pair.into_inner();
        let decl_type = inner.next().unwrap();
        let decl_name = inner.next().unwrap();
        let var_type = VarType::from_rule(&decl_type.as_rule());
        let name = decl_name.as_span().as_str().to_string();
        let var = Variable {
            name,
            var_type,
            stack: None,
        };
        vars.variables.push(var.clone());
        let assign = inner.peek();
        if assign.is_some() {
            self.declaration_assignment(inner, code, vars, var)?;
        }

        Ok(())
    }

    fn declaration_assignment(
        &mut self,
        mut inner: Pairs<Rule>,
        code: &mut Block,
        vars: &mut VarTree,
        var: Variable,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ident_type = var.var_type.clone();
        let assign_type = inner.next().unwrap().as_rule();
        Self::check_can_assign(&ident_type, assign_type)?;
        let val = inner.next().unwrap();
        let val_type = val.as_rule();

        self.assignment_inner(code, vars, &var, assign_type, &val, val_type)
    }

    fn assignment(&mut self, pair: Pair<Rule>, code: &mut Block, vars: &mut VarTree) -> Result<(), Box<dyn std::error::Error>> {
        let mut inner = pair.into_inner();
        let ident = inner.next().unwrap().as_span().as_str().to_string();
        let ident_info = vars.variables.iter().position(|v| v.name == ident.clone());
        if ident_info.is_none() {
            return Err(format!("Unknown variable: {}", ident).into());
        }
        let ident_info = vars.variables.get(ident_info.unwrap()).unwrap().clone();
        let ident_type = ident_info.var_type.clone();
        let assign_type = inner.next().unwrap().into_inner().next().unwrap().as_rule();
        Self::check_can_assign(&ident_type, assign_type)?;
        let val = inner.next().unwrap();
        let val_type = val.as_rule();

        self.assignment_inner(code, vars, &ident_info, assign_type, &val, val_type)
    }

    fn assignment_inner(
        &mut self,
        code: &mut Block,
        vars: &mut VarTree,
        ident_info: &Variable,
        assign_type: Rule,
        val: &Pair<Rule>,
        val_type: Rule,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let parameter = match val_type {
            Rule::literal => {
                let literal_type = VarType::from_rule(&val.clone().into_inner().next().unwrap().as_rule());
                Self::check_can_assign(&literal_type, assign_type)?;
                let id = Some(self.strings.len());
                let value = val.clone().into_inner().next().unwrap().as_span().as_str().to_string();
                self.strings.push(value.clone());
                Parameter {
                    name: "".to_string(),
                    value: Some(value),
                    id,
                    var_type: literal_type,
                    is_literal: true,
                }
            }
            Rule::identifier => {
                let name = val.as_span().as_str().to_string();
                let ident_info2 = vars.variables.iter().find(|v| v.name == name.clone());
                if ident_info2.is_none() {
                    return Err(format!("Unknown variable: {}", name).into());
                }
                let ident_type2 = ident_info2.unwrap().var_type.clone();
                Self::check_can_assign(&ident_type2, assign_type)?;
                todo!()
            }
            _ => {
                panic!("Unknown value: {:?}", val);
            }
        };
        // if this pass, the types are correct, but we do not check for uninitialized variables

        let variable = ident_info.name.clone();
        let rule = assign_type;
        let stmt = Statement::Assignment(variable, rule, parameter);
        code.statements.push(stmt);

        Ok(())
    }

    fn check_can_assign(ident_type: &VarType, assign_type: Rule) -> Result<(), Box<dyn std::error::Error>> {
        match assign_type {
            Rule::ASSIGN => {
                // Ok to any kind of assignment
            }
            Rule::ASSIGN_PLUS | Rule::ASSIGN_MINUS | Rule::ASSIGN_MULTI | Rule::ASSIGN_DIV | Rule::ASSIGN_MOD => {
                if !VAR_TYPES_MATH.contains(&ident_type) {
                    return Err(format!("Cannot perform math on {:?}", ident_type).into());
                }
            }
            Rule::ASSIGN_AND | Rule::ASSIGN_OR => {
                if !VAR_TYPES_LOGIC.contains(&ident_type) {
                    return Err(format!("Cannot perform logic on {:?}", ident_type).into());
                }
            }
            _ => {
                return Err(format!("Unknown assignment: {:?}", assign_type).into());
            }
        }
        Ok(())
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
pub(crate) struct VarTree {
    pub(crate) father: Option<usize>,
    pub(crate) variables: Vec<Variable>,
    pub(crate) children: HashMap<usize, VarTree>,
    pub(crate) stack: usize,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub(crate) name: String,
    var_type: VarType,
    pub stack: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub(crate) name: String,
    pub(crate) value: Option<String>,
    pub(crate) id: Option<usize>,
    pub(crate) var_type: VarType,
    pub(crate) is_literal: bool,
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
            Rule::STRING => VarType::String,
            Rule::char => VarType::Char,
            Rule::integer => VarType::Int,
            Rule::INT => VarType::Int,
            _ => panic!("Unknown type: {:?}", r),
        }
    }
}

const VAR_TYPES_MATH: [VarType; 2] = [VarType::Int, VarType::Char];
const VAR_TYPES_LOGIC: [VarType; 3] = [VarType::Bool, VarType::Int, VarType::Char];

#[derive(Debug, Clone)]
pub struct Function {
    pub(crate) name: String,
    pub(crate) id: usize,
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
    Assignment(String, Rule, Parameter),
}
