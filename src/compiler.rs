use super::ast::*;
use super::bytecode::*;

pub fn compile(expr: &Expr) -> Chunk {
    let mut chunk = Chunk::new();
    compile_expr(&mut chunk, expr);
    chunk
}

fn compile_expr(chunk: &mut Chunk, expr: &Expr) {
    match expr {
        &Expr::Number(num) => compile_number(chunk, num),
        &Expr::Binary(ref left, BinaryOperator::Plus, ref right) => compile_add(chunk, left, right),
        &Expr::Unary(UnaryOperator::Minus, ref left) => compile_negate(chunk, left),
        _ => unimplemented!()
    }
}

fn compile_number(chunk: &mut Chunk, num: f64) {
    let index = chunk.add_constant(Constant::Number(num));
    chunk.add_instruction(Instruction::Constant(index));
}

fn compile_add(chunk: &mut Chunk, left: &Box<Expr>, right: &Box<Expr>) {
    compile_expr(chunk, left);
    compile_expr(chunk, right);
    chunk.add_instruction(Instruction::Add);
}

fn compile_negate(chunk: &mut Chunk, left: &Box<Expr>) {
    compile_expr(chunk, left);
    chunk.add_instruction(Instruction::Negate);
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tokenizer::*;
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
        assert_chunk("1", vec![Instruction::Constant(0)], vec![Constant::Number(1.0)]);
    }

    #[test]
    fn test_add() {
        assert_chunk(
            "1+2", 
            vec![
                Instruction::Constant(0), 
                Instruction::Constant(1), 
                Instruction::Add], 
            vec![Constant::Number(1.0), Constant::Number(2.0)]
        );
    }
    
    #[test]
    fn test_negate() {
        assert_chunk(
            "-1", 
            vec![Instruction::Constant(0), Instruction::Negate], 
            vec![Constant::Number(1.0)]
        );
    }
}