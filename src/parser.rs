use crate::tokeniser::{Token, TokenListIterator, TokenList, TokenKind};
//use crate::cc_util::{error, errors};
use crate::cc_util;

use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug)]
pub enum Node {
    Add { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> }, // +
    Sub { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> }, // -
    Mul { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> }, // * (multiply)
    Div { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> }, // /
    Eq { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> },  // ==
    Ne { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> },  // !=
    Lt { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> },  // <
    Le { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> },  // <=
    Assign { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> }, // =
    Lvar { offset: i32 }, // local variables + offset
    Num {value: i32 }, // integer + value
    Return { lhs: Option<Box<Node>> }, // return
    If { cond: Option<Box<Node>>, then: Option<Box<Node>>, else_then: Option<Box<Node>> }, // if
    For { init: Option<Box<Node>>, cond: Option<Box<Node>>, inc: Option<Box<Node>>, then: Option<Box<Node>>}, // for or while
    Block { body: Vec<Option<Box<Node>>> }, // block
    FuncCall { name: String, args: Vec<Option<Box<Node>>> }, // func call
    FuncDef { name: String, r_type: String, params: Vec<Option<Box<Node>>>, block: Option<Box<Node>> }, // func define
    Addr { lhs: Option<Box<Node>> }, // & (pointer)
    Deref { lhs: Option<Box<Node>> }, // * (pointer)
}

impl Node {
    fn wrap(self) -> Option<Box<Node>> {
        Some(Box::new(self))
    }
}

#[derive(Debug, Clone)]
pub enum VarType {
    Int,
}

impl FromStr for VarType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "int" => Ok(VarType::Int),
            _ => Err(()),
        }
    }

}


// struct for local variales 
struct LocalVariable {
    latest_offset: i32,
    variables: HashMap<String, (i32, VarType)>,
}

impl LocalVariable {
    fn new() -> LocalVariable {
        LocalVariable {
            latest_offset: 0,
            variables: HashMap::new(),
        }
    }

    fn add_variable(&mut self, variale_name: String, v_type: VarType) -> bool {
        match self.variables.entry(variale_name) {
            std::collections::hash_map::Entry::Occupied(_entry) => false,
            std::collections::hash_map::Entry::Vacant(entry) => {
                self.latest_offset = self.latest_offset + 8;
                entry.insert((self.latest_offset, v_type));
                true
            }
        }
    }

    fn find_offset(&mut self, variale_name: &str) -> Option<i32> {
        match self.variables.get(variale_name) {
            Some((offset, _)) => Some(*offset),
            _ => None
        }
    }
}



pub struct Parser<'a> {
    token_iter: TokenListIterator<'a>,
    origin_formula: &'a str,
    cur_func: String,
    local_variables: HashMap<String, LocalVariable>,
}

impl<'a> Parser<'a> {
    pub fn new(token_list: &'a TokenList) -> Parser<'a> {

        let token_iter: TokenListIterator<'a> = token_list.iter();
        let origin_formula = &token_list.origin_formula;
        let local_variables = HashMap::new();

        Parser {
            token_iter,
            origin_formula,
            cur_func: Default::default(),
            local_variables,
        }
    }

    pub fn set_cur_func(&mut self, name: String) {
        self.cur_func = name.clone();
        self.local_variables.insert(name.to_string(), LocalVariable::new());
    }

    pub fn cur_func_add_local_variable(&mut self, variale_name: &str, v_type_name: &str) -> Result<(), String> {

        match v_type_name.parse::<VarType>() {
            Ok(v_type) => self.cur_func_add_local_variable_by_type(variale_name, v_type),
            Err(_) => {
                Err(format!("type: {} is not defined in {}", v_type_name, &self.cur_func))
            }
        }

    }

    pub fn cur_func_add_local_variable_by_type(&mut self, variale_name: &str, v_type: VarType) -> Result<(), String> {

        if self.local_variables.get_mut(&self.cur_func).unwrap().add_variable(variale_name.to_string(), v_type) {
            Ok(())
        } else {
            Err(format!("variable: {} is already defined in {}", variale_name, &self.cur_func))
        }
    }

    pub fn cur_func_local_variable_offset(&mut self, variale_name: &str) -> Result<i32, String> {
        match self.local_variables.get_mut(&self.cur_func).unwrap().find_offset(variale_name) {
            Some(offset) => Ok(offset),
            None => Err(format!("variable: {} is not defined in {}", variale_name, &self.cur_func))
        }
    }

    fn cur_token(&self) -> &Token {
        &self.token_iter.current().unwrap()
    }

    fn next_token(&mut self) -> &'a Token {
        &self.token_iter.next().unwrap()
    }

