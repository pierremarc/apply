use pom::parser::{call, end, list, none_of, one_of, seq, sym, Parser};
use pom::Error as PomError;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::str::{self, FromStr};
use std::{cell::RefCell, cmp::max};

use crate::ast::{
    pair, Circle, Command, Constructor, Data, DataType, Directive, Driver, Extent, Fill,
    FunctionCall, Label, LayerBlock, Literal, MapBlock, MapSpec, Num, Pattern, Predicate, Source,
    Square, Srid, Stroke, Sym, Value,
};
#[derive(Debug, Clone)]
pub enum ParseError {
    Mysterious,
    Wrap(PomError),
    DataNotInScope(String),
    UnknownPredicate(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mysterious => write!(f, "Something bad happened..."),
            Self::Wrap(e) => write!(f, "PomError: {}", e),
            Self::DataNotInScope(e) => write!(f, "Data Not In Scope: \"{}\"", e),
            Self::UnknownPredicate(e) => write!(f, "Unknown Predicate: \"{}\"", e),
        }
    }
}

// type Function = Box<dyn Fn(ValueList) -> Value>;

#[derive(Clone)]
pub struct Scope {
    values: HashMap<String, Data>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            values: HashMap::new(),
        }
    }

    pub fn find_value(&self, name: String) -> Option<Data> {
        println!(
            "find_value({}) -> {}",
            &name,
            self.values
                .keys()
                .collect::<Vec<_>>()
                .into_iter()
                .map(|s| s.clone())
                .collect::<Vec<_>>()
                .join(", ")
        );
        self.values.get(&name).map(|v| v.clone())
    }
}

pub struct Context {
    scopes: Vec<Scope>,
    depth: usize,
}

impl Context {
    fn new() -> Self {
        Context {
            scopes: Vec::new(),
            depth: 0,
        }
    }

    fn inc_depth(&mut self) {
        self.depth += 1;
    }

    fn dec_depth(&mut self) {
        self.depth -= 1;
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn put_data(&mut self, name: String, value: Data) {
        self.scopes
            .last_mut()
            .map(|scope| scope.values.insert(name, value));
    }

    fn get_data(&self, name: String) -> Option<Data> {
        let default_return: Option<Data> = None;
        self.scopes
            .iter()
            .rev()
            .fold(default_return, |opt_data, scope| {
                opt_data.or(scope.find_value(name.clone()))
            })
    }
}

pub type SharedContext = Rc<RefCell<Context>>;

pub fn new_context() -> SharedContext {
    Rc::new(RefCell::new(Context::new()))
}

pub fn inc_depth(ctx: &SharedContext) {
    let mut ctx = ctx.borrow_mut();
    ctx.inc_depth();
    assert!(ctx.depth < 32);
    // println!("Depth: {}", ctx.depth);
}

pub fn dec_depth(ctx: &SharedContext) {
    let mut ctx = ctx.borrow_mut();
    ctx.dec_depth();
}

pub fn push_scope(ctx: &SharedContext) {
    let mut ctx = ctx.borrow_mut();
    ctx.push_scope();
    println!(">> push_scope {}", ctx.scopes.len());
}

pub fn pop_scope(ctx: &SharedContext) {
    let mut ctx = ctx.borrow_mut();
    ctx.pop_scope();
    println!(">> pop_scope {}", ctx.scopes.len());
}

pub fn get_data(ctx: &SharedContext, name: String) -> Option<Data> {
    ctx.try_borrow().ok().and_then(|c| c.get_data(name))
}

pub fn put_data(ctx: &SharedContext, name: String, value: Data) {
    println!("PUT DATA: '{}'", name.clone());
    let _ = ctx
        .try_borrow_mut()
        .map(|mut ctx| ctx.put_data(name, value));
}

pub fn with_init<'a, I, O, E>(parser: Parser<'a, I, O>, init: E) -> Parser<'a, I, O>
where
    I: 'a + PartialEq + std::fmt::Debug,
    O: 'a,
    E: 'a + Fn(),
{
    Parser::new(move |input: &'a [I], start: usize| {
        init();
        (parser.method)(input, start)
    })
}

