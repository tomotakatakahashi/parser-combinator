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

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

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
}
