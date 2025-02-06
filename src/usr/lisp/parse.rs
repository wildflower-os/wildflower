use super::{Err, Exp, Number};
use crate::could_not;

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;

use nom::branch::alt;
use nom::bytes::complete::{escaped_transform, is_not, tag, take_while1};
use nom::character::complete::{char, multispace0, multispace1, one_of};
use nom::combinator::{map, opt, recognize, value};
use nom::multi::{many0, many1, separated_list0};
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::Err::Error;
use nom::IResult;
use nom::Parser;

// Existing number parsers...
fn hexadecimal(input: &str) -> IResult<&str, &str> {
    preceded(
        tag("0x"),
        recognize(many1(terminated(
            one_of("0123456789abcdefABCDEF"),
            many0(char('_'))
        )))
    ).parse(input)
}

fn decimal(input: &str) -> IResult<&str, &str> {
    recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))
        .parse(input)
}

fn binary(input: &str) -> IResult<&str, &str> {
    preceded(
        tag("0b"),
        recognize(many1(terminated(one_of("01"), many0(char('_')))))
    ).parse(input)
}

fn float(input: &str) -> IResult<&str, &str> {
    alt((
        recognize((
            // .42
            char('.'),
            decimal,
            opt((one_of("eE"), opt(one_of("+-")), decimal))
        )),
        recognize((
            // 42e42 and 42.42e42
            decimal,
            opt(preceded(char('.'), decimal)),
            one_of("eE"),
            opt(one_of("+-")),
            decimal
        )),
        recognize((
            // 42. and 42.42
            decimal,
            char('.'),
            opt(decimal)
        ))
    )).parse(input)
}

fn is_symbol_letter(c: char) -> bool {
    let chars = "$<>=-+*/%^?!.";
    c.is_alphanumeric() || chars.contains(c)
}

fn parse_str(input: &str) -> IResult<&str, Exp> {
    let escaped = map(
        opt(escaped_transform(
            is_not("\\\""),
            '\\',
            alt((
                value("\\", tag("\\")),
                value("\"", tag("\"")),
                value("\n", tag("n")),
                value("\r", tag("r")),
                value("\t", tag("t")),
                value("\x08", tag("b")),
                value("\x1B", tag("e"))
            ))
        )),
        |inner| inner.unwrap_or("".to_string())
    );
    let (input, s) = delimited(char('"'), escaped, char('"')).parse(input)?;
    Ok((input, Exp::Str(s)))
}

fn parse_sym(input: &str) -> IResult<&str, Exp> {
    let (input, sym) = take_while1(is_symbol_letter).parse(input)?;
    Ok((input, Exp::Sym(sym.to_string())))
}

fn parse_num(input: &str) -> IResult<&str, Exp> {
    let (input, num) = recognize((
        opt(alt((char('+'), char('-')))),
        alt((float, hexadecimal, binary, decimal))
    )).parse(input)?;
    Ok((input, Exp::Num(Number::from(num))))
}

fn parse_bool(input: &str) -> IResult<&str, Exp> {
    let (input, s) = alt((tag("true"), tag("false"))).parse(input)?;
    Ok((input, Exp::Bool(s == "true")))
}

fn parse_list(input: &str) -> IResult<&str, Exp> {
    let (input, list) = delimited(
        char('('),
        many0(parse_exp),
        char(')')
    ).parse(input)?;
    Ok((input, Exp::List(list)))
}

fn parse_quote(input: &str) -> IResult<&str, Exp> {
    let (input, list) = preceded(char('\''), parse_exp).parse(input)?;
    let list = vec![Exp::Sym("quote".to_string()), list];
    Ok((input, Exp::List(list)))
}

fn parse_unquote_splice(input: &str) -> IResult<&str, Exp> {
    let (input, list) = preceded(tag(",@"), parse_exp).parse(input)?;
    let list = vec![Exp::Sym("unquote-splice".to_string()), list];
    Ok((input, Exp::List(list)))
}

fn parse_splice(input: &str) -> IResult<&str, Exp> {
    let (input, list) = preceded(tag("@"), parse_exp).parse(input)?;
    let list = vec![Exp::Sym("splice".to_string()), list];
    Ok((input, Exp::List(list)))
}

fn parse_unquote(input: &str) -> IResult<&str, Exp> {
    let (input, list) = preceded(char(','), parse_exp).parse(input)?;
    let list = vec![Exp::Sym("unquote".to_string()), list];
    Ok((input, Exp::List(list)))
}

fn parse_quasiquote(input: &str) -> IResult<&str, Exp> {
    let (input, list) = preceded(char('`'), parse_exp).parse(input)?;
    let list = vec![Exp::Sym("quasiquote".to_string()), list];
    Ok((input, Exp::List(list)))
}

fn parse_comment(input: &str) -> IResult<&str, &str> {
    preceded(multispace0, preceded(char('#'), is_not("\n"))).parse(input)
}

// ===== Flower Lisp Extensions =====

