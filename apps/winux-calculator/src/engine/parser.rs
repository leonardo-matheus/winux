// Winux Calculator - Expression Parser
// Copyright (c) 2026 Winux OS Project

/// Token types for the parser
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Operator(Operator),
    Function(String),
    OpenParen,
    CloseParen,
    Constant(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Modulo,
    // Bitwise operators (for programmer mode)
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseNot,
    ShiftLeft,
    ShiftRight,
}

impl Operator {
    pub fn precedence(&self) -> u8 {
        match self {
            Operator::BitwiseOr | Operator::BitwiseXor => 1,
            Operator::BitwiseAnd => 2,
            Operator::ShiftLeft | Operator::ShiftRight => 3,
            Operator::Add | Operator::Subtract => 4,
            Operator::Multiply | Operator::Divide | Operator::Modulo => 5,
            Operator::Power => 6,
            Operator::BitwiseNot => 7,
        }
    }

    pub fn is_right_associative(&self) -> bool {
        matches!(self, Operator::Power)
    }
}

/// Expression parser
pub struct Parser {
    // Reserved for future state
}

impl Parser {
    pub fn new() -> Self {
        Self {}
    }

    /// Parse an expression string into tokens
    pub fn parse(&self, expr: &str) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = expr.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            // Skip whitespace
            if ch.is_whitespace() {
                i += 1;
                continue;
            }

            // Number (including negative numbers at start or after operator)
            if ch.is_ascii_digit() || ch == '.' ||
               (ch == '-' && (tokens.is_empty() ||
                matches!(tokens.last(), Some(Token::Operator(_)) | Some(Token::OpenParen)))) {
                let start = i;

                // Handle negative sign
                if ch == '-' {
                    i += 1;
                }

                // Integer part
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }

                // Decimal part
                if i < chars.len() && chars[i] == '.' {
                    i += 1;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                }

                // Scientific notation
                if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                    i += 1;
                    if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                        i += 1;
                    }
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                }

                let num_str: String = chars[start..i].iter().collect();
                match num_str.parse::<f64>() {
                    Ok(n) => tokens.push(Token::Number(n)),
                    Err(_) => return Err(format!("Numero invalido: {}", num_str)),
                }
                continue;
            }

            // Operators and symbols
            match ch {
                '+' => tokens.push(Token::Operator(Operator::Add)),
                '-' => tokens.push(Token::Operator(Operator::Subtract)),
                '*' | 'x' | 'X' => tokens.push(Token::Operator(Operator::Multiply)),
                '/' => tokens.push(Token::Operator(Operator::Divide)),
                '^' => tokens.push(Token::Operator(Operator::Power)),
                '%' => tokens.push(Token::Operator(Operator::Modulo)),
                '(' => tokens.push(Token::OpenParen),
                ')' => tokens.push(Token::CloseParen),
                '&' => tokens.push(Token::Operator(Operator::BitwiseAnd)),
                '|' => tokens.push(Token::Operator(Operator::BitwiseOr)),
                '~' => tokens.push(Token::Operator(Operator::BitwiseNot)),
                '<' => {
                    if i + 1 < chars.len() && chars[i + 1] == '<' {
                        tokens.push(Token::Operator(Operator::ShiftLeft));
                        i += 1;
                    } else {
                        return Err("Operador invalido: <".to_string());
                    }
                }
                '>' => {
                    if i + 1 < chars.len() && chars[i + 1] == '>' {
                        tokens.push(Token::Operator(Operator::ShiftRight));
                        i += 1;
                    } else {
                        return Err("Operador invalido: >".to_string());
                    }
                }
                _ if ch.is_alphabetic() => {
                    // Function or constant name
                    let start = i;
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    let name: String = chars[start..i].iter().collect();
                    let name_lower = name.to_lowercase();

                    // Check if it's a constant
                    match name_lower.as_str() {
                        "pi" => tokens.push(Token::Constant("pi".to_string())),
                        "e" if i >= chars.len() || !chars[i].is_alphabetic() => {
                            tokens.push(Token::Constant("e".to_string()))
                        }
                        "phi" => tokens.push(Token::Constant("phi".to_string())),
                        // XOR operator
                        "xor" => tokens.push(Token::Operator(Operator::BitwiseXor)),
                        "and" => tokens.push(Token::Operator(Operator::BitwiseAnd)),
                        "or" => tokens.push(Token::Operator(Operator::BitwiseOr)),
                        "not" => tokens.push(Token::Operator(Operator::BitwiseNot)),
                        "shl" => tokens.push(Token::Operator(Operator::ShiftLeft)),
                        "shr" => tokens.push(Token::Operator(Operator::ShiftRight)),
                        "mod" => tokens.push(Token::Operator(Operator::Modulo)),
                        // Otherwise it's a function
                        _ => tokens.push(Token::Function(name_lower)),
                    }
                    continue;
                }
                _ => return Err(format!("Caractere invalido: {}", ch)),
            }
            i += 1;
        }

        Ok(tokens)
    }

    /// Parse a hexadecimal string to number
    pub fn parse_hex(s: &str) -> Result<i64, String> {
        let s = s.trim_start_matches("0x").trim_start_matches("0X");
        i64::from_str_radix(s, 16).map_err(|_| "Numero hexadecimal invalido".to_string())
    }

    /// Parse a binary string to number
    pub fn parse_binary(s: &str) -> Result<i64, String> {
        let s = s.trim_start_matches("0b").trim_start_matches("0B");
        i64::from_str_radix(s, 2).map_err(|_| "Numero binario invalido".to_string())
    }

    /// Parse an octal string to number
    pub fn parse_octal(s: &str) -> Result<i64, String> {
        let s = s.trim_start_matches("0o").trim_start_matches("0O");
        i64::from_str_radix(s, 8).map_err(|_| "Numero octal invalido".to_string())
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
