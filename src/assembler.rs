#![allow(dead_code)]

use crate::lexer::Rule;
use crate::syntax::{Function, Parameter, Statement, Syntax, VarTree, VarType, Variable};

pub struct Assembler<'a> {
    syntax: Syntax<'a>,
    asm: Vec<String>,
}

struct Register<'a> {
    x64: &'a str,
    x32: &'a str,
    x16: &'a str,
}

const REGS: [Register; 6] = [
    Register {
        x64: "rdi",
        x32: "edi",
        x16: "di",
    },
    Register {
        x64: "rsi",
        x32: "esi",
        x16: "si",
    },
    Register {
        x64: "rdx",
        x32: "edx",
        x16: "dx",
    },
    Register {
        x64: "rcx",
        x32: "ecx",
        x16: "cx",
    },
    Register {
        x64: "r8",
        x32: "r8d",
        x16: "r8w",
    },
    Register {
        x64: "r9",
        x32: "r9d",
        x16: "r9w",
    },
];

impl<'a> Assembler<'a> {
    pub fn new(syntax: Syntax<'a>) -> Assembler<'a> {
        Assembler { syntax, asm: Vec::new() }
    }

    pub fn assemble(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        if !self.syntax.functions.contains_key("main") {
            panic!("No main function");
        }

        self.push_asm(".text");
        self.push_asm(".section	.rodata");
        self.push_asm(".align 8");
        for id in 0..self.syntax.strings.len() {
            let str = self.syntax.strings[id].clone();
            self.push_asm(format!(".STR{}:", id));
            self.push_asm(format!("  .string {}", str));
        }
        self.push_asm("");

        let mut main = self.syntax.functions.remove("main").unwrap();
        self.push_asm(".text\n.globl main");
        self.asm_function(&mut main)?;

        // Collect all function names first to avoid the borrow checker error
        let function_names: Vec<String> = self.syntax.functions.keys().cloned().collect();
        self.push_asm("");

        // Process each function by name
        for name in function_names {
            if name == "main" {
                continue;
            }
            let f = self.syntax.functions.get(&name).unwrap().clone();
            self.asm_function(&f)?;
            self.push_asm("");
        }

        Ok(self.asm.join("\n"))
    }

    fn calc_stack(vt: &mut VarTree, start: usize) -> usize {
        let mut stack = start;
        for var in &mut vt.variables {
            var.stack = Some(stack);
            stack += 8;
        }
        for (_, subtree) in &mut vt.children {
            stack = Self::calc_stack(subtree, stack);
        }
        stack
    }

    fn asm_function(&mut self, function: &Function) -> Result<(), Box<dyn std::error::Error>> {
        let stack = {
            let vt = self.syntax.variables.children.get_mut(&function.id).unwrap();
            vt.stack = Self::calc_stack(vt, 0);
            vt.stack
        };
        self.push_asm(format!("{}:", function.name));
        self.push_asm("  pushq %rbp");
        self.push_asm("  movq %rsp, %rbp");
        if stack > 0 {
            self.push_asm(format!("  subq ${}, %rsp", stack));
        }
        let add_return = true;
        for stmt in &function.code.statements {
            match stmt {
                Statement::Block(blk) => {
                    todo!("{:?}", blk)
                }
                Statement::FunctionCall(call) => {
                    self.push_asm("# Function call");
                    self.asm_pass_parameters(call.parameters.clone(), function.id)?;
                    self.push_asm(format!("  call {}", call.name));
                }
                Statement::ExternFunctionCall(call) => {
                    self.push_asm("# Extern function call");
                    self.asm_pass_parameters(call.parameters.clone(), function.id)?;
                    if call.name == "printf" {
                        /*
                        For libc printf and it's variants %AL contains the number of
                        vector registers (XMM0-XMM7) used for floating-point arguments.
                        First 8 float args goes in XMM0-XMM7.
                        Push additional float args to stack in reverse order.
                        For now we do not have floating point support, so we set it to 0.
                         */
                        let f = self.syntax.externs.get(&call.name).unwrap();
                        if f.parameters.contains(&VarType::VarArgs) {
                            self.push_asm("  xor %eax, %eax");
                        }
                    }
                    self.push_asm(format!("  call {}@PLT", call.name));
                }
                Statement::Return(ret) => {
                    self.push_asm("# Return");
                    //add_return = false;
                    self.push_asm("  movq %rbp, %rsp");
                    self.push_asm("  popq %rbp");
                    todo!("{:?}", ret)
                }
                Statement::Assignment(name, rule, param) => {
                    self.push_asm("# Assignment");
                    let vt = self.syntax.variables.children.get(&function.id).unwrap();
                    let var = self.find_variable(vt, name);
                    if var.is_none() {
                        return Err(format!("Cannot find variable {}", name).into());
                    }
                    match rule {
                        Rule::ASSIGN => match param.var_type {
                            VarType::Int => todo!("{:?}", param.var_type),
                            VarType::Char => todo!("{:?}", param.var_type),
                            VarType::String => {
                                self.push_asm(format!("  leaq .STR{}(%rip), %rax", param.id.unwrap()));
                                self.push_asm(format!("  movq %rax, -{}(%rbp)", var.unwrap().stack.unwrap().to_string()));
                            }
                            VarType::Bool => todo!("{:?}", param.var_type),
                            VarType::Void => todo!("{:?}", param.var_type),
                            VarType::VarArgs => todo!("{:?}", param.var_type),
                        },
                        Rule::ASSIGN_PLUS => todo!("{:?}", rule),
                        Rule::ASSIGN_MINUS => todo!("{:?}", rule),
                        Rule::ASSIGN_MULTI => todo!("{:?}", rule),
                        Rule::ASSIGN_DIV => todo!("{:?}", rule),
                        Rule::ASSIGN_MOD => todo!("{:?}", rule),
                        Rule::ASSIGN_AND => todo!("{:?}", rule),
                        Rule::ASSIGN_OR => todo!("{:?}", rule),
                        _ => panic!("Unknown assignment {:?}", rule),
                    }
                }
                _ => todo!("{:?}", stmt),
            }
        }
        self.push_asm("# End Function");
        if add_return {
            self.push_asm("  xor  %eax, %eax");
            self.push_asm("  movq %rbp, %rsp");
            self.push_asm("  popq %rbp");
            self.push_asm("  ret");
        }
        Ok(())
    }

    fn asm_pass_parameters(&mut self, params: Vec<Parameter>, func_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        for idx in 0..params.len() {
            let param = &params[idx];
            if idx > 5 {
                self.asm_push_parameter(param, func_id)?;
            } else {
                self.asm_pass_param(param, REGS[idx].x64, func_id)?;
            }
        }
        Ok(())
    }

    fn find_variable(&self, vt: &VarTree, name: &str) -> Option<Variable> {
        for var_info in &vt.variables {
            if var_info.name == name {
                return Some(var_info.clone());
            }
        }
        // Todo - check upper the three
        None
    }

    fn asm_pass_param(&mut self, param: &Parameter, dest: &str, func_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        match param.var_type {
            VarType::Int => {
                if param.is_literal {
                    self.push_asm(format!("  movq ${}, %{}", param.clone().value.unwrap(), dest));
                } else {
                    todo!("{:?}", param);
                }
            }
            VarType::Char => {
                todo!()
            }
            VarType::String => {
                if param.is_literal {
                    self.push_asm(format!("  leaq .STR{}(%rip), %{}", param.id.unwrap(), dest));
                } else {
                    let vt = self.syntax.variables.children.get(&func_id).unwrap();
                    let var = self.find_variable(vt, param.name.as_str());
                    if var.is_none() {
                        return Err(format!("Cannot find variable {}", param.name).into());
                    }
                    self.push_asm(format!("  movq -{}(%rbp), %{}", var.unwrap().stack.unwrap().to_string(), dest));
                }
            }
            VarType::Bool => {
                todo!()
            }
            VarType::Void => return Err("Cannot pass void type".into()),
            VarType::VarArgs => return Err("Cannot pass varargs".into()),
        }
        Ok(())
    }

    fn asm_push_parameter(&mut self, param: &Parameter, func_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        match param.var_type {
            VarType::Int => {
                todo!()
            }
            VarType::Char => {
                todo!()
            }
            VarType::String => {
                self.push_asm(format!("  leaq .STR{}(%rip), %rax", param.id.unwrap()));
                self.push_asm("  pushq %rax");
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
        Ok(())
    }

    fn push_asm<T: std::borrow::Borrow<str>>(&mut self, s: T) {
        self.asm.push(s.borrow().into());
    }
}
