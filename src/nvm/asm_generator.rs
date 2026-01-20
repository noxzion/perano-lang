use crate::ast::*;
use std::collections::HashMap;

pub struct NVMAssemblyGenerator {
    output: String,
    #[allow(dead_code)]
    labels: HashMap<String, String>,
    label_counter: u32,
    local_vars: HashMap<String, u8>,
    next_local: u8,
    loop_stack: Vec<(String, String)>,
    current_function: String,
    print_int_helper_emitted: bool,
}
impl NVMAssemblyGenerator {
    fn ensure_print_int_helper(&mut self) {
        if self.print_int_helper_emitted { return; }
        self.print_int_helper_emitted = true;
        
        self.output.push_str("__print_int_sys:\n");
        
        self.output.push_str("    store 250\n");
        
        self.output.push_str("    load 250\n");
        self.output.push_str("    push 0\n");
        self.output.push_str("    eq\n");
        let lbl_not_zero = "__pint_not_zero";
        self.output.push_str(&format!("    jz32 {}\n", lbl_not_zero));
        self.output.push_str("    push '0'\n");
        self.output.push_str("    syscall print\n");
        self.output.push_str("    ret\n");
        self.output.push_str(&format!("{}:\n", lbl_not_zero));
        
        self.output.push_str("    load 250\n");
        self.output.push_str("    push 0\n");
        self.output.push_str("    lt\n");
        let lbl_not_neg = "__pint_not_neg";
        self.output.push_str(&format!("    jz32 {}\n", lbl_not_neg));
        self.output.push_str("    push '-'\n");
        self.output.push_str("    syscall print\n");
        self.output.push_str("    load 250\n");
        self.output.push_str("    push 0\n");
        self.output.push_str("    swap\n");
        self.output.push_str("    sub\n");
        self.output.push_str("    store 250\n");
        self.output.push_str(&format!("{}:\n", lbl_not_neg));
        
        self.output.push_str("    push 1\n");
        self.output.push_str("    store 251\n");
        let lbl_find = "__pint_find";
        let lbl_find_done = "__pint_find_done";
        self.output.push_str(&format!("{}:\n", lbl_find));
        self.output.push_str("    load 251\n");
        self.output.push_str("    push 10\n");
        self.output.push_str("    mul\n");
        self.output.push_str("    load 250\n");
        self.output.push_str("    gt\n");
        self.output.push_str(&format!("    jnz32 {}\n", lbl_find_done));
        self.output.push_str("    load 251\n");
        self.output.push_str("    push 10\n");
        self.output.push_str("    mul\n");
        self.output.push_str("    store 251\n");
        self.output.push_str(&format!("    jmp32 {}\n", lbl_find));
        self.output.push_str(&format!("{}:\n", lbl_find_done));
        
        let lbl_loop = "__pint_loop";
        let lbl_done = "__pint_done";
        self.output.push_str(&format!("{}:\n", lbl_loop));
        self.output.push_str("    load 251\n");
        self.output.push_str("    push 0\n");
        self.output.push_str("    gt\n");
        self.output.push_str(&format!("    jz32 {}\n", lbl_done));
        self.output.push_str("    load 250\n");
        self.output.push_str("    load 251\n");
        self.output.push_str("    div\n");
        self.output.push_str("    push '0'\n");
        self.output.push_str("    add\n");
        self.output.push_str("    syscall print\n");
        self.output.push_str("    load 250\n");
        self.output.push_str("    load 251\n");
        self.output.push_str("    mod\n");
        self.output.push_str("    store 250\n");
        self.output.push_str("    load 251\n");
        self.output.push_str("    push 10\n");
        self.output.push_str("    div\n");
        self.output.push_str("    store 251\n");
        self.output.push_str(&format!("    jmp32 {}\n", lbl_loop));
        self.output.push_str(&format!("{}:\n", lbl_done));
        self.output.push_str("    ret\n");
    }

    pub fn new() -> Self {
        Self {
            output: String::new(),
            labels: HashMap::new(),
            label_counter: 0,
            local_vars: HashMap::new(),
            next_local: 0,
            loop_stack: Vec::new(),
            current_function: String::new(),
            print_int_helper_emitted: false,
        }
    }
    
