use std::convert::TryInto;

use parser::ast::Num;
use serde_json::value::Value;

#[derive(Clone)]
pub enum PathIndex {
    Key(String),
    Index(usize),
}

impl From<String> for PathIndex {
    fn from(arg: String) -> Self {
        PathIndex::Key(arg)
    }
}

impl From<&str> for PathIndex {
    fn from(arg: &str) -> Self {
        PathIndex::Key(String::from(arg))
    }
}

impl From<usize> for PathIndex {
    fn from(arg: usize) -> Self {
        PathIndex::Index(arg)
    }
}

pub struct Path {
    p: Vec<PathIndex>,
}

fn append<T>(s: &Vec<T>, a: T) -> Vec<T>
where
    T: Clone,
{
    let mut t = s.clone();
    t.push(a);
    t
}

impl Path {
    pub fn new() -> Self {
        Path { p: Vec::new() }
    }

    pub fn add<I>(&self, i: I) -> Self
    where
        I: Into<PathIndex>,
    {
        Path {
            p: append(&self.p, i.into()),
        }
    }

    fn view<'a>(&self, v: &'a Value) -> Option<&'a Value> {
        self.p.iter().fold(Some(v), |acc, i| {
            acc.and_then(|v| match i {
                PathIndex::Key(k) => v.get(k),
                PathIndex::Index(i) => v.get(i),
            })
        })
    }
}

pub trait Select<Output> {
    fn select_bool(&self, _value: bool) -> Option<Output> {
        None
    }

    fn select_string(&self, _value: &str) -> Option<Output> {
        None
    }

    fn select_u64(&self, _value: u64) -> Option<Output> {
        None
    }

    fn select_i64(&self, _value: i64) -> Option<Output> {
        None
    }

    fn select_f64(&self, _value: f64) -> Option<Output> {
        None
    }
}

pub fn select<S, Output>(s: &S, p: &Path, data: &Value) -> Option<Output>
where
    S: Select<Output>,
{
    p.view(data).and_then(|value| match value {
        Value::Bool(b) => s.select_bool(*b),
        Value::String(st) => s.select_string(&st),
        Value::Number(n) => {
            if n.is_i64() {
                s.select_i64(n.as_i64().unwrap())
            } else if n.is_u64() {
                s.select_u64(n.as_u64().unwrap())
            } else {
                s.select_f64(n.as_f64().unwrap())
            }
        }
        _ => None,
    })
}

pub struct SelectorString<'a, T>(Box<dyn 'a + Fn(&str) -> Option<T>>);

impl<'a, T> Select<T> for SelectorString<'a, T> {
    fn select_string(&self, value: &str) -> Option<T> {
        (self.0)(value)
    }
}

pub fn selector_string<'a, T, F>(f: F) -> SelectorString<'a, T>
where
    F: 'a + Fn(&str) -> Option<T>,
{
    SelectorString(Box::new(f))
}
pub struct SelectorNumber<'a, T>(Box<dyn 'a + Fn(Num) -> Option<T>>);

impl<'a, T> Select<T> for SelectorNumber<'a, T> {
    fn select_u64(&self, value: u64) -> Option<T> {
        let value: i64 = value.try_into().ok()?;
        (self.0)(Num::Integer(value))
    }
    fn select_i64(&self, value: i64) -> Option<T> {
        (self.0)(Num::Integer(value))
    }
    fn select_f64(&self, value: f64) -> Option<T> {
        (self.0)(Num::Float(value))
    }
}

pub fn selector_number<'a, T, F>(f: F) -> SelectorNumber<'a, T>
where
    F: 'a + Fn(Num) -> Option<T>,
{
    SelectorNumber(Box::new(f))
}

pub struct SelectorBool<'a, T>(Box<dyn 'a + Fn(bool) -> Option<T>>);

impl<'a, T> Select<T> for SelectorBool<'a, T> {
    fn select_bool(&self, value: bool) -> Option<T> {
        (self.0)(value)
    }
}

pub fn selector_bool<'a, T, F>(f: F) -> SelectorBool<'a, T>
where
    F: 'a + Fn(bool) -> Option<T>,
{
    SelectorBool(Box::new(f))
}
