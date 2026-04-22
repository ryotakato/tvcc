use crate::tokeniser::{Token, TokenListIterator, TokenList, TokenKind};

use crate::cc_util::CompileError;

use std::collections::HashMap;
use std::str::FromStr;


//pub struct Node {
//    pub kind: NodeKind,
//    pub ty: Ty
//
//}

#[derive(Debug)]
pub enum NodeKind {
    Add { lhs: Option<Box<NodeKind>>, rhs: Option<Box<NodeKind>> }, // +
    Sub { lhs: Option<Box<NodeKind>>, rhs: Option<Box<NodeKind>> }, // -
    Mul { lhs: Option<Box<NodeKind>>, rhs: Option<Box<NodeKind>> }, // * (multiply)
    Div { lhs: Option<Box<NodeKind>>, rhs: Option<Box<NodeKind>> }, // /
    Eq { lhs: Option<Box<NodeKind>>, rhs: Option<Box<NodeKind>> },  // ==
    Ne { lhs: Option<Box<NodeKind>>, rhs: Option<Box<NodeKind>> },  // !=
    Lt { lhs: Option<Box<NodeKind>>, rhs: Option<Box<NodeKind>> },  // <
    Le { lhs: Option<Box<NodeKind>>, rhs: Option<Box<NodeKind>> },  // <=
    Assign { lhs: Option<Box<NodeKind>>, rhs: Option<Box<NodeKind>> }, // =
    Lvar { offset: i32 }, // local variables + offset
    Num {value: i32 }, // integer + value
    Return { lhs: Option<Box<NodeKind>> }, // return
    If { cond: Option<Box<NodeKind>>, then: Option<Box<NodeKind>>, else_then: Option<Box<NodeKind>> }, // if
    For { init: Option<Box<NodeKind>>, cond: Option<Box<NodeKind>>, inc: Option<Box<NodeKind>>, then: Option<Box<NodeKind>>}, // for or while
    Block { body: Vec<Option<Box<NodeKind>>> }, // block
    FuncCall { name: String, args: Vec<Option<Box<NodeKind>>> }, // func call
    FuncDef { name: String, r_type: Ty, params: Vec<Option<Box<NodeKind>>>, stack_size: i32, block: Option<Box<NodeKind>> }, // func define
    Addr { lhs: Option<Box<NodeKind>> }, // & (pointer)
    Deref { lhs: Option<Box<NodeKind>> }, // * (pointer)
}

impl NodeKind {
    fn wrap(self) -> Option<Box<NodeKind>> {
        Some(Box::new(self))
    }
}

#[derive(Debug, Clone)]
pub enum Ty {
    Int,
    Pointer { base: Option<Box<Ty>> },
}

impl FromStr for Ty {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "int" => Ok(Ty::Int),
            _ => Err(()),
        }
    }
}

impl Ty {
    fn new_pointer(base: Ty) -> Ty {
        Ty::Pointer {
            base: Some(Box::new(base))
        }
    }

    fn is_int(self) -> bool {
        match self {
            Ty::Int => true,
            _ => false,
        }
    }

    fn is_pointer(self) -> bool {
        match self {
            Ty::Pointer { base: _ } => true,
            _ => false,
        }
    }
}


// struct for local variales 
struct LocalVariable {
    latest_offset: i32,
    variables: HashMap<String, (i32, Ty)>,
}

impl LocalVariable {
    fn new() -> LocalVariable {
        LocalVariable {
            latest_offset: 0,
            variables: HashMap::new(),
        }
    }

    fn add_variable(&mut self, variale_name: String, v_type: Ty) -> bool {
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
    cur_func: String,
    local_variables: HashMap<String, LocalVariable>,
}

impl<'a> Parser<'a> {
    pub fn new(token_list: &'a TokenList) -> Parser<'a> {

        let token_iter: TokenListIterator<'a> = token_list.iter();
        let local_variables = HashMap::new();

        Parser {
            token_iter,
            cur_func: Default::default(),
            local_variables,
        }
    }

    fn set_cur_func(&mut self, name: String) {
        self.cur_func = name.clone();
        self.local_variables.insert(name.to_string(), LocalVariable::new());
    }

    //fn cur_func_add_local_variable(&mut self, variale_name: &str, v_type_name: &str) -> Result<(), CompileError> {
    //    match v_type_name.parse::<Ty>() {
    //        Ok(v_type) => self.cur_func_add_local_variable_by_type(variale_name, v_type),
    //        Err(_) => {
    //            Err(CompileError::new(&[&format!("type: {} is not defined in {}", v_type_name, &self.cur_func)]))
    //        }
    //    }
    //}

    fn cur_func_add_local_variable_by_type(&mut self, variale_name: &str, v_type: Ty) -> Result<(), CompileError> {

        if self.local_variables.get_mut(&self.cur_func).unwrap().add_variable(variale_name.to_string(), v_type) {
            Ok(())
        } else {
            Err(CompileError::new(&[&format!("variable: {} is already defined in {}", variale_name, &self.cur_func)]))
        }
    }

    fn cur_func_local_variable_offset(&mut self, variale_name: &str) -> Result<i32, CompileError> {
        match self.local_variables.get_mut(&self.cur_func).unwrap().find_offset(variale_name) {
            Some(offset) => Ok(offset),
            None => Err(CompileError::new(&[&format!("variable: {} is not defined in {}", variale_name, &self.cur_func)]))
        }
    }


    fn cur_func_calculate_stack_size(&mut self) -> i32 {
        // calucuate offset and align
        let total_offset = self.local_variables.get_mut(&self.cur_func).unwrap().latest_offset;
        Self::align_to(total_offset, 16)
    }

    fn align_to(n: i32, align: i32) -> i32 {
        (n + align - 1) / align * align
    }



    fn cur_token(&self) -> &Token {
        &self.token_iter.current().unwrap()
    }

    fn next_token(&mut self) -> &'a Token {
        &self.token_iter.next().unwrap()
    }



