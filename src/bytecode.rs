pub enum Object {
    String(String),
}
pub enum Value {
    Number(f64),
    Boolean(bool),
    Nil,
    Object(Object),
}

pub enum Operator {
    Constant(Value),
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
}