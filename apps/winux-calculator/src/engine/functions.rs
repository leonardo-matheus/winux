// Winux Calculator - Mathematical Functions
// Copyright (c) 2026 Winux OS Project

use super::AngleMode;

/// Collection of mathematical functions
pub struct MathFunctions;

impl MathFunctions {
    /// Apply a named function to a value
    pub fn apply(name: &str, value: f64, angle_mode: AngleMode) -> Result<f64, String> {
        match name.to_lowercase().as_str() {
            // Trigonometric functions
            "sin" => Ok(Self::sin(value, angle_mode)),
            "cos" => Ok(Self::cos(value, angle_mode)),
            "tan" => Ok(Self::tan(value, angle_mode)),
            "asin" | "arcsin" => Self::asin(value, angle_mode),
            "acos" | "arccos" => Self::acos(value, angle_mode),
            "atan" | "arctan" => Ok(Self::atan(value, angle_mode)),

            // Hyperbolic functions
            "sinh" => Ok(value.sinh()),
            "cosh" => Ok(value.cosh()),
            "tanh" => Ok(value.tanh()),
            "asinh" | "arcsinh" => Ok(value.asinh()),
            "acosh" | "arccosh" => {
                if value < 1.0 {
                    Err("Valor deve ser >= 1 para acosh".to_string())
                } else {
                    Ok(value.acosh())
                }
            }
            "atanh" | "arctanh" => {
                if value.abs() >= 1.0 {
                    Err("Valor deve estar entre -1 e 1 para atanh".to_string())
                } else {
                    Ok(value.atanh())
                }
            }

            // Logarithmic functions
            "ln" | "log" => {
                if value <= 0.0 {
                    Err("Logaritmo de numero nao-positivo".to_string())
                } else {
                    Ok(value.ln())
                }
            }
            "log10" => {
                if value <= 0.0 {
                    Err("Logaritmo de numero nao-positivo".to_string())
                } else {
                    Ok(value.log10())
                }
            }
            "log2" => {
                if value <= 0.0 {
                    Err("Logaritmo de numero nao-positivo".to_string())
                } else {
                    Ok(value.log2())
                }
            }
            "exp" => Ok(value.exp()),
            "exp2" => Ok(value.exp2()),

            // Power and root functions
            "sqrt" => {
                if value < 0.0 {
                    Err("Raiz quadrada de numero negativo".to_string())
                } else {
                    Ok(value.sqrt())
                }
            }
            "cbrt" => Ok(value.cbrt()),
            "sq" | "square" => Ok(value * value),
            "cube" => Ok(value * value * value),

            // Rounding functions
            "abs" => Ok(value.abs()),
            "floor" => Ok(value.floor()),
            "ceil" => Ok(value.ceil()),
            "round" => Ok(value.round()),
            "trunc" => Ok(value.trunc()),
            "frac" => Ok(value.fract()),

            // Factorial
            "fact" | "factorial" => Self::factorial(value),

            // Reciprocal
            "inv" | "reciprocal" => {
                if value == 0.0 {
                    Err("Divisao por zero".to_string())
                } else {
                    Ok(1.0 / value)
                }
            }

            // Degrees/Radians conversion
            "deg" | "todeg" => Ok(value.to_degrees()),
            "rad" | "torad" => Ok(value.to_radians()),

            // Sign function
            "sign" | "sgn" => Ok(value.signum()),

            _ => Err(format!("Funcao desconhecida: {}", name)),
        }
    }

    /// Sine with angle mode support
    fn sin(value: f64, mode: AngleMode) -> f64 {
        match mode {
            AngleMode::Degrees => value.to_radians().sin(),
            AngleMode::Radians => value.sin(),
        }
    }

    /// Cosine with angle mode support
    fn cos(value: f64, mode: AngleMode) -> f64 {
        match mode {
            AngleMode::Degrees => value.to_radians().cos(),
            AngleMode::Radians => value.cos(),
        }
    }

    /// Tangent with angle mode support
    fn tan(value: f64, mode: AngleMode) -> f64 {
        match mode {
            AngleMode::Degrees => value.to_radians().tan(),
            AngleMode::Radians => value.tan(),
        }
    }

    /// Arc sine with angle mode support
    fn asin(value: f64, mode: AngleMode) -> Result<f64, String> {
        if value < -1.0 || value > 1.0 {
            return Err("Valor deve estar entre -1 e 1 para asin".to_string());
        }
        let result = value.asin();
        Ok(match mode {
            AngleMode::Degrees => result.to_degrees(),
            AngleMode::Radians => result,
        })
    }

    /// Arc cosine with angle mode support
    fn acos(value: f64, mode: AngleMode) -> Result<f64, String> {
        if value < -1.0 || value > 1.0 {
            return Err("Valor deve estar entre -1 e 1 para acos".to_string());
        }
        let result = value.acos();
        Ok(match mode {
            AngleMode::Degrees => result.to_degrees(),
            AngleMode::Radians => result,
        })
    }

    /// Arc tangent with angle mode support
    fn atan(value: f64, mode: AngleMode) -> f64 {
        let result = value.atan();
        match mode {
            AngleMode::Degrees => result.to_degrees(),
            AngleMode::Radians => result,
        }
    }

    /// Factorial function (for non-negative integers up to 170)
    fn factorial(value: f64) -> Result<f64, String> {
        if value < 0.0 {
            return Err("Fatorial de numero negativo".to_string());
        }
        if value.fract() != 0.0 {
            // Use gamma function for non-integers
            return Ok(Self::gamma(value + 1.0));
        }
        let n = value as u64;
        if n > 170 {
            return Err("Fatorial muito grande (max 170)".to_string());
        }

        let mut result = 1.0_f64;
        for i in 2..=n {
            result *= i as f64;
        }
        Ok(result)
    }

    /// Gamma function approximation (Stirling's approximation for large values)
    fn gamma(x: f64) -> f64 {
        // Lanczos approximation coefficients
        const G: f64 = 7.0;
        const C: [f64; 9] = [
            0.99999999999980993,
            676.5203681218851,
            -1259.1392167224028,
            771.32342877765313,
            -176.61502916214059,
            12.507343278686905,
            -0.13857109526572012,
            9.9843695780195716e-6,
            1.5056327351493116e-7,
        ];

        if x < 0.5 {
            std::f64::consts::PI / ((std::f64::consts::PI * x).sin() * Self::gamma(1.0 - x))
        } else {
            let x = x - 1.0;
            let mut a = C[0];
            for i in 1..9 {
                a += C[i] / (x + i as f64);
            }
            let t = x + G + 0.5;
            (2.0 * std::f64::consts::PI).sqrt() * t.powf(x + 0.5) * (-t).exp() * a
        }
    }

    /// Calculate nth root
    pub fn nth_root(value: f64, n: f64) -> Result<f64, String> {
        if n == 0.0 {
            return Err("Raiz de indice zero".to_string());
        }
        if value < 0.0 && n.fract() == 0.0 && (n as i64) % 2 == 0 {
            return Err("Raiz par de numero negativo".to_string());
        }
        Ok(value.powf(1.0 / n))
    }

    /// Calculate power
    pub fn power(base: f64, exp: f64) -> f64 {
        base.powf(exp)
    }

    /// Calculate percentage of a value
    pub fn percentage(value: f64, percent: f64) -> f64 {
        value * percent / 100.0
    }
}