    pub fn parse(&mut self) -> Result<Vec<Option<Box<NodeKind>>>, CompileError> {

        //program

        let mut functions: Vec<Option<Box<NodeKind>>> = Vec::new();

        while !self.cur_token().at_eof() {
            functions.push(self.function()?);
        }

        Ok(functions)
    }

    fn function(&mut self) -> Result<Option<Box<NodeKind>>, CompileError> {
        // return type
        let r_type: Ty = self.declspec()?;

        // func name
        let name = self.cur_token().expect_ident()?.to_string();

        self.set_cur_func(name.clone());

        let _ = &self.next_token();

        // argument
        self.stmt_expect_symbol("(")?;

        let mut params: Vec<Option<Box<NodeKind>>> = Vec::new();

        // func args
        while let Err(_) = self.cur_token().expect_symbol(")") {

            if let Ok(_) = self.cur_token().expect_symbol(",") {
                let _ = &self.next_token();
            }

            // identify local variable
            let v_type: Ty = self.declspec()?;

            // define local variable
            let v_name = self.declarator(v_type)?;
            let offset = self.cur_func_local_variable_offset(&v_name)?; // definitely success because it is just after add variable
            params.push(NodeKind::Lvar{ offset }.wrap());

        }

        let _ = &self.next_token(); // skip ")"

        // block
        self.stmt_expect_symbol("{")?;
        let block = self.compound_stmt()?;

        let stack_size = self.cur_func_calculate_stack_size();

        let node = NodeKind::FuncDef { name, r_type, params, stack_size, block }.wrap();
        return Ok(node);
    }



    // compound_stmt = (declaration | stmt)* "}"
    fn compound_stmt(&mut self) -> Result<Option<Box<NodeKind>>, CompileError> {

        let mut stmts: Vec<Option<Box<NodeKind>>> = Vec::new();
        while let Err(_) = self.cur_token().expect_symbol("}") {

            if let Token { kind: TokenKind::Type(_), .. } = self.cur_token() {
                stmts.push(self.declaration()?);
            } else {
                stmts.push(self.stmt()?);
            }
        }
        self.stmt_expect_symbol("}")?;

        let node = NodeKind::Block { body: stmts, }.wrap();
        return Ok(node);
    }



    fn declaration(&mut self) -> Result<Option<Box<NodeKind>>, CompileError> {
        let base_type: Ty = self.declspec()?;

        let mut assigns: Vec<Option<Box<NodeKind>>> = Vec::new();
        while let Err(_) = self.cur_token().expect_symbol(";") {

            if let Ok(_) = self.cur_token().expect_symbol(",") {
                let _ = &self.next_token();
            }

            let v_name = self.declarator(base_type.clone())?;

            let Ok(_) = self.cur_token().expect_symbol("=") else {
                continue;
            };

            let _ = &self.next_token();

            let offset = self.cur_func_local_variable_offset(&v_name)?; // definitely success because it is just after add variable
            assigns.push(NodeKind::Assign { lhs: NodeKind::Lvar { offset }.wrap(), rhs: self.expr()?, }.wrap());


        }

        let node = NodeKind::Block { body: assigns, }.wrap();
        return Ok(node);
    }

    fn declspec(&mut self) -> Result<Ty, CompileError> {
        let v_type_name = self.cur_token().expect_type()?.to_string();

        let _ = &self.next_token();

        match v_type_name.parse::<Ty>() {
            Ok(v_type) => Ok(v_type),
            Err(_) => {
                Err(CompileError::new(&[&format!("type: {} is not defined in {}", v_type_name, &self.cur_func)]))
            }
        }

    }

