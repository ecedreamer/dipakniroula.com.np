
#[derive(Debug, PartialEq)]
enum Token {
    LeftBrace,      // `{`
    RightBrace,     // `}`
    LeftBracket,    // `[`
    RightBracket,   // `]`
    Colon,          // `:`
    Comma,          // `,`
    String(String), // "..."
    Number(f64),    // 1234.56
    Boolean(bool),  // true or false
    Null,           // null
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            '{' => { tokens.push(Token::LeftBrace); chars.next(); }
            '}' => { tokens.push(Token::RightBrace); chars.next(); }
            '[' => { tokens.push(Token::LeftBracket); chars.next(); }
            ']' => { tokens.push(Token::RightBracket); chars.next(); }
            ':' => { tokens.push(Token::Colon); chars.next(); }
            ',' => { tokens.push(Token::Comma); chars.next(); }
            '"' => {
                chars.next(); // consume the opening quote
                let mut string = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '"' { break; }
                    string.push(c);
                    chars.next();
                }
                chars.next(); // consume the closing quote
                tokens.push(Token::String(string));
            }
            '0'..='9' | '-' => {
                let mut num_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_digit(10) || c == '.' || c == '-' {
                        num_str.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let number = num_str.parse::<f64>().map_err(|e| e.to_string())?;
                tokens.push(Token::Number(number));
            }
            't' => {
                let token: String = chars.by_ref().take(4).collect();
                if token == "true" {
                    tokens.push(Token::Boolean(true));
                } else {
                    return Err("Unexpected token".into());
                }
            }
            'f' => {
                let token: String = chars.by_ref().take(5).collect();
                if token == "false" {
                    tokens.push(Token::Boolean(false));
                } else {
                    return Err("Unexpected token".into());
                }
            }
            'n' => {
                let token: String = chars.by_ref().take(4).collect();
                if token == "null" {
                    tokens.push(Token::Null);
                } else {
                    return Err("Unexpected token".into());
                }
            }
            _ if ch.is_whitespace() => { chars.next(); } // Ignore whitespace
            _ => return Err(format!("Unexpected character: {}", ch)),
        }
    }
    Ok(tokens)
}

#[derive(Debug, PartialEq)]
enum JsonValue {
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn parse(&mut self) -> Result<JsonValue, String> {
        self.parse_value()
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        match self.tokens.get(self.pos) {
            Some(Token::LeftBrace) => self.parse_object(),
            Some(Token::LeftBracket) => self.parse_array(),
            Some(Token::String(s)) => {
                self.pos += 1;
                Ok(JsonValue::String(s.clone()))
            }
            Some(Token::Number(n)) => {
                self.pos += 1;
                Ok(JsonValue::Number(*n))
            }
            Some(Token::Boolean(b)) => {
                self.pos += 1;
                Ok(JsonValue::Boolean(*b))
            }
            Some(Token::Null) => {
                self.pos += 1;
                Ok(JsonValue::Null)
            }
            _ => Err("Unexpected token".into()),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // skip '{'
        let mut object = Vec::new();

        while self.tokens.get(self.pos) != Some(&Token::RightBrace) {
            if self.pos >= self.tokens.len() {
                return Err("Unexpected end of input".into());
            }
            let key = match self.tokens.get(self.pos) {
                Some(Token::String(s)) => s.clone(),
                _ => return Err("Expected string as key".into()),
            };
            self.pos += 1; // skip the key
            if self.tokens.get(self.pos) != Some(&Token::Colon) {
                return Err("Expected colon after key".into());
            }
            self.pos += 1; // skip ':'

            let value = self.parse_value()?;
            object.push((key, value));

            match self.tokens.get(self.pos) {
                Some(Token::Comma) => self.pos += 1,
                Some(Token::RightBrace) => break,
                _ => return Err("Expected comma or '}'".into()),
            }
        }
        self.pos += 1; // skip '}'
        Ok(JsonValue::Object(object))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // skip '['
        let mut array = Vec::new();

        while self.tokens.get(self.pos) != Some(&Token::RightBracket) {
            if self.pos >= self.tokens.len() {
                return Err("Unexpected end of input".into());
            }

            let value = self.parse_value()?;
            array.push(value);

            match self.tokens.get(self.pos) {
                Some(Token::Comma) => self.pos += 1,
                Some(Token::RightBracket) => break,
                _ => return Err("Expected comma or ']'".into()),
            }
        }
        self.pos += 1; // skip ']'
        Ok(JsonValue::Array(array))
    }
}


fn parse_json(json_str: &str) -> Result<JsonValue, String> {
    match tokenize(json_str) {
        Ok(tokens) => {
            let mut parser = Parser::new(tokens);
            parser.parse()
        }
        Err(err) => {
            Err("Tremendous error".to_string())
        }
    }
}

fn main() {
    let input = r#"{"name": "John", "interests": ["Travelling", "Reading"], "age": 30, "is_student": false, "address": {"city": "New York"}}"#;
    let result = parse_json(input);
    match result {
        Ok(parsed) => println!("{:#?}", parsed),
        Err(err) => println!("Error: {}", err)
    }
}


/*
<p>Website: <a href="https://dipakniroula.com.np" target="_blank">https://dipakniroula.com.np</a></p>
<p>Github Profile: <a href="https://github.com/ecedreamer" target="_blank">https://github.com/ecedreamer</a></p>
<p>Linkedin Profile: <a href="https://linkedin.com/in/dipak-niroula-90b11610b/" target="_blank">https://linkedin.com/in/dipak-niroula-90b11610b/</a></p>
<p>Youtube Channel: <a href="https://www.youtube.com/@sangitniroula/videos" target="_blank">https://youtube.com/@sangitniroula</a></p>
 */