pub fn with_finalizer<'a, I, O, E>(parser: Parser<'a, I, O>, finalizer: E) -> Parser<'a, I, O>
where
    I: 'a + PartialEq + std::fmt::Debug,
    O: 'a,
    E: 'a + Fn(),
{
    Parser::new(move |input: &'a [I], start: usize| {
        let result = (parser.method)(input, start);
        finalizer();
        result
    })
}

pub fn trace<'a, O>(name: &'a str, parser: Parser<'a, u8, O>) -> Parser<'a, u8, O>
where
    O: 'a + Clone,
{
    Parser::new(move |input: &'a [u8], start: usize| {
        let result = (parser.method)(input, start);
        println!(
            "[trace:{}] {} -> {}",
            name,
            start,
            match result.clone() {
                Ok((_, end)) => format!("ok({})", end),
                Err(e) => {
                    match e {
                        PomError::Mismatch {
                            ref message,
                            position,
                        } => {
                            let context: String =
                                String::from_utf8(input[max(0, start - 18)..position].into())
                                    .unwrap();
                            format!("err({} -> '{}')", message, context)
                        }
                        PomError::Custom {
                            ref message,
                            position,
                            ref inner,
                        } => {
                            let context: String =
                                String::from_utf8(input[max(0, start - 18)..position].into())
                                    .unwrap();
                            format!("err({} -> '{}')", message, context)
                        }
                        _ => format!("err({})", e),
                    }
                }
            }
        );
        result
    })
}

fn eol<'a>() -> Parser<'a, u8, ()> {
    sym(b'\n').discard().name("eol")
}

fn block_sep<'a>() -> Parser<'a, u8, ()> {
    let first_eol = trace("first_eol", trailing_space());
    let second_eol = trace("second_eol", trailing_space()).repeat(1..);
    trace("block_sep", first_eol - second_eol).name("block_sep")
}

fn spacing<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \t").repeat(1..).discard().name("spacing")
}

fn trailing_space<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \t").repeat(0..).discard() - eol()
}

fn string<'a>() -> Parser<'a, u8, String> {
    let special_char = sym(b'\\')
        | sym(b'/')
        | sym(b'"')
        | sym(b'b').map(|_| b'\x08')
        | sym(b'f').map(|_| b'\x0C')
        | sym(b'n').map(|_| b'\n')
        | sym(b'r').map(|_| b'\r')
        | sym(b't').map(|_| b'\t');
    let escape_sequence = sym(b'\\') * special_char;
    let string = sym(b'"') * (none_of(b"\\\"") | escape_sequence).repeat(0..) - sym(b'"');
    string.convert(String::from_utf8).name("string")
}