    fn declarator(&mut self, base_type: Ty) -> Result<String, CompileError> {
        // while "*" continues, creates Ty including original type
        let mut ty = base_type;
        while let Ok(_) = self.cur_token().expect_symbol("*") {
            ty = Ty::new_pointer(ty);
            let _ = &self.next_token();
        }

        let v_name = self.cur_token().expect_ident()?.to_string();

        // define local variable
        let _ = self.cur_func_add_local_variable_by_type(&v_name, ty)?;

        let _ = &self.next_token();
        return Ok(v_name);
    }



    fn stmt(&mut self) -> Result<Option<Box<NodeKind>>, CompileError> {


        let cur = self.cur_token();

        match cur {
            // "return" expr ";"
            Token { kind: TokenKind::Return, .. } => {
                let _ = &self.next_token();
                let node = NodeKind::Return { lhs: self.expr()?, }.wrap();
                self.stmt_expect_symbol(";")?;
                return Ok(node);
            },
            // "if" "(" expr ")" stmt ("else" stmt)?
            Token { kind: TokenKind::If, .. } => {


                let _ = &self.next_token();
                self.stmt_expect_symbol("(")?;
                // cond
                let cond = self.expr()?;

                self.stmt_expect_symbol(")")?;

                // then
                let then = self.stmt()?;

                // else_then
                match self.cur_token().at_else() {
                    true => {
                        let _ = &self.next_token();
                        let node = NodeKind::If { cond, then, else_then: self.stmt()? }.wrap();
                        return Ok(node);
                    },
                    false => {
                        let node = NodeKind::If { cond, then, else_then: None }.wrap();
                        return Ok(node);
                    }
                }

            },
            // "for" "(" expr? ";" expr? ";" expr? ")" stmt
            Token { kind: TokenKind::For, .. } => {

                let _ = &self.next_token();
                self.stmt_expect_symbol("(")?;

                // init
                let init = match self.cur_token().expect_symbol(";") {
                    Ok(_n) => None,
                    Err(_) => self.expr()?,
                };

                // cond
                let _ = &self.next_token();
                let cond = match self.cur_token().expect_symbol(";") {
                    Ok(_n) => None,
                    Err(_) => self.expr()?,
                };

                // inc
                let _ = &self.next_token();
                let inc = match self.cur_token().expect_symbol(")") {
                    Ok(_n) => None,
                    Err(_) => self.expr()?,
                };

                // then
                let _ = &self.next_token();
                let then = self.stmt()?;

                let node = NodeKind::For { init, cond, inc, then }.wrap();
                return Ok(node);

            },
            // "while" "(" expr ")" stmt
            Token { kind: TokenKind::While, .. } => {

                let _ = &self.next_token();
                self.stmt_expect_symbol("(")?;
                // cond
                let cond = self.expr()?;

                self.stmt_expect_symbol(")")?;

                // then
                let then = self.stmt()?;

                let node = NodeKind::For { init: None, cond, inc: None, then }.wrap();
                return Ok(node);
            },
            _ => {
                match cur.expect_symbol("{") {
                    // "{" compound_stmt
                    Ok(_) => {
                        let _ = &self.next_token();
                        let node = self.compound_stmt()?;
                        return Ok(node);
                    },
                    // expr? ";"
                    Err(_) => {
                        match cur.expect_symbol(";") {
                            Ok(_) => {
                                let _ = &self.next_token();
                                let node = NodeKind::Block { body: Vec::new() }.wrap();
                                return Ok(node);
                            },
                            Err(_) => {
                                let node = self.expr()?;
                                self.stmt_expect_symbol(";")?;
                                return Ok(node);
                            }
                        }
                    }
                }
            }
        }
    }

    fn stmt_expect_symbol(&mut self, symbol: &str) -> Result<(), CompileError> {

        let _ = self.cur_token().expect_symbol(symbol)?;
        let _ = &self.next_token();
        Ok(())
    }


    fn expr(&mut self) -> Result<Option<Box<NodeKind>>, CompileError> {
        self.assign()
    }

    fn assign(&mut self) -> Result<Option<Box<NodeKind>>, CompileError> {
        let mut node: Option<Box<NodeKind>> = self.equality()?;

        if let Ok(_) = self.cur_token().expect_symbol("=") {
            let _ = &self.next_token();
            node = NodeKind::Assign { lhs: node, rhs: self.assign()?, }.wrap();
            return Ok(node);
        }

        return Ok(node);
    }

