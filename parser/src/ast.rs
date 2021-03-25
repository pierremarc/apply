#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Num {
    Integer(i64),
    Float(f64),
}

// impl PartialEq for Num {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (Num::Float(a), Num::Float(b)) => a == b,
//             (Num::Integer(a), Num::Integer(b)) => a == b,
//             _ => false,
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Literal {
    Nil,
    Number(Num),
    String(String),
    Boolean(bool),
}

// impl PartialEq for Literal {
//     fn eq(&self, other: &Self) -> bool {
//         match (self, other) {
//             (Literal::Nil, Literal::Nil) => true,
//             (Literal::String(a), Literal::String(b)) => a == b,
//             (Literal::Boolean(a), Literal::Boolean(b)) => a == b,
//             (Literal::Number(a), Literal::Number(b)) => a == b,
//             _ => false,
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub args: ValueList,
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
pub struct Select {
    pub selector: String,
    pub datatype: DataType,
}

#[derive(Debug, Clone)]
pub enum Constructor {
    Select(Select),
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

pub type ValuePair = (Value, Value);

pub fn pair(left: Value, right: Value) -> ValuePair {
    (left, right)
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
pub enum PredGroup {
    Empty,
    Pred(Predicate),
    Or {
        left: Box<PredGroup>,
        right: Box<PredGroup>,
    },
    And {
        left: Box<PredGroup>,
        right: Box<PredGroup>,
    },
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
pub struct Clear;

#[derive(Debug, Clone)]
pub struct DrawGeometry;

#[derive(Debug, Clone)]
pub enum Command {
    Clear(Clear),
    DrawGeometry(DrawGeometry),
    Circle(Circle),
    Square(Square),
    Fill(Fill),
    Stroke(Stroke),
    Pattern(Pattern),
    Label(Label),
}

#[derive(Debug, Clone)]
pub struct Sym {
    pub predicate: PredGroup,
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
