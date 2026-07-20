#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, start: u32, end: u32) -> Self {
        Self {
            kind,
            span: Span { start, end },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl From<Span> for miette::SourceSpan {
    fn from(value: Span) -> Self {
        Self::new(
            (value.start as usize).into(),
            #[allow(clippy::useless_conversion)]
            ((value.end - value.start) as usize).into(),
        )
    }
}

pub fn from_spans(span1: Span, span2: Span) -> miette::SourceSpan {
    miette::SourceSpan::new(
        (span1.start as usize).into(),
        #[allow(clippy::useless_conversion)]
        ((span2.end - span1.start) as usize).into(),
    )
}

/*---------------------------------------------------------------------------------------------------*/

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    String,
    Ident,
    Number,
    //TODO: add %
    Char,
    Operator(Operator),
    Keyword(Keyword),
    Punctuation(Punctuation),
    Delimiter(Delimiter),
    Unknown,
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Boolean {
    True,
    False,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    Type(Ty),
    Boolean(Boolean),
    This,
    If,
    Else,
    While,
    For,
    Proc,
    Seed,
    Get,
    From,
    In,
    Range,
    Methods,
    Require,
    Aor,
    Packing,
    Api,
    Also,
    Break,
    Continue,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ty {
    Signed(BitWidth),
    Unsigned(BitWidth),
    Float32,
    Float64,
    Arr,
    HeapArr,
    String,
    HeapString,
    Char,
    Bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BitWidth {
    Byte,
    Sixteen,
    ThirtyTwo,
    SixtyFour,
    Word,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operator {
    BitwiseAnd,
    BitwiseOr,
    Not,
    Assign,
    Plus,
    Minus,
    Star,
    ForwardSlash,
    Dot,
    Comparision(ComparisionOperator),
    Logical(LogicalOperator),
    CompoundAssign(CompoundAssignOperator),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Punctuation {
    Semicolon,
    Colon,
    Comma,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Delimiter {
    ParenOpen,
    ParenClose,
    SquareOpen,
    SquareClose,
    CurlyOpen,
    CurlyClose,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ComparisionOperator {
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    Equal,
    NotEqual,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompoundAssignOperator {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
}

/*---------------------------------------------------------------------------------------------------*/

impl<'a> TryFrom<&'a str> for Keyword {
    type Error = ();
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let k = match value {
            "i8" | "i16" | "i32" | "i64" => Self::Type(Ty::Signed(
                BitWidth::try_from(value[1..].parse::<u8>().unwrap()).unwrap(),
            )),
            "u8" | "u16" | "u32" | "u64" => Self::Type(Ty::Unsigned(
                BitWidth::try_from(value[1..].parse::<u8>().unwrap()).unwrap(),
            )),
            "true" => Self::Boolean(Boolean::True),
            "false" => Self::Boolean(Boolean::False),
            "isize" => Self::Type(Ty::Signed(BitWidth::Word)),
            "usize" => Self::Type(Ty::Unsigned(BitWidth::Word)),
            "f32" => Self::Type(Ty::Float32),
            "f64" => Self::Type(Ty::Float64),
            "this" => Self::This,
            "if" => Self::If,
            "else" => Self::Else,
            "while" => Self::While,
            "for" => Self::For,
            "proc" => Self::Proc,
            "seed" => Self::Seed,
            "get" => Self::Get,
            "from" => Self::From,
            "in" => Self::In,
            "range" => Self::Range,
            "arr" => Self::Type(Ty::Arr),
            "heaparr" => Self::Type(Ty::HeapArr),
            "string" => Self::Type(Ty::String),
            "heapstring" => Self::Type(Ty::HeapString),
            "char" => Self::Type(Ty::Char),
            "methods" => Self::Methods,
            "require" => Self::Require,
            "aor" => Self::Aor,
            "packing" => Self::Packing,
            "api" => Self::Api,
            "also" => Self::Also,
            "break" => Self::Break,
            "continue" => Self::Continue,
            _ => return Err(()),
        };
        Ok(k)
    }
}

impl TryFrom<u8> for BitWidth {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let b = match value {
            8 => BitWidth::Byte,
            16 => BitWidth::Sixteen,
            32 => BitWidth::ThirtyTwo,
            64 => BitWidth::SixtyFour,
            _ => return Err(value),
        };
        Ok(b)
    }
}

impl Operator {
    pub fn suffix_equal(self) -> Option<Operator> {
        let new_op = match self {
            Operator::Not => Operator::Comparision(ComparisionOperator::NotEqual),
            Operator::Assign => Operator::Comparision(ComparisionOperator::Equal),
            Operator::Comparision(ComparisionOperator::LessThan) => {
                Operator::Comparision(ComparisionOperator::LessEqual)
            }
            Operator::Comparision(ComparisionOperator::GreaterThan) => {
                Operator::Comparision(ComparisionOperator::GreaterEqual)
            }
            _ => return None,
        };
        Some(new_op)
    }
}

impl TryFrom<char> for Operator {
    type Error = ();
    fn try_from(value: char) -> Result<Self, Self::Error> {
        let op = match value {
            '&' => Operator::BitwiseAnd,
            '|' => Operator::BitwiseOr,
            '!' => Operator::Not,
            '=' => Operator::Assign,
            '+' => Operator::Plus,
            '-' => Operator::Minus,
            '*' => Operator::Star,
            '/' => Operator::ForwardSlash,
            '<' => Operator::Comparision(ComparisionOperator::LessThan),
            '>' => Operator::Comparision(ComparisionOperator::GreaterThan),
            _ => return Err(()),
        };
        Ok(op)
    }
}

impl TryFrom<char> for Punctuation {
    type Error = ();
    fn try_from(value: char) -> Result<Self, Self::Error> {
        let punct = match value {
            ';' => Punctuation::Semicolon,
            ':' => Punctuation::Colon,
            ',' => Punctuation::Comma,
            _ => return Err(()),
        };
        Ok(punct)
    }
}

impl TryFrom<char> for Delimiter {
    type Error = char;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        let a = match value {
            '(' => Self::ParenOpen,
            ')' => Self::ParenClose,
            '[' => Self::SquareOpen,
            ']' => Self::SquareClose,
            '{' => Self::CurlyOpen,
            '}' => Self::CurlyClose,
            _ => return Err(value),
        };
        Ok(a)
    }
}
impl TryFrom<Operator> for LogicalOperator {
    type Error = ();

    fn try_from(value: Operator) -> Result<Self, Self::Error> {
        let lop = match value {
            Operator::BitwiseOr => Self::Or,
            Operator::BitwiseAnd => Self::And,
            _ => return Err(()),
        };
        Ok(lop)
    }
}

impl TryFrom<Operator> for CompoundAssignOperator {
    type Error = ();
    fn try_from(value: Operator) -> Result<Self, Self::Error> {
        let caop = match value {
            Operator::BitwiseAnd => Self::And,
            Operator::BitwiseOr => Self::Or,
            Operator::Plus => Self::Add,
            Operator::Minus => Self::Sub,
            Operator::Star => Self::Mul,
            Operator::ForwardSlash => Self::Div,
            _ => return Err(()),
        };
        Ok(caop)
    }
}

/*---------------------------------------------------------------------------------------------------*/

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::String => f.write_str("string literal"),
            TokenKind::Ident => f.write_str("identifier"),
            TokenKind::Number => f.write_str("number"),
            TokenKind::Char => f.write_str("char"),
            TokenKind::Operator(operator) => write!(f, "{operator}"),
            TokenKind::Keyword(keyword) => write!(f, "{keyword}"),
            TokenKind::Punctuation(punctuation) => write!(f, "{punctuation}"),
            TokenKind::Delimiter(delimiter) => write!(f, "{delimiter}"),
            TokenKind::Unknown => write!(f, "Unknown character"),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

impl std::fmt::Display for Boolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Boolean::True => "true",
            Boolean::False => "false",
        };
        write!(f, "{s}")
    }
}

impl std::fmt::Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Ty::Signed(bit_width) => return write!(f, "i{bit_width}"),
            Ty::Unsigned(bit_width) => return write!(f, "u{bit_width}"),
            Ty::Float32 => "f32",
            Ty::Float64 => "f64",
            Ty::Arr => "arr",
            Ty::HeapArr => "heaparr",
            Ty::String => "string",
            Ty::HeapString => "heapstring",
            Ty::Char => "char",
            Ty::Bool => "bool",
        };
        write!(f, "{s}")
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Keyword::Type(ty) => return write!(f, "{ty}"),
            Keyword::This => "this",
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::While => "while",
            Keyword::For => "for",
            Keyword::Proc => "proc",
            Keyword::Seed => "seed",
            Keyword::Get => "get",
            Keyword::From => "from",
            Keyword::In => "in",
            Keyword::Range => "range",
            Keyword::Methods => "methods",
            Keyword::Require => "require",
            Keyword::Aor => "aor",
            Keyword::Packing => "packing",
            Keyword::Api => "api",
            Keyword::Also => "also",
            Keyword::Break => "break",
            Keyword::Continue => "continue",
            Keyword::Boolean(boolean) => return write!(f, "{boolean}"),
        };
        write!(f, "{s}")
    }
}