    fn equality(&mut self) -> Result<Option<Box<NodeKind>>, CompileError> {
        let mut node: Option<Box<NodeKind>> = self.relational()?;

        loop {

            if let Ok(_) = self.cur_token().expect_symbol("==") {
                let _ = &self.next_token();
                node = NodeKind::Eq { lhs: node, rhs: self.relational()?, }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("!=") {
                let _ = &self.next_token();
                node = NodeKind::Ne { lhs: node, rhs: self.relational()?, }.wrap();
                continue;
            }

            return Ok(node);
        }
    }

    fn relational(&mut self) -> Result<Option<Box<NodeKind>>, CompileError> {
        let mut node: Option<Box<NodeKind>> = self.add()?;

        loop {

            if let Ok(_) = self.cur_token().expect_symbol("<") {
                let _ = &self.next_token();
                node = NodeKind::Lt { lhs: node, rhs: self.add()?, }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("<=") {
                let _ = &self.next_token();
                node = NodeKind::Le { lhs: node, rhs: self.add()?, }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol(">") {
                let _ = &self.next_token();
                node = NodeKind::Lt { lhs: self.add()?, rhs: node, }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol(">=") {
                let _ = &self.next_token();
                node = NodeKind::Le { lhs: self.add()?, rhs: node, }.wrap();
                continue;
            }

            return Ok(node);
        }
    }

    fn add(&mut self) -> Result<Option<Box<NodeKind>>, CompileError> {
        let mut node: Option<Box<NodeKind>> = self.mul()?;

        loop {

            if let Ok(_) = self.cur_token().expect_symbol("+") {
                let _ = &self.next_token();
                node = NodeKind::Add { lhs: node, rhs: self.mul()?, }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("-") {
                let _ = &self.next_token();
                node = NodeKind::Sub { lhs: node, rhs: self.mul()?, }.wrap();
                continue;
            }

            return Ok(node);
        }
    }

    fn mul(&mut self) -> Result<Option<Box<NodeKind>>, CompileError> {
        let mut node: Option<Box<NodeKind>> = self.unary()?;

        loop {

            if let Ok(_) = self.cur_token().expect_symbol("*") {
                let _ = &self.next_token();
                node = NodeKind::Mul { lhs: node, rhs: self.unary()?, }.wrap();
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("/") {
                let _ = &self.next_token();
                node = NodeKind::Div { lhs: node, rhs: self.unary()?, }.wrap();
                continue;
            }

            return Ok(node);
        }
    }

    fn unary(&mut self) -> Result<Option<Box<NodeKind>>, CompileError> {
        if let Ok(_) = self.cur_token().expect_symbol("+") {
            let _ = &self.next_token();
            return self.unary();
        }
        if let Ok(_) = self.cur_token().expect_symbol("-") {
            let _ = &self.next_token();
            let zero = NodeKind::Num { value: 0, }.wrap();
            return Ok(NodeKind::Sub { lhs: zero, rhs: self.unary()?, }.wrap());
        }
        if let Ok(_) = self.cur_token().expect_symbol("&") {
            let _ = &self.next_token();
            return Ok(NodeKind::Addr { lhs: self.unary()?, }.wrap());
        }
        if let Ok(_) = self.cur_token().expect_symbol("*") {
            let _ = &self.next_token();
            return Ok(NodeKind::Deref { lhs: self.unary()?, }.wrap());
        }

        return self.primary();
    }


    fn primary(&mut self) -> Result<Option<Box<NodeKind>>, CompileError> {

        if let Ok(_) = self.cur_token().expect_symbol("(") {
            let _ = &self.next_token();
            let node = self.expr()?;

            let _ = self.cur_token().expect_symbol(")")?;
            let _ = &self.next_token();

            return Ok(node);
        }

        if let Ok(name) = self.cur_token().expect_ident() {
            let name = name.to_string();
            let _ = &self.next_token();

            match self.cur_token().expect_symbol("(") {
                Ok(_) => {
                    // func call
                    return self.func_call(name);
                },
                Err(_) => {
                    // local variable
                    let offset = self.cur_func_local_variable_offset(&name)?;
                    return Ok(NodeKind::Lvar{ offset }.wrap());
                }
            }
        }

        let n = self.cur_token().expect_number()?;
        let node = NodeKind::Num { value: n }.wrap();
        let _ = &self.next_token();
        return Ok(node);
    }



    fn func_call(&mut self, name: String) -> Result<Option<Box<NodeKind>>, CompileError> {

        let _ = &self.next_token();

        let mut args: Vec<Option<Box<NodeKind>>> = Vec::new();

        while let Err(_) = self.cur_token().expect_symbol(")") {
            args.push(self.assign()?);

            if let Ok(_) = self.cur_token().expect_symbol(",") {
                let _ = &self.next_token();
            }
        }

        let _ = &self.next_token(); // skip ")"

        Ok(NodeKind::FuncCall { name, args, }.wrap())
    }
}