    fn has_return_or_exit(&self, stmts: &[Statement]) -> bool {
        for stmt in stmts {
            match stmt {
                Statement::Return(_) => return true,
                Statement::InlineAsm { parts } => {
                    for part in parts {
                        if let crate::ast::AsmPart::Literal(s) = part {
                            if s.contains("syscall") && s.contains("exit") {
                                return true;
                            }
                        }
                    }
                }
                Statement::If { then_body, else_body, .. } => {
                    if self.has_return_or_exit(then_body) {
                        return true;
                    }
                    if let Some(else_stmts) = else_body {
                        if self.has_return_or_exit(else_stmts) {
                            return true;
                        }
                    }
                }
                Statement::For { body, .. } => {
                    if self.has_return_or_exit(body) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    pub fn generate(&mut self, program: &Program) -> String {
        
        self.output.push_str(".NVM0\n");
        self.output.push_str("; Generated by Perano Language Compiler\n\n");

        
        if let Some(main_func) = program.functions.iter().find(|f| f.name == "main") {
            self.generate_function(main_func, program);
        }

        
        for func in &program.functions {
            if func.name != "main" {
                self.generate_function(func, program);
            }
        }

        
        for (module_name, module) in &program.modules {
            if module_name == "stdio" {
                continue;
            }
            for func in &module.functions {
                if func.is_exported {
                    let full_name = format!("{}_{}", module.name, func.name);
                    self.generate_module_function(func, &full_name, program);
                }
            }
        }

        self.output.clone()
    }

    fn generate_function(&mut self, func: &Function, program: &Program) {
        self.current_function = func.name.clone();
        self.local_vars.clear();
        self.next_local = 0;

        self.output.push_str(&format!("; Function: {}\n", func.name));
        self.output.push_str(&format!("fn_{}:\n", func.name));

        
        for (i, param) in func.params.iter().enumerate() {
            self.local_vars.insert(param.name.clone(), i as u8);
            self.next_local = (i + 1) as u8;
            self.output.push_str(&format!("    ; param: {} -> local {}\n", param.name, i));
        }

        
        for stmt in &func.body {
            self.generate_statement(stmt, program);
        }

        if func.name == "main" && !self.has_return_or_exit(&func.body) {
            self.output.push_str("    ; Main returns 0 by default\n");
            
            self.output.push_str("    push 10\n");
            self.output.push_str("    syscall print\n");
            self.output.push_str("    push 0\n");
            self.output.push_str("    syscall exit\n");
        }
        
        self.output.push_str("    ret\n\n");
    }

    fn generate_module_function(&mut self, func: &Function, full_name: &str, program: &Program) {
        self.current_function = full_name.to_string();
        self.local_vars.clear();
        self.next_local = 0;

        self.output.push_str(&format!("; Module Function: {}\n", full_name));
        self.output.push_str(&format!("fn_{}:\n", full_name));

        for (i, param) in func.params.iter().enumerate() {
            self.local_vars.insert(param.name.clone(), i as u8);
            self.next_local = (i + 1) as u8;
            self.output.push_str(&format!("    ; param: {} -> local {}\n", param.name, i));
        }

        for stmt in &func.body {
            self.generate_statement(stmt, program);
        }

        self.output.push_str("    ret\n\n");
    }

    fn generate_statement(&mut self, stmt: &Statement, program: &Program) {
        match stmt {
            Statement::VarDecl { name, var_type, value } => {
                self.output.push_str(&format!("    ; var {} {}\n", name, 
                    var_type.as_ref().map(|t| t.as_str()).unwrap_or("int")));
                
                if let Some(init_expr) = value {
                    self.generate_expression(init_expr, program);
                } else {
                    self.output.push_str("    push 0\n");
                }
                
                let local_index = self.next_local;
                self.local_vars.insert(name.clone(), local_index);
                self.next_local += 1;
                
                self.output.push_str(&format!("    store {}\n", local_index));
            }

            Statement::Assignment { name, value } => {
                self.output.push_str(&format!("    ; {} = ...\n", name));
                self.generate_expression(value, program);
                
                if let Some(&local_index) = self.local_vars.get(name) {
                    self.output.push_str(&format!("    store {}\n", local_index));
                } else {
                    self.output.push_str(&format!("    ; ERROR: Variable not found: {}\n", name));
                }
            }

            Statement::If { condition, then_body, else_body } => {
                self.output.push_str("    ; if condition\n");
                self.generate_expression(condition, program);
                
                let else_label = self.generate_label("else");
                let end_label = self.generate_label("endif");
                
                self.output.push_str(&format!("    jz32 {}\n", else_label));
                
                self.output.push_str("    ; then block\n");
                for stmt in then_body {
                    self.generate_statement(stmt, program);
                }
                
                self.output.push_str(&format!("    jmp32 {}\n", end_label));
                
                self.output.push_str(&format!("{}:\n", else_label));
                
                if let Some(else_stmts) = else_body {
                    self.output.push_str("    ; else block\n");
                    for stmt in else_stmts {
                        self.generate_statement(stmt, program);
                    }
                }
                
                self.output.push_str(&format!("{}:\n", end_label));
            }

            Statement::For { init, condition, post, body } => {
                self.output.push_str("    ; for loop\n");
                
                if let Some(init_stmt) = init {
                    self.output.push_str("    ; init\n");
                    self.generate_statement(init_stmt, program);
                }
                
                let loop_start = self.generate_label("for_start");
                let loop_end = self.generate_label("for_end");
                let loop_continue = self.generate_label("for_continue");
                
                self.loop_stack.push((loop_end.clone(), loop_continue.clone()));
                
                self.output.push_str(&format!("{}:\n", loop_start));
                
                if let Some(cond) = condition {
                    self.output.push_str("    ; condition\n");
                    self.generate_expression(cond, program);
                    self.output.push_str(&format!("    jz32 {}\n", loop_end));
                }
                
                self.output.push_str("    ; body\n");
                for stmt in body {
                    self.generate_statement(stmt, program);
                }
                
                self.output.push_str(&format!("{}:\n", loop_continue));
                
                if let Some(post_stmt) = post {
                    self.output.push_str("    ; post\n");
                    self.generate_statement(post_stmt, program);
                }
                
                self.output.push_str(&format!("    jmp32 {}\n", loop_start));
                
                self.output.push_str(&format!("{}:\n", loop_end));
                self.loop_stack.pop();
            }

            Statement::Return(value) => {
                if let Some(_expr) = value {
                }
            }

            Statement::Expression(expr) => {
                self.generate_expression(expr, program);
            }

            Statement::InlineAsm { parts } => {
                use crate::ast::AsmPart;
                
                self.output.push_str("    ; inline asm\n");
                
                for part in parts {
                    match part {
                        AsmPart::Literal(s) => {
                            for line in s.lines() {
                                let trimmed = line.trim();
                                if !trimmed.is_empty() {
                                    self.output.push_str("    ");
                                    self.output.push_str(trimmed);
                                    self.output.push_str("\n");
                                }
                            }
                        }
                        AsmPart::Variable(var_name) => {
                            if let Some(&local_index) = self.local_vars.get(var_name) {
                                self.output.push_str(&format!("    load {}\n", local_index));
                            } else {
                                self.output.push_str(&format!("    ; ERROR: Unknown variable: {}\n", var_name));
                            }
                        }
                    }
                }
            }

            _ => {
                self.output.push_str("    ; unsupported statement\n");
            }
        }
    }

    fn generate_expression(&mut self, expr: &Expression, program: &Program) {
        match expr {
            Expression::Number(n) => {
                self.output.push_str(&format!("    push {}\n", n));
            }

            Expression::String(_s) => {
                self.output.push_str("    push 0  ; string not supported\n");
            }

            Expression::TemplateString { parts } => {
                use crate::ast::TemplateStringPart;
                
                self.output.push_str("    ; template string\n");
                
                for part in parts {
                    match part {
                        TemplateStringPart::Literal(lit) => {
                            for &ch in lit.as_bytes() {
                                let val: u8 = match ch {
                                    b'\n' => 10,
                                    b'\r' => 13,
                                    b'\t' => 9,
                                    _ => ch,
                                };
                                self.output.push_str(&format!("    push {}\n", val));
                                self.output.push_str("    syscall print\n");
                            }
                        }
                        TemplateStringPart::Expression { expr, format: _ } => {
                            self.generate_expression(expr, program);
                            self.ensure_print_int_helper();
                            self.output.push_str("    call __print_int_sys\n");
                        }
                    }
                }
            }

            Expression::Identifier(name) => {
                if let Some(&local_index) = self.local_vars.get(name) {
                    self.output.push_str(&format!("    load {}  ; {}\n", local_index, name));
                } else {
                    self.output.push_str(&format!("    ; ERROR: Variable not found: {}\n", name));
                    self.output.push_str("    push 0\n");
                }
            }

            Expression::Binary { op, left, right } => {
                self.generate_expression(left, program);
                self.generate_expression(right, program);
                
                match op {
                    BinaryOp::Add => self.output.push_str("    add\n"),
                    BinaryOp::Sub => self.output.push_str("    sub\n"),
                    BinaryOp::Mul => self.output.push_str("    mul\n"),
                    BinaryOp::Div => self.output.push_str("    div\n"),
                    BinaryOp::Mod => self.output.push_str("    mod\n"),
                    BinaryOp::Equal => self.output.push_str("    eq\n"),
                    BinaryOp::NotEqual => self.output.push_str("    neq\n"),
                    BinaryOp::Less => self.output.push_str("    lt\n"),
                    BinaryOp::Greater => self.output.push_str("    gt\n"),
                    BinaryOp::LessEqual => {
                        self.output.push_str("    gt\n");
                        self.output.push_str("    push 0\n");
                        self.output.push_str("    eq\n");
                    }
                    BinaryOp::GreaterEqual => {
                        self.output.push_str("    lt\n");
                        self.output.push_str("    push 0\n");
                        self.output.push_str("    eq\n");
                    }
                    _ => {
                        self.output.push_str("    ; unsupported binary op\n");
                    }
                }
            }

            Expression::Unary { op, operand } => {
                self.generate_expression(operand, program);
                
                match op {
                    UnaryOp::Neg => {
                        self.output.push_str("    push 0\n");
                        self.output.push_str("    swap\n");
                        self.output.push_str("    sub\n");
                    }
                    UnaryOp::Not => {
                        self.output.push_str("    push 0\n");
                        self.output.push_str("    eq\n");
                    }
                }
            }

            Expression::Call { function, args } => {
                self.output.push_str(&format!("    ; call {}\n", function));
                
                for arg in args.iter().rev() {
                    self.generate_expression(arg, program);
                }
                
                for (i, _) in args.iter().enumerate() {
                    let param_index = i as u8;
                    self.output.push_str(&format!("    store {}\n", param_index));
                }
                
                self.output.push_str(&format!("    call fn_{}\n", function));
            }

            Expression::ModuleCall { module, function, args } => {
                if module == "stdio" {
                    if function == "Print" || function == "Println" {
                        self.output.push_str(&format!("    ; call {}.{}\n", module, function));
                        if !args.is_empty() {
                            if let Expression::String(s) = &args[0] {
                                for &ch in s.as_bytes() {
                                    let val: u8 = match ch {
                                        b'\n' => 10,
                                        b'\r' => 13,
                                        b'\t' => 9,
                                        _ => ch,
                                    };
                                    self.output.push_str(&format!("    push {}\n", val));
                                    self.output.push_str("    syscall print\n");
                                }
                                if function == "Println" {
                                    self.output.push_str("    push '\n'\n");
                                    self.output.push_str("    syscall print\n");
                                }
                            } else if let Expression::TemplateString { .. } = &args[0] {
                                self.generate_expression(&args[0], program);
                                if function == "Println" {
                                    self.output.push_str("    push '\n'\n");
                                    self.output.push_str("    syscall print\n");
                                }
                            } else {
                                self.generate_expression(&args[0], program);
                                self.ensure_print_int_helper();
                                self.output.push_str("    call __print_int_sys\n");
                                if function == "Println" {
                                    self.output.push_str("    push '\n'\n");
                                    self.output.push_str("    syscall print\n");
                                }
                            }
                        }
                        return;
                    }
                }

                self.output.push_str(&format!("    ; call {}.{}\n", module, function));
                for arg in args.iter().rev() {
                    self.generate_expression(arg, program);
                }
                self.output.push_str(&format!("    call fn_{}_{}\n", module, function));
            }

            _ => {
                self.output.push_str("    ; unsupported expression\n");
                self.output.push_str("    push 0\n");
            }
        }
    }

    fn generate_label(&mut self, prefix: &str) -> String {
        self.label_counter += 1;
        format!("{}_{}_{}", prefix, self.current_function, self.label_counter)
    }

    


        
    }