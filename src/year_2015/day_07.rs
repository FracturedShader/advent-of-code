use std::fmt;
use std::{collections::HashMap, error::Error, io::BufRead};

enum Token {
    Literal(String),
    Ident(String),
    Op(String),
}

impl Token {
    fn parse(piece: &str) -> Option<Token> {
        if piece.is_empty() {
            None
        } else {
            let first_char = piece.chars().next().unwrap();

            if first_char.is_ascii_digit() {
                Some(Token::Literal(piece.to_string()))
            } else if first_char.is_alphabetic() {
                Some(Token::Ident(piece.to_string()))
            } else if first_char.is_ascii_punctuation() {
                Some(Token::Op(piece.to_string()))
            } else {
                None
            }
        }
    }
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Token::Literal(s) => s.clone(),
            Token::Ident(s) => s.clone(),
            Token::Op(s) => s.clone(),
        }
    }
}

#[derive(Debug)]
struct WireDependencyError(String);

impl fmt::Display for WireDependencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Need '{}' to evaluate", self.0)
    }
}

impl Error for WireDependencyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

type WireState = HashMap<String, u16>;
type WireResult = Result<u16, WireDependencyError>;

enum WireValue {
    Literal(u16),
    Ident(String),
}

impl WireValue {
    fn from_token(token: &Token) -> Option<Self> {
        match token {
            Token::Literal(l) => Some(WireValue::Literal(l.parse().unwrap())),
            Token::Ident(s) => Some(WireValue::Ident(s.clone())),
            _ => None,
        }
    }

    fn val(&self, states: &WireState) -> WireResult {
        match *self {
            WireValue::Literal(v) => Ok(v),
            WireValue::Ident(ref i) => {
                if states.contains_key(i) {
                    Ok(states[i])
                } else {
                    Err(WireDependencyError(i.clone()))
                }
            }
        }
    }
}

enum LogicGate {
    Not(WireValue),
    And(WireValue, WireValue),
    Or(WireValue, WireValue),
    LShift(WireValue, WireValue),
    RShift(WireValue, WireValue),
}

impl LogicGate {
    fn binary_from_tokens(val1: WireValue, op: &Token, val2: WireValue) -> Option<LogicGate> {
        match op.to_string().as_str() {
            "AND" => Some(LogicGate::And(val1, val2)),
            "OR" => Some(LogicGate::Or(val1, val2)),
            "LSHIFT" => Some(LogicGate::LShift(val1, val2)),
            "RSHIFT" => Some(LogicGate::RShift(val1, val2)),
            _ => None,
        }
    }

    fn val(&self, states: &WireState) -> WireResult {
        Ok(match *self {
            LogicGate::Not(ref v) => !v.val(states)?,
            LogicGate::And(ref lhs, ref rhs) => lhs.val(states)? & rhs.val(states)?,
            LogicGate::Or(ref lhs, ref rhs) => lhs.val(states)? | rhs.val(states)?,
            LogicGate::LShift(ref lhs, ref rhs) => lhs.val(states)? << rhs.val(states)?,
            LogicGate::RShift(ref lhs, ref rhs) => lhs.val(states)? >> rhs.val(states)?,
        })
    }
}

enum WireSource {
    Gate(LogicGate),
    Value(WireValue),
}

impl WireSource {
    fn val(&self, states: &WireState) -> WireResult {
        Ok(match *self {
            WireSource::Gate(ref g) => g.val(states)?,
            WireSource::Value(ref v) => v.val(states)?,
        })
    }
}

type WireMap = HashMap<String, WireSource>;

#[derive(Default)]
struct LogicWires {
    state: WireState,
    connections: WireMap,
}

impl LogicWires {
    fn add_connection(&mut self, statement: &str) {
        let mut tokens = statement.split_ascii_whitespace().filter_map(Token::parse);
        let first = tokens.next().unwrap();
        let source;

        if first.to_string() == "NOT" {
            source = WireSource::Gate(LogicGate::Not(
                WireValue::from_token(&tokens.next().unwrap()).unwrap(),
            ));

            tokens.next(); // ->
        } else {
            let val1 = WireValue::from_token(&first).unwrap();
            let op = tokens.next().unwrap();

            if op.to_string() == "->" {
                source = WireSource::Value(val1);
            } else {
                let val2 = WireValue::from_token(&tokens.next().unwrap()).unwrap();

                source = WireSource::Gate(LogicGate::binary_from_tokens(val1, &op, val2).unwrap());

                tokens.next(); // ->
            }
        }

        self.connections
            .insert(tokens.next().unwrap().to_string(), source);
    }

    fn val(&mut self, wire: &str) -> u16 {
        if self.state.contains_key(wire) {
            return self.state[wire];
        }

        let mut ask_stack = vec![wire.to_string()];

        while let Some(wire) = ask_stack.pop() {
            match self.connections[&wire].val(&self.state) {
                Ok(v) => {
                    self.state.insert(wire, v);
                }
                Err(e) => {
                    ask_stack.push(wire);
                    ask_stack.push(e.0);
                }
            }
        }

        self.state[wire]
    }
}

pub fn part_01(reader: Option<impl BufRead>) {
    let mut wires: LogicWires = Default::default();

    for line in reader.unwrap().lines().filter_map(|l| l.ok()) {
        wires.add_connection(&line);
    }

    println!("{:?}", wires.val("a"));
}

pub fn part_02(reader: Option<impl BufRead>) {
    let mut wires: LogicWires = Default::default();

    for line in reader.unwrap().lines().filter_map(|l| l.ok()) {
        wires.add_connection(&line);
    }

    wires.add_connection("956 -> b");
    println!("{:?}", wires.val("a"));
}

#[cfg(test)]
mod test {
    use super::LogicWires;

    #[test]
    fn bitwise() {
        let input = r#"123 -> x
456 -> y
x AND y -> d
x OR y -> e
x LSHIFT 2 -> f
y RSHIFT 2 -> g
NOT x -> h
NOT y -> i"#;

        let mut wires: LogicWires = Default::default();

        for l in input.lines() {
            wires.add_connection(l);
        }

        let expected: Vec<(&str, u16)> = vec![
            ("d", 72),
            ("e", 507),
            ("f", 492),
            ("g", 114),
            ("h", 65412),
            ("i", 65079),
            ("x", 123),
            ("y", 456),
        ];

        for (wire, val) in expected {
            assert_eq!(val, wires.val(wire));
        }
    }
}
