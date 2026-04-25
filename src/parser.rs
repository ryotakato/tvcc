use crate::tokeniser::{Token, TokenListIterator, TokenList, TokenKind};

use crate::cc_util::CompileError;

use std::collections::HashMap;
use std::str::FromStr;


#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub ty: Option<Ty>
}

impl Node {

    fn ty(&mut self) -> &mut Ty {
        // for lazy evaluation, cache in ty property (Option)
        self.ty.get_or_insert_with(|| {

            match &mut self.kind {
                NodeKind::Add {lhs, .. } | NodeKind::Sub {lhs, .. }  | NodeKind::Mul {lhs, .. }  | NodeKind::Div {lhs, .. }  | NodeKind::Assign {lhs, .. } => {
                    // extract the type of lhs, and clone
                    let mut bx = lhs.as_mut().unwrap();
                    (**(&mut bx)).ty().clone()
                },
                NodeKind::Eq { .. }  | NodeKind::Ne {.. }  | NodeKind::Lt {.. }  | NodeKind::Le {.. }  | NodeKind::Num {.. }  | NodeKind::FuncCall {.. } => {
                    Ty::Int
                },
                NodeKind::Lvar { name, offset:_, ty } => {
                    ty.clone()
                },
                NodeKind::Addr { lhs } => {
                    // extract the type of lhs, and create new pointer based on the type of lhs
                    let mut bx = lhs.as_mut().unwrap();
                    let ty = (**(&mut bx)).ty().clone();
                    Ty::new_pointer(ty)
                },
                NodeKind::Deref { lhs } => {
                    // extract the type of lhs, and clone
                    let mut bx = lhs.as_mut().unwrap();
                    let ty = (**(&mut bx)).ty().clone();

                    match ty {
                        Ty::Pointer { base } => {
                            (*base.unwrap()).clone()
                        },
                        _ => Ty::Int,
                    }
                },
                _ => Ty::Int,
            }

        })
    }

}

#[derive(Debug)]
pub enum NodeKind {
    Add { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> }, // +
    Sub { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> }, // -
    Mul { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> }, // * (multiply)
    Div { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> }, // /
    Eq { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> },  // ==
    Ne { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> },  // !=
    Lt { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> },  // <
    Le { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> },  // <=
    Assign { lhs: Option<Box<Node>>, rhs: Option<Box<Node>> }, // =
    Lvar { name: String, offset: i32, ty:Ty }, // local variables + name, offset
    Num {value: i32 }, // integer + value
    Return { lhs: Option<Box<Node>> }, // return
    If { cond: Option<Box<Node>>, then: Option<Box<Node>>, else_then: Option<Box<Node>> }, // if
    For { init: Option<Box<Node>>, cond: Option<Box<Node>>, inc: Option<Box<Node>>, then: Option<Box<Node>>}, // for or while
    Block { body: Vec<Option<Box<Node>>> }, // block
    FuncCall { name: String, args: Vec<Option<Box<Node>>> }, // func call
    FuncDef { name: String, r_type: Ty, params: Vec<Option<Box<Node>>>, stack_size: i32, block: Option<Box<Node>> }, // func define
    Addr { lhs: Option<Box<Node>> }, // & (pointer)
    Deref { lhs: Option<Box<Node>> }, // * (pointer)
}

impl NodeKind {
    fn wrap(self) -> Option<Box<Node>> {
        Some(Box::new(Node { kind: self, ty: None }))
    }

    fn eight() -> Option<Box<Node>> {
        NodeKind::Num { value: 8 }.wrap()
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

    fn find_variable(&mut self, variale_name: &str) -> Option<(i32, Ty)> {
        match self.variables.get(variale_name) {
            Some((offset, ty)) => Some((*offset, ty.clone())), // TODO although "find" method, it is need "clone". waste of memory
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

    fn cur_func_local_variable_offset(&mut self, variale_name: &str) -> Result<(i32, Ty), CompileError> {
        match self.local_variables.get_mut(&self.cur_func).unwrap().find_variable(variale_name) {
            Some((offset, ty)) => Ok((offset, ty)),
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



    pub fn parse(&mut self) -> Result<Vec<Option<Box<Node>>>, CompileError> {

        //program

        let mut functions: Vec<Option<Box<Node>>> = Vec::new();

        while !self.cur_token().at_eof() {
            functions.push(self.function()?);
        }

        Ok(functions)
    }

    fn function(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        // return type
        let r_type: Ty = self.declspec()?;

        // func name
        let name = self.cur_token().expect_ident()?.to_string();

        self.set_cur_func(name.clone());

        let _ = &self.next_token();

        // argument
        self.stmt_expect_symbol("(")?;

        let mut params: Vec<Option<Box<Node>>> = Vec::new();

        // func args
        while let Err(_) = self.cur_token().expect_symbol(")") {

            if let Ok(_) = self.cur_token().expect_symbol(",") {
                let _ = &self.next_token();
            }

            // identify local variable
            let v_type: Ty = self.declspec()?;

            // define local variable
            let (v_name, v_ty) = self.declarator(v_type)?;
            let (offset, _) = self.cur_func_local_variable_offset(&v_name)?; // definitely success because it is just after add variable
            params.push(NodeKind::Lvar{ name: v_name, offset, ty: v_ty }.wrap());

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
    fn compound_stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {

        let mut stmts: Vec<Option<Box<Node>>> = Vec::new();
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



    fn declaration(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let base_type: Ty = self.declspec()?;

        let mut assigns: Vec<Option<Box<Node>>> = Vec::new();
        while let Err(_) = self.cur_token().expect_symbol(";") {

            if let Ok(_) = self.cur_token().expect_symbol(",") {
                let _ = &self.next_token();
            }

            let (v_name, v_ty) = self.declarator(base_type.clone())?;

            let Ok(_) = self.cur_token().expect_symbol("=") else {
                continue;
            };

            let _ = &self.next_token();

            let (offset, _) = self.cur_func_local_variable_offset(&v_name)?; // definitely success because it is just after add variable
            assigns.push(NodeKind::Assign { lhs: NodeKind::Lvar { name: v_name, offset, ty:v_ty }.wrap(), rhs: self.expr()?, }.wrap());


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

    fn declarator(&mut self, base_type: Ty) -> Result<(String, Ty), CompileError> {
        // while "*" continues, creates Ty including original type
        let mut ty = base_type;
        while let Ok(_) = self.cur_token().expect_symbol("*") {
            ty = Ty::new_pointer(ty);
            let _ = &self.next_token();
        }

        let v_name = self.cur_token().expect_ident()?.to_string();

        // define local variable (TODO here, deep copy ty instance, it is waste of memory, but need re-think the structure local variables)
        let _ = self.cur_func_add_local_variable_by_type(&v_name, ty.clone())?;

        let _ = &self.next_token();
        return Ok((v_name, ty)); 
    }



    fn stmt(&mut self) -> Result<Option<Box<Node>>, CompileError> {


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


    fn expr(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        self.assign()
    }

    fn assign(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node: Option<Box<Node>> = self.equality()?;

        if let Ok(_) = self.cur_token().expect_symbol("=") {
            let _ = &self.next_token();
            node = NodeKind::Assign { lhs: node, rhs: self.assign()?, }.wrap();
            return Ok(node);
        }

        return Ok(node);
    }

    fn equality(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node: Option<Box<Node>> = self.relational()?;

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

    fn relational(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node: Option<Box<Node>> = self.add()?;

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

    fn add(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node: Option<Box<Node>> = self.mul()?;

        loop {

            if let Ok(_) = self.cur_token().expect_symbol("+") {
                let _ = &self.next_token();
                //node = NodeKind::Add { lhs: node, rhs: self.mul()?, }.wrap();
                let rhs = self.mul()?;
                node = self.new_add(node, rhs)?;
                continue;
            }

            if let Ok(_) = self.cur_token().expect_symbol("-") {
                let _ = &self.next_token();
                //node = NodeKind::Sub { lhs: node, rhs: self.mul()?, }.wrap();
                let rhs = self.mul()?;
                node = self.new_sub(node, rhs)?;
                continue;
            }

            return Ok(node);
        }
    }

    fn mul(&mut self) -> Result<Option<Box<Node>>, CompileError> {
        let mut node: Option<Box<Node>> = self.unary()?;

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

    fn unary(&mut self) -> Result<Option<Box<Node>>, CompileError> {
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


    fn primary(&mut self) -> Result<Option<Box<Node>>, CompileError> {

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
                    let (offset, v_ty) = self.cur_func_local_variable_offset(&name)?;
                    return Ok(NodeKind::Lvar{ name, offset, ty:v_ty }.wrap());
                }
            }
        }

        let n = self.cur_token().expect_number()?;
        let node = NodeKind::Num { value: n }.wrap();
        let _ = &self.next_token();
        return Ok(node);
    }



    fn func_call(&mut self, name: String) -> Result<Option<Box<Node>>, CompileError> {

        let _ = &self.next_token();

        let mut args: Vec<Option<Box<Node>>> = Vec::new();

        while let Err(_) = self.cur_token().expect_symbol(")") {
            args.push(self.assign()?);

            if let Ok(_) = self.cur_token().expect_symbol(",") {
                let _ = &self.next_token();
            }
        }

        let _ = &self.next_token(); // skip ")"

        Ok(NodeKind::FuncCall { name, args, }.wrap())
    }

    fn new_add(&self, mut l: Option<Box<Node>>, mut r: Option<Box<Node>>) -> Result<Option<Box<Node>>, CompileError> {

        let lb: &mut Box<Node> = l.as_mut().unwrap();
        let lty = (*lb).ty();
        let rb: &mut Box<Node> = r.as_mut().unwrap();
        let rty = (*rb).ty();


        return match (lty, rty) {
            // int + int
            (Ty::Int, Ty::Int) => Ok(NodeKind::Add { lhs: l, rhs: r, }.wrap()),
            // pointer + pointer -> error
            (Ty::Pointer {..}, Ty::Pointer {..}) => Err(CompileError::new(&[&format!("invalid operand in {}", &self.cur_func)])),
            // pointer + int -> pointer + (int * 8)
            (Ty::Pointer {..}, Ty::Int) => Ok(NodeKind::Add { lhs: l, rhs: NodeKind::Mul { lhs: r, rhs: NodeKind::eight(), }.wrap(), }.wrap()),
            // int + pointer -> pointer + (int * 8) (l, r are reverse)
            (Ty::Int, Ty::Pointer {..}) => Ok(NodeKind::Add { lhs: r, rhs: NodeKind::Mul { lhs: l, rhs: NodeKind::eight(), }.wrap(), }.wrap()),
            // not reach here
            //(_, _) => Err(CompileError::new(&[&format!("invalid combination of add in {}", &self.cur_func)])),
        }
    }

    fn new_sub(&self, mut l: Option<Box<Node>>, mut r: Option<Box<Node>>) -> Result<Option<Box<Node>>, CompileError> {

        let lb: &mut Box<Node> = l.as_mut().unwrap();
        let lty = (*lb).ty();
        let rb: &mut Box<Node> = r.as_mut().unwrap();
        let rty = (*rb).ty();


        return match (lty, rty) {
            // int - int
            (Ty::Int, Ty::Int) => Ok(NodeKind::Sub { lhs: l, rhs: r, }.wrap()),
            // pointer - pointer -> calc how many elements between lhs, rhs
            (Ty::Pointer {..}, Ty::Pointer {..}) => {
                let mut node = NodeKind::Sub { lhs: l, rhs: r, }.wrap();
                node.as_mut().unwrap().ty = Some(Ty::Int);
                Ok(NodeKind::Div { lhs: node, rhs: NodeKind::eight(), }.wrap())
            },
            // pointer - int -> pointer - (int * 8)
            (Ty::Pointer {..}, Ty::Int) => Ok(NodeKind::Sub { lhs: l, rhs: NodeKind::Mul { lhs: r, rhs: NodeKind::eight(), }.wrap(), }.wrap()),
            // int - pointer -> error
            (Ty::Int, Ty::Pointer {..}) => Err(CompileError::new(&[&format!("invalid operand in {}", &self.cur_func)])),
            // not reach here
            //(_, _) => Err(CompileError::new(&[&format!("invalid combination of sub in {}", &self.cur_func)])),
        }
    }
}



