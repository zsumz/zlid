use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Json {
    Null,
    Bool(()),
    Number(i64),
    String(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

impl Json {
    pub(crate) fn into_object(self) -> BTreeMap<String, Json> {
        match self {
            Json::Object(value) => value,
            other => panic!("expected root object, got {other:?}"),
        }
    }
}

pub(crate) struct JsonParser<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> JsonParser<'a> {
    pub(crate) fn parse(input: &'a str) -> std::result::Result<Json, String> {
        let mut parser = JsonParser { input, index: 0 };
        let value = parser.parse_value()?;
        parser.skip_whitespace();
        if parser.is_end() {
            Ok(value)
        } else {
            Err(format!("trailing JSON input at offset {}", parser.index))
        }
    }

    fn parse_value(&mut self) -> std::result::Result<Json, String> {
        self.skip_whitespace();
        let ch = self
            .peek_char()
            .ok_or_else(|| "unexpected end of JSON input".to_string())?;
        match ch {
            '{' => self.parse_object(),
            '[' => self.parse_array(),
            '"' => self.parse_string().map(Json::String),
            't' => {
                self.expect_literal("true")?;
                Ok(Json::Bool(()))
            }
            'f' => {
                self.expect_literal("false")?;
                Ok(Json::Bool(()))
            }
            'n' => {
                self.expect_literal("null")?;
                Ok(Json::Null)
            }
            '-' | '0'..='9' => self.parse_number().map(Json::Number),
            _ => Err(format!(
                "unexpected JSON character {ch:?} at offset {}",
                self.index
            )),
        }
    }

    fn parse_object(&mut self) -> std::result::Result<Json, String> {
        self.expect_char('{')?;
        let mut map = BTreeMap::new();
        self.skip_whitespace();
        if self.consume_if('}') {
            return Ok(Json::Object(map));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect_char(':')?;
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            if self.consume_if('}') {
                return Ok(Json::Object(map));
            }
            self.expect_char(',')?;
        }
    }

    fn parse_array(&mut self) -> std::result::Result<Json, String> {
        self.expect_char('[')?;
        let mut values = Vec::new();
        self.skip_whitespace();
        if self.consume_if(']') {
            return Ok(Json::Array(values));
        }
        loop {
            values.push(self.parse_value()?);
            self.skip_whitespace();
            if self.consume_if(']') {
                return Ok(Json::Array(values));
            }
            self.expect_char(',')?;
        }
    }

    fn parse_string(&mut self) -> std::result::Result<String, String> {
        self.expect_char('"')?;
        let mut out = String::new();
        while let Some(ch) = self.next_char() {
            match ch {
                '"' => return Ok(out),
                '\\' => {
                    let escape = self
                        .next_char()
                        .ok_or_else(|| "unterminated JSON escape".to_string())?;
                    match escape {
                        '"' | '\\' | '/' => out.push(escape),
                        'b' => out.push('\u{0008}'),
                        'f' => out.push('\u{000c}'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => {
                            let code = self.read_hex4()?;
                            let ch = char::from_u32(code)
                                .ok_or_else(|| format!("invalid unicode escape {code:04X}"))?;
                            out.push(ch);
                        }
                        other => {
                            return Err(format!(
                                "invalid JSON escape \\{other} at offset {}",
                                self.index
                            ));
                        }
                    }
                }
                other => out.push(other),
            }
        }
        Err("unterminated JSON string".to_string())
    }

    fn parse_number(&mut self) -> std::result::Result<i64, String> {
        let start = self.index;
        self.consume_if('-');
        if self.consume_if('0') {
            // Single leading zero is allowed; additional digits are handled by parse().
        } else {
            self.read_digits()?;
        }
        if matches!(self.peek_char(), Some('.') | Some('e') | Some('E')) {
            return Err("floating point numbers are not used in the fixture".to_string());
        }
        self.input[start..self.index]
            .parse::<i64>()
            .map_err(|error| format!("invalid JSON number at offset {start}: {error}"))
    }

    fn read_digits(&mut self) -> std::result::Result<(), String> {
        let start = self.index;
        while matches!(self.peek_char(), Some('0'..='9')) {
            self.next_char();
        }
        if self.index == start {
            Err(format!("expected JSON digits at offset {}", self.index))
        } else {
            Ok(())
        }
    }

    fn read_hex4(&mut self) -> std::result::Result<u32, String> {
        let start = self.index;
        let mut value = 0u32;
        for _ in 0..4 {
            let ch = self
                .next_char()
                .ok_or_else(|| "short JSON unicode escape".to_string())?;
            let digit = ch
                .to_digit(16)
                .ok_or_else(|| format!("invalid unicode escape at offset {start}"))?;
            value = (value << 4) | digit;
        }
        Ok(value)
    }

    fn expect_literal(&mut self, literal: &str) -> std::result::Result<(), String> {
        if self.input[self.index..].starts_with(literal) {
            self.index += literal.len();
            Ok(())
        } else {
            Err(format!("expected {literal} at offset {}", self.index))
        }
    }

    fn expect_char(&mut self, expected: char) -> std::result::Result<(), String> {
        self.skip_whitespace();
        match self.next_char() {
            Some(actual) if actual == expected => Ok(()),
            Some(actual) => Err(format!(
                "expected {expected:?} at offset {}, got {actual:?}",
                self.index
            )),
            None => Err(format!("expected {expected:?} at end of input")),
        }
    }

    fn consume_if(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.next_char();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_char(), Some(' ' | '\n' | '\r' | '\t')) {
            self.next_char();
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.index..].chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.index += ch.len_utf8();
        Some(ch)
    }

    fn is_end(&self) -> bool {
        self.index >= self.input.len()
    }
}

pub(crate) fn get<'a>(object: &'a BTreeMap<String, Json>, key: &str) -> &'a Json {
    object
        .get(key)
        .unwrap_or_else(|| panic!("missing key {key}"))
}

pub(crate) fn object(value: &Json) -> &BTreeMap<String, Json> {
    match value {
        Json::Object(value) => value,
        other => panic!("expected object, got {other:?}"),
    }
}

pub(crate) fn array(value: &Json) -> &[Json] {
    match value {
        Json::Array(value) => value,
        other => panic!("expected array, got {other:?}"),
    }
}

pub(crate) fn string(value: &Json) -> &str {
    match value {
        Json::String(value) => value,
        other => panic!("expected string, got {other:?}"),
    }
}

pub(crate) fn number(value: &Json) -> i64 {
    match value {
        Json::Number(value) => *value,
        other => panic!("expected number, got {other:?}"),
    }
}
