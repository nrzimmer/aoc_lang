#![allow(dead_code)]
use crate::syntax::{Function, Parameter, Statement, Syntax, VarType};

pub struct Assembler<'a> {
    syntax: Syntax<'a>,
    asm: Vec<String>,
}

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

    fn asm_function(&mut self, function: &Function) -> Result<(), Box<dyn std::error::Error>> {
        self.push_asm(format!("{}:", function.name));
        let add_return = true;
        for stmt in &function.code.statements {
            match stmt {
                Statement::Block(blk) => {
                    todo!("{:?}", blk)
                }
                Statement::FunctionCall(call) => {
                    self.asm_pass_parameters(call.parameters.clone())?;
                    self.push_asm(format!("  call {}", call.name));
                }
                Statement::ExternFunctionCall(call) => {
                    self.asm_pass_parameters(call.parameters.clone())?;
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
                            self.push_asm("  xorl %eax, %eax");
                        }
                    }
                    self.push_asm(format!("  call {}", call.name));
                }
                Statement::Return(ret) => {
                    //add_return = false;
                    todo!("{:?}", ret)
                }
            }
        }
        if add_return {
            self.push_asm("  xor  %eax, %eax\n  ret")
        }
        Ok(())
    }

    fn asm_pass_parameters(&mut self, params: Vec<Parameter>) -> Result<(), Box<dyn std::error::Error>> {
        for idx in (0..params.len()).rev() {
            let param = &params[idx];
            if idx > 5 {
                self.asm_push_parameter(param)?;
            }
            match idx {
                5 => self.push_asm("  movl $4, %r9d"),
                4 => self.push_asm("  movl $3, %r8d"),
                3 => self.push_asm("  movl $2, %ecx"),
                2 => self.push_asm("  movl $1, %edx"),
                1 => {
                    self.asm_pass_param(param, "%esi")?;
                }
                0 => {
                    self.asm_pass_param(param, "%rdi")?;
                }
                _ => {
                    panic!("Should not happen!!! parameter index: {}", idx)
                }
            }
        }
        Ok(())
    }

    fn asm_pass_param(&mut self, param: &Parameter, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
        match param.var_type {
            VarType::Int => {
                todo!()
            }
            VarType::Char => {
                todo!()
            }
            VarType::String => {
                self.push_asm(format!("  leaq .STR{}(%rip), {}", param.id.unwrap(), dest));
            }
            VarType::Bool => {
                todo!()
            }
            VarType::Void => return Err("Cannot pass void type".into()),
            VarType::VarArgs => return Err("Cannot pass varargs".into()),
        }
        Ok(())
    }

    fn asm_push_parameter(&self, param: &Parameter) -> Result<(), Box<dyn std::error::Error>> {
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

    fn push_asm<T: std::borrow::Borrow<str>>(&mut self, s: T) {
        self.asm.push(s.borrow().into());
    }
}
