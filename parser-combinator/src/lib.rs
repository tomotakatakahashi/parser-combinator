use std::cell::RefCell;
use std::rc::Rc;

pub type Parser<'a, T> = dyn Fn(&'a str) -> Option<(T, &'a str)> + 'a;

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

pub fn char_parser<'a>(target: char) -> Box<Parser<'a, char>> {
    sat(move |c| c == target)
}

fn many<'a, T: 'a>(parser: Rc<Parser<'a, T>>) -> Box<Parser<'a, Vec<T>>> {
    Box::new(move |mut input: &'a str| {
        let mut results: Vec<T> = Vec::new();

        loop {
            match parser(input) {
                Some((result, next_input)) => {
                    results.push(result);
                    input = next_input;
                }
                None => {
                    break;
                }
            }
        }

        Some((results, input))
    })
}

fn space<'a>() -> Box<Parser<'a, Vec<char>>> {
    many(Rc::from(sat(|c| c.is_whitespace())))
}

pub fn seq<'a, S: 'a, T: 'a>(
    parser1: Box<Parser<'a, S>>,
    parser2: Box<Parser<'a, T>>,
) -> Box<Parser<'a, (S, T)>> {
    Box::new(move |input: &str| match parser1(input) {
        None => None,
        Some((v1, out1)) => match parser2(out1) {
            None => None,
            Some((v2, out2)) => Some(((v1, v2), out2)),
        },
    })
}

pub fn alt<'a, T: 'a>(
    parser1: Box<Parser<'a, T>>,
    parser2: Box<Parser<'a, T>>,
) -> Box<Parser<'a, T>> {
    Box::new(move |input: &str| match parser1(input) {
        Some(x) => Some(x),
        None => parser2(input),
    })
}

pub fn ret<'a, S: 'a, T, F>(parser: Box<Parser<'a, S>>, converter: F) -> Box<Parser<'a, T>>
where
    F: Fn(S) -> T + 'static,
{
    Box::new(move |input: &str| match parser(input) {
        None => None,
        Some((v, out)) => Some((converter(v), out)),
    })
}

fn some<'a, T: 'a>(parser: Box<Parser<'a, T>>) -> Box<Parser<'a, Vec<T>>> {
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

pub fn token<'a, T: 'a>(parser: Box<Parser<'a, T>>) -> Box<Parser<'a, T>> {
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

pub fn nat<'a>() -> Box<Parser<'a, u32>> {
    ret(some(digit()), |v| {
        let string_repr: String = v.into_iter().collect();
        string_repr.parse().unwrap()
    })
}

// 遅延評価を実現するパーサーコンビネーター
pub fn lazy<'a, T: 'a>(parser_fn: impl Fn() -> Box<Parser<'a, T>> + 'a) -> Box<Parser<'a, T>> {
    // RcとRefCellを使って、パーサーを一度だけ初期化する
    let parser_cell = Rc::new(RefCell::new(None));
    Box::new(move |input: &str| {
        // もしパーサーがまだ初期化されていなければ、ここで初期化
        if parser_cell.borrow().is_none() {
            *parser_cell.borrow_mut() = Some(parser_fn());
        }
        // 初期化されたパーサーを実行
        parser_cell.borrow().as_ref().unwrap()(input)
    })
}

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
        let parser = token(many(Rc::from(digit())));
        assert_eq!(parser("  12  a"), Some((vec!['1', '2'], "a")));
    }

    #[test]
    fn nat_works() {
        assert_eq!(nat()("123abc"), Some((123, "abc")));
        assert_eq!(nat()(""), None);
    }
}
