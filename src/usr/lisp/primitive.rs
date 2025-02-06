use super::parse::parse;
use super::{bytes, numbers, strings};
use super::{float, number, string};
use super::{Err, Exp, Number};

use crate::api;
use crate::api::regex::Regex;
use crate::api::syscall;
use crate::api::time::format_offset_time;
use crate::sys::fs::OpenFlag;
use crate::usr::host;
use crate::usr::shell;
use crate::{could_not, ensure_length_eq, ensure_length_gt, expected};

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering::Equal;
use core::convert::TryFrom;
use core::convert::TryInto;
use core::str::FromStr;
use num_bigint::BigInt;
use smoltcp::wire::IpAddress;

pub fn lisp_vector(args: &[Exp]) -> Result<Exp, Err> {
    // Constructs a vector literal from its arguments.
    Ok(Exp::Vector(args.to_vec()))
}

pub fn lisp_block(args: &[Exp]) -> Result<Exp, Err> {
    // Constructs a block (code block) literal from its arguments.
    Ok(Exp::Block(args.to_vec()))
}

pub fn lisp_keyword(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let s = string(&args[0])?;
    // Constructs a keyword, note that keywords are stored without the ':'.
    Ok(Exp::Keyword(s))
}

pub fn lisp_struct(args: &[Exp]) -> Result<Exp, Err> {
    // Expects at least a name and one field (so an odd number of arguments)
    ensure_length_gt!(args, 1);
    if args.len() % 2 == 0 {
        return expected!("an odd number of arguments (name and field pairs) for struct");
    }
    let name = match &args[0] {
        Exp::Sym(s) => s.clone(),
        _ => return expected!("a symbol for struct name"),
    };
    let mut fields = BTreeMap::new();
    // Process the field pairs.
    for pair in args[1..].chunks(2) {
        let key = match &pair[0] {
            Exp::Sym(s) => s.clone(),
            _ => return expected!("struct field name must be a symbol"),
        };
        fields.insert(key, pair[1].clone());
    }
    Ok(Exp::Struct { name, fields })
}

pub fn lisp_enum(args: &[Exp]) -> Result<Exp, Err> {
    // Expects at least the enum name and variant (so a minimum of 2 arguments)
    ensure_length_gt!(args, 1);
    if args.len() < 2 {
        return expected!("at least two arguments (enum name and variant) for enum");
    }
    let name = match &args[0] {
        Exp::Sym(s) => s.clone(),
        _ => return expected!("a symbol for enum name"),
    };
    let variant = match &args[1] {
        Exp::Sym(s) => s.clone(),
        _ => return expected!("a symbol for enum variant"),
    };
    let value = if args.len() > 2 {
        Some(Box::new(args[2].clone()))
    } else {
        None
    };
    Ok(Exp::Enum { name, variant, value })
}

pub fn lisp_eq(args: &[Exp]) -> Result<Exp, Err> {
    Ok(Exp::Bool(
        numbers(args)?.windows(2).all(|nums| nums[0] == nums[1]),
    ))
}

pub fn lisp_gt(args: &[Exp]) -> Result<Exp, Err> {
    Ok(Exp::Bool(
        numbers(args)?.windows(2).all(|nums| nums[0] > nums[1]),
    ))
}

pub fn lisp_gte(args: &[Exp]) -> Result<Exp, Err> {
    Ok(Exp::Bool(
        numbers(args)?.windows(2).all(|nums| nums[0] >= nums[1]),
    ))
}

pub fn lisp_lt(args: &[Exp]) -> Result<Exp, Err> {
    Ok(Exp::Bool(
        numbers(args)?.windows(2).all(|nums| nums[0] < nums[1]),
    ))
}

pub fn lisp_lte(args: &[Exp]) -> Result<Exp, Err> {
    Ok(Exp::Bool(
        numbers(args)?.windows(2).all(|nums| nums[0] <= nums[1]),
    ))
}

pub fn lisp_mul(args: &[Exp]) -> Result<Exp, Err> {
    let res = numbers(args)?
        .iter()
        .fold(Number::Int(1), |acc, a| acc * a.clone());
    Ok(Exp::Num(res))
}

pub fn lisp_add(args: &[Exp]) -> Result<Exp, Err> {
    let res = numbers(args)?
        .iter()
        .fold(Number::Int(0), |acc, a| acc + a.clone());
    Ok(Exp::Num(res))
}

