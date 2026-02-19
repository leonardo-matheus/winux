// Winux Calculator - Expression Evaluator
// Copyright (c) 2026 Winux OS Project

use super::parser::{Token, Operator};
use super::functions::MathFunctions;
use super::AngleMode;

/// Evaluator for parsed expressions using shunting-yard algorithm
pub struct Evaluator {
    // Reserved for future state
}

impl Evaluator {
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluate a list of tokens using shunting-yard algorithm
    pub fn evaluate(&self, tokens: &[Token], angle_mode: AngleMode) -> Result<f64, String> {
        // Convert to postfix (Reverse Polish Notation)
        let postfix = self.to_postfix(tokens)?;

        // Evaluate postfix expression
        self.evaluate_postfix(&postfix, angle_mode)
    }

    /// Convert infix tokens to postfix using shunting-yard algorithm
    fn to_postfix(&self, tokens: &[Token]) -> Result<Vec<Token>, String> {
        let mut output: Vec<Token> = Vec::new();
        let mut operator_stack: Vec<Token> = Vec::new();

        for token in tokens {
            match token {
                Token::Number(_) | Token::Constant(_) => {
                    output.push(token.clone());
                }
                Token::Function(_) => {
                    operator_stack.push(token.clone());
                }
                Token::Operator(op) => {
                    while let Some(top) = operator_stack.last() {
                        match top {
                            Token::Operator(top_op) => {
                                let should_pop = if op.is_right_associative() {
                                    top_op.precedence() > op.precedence()
                                } else {
                                    top_op.precedence() >= op.precedence()
                                };
                                if should_pop {
                                    output.push(operator_stack.pop().unwrap());
                                } else {
                                    break;
                                }
                            }
                            Token::Function(_) => {
                                break;
                            }
                            _ => break,
                        }
                    }
                    operator_stack.push(token.clone());
                }
                Token::OpenParen => {
                    operator_stack.push(token.clone());
                }
                Token::CloseParen => {
                    while let Some(top) = operator_stack.last() {
                        if matches!(top, Token::OpenParen) {
                            break;
                        }
                        output.push(operator_stack.pop().unwrap());
                    }
                    if operator_stack.is_empty() {
                        return Err("Parenteses desbalanceados".to_string());
                    }
                    operator_stack.pop(); // Remove open paren

                    // If there's a function before the paren, pop it too
                    if let Some(Token::Function(_)) = operator_stack.last() {
                        output.push(operator_stack.pop().unwrap());
                    }
                }
            }
        }

        // Pop remaining operators
        while let Some(token) = operator_stack.pop() {
            if matches!(token, Token::OpenParen | Token::CloseParen) {
                return Err("Parenteses desbalanceados".to_string());
            }
            output.push(token);
        }

        Ok(output)
    }

    /// Evaluate a postfix expression
    fn evaluate_postfix(&self, tokens: &[Token], angle_mode: AngleMode) -> Result<f64, String> {
        let mut stack: Vec<f64> = Vec::new();

        for token in tokens {
            match token {
                Token::Number(n) => {
                    stack.push(*n);
                }
                Token::Constant(name) => {
                    let value = match name.as_str() {
                        "pi" => std::f64::consts::PI,
                        "e" => std::f64::consts::E,
                        "phi" => 1.618033988749895,
                        _ => return Err(format!("Constante desconhecida: {}", name)),
                    };
                    stack.push(value);
                }
                Token::Operator(op) => {
                    // Handle unary NOT
                    if matches!(op, Operator::BitwiseNot) {
                        let a = stack.pop().ok_or("Expressao invalida")?;
                        let result = !(a as i64) as f64;
                        stack.push(result);
                        continue;
                    }

                    // Binary operators
                    let b = stack.pop().ok_or("Expressao invalida")?;
                    let a = stack.pop().ok_or("Expressao invalida")?;

                    let result = match op {
                        Operator::Add => a + b,
                        Operator::Subtract => a - b,
                        Operator::Multiply => a * b,
                        Operator::Divide => {
                            if b == 0.0 {
                                return Err("Divisao por zero".to_string());
                            }
                            a / b
                        }
                        Operator::Power => a.powf(b),
                        Operator::Modulo => {
                            if b == 0.0 {
                                return Err("Divisao por zero".to_string());
                            }
                            a % b
                        }
                        Operator::BitwiseAnd => ((a as i64) & (b as i64)) as f64,
                        Operator::BitwiseOr => ((a as i64) | (b as i64)) as f64,
                        Operator::BitwiseXor => ((a as i64) ^ (b as i64)) as f64,
                        Operator::ShiftLeft => ((a as i64) << (b as i64)) as f64,
                        Operator::ShiftRight => ((a as i64) >> (b as i64)) as f64,
                        Operator::BitwiseNot => unreachable!(),
                    };
                    stack.push(result);
                }
                Token::Function(name) => {
                    let a = stack.pop().ok_or("Expressao invalida")?;
                    let result = MathFunctions::apply(name, a, angle_mode)?;
                    stack.push(result);
                }
                Token::OpenParen | Token::CloseParen => {
                    return Err("Erro interno: parentese em postfix".to_string());
                }
            }
        }

        if stack.len() != 1 {
            return Err("Expressao invalida".to_string());
        }

        Ok(stack.pop().unwrap())
    }

    /// Evaluate bitwise operations for programmer mode
    pub fn evaluate_bitwise(&self, a: i64, op: &str, b: i64) -> Result<i64, String> {
        match op.to_lowercase().as_str() {
            "and" | "&" => Ok(a & b),
            "or" | "|" => Ok(a | b),
            "xor" | "^" => Ok(a ^ b),
            "not" | "~" => Ok(!a),
            "shl" | "<<" => Ok(a << b),
            "shr" | ">>" => Ok(a >> b),
            _ => Err(format!("Operador bitwise desconhecido: {}", op)),
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