fn ascii_letter<'a>() -> Parser<'a, u8, u8> {
    let lc = one_of(b"abcdefghijklmnopqrstuvwxyz");
    let uc = one_of(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    (lc | uc).name("ascii_letter")
}

fn digit<'a>() -> Parser<'a, u8, u8> {
    let n = one_of(b"0123456789");
    n.name("digit")
}

fn integer<'a>() -> Parser<'a, u8, i64> {
    let i = sym(b'-').opt() + one_of(b"0123456789").repeat(0..);
    i.collect()
        .convert(str::from_utf8)
        .convert(|s| i64::from_str(&s))
        .name("integer")
}

fn number<'a>() -> Parser<'a, u8, Num> {
    let integer = one_of(b"123456789") - one_of(b"0123456789").repeat(0..) | sym(b'0');
    let frac = sym(b'.') + one_of(b"0123456789").repeat(1..);
    let exp = one_of(b"eE") + one_of(b"+-").opt() + one_of(b"0123456789").repeat(1..);
    let number = sym(b'-').opt() + integer + frac.opt() + exp.opt();
    number
        .collect()
        .convert(str::from_utf8)
        .convert(|s| {
            i64::from_str(&s)
                .map(|n| Num::Integer(n))
                .or_else(|_| f64::from_str(&s).map(|n| Num::Float(n)))
        })
        .name("number")
}

fn ident<'a>() -> Parser<'a, u8, String> {
    let char_string = (ascii_letter() | digit() | one_of(b"_-.")).repeat(1..);
    char_string.convert(|chars| String::from_utf8(chars))
}

fn literal<'a>() -> Parser<'a, u8, Literal> {
    let n = number().map(|n| Literal::Number(n));
    let s = string().map(|s| Literal::String(s));
    let b = (seq(TRUE) | seq(FALSE)).map(|s| Literal::Boolean(s == TRUE));
    (n | b | s).name("literal")
}

fn srid<'a>(ctx: &SharedContext) -> Parser<'a, u8, Directive> {
    let kw = seq(KEYWORD_SRID);
    let srid_number = integer().expect("srid wants a srid");
    (kw * (spacing() * srid_number))
        .map(|value| Srid { value }.into())
        .name("srid")
}

fn extent<'a>(ctx: &SharedContext) -> Parser<'a, u8, Directive> {
    let kw = seq(KEYWORD_EXTENT) - spacing();
    let minx = number() - spacing();
    let maxx = number() - spacing();
    let miny = number() - spacing();
    let maxy = number();
    let extent = minx + maxx + miny + maxy;

    (kw * extent)
        .map(|(((minx, maxx), miny), maxy)| {
            Extent {
                minx,
                maxx,
                miny,
                maxy,
            }
            .into()
        })
        .name("extent")
}

fn map<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, MapBlock> {
    let map = seq(KEYWORD_MAP) - eol();
    let body = srid(ctx) | extent(ctx) | data(ctx);
    let expressions = list(body, trailing_space());
    (map * expressions)
        .map(|directives| MapBlock { directives })
        .name("map")
    // with_init(
    //     with_finalizer(
    //         (map * expressions)
    //             .map(|directives| MapBlock { directives })
    //             .name("map"),
    //         move || pop_scope(&ctx.clone()),
    //     ),
    //     move || push_scope(&ctx.clone()),
    // )
}

fn source<'a>(ctx: &SharedContext) -> Parser<'a, u8, Directive> {
    let source = seq(KEYWORD_SOURCE) - spacing();
    let driver = (seq(SOURCE_DRIVER_GEOJSON).map(|_| Driver::Geojson)
        | seq(SOURCE_DRIVER_POSTGIS).map(|_| Driver::Postgis)
        | seq(SOURCE_DRIVER_SHAPEFILE).map(|_| Driver::Shapefile))
        - spacing();
    let path = string();
    let opt_srid = (spacing() * number()).opt();
    let everything = source * (driver + path + opt_srid);
    trace(
        "source",
        everything.map(|((driver, path), srid)| Source { driver, path, srid }.into()),
    )
}

fn datatype<'a>(ctx: &SharedContext) -> Parser<'a, u8, DataType> {
    seq(DATATYPE_STRING).map(|_| DataType::String)
        | seq(DATATYPE_NUMBER).map(|_| DataType::Number)
        | seq(DATATYPE_BOOLEAN).map(|_| DataType::Boolean)
}

fn constructor<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, Constructor> {
    let sel = ((seq(KEYWORD_SELECT) - spacing()) * (string() + (spacing() * datatype(ctx))))
        .map(|(selector, datatype)| Constructor::Select { selector, datatype });
    let val = value(ctx).map(|v| Constructor::Val(v));
    trace("constructor", sel | val)
}

fn data<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, Directive> {
    let data = trace("data_keyword", seq(KEYWORD_DATA)) - spacing();
    let identifier = ident() - spacing();
    let constructor = constructor(ctx);
    trace(
        "data",
        (data * (identifier + constructor)).map(move |(ident, constructor)| {
            let data = Data {
                constructor: Box::new(constructor),
                ident: ident.clone(),
            };
            put_data(ctx, ident, data.clone());
            data.into()
        }),
    )
}

fn function<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, FunctionCall> {
    let sep = spacing().opt() + sym(b',') + spacing().opt();
    let args: Parser<'a, u8, Vec<Value>> = list(call(move || value(ctx)), sep);
    let f = ident() - sym(b'(') + args - sym(b')');

    f.map(|(name, args)| FunctionCall { name, args })
        .name("function")
}

fn value<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, Value> {
    let lit = literal().map(|l| Value::Lit(l));
    let dat = ident()
        .convert(move |s| match get_data(ctx, s.clone()) {
            Some(d) => Ok(d),
            _ => {
                println!("NO DATA FOR: '{}'", s.clone());
                Err(ParseError::DataNotInScope(s.clone()))
            }
        })
        // .convert(move |s| get_data(ctx, s.clone()).ok_or(ParseError::DataNotInScope(s)))
        .map(|d| Value::Data(d));
    let fun = function(ctx).map(|f| Value::Fn(f));
    with_init(
        with_finalizer(lit | fun | dat, move || {
            dec_depth(&ctx.clone());
        }),
        move || inc_depth(&ctx.clone()),
    )
}

const PRED_OP_EQ: &[u8] = b"=";
const PRED_OP_NOTEQ: &[u8] = b"!=";
const PRED_OP_GT: &[u8] = b">";
const PRED_OP_GTE: &[u8] = b">=";
const PRED_OP_LT: &[u8] = b"<";
const PRED_OP_LTE: &[u8] = b"<=";

fn predicate<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, Predicate> {
    let op = seq(PRED_OP_EQ)
        | seq(PRED_OP_NOTEQ)
        | seq(PRED_OP_GT)
        | seq(PRED_OP_GTE)
        | seq(PRED_OP_LT)
        | seq(PRED_OP_LTE);
    trace(
        "predicate",
        ((value(ctx) - spacing()) + (op - spacing()) + value(ctx)).convert(
            |((left, op), right)| match op {
                PRED_OP_EQ => Ok(Predicate::Equal(pair(left, right))),
                PRED_OP_NOTEQ => Ok(Predicate::NotEqual(pair(left, right))),
                PRED_OP_GT => Ok(Predicate::GreaterThan(pair(left, right))),
                PRED_OP_GTE => Ok(Predicate::GreaterThanOrEqual(pair(left, right))),
                PRED_OP_LT => Ok(Predicate::LesserThan(pair(left, right))),
                PRED_OP_LTE => Ok(Predicate::LesserThanOrEqual(pair(left, right))),
                _ => Err(ParseError::UnknownPredicate(
                    String::from_utf8(op.into()).unwrap_or(String::new()),
                )),
            },
        ),
    )
}

fn circle<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, Command> {
    let kw = seq(COMMAND_CIRCLE) - spacing();
    (kw * value(ctx)).map(|radius| Command::Circle(Circle { radius }))
}
fn square<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, Command> {
    let kw = seq(COMMAND_SQUARE) - spacing();
    (kw * value(ctx)).map(|size| Command::Square(Square { size }))
}
fn fill<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, Command> {
    let kw = seq(COMMAND_FILL) - spacing();
    (kw * value(ctx)).map(|color| Command::Fill(Fill { color }))
}
fn stroke<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, Command> {
    let kw = seq(COMMAND_STROKE) - spacing();
    (kw * (value(ctx) - spacing() + value(ctx)))
        .map(|(color, size)| Command::Stroke(Stroke { color, size }))
}
fn pattern<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, Command> {
    let kw = seq(COMMAND_PATTERN) - spacing();
    (kw * value(ctx)).map(|path| Command::Pattern(Pattern { path }))
}
fn label<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, Command> {
    let kw = seq(COMMAND_LABEL) - spacing();
    (kw * value(ctx)).map(|content| Command::Label(Label { content }))
}

fn command<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, Command> {
    trace(
        "command",
        circle(ctx) | square(ctx) | fill(ctx) | stroke(ctx) | pattern(ctx) | label(ctx),
    )
}

fn symbology<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, Directive> {
    let kw = seq(KEYWORD_SYM) - spacing();
    let pred = predicate(ctx) - spacing();
    let sep = seq(KEYWORD_COMMAND) - spacing().opt();
    let command = command(ctx) - spacing().opt();
    let commands = (sep * command).repeat(1..);

    trace(
        "sym",
        (kw * (pred + commands)).map(|(predicate, consequent)| {
            Sym {
                predicate,
                consequent,
            }
            .into()
        }),
    )
}

fn directive<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, Directive> {
    trace("directive", source(ctx) | data(ctx) | symbology(ctx))
}

fn layer<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, LayerBlock> {
    let layer = seq(KEYWORD_LAYER) - trailing_space();
    let sep = trailing_space();
    let directives = list(directive(ctx), sep);

    trace(
        "layer",
        with_init(
            with_finalizer(
                (layer * directives)
                    .map(|directives| LayerBlock { directives })
                    .name("layer"),
                move || pop_scope(&ctx.clone()),
            ),
            move || push_scope(&ctx.clone()),
        ),
    )
}

pub fn parse<'a>(map_str: &'a str, ctx: &SharedContext) -> Result<MapSpec, ParseError> {
    push_scope(ctx);
    let map = map(ctx) - block_sep();
    let layers = list(layer(ctx), block_sep());
    let everything = map + layers;
    everything
        .map(|(map, layers)| MapSpec { map, layers })
        .parse(map_str.as_bytes())
        .map_err(|e| ParseError::Wrap(e))
}