impl std::fmt::Display for BitWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            BitWidth::Byte => "8",
            BitWidth::Sixteen => "16",
            BitWidth::ThirtyTwo => "32",
            BitWidth::SixtyFour => "64",
            //TODO: word depends on the machine
            BitWidth::Word => "64",
        };
        write!(f, "{s}")
    }
}
impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Operator::BitwiseAnd => "&",
            Operator::BitwiseOr => "|",
            Operator::Not => "!",
            Operator::Assign => "=",
            Operator::Plus => "+",
            Operator::Minus => "-",
            Operator::Star => "*",
            Operator::ForwardSlash => "/",
            Operator::Dot => ".",
            Operator::Comparision(comparision_operator) => {
                return write!(f, "{comparision_operator}");
            }
            Operator::Logical(logical_operator) => {
                return write!(f, "{logical_operator}");
            }
            Operator::CompoundAssign(compound_assign_operator) => {
                return write!(f, "{compound_assign_operator}");
            }
        };
        write!(f, "{s}")
    }
}

impl std::fmt::Display for ComparisionOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ComparisionOperator::LessThan => "<",
            ComparisionOperator::GreaterThan => ">",
            ComparisionOperator::LessEqual => "<=",
            ComparisionOperator::GreaterEqual => ">=",
            ComparisionOperator::Equal => "==",
            ComparisionOperator::NotEqual => "!=",
        };
        write!(f, "{s}")
    }
}

impl std::fmt::Display for LogicalOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogicalOperator::And => "&&",
            LogicalOperator::Or => "||",
        };
        write!(f, "{s}")
    }
}

impl std::fmt::Display for CompoundAssignOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CompoundAssignOperator::Add => "+=",
            CompoundAssignOperator::Sub => "-=",
            CompoundAssignOperator::Mul => "*=",
            CompoundAssignOperator::Div => "/=",
            CompoundAssignOperator::And => "&=",
            CompoundAssignOperator::Or => "|=",
        };
        write!(f, "{s}")
    }
}
impl std::fmt::Display for Punctuation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Punctuation::Semicolon => ";",
            Punctuation::Colon => ":",
            Punctuation::Comma => ",",
        };
        write!(f, "{s}")
    }
}
impl std::fmt::Display for Delimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Delimiter::ParenOpen => "(",
            Delimiter::ParenClose => ")",
            Delimiter::SquareOpen => "[",
            Delimiter::SquareClose => "]",
            Delimiter::CurlyOpen => "{",
            Delimiter::CurlyClose => "}",
        };
        write!(f, "{s}")
    }
}
