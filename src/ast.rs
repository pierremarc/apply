pub enum Num {
    Integer(i64),
    Float(f64),
}

pub enum Literal {
    Number(Num),
    String(String),
    Boolean(bool),
}

pub enum Keyword {
    //
    Map,
    Srid,
    Extent,
    Layer,
    Data,
    Sym,

    //
    Fill,
    Stroke,
    Pattern,
    Label,
}

pub enum Operator {
    Eq,
    Lt,
    Lte,
    Gt,
    Gte,
}

pub enum Symbol {
    Or,
    Then,
    Pipe,
    Ident(String),
}

pub type Args = Vec<Literal>;

pub struct Cell {
    pub get: Box<dyn Fn(Args) -> Literal>,
}

pub enum Function {
    Apply(Args),
    Retain(Vec<Cell>),
    Compose(Vec<Function>),
}

pub enum Node {
    Lit(Literal),
    Key(Keyword),
    Op(Operator),
    Sym(Symbol),
    Fn(Function),
    Comment(String),
}

pub struct Expression {
    pub nodes: Vec<Node>,
}

pub struct Block {
    pub expr: Vec<Expression>,
}