pub fn lisp_sub(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_gt!(args, 0);
    let args = numbers(args)?;
    let head = args[0].clone();
    if args.len() == 1 {
        Ok(Exp::Num(-head))
    } else {
        let res = args[1..]
            .iter()
            .fold(Number::Int(0), |acc, a| acc + a.clone());
        Ok(Exp::Num(head - res))
    }
}

pub fn lisp_div(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_gt!(args, 0);
    let mut args = numbers(args)?;
    if args.len() == 1 {
        args.insert(0, Number::Int(1));
    }
    for arg in &args[1..] {
        if arg.is_zero() {
            return expected!("non-zero number");
        }
    }
    let head = args[0].clone();
    let res = args[1..].iter().fold(head, |acc, a| acc / a.clone());
    Ok(Exp::Num(res))
}

pub fn lisp_rem(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_gt!(args, 0);
    let args = numbers(args)?;
    for arg in &args[1..] {
        if arg.is_zero() {
            return expected!("non-zero number");
        }
    }
    let head = args[0].clone();
    let res = args[1..].iter().fold(head, |acc, a| acc % a.clone());
    Ok(Exp::Num(res))
}

pub fn lisp_exp(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_gt!(args, 0);
    let args = numbers(args)?;
    let head = args[0].clone();
    let res = args[1..].iter().fold(head, |acc, a| acc.pow(a));
    Ok(Exp::Num(res))
}

pub fn lisp_shl(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let args = numbers(args)?;
    let res = args[0].clone() << args[1].clone();
    Ok(Exp::Num(res))
}

pub fn lisp_shr(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let args = numbers(args)?;
    let res = args[0].clone() >> args[1].clone();
    Ok(Exp::Num(res))
}

pub fn lisp_cos(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    Ok(Exp::Num(number(&args[0])?.cos()))
}

pub fn lisp_acos(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    if -1.0 <= float(&args[0])? && float(&args[0])? <= 1.0 {
        Ok(Exp::Num(number(&args[0])?.acos()))
    } else {
        expected!("argument to be between -1.0 and 1.0")
    }
}

pub fn lisp_asin(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    if -1.0 <= float(&args[0])? && float(&args[0])? <= 1.0 {
        Ok(Exp::Num(number(&args[0])?.asin()))
    } else {
        expected!("argument to be between -1.0 and 1.0")
    }
}

pub fn lisp_atan(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    Ok(Exp::Num(number(&args[0])?.atan()))
}

pub fn lisp_sin(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    Ok(Exp::Num(number(&args[0])?.sin()))
}

pub fn lisp_tan(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    Ok(Exp::Num(number(&args[0])?.tan()))
}

pub fn lisp_trunc(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    Ok(Exp::Num(number(&args[0])?.trunc()))
}

pub fn lisp_shell(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_gt!(args, 0);
    let cmd = strings(args)?.join(" ");
    match shell::exec(&cmd) {
        Ok(()) => Ok(Exp::Num(Number::from(0 as u8))),
        Err(code) => Ok(Exp::Num(Number::from(code as u8))),
    }
}

pub fn lisp_string(args: &[Exp]) -> Result<Exp, Err> {
    let args: Vec<String> = args
        .iter()
        .map(|arg| match arg {
            Exp::Str(s) => format!("{}", s),
            exp => format!("{}", exp),
        })
        .collect();
    Ok(Exp::Str(args.join("")))
}

pub fn lisp_string_binary(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let s = string(&args[0])?;
    let buf = s.as_bytes();
    Ok(Exp::List(
        buf.iter().map(|b| Exp::Num(Number::from(*b))).collect(),
    ))
}

pub fn lisp_binary_string(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match &args[0] {
        Exp::List(list) => {
            let buf = bytes(list)?;
            let s = String::from_utf8(buf).or(expected!("a valid UTF-8 string"))?;
            Ok(Exp::Str(s))
        }
        _ => expected!("argument to be a list"),
    }
}

pub fn lisp_binary_number(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match (&args[0], &args[1]) {
        // TODO: default type to "int" and make it optional
        (Exp::List(list), Exp::Str(kind)) => {
            let buf = bytes(list)?;
            ensure_length_eq!(buf, 8);
            match kind.as_str() {
                // TODO: bigint
                "int" => Ok(Exp::Num(Number::Int(i64::from_be_bytes(
                    buf[0..8].try_into().unwrap(),
                )))),
                "float" => Ok(Exp::Num(Number::Float(f64::from_be_bytes(
                    buf[0..8].try_into().unwrap(),
                )))),
                _ => expected!("valid number type"),
            }
        }
        _ => {
            expected!("arguments to be the type of number and a list of bytes")
        }
    }
}

pub fn lisp_number_binary(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let n = number(&args[0])?;
    Ok(Exp::List(
        n.to_be_bytes()
            .iter()
            .map(|b| Exp::Num(Number::from(*b)))
            .collect(),
    ))
}

pub fn lisp_number_string(args: &[Exp]) -> Result<Exp, Err> {
    let r = match args.len() {
        2 => {
            let r = number(&args[1])?.try_into()?; // TODO: Reject Number::Float
            if !(2..37).contains(&r) {
                return expected!("radix in the range 2..37");
            }
            r
        }
        _ => 10,
    };
    let s = match number(&args[0])? {
        Number::Int(n) if args.len() == 2 => BigInt::from(n).to_str_radix(r).to_uppercase(),
        Number::BigInt(n) if args.len() == 2 => n.to_str_radix(r).to_uppercase(),
        n => {
            ensure_length_eq!(args, 1);
            format!("{}", n)
        }
    };
    Ok(Exp::Str(s))
}

pub fn lisp_string_number(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let s = string(&args[0])?;
    let n = s.parse().or(could_not!("parse number"))?;
    Ok(Exp::Num(n))
}

pub fn lisp_type(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let exp = match args[0] {
        Exp::Primitive(_) => "function",
        Exp::Function(_) => "function",
        Exp::Macro(_) => "macro",
        Exp::List(_) => "list",
        Exp::Dict(_) => "dict",
        Exp::Bool(_) => "boolean",
        Exp::Str(_) => "string",
        Exp::Sym(_) => "symbol",
        Exp::Num(_) => "number",
        Exp::Enum { .. } => "enum",
        Exp::Vector(_) => "vector",
        Exp::Block(_) => "block",
        Exp::Struct { .. } => "struct",
        Exp::Keyword(_) => "keyword",
    };
    Ok(Exp::Str(exp.to_string()))
}

pub fn lisp_parse(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let s = string(&args[0])?;
    let (_, exp) = parse(&s)?;
    Ok(exp)
}

pub fn lisp_list(args: &[Exp]) -> Result<Exp, Err> {
    Ok(Exp::List(args.to_vec()))
}

pub fn lisp_unique(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    if let Exp::List(list) = &args[0] {
        let mut list = list.clone();
        list.dedup();
        Ok(Exp::List(list))
    } else {
        expected!("argument to be a list")
    }
}

pub fn lisp_sort(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    if let Exp::List(list) = &args[0] {
        let mut list = list.clone();
        list.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Equal));
        Ok(Exp::List(list))
    } else {
        expected!("argument to be a list")
    }
}

pub fn lisp_contains(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match &args[0] {
        Exp::Dict(d) => Ok(Exp::Bool(d.contains_key(&format!("{}", args[1])))),
        Exp::List(l) => Ok(Exp::Bool(l.contains(&args[1]))),
        Exp::Str(s) => Ok(Exp::Bool(s.contains(&string(&args[1])?))),
        _ => expected!("first argument to be a list or a string"),
    }
}

pub fn lisp_slice(args: &[Exp]) -> Result<Exp, Err> {
    let (a, b) = match args.len() {
        2 => (usize::try_from(number(&args[1])?)?, 1),
        3 => (
            usize::try_from(number(&args[1])?)?,
            usize::try_from(number(&args[2])?)?,
        ),
        _ => return expected!("2 or 3 arguments"),
    };
    match &args[0] {
        Exp::List(l) => {
            let l: Vec<Exp> = l.iter().skip(a).take(b).cloned().collect();
            Ok(Exp::List(l))
        }
        Exp::Str(s) => {
            let s: String = s.chars().skip(a).take(b).collect();
            Ok(Exp::Str(s))
        }
        _ => expected!("first argument to be a list or a string"),
    }
}

pub fn lisp_chunks(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match (&args[0], &args[1]) {
        (Exp::List(list), Exp::Num(num)) => {
            let n = usize::try_from(num.clone())?;
            Ok(Exp::List(
                list.chunks(n).map(|a| Exp::List(a.to_vec())).collect(),
            ))
        }
        _ => expected!("a list and a number"),
    }
}

pub fn lisp_length(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match &args[0] {
        Exp::List(list) => Ok(Exp::Num(Number::from(list.len()))),
        Exp::Str(string) => Ok(Exp::Num(Number::from(string.chars().count()))),
        _ => expected!("a list or a string"),
    }
}