const KEYWORD_MAP: &[u8] = b"map";
const KEYWORD_LAYER: &[u8] = b"layer";
const KEYWORD_SOURCE: &[u8] = b"source";
const KEYWORD_SRID: &[u8] = b"srid";
const KEYWORD_EXTENT: &[u8] = b"extent";
const KEYWORD_DATA: &[u8] = b"data";
const KEYWORD_SYM: &[u8] = b"sym";
const KEYWORD_SELECT: &[u8] = b"select";
const KEYWORD_COMMAND: &[u8] = b"->";

const COMMAND_CIRCLE: &[u8] = b"circle";
const COMMAND_SQUARE: &[u8] = b"square";
const COMMAND_FILL: &[u8] = b"fill";
const COMMAND_STROKE: &[u8] = b"stroke";
const COMMAND_PATTERN: &[u8] = b"pattern";
const COMMAND_LABEL: &[u8] = b"label";

const SOURCE_DRIVER_GEOJSON: &[u8] = b"geojson";
const SOURCE_DRIVER_POSTGIS: &[u8] = b"postgis";
const SOURCE_DRIVER_SHAPEFILE: &[u8] = b"shapefile";

const DATATYPE_STRING: &[u8] = b"string";
const DATATYPE_NUMBER: &[u8] = b"number";
const DATATYPE_BOOLEAN: &[u8] = b"bool";

