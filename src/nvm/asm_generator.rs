use crate::ast::*;
use std::collections::{HashMap, HashSet};

pub struct NVMAssemblyGenerator {
    output: String,
    #[allow(dead_code)]
    labels: HashMap<String, String>,
    label_counter: u32,
    local_vars: HashMap<String, u8>,
    next_local: u8,
    param_vars: HashMap<String, u8>,
    param_types: HashMap<String, String>,
    loop_stack: Vec<(String, String)>,
    current_function: String,
    print_int_helper_emitted: bool,
    print_int_helper_code: String,
    scratch_local: u8,
}
impl NVMAssemblyGenerator {
    fn ensure_print_int_helper(&mut self) {
        if self.print_int_helper_emitted { return; }
        self.print_int_helper_emitted = true;
        self.print_int_helper_code = String::from(
"__print_int_sys:\n").to_string();
        self.print_int_helper_code.push_str(
"    enter 3\n");
        self.print_int_helper_code.push_str(
"    loada 0\n");
        self.print_int_helper_code.push_str(
"    storer 0\n");
        self.print_int_helper_code.push_str(
"    loadr 0\n");
        self.print_int_helper_code.push_str(
"    push 0\n");
        self.print_int_helper_code.push_str(
"    eq\n");
        self.print_int_helper_code.push_str(
"    jz __pint_zero_cont\n");
        self.print_int_helper_code.push_str(
"__pint_zero:\n");
        self.print_int_helper_code.push_str(
"    push '0'\n");
        self.print_int_helper_code.push_str(
"    syscall print\n");
        self.print_int_helper_code.push_str(
"    leave\n");
        self.print_int_helper_code.push_str(
"    ret\n");
        self.print_int_helper_code.push_str(
"__pint_zero_cont:\n");
        self.print_int_helper_code.push_str(
"    loadr 0\n");
        self.print_int_helper_code.push_str(
"    push 0\n");
        self.print_int_helper_code.push_str(
"    lt\n");
        self.print_int_helper_code.push_str(
"    jz __pint_not_neg\n");
        self.print_int_helper_code.push_str(
"    push '-'\n");
        self.print_int_helper_code.push_str(
"    syscall print\n");
        self.print_int_helper_code.push_str(
"    loadr 0\n");
        self.print_int_helper_code.push_str(
"    push 0\n");
        self.print_int_helper_code.push_str(
"    swap\n");
        self.print_int_helper_code.push_str(
"    sub\n");
        self.print_int_helper_code.push_str(
"    storer 0\n");
        self.print_int_helper_code.push_str(
"__pint_not_neg:\n");
        self.print_int_helper_code.push_str(
"    push 1\n");
        self.print_int_helper_code.push_str(
"    storer 1\n");
        self.print_int_helper_code.push_str(
"__pint_find:\n");
        self.print_int_helper_code.push_str(
"    loadr 1\n");
        self.print_int_helper_code.push_str(
"    push 10\n");
        self.print_int_helper_code.push_str(
"    mul\n");
        self.print_int_helper_code.push_str(
"    loadr 0\n");
        self.print_int_helper_code.push_str(
"    gt\n");
        self.print_int_helper_code.push_str(
"    jnz __pint_find_done\n");
        self.print_int_helper_code.push_str(
"    loadr 1\n");
        self.print_int_helper_code.push_str(
"    push 10\n");
        self.print_int_helper_code.push_str(
"    mul\n");
        self.print_int_helper_code.push_str(
"    storer 1\n");
        self.print_int_helper_code.push_str(
"    jmp __pint_find\n");
        self.print_int_helper_code.push_str(
"__pint_find_done:\n");
        self.print_int_helper_code.push_str(
"__pint_loop:\n");
        self.print_int_helper_code.push_str(
"    loadr 1\n");
        self.print_int_helper_code.push_str(
"    push 0\n");
        self.print_int_helper_code.push_str(
"    gt\n");
        self.print_int_helper_code.push_str(
"    jz __pint_done\n");
        self.print_int_helper_code.push_str(
"    loadr 0\n");
        self.print_int_helper_code.push_str(
"    loadr 1\n");
        self.print_int_helper_code.push_str(
"    div\n");
        self.print_int_helper_code.push_str(
"    storer 2\n");
        self.print_int_helper_code.push_str(
"    loadr 2\n");
        self.print_int_helper_code.push_str(
"    push '0'\n");
        self.print_int_helper_code.push_str(
"    add\n");
        self.print_int_helper_code.push_str(
"    syscall print\n");
        self.print_int_helper_code.push_str(
"    loadr 0\n");
        self.print_int_helper_code.push_str(
"    loadr 1\n");
        self.print_int_helper_code.push_str(
"    mod\n");
        self.print_int_helper_code.push_str(
"    storer 0\n");
        self.print_int_helper_code.push_str(
"    loadr 1\n");
        self.print_int_helper_code.push_str(
"    push 10\n");
        self.print_int_helper_code.push_str(
"    div\n");
        self.print_int_helper_code.push_str(
"    storer 1\n");
        self.print_int_helper_code.push_str(
"    jmp __pint_loop\n");
        self.print_int_helper_code.push_str(
"__pint_done:\n");
        self.print_int_helper_code.push_str(
"    leave\n");
        self.print_int_helper_code.push_str(
"    ret\n");
    }

    pub fn new() -> Self {
        Self {
            output: String::new(),
            labels: HashMap::new(),
            label_counter: 0,
            local_vars: HashMap::new(),
            next_local: 0,
            param_vars: HashMap::new(),
            param_types: HashMap::new(),
            loop_stack: Vec::new(),
            current_function: String::new(),
            print_int_helper_emitted: false,
            print_int_helper_code: String::new(),
            scratch_local: 0,
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
        self.output.push_str("\n");

        
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

        if self.print_int_helper_emitted {
            self.output.push_str(&self.print_int_helper_code);
        }
        self.output.clone()
    }

    fn count_locals(stmts: &[Statement]) -> u8 {
        let mut names: HashSet<String> = HashSet::new();
        fn walk(stmts: &[Statement], out: &mut HashSet<String>) {
            for s in stmts {
                match s {
                    Statement::VarDecl { name, .. } => {
                        out.insert(name.clone());
                    }
                    Statement::If { then_body, else_body, .. } => {
                        walk(then_body, out);
                        if let Some(else_b) = else_body { walk(else_b, out); }
                    }
                    Statement::For { init, condition: _, post: _, body } => {
                        if let Some(init_stmt) = init { walk(std::slice::from_ref(init_stmt), out); }
                        walk(body, out);
                    }
                    Statement::Expression(_) | Statement::Assignment { .. } | Statement::Return(_) | Statement::InlineAsm { .. } | Statement::ArrayDecl { .. } | Statement::ArrayAssignment { .. } => {}
                }
            }
        }
        walk(stmts, &mut names);
        u8::try_from(names.len()).unwrap_or(u8::MAX)
    }

    fn generate_function(&mut self, func: &Function, program: &Program) {
        self.current_function = func.name.clone();
        self.local_vars.clear();
        self.param_vars.clear();
        self.param_types.clear();
        self.next_local = 0;

        self.output.push_str(&format!("fn_{}:\n", func.name));

        
        let mut locals_count = Self::count_locals(&func.body);
        self.scratch_local = locals_count;
        locals_count = locals_count.saturating_add(1);
        self.output.push_str(&format!("    enter {}\n", locals_count));

        for (i, param) in func.params.iter().enumerate() {
            self.param_vars.insert(param.name.clone(), i as u8);
            self.param_types.insert(param.name.clone(), param.param_type.clone());
        }

        for stmt in &func.body {
            self.generate_statement(stmt, program);
        }

        if func.name == "main" && !self.has_return_or_exit(&func.body) {
            self.output.push_str("    push 10\n");
            self.output.push_str("    syscall print\n");
            self.output.push_str("    push 0\n");
            self.output.push_str("    syscall exit\n");
        }

        self.output.push_str("    leave\n");
        self.output.push_str("    ret\n\n");
    }

    fn generate_module_function(&mut self, func: &Function, full_name: &str, program: &Program) {
        self.current_function = full_name.to_string();
        self.local_vars.clear();
        self.param_vars.clear();
        self.param_types.clear();
        self.next_local = 0;

        self.output.push_str(&format!("fn_{}:\n", full_name));

        let is_string_syscall = full_name.starts_with("novaria_") && 
            (full_name.ends_with("_Open") || full_name.ends_with("_Create") || full_name.ends_with("_Delete"));
        
        if !is_string_syscall {
            let locals_count = Self::count_locals(&func.body);
            self.output.push_str(&format!("    enter {}\n", locals_count));
        }

        for (i, param) in func.params.iter().enumerate() {
            self.param_vars.insert(param.name.clone(), i as u8);
            self.param_types.insert(param.name.clone(), param.param_type.clone());
        }

        for stmt in &func.body {
            self.generate_statement(stmt, program);
        }

        if !is_string_syscall {
            self.output.push_str("    leave\n");
        }
        self.output.push_str("    ret\n\n");
    }

    fn generate_statement(&mut self, stmt: &Statement, program: &Program) {
        match stmt {
            Statement::VarDecl { name, var_type, value } => {
                
                if let Some(init_expr) = value {
                    self.generate_expression(init_expr, program);
                } else {
                    self.output.push_str("    push 0\n");
                }
                
                let local_index = self.next_local;
                self.local_vars.insert(name.clone(), local_index);
                self.next_local += 1;
                
                self.output.push_str(&format!("    storer {}\n", local_index));
            }

            Statement::Assignment { name, value } => {
                self.generate_expression(value, program);
                
                if let Some(&local_index) = self.local_vars.get(name) {
                    self.output.push_str(&format!("    storer {}\n", local_index));
                } else if let Some(&arg_index) = self.param_vars.get(name) {
                    self.output.push_str(&format!("    storea {}\n", arg_index));
                } else {
                    self.output.push_str(&format!("    ; ERROR: Variable not found: {}\n", name));
                }
            }

            Statement::If { condition, then_body, else_body } => {
                self.generate_expression(condition, program);
                
                let else_label = self.generate_label("else");
                let end_label = self.generate_label("endif");
                
                self.output.push_str(&format!("    jz {}\n", else_label));
                
                for stmt in then_body {
                    self.generate_statement(stmt, program);
                }
                
                self.output.push_str(&format!("    jmp {}\n", end_label));
                
                self.output.push_str(&format!("{}:\n", else_label));
                
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.generate_statement(stmt, program);
                    }
                }
                
                self.output.push_str(&format!("{}:\n", end_label));
            }

            Statement::For { init, condition, post, body } => {
                
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
                    self.output.push_str(&format!("    jz {}\n", loop_end));
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
                
                self.output.push_str(&format!("    jmp {}\n", loop_start));
                
                self.output.push_str(&format!("{}:\n", loop_end));
                self.loop_stack.pop();
            }

            Statement::Return(value) => {
                if let Some(expr) = value {
                    self.generate_expression(expr, program);
                }
                self.output.push_str("    leave\n");
                self.output.push_str("    ret\n");
            }

            Statement::Expression(expr) => {
                self.generate_expression(expr, program);
            }

            Statement::InlineAsm { parts } => {
                use crate::ast::AsmPart;
                
                
                let mut i = 0;
                while i < parts.len() {
                    match &parts[i] {
                        AsmPart::Literal(s) => {
                            for line in s.lines() {
                                let trimmed = line.trim();
                                if !trimmed.is_empty() {
                                    self.output.push_str("    ");
                                    self.output.push_str(trimmed);

                                    if i + 1 < parts.len() {
                                        if let AsmPart::Variable(var_name) = &parts[i + 1] {
                                            let instr = trimmed.split_whitespace().next().unwrap_or("");
                                            match instr.to_lowercase().as_str() {
                                                "push" => {
                                                    if let Some(var_type) = self.param_types.get(var_name) {
                                                        if var_type == "string" {
                                                            self.output.push_str(&format!("\n    ; ERROR: Cannot push string parameter '{}' - strings not yet supported in inline asm\n", var_name));
                                                            self.output.push_str("    ; TODO: Implement string expansion");
                                                        } else {
                                                            self.output.push_str(&format!(" {}", self.param_vars.get(var_name).unwrap()));
                                                        }
                                                    } else if let Some(&local_index) = self.local_vars.get(var_name) {
                                                        self.output.push_str(&format!(" {}", local_index));
                                                    } else {
                                                        self.output.push_str(&format!("\n    ; ERROR: Unknown variable: {}\n", var_name));
                                                    }
                                                }
                                                "load" => {
                                                    if let Some(&param_index) = self.param_vars.get(var_name) {
                                                        self.output.push_str("a");
                                                        self.output.push_str(&format!(" {}", param_index));
                                                    } else if let Some(&local_index) = self.local_vars.get(var_name) {
                                                        self.output.push_str("r");
                                                        self.output.push_str(&format!(" {}", local_index));
                                                    } else {
                                                        self.output.push_str(&format!("\n    ; ERROR: Unknown variable: {}\n", var_name));
                                                    }
                                                }
                                                "store" => {
                                                    if let Some(&param_index) = self.param_vars.get(var_name) {
                                                        self.output.push_str("a");
                                                        self.output.push_str(&format!(" {}", param_index));
                                                    } else if let Some(&local_index) = self.local_vars.get(var_name) {
                                                        self.output.push_str("r");
                                                        self.output.push_str(&format!(" {}", local_index));
                                                    } else {
                                                        self.output.push_str(&format!("\n    ; ERROR: Unknown variable: {}\n", var_name));
                                                    }
                                                }
                                                _ => {
                                                    if let Some(&param_index) = self.param_vars.get(var_name) {
                                                        self.output.push_str(&format!(" {}", param_index));
                                                    } else if let Some(&local_index) = self.local_vars.get(var_name) {
                                                        self.output.push_str(&format!(" {}", local_index));
                                                    } else {
                                                        self.output.push_str(&format!("\n    ; ERROR: Unknown variable: {}\n", var_name));
                                                    }
                                                }
                                            }
                                            i += 1;
                                        }
                                    }
                                    
                                    self.output.push_str("\n");
                                }
                            }
                        }
                        AsmPart::Variable(_) => {
                        }
                    }
                    i += 1;
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
                    self.output.push_str(&format!("    loadr {}  ; local {}\n", local_index, name));
                } else if let Some(&arg_index) = self.param_vars.get(name) {
                    self.output.push_str(&format!("    loada {}  ; arg {}\n", arg_index, name));
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
                                    self.output.push_str("    push '\\n'\n");
                                    self.output.push_str("    syscall print\n");
                                }
                            } else if let Expression::TemplateString { .. } = &args[0] {
                                self.generate_expression(&args[0], program);
                                if function == "Println" {
                                    self.output.push_str("    push '\\n'\n");
                                    self.output.push_str("    syscall print\n");
                                }
                            } else {
                                self.generate_expression(&args[0], program);
                                self.output.push_str(&format!("    storer {}\n", self.scratch_local));
                                self.ensure_print_int_helper();
                                self.output.push_str(&format!("    loadr {}\n", self.scratch_local));
                                self.output.push_str("    call __print_int_sys\n");
                                if function == "Println" {
                                    self.output.push_str("    push '\\n'\n");
                                    self.output.push_str("    syscall print\n");
                                }
                            }
                        }
                        return;
                    }
                }

                self.output.push_str(&format!("    ; call {}.{}\n", module, function));

                if module == "novaria" {
                    let string_syscalls = [("Open", 2), ("Create", 5), ("Delete", 6)];
                    for &(name, syscall_num) in &string_syscalls {
                        if function == name && !args.is_empty() {
                            if let Expression::String(s) = &args[0] {
                                self.output.push_str("    push 0\n");
                                for &ch in s.as_bytes().iter().rev() {
                                    self.output.push_str(&format!("    push {}\n", ch));
                                }
                                self.output.push_str(&format!("    syscall {}\n", syscall_num));
                                return;
                            }
                        }
                    }
                }
                
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