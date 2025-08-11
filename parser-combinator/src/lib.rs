trait Parser<T> {
    fn parse<'a>(&self, input: &'a str) -> Option<(T, &'a str)>;
}

struct Item;

impl Parser<char> for Item {
    fn parse<'a>(&self, input: &'a str) -> Option<(char, &'a str)> {
        let mut chars = input.chars();
        return match chars.next() {
            Some(c) => return Some((c, chars.as_str())),
            None => None,
        };
    }
}

struct Sat<F>
where
    F: Fn(char) -> bool,
{
    predicate: F,
}

impl<F> Parser<char> for Sat<F>
where
    F: Fn(char) -> bool,
{
    fn parse<'a>(&self, input: &'a str) -> Option<(char, &'a str)> {
        let item = Item {};
        let item_result = item.parse(input);
        return match item_result {
            None => None,
            Some(out) => {
                if (self.predicate)(out.0) {
                    item_result
                } else {
                    None
                }
            }
        };
    }
}

fn digit() -> Sat<impl Fn(char) -> bool> {
    Sat {
        predicate: |c| c.is_ascii_digit(),
    }
}

fn char_parser(target: char) -> Sat<impl Fn(char) -> bool> {
    Sat {
        predicate: move |c: char| c == target,
    }
}

// `many`パーサーを表現する構造体
// Pは元のパーサー、Tは元のパーサーが返す値の型
struct Many<P> {
    parser: P,
}

// `many`パーサーの`Parser`トレイト実装
impl<P, T> Parser<Vec<T>> for Many<P>
where
    P: Parser<T>,
{
    fn parse<'a>(&self, mut input: &'a str) -> Option<(Vec<T>, &'a str)> {
        let mut results = Vec::new();
        let mut original_input = input;

        loop {
            // 元のパーサーで解析を試みる
            match self.parser.parse(original_input) {
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
    }
}

fn space() -> Many<Sat<impl Fn(char) -> bool>> {
    Many {
        parser: Sat {
            predicate: |c| c.is_whitespace(),
        },
    }
}

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

struct Seq<P1, P2> {
    parser1: P1,
    parser2: P2,
}

impl<P1, P2, S, T> Parser<(S, T)> for Seq<P1, P2>
where
    P1: Parser<S>,
    P2: Parser<T>,
{
    fn parse<'a>(&self, input: &'a str) -> Option<((S, T), &'a str)> {
        match self.parser1.parse(input) {
            None => None,
            Some((v1, out1)) => match self.parser2.parse(out1) {
                None => None,
                Some((v2, out2)) => Some(((v1, v2), out2)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item() {
        let input = "abc";
        let item = Item {};
        let result = item.parse(input);
        assert_eq!(result, Some(('a', "bc")));
    }

    #[test]
    fn digit_works() {
        let digit = digit();

        assert_eq!(digit.parse("123"), Some(('1', "23")));
        assert_eq!(digit.parse("abc"), None);
    }

    #[test]
    fn char_parser_works() {
        let parser = char_parser('a');
        assert_eq!(parser.parse("abc"), Some(('a', "bc")));
        assert_eq!(parser.parse("123"), None);
    }

    #[test]
    fn digits_work() {
        let parser = Many { parser: digit() };
        assert_eq!(parser.parse("123abc"), Some((vec!['1', '2', '3'], "abc")));
        assert_eq!(parser.parse("abc"), Some((vec![], "abc")));
    }

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
    fn seq_works() {
        let paren_parser = Seq {
            parser1: Token {
                parser: char_parser('('),
            },
            parser2: Seq {
                parser1: Token { parser: Nat {} },
                parser2: Token {
                    parser: char_parser(')'),
                },
            },
        };

        assert_eq!(paren_parser.parse("  (  123  )  a").unwrap().0 .1 .0, 123);
        assert_eq!(paren_parser.parse("  (  123  )  a").unwrap().1, "a");
        assert_eq!(paren_parser.parse("a 123 "), None);
    }
}