const GEOMETRY_POINT: &[u8] = b"point";
const GEOMETRY_LINE: &[u8] = b"line";
const GEOMETRY_POLYGON: &[u8] = b"polygon";

const TRUE: &[u8] = b"true";
const FALSE: &[u8] = b"false";

#[cfg(test)]
mod parser_test {
    use super::*;
    use crate::ast::*;
    #[test]
    fn trailing_space_works() {
        let map_str = "aaaaaaaaaaaaaa
bbbbbbbbbbbbbb
bbbbbbbbbbbbbb
        ";

        assert_eq!(
            trace(
                "tout",
                list(ascii_letter().repeat(1..), trace("ts", trailing_space())),
            )
            .parse(map_str.as_bytes())
            .unwrap()
            .len(),
            3
        );
    }
    #[test]
    fn map_block_works() {
        let map_str = "map
srid 3857
extent 11111 22222.2 333333 444444
data blue  rgb(0, 0, 255)
        ";

        let result = map(&new_context()).parse(map_str.as_bytes());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().directives.len(), 3);
    }

    #[test]
    fn parse_basic() {
        let map_str = include_str!("../data/map-format-basic");
        // let expected = MapSpec {
        //     map: MapBlock {
        //         directives: [
        //             Srid(Srid { value: 3857 }),
        //             Extent(Extent {
        //                 minx: Integer(11111),
        //                 miny: Integer(333333),
        //                 maxx: Float(22222.2),
        //                 maxy: Integer(444444),
        //             }),
        //             Data(Data {
        //                 ident: "blue",
        //                 constructor: Val(Fn(FunctionCall {
        //                     name: "rgb",
        //                     args: [
        //                         Lit(Number(Integer(0))),
        //                         Lit(Number(Integer(0))),
        //                         Lit(Number(Integer(255))),
        //                     ],
        //                 })),
        //             }),
        //         ],
        //     },
        //     layers: [
        //         LayerBlock {
        //             directives: [
        //                 Source(Source {
        //                     driver: Geojson,
        //                     path: "files/layer1.geojson",
        //                     srid: None,
        //                 }),
        //                 Data(Data {
        //                     ident: "prop",
        //                     constructor: Select {
        //                         selector: "colum name",
        //                         datatype: String,
        //                     },
        //                 }),
        //                 Data(Data {
        //                     ident: "threshold",
        //                     constructor: Val(Lit(Number(Integer(100)))),
        //                 }),
        //                 Sym(Sym {
        //                     predicate: LesserThan(ValuePair(
        //                         Data(Data {
        //                             ident: "prop",
        //                             constructor: Select {
        //                                 selector: "colum name",
        //                                 datatype: String,
        //                             },
        //                         }),
        //                         Data(Data {
        //                             ident: "threshold",
        //                             constructor: Val(Lit(Number(Integer(100)))),
        //                         }),
        //                     )),
        //                     consequent: [Fill(Fill {
        //                         color: Data(Data {
        //                             ident: "blue",
        //                             constructor: Val(Fn(FunctionCall {
        //                                 name: "rgb",
        //                                 args: [
        //                                     Lit(Number(Integer(0))),
        //                                     Lit(Number(Integer(0))),
        //                                     Lit(Number(Integer(255))),
        //                                 ],
        //                             })),
        //                         }),
        //                     })],
        //                 }),
        //             ],
        //         },
        //         LayerBlock {
        //             directives: [
        //                 Source(Source {
        //                     driver: Geojson,
        //                     path: "files/layer2.geojson",
        //                     srid: None,
        //                 }),
        //                 Data(Data {
        //                     ident: "prop1",
        //                     constructor: Select {
        //                         selector: "col1",
        //                         datatype: String,
        //                     },
        //                 }),
        //                 Data(Data {
        //                     ident: "prop2",
        //                     constructor: Select {
        //                         selector: "col2",
        //                         datatype: String,
        //                     },
        //                 }),
        //                 Data(Data {
        //                     ident: "green",
        //                     constructor: Val(Fn(FunctionCall {
        //                         name: "rgb",
        //                         args: [
        //                             Lit(Number(Integer(0))),
        //                             Lit(Number(Integer(255))),
        //                             Lit(Number(Integer(0))),
        //                         ],
        //                     })),
        //                 }),
        //                 Sym(Sym {
        //                     predicate: Equal(ValuePair(
        //                         Data(Data {
        //                             ident: "prop1",
        //                             constructor: Select {
        //                                 selector: "col1",
        //                                 datatype: String,
        //                             },
        //                         }),
        //                         Lit(String("park")),
        //                     )),
        //                     consequent: [
        //                         Fill(Fill {
        //                             color: Data(Data {
        //                                 ident: "green",
        //                                 constructor: Val(Fn(FunctionCall {
        //                                     name: "rgb",
        //                                     args: [
        //                                         Lit(Number(Integer(0))),
        //                                         Lit(Number(Integer(255))),
        //                                         Lit(Number(Integer(0))),
        //                                     ],
        //                                 })),
        //                             }),
        //                         }),
        //                         Stroke(Stroke {
        //                             color: Data(Data {
        //                                 ident: "blue",
        //                                 constructor: Val(Fn(FunctionCall {
        //                                     name: "rgb",
        //                                     args: [
        //                                         Lit(Number(Integer(0))),
        //                                         Lit(Number(Integer(0))),
        //                                         Lit(Number(Integer(255))),
        //                                     ],
        //                                 })),
        //                             }),
        //                             size: Lit(Number(Integer(2))),
        //                         }),
        //                     ],
        //                 }),
        //                 Sym(Sym {
        //                     predicate: Equal(ValuePair(
        //                         Data(Data {
        //                             ident: "prop2",
        //                             constructor: Select {
        //                                 selector: "col2",
        //                                 datatype: String,
        //                             },
        //                         }),
        //                         Lit(String("public")),
        //                     )),
        //                     consequent: [Pattern(Pattern {
        //                         path: Lit(String("files/dot.svg")),
        //                     })],
        //                 }),
        //             ],
        //         },
        //         LayerBlock {
        //             directives: [Source(Source {
        //                 driver: Postgis,
        //                 path: "user:pwd@localhost/schema_name/table_name",
        //                 srid: None,
        //             })],
        //         },
        //     ],
        // };

        match parse(map_str, &new_context()) {
            Ok(spec) => {
                print!("\n**OK**\n{:?}\n", spec);
            }
            Err(err) => {
                panic!("\n**ERROR**\n{}\n", err);
            }
        };
    }
}
