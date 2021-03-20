use pom::parser::{end, list, none_of, one_of, seq, sym, Parser};
use pom::Error as PomError;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::str::{self, FromStr};
use std::{borrow::BorrowMut, cell::RefCell};

use crate::ast::{
    Directive, Extent, FunctionCall, LayerBlock, Literal, MapBlock, MapSpec, Num, Srid, Value,
    ValueList,
};
#[derive(Debug, Clone)]
pub enum ParseError {
    Mysterious,
    Wrap(PomError),
    DataNotInScope(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mysterious => write!(f, "Something bad happened..."),
            Self::Wrap(e) => write!(f, "PomError: {}", e),
            Self::DataNotInScope(e) => write!(f, "Data Not In Scope: \"{}\"", e),
        }
    }
}

// type Function = Box<dyn Fn(ValueList) -> Value>;

#[derive(Clone)]
pub struct Scope {
    parent: Rc<RefCell<Option<Scope>>>,
    values: HashMap<String, Value>,
}

impl Scope {
    pub fn root() -> Self {
        Scope {
            parent: Rc::new(RefCell::new(None)),
            values: HashMap::new(),
        }
    }

    pub fn find_value(&self, name: String) -> Option<Value> {
        let parent_scope = self.parent.borrow();
        let parent_value = parent_scope
            .as_ref()
            .and_then(|scope| scope.find_value(name.clone()));
        self.values.get(&name).map(|v| v.clone()).or(parent_value)
    }
}

pub struct Context {
    pub error: Option<ParseError>,
    pub scope: Scope,
}

impl Context {
    fn put_data(&mut self, name: String, value: Value) {
        self.scope.values.insert(name, value);
    }
}

pub type SharedContext = Rc<RefCell<Context>>;

pub fn get_data(ctx: &SharedContext, name: String) -> Option<Value> {
    let ctx = ctx.borrow();
    ctx.scope.find_value(name).map(|v| v.clone())
}

pub fn put_data(ctx: SharedContext, name: String, value: Value) {
    let _ = ctx
        .try_borrow_mut()
        .map(|mut ctx| ctx.put_data(name, value));
}

// pub fn with_context<'a, I, O, S>(parser: Parser<'a, I, O>, ctx: SharedContext) -> Parser<'a, I, O>
// where
//     I: 'a + PartialEq + Debug,
//     O: 'a,
//     S: 'a + Fn(),
// {
//     let mut ctx = ctx.borrow_mut();
//     Parser::new(move |input: &'a [I], start: usize| {
//         (parser.method)(input, start).map(|(o, s)| {
//             on_success();
//             (o, s)
//         })
//     })
// }

fn eol<'a>() -> Parser<'a, u8, ()> {
    sym(b'\n').discard().name("eol")
}

fn block_sep<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \t").repeat(0..).discard() - (eol() + trailing_space()).name("block_sep")
}

fn spacing<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \t").repeat(1..).discard().name("spacing")
}

fn trailing_space<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \t").repeat(0..).discard() - (eol() | end()).name("trailing_space")
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

fn function<'a>(ctx: &'a SharedContext) -> Parser<'a, u8, FunctionCall> {
    let lit = literal().map(|l| Value::Lit(l));
    let dat =
        ident().convert(move |s| get_data(ctx, s.clone()).ok_or(ParseError::DataNotInScope(s)));
    let arg = lit | dat;
    let args: Parser<'a, u8, Vec<Value>> = list(arg, spacing().opt() + sym(b',') + spacing().opt());
    let f = ident() - sym(b'(') + args - sym(b')');

    f.map(|(name, args)| FunctionCall { name, args })
        .name("function")
}

// fn data_function<'a>() -> Parser<'a, u8, Function> {
//     let arg = string().map(|s| Value::Ref(s));
//     let f = ident() - sym(b'(') + arg - sym(b')');

//     f.map(|(name, arg)| Function {
//         name,
//         args: vec![arg],
//     })
//     .name("data_function")
// }

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

fn map<'a>(ctx: &SharedContext) -> Parser<'a, u8, MapBlock> {
    let map = seq(KEYWORD_MAP) - eol();
    let srid_and_extent = (srid(ctx) - trailing_space()) + extent(ctx);
    let extent_and_srid = (extent(ctx) - trailing_space()) + srid(ctx);
    let expressions = srid_and_extent | extent_and_srid;
    (map * expressions)
        .map(|(e1, e2)| MapBlock {
            directives: vec![e1, e2],
        })
        .name("map")
}

// fn geometry_type<'a>() -> Parser<'a, u8, Node> {
//     (seq(GEOMETRY_POINT) | seq(GEOMETRY_LINE) | seq(GEOMETRY_POLYGON))
//         .convert(|arg| String::from_utf8(arg.to_vec()))
//         .map(|s| Symbol::Ident(s).into())
//         .name("geometry_type")
// }

fn layer<'a>(ctx: &SharedContext) -> Parser<'a, u8, LayerBlock> {
    let layer = seq(KEYWORD_LAYER) - trailing_space();

    layer
        .map(|_| LayerBlock {
            directives: Vec::new(),
        })
        .name("layer")
}

pub fn parse<'a>(map_str: &'a str, ctx: &SharedContext) -> Result<MapSpec, ParseError> {
    let everything = map(ctx) + list(layer(ctx), block_sep()).name("Everything");
    everything
        .map(|(map, layers)| MapSpec { map, layers })
        .parse(map_str.as_bytes())
        .map_err(|e| ParseError::Wrap(e))
}

const KEYWORD_MAP: &[u8] = b"map";
const KEYWORD_LAYER: &[u8] = b"layer";
const KEYWORD_SRID: &[u8] = b"srid";
const KEYWORD_EXTENT: &[u8] = b"extent";
const KEYWORD_DATA: &[u8] = b"data";
const KEYWORD_SYM: &[u8] = b"sym";
const KEYWORD_FILL: &[u8] = b"fill";
const KEYWORD_STROKE: &[u8] = b"stroke";
const KEYWORD_PATTERN: &[u8] = b"pattern";
const KEYWORD_LABEL: &[u8] = b"label";

const GEOMETRY_POINT: &[u8] = b"point";
const GEOMETRY_LINE: &[u8] = b"line";
const GEOMETRY_POLYGON: &[u8] = b"polygon";

const TRUE: &[u8] = b"true";
const FALSE: &[u8] = b"false";

#[cfg(test)]
mod parser_test {
    use super::*;

    #[test]
    fn parse_basic() {
        let map_str = include_str!("../data/map-format-basic");
        match parse(map_str) {
            Ok(spec) => {
                print!("\n**OK**\n{:?}\n", spec);
            }
            Err(err) => {
                panic!("\n**ERROR**\n{}\n", err);
            }
        };
    }
}
