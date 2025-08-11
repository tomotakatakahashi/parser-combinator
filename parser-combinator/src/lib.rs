trait Parser<T> {
    fn parse<'a>(&self, input: &'a str) -> Vec<(T, &'a str)>;
}

struct Item;

impl Parser<char> for Item {
    fn parse<'a>(&self, input: &'a str) -> Vec<(char, &'a str)> {
        let mut chars = input.chars();
        return match chars.next() {
            Some(c) => return vec![(c, chars.as_str())],
            None => vec![],
        };
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

    fn item() {
        let input = "abc";
        let item = Item {};
        let result = item.parse(input);
        assert_eq!(result, vec![('a', "bc")]);
    }
}
