#![cfg(test)]
use crate::usr::lisp::*;
use alloc::vec;


#[test_case]
fn test_exp() {
    assert_eq!(Exp::Bool(true).is_truthy(), true);
    assert_eq!(Exp::Bool(false).is_truthy(), false);
    assert_eq!(Exp::Num(Number::Int(42)).is_truthy(), true);
    assert_eq!(Exp::List(vec![]).is_truthy(), false);
}

#[allow(unused_must_use)]
#[test_case]
fn test_lisp() {
    use core::f64::consts::PI;
    let env = &mut default_env();

    macro_rules! eval {
        ($e:expr) => {
            format!("{}", parse_eval($e, env).unwrap().1)
        };
    }

    // vectors
    assert_eq!(eval!("(vector 1 2 3)"), "[1 2 3]");

    // block literal
    assert_eq!(eval!("(block 1 2 3)"), "{1 2 3}");

    // keyword literal
    assert_eq!(eval!("(keyword \"foo\")"), ":foo");

    // struct literal
    assert_eq!(
        eval!("(struct Person name \"Alice\" age 30)"),
        "struct Person { age: 30, name: \"Alice\" }" // BTreeMap changes order
    );

    // enum literal:
    assert_eq!(
        eval!("(enum Option::Some 42)"),
        "enum Option::Some(42)"
    );

    // num
    assert_eq!(eval!("6"), "6");
    assert_eq!(eval!("16"), "16");
    assert_eq!(eval!("0x6"), "6");
    assert_eq!(eval!("0xf"), "15");
    assert_eq!(eval!("0x10"), "16");
    assert_eq!(eval!("1.5"), "1.5");
    assert_eq!(eval!("0xff"), "255");
    assert_eq!(eval!("0b0"), "0");
    assert_eq!(eval!("0b1"), "1");
    assert_eq!(eval!("0b10"), "2");
    assert_eq!(eval!("0b11"), "3");

    assert_eq!(eval!("-6"), "-6");
    assert_eq!(eval!("-16"), "-16");
    assert_eq!(eval!("-0x6"), "-6");
    assert_eq!(eval!("-0xF"), "-15");
    assert_eq!(eval!("-0x10"), "-16");
    assert_eq!(eval!("-1.5"), "-1.5");
    assert_eq!(eval!("-0xff"), "-255");
    assert_eq!(eval!("-0b11"), "-3");

    // quote
    assert_eq!(eval!("(quote (1 2 3))"), "(1 2 3)");
    assert_eq!(eval!("'(1 2 3)"), "(1 2 3)");
    assert_eq!(eval!("(quote 1)"), "1");
    assert_eq!(eval!("'1"), "1");
    assert_eq!(eval!("(quote a)"), "a");
    assert_eq!(eval!("'a"), "a");
    assert_eq!(eval!("(quote '(a b c))"), "(quote (a b c))");

    // atom?
    assert_eq!(eval!("(atom? (quote a))"), "true");
    assert_eq!(eval!("(atom? (quote (1 2 3)))"), "false");
    assert_eq!(eval!("(atom? 1)"), "true");

    // equal?
    assert_eq!(eval!("(equal? (quote a) (quote a))"), "true");
    assert_eq!(eval!("(equal? (quote a) (quote b))"), "false");
    assert_eq!(eval!("(equal? (quote a) (quote ()))"), "false");
    assert_eq!(eval!("(equal? (quote ()) (quote ()))"), "true");
    assert_eq!(eval!("(equal? \"a\" \"a\")"), "true");
    assert_eq!(eval!("(equal? \"a\" \"b\")"), "false");
    assert_eq!(eval!("(equal? \"a\" 'b)"), "false");
    assert_eq!(eval!("(equal? 1 1)"), "true");
    assert_eq!(eval!("(equal? 1 2)"), "false");
    assert_eq!(eval!("(equal? 1 1.0)"), "false");
    assert_eq!(eval!("(equal? 1.0 1.0)"), "true");

    // head
    assert_eq!(eval!("(head (quote (1)))"), "1");
    assert_eq!(eval!("(head (quote (1 2 3)))"), "1");

    // tail
    assert_eq!(eval!("(tail (quote (1)))"), "()");
    assert_eq!(eval!("(tail (quote (1 2 3)))"), "(2 3)");

    // cons
    assert_eq!(eval!("(cons (quote 1) (quote (2 3)))"), "(1 2 3)");
    assert_eq!(
        eval!("(cons (quote 1) (cons (quote 2) (cons (quote 3) (quote ()))))"),
        "(1 2 3)"
    );

    // cond
    assert_eq!(eval!("(cond ((< 2 4) 1))"), "1");
    assert_eq!(eval!("(cond ((> 2 4) 1))"), "()");
    assert_eq!(eval!("(cond ((< 2 4) 1) (true 2))"), "1");
    assert_eq!(eval!("(cond ((> 2 4) 1) (true 2))"), "2");

    // if
    assert_eq!(eval!("(if (< 2 4) 1)"), "1");
    assert_eq!(eval!("(if (> 2 4) 1)"), "()");
    assert_eq!(eval!("(if (< 2 4) 1 2)"), "1");
    assert_eq!(eval!("(if (> 2 4) 1 2)"), "2");
    assert_eq!(eval!("(if true 1 2)"), "1");
    assert_eq!(eval!("(if false 1 2)"), "2");
    assert_eq!(eval!("(if '() 1 2)"), "2");
    assert_eq!(eval!("(if 0 1 2)"), "1");
    assert_eq!(eval!("(if 42 1 2)"), "1");
    assert_eq!(eval!("(if \"\" 1 2)"), "1");

    // while
    assert_eq!(
        eval!("(do (variable i 0) (while (< i 5) (set i (+ i 1))) i)"),
        "5"
    );

    // variable
    eval!("(variable a 2)");
    assert_eq!(eval!("(+ a 1)"), "3");
    eval!("(variable add-one (function (b) (+ b 1)))");
    assert_eq!(eval!("(add-one 2)"), "3");
    eval!(
        "(variable fibonacci (function (n) \
             (if (< n 2) n (+ (fibonacci (- n 1)) (fibonacci (- n 2))))))"
    );
    assert_eq!(eval!("(fibonacci 6)"), "8");

    // function
    assert_eq!(eval!("((function (a) (+ 1 a)) 2)"), "3");
    assert_eq!(eval!("((function (a) (* a a)) 2)"), "4");
    assert_eq!(eval!("((function (x) (cons x '(b c))) 'a)"), "(a b c)");

    // function definition shortcut
    eval!("(define (double x) (* x 2))");
    assert_eq!(eval!("(double 2)"), "4");
    eval!("(define-function (triple x) (* x 3))");
    assert_eq!(eval!("(triple 2)"), "6");

    // addition
    assert_eq!(eval!("(+)"), "0");
    assert_eq!(eval!("(+ 2)"), "2");
    assert_eq!(eval!("(+ 2 2)"), "4");
    assert_eq!(eval!("(+ 2 3 4)"), "9");
    assert_eq!(eval!("(+ 2 (+ 3 4))"), "9");

    // subtraction
    assert_eq!(eval!("(- 2)"), "-2");
    assert_eq!(eval!("(- 2 1)"), "1");
    assert_eq!(eval!("(- 1 2)"), "-1");
    assert_eq!(eval!("(- 2 -1)"), "3");
    assert_eq!(eval!("(- 8 4 2)"), "2");

    // multiplication
    assert_eq!(eval!("(*)"), "1");
    assert_eq!(eval!("(* 2)"), "2");
    assert_eq!(eval!("(* 2 2)"), "4");
    assert_eq!(eval!("(* 2 3 4)"), "24");
    assert_eq!(eval!("(* 2 (* 3 4))"), "24");

    // division
    assert_eq!(eval!("(/ 4)"), "0");
    assert_eq!(eval!("(/ 4.0)"), "0.25");
    assert_eq!(eval!("(/ 4 2)"), "2");
    assert_eq!(eval!("(/ 1 2)"), "0");
    assert_eq!(eval!("(/ 1 2.0)"), "0.5");
    assert_eq!(eval!("(/ 8 4 2)"), "1");

    // exponential
    assert_eq!(eval!("(^ 2 4)"), "16");
    assert_eq!(eval!("(^ 2 4 2)"), "256"); // Left to right

    // remainder
    assert_eq!(eval!("(rem 0 2)"), "0");
    assert_eq!(eval!("(rem 1 2)"), "1");
    assert_eq!(eval!("(rem 2 2)"), "0");
    assert_eq!(eval!("(rem 3 2)"), "1");
    assert_eq!(eval!("(rem -1 2)"), "-1");

    // comparisons
    assert_eq!(eval!("(< 6 4)"), "false");
    assert_eq!(eval!("(> 6 4)"), "true");
    assert_eq!(eval!("(> 6 4 2)"), "true");
    assert_eq!(eval!("(> 6)"), "true");
    assert_eq!(eval!("(>)"), "true");
    assert_eq!(eval!("(> 6.0 4)"), "true");
    assert_eq!(eval!("(= 6 4)"), "false");
    assert_eq!(eval!("(= 6 6)"), "true");
    assert_eq!(eval!("(= (+ 0.15 0.15) (+ 0.1 0.2))"), "false"); // FIXME?

    // number
    assert_eq!(eval!("(binary->number (number->binary 42) \"int\")"), "42");
    assert_eq!(
        eval!("(binary->number (number->binary 42.0) \"float\")"),
        "42.0"
    );

    // string
    assert_eq!(eval!("(parse \"9.75\")"), "9.75");
    assert_eq!(eval!("(string \"a\" \"b\" \"c\")"), "\"abc\"");
    assert_eq!(eval!("(string \"a\" \"\")"), "\"a\"");
    assert_eq!(eval!("(string \"foo \" 3)"), "\"foo 3\"");
    assert_eq!(eval!("(equal? \"foo\" \"foo\")"), "true");
    assert_eq!(eval!("(equal? \"foo\" \"bar\")"), "false");
    assert_eq!(eval!("(string/trim \"abc\n\")"), "\"abc\"");
    assert_eq!(
        eval!("(string/split \"a\nb\nc\" \"\n\")"),
        "(\"a\" \"b\" \"c\")"
    );

    // apply
    assert_eq!(eval!("(apply + '(1 2 3))"), "6");
    assert_eq!(eval!("(apply + 1 '(2 3))"), "6");
    assert_eq!(eval!("(apply + 1 2 '(3))"), "6");
    assert_eq!(eval!("(apply + 1 2 3 '())"), "6");

    // trigo
    assert_eq!(eval!("(acos (cos pi))"), PI.to_string());
    assert_eq!(eval!("(acos 0)"), (PI / 2.0).to_string());
    assert_eq!(eval!("(asin 1)"), (PI / 2.0).to_string());
    assert_eq!(eval!("(atan 0)"), "0.0");
    assert_eq!(eval!("(cos pi)"), "-1.0");
    assert_eq!(eval!("(sin (/ pi 2))"), "1.0");
    assert_eq!(eval!("(tan 0)"), "0.0");

    // list
    assert_eq!(eval!("(list)"), "()");
    assert_eq!(eval!("(list 1)"), "(1)");
    assert_eq!(eval!("(list 1 2)"), "(1 2)");
    assert_eq!(eval!("(list 1 2 (+ 1 2))"), "(1 2 3)");

    // bigint
    assert_eq!(
        eval!("9223372036854775807"),
        "9223372036854775807" // -> int
    );
    assert_eq!(
        eval!("9223372036854775808"),
        "9223372036854775808" // -> bigint
    );
    assert_eq!(
        eval!("0x7fffffffffffffff"),
        "9223372036854775807" // -> int
    );
    assert_eq!(
        eval!("0x8000000000000000"),
        "9223372036854775808" // -> bigint
    );
    assert_eq!(
        eval!("0x800000000000000f"),
        "9223372036854775823" // -> bigint
    );
    assert_eq!(
        eval!("(+ 9223372036854775807 0)"),
        "9223372036854775807" // -> int
    );
    assert_eq!(
        eval!("(- 9223372036854775808 1)"),
        "9223372036854775807" // -> bigint
    );
    assert_eq!(
        eval!("(+ 9223372036854775807 1)"),
        "9223372036854775808" // -> bigint
    );
    assert_eq!(
        eval!("(+ 9223372036854775807 1.0)"),
        "9223372036854776000.0" // -> float
    );
    assert_eq!(
        eval!("(+ 9223372036854775807 10)"),
        "9223372036854775817" // -> bigint
    );
    assert_eq!(
        eval!("(* 9223372036854775807 10)"),
        "92233720368547758070" // -> bigint
    );

    assert_eq!(
        eval!("(^ 2 16)"),
        "65536" // -> int
    );
    assert_eq!(
        eval!("(^ 2 128)"),
        "340282366920938463463374607431768211456" // -> bigint
    );
    assert_eq!(
        eval!("(^ 2.0 128)"),
        "340282366920938500000000000000000000000.0" // -> float
    );

    assert_eq!(eval!("(number/type 9223372036854775807)"), "\"int\"");
    assert_eq!(eval!("(number/type 9223372036854775808)"), "\"bigint\"");
    assert_eq!(eval!("(number/type 9223372036854776000.0)"), "\"float\"");

    // quasiquote
    eval!("(variable x 'a)");
    assert_eq!(eval!("`(x ,x y)"), "(x a y)");
    assert_eq!(eval!("`(x ,x y ,(+ 1 2))"), "(x a y 3)");
    assert_eq!(eval!("`(list ,(+ 1 2) 4)"), "(list 3 4)");

    // unquote-splice
    eval!("(variable x '(1 2 3))");
    assert_eq!(eval!("`(+ ,x)"), "(+ (1 2 3))");
    assert_eq!(eval!("`(+ ,@x)"), "(+ 1 2 3)");

    // splice
    assert_eq!(eval!("((function (a @b) a) 1 2 3)"), "1");
    assert_eq!(eval!("((function (a @b) b) 1 2 3)"), "(2 3)");

    // macro
    eval!("(variable foo 42)");
    eval!("(variable set-10 (macro (x) `(set ,x 10)))");
    eval!("(set-10 foo)");
    assert_eq!(eval!("foo"), "10");

    // args
    eval!("(variable list* (function args (concat args '())))");
    assert_eq!(eval!("(list* 1 2 3)"), "(1 2 3)");

    // comments
    assert_eq!(eval!("# comment"), "()");
    assert_eq!(eval!("# comment\n# comment"), "()");
    assert_eq!(eval!("(+ 1 2 3) # comment"), "6");
    assert_eq!(eval!("(+ 1 2 3) # comment\n# comment"), "6");

    // list
    assert_eq!(eval!("(list 1 2 3)"), "(1 2 3)");

    // dict
    assert_eq!(
        eval!("(dict \"a\" 1 \"b\" 2 \"c\" 3)"),
        "(dict \"a\" 1 \"b\" 2 \"c\" 3)"
    );

    // get
    assert_eq!(eval!("(get \"Hello\" 0)"), "\"H\"");
    assert_eq!(eval!("(get \"Hello\" 6)"), "\"\"");
    assert_eq!(eval!("(get (list 1 2 3) 0)"), "1");
    assert_eq!(eval!("(get (list 1 2 3) 3)"), "()");
    assert_eq!(eval!("(get (dict \"a\" 1 \"b\" 2 \"c\" 3) \"a\")"), "1");
    assert_eq!(eval!("(get (dict \"a\" 1 \"b\" 2 \"c\" 3) \"d\")"), "()");

    // put
    assert_eq!(
        eval!("(put (dict \"a\" 1 \"b\" 2) \"c\" 3)"),
        "(dict \"a\" 1 \"b\" 2 \"c\" 3)"
    );
    assert_eq!(eval!("(put (list 1 3) 1 2)"), "(1 2 3)");
    assert_eq!(eval!("(put \"Heo\" 2 \"ll\")"), "\"Hello\"");
}