    pub fn parse(&mut self) -> Vec<Option<Box<Node>>> {
        //self.stmt_expect_symbol("{");

        //let mut program: Vec<Option<Box<Node>>> = Vec::new();
        //program.push(self.compound_stmt());

        //if !self.cur_token().at_eof() {
        //    cc_util::errors(&[&self.origin_formula, "The last } is unexpected"]);
        //}

        //program

        let mut functions: Vec<Option<Box<Node>>> = Vec::new();

        while !self.cur_token().at_eof() {
            functions.push(self.function());
        }

        functions
    }

    fn function(&mut self) -> Option<Box<Node>> {
        // return type
        let r_type = match self.cur_token().expect_type() {
            Ok(r_type) => r_type.to_string(),
            Err(e) => {
                cc_util::errors(&[&self.origin_formula, &e]);
            }
        };

        let _ = &self.next_token();

        // func name
        let name = match self.cur_token().expect_ident() {
            Ok(name) => name.to_string(),
            Err(e) => {
                cc_util::errors(&[&self.origin_formula, &e]);
            }
        };

        self.set_cur_func(name.clone());

        let _ = &self.next_token();

        // argument
        self.stmt_expect_symbol("(");

        let mut params: Vec<Option<Box<Node>>> = Vec::new();

        while let Err(_) = self.cur_token().expect_symbol(")") {

            let v_type = match self.cur_token().expect_type() {
                Ok(v_type) => v_type.to_string(),
                Err(e) => {
                    cc_util::errors(&[&self.origin_formula, &e]);
                }
            };

            let _ = &self.next_token();

            let v_name = match self.cur_token().expect_ident() {
                Ok(param_name) => param_name.to_string(),
                Err(e) => {
                    cc_util::errors(&[&self.origin_formula, &e]);
                }
            };

            let _ = &self.next_token();

            // define local variable
            match self.cur_func_add_local_variable(&v_name, &v_type) {
                Ok(()) => {
                    let offset = self.cur_func_local_variable_offset(&v_name).unwrap(); // definitely success because it is just after add variable
                    params.push(Node::Lvar{ offset }.wrap());
                },
                Err(e) => {
                    cc_util::errors(&[&self.origin_formula, &e]);
                }
            }

            if let Ok(_) = self.cur_token().expect_symbol(",") {
                let _ = &self.next_token();
            }
        }

        let _ = &self.next_token(); // skip ")"

        // block
        self.stmt_expect_symbol("{");
        let block = self.compound_stmt();

        let node = Node::FuncDef { name, r_type, params, block }.wrap();
        return node;
    }

    // compound_stmt = (declaration | stmt)* "}"
    fn compound_stmt(&mut self) -> Option<Box<Node>> {

        let mut stmts: Vec<Option<Box<Node>>> = Vec::new();
        while let Err(_) = self.cur_token().expect_symbol("}") {

            if let Token { kind: TokenKind::Type(_), .. } = self.cur_token() {
                stmts.push(self.declaration());
            } else {
                stmts.push(self.stmt());
            }
        }
        self.stmt_expect_symbol("}");

        let node = Node::Block { body: stmts, }.wrap();
        return node;
    }

    fn declaration(&mut self) -> Option<Box<Node>> {
        let base_type: VarType = self.declspec();

        let mut assigns: Vec<Option<Box<Node>>> = Vec::new();
        while let Err(_) = self.cur_token().expect_symbol(";") {

            if let Ok(_) = self.cur_token().expect_symbol(",") {
                let _ = &self.next_token();
            }

            let v_name = self.declarator(base_type.clone());

            let Ok(_) = self.cur_token().expect_symbol("=") else {
                continue;
            };

            let _ = &self.next_token();

            let offset = self.cur_func_local_variable_offset(&v_name).unwrap(); // definitely success because it is just after add variable
            assigns.push(Node::Assign { lhs: Node::Lvar { offset }.wrap(), rhs: self.expr(), }.wrap());


        }

        let node = Node::Block { body: assigns, }.wrap();
        return node;
    }

    fn declspec(&mut self) -> VarType {
        let v_type_name = match self.cur_token().expect_type() {
            Ok(v_type_name) => v_type_name.to_string(),
            Err(e) => {
                cc_util::errors(&[&self.origin_formula, &e]);
            }
        };

        let _ = &self.next_token();

        match v_type_name.parse::<VarType>() {
            Ok(v_type) => v_type,
            Err(_) => {
                cc_util::errors(&[&self.origin_formula, &format!("type: {} is not defined in {}", v_type_name, &self.cur_func)]);
            }
        }

    }