// TODO: This could also concat strings
pub fn lisp_concat(args: &[Exp]) -> Result<Exp, Err> {
    let mut res = vec![];
    for arg in args {
        if let Exp::List(list) = arg {
            res.extend_from_slice(list);
        } else {
            return expected!("a list");
        }
    }
    Ok(Exp::List(res))
}

// Number module

pub fn lisp_number_type(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match args[0] {
        Exp::Num(Number::Int(_)) => Ok(Exp::Str("int".to_string())),
        Exp::Num(Number::BigInt(_)) => Ok(Exp::Str("bigint".to_string())),
        Exp::Num(Number::Float(_)) => Ok(Exp::Str("float".to_string())),
        _ => expected!("argument to be a number"),
    }
}

// Regex module

pub fn lisp_regex_find(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match (&args[0], &args[1]) {
        (Exp::Str(regex), Exp::Str(s)) => {
            let res = Regex::new(regex)
                .find(s)
                .map(|(a, b)| vec![Exp::Num(Number::from(a)), Exp::Num(Number::from(b))])
                .unwrap_or_default();
            Ok(Exp::List(res))
        }
        _ => expected!("arguments to be a regex and a string"),
    }
}

// String module

pub fn lisp_string_split(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match (&args[0], &args[1]) {
        (Exp::Str(string), Exp::Str(pattern)) => {
            let list = if pattern.is_empty() {
                // NOTE: "abc".split("") => ["", "b", "c", ""]
                string.chars().map(|s| Exp::Str(s.to_string())).collect()
            } else {
                string
                    .split(pattern)
                    .map(|s| Exp::Str(s.to_string()))
                    .collect()
            };
            Ok(Exp::List(list))
        }
        _ => expected!("a string and a pattern"),
    }
}

pub fn lisp_string_trim(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    if let Exp::Str(s) = &args[0] {
        Ok(Exp::Str(s.trim().to_string()))
    } else {
        expected!("a string and a pattern")
    }
}

// File module

pub fn lisp_file_size(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let path = string(&args[0])?;
    match syscall::info(&path) {
        Some(info) => Ok(Exp::Num(Number::from(info.size() as usize))),
        None => could_not!("open file"),
    }
}

pub fn lisp_file_exists(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let path = string(&args[0])?;
    Ok(Exp::Bool(syscall::info(&path).is_some()))
}

pub fn lisp_file_open(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let path = string(&args[0])?;
    let mode = string(&args[1])?;

    let mut flags = match mode.as_ref() {
        "a" => OpenFlag::Append as u8,
        "r" => OpenFlag::Read as u8,
        "w" => OpenFlag::Write as u8,
        _ => return expected!("valid mode"),
    };
    flags |= match syscall::info(&path) {
        Some(info) if info.is_device() => OpenFlag::Device as u8,
        Some(info) if info.is_dir() => OpenFlag::Dir as u8,
        None if &mode == "r" => return could_not!("open file"),
        None => OpenFlag::Create as u8,
        _ => 0,
    };

    match syscall::open(&path, flags) {
        Some(handle) => Ok(Exp::Num(Number::from(handle))),
        None => could_not!("open file"),
    }
}

pub fn lisp_file_close(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let handle = number(&args[0])?.try_into()?;
    syscall::close(handle);
    Ok(Exp::List(vec![]))
}

pub fn lisp_file_read(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let handle = number(&args[0])?.try_into()?;
    let len = number(&args[1])?;

    let mut buf = vec![0; len.try_into()?];
    match syscall::read(handle, &mut buf) {
        Some(n) => {
            buf.resize(n, 0);
            Ok(Exp::List(
                buf.iter().map(|b| Exp::Num(Number::from(*b))).collect(),
            ))
        }
        None => could_not!("read file"),
    }
}

pub fn lisp_file_write(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let handle = number(&args[0])?.try_into()?;

    match &args[1] {
        Exp::List(list) => {
            let buf = bytes(list)?;
            match syscall::write(handle, &buf) {
                Some(n) => Ok(Exp::Num(Number::from(n))),
                None => could_not!("write file"),
            }
        }
        _ => expected!("second argument to be a list"),
    }
}

