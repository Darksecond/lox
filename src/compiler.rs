use super::ast::*;
use super::bytecode::*;

pub fn compile_expr(chunk: &mut Chunk, expr: &Expr) {
    match *expr {
        Expr::Number(num) => compile_number(chunk, num),
        Expr::Boolean(state) => compile_bool(chunk, state),
        Expr::Nil => compile_nil(chunk),
        Expr::String(ref string) => compile_string(chunk, string),
        Expr::Binary(ref left, operator, ref right) => compile_binary(chunk, left, right, operator),
        Expr::Unary(operator, ref left) => compile_unary(chunk, left, operator),
        _ => unimplemented!(),
    }
}

fn compile_string(chunk: &mut Chunk, string: &str) {
    let constant = chunk.add_constant(Constant::String(String::from(string)));
    chunk.add_instruction(Instruction::Constant(constant));
}

fn compile_nil(chunk: &mut Chunk) {
    chunk.add_instruction(Instruction::Nil);
}

fn compile_bool(chunk: &mut Chunk, state: bool) {
    let instruction = if state {
        Instruction::True
    } else {
        Instruction::False
    };
    chunk.add_instruction(instruction);
}

fn compile_number(chunk: &mut Chunk, num: f64) {
    let constant = chunk.add_constant(Constant::Number(num));
    chunk.add_instruction(Instruction::Constant(constant));
}

fn compile_binary(chunk: &mut Chunk, left: &Expr, right: &Expr, operator: BinaryOperator) {
    compile_expr(chunk, left);
    compile_expr(chunk, right);
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

fn compile_unary(chunk: &mut Chunk, left: &Expr, operator: UnaryOperator) {
    compile_expr(chunk, left);
    match operator {
        UnaryOperator::Minus => chunk.add_instruction(Instruction::Negate),
        UnaryOperator::Bang => chunk.add_instruction(Instruction::Not),
    };
}

#[cfg(test)]
mod tests {
    use super::super::tokenizer::*;
    use super::*;

    fn compile(expr: &Expr) -> Chunk {
        let mut chunk = Chunk::new();
        compile_expr(&mut chunk, expr);
        chunk
    }

    fn parse_str(data: &str) -> Result<Expr, String> {
        let tokens = tokenize(data);
        let mut it = tokens.as_slice().into_iter().peekable();
        super::super::expr_parser::parse(&mut it)
    }

    fn assert_chunk(data: &str, instructions: Vec<Instruction>, constants: Vec<Constant>) {
        let chunk = compile(&parse_str(data).unwrap());
        assert_eq!(instructions, chunk.instructions());
        assert_eq!(constants, chunk.constants());
    }

    #[test]
    fn test_number() {
        assert_chunk(
            "1",
            vec![Instruction::Constant(0)],
            vec![Constant::Number(1.0)],
        );
    }

    #[test]
    fn test_bool() {
        assert_chunk("true", vec![Instruction::True], vec![]);
        assert_chunk("false", vec![Instruction::False], vec![]);
    }
    #[test]
    fn test_nil() {
        assert_chunk("nil", vec![Instruction::Nil], vec![]);
    }

    #[test]
    fn test_string() {
        assert_chunk(
            "\"\"",
            vec![Instruction::Constant(0)],
            vec![Constant::String("".into())],
        );
        assert_chunk(
            "\"test\"",
            vec![Instruction::Constant(0)],
            vec![Constant::String("test".into())],
        );
        assert_chunk(
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
        assert_chunk(
            "1+2",
            vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Add,
            ],
            vec![Constant::Number(1.0), Constant::Number(2.0)],
        );
        assert_chunk(
            "1-2",
            vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Subtract,
            ],
            vec![Constant::Number(1.0), Constant::Number(2.0)],
        );
        assert_chunk(
            "1*2",
            vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Multiply,
            ],
            vec![Constant::Number(1.0), Constant::Number(2.0)],
        );
        assert_chunk(
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
        assert_chunk(
            "true != false",
            vec![
                Instruction::True,
                Instruction::False,
                Instruction::Equal,
                Instruction::Not,
            ],
            vec![],
        );
        assert_chunk(
            "true == false",
            vec![Instruction::True, Instruction::False, Instruction::Equal],
            vec![],
        );

        assert_chunk(
            "true > false",
            vec![Instruction::True, Instruction::False, Instruction::Greater],
            vec![],
        );

        assert_chunk(
            "true >= false",
            vec![
                Instruction::True,
                Instruction::False,
                Instruction::Less,
                Instruction::Not,
            ],
            vec![],
        );

        assert_chunk(
            "true < false",
            vec![Instruction::True, Instruction::False, Instruction::Less],
            vec![],
        );

        assert_chunk(
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
        assert_chunk(
            "-1",
            vec![Instruction::Constant(0), Instruction::Negate],
            vec![Constant::Number(1.0)],
        );
        assert_chunk("!false", vec![Instruction::False, Instruction::Not], vec![]);
    }
}
