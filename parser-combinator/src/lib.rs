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

fn ret<S: 'static, T, F>(parser: Box<Parser<'static, S>>, converter: F) -> Box<Parser<'static, T>>
where
    F: Fn(S) -> T + 'static,
{
    Box::new(move |input: &str| match parser(input) {
        None => None,
        Some((v, out)) => Some((converter(v), out)),
    })
}

fn some<T: 'static>(parser: Box<Parser<'static, T>>) -> Box<Parser<'static, Vec<T>>> {
    // Copy and paste the many function becuase I couldn't resolve an ownership issue.
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

        if results.is_empty() {
            None
        } else {
            Some((results, original_input))
        }
    })
}

fn token<T: 'static>(parser: Box<Parser<'static, T>>) -> Box<Parser<'static, T>> {
    Box::new(move |input: &str| {
        let leading_spaces_trimmed = space()(input).unwrap().1;
        return match parser(leading_spaces_trimmed) {
            None => None,
            Some((v, out)) => {
                let (_, rest) = space()(out).unwrap();
                Some((v, rest))
            }
        };
    })
}

fn nat() -> Box<Parser<'static, u32>> {
    ret(some(digit()), |v| {
        let string_repr: String = v.into_iter().collect();
        string_repr.parse().unwrap()
    })
}

fn expr() -> Box<Parser<'static, u32>> {
    let add = ret(
        seq(token(nat()), seq(token(char_parser('+')), token(nat()))),
        |v| v.0 + v.1 .1,
    );
    alt(add, term())
}

fn term() -> Box<Parser<'static, u32>> {
    alt(expr(), nat())
}

/*
Expr = Add Term Expr | Val Term
Term = Expr Expr | Val Int
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

    #[test]
    fn token_works() {
        let parser = token(many(digit()));
        assert_eq!(parser("  12  a"), Some((vec!['1', '2'], "a")));
    }

    #[test]
    fn nat_works() {
        assert_eq!(nat()("123abc"), Some((123, "abc")));
        assert_eq!(nat()(""), None);
    }

    #[test]
    fn expr_works() {
        assert_eq!(expr()("123"), Some((123, "")));
    }
}
