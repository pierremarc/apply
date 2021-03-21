#[derive(Debug, Clone)]
pub enum Num {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(Num),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub enum BlockKeyword {
    Map,
    Layer,
}

#[derive(Debug, Clone)]
pub enum ExprKeyword {
    Srid,
    Extent,
    Data,
    Sym,
}

#[derive(Debug, Clone)]
pub enum SymKeyword {
    Fill,
    Stroke,
    Pattern,
    Label,
}

#[derive(Debug, Clone)]
pub enum Keyword {
    Block(BlockKeyword),
    Expr(ExprKeyword),
    Sym(SymKeyword),
}

#[derive(Debug, Clone)]
pub enum Operator {
    Eq,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Debug, Clone)]
pub enum Symbol {
    Or,
    Then,
    Pipe,
    Ident(String),
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: ValueList,
}

#[derive(Debug, Clone)]
pub enum Node {
    Lit(Literal),
    Key(Keyword),
    Op(Operator),
    Sym(Symbol),
    Fn(FunctionCall),
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

#[derive(Debug, Clone)]
pub struct Srid {
    pub value: i64,
}

#[derive(Debug, Clone)]
pub struct Extent {
    pub minx: Num,
    pub miny: Num,
    pub maxx: Num,
    pub maxy: Num,
}

#[derive(Debug, Clone)]
pub enum DataType {
    String,
    Number,
    Boolean,
}

#[derive(Debug, Clone)]
pub enum Constructor {
    Select {
        selector: String,
        datatype: DataType,
    },
    Val(Value),
}

#[derive(Debug, Clone)]
pub struct Data {
    pub ident: String,
    pub constructor: Box<Constructor>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Lit(Literal),
    Data(Data),
    Fn(FunctionCall),
}

#[derive(Debug, Clone)]
pub struct ValuePair(Value, Value);

pub fn pair(left: Value, right: Value) -> ValuePair {
    ValuePair(left, right)
}

pub type ValueList = Vec<Value>;

#[derive(Debug, Clone)]
pub enum Predicate {
    Equal(ValuePair),
    NotEqual(ValuePair),
    GreaterThan(ValuePair),
    GreaterThanOrEqual(ValuePair),
    LesserThan(ValuePair),
    LesserThanOrEqual(ValuePair),
}

#[derive(Debug, Clone)]
pub struct Circle {
    pub radius: Value,
}
#[derive(Debug, Clone)]
pub struct Square {
    pub size: Value,
}

#[derive(Debug, Clone)]
pub struct Color;
#[derive(Debug, Clone)]
pub struct Fill {
    pub color: Value,
}
#[derive(Debug, Clone)]
pub struct Stroke {
    pub color: Value,
    pub size: Value,
}
#[derive(Debug, Clone)]
pub struct Pattern {
    pub path: Value,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub content: Value,
}

#[derive(Debug, Clone)]
pub enum Command {
    Circle(Circle),
    Square(Square),
    Fill(Fill),
    Stroke(Stroke),
    Pattern(Pattern),
    Label(Label),
}

#[derive(Debug, Clone)]
pub struct Sym {
    pub predicate: Predicate,
    pub consequent: Vec<Command>,
}

#[derive(Debug, Clone)]
pub enum Driver {
    Geojson,
    Postgis,
    Shapefile,
}

#[derive(Debug, Clone)]
pub struct Source {
    pub driver: Driver,
    pub path: String,
    pub srid: Option<Num>,
}

#[derive(Debug, Clone)]
pub enum Directive {
    Srid(Srid),
    Extent(Extent),
    Data(Data),
    Sym(Sym),
    Source(Source),
}

impl From<Srid> for Directive {
    fn from(arg: Srid) -> Self {
        Directive::Srid(arg)
    }
}

impl From<Extent> for Directive {
    fn from(arg: Extent) -> Self {
        Directive::Extent(arg)
    }
}

impl From<Data> for Directive {
    fn from(arg: Data) -> Self {
        Directive::Data(arg)
    }
}

impl From<Sym> for Directive {
    fn from(arg: Sym) -> Self {
        Directive::Sym(arg)
    }
}

impl From<Source> for Directive {
    fn from(arg: Source) -> Self {
        Directive::Source(arg)
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct MapBlock {
    pub directives: Vec<Directive>,
}

#[derive(Debug, Clone)]
pub struct LayerBlock {
    pub directives: Vec<Directive>,
}

#[derive(Debug, Clone)]
pub struct MapSpec {
    pub map: MapBlock,
    pub layers: Vec<LayerBlock>,
}
