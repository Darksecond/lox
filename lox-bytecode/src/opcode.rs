pub const CONSTANT     : u8 = 0;
pub const TRUE         : u8 = 1;
pub const FALSE        : u8 = 2;
pub const NIL          : u8 = 3;

pub const NEGATE       : u8 = 4;
pub const ADD          : u8 = 5;
pub const SUBTRACT     : u8 = 6;
pub const MULTIPLY     : u8 = 7;
pub const DIVIDE       : u8 = 8;

pub const NOT          : u8 = 9;
pub const EQUAL        : u8 = 10;
pub const GREATER      : u8 = 11;
pub const LESS         : u8 = 12;

pub const POP          : u8 = 13;

pub const RETURN       : u8 = 14;
pub const PRINT        : u8 = 15;

pub const DEFINE_GLOBAL: u8 = 16;
pub const GET_GLOBAL   : u8 = 17;
pub const SET_GLOBAL   : u8 = 18;
pub const GET_LOCAL    : u8 = 19;
pub const SET_LOCAL    : u8 = 20;
pub const GET_UPVALUE  : u8 = 21;
pub const SET_UPVALUE  : u8 = 22;
pub const SET_PROPERTY : u8 = 23;
pub const GET_PROPERTY : u8 = 24;

pub const JUMP         : u8 = 25;
pub const JUMP_IF_FALSE: u8 = 26;
pub const CALL         : u8 = 27;
pub const INVOKE       : u8 = 28;
pub const CLOSE_UPVALUE: u8 = 29;

pub const CLASS        : u8 = 30;
pub const CLOSURE      : u8 = 31;
pub const METHOD       : u8 = 32;

pub const IMPORT       : u8 = 33;
pub const IMPORT_GLOBAL: u8 = 34;