    fn declarator(&mut self, base_type: VarType) -> String {

        let v_name = match self.cur_token().expect_ident() {
            Ok(v_name) => v_name.to_string(),
            Err(e) => {
                cc_util::errors(&[&self.origin_formula, &e]);
            }
        };

        // define local variable
        match self.cur_func_add_local_variable_by_type(&v_name, base_type) {
            Ok(()) => {
                let _ = &self.next_token();
                return v_name;
            },
            Err(e) => {
                cc_util::errors(&[&self.origin_formula, &e]);
            }
        };
    }



    fn stmt(&mut self) -> Option<Box<Node>> {


        let cur = self.cur_token();

        match cur {
            // "return" expr ";"
            Token { kind: TokenKind::Return, .. } => {
                let _ = &self.next_token();
                let node = Node::Return { lhs: self.expr(), }.wrap();
                self.stmt_expect_symbol(";");
                return node;
            },
            // "if" "(" expr ")" stmt ("else" stmt)?
            Token { kind: TokenKind::If, .. } => {


                let _ = &self.next_token();
                self.stmt_expect_symbol("(");
                // cond
                let cond = self.expr();

                self.stmt_expect_symbol(")");

                // then
                let then = self.stmt();

                // else_then
                match self.cur_token().at_else() {
                    true => {
                        let _ = &self.next_token();
                        let else_then = self.stmt();
                        let node = Node::If { cond, then, else_then }.wrap();
                        return node;
                    },
                    false => {
                        let node = Node::If { cond, then, else_then: None }.wrap();
                        return node;
                    }
                }

            },
            // "for" "(" expr? ";" expr? ";" expr? ")" stmt
            Token { kind: TokenKind::For, .. } => {

                let _ = &self.next_token();
                self.stmt_expect_symbol("(");

                // init
                let init = match self.cur_token().expect_symbol(";") {
                    Ok(_n) => None,
                    Err(_) => self.expr(),
                };

                // cond
                let _ = &self.next_token();
                let cond = match self.cur_token().expect_symbol(";") {
                    Ok(_n) => None,
                    Err(_) => self.expr(),
                };

                // inc
                let _ = &self.next_token();
                let inc = match self.cur_token().expect_symbol(")") {
                    Ok(_n) => None,
                    Err(_) => self.expr(),
                };

                // then
                let _ = &self.next_token();
                let then = self.stmt();

                let node = Node::For { init, cond, inc, then }.wrap();
                return node;

            },
            // "while" "(" expr ")" stmt
            Token { kind: TokenKind::While, .. } => {

                let _ = &self.next_token();
                self.stmt_expect_symbol("(");
                // cond
                let cond = self.expr();

                self.stmt_expect_symbol(")");

                // then
                let then = self.stmt();

                let node = Node::For { init: None, cond, inc: None, then }.wrap();
                return node;
            },
            _ => {
                match cur.expect_symbol("{") {
                    // "{" compound_stmt
                    Ok(_) => {
                        let _ = &self.next_token();
                        let node = self.compound_stmt();
                        return node;
                    },
                    // expr? ";"
                    Err(_) => {
                        match cur.expect_symbol(";") {
                            Ok(_) => {
                                let _ = &self.next_token();
                                let node = Node::Block { body: Vec::new() }.wrap();
                                return node;
                            },
                            Err(_) => {
                                let node = self.expr();
                                self.stmt_expect_symbol(";");
                                return node;
                            }
                        }
                    }
                }
            }
        }
    }

    fn stmt_expect_symbol(&mut self, symbol: &str) {

        match self.cur_token().expect_symbol(symbol) {
            Ok(_n) => {
                let _ = &self.next_token();
            },
            Err(e) => {
                cc_util::errors(&[&self.origin_formula, &e]);
            },
        };
    }


    fn expr(&mut self) -> Option<Box<Node>> {
        self.assign()
    }

    fn assign(&mut self) -> Option<Box<Node>> {
        let mut node: Option<Box<Node>> = self.equality();

        if let Ok(_) = self.cur_token().expect_symbol("=") {
            let _ = &self.next_token();
            node = Node::Assign { lhs: node, rhs: self.assign(), }.wrap();
            return node;
        }

        return node;
    }

    fn equality(&mut self) -> Option<Box<Node>> {
        let mut node: Option<Box<Node>> = self.relational();

        loop {

            if let Ok(_) = self.cur_token().expect_symbol("==") {
                let _ = &self.next_token();
                node = Node::Eq { lhs: node, rhs: self.relational(), }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("!=") {
                let _ = &self.next_token();
                node = Node::Ne { lhs: node, rhs: self.relational(), }.wrap();
                continue;
            }

            return node;
        }
    }

    fn relational(&mut self) -> Option<Box<Node>> {
        let mut node: Option<Box<Node>> = self.add();

        loop {

            if let Ok(_) = self.cur_token().expect_symbol("<") {
                let _ = &self.next_token();
                node = Node::Lt { lhs: node, rhs: self.add(), }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("<=") {
                let _ = &self.next_token();
                node = Node::Le { lhs: node, rhs: self.add(), }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol(">") {
                let _ = &self.next_token();
                node = Node::Lt { lhs: self.add(), rhs: node, }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol(">=") {
                let _ = &self.next_token();
                node = Node::Le { lhs: self.add(), rhs: node, }.wrap();
                continue;
            }

            return node;
        }
    }

    fn add(&mut self) -> Option<Box<Node>> {
        let mut node: Option<Box<Node>> = self.mul();

        loop {

            if let Ok(_) = self.cur_token().expect_symbol("+") {
                let _ = &self.next_token();
                node = Node::Add { lhs: node, rhs: self.mul(), }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("-") {
                let _ = &self.next_token();
                node = Node::Sub { lhs: node, rhs: self.mul(), }.wrap();
                continue;
            }

            return node;
        }
    }

    fn mul(&mut self) -> Option<Box<Node>> {
        let mut node: Option<Box<Node>> = self.unary();

        loop {

            if let Ok(_) = self.cur_token().expect_symbol("*") {
                let _ = &self.next_token();
                node = Node::Mul { lhs: node, rhs: self.unary(), }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("/") {
                let _ = &self.next_token();
                node = Node::Div { lhs: node, rhs: self.unary(), }.wrap();
                continue;
            }

            return node;
        }
    }

    fn unary(&mut self) -> Option<Box<Node>> {
        if let Ok(_) = self.cur_token().expect_symbol("+") {
            let _ = &self.next_token();
            return self.unary();
        }
        if let Ok(_) = self.cur_token().expect_symbol("-") {
            let _ = &self.next_token();
            let zero = Node::Num { value: 0, }.wrap();
            return Node::Sub { lhs: zero, rhs: self.unary(), }.wrap();
        }
        if let Ok(_) = self.cur_token().expect_symbol("&") {
            let _ = &self.next_token();
            return Node::Addr { lhs: self.unary(), }.wrap();
        }
        if let Ok(_) = self.cur_token().expect_symbol("*") {
            let _ = &self.next_token();
            return Node::Deref { lhs: self.unary(), }.wrap();
        }

        return self.primary();
    }


    fn primary(&mut self) -> Option<Box<Node>> {

        if let Ok(_) = self.cur_token().expect_symbol("(") {
            let _ = &self.next_token();
            let node = self.expr();

            match self.cur_token().expect_symbol(")") {
                Ok(_) => &self.next_token(),
                Err(e) => {
                    cc_util::errors(&[&self.origin_formula, &e]);
                    //return None;
                },
            };

            return node;
        }

        if let Ok(name) = self.cur_token().expect_ident() {
            let name = name.to_string();
            let _ = &self.next_token();

            let node = match self.cur_token().expect_symbol("(") {
                Ok(_) => {
                    // func call
                    self.func_call(name)
                },
                Err(_) => {
                    // local variable
                    match self.cur_func_local_variable_offset(&name) {
                        Ok(offset) => Node::Lvar{ offset }.wrap(),
                        Err(e) => {
                            cc_util::errors(&[&self.origin_formula, &e]);
                            //return None;
                        }
                    }
                }
            };

            return node;
        }


        match self.cur_token().expect_number() {
            Ok(n) => {
                let node = Node::Num { value: n }.wrap();
                let _ = &self.next_token();
                return node;
            },
            Err(e) => {
                cc_util::errors(&[&self.origin_formula, &e]);
                //return None;
            },
        };
    }



    fn func_call(&mut self, name: String) -> Option<Box<Node>> {

        let _ = &self.next_token();

        let mut args: Vec<Option<Box<Node>>> = Vec::new();

        while let Err(_) = self.cur_token().expect_symbol(")") {
            args.push(self.assign());

            if let Ok(_) = self.cur_token().expect_symbol(",") {
                let _ = &self.next_token();
            }
        }

        let _ = &self.next_token(); // skip ")"

        Node::FuncCall { name, args, }.wrap()
    }
}



