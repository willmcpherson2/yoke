pub mod compile;
pub mod parse;

pub use std::collections::HashMap;

pub type Program = HashMap<String, Switch>;

pub type Switch = Vec<Case>;

#[derive(Debug, PartialEq)]
pub struct Case(Vec<Pat>, Term);

#[derive(Debug, PartialEq)]
pub enum Pat {
    Var(String),
    Ctor(String, Vec<Pat>),
}

#[derive(Debug, PartialEq)]
pub enum Term {
    Var(String),
    Ctor(String, Vec<Term>),
    App(String, Vec<Term>),
}
