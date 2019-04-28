use super::ast::*;
use super::bytecode::*;

pub struct Compiler {
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {}
    }

    pub fn compile_stmt(&mut self, chunk: &mut Chunk, stmt: &Stmt) {
        match stmt {
            Stmt::Expression(expr) =>  {
                self.compile_expr(chunk, expr);
                chunk.add_instruction(Instruction::Pop);
            },
            Stmt::Print(ref expr) => {
                self.compile_expr(chunk, expr);
                chunk.add_instruction(Instruction::Print);
            },
            Stmt::Var(ref identifier, ref expr) => self.compile_var_stmt(chunk, identifier, expr),
            _ => unimplemented!(),
        }
    }

    fn compile_var_stmt(&mut self, chunk: &mut Chunk, identifier: &str, expr: &Option<Box<Expr>>) {
        if let Some(expr) = expr {
            self.compile_expr(chunk, expr);
        } else {
            self.compile_nil(chunk);
        }
        
        let constant = chunk.add_str_constant(identifier);
        chunk.add_instruction(Instruction::DefineGlobal(constant));
    }

    fn compile_expr(&mut self, chunk: &mut Chunk, expr: &Expr) {
        match *expr {
            Expr::Number(num) => self.compile_number(chunk, num),
            Expr::Boolean(state) => self.compile_bool(chunk, state),
            Expr::Nil => self.compile_nil(chunk),
            Expr::String(ref string) => self.compile_string(chunk, string),
            Expr::Binary(ref left, operator, ref right) => self.compile_binary(chunk, left, right, operator),
            Expr::Unary(operator, ref left) => self.compile_unary(chunk, left, operator),
            Expr::Variable(ref identifier) => self.compile_variable(chunk, identifier),
            Expr::Assign(ref identifier, ref expr) => self.compile_assign(chunk, identifier, expr),
            _ => unimplemented!(),
        }
    }

    fn compile_variable(&mut self, chunk: &mut Chunk, identifier: &str) {
        let constant = chunk.add_str_constant(identifier);
        chunk.add_instruction(Instruction::GetGlobal(constant));
    }

    fn compile_assign(&mut self, chunk: &mut Chunk, identifier: &str, expr: &Expr) {
        self.compile_expr(chunk, expr);
        let constant = chunk.add_str_constant(identifier);
        chunk.add_instruction(Instruction::SetGlobal(constant));
    }

    fn compile_string(&mut self, chunk: &mut Chunk, string: &str) {
        let constant = chunk.add_str_constant(string);
        chunk.add_instruction(Instruction::Constant(constant));
    }

    fn compile_nil(&mut self, chunk: &mut Chunk) {
        chunk.add_instruction(Instruction::Nil);
    }

    fn compile_bool(&mut self, chunk: &mut Chunk, state: bool) {
        let instruction = if state {
            Instruction::True
        } else {
            Instruction::False
        };
        chunk.add_instruction(instruction);
    }

    fn compile_number(&mut self, chunk: &mut Chunk, num: f64) {
        let constant = chunk.add_constant(Constant::Number(num));
        chunk.add_instruction(Instruction::Constant(constant));
    }

    fn compile_binary(&mut self, chunk: &mut Chunk, left: &Expr, right: &Expr, operator: BinaryOperator) {
        self.compile_expr(chunk, left);
        self.compile_expr(chunk, right);
        match operator {
            BinaryOperator::Plus => chunk.add_instruction(Instruction::Add),
            BinaryOperator::Minus => chunk.add_instruction(Instruction::Subtract),
            BinaryOperator::Star => chunk.add_instruction(Instruction::Multiply),
            BinaryOperator::Slash => chunk.add_instruction(Instruction::Divide),
            BinaryOperator::BangEqual => {
                chunk.add_two_instructions(Instruction::Equal, Instruction::Not)
            }
            BinaryOperator::EqualEqual => chunk.add_instruction(Instruction::Equal),
            BinaryOperator::Greater => chunk.add_instruction(Instruction::Greater),
            BinaryOperator::GreaterEqual => {
                chunk.add_two_instructions(Instruction::Less, Instruction::Not)
            }
            BinaryOperator::Less => chunk.add_instruction(Instruction::Less),
            BinaryOperator::LessEqual => {
                chunk.add_two_instructions(Instruction::Greater, Instruction::Not)
            }
        };
    }

    fn compile_unary(&mut self, chunk: &mut Chunk, left: &Expr, operator: UnaryOperator) {
        self.compile_expr(chunk, left);
        match operator {
            UnaryOperator::Minus => chunk.add_instruction(Instruction::Negate),
            UnaryOperator::Bang => chunk.add_instruction(Instruction::Not),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::super::tokenizer::*;
    use super::*;

    fn compile_expr_to_chunk(expr: &Expr) -> Chunk {
        let mut chunk = Chunk::new();
        let mut compiler = Compiler::new();
        compiler.compile_expr(&mut chunk, expr);
        chunk
    }

    fn compile_stmt_to_chunk(stmts: &Vec<Stmt>) -> Chunk {
        let mut chunk = Chunk::new();
        let mut compiler = Compiler::new();
        for stmt in stmts {
            compiler.compile_stmt(&mut chunk, stmt);
        }
        chunk
    }

    fn parse_expr_string(data: &str) -> Result<Expr, String> {
        let tokens = tokenize(data);
        let mut it = tokens.as_slice().into_iter().peekable();
        super::super::expr_parser::parse(&mut it)
    }

    fn parse_stmt_string(data: &str) -> Result<Vec<Stmt>, String> {
        let tokens = tokenize(data);
        let mut it = tokens.as_slice().into_iter().peekable();
        super::super::stmt_parser::parse(&mut it)
    }

    fn assert_expr_chunk(data: &str, instructions: Vec<Instruction>, constants: Vec<Constant>) {
        let chunk = compile_expr_to_chunk(&parse_expr_string(data).unwrap());
        assert_eq!(instructions, chunk.instructions());
        assert_eq!(constants, chunk.constants());
    }

    fn assert_stmt_chunk(data: &str, instructions: Vec<Instruction>, constants: Vec<Constant>) {
        let chunk = compile_stmt_to_chunk(&parse_stmt_string(data).unwrap());
        assert_eq!(instructions, chunk.instructions());
        assert_eq!(constants, chunk.constants());
    }

    #[test]
    fn test_stmt_print() {
        assert_stmt_chunk(
            "print 3;", 
            vec![Instruction::Constant(0), Instruction::Print], 
            vec![Constant::Number(3.0)]
        );
    }

    #[test]
    fn test_stmt_expr() {
        assert_stmt_chunk(
            "3;", 
            vec![Instruction::Constant(0), Instruction::Pop], 
            vec![Constant::Number(3.0)]
        );
    }

    #[test]
    fn test_stmt_global_var_expr() {
        assert_stmt_chunk(
            "var x = 3;", 
            vec![Instruction::Constant(0), Instruction::DefineGlobal(1)], 
            vec![Constant::Number(3.0), Constant::String("x".into())]
        );
    }

    #[test]
    fn test_stmt_global_var_complex_expr() {
        assert_stmt_chunk(
            "var x = 2+3;", 
            vec![Instruction::Constant(0), Instruction::Constant(1), Instruction::Add, Instruction::DefineGlobal(2)], 
            vec![Constant::Number(2.0), Constant::Number(3.0), Constant::String("x".into())]
        );
    }

    #[test]
    fn test_stmt_global_var_nil() {
        assert_stmt_chunk(
            "var x;", 
            vec![Instruction::Nil, Instruction::DefineGlobal(0)], 
            vec![Constant::String("x".into())]
        );
    }

    #[test]
    fn test_stmt_global_get() {
        assert_stmt_chunk(
            "x;", 
            vec![Instruction::GetGlobal(0), Instruction::Pop], 
            vec![Constant::String("x".into())]
        );
    }

    #[test]
    fn test_stmt_global_set() {
        assert_stmt_chunk(
            "x=3;", 
            vec![Instruction::Constant(0), Instruction::SetGlobal(1), Instruction::Pop], 
            vec![Constant::Number(3.0), Constant::String("x".into())]
        );
    }

    #[test]
    fn test_number() {
        assert_expr_chunk(
            "1",
            vec![Instruction::Constant(0)],
            vec![Constant::Number(1.0)],
        );
    }

    #[test]
    fn test_bool() {
        assert_expr_chunk("true", vec![Instruction::True], vec![]);
        assert_expr_chunk("false", vec![Instruction::False], vec![]);
    }
    #[test]
    fn test_nil() {
        assert_expr_chunk("nil", vec![Instruction::Nil], vec![]);
    }

    #[test]
    fn test_string() {
        assert_expr_chunk(
            "\"\"",
            vec![Instruction::Constant(0)],
            vec![Constant::String("".into())],
        );
        assert_expr_chunk(
            "\"test\"",
            vec![Instruction::Constant(0)],
            vec![Constant::String("test".into())],
        );
        assert_expr_chunk(
            "\"te\"+\"st\"",
            vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Add,
            ],
            vec![Constant::String("te".into()), Constant::String("st".into())],
        );
    }

    #[test]
    fn test_binary() {
        assert_expr_chunk(
            "1+2",
            vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Add,
            ],
            vec![Constant::Number(1.0), Constant::Number(2.0)],
        );
        assert_expr_chunk(
            "1-2",
            vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Subtract,
            ],
            vec![Constant::Number(1.0), Constant::Number(2.0)],
        );
        assert_expr_chunk(
            "1*2",
            vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Multiply,
            ],
            vec![Constant::Number(1.0), Constant::Number(2.0)],
        );
        assert_expr_chunk(
            "1/2",
            vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Divide,
            ],
            vec![Constant::Number(1.0), Constant::Number(2.0)],
        );
    }

    #[test]
    fn test_binary_logic() {
        assert_expr_chunk(
            "true != false",
            vec![
                Instruction::True,
                Instruction::False,
                Instruction::Equal,
                Instruction::Not,
            ],
            vec![],
        );
        assert_expr_chunk(
            "true == false",
            vec![Instruction::True, Instruction::False, Instruction::Equal],
            vec![],
        );

        assert_expr_chunk(
            "true > false",
            vec![Instruction::True, Instruction::False, Instruction::Greater],
            vec![],
        );

        assert_expr_chunk(
            "true >= false",
            vec![
                Instruction::True,
                Instruction::False,
                Instruction::Less,
                Instruction::Not,
            ],
            vec![],
        );

        assert_expr_chunk(
            "true < false",
            vec![Instruction::True, Instruction::False, Instruction::Less],
            vec![],
        );

        assert_expr_chunk(
            "true <= false",
            vec![
                Instruction::True,
                Instruction::False,
                Instruction::Greater,
                Instruction::Not,
            ],
            vec![],
        );
    }

    #[test]
    fn test_unary() {
        assert_expr_chunk(
            "-1",
            vec![Instruction::Constant(0), Instruction::Negate],
            vec![Constant::Number(1.0)],
        );
        assert_expr_chunk("!false", vec![Instruction::False, Instruction::Not], vec![]);
    }
}
