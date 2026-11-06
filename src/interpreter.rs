use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    MSPResult, Program, Token,
    ast::{self, BinOp, Block, Expr, For, If, Literal, Stmt, UnaryOp},
    errors::EvalErrors,
};

pub type EnvRef = Rc<RefCell<Env>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Fn {
    pub env: EnvRef,
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Flow {
    Normal,
    Break,
    Continue,
    Return(Value),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Literal(Literal),
    Func(Fn),
    Array(Vec<Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Env {
    pub parent: Option<EnvRef>,
    pub env: HashMap<String, Value>,
}

impl Env {
    pub fn new(parent: Option<EnvRef>) -> EnvRef {
        Rc::new(RefCell::new(Self {
            parent,
            env: HashMap::new(),
        }))
    }

    pub fn store(&mut self, key: String, val: Value) {
        self.env.insert(key, val);
    }

    pub fn assign_existing(&mut self, key: &str, value: Value) -> MSPResult<()> {
        if let Some(slot) = self.env.get_mut(key) {
            *slot = value;
            return Ok(());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign_existing(key, value);
        }

        Err(EvalErrors::VlaueNotFound(key.into()).into())
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        if let Some(v) = self.env.get(key) {
            return Some(v.clone());
        }
        if let Some(p) = &self.parent {
            return p.borrow().get(key);
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct Interpreter {
    pub env: EnvRef,
}

impl Interpreter {
    pub fn new(env: EnvRef) -> Self {
        Self { env }
    }

    pub fn execute_print(&mut self, args: &Vec<Expr>, env: &EnvRef) -> MSPResult<Value> {
        for arg in args {
            let val = self.execute_expr(arg, env)?;
            print!("{:?}", val);
        }
        println!();

        Ok(Value::Literal(Literal::Null))
    }

    pub fn execute_strlen(&mut self, str_exp: &Expr, env: &EnvRef) -> MSPResult<Value> {
        let value = self.execute_expr(str_exp, env)?;
        if let Value::Literal(Literal::Str(s)) = value {
            return Ok(Value::Literal(Literal::Number(s.len() as i64)));
        }
        Err(EvalErrors::InvalidSyntax("strlen need str".into()).into())
    }

    pub fn execute_arrlen(&mut self, arr_val: &Expr, env: &EnvRef) -> MSPResult<Value> {
        let value = self.execute_expr(arr_val, env)?;
        if let Value::Array(arrs) = value {
            return Ok(Value::Literal(Literal::Number(arrs.len() as i64)));
        }
        Err(EvalErrors::InvalidSyntax("arrlen need array".into()).into())
    }

    pub fn execute_append(
        &mut self,
        target: &Expr,
        elements: Vec<Expr>,
        env: &EnvRef,
    ) -> MSPResult<Value> {
        let Expr::Ident(name) = target else {
            return Err(EvalErrors::InvalidSyntax("append needs an array name".into()).into());
        };

        if !matches!(self.execute_expr(target, env)?, Value::Array(_)) {
            return Err(EvalErrors::InvalidSyntax("append needs an array".into()).into());
        }

        let values = elements
            .iter()
            .map(|element| self.execute_expr(element, env))
            .collect::<MSPResult<Vec<Value>>>()?;

        // Arguments can modify the target, so read its current value afterward.
        let Value::Array(mut array) = self.execute_expr(target, env)? else {
            return Err(EvalErrors::InvalidSyntax("append needs an array".into()).into());
        };
        array.extend(values);

        let result = Value::Array(array);
        env.borrow_mut().assign_existing(name, result.clone())?;
        Ok(result)
    }

    pub fn execute(&mut self, program: &Program) -> MSPResult<()> {
        let stmts = program
            .nodes
            .iter()
            .map(|n| n.stmt.clone())
            .collect::<Vec<Stmt>>();

        let env = self.env.clone();
        for stmt in stmts {
            match self.execute_stmt(&stmt, &env)? {
                Flow::Normal => {}
                Flow::Break => {
                    return Err(EvalErrors::InvalidSyntax(format!("break outside of loop")).into());
                }
                Flow::Continue => {
                    return Err(
                        EvalErrors::InvalidSyntax(format!("continue outside of loop")).into(),
                    );
                }
                Flow::Return(_) => {
                    return Err(
                        EvalErrors::InvalidSyntax(format!("return outside of function")).into(),
                    );
                }
            }
        }

        Ok(())
    }

    fn execute_stmts(&mut self, stmts: &Vec<Stmt>, env: &EnvRef) -> MSPResult<Flow> {
        for stmt in stmts {
            match self.execute_stmt(stmt, env)? {
                Flow::Normal => {}
                flow => return Ok(flow),
            }
        }

        Ok(Flow::Normal)
    }

    fn execute_stmt(&mut self, stmt: &Stmt, env: &EnvRef) -> MSPResult<Flow> {
        match stmt {
            Stmt::Break => Ok(Flow::Break),
            Stmt::Continue => Ok(Flow::Continue),
            Stmt::Return { value } => {
                let val = self.execute_return(&value, env)?;
                Ok(Flow::Return(val))
            }

            Stmt::Func(func) => {
                self.execute_func(func, env)?;
                Ok(Flow::Normal)
            }

            Stmt::For(for_stmt) => self.execute_for(for_stmt, &env),
            Stmt::While { cond, body } => self.execute_while(cond, body, &env),
            Stmt::If(if_stmt) => self.execute_if(if_stmt, &env),
            Stmt::Block(block) => self.execute_block(block, &env),
            Stmt::Expr(expr) => {
                self.execute_expr(expr, env)?;
                Ok(Flow::Normal)
            }
        }
    }

    fn execute_return(&mut self, val: &Option<Expr>, env: &EnvRef) -> MSPResult<Value> {
        match val {
            Some(val) => self.execute_expr(val, env),
            None => Ok(Value::Literal(Literal::Null)),
        }
    }

    fn execute_func(&mut self, func: &ast::Fn, env: &EnvRef) -> MSPResult<()> {
        let name = match &func.name {
            Token::Ident(id) => id.clone(),
            _ => return Err(EvalErrors::RequireIdent.into()),
        };

        env.borrow_mut().store(
            name.clone(),
            Value::Func(Fn {
                env: env.clone(),
                name: Token::Ident(name.clone()),
                params: func.params.clone(),
                body: func.body.clone(),
            }),
        );

        Ok(())
    }

    fn execute_block(&mut self, block: &Block, env: &EnvRef) -> MSPResult<Flow> {
        self.execute_stmts(&block.stmts, env)
    }

    fn execute_if(&mut self, if_stmt: &If, env: &EnvRef) -> MSPResult<Flow> {
        let cond_res = self.execute_expr(&if_stmt.cond, env)?;
        let mut value = Flow::Normal;
        match cond_res {
            Value::Literal(Literal::Boolean(b)) => {
                if b {
                    value = self.execute_block(&if_stmt.then, env)?;
                } else {
                    if let Some(ref branch) = if_stmt.else_body {
                        value = self.execute_block(branch, env)?;
                    }
                }

                Ok(value)
            }
            _ => {
                return Err(EvalErrors::ExpectValue(format!("expect true or false")).into());
            }
        }
    }

    fn execute_for(&mut self, for_stmt: &For, env: &EnvRef) -> MSPResult<Flow> {
        let loop_env = Env::new(Some(env.clone()));
        self.execute_expr(&for_stmt.init, &loop_env)?;

        loop {
            let cond_res = self.execute_expr(&for_stmt.cond, &loop_env)?;
            match cond_res {
                Value::Literal(Literal::Boolean(b)) => {
                    let body_env = Env::new(Some(loop_env.clone()));
                    if !b {
                        break;
                    }

                    match self.execute_block(&for_stmt.body, &body_env)? {
                        Flow::Normal | Flow::Continue => {}
                        Flow::Break => break,
                        Flow::Return(value) => return Ok(Flow::Return(value)),
                    };
                    self.execute_expr(&for_stmt.update, &loop_env)?;
                }
                _ => {
                    return Err(EvalErrors::ExpectValue(format!("expect true or false")).into());
                }
            };
        }
        Ok(Flow::Normal)
    }

    fn execute_while(&mut self, cond: &Expr, body: &Block, env: &EnvRef) -> MSPResult<Flow> {
        let while_env = Env::new(Some(env.clone()));
        loop {
            let cond_res = self.execute_expr(cond, &while_env)?;
            if let Value::Literal(Literal::Boolean(b)) = cond_res {
                if b {
                    match self.execute_block(&body, &while_env)? {
                        Flow::Normal | Flow::Continue => {}
                        Flow::Break => break,
                        Flow::Return(value) => return Ok(Flow::Return(value)),
                    }
                } else {
                    return Ok(Flow::Normal);
                }
            } else {
                return Err(EvalErrors::ExpectValue(format!("expect true or false")).into());
            }
        }

        Ok(Flow::Normal)
    }

    fn execute_expr(&mut self, expr: &Expr, env: &EnvRef) -> MSPResult<Value> {
        match expr {
            Expr::Ident(id_name) => {
                let val = env.borrow().get(&id_name);
                match val {
                    Some(v) => Ok(v.clone()),
                    None => {
                        return Err(EvalErrors::VlaueNotFound(format!(
                            "value not found for: {}",
                            id_name
                        ))
                        .into());
                    }
                }
            }
            Expr::ArrLiteral(arr) => {
                let mut res = Vec::new();
                for e in arr {
                    res.push(self.execute_expr(e, env)?);
                }
                Ok(Value::Array(res))
            }
            Expr::Literal(literal) => Ok(Value::Literal(literal.clone())),
            Expr::Assign { target, value } => self.execute_assign(env, target, value),
            Expr::FuncCall { fn_name, params } => self.execute_fn_call(fn_name, params, env),
            Expr::BinaryExpr { op, left, right } => self.execute_binary(op, left, right, env),
            Expr::Unary { op, expr } => self.execute_unary(op, expr, env),
            Expr::Increment { target } => self.execute_increment(env, target),
            Expr::Index { target, index } => self.execute_index(env, target, index),
        }
    }

    fn execute_increment(&mut self, env: &EnvRef, target: &Expr) -> MSPResult<Value> {
        match self.execute_expr(target, env)? {
            Value::Literal(Literal::Number(n)) => {
                self.execute_assign(env, target, &Expr::Literal(Literal::Number(n + 1)))
            }
            _ => Err(
                EvalErrors::InvalidSyntax(format!("increment require target value number")).into(),
            ),
        }
    }

    fn execute_index(&mut self, env: &EnvRef, target: &Expr, idx: &Expr) -> MSPResult<Value> {
        let idx_val = self.execute_expr(idx, env)?;
        let mut idx = 0 as usize;
        if let Value::Literal(Literal::Number(id)) = idx_val {
            idx = id as usize;
        } else {
            return Err(EvalErrors::InvalidSyntax("idx must be number".into()).into());
        }

        match self.execute_expr(target, env)? {
            Value::Array(arr) => {
                if idx >= arr.len() {
                    return Err(EvalErrors::InvalidSyntax("idx outof range".into()).into());
                }

                return Ok(arr[idx].clone());
            }
            Value::Literal(Literal::Str(s)) => {
                if idx >= s.len() {
                    return Err(EvalErrors::InvalidSyntax("idx outof range".into()).into());
                }

                let ch = s.chars().nth(idx);
                return Ok(Value::Literal(Literal::Char(ch.unwrap())));
            }
            _ => Err(
                EvalErrors::InvalidSyntax(format!("increment require target value number")).into(),
            ),
        }
    }

    fn execute_assign(&mut self, env: &EnvRef, target: &Expr, val: &Expr) -> MSPResult<Value> {
        match target {
            Expr::Ident(id) => {
                let v = self.execute_expr(val, env)?;
                env.borrow_mut().store(id.clone(), v.clone());
                Ok(v)
            }
            Expr::Index { target, index } => {
                let name = get_name(target).ok_or_else(|| EvalErrors::RequireIdent)?;
                let arr = env.borrow().get(&name);

                if let Some(Value::Array(arr)) = arr {
                    let idx = self.execute_expr(index, env)?;

                    if let Value::Literal(Literal::Number(n)) = idx {
                        let mut new_arr = arr.clone();
                        new_arr[n as usize] = self.execute_expr(val, env)?;
                        env.borrow_mut().store(name, Value::Array(new_arr.clone()));
                        Ok(Value::Array(new_arr))
                    } else {
                        return Err(EvalErrors::InvalidSyntax(format!(
                            "assign target must be ident or index"
                        ))
                        .into());
                    }
                } else {
                    Err(
                        EvalErrors::InvalidSyntax(format!("assign target must be ident or index"))
                            .into(),
                    )
                }
            }
            _ => Err(
                EvalErrors::InvalidSyntax(format!("assign target must be ident or index")).into(),
            ),
        }
    }

    fn execute_fn_call(
        &mut self,
        fn_name: &Expr,
        params: &Vec<Expr>,
        env: &EnvRef,
    ) -> MSPResult<Value> {
        let name = match fn_name {
            Expr::Ident(v) => v.clone(),
            _ => return Err(EvalErrors::RequireIdent.into()),
        };

        match name.as_str() {
            "print" => return self.execute_print(params, env),
            "strlen" => return self.execute_strlen(&params[0], env),
            "arrlen" => return self.execute_arrlen(&params[0], env),
            "append" => {
                return self.execute_append(&params[0], params[1..].to_vec(), env);
            }
            _ => {}
        }

        let value = env.borrow().get(&name);
        if let Some(Value::Func(func)) = value {
            println!("calling {:?}", func.name);

            let inner_env = Env::new(Some(func.env.clone()));

            for (idx, p) in params.iter().enumerate() {
                let val = self.execute_expr(p, env)?;
                if let Token::Ident(name) = &func.params[idx] {
                    inner_env.borrow_mut().store(name.clone(), val);
                }
            }

            match self.execute_block(&func.body, &inner_env)? {
                Flow::Return(val) => Ok(val),
                _ => Ok(Value::Literal(Literal::Null)),
            }
        } else {
            return Err(EvalErrors::VlaueNotFound(format!("func not found")).into());
        }
    }

    fn execute_binary(
        &mut self,
        op: &BinOp,
        left: &Expr,
        right: &Expr,
        env: &EnvRef,
    ) -> MSPResult<Value> {
        let left_val = self.execute_expr(left, env)?;
        let right_val = self.execute_expr(right, env)?;
        match op {
            BinOp::Plus => {
                let (n1, n2) = match_number(left_val, right_val)?;
                Ok(Value::Literal(Literal::Number(n1 + n2)))
            }
            BinOp::Minus => {
                let (n1, n2) = match_number(left_val, right_val)?;
                Ok(Value::Literal(Literal::Number(n1 - n2)))
            }
            BinOp::Mul => {
                let (n1, n2) = match_number(left_val, right_val)?;
                Ok(Value::Literal(Literal::Number(n1 * n2)))
            }
            BinOp::Div => {
                let (n1, n2) = match_number(left_val, right_val)?;
                if n2 == 0 {
                    return Err(EvalErrors::InvalidSyntax("divisor is zero".into()).into());
                }
                Ok(Value::Literal(Literal::Number(n1 / n2)))
            }
            BinOp::Mod => {
                let (n1, n2) = match_number(left_val, right_val)?;
                Ok(Value::Literal(Literal::Number(n1 % n2)))
            }
            BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq | BinOp::Eq | BinOp::NotEq => {
                let order = compare_values(&left_val, &right_val)?;
                let result = match op {
                    BinOp::Lt => order.is_lt(),
                    BinOp::LtEq => order.is_le(),
                    BinOp::Gt => order.is_gt(),
                    BinOp::GtEq => order.is_ge(),
                    BinOp::Eq => order.is_eq(),
                    BinOp::NotEq => !order.is_eq(),
                    _ => unreachable!(),
                };
                Ok(Value::Literal(Literal::Boolean(result)))
            }
            BinOp::And => {
                let (t1, t2) = match_truth(left_val, right_val)?;
                Ok(Value::Literal(Literal::Boolean(t1 && t2)))
            }
            BinOp::Or => {
                let (t1, t2) = match_truth(left_val, right_val)?;
                Ok(Value::Literal(Literal::Boolean(t1 || t2)))
            }
        }
    }

    fn execute_unary(&mut self, op: &UnaryOp, exp: &Expr, env: &EnvRef) -> MSPResult<Value> {
        let val = self.execute_expr(exp, env)?;
        match op {
            UnaryOp::Plus => {
                if let Value::Literal(Literal::Number(n1)) = val {
                    let n = n1 + 1;
                    return Ok(Value::Literal(Literal::Number(n)));
                } else {
                    Err(EvalErrors::InvalidSyntax(format!("plus require number")).into())
                }
            }
            UnaryOp::Minus => {
                if let Value::Literal(Literal::Number(n1)) = val {
                    let n = n1 - 1;
                    return Ok(Value::Literal(Literal::Number(n)));
                } else {
                    Err(EvalErrors::InvalidSyntax(format!("plus require number")).into())
                }
            }
            UnaryOp::Bang => {
                if let Value::Literal(Literal::Boolean(t)) = val {
                    return Ok(Value::Literal(Literal::Boolean(!t)));
                } else {
                    Err(EvalErrors::InvalidSyntax(format!("bang require true or false")).into())
                }
            }
        }
    }
}

fn match_number(exp1: Value, exp2: Value) -> MSPResult<(i64, i64)> {
    if let Value::Literal(Literal::Number(n1)) = exp1
        && let Value::Literal(Literal::Number(n2)) = exp2
    {
        return Ok((n1, n2));
    } else {
        Err(EvalErrors::InvalidSyntax(format!("plus require number")).into())
    }
}

fn compare_values(left: &Value, right: &Value) -> MSPResult<std::cmp::Ordering> {
    match (left, right) {
        (Value::Literal(Literal::Number(a)), Value::Literal(Literal::Number(b))) => Ok(a.cmp(b)),
        (Value::Literal(Literal::Str(a)), Value::Literal(Literal::Str(b))) => Ok(a.cmp(b)),
        (Value::Literal(Literal::Char(a)), Value::Literal(Literal::Char(b))) => Ok(a.cmp(b)),
        (Value::Literal(Literal::Char(a)), Value::Literal(Literal::Str(b))) => {
            Ok(a.to_string().cmp(b))
        }
        (Value::Literal(Literal::Str(a)), Value::Literal(Literal::Char(b))) => {
            Ok(a.cmp(&b.to_string()))
        }
        _ => Err(EvalErrors::InvalidSyntax(
            "comparison requires two numbers or two text values (string or char)".into(),
        )
        .into()),
    }
}

fn match_truth(exp1: Value, exp2: Value) -> MSPResult<(bool, bool)> {
    if let Value::Literal(Literal::Boolean(t1)) = exp1
        && let Value::Literal(Literal::Boolean(t2)) = exp2
    {
        return Ok((t1, t2));
    } else {
        Err(EvalErrors::InvalidSyntax(format!("require true or false")).into())
    }
}

fn get_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Ident(id) => Some(id.clone()),
        _ => None,
    }
}
