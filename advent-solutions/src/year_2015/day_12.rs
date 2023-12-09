use std::{collections::HashMap, io::BufRead, iter::Peekable};

fn sum_nums(json: &str) -> i64 {
    let mut num_start = None;
    let mut sum = 0;

    for (i, c) in json.char_indices() {
        if c.is_ascii_digit() || (c == '-') {
            if num_start.is_none() {
                num_start = Some(i);
            }
        } else if let Some(s) = num_start.take() {
            sum += json[s..i].parse::<i64>().unwrap();
        }
    }

    sum
}

#[derive(Debug, PartialEq)]
enum BorrowedJSON<'s> {
    Empty,
    Number(i64),
    String(&'s str),
    Boolean(bool),
    Array(Vec<BorrowedJSON<'s>>),
    Object(HashMap<&'s str, BorrowedJSON<'s>>),
}

// Bespoke implementation which assumes compact representation and no escaped characters
impl<'s> BorrowedJSON<'s> {
    fn from_str(source: &'s str) -> BorrowedJSON<'s> {
        BorrowedJSON::from_iter(source, &mut source.char_indices().peekable())
    }

    fn from_iter<I>(source: &'s str, iter: &mut Peekable<I>) -> BorrowedJSON<'s>
    where
        I: Iterator<Item = (usize, char)>,
    {
        match iter.peek().unwrap().1 {
            '[' => BorrowedJSON::parse_array(source, iter),
            't' | 'f' => BorrowedJSON::parse_boolean(iter),
            '-' | '0'..='9' => BorrowedJSON::parse_number(source, iter),
            '{' => BorrowedJSON::parse_object(source, iter),
            '"' => BorrowedJSON::parse_string(source, iter),
            _ => BorrowedJSON::Empty,
        }
    }

    fn non_red_sum(&self) -> Option<i64> {
        match self {
            BorrowedJSON::Number(v) => Some(*v),
            BorrowedJSON::String(v) => {
                if *v == "red" {
                    None
                } else {
                    Some(0)
                }
            }
            BorrowedJSON::Array(a) => Some(a.iter().filter_map(Self::non_red_sum).sum()),
            BorrowedJSON::Object(o) => {
                let mut s = 0;

                for (k, v) in o {
                    if *k == "red" {
                        return Some(0);
                    }

                    match v.non_red_sum() {
                        Some(si) => s += si,
                        None => {
                            return Some(0);
                        }
                    }
                }

                Some(s)
            }
            _ => Some(0),
        }
    }

    fn parse_array<I>(source: &'s str, iter: &mut Peekable<I>) -> BorrowedJSON<'s>
    where
        I: Iterator<Item = (usize, char)>,
    {
        iter.next(); // [

        let mut p = iter.peek().unwrap();
        let mut entries = Vec::new();

        while p.1 != ']' {
            if p.1 == ',' {
                iter.next();
            }

            entries.push(BorrowedJSON::from_iter(source, iter));

            p = iter.peek().unwrap();
        }

        iter.next(); // ]

        BorrowedJSON::Array(entries)
    }

    fn parse_boolean<I>(iter: &mut Peekable<I>) -> BorrowedJSON<'s>
    where
        I: Iterator<Item = (usize, char)>,
    {
        let v = iter.next().unwrap().1 == 't';

        while iter.next().unwrap().1 != 'e' {}

        BorrowedJSON::Boolean(v)
    }

    fn parse_number<I>(source: &'s str, iter: &mut Peekable<I>) -> BorrowedJSON<'s>
    where
        I: Iterator<Item = (usize, char)>,
    {
        let num_start = iter.next().unwrap().0;
        let mut p = iter.peek();

        while let Some(pu) = p {
            if !pu.1.is_ascii_digit() {
                return BorrowedJSON::Number(source[num_start..pu.0].parse::<i64>().unwrap());
            }

            iter.next();
            p = iter.peek();
        }

        BorrowedJSON::Number(source[num_start..].parse::<i64>().unwrap())
    }

    fn parse_object<I>(source: &'s str, iter: &mut Peekable<I>) -> BorrowedJSON<'s>
    where
        I: Iterator<Item = (usize, char)>,
    {
        iter.next(); // {

        let mut entries = HashMap::new();
        let mut p = iter.peek().unwrap();

        while p.1 != '}' {
            if p.1 == ',' {
                iter.next();
            }

            let k = match BorrowedJSON::parse_string(source, iter) {
                BorrowedJSON::String(s) => s,
                _ => "",
            };

            iter.next(); // :

            let v = BorrowedJSON::from_iter(source, iter);

            entries.insert(k, v);

            p = iter.peek().unwrap();
        }

        iter.next(); // }

        BorrowedJSON::Object(entries)
    }

    fn parse_string<I>(source: &'s str, iter: &mut Peekable<I>) -> BorrowedJSON<'s>
    where
        I: Iterator<Item = (usize, char)>,
    {
        iter.next(); // "

        let mut n = iter.next().unwrap();
        let str_start = n.0;

        while n.1 != '"' {
            n = iter.next().unwrap();
        }

        BorrowedJSON::String(&source[str_start..n.0])
    }
}

fn non_red_sum(json: &str) -> i64 {
    let structure = BorrowedJSON::from_str(json);

    structure.non_red_sum().unwrap_or(0)
}

pub fn part_01(reader: Option<impl BufRead>) {
    let total = reader
        .unwrap()
        .lines()
        .map_while(Result::ok)
        .map(|l| sum_nums(&l))
        .sum::<i64>();

    println!("Total: {total}");
}

pub fn part_02(reader: Option<impl BufRead>) {
    let total = reader
        .unwrap()
        .lines()
        .map_while(Result::ok)
        .map(|l| non_red_sum(&l))
        .sum::<i64>();

    println!("Total: {total}");
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn parse_array() {
        assert_eq!(
            BorrowedJSON::from_str("[]"),
            BorrowedJSON::Array(Vec::new())
        );

        assert_eq!(
            BorrowedJSON::from_str("[1]"),
            BorrowedJSON::Array(vec![BorrowedJSON::Number(1)])
        );

        assert_eq!(
            BorrowedJSON::from_str(r#"["a",4,false]"#),
            BorrowedJSON::Array(vec![
                BorrowedJSON::String("a"),
                BorrowedJSON::Number(4),
                BorrowedJSON::Boolean(false)
            ])
        );

        assert_eq!(
            BorrowedJSON::from_str(r#"[1,,true,,"val"]"#),
            BorrowedJSON::Array(vec![
                BorrowedJSON::Number(1),
                BorrowedJSON::Empty,
                BorrowedJSON::Boolean(true),
                BorrowedJSON::Empty,
                BorrowedJSON::String("val")
            ])
        );
    }

    #[test]
    fn parse_number() {
        assert_eq!(BorrowedJSON::from_str("64"), BorrowedJSON::Number(64));
        assert_eq!(BorrowedJSON::from_str("-12"), BorrowedJSON::Number(-12));
    }

    #[test]
    fn parse_object() {
        assert_eq!(
            BorrowedJSON::from_str("{}"),
            BorrowedJSON::Object(HashMap::new())
        );

        assert_eq!(
            BorrowedJSON::from_str(r#"{"key":"value"}"#),
            BorrowedJSON::Object(HashMap::from([("key", BorrowedJSON::String("value"))]))
        );

        assert_eq!(
            BorrowedJSON::from_str(r#"{"a":4,"b":true}"#),
            BorrowedJSON::Object(HashMap::from([
                ("a", BorrowedJSON::Number(4)),
                ("b", BorrowedJSON::Boolean(true))
            ]))
        );

        assert_eq!(
            BorrowedJSON::from_str(r#"{"a":"b","c":4,"d":false,"e":[]}"#),
            BorrowedJSON::Object(HashMap::from([
                ("a", BorrowedJSON::String("b")),
                ("c", BorrowedJSON::Number(4)),
                ("d", BorrowedJSON::Boolean(false)),
                ("e", BorrowedJSON::Array(Vec::new()))
            ]))
        );
    }

    #[test]
    fn parse_string() {
        assert_eq!(
            BorrowedJSON::from_str(r#""test""#),
            BorrowedJSON::String("test")
        );

        assert_eq!(BorrowedJSON::from_str(r#""""#), BorrowedJSON::String(""));
    }

    #[test]
    fn basic_counts() {
        let tests = vec![
            ("[1,2,3]", 6),
            (r#"{"a":2,"b":4}"#, 6),
            ("[[[3]]]", 3),
            (r#"{"a":{"b":4},"c":-1}"#, 3),
            (r#"{"a":[-1,1]}"#, 0),
            (r#"[-1,{"a":1}]"#, 0),
            ("[]", 0),
            ("{}", 0),
        ];

        for (s, e) in tests {
            assert_eq!(sum_nums(s), e);
        }
    }

    #[test]
    fn structured_counts() {
        let tests = vec![
            ("[1,2,3]", 6),
            (r#"[1,{"c":"red","b":2},3]"#, 4),
            (r#"{"d":"red","e":[1,2,3,4],"f":5}"#, 0),
            (r#"[1,"red",5]"#, 6),
        ];

        for (s, e) in tests {
            assert_eq!(non_red_sum(s), e);
        }
    }
}
