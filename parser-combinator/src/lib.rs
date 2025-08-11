type Parser<'a, T> = dyn Fn(&'a str) -> Option<(T, &'a str)>;

fn item<'a>() -> Box<Parser<'a, char>> {
    Box::new(|input: &str| {
        let mut chars = input.chars();
        return match chars.next() {
            Some(c) => Some((c, chars.as_str())),
            None => None,
        };
    })
}

fn sat<'a, F>(predicate: F) -> Box<Parser<'a, char>>
where
    F: Fn(char) -> bool + 'static,
{
    Box::new(move |input: &str| match item()(input) {
        None => None,
        Some((v, out)) => {
            if predicate(v) {
                Some((v, out))
            } else {
                None
            }
        }
    })
}

fn digit<'a>() -> Box<Parser<'a, char>> {
    sat(|c| c.is_ascii_digit())
}

fn char_parser<'a>(target: char) -> Box<Parser<'a, char>> {
    sat(move |c| c == target)
}

fn many<T: 'static>(parser: Box<Parser<'static, T>>) -> Box<Parser<'static, Vec<T>>> {
    Box::new(move |input: &str| {
        let mut results = Vec::new();
        let mut original_input = input;

        loop {
            // 元のパーサーで解析を試みる
            match parser(original_input) {
                Some((result, next_input)) => {
                    // 成功したら結果を保存し、次の入力に進む
                    results.push(result);
                    original_input = next_input;
                }
                None => {
                    // 失敗したらループを抜ける
                    break;
                }
            }
        }

        // 0回でも成功とみなす
        Some((results, original_input))
    })
}

fn space() -> Box<Parser<'static, Vec<char>>> {
    many(sat(|c| c.is_whitespace()))
}

fn seq<S: 'static, T: 'static>(
    parser1: Box<Parser<'static, S>>,
    parser2: Box<Parser<'static, T>>,
) -> Box<Parser<'static, (S, T)>> {
    Box::new(move |input: &str| match parser1(input) {
        None => None,
        Some((v1, out1)) => match parser2(out1) {
            None => None,
            Some((v2, out2)) => Some(((v1, v2), out2)),
        },
    })
}

fn alt<T: 'static>(
    parser1: Box<Parser<'static, T>>,
    parser2: Box<Parser<'static, T>>,
) -> Box<Parser<'static, T>> {
    Box::new(move |input: &str| match parser1(input) {
        Some(x) => Some(x),
        None => parser2(input),
    })
}

/*
struct Token<P> {
    parser: P,
}

impl<P, T> Parser<T> for Token<P>
where
    P: Parser<T>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(T, &'a str)> {
        let space_parser = space();
        let leading_spaces_trimmed = space_parser.parse(input).unwrap().1;
        return match self.parser.parse(leading_spaces_trimmed) {
            None => None,
            Some((v, out)) => {
                let (_, rest) = space_parser.parse(out).unwrap();
                Some((v, rest))
            }
        };
    }
}

struct Nat {}
impl Parser<u32> for Nat {
    fn parse<'a>(&self, input: &'a str) -> Option<(u32, &'a str)> {
        let digits_parser = Many { parser: digit() };
        match digits_parser.parse(input) {
            None => None,
            Some((v, out)) => {
                let string_repr: String = v.into_iter().collect();
                Some((string_repr.parse().unwrap(), out))
            }
        }
    }
}

struct Return<P, F, S, T>
where
    P: Parser<S>,
    F: Fn(S) -> T,
{
    parser: P,
    converter: F,
    _phantom_s: PhantomData<S>,
    _phantom_t: PhantomData<T>,
}

impl<P, F, S, T> Parser<T> for Return<P, F, S, T>
where
    P: Parser<S>,
    F: Fn(S) -> T,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(T, &'a str)> {
        return match self.parser.parse(input) {
            None => None,
            Some((v, out)) => Some(((self.converter)(v), out)),
        };
    }
}

*/

/*
Expr = Add Term Expr | Val Term
Term = Expr Expr | Val Int

fn expr() -> Box<dyn Parser<u32>> {
    let add = Return {
        parser: Seq {
            parser1: Token { parser: Nat {} },
            parser2: Seq {
                parser1: Token {
                    parser: char_parser('+'),
                },
                parser2: Token { parser: Nat {} },
            },
        },
        converter: |v| v.0 + v.1 .1,
        _phantom_s: PhantomData,
        _phantom_t: PhantomData,
    };
    Box::new(Alt {
        parser1: add,
        parser2: *term(),
    })
}

fn term() -> Box<dyn Parser<u32>> {
    Box::new(Alt {
        parser1: *expr(),
        parser2: Nat {},
    })
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_works() {
        assert_eq!(item()("abc"), Some(('a', "bc")));
    }

    #[test]
    fn digit_works() {
        assert_eq!(digit()("123"), Some(('1', "23")));
        assert_eq!(digit()("abc"), None);
    }

    #[test]
    fn char_parser_works() {
        let parser = char_parser('a');
        assert_eq!(parser("abc"), Some(('a', "bc")));
        assert_eq!(parser("123"), None);
    }

    #[test]
    fn space_work() {
        assert_eq!(space()("   abc"), Some((vec![' ', ' ', ' '], "abc")));
        assert_eq!(space()("abc"), Some((vec![], "abc")));
    }

    #[test]
    fn seq_works() {
        let paren_parser = seq(char_parser('('), seq(digit(), char_parser(')')));
        assert_eq!(paren_parser("(1)2"), Some((('(', ('1', ')')), "2")));
        assert_eq!(paren_parser("1)"), None);
    }

    #[test]
    fn alt_works() {
        let alt_parser = alt(char_parser('+'), char_parser('-'));

        assert_eq!(alt_parser("+1"), Some(('+', "1")));
        assert_eq!(alt_parser("-2"), Some(('-', "2")));
        assert_eq!(alt_parser("~3"), None);
    }

    /*
    #[test]
    fn token() {
        let parser = Token {
            parser: Many { parser: digit() },
        };
        assert_eq!(parser.parse("  12  a"), Some((vec!['1', '2'], "a")));
    }

    #[test]
    fn nat_works() {
        let parser = Nat {};
        assert_eq!(parser.parse("123abc"), Some((123, "abc")));
    }

    #[test]
    fn return_works() {
        let factor_parser = Alt {
            parser1: Return {
                parser: Seq {
                    parser1: Token {
                        parser: char_parser('('),
                    },
                    parser2: Seq {
                        parser1: Token { parser: Nat {} },
                        parser2: Token {
                            parser: char_parser(')'),
                        },
                    },
                },
                converter: |c| c.1 .0,
                _phantom_s: PhantomData,
                _phantom_t: PhantomData,
            },
            parser2: Token { parser: Nat {} },
        };

        assert_eq!(factor_parser.parse(" ( 123 )"), Some((123, "")));
        assert_eq!(factor_parser.parse("123"), Some((123, "")));
    }*/
}
