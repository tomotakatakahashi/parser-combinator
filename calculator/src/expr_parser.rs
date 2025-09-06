use parser_combinator::{alt, char_parser, lazy, nat, ret, seq, token, Parser};

pub fn expr<'a>() -> Box<Parser<'a, u32>> {
    let add = ret(
        seq(
            token(lazy(|| term())),
            seq(token(char_parser('+')), token(lazy(|| expr()))),
        ),
        |v| v.0 + v.1 .1,
    );
    alt(add, lazy(|| term()))
}

fn term<'a>() -> Box<Parser<'a, u32>> {
    let mul = ret(
        seq(
            token(lazy(|| factor())),
            seq(token(char_parser('*')), token(lazy(|| term()))),
        ),
        |(left, (_, right))| left * right,
    );
    alt(mul, lazy(|| factor()))
}

fn factor<'a>() -> Box<Parser<'a, u32>> {
    let paren = ret(
        seq(
            token(char_parser('(')),
            seq(lazy(|| expr()), token(char_parser(')'))),
        ),
        |(_c1, (x, _c2))| x,
    );
    alt(paren, nat())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expr_works() {
        assert_eq!(expr()("123"), Some((123, "")));
        assert_eq!(expr()("1 + 23"), Some((24, "")));
        assert_eq!(expr()("1 + (2 + 3)"), Some((6, "")));
        assert_eq!(expr()("1 + (2 + 3) * ((4))"), Some((21, "")));
    }
}
