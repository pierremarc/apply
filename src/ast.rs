#[derive(Clone)]
pub enum Num {
    Integer(i64),
    Float(f64),
}

#[derive(Clone)]
pub enum Literal {
    Number(Num),
    String(String),
    Boolean(bool),
}

#[derive(Clone)]
pub enum BlockKeyword {
    Map,
    Layer,
}

#[derive(Clone)]
pub enum ExprKeyword {
    Srid,
    Extent,
    Data,
    Sym,
}

#[derive(Clone)]
pub enum SymKeyword {
    Fill,
    Stroke,
    Pattern,
    Label,
}

#[derive(Clone)]
pub enum Keyword {
    Block(BlockKeyword),
    Expr(ExprKeyword),
    Sym(SymKeyword),
}

#[derive(Clone)]
pub enum Operator {
    Eq,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Clone)]
pub enum Symbol {
    Or,
    Then,
    Pipe,
    Ident(String),
}

#[derive(Clone)]
pub enum Value {
    Lit(Literal),
    Ref(String),
}

#[derive(Clone)]
pub struct Function {
    pub name: String,
    pub args: Vec<Value>,
}

#[derive(Clone)]
pub enum Node {
    Lit(Literal),
    Key(Keyword),
    Op(Operator),
    Sym(Symbol),
    Fn(Function),
    Comment(String),
}

impl From<Num> for Node {
    fn from(arg: Num) -> Self {
        Node::Lit(Literal::Number(arg))
    }
}

impl From<i64> for Node {
    fn from(arg: i64) -> Self {
        Node::Lit(Literal::Number(Num::Integer(arg)))
    }
}

impl From<f64> for Node {
    fn from(arg: f64) -> Self {
        Node::Lit(Literal::Number(Num::Float(arg)))
    }
}

impl From<String> for Node {
    fn from(arg: String) -> Self {
        Node::Lit(Literal::String(arg))
    }
}

impl From<bool> for Node {
    fn from(arg: bool) -> Self {
        Node::Lit(Literal::Boolean(arg))
    }
}

impl From<BlockKeyword> for Node {
    fn from(arg: BlockKeyword) -> Self {
        Node::Key(Keyword::Block(arg))
    }
}

impl From<ExprKeyword> for Node {
    fn from(arg: ExprKeyword) -> Self {
        Node::Key(Keyword::Expr(arg))
    }
}

impl From<SymKeyword> for Node {
    fn from(arg: SymKeyword) -> Self {
        Node::Key(Keyword::Sym(arg))
    }
}

impl From<Operator> for Node {
    fn from(arg: Operator) -> Self {
        Node::Op(arg)
    }
}

impl From<Symbol> for Node {
    fn from(arg: Symbol) -> Self {
        Node::Sym(arg)
    }
}

#[derive(Clone)]
pub struct Expression {
    pub nodes: Vec<Node>,
}

impl Expression {
    pub fn new(nodes: Vec<Node>) -> Self {
        Expression { nodes }
    }

    pub fn empty() -> Self {
        Expression { nodes: Vec::new() }
    }

    pub fn add<N>(&self, n: N) -> Self
    where
        N: Into<Node>,
    {
        let mut nodes = self.nodes.clone();
        nodes.push(n.into());
        Expression { nodes }
    }
}

pub fn expr() -> Expression {
    Expression::empty()
}

pub struct MapBlock {
    pub expressions: Vec<Expression>,
}

pub struct LayerBlock {
    pub expressions: Vec<Expression>,
}

pub struct MapSpec {
    pub map: MapBlock,
    pub layers: Vec<LayerBlock>,
}
