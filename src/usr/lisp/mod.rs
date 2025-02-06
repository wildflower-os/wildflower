mod env;
mod eval;
mod expand;
mod number;
mod parse;
mod primitive;
mod tests;

pub use env::Env;
pub use number::Number;

use env::default_env;
use eval::{eval, eval_variable_args};
use expand::expand;
use parse::parse;

use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::prompt::Prompt;

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::cmp;
use core::convert::TryInto;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

// Wildflower Lisp is a lisp-1 like Scheme and Clojure
// This was forked from MOROS
//
// Eval & Env adapted from Risp
// Copyright 2019 Stepan Parunashvili
// https://github.com/stopachka/risp
//
// Parser rewritten from scratch using Nom
// https://github.com/geal/nom
//
// References:
//
// "Recursive Functions of Symic Expressions and Their Computation by Machine"
// by John McCarthy (1960)
//
// "The Roots of Lisp"
// by Paul Graham (2002)
//
// "Technical Issues of Separation in Function Cells and Value Cells"
// by Richard P. Gabriel (1982)

// Types

#[derive(Clone)]
pub enum Exp {
    Primitive(fn(&[Exp]) -> Result<Exp, Err>),
    Function(Box<Function>),
    Macro(Box<Function>),
    List(Vec<Exp>),
    Dict(BTreeMap<String, Exp>),
    Bool(bool),
    Num(Number),
    Str(String),
    Sym(String),
}

impl Exp {
    pub fn is_truthy(&self) -> bool {
        match self {
            Exp::Bool(b) => *b,
            Exp::List(l) => !l.is_empty(),
            _ => true,
        }
    }
}

impl PartialEq for Exp {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Exp::Function(a), Exp::Function(b)) => a == b,
            (Exp::Macro(a), Exp::Macro(b)) => a == b,
            (Exp::List(a), Exp::List(b)) => a == b,
            (Exp::Dict(a), Exp::Dict(b)) => a == b,
            (Exp::Bool(a), Exp::Bool(b)) => a == b,
            (Exp::Num(a), Exp::Num(b)) => a == b,
            (Exp::Str(a), Exp::Str(b)) => a == b,
            (Exp::Sym(a), Exp::Sym(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for Exp {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (Exp::Function(a), Exp::Function(b)) => a.partial_cmp(b),
            (Exp::Macro(a), Exp::Macro(b)) => a.partial_cmp(b),
            (Exp::List(a), Exp::List(b)) => a.partial_cmp(b),
            (Exp::Dict(a), Exp::Dict(b)) => a.partial_cmp(b),
            (Exp::Bool(a), Exp::Bool(b)) => a.partial_cmp(b),
            (Exp::Num(a), Exp::Num(b)) => a.partial_cmp(b),
            (Exp::Str(a), Exp::Str(b)) => a.partial_cmp(b),
            (Exp::Sym(a), Exp::Sym(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match self {
            Exp::Primitive(_) => format!("(function args)"),
            Exp::Function(f) => format!("(function {})", f.params),
            Exp::Macro(m) => format!("(macro {})", m.params),
            Exp::Bool(a) => a.to_string(),
            Exp::Num(n) => n.to_string(),
            Exp::Sym(s) => s.clone(),
            Exp::Str(s) => format!("{:?}", s)
                .replace("\\u{8}", "\\b")
                .replace("\\u{1b}", "\\e"),
            Exp::List(list) => {
                let xs: Vec<_> = list.iter().map(|x| x.to_string()).collect();
                format!("({})", xs.join(" "))
            }
            Exp::Dict(dict) => {
                let mut xs: Vec<_> = dict.iter().map(|(k, v)| format!("{} {}", k, v)).collect();
                xs.insert(0, "dict".into());
                format!("({})", xs.join(" "))
            }
        };
        write!(f, "{}", out)
    }
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Function {
    params: Exp,
    body: Exp,
    doc: Option<String>,
}

#[derive(Debug)]
pub enum Err {
    Reason(String),
}

lazy_static! {
    pub static ref FUNCTIONS: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

#[macro_export]
macro_rules! ensure_length_eq {
    ($list:expr, $count:expr) => {
        if $list.len() != $count {
            let plural = if $count != 1 { "s" } else { "" };
            return expected!("{} expression{}", $count, plural);
        }
    };
}

#[macro_export]
macro_rules! ensure_length_gt {
    ($list:expr, $count:expr) => {
        if $list.len() <= $count {
            let plural = if $count != 1 { "s" } else { "" };
            return expected!("more than {} expression{}", $count, plural);
        }
    };
}

#[macro_export]
macro_rules! ensure_string {
    ($exp:expr) => {
        match $exp {
            Exp::Str(_) => {}
            _ => return expected!("a string"),
        }
    };
}

#[macro_export]
macro_rules! ensure_list {
    ($exp:expr) => {
        match $exp {
            Exp::List(_) => {}
            _ => return expected!("a list"),
        }
    };
}

#[macro_export]
macro_rules! expected {
    ($($arg:tt)*) => ({
        use alloc::format;
        Err(Err::Reason(format!("Expected {}", format_args!($($arg)*))))
    });
}

#[macro_export]
macro_rules! could_not {
    ($($arg:tt)*) => ({
        use alloc::format;
        Err(Err::Reason(format!("Could not {}", format_args!($($arg)*))))
    });
}

pub fn bytes(args: &[Exp]) -> Result<Vec<u8>, Err> {
    args.iter().map(byte).collect()
}

pub fn strings(args: &[Exp]) -> Result<Vec<String>, Err> {
    args.iter().map(string).collect()
}

pub fn numbers(args: &[Exp]) -> Result<Vec<Number>, Err> {
    args.iter().map(number).collect()
}

pub fn string(exp: &Exp) -> Result<String, Err> {
    match exp {
        Exp::Str(s) => Ok(s.to_string()),
        _ => expected!("a string"),
    }
}

pub fn number(exp: &Exp) -> Result<Number, Err> {
    match exp {
        Exp::Num(num) => Ok(num.clone()),
        _ => expected!("a number"),
    }
}

pub fn float(exp: &Exp) -> Result<f64, Err> {
    match exp {
        Exp::Num(num) => Ok(num.into()),
        _ => expected!("a float"),
    }
}

pub fn byte(exp: &Exp) -> Result<u8, Err> {
    number(exp)?.try_into()
}

// REPL

fn parse_eval(input: &str, env: &mut Rc<RefCell<Env>>) -> Result<(String, Exp), Err> {
    let (rest, exp) = parse(input)?;
    let exp = expand(&exp, env)?;
    let exp = eval(&exp, env)?;
    Ok((rest, exp))
}

fn lisp_completer(line: &str) -> Vec<String> {
    let mut entries = Vec::new();
    if let Some(last_word) = line.split_whitespace().next_back() {
        if let Some(f) = last_word.strip_prefix('(') {
            for function in &*FUNCTIONS.lock() {
                if let Some(entry) = function.strip_prefix(f) {
                    entries.push(entry.into());
                }
            }
        }
    }
    entries
}

fn repl(env: &mut Rc<RefCell<Env>>) -> Result<(), ExitCode> {
    let csi_color = Style::color("teal");
    let csi_reset = Style::reset();
    let prompt_string = format!("{}>{} ", csi_color, csi_reset);

    println!("Wildflower Lisp v0.7.0\n");

    let mut prompt = Prompt::new();
    let history_file = "~/.lisp-history";
    prompt.history.load(history_file);
    prompt.completion.set(&lisp_completer);

    while let Some(input) = prompt.input(&prompt_string) {
        if input == "(quit)" {
            break;
        }
        if input.is_empty() {
            println!();
            continue;
        }
        match parse_eval(&input, env) {
            Ok((_, exp)) => {
                println!("{}\n", exp);
            }
            Err(e) => match e {
                Err::Reason(msg) => error!("{}\n", msg),
            },
        }
        prompt.history.add(&input);
        prompt.history.save(history_file);
    }
    Ok(())
}

fn exec(env: &mut Rc<RefCell<Env>>, path: &str) -> Result<(), ExitCode> {
    if let Ok(mut input) = fs::read_to_string(path) {
        loop {
            match parse_eval(&input, env) {
                Ok((rest, _)) => {
                    if rest.is_empty() {
                        break;
                    }
                    input = rest;
                }
                Err(Err::Reason(msg)) => {
                    error!("{}", msg);
                    return Err(ExitCode::Failure);
                }
            }
        }
        Ok(())
    } else {
        error!("Could not find file '{}'", path);
        Err(ExitCode::Failure)
    }
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let env = &mut default_env();

    // Store args in env
    let key = Exp::Sym("args".to_string());
    let list = Exp::List(if args.len() < 2 {
        vec![]
    } else {
        args[2..]
            .iter()
            .map(|arg| Exp::Str(arg.to_string()))
            .collect()
    });
    let quote = Exp::List(vec![Exp::Sym("quote".to_string()), list]);
    if eval_variable_args(&[key, quote], env).is_err() {
        error!("Could not parse args");
        return Err(ExitCode::Failure);
    }

    if args.len() < 2 {
        let init = "/ini/lisp.lsp";
        if fs::exists(init) {
            exec(env, init)?;
        }
        repl(env)
    } else {
        if args[1] == "-h" || args[1] == "--help" {
            return help();
        }
        let path = args[1];
        if let Ok(mut input) = fs::read_to_string(path) {
            loop {
                match parse_eval(&input, env) {
                    Ok((rest, _)) => {
                        if rest.is_empty() {
                            break;
                        }
                        input = rest;
                    }
                    Err(Err::Reason(msg)) => {
                        error!("{}", msg);
                        return Err(ExitCode::Failure);
                    }
                }
            }
            Ok(())
        } else {
            error!("Could not read file '{}'", path);
            Err(ExitCode::Failure)
        }
    }
}

fn help() -> Result<(), ExitCode> {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} lisp {}[<file> [<args>]]{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
    Ok(())
}