pub fn lisp_socket_connect(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 3);
    let kind = string(&args[0])?;
    let addr_str = string(&args[1])?;
    let addr = match IpAddress::from_str(&addr_str) {
        Ok(addr) => addr,
        Err(()) => return expected!("valid IP address"),
    };
    let port: usize = number(&args[2])?.try_into()?;
    let flags = OpenFlag::Device as u8;
    if let Some(handle) = syscall::open(&format!("/dev/net/{}", kind), flags) {
        if syscall::connect(handle, addr, port as u16).is_ok() {
            return Ok(Exp::Num(Number::from(handle)));
        }
    }
    could_not!("connect to {}:{}", addr, port)
}

pub fn lisp_socket_listen(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let kind = string(&args[0])?;
    let port: usize = number(&args[1])?.try_into()?;
    let flags = OpenFlag::Device as u8;
    if let Some(handle) = syscall::open(&format!("/dev/net/{}", kind), flags) {
        if syscall::listen(handle, port as u16).is_ok() {
            return Ok(Exp::Num(Number::from(handle)));
        }
    }
    could_not!("listen to 0.0.0.0:{}", port)
}

pub fn lisp_socket_accept(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let handle: usize = number(&args[0])?.try_into()?;
    if let Ok(addr) = syscall::accept(handle) {
        Ok(Exp::Str(format!("{}", addr)))
    } else {
        could_not!("accept connections")
    }
}

pub fn lisp_dict(args: &[Exp]) -> Result<Exp, Err> {
    let mut dict = BTreeMap::new();
    for chunk in args.chunks(2) {
        match chunk {
            [k, v] => dict.insert(format!("{}", k), v.clone()),
            [k] => dict.insert(format!("{}", k), Exp::List(vec![])),
            _ => unreachable!(),
        };
    }
    Ok(Exp::Dict(dict))
}

pub fn lisp_get(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match &args[0] {
        Exp::Dict(d) => {
            let k = format!("{}", args[1]);
            if let Some(v) = d.get(&k) {
                Ok(v.clone())
            } else {
                Ok(Exp::List(vec![]))
            }
        }
        Exp::List(l) => {
            let i = usize::try_from(number(&args[1])?)?;
            if let Some(v) = l.get(i) {
                Ok(v.clone())
            } else {
                Ok(Exp::List(vec![]))
            }
        }
        Exp::Str(s) => {
            let i = usize::try_from(number(&args[1])?)?;
            if let Some(c) = s.chars().nth(i) {
                Ok(Exp::Str(c.to_string()))
            } else {
                Ok(Exp::Str("".to_string()))
            }
        }
        // New: allow field lookup on a struct literal.
        Exp::Struct { ref fields, .. } => {
            let key = format!("{}", args[1]);
            if let Some(v) = fields.get(&key) {
                Ok(v.clone())
            } else {
                Ok(Exp::List(vec![]))
            }
        }
        _ => expected!("first argument to be a dict, a list, a string, or a struct"),
    }
}

pub fn lisp_put(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 3);
    match &args[0] {
        Exp::Dict(d) => {
            let mut d = d.clone();
            let k = format!("{}", args[1]);
            let v = args[2].clone();
            d.insert(k, v);
            Ok(Exp::Dict(d))
        }
        Exp::List(l) => {
            let mut l = l.clone();
            let i = usize::try_from(number(&args[1])?)?;
            let v = args[2].clone();
            l.insert(i, v);
            Ok(Exp::List(l))
        }
        Exp::Str(s) => {
            let mut s: Vec<char> = s.chars().collect();
            let i = usize::try_from(number(&args[1])?)?;
            let v: Vec<char> = string(&args[2])?.chars().collect();
            s.splice(i..i, v);
            let s: String = s.into_iter().collect();
            Ok(Exp::Str(s))
        }
        // New: support updating a struct literal.
        Exp::Struct { ref name, ref fields } => {
            let mut new_fields = fields.clone();
            let key = format!("{}", args[1]);
            new_fields.insert(key, args[2].clone());
            Ok(Exp::Struct {
                name: name.clone(),
                fields: new_fields,
            })
        }
        _ => expected!("first argument to be a dict, a list, a string, or a struct"),
    }
}

pub fn lisp_host(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let hostname = string(&args[0])?;
    match host::resolve(&hostname) {
        Ok(addr) => Ok(Exp::Str(format!("{}", addr))),
        Err(_) => Ok(Exp::List(vec![])),
    }
}

pub fn lisp_date(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let ts = usize::try_from(number(&args[0])?)? as i64;
    let date = api::time::from_timestamp_utc(ts);
    let date_str = format_offset_time(date);
    Ok(Exp::Str(date_str))
}