/// Parses a keyword literal, e.g. ":foo"
fn parse_keyword(input: &str) -> IResult<&str, Exp> {
    let (input, _) = char(':')(input)?;
    let (input, kw) = take_while1(is_symbol_letter).parse(input)?;
    Ok((input, Exp::Keyword(kw.to_string())))
}

/// Parses a vector literal, e.g. "[1 2 3]"
fn parse_vector(input: &str) -> IResult<&str, Exp> {
    let (input, elems) = delimited(
        char('['),
        many0(parse_exp),
        char(']')
    ).parse(input)?;
    Ok((input, Exp::Vector(elems)))
}

/// Parses a block literal, e.g. "{ expr1 expr2 }"
fn parse_block(input: &str) -> IResult<&str, Exp> {
    let (input, stmts) = delimited(
        char('{'),
        many0(parse_exp),
        char('}')
    ).parse(input)?;
    Ok((input, Exp::Block(stmts)))
}

/// Parses a dictionary literal using the syntax "#{ key: value, ... }"
fn parse_dict(input: &str) -> IResult<&str, Exp> {
    let (input, _) = tag("#{")(input)?;
    let (input, pairs) = separated_list0(
            delimited(multispace0, char(','), multispace0),
            separated_pair(
                alt((parse_str, parse_sym)),
                delimited(multispace0, char(':'), multispace0),
                parse_exp
            )
        ).parse(input)?;
    let (input, _) = char('}')(input)?;
    let mut map = BTreeMap::new();
    for (key_exp, value) in pairs {
        let key = match key_exp {
            Exp::Str(s) => s,
            Exp::Sym(s) => s,
            _ => String::new(),
        };
        map.insert(key, value);
    }
    Ok((input, Exp::Dict(map)))
}

/// Parses a struct literal, e.g. "struct Person { name: \"Alice\", age: 30 }"
fn parse_struct(input: &str) -> IResult<&str, Exp> {
    let (input, _) = tag("struct")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, name) = take_while1(is_symbol_letter).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, fields) = delimited(
        char('{'),
        separated_list0(
            delimited(multispace0, char(','), multispace0),
            separated_pair(
                take_while1(is_symbol_letter),
                delimited(multispace0, char(':'), multispace0),
                parse_exp
            )
        ),
        char('}')
    ).parse(input)?;
    let mut field_map = BTreeMap::new();
    for (k, v) in fields {
        field_map.insert(k.to_string(), v);
    }
    Ok((input, Exp::Struct { name: name.to_string(), fields: field_map }))
}

/// Parses an enum literal using the syntax "(enum Color::Red)" or "(enum Option::Some(42))"
fn parse_paren_enum(input: &str) -> IResult<&str, Exp> {
    let (input, _) = char('(')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("enum")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, name) = take_while1(is_symbol_letter)(input)?;
    let (input, _) = tag("::")(input)?;
    let (input, variant) = take_while1(is_symbol_letter)(input)?;
    let (input, opt_val) = opt(delimited(
        multispace1,
        parse_exp,
        multispace0
    )).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char(')')(input)?;
    Ok((input, Exp::Enum {
        name: name.to_string(),
        variant: variant.to_string(),
        value: opt_val.map(Box::new)
    }))
}

/// Parses an enum literal, e.g. "enum Color::Red" or "enum Option::Some(42)"
fn parse_enum(input: &str) -> IResult<&str, Exp> {
    let (input, _) = tag("enum")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, name) = take_while1(is_symbol_letter).parse(input)?;
    let (input, _) = tag("::")(input)?;
    let (input, variant) = take_while1(is_symbol_letter).parse(input)?;
    let (input, opt_val) = opt(delimited(
        char('('),
        parse_exp,
        char(')')
    )).parse(input)?;
    Ok((input, Exp::Enum {
        name: name.to_string(),
        variant: variant.to_string(),
        value: opt_val.map(Box::new)
    }))
}

// ===== End Flower Lisp Extensions =====

fn parse_exp(input: &str) -> IResult<&str, Exp> {
    let (input, _) = opt(many0(parse_comment)).parse(input)?;
    delimited(
        multispace0,
        alt((
            parse_keyword,
            parse_vector,
            parse_block,
            parse_dict,
            parse_struct,
            parse_paren_enum,
            parse_enum,
            parse_num,
            parse_bool,
            parse_str,
            parse_list,
            parse_quote,
            parse_quasiquote,
            parse_unquote_splice,
            parse_unquote,
            parse_splice,
            parse_sym
        )),
        alt((parse_comment, multispace0))
    ).parse(input)
}

pub fn parse(input: &str) -> Result<(String, Exp), Err> {
    match parse_exp(input) {
        Ok((input, exp)) => Ok((input.to_string(), exp)),
        Err(Error(err)) => {
            if err.input.is_empty() {
                Ok(("".to_string(), Exp::List(vec![
                    Exp::Sym("quote".to_string()),
                    Exp::List(vec![])
                ])))
            } else {
                let line = err.input.lines().next().unwrap();
                could_not!("parse '{}'", line)
            }
        }
        _ => could_not!("parse input"),
    }
}