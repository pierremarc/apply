use fmt::Error;
use pom::parser::{end, is_a, list, none_of, one_of, seq, sym, Parser};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::str::{self, FromStr};

use crate::ast::{
    expr, ExprKeyword, Expression, Function, LayerBlock, Literal, MapBlock, MapSpec, Node, Num,
    Symbol, Value,
};
#[derive(Debug, Clone)]
pub enum ParseError {
    Mysterious,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mysterious => write!(f, "Something bad happened..."),
        }
    }
}

pub struct Context {
    pub error: Option<ParseError>,
}

pub type SharedContext = Rc<RefCell<Context>>;

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
    sym(b'\n').discard()
}

fn block_sep<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \t").repeat(0..).discard() - (eol() + trailing_space())
}

fn spacing<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \t").repeat(1..).discard()
}

fn trailing_space<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \t").repeat(0..).discard() - (eol() | end())
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
    string.convert(String::from_utf8)
}

fn ascii_letter<'a>() -> Parser<'a, u8, u8> {
    let lc = one_of(b"abcdefghijklmnopqrstuvwxyz");
    let uc = one_of(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    lc | uc
}

fn digit<'a>() -> Parser<'a, u8, u8> {
    let n = one_of(b"0123456789");
    n
}

fn integer<'a>() -> Parser<'a, u8, i64> {
    let i = sym(b'-').opt() + one_of(b"0123456789").repeat(0..);
    i.collect()
        .convert(str::from_utf8)
        .convert(|s| i64::from_str(&s))
}

fn number<'a>() -> Parser<'a, u8, Num> {
    let integer = one_of(b"123456789") - one_of(b"0123456789").repeat(0..) | sym(b'0');
    let frac = sym(b'.') + one_of(b"0123456789").repeat(1..);
    let exp = one_of(b"eE") + one_of(b"+-").opt() + one_of(b"0123456789").repeat(1..);
    let number = sym(b'-').opt() + integer + frac.opt() + exp.opt();
    number.collect().convert(str::from_utf8).convert(|s| {
        i64::from_str(&s)
            .map(|n| Num::Integer(n))
            .or_else(|_| f64::from_str(&s).map(|n| Num::Float(n)))
    })
}

fn ident<'a>() -> Parser<'a, u8, String> {
    let char_string = (ascii_letter() | digit() | one_of(b"_-.")).repeat(1..);
    char_string.convert(|chars| String::from_utf8(chars))
}

fn literal<'a>() -> Parser<'a, u8, Literal> {
    let n = number().map(|n| Literal::Number(n));
    let s = string().map(|s| Literal::String(s));
    let b = (seq(TRUE) | seq(FALSE)).map(|s| Literal::Boolean(s == TRUE));
    n | b | s
}

fn function<'a>() -> Parser<'a, u8, Function> {
    let arg = literal().map(|l| Value::Lit(l)) | (sym(b'@') * ident()).map(|s| Value::Ref(s));
    let args: Parser<'a, u8, Vec<Value>> = list(arg, sym(b','));
    let f = ident() - sym(b'(') + args - sym(b')');

    f.map(|(name, args)| Function { name, args })
}

fn data_function<'a>() -> Parser<'a, u8, Function> {
    let arg = string().map(|s| Value::Ref(s));
    let f = ident() - sym(b'(') + arg - sym(b')');

    f.map(|(name, arg)| Function {
        name,
        args: vec![arg],
    })
}

fn srid<'a>() -> Parser<'a, u8, Expression> {
    let kw = seq(KEYWORD_SRID);
    let srid_number = integer().expect("srid wants a srid");
    (kw * (spacing() * (integer() - trailing_space())))
        .map(|i| expr().add(ExprKeyword::Srid).add(i))
}

fn extent<'a>() -> Parser<'a, u8, Expression> {
    let kw = seq(KEYWORD_EXTENT);
    let minx = number() - spacing();
    let maxx = number() - spacing();
    let miny = number() - spacing();
    let maxy = number() - trailing_space();
    let extent = minx + maxx + miny + maxy;

    (kw * extent).map(|(((minx, maxx), miny), maxy)| {
        expr()
            .add(ExprKeyword::Extent)
            .add(minx)
            .add(maxx)
            .add(miny)
            .add(maxy)
    })
}

fn expression<'a>() -> Parser<'a, u8, Expression> {
    srid()
    //  seq(KEYWORD_EXTENT)
    // seq(KEYWORD_DATA)
    // seq(KEYWORD_SYM);
}

fn map<'a>() -> Parser<'a, u8, MapBlock> {
    let map = seq(KEYWORD_MAP) - trailing_space();
    let s = srid() - trailing_space();
    let p = extent() - trailing_space();
    let expressions = (s | p).repeat(1..);
    (map * expressions).map(|expressions: Vec<Expression>| MapBlock { expressions })
}

fn geometry_type<'a>() -> Parser<'a, u8, Node> {
    (seq(GEOMETRY_POINT) | seq(GEOMETRY_LINE) | seq(GEOMETRY_POLYGON))
        .convert(|arg| String::from_utf8(arg.to_vec()))
        .map(|s| Symbol::Ident(s).into())
}

fn layer<'a>() -> Parser<'a, u8, LayerBlock> {
    let layer = seq(KEYWORD_LAYER) - spacing();
    let gt = geometry_type() - spacing();

    layer.map(|_| LayerBlock {
        expressions: Vec::new(),
    })
}

pub fn parse<'a>(map_str: &'a str) -> Result<MapSpec, ParseError> {
    let everything = map() + list(layer(), block_sep());
    everything
        .map(|(map, layers)| MapSpec { map, layers })
        .parse(map_str.as_bytes())
        .map_err(|_| ParseError::Mysterious)
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
