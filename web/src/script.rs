//! Script runner for command-oriented calculator automation.

use serde::{Deserialize, Serialize};

use crate::api::{
    ApiAngleMode, ApiDisplayMode, ApiResponse, ApiState, CalculatorApi, ComplexInput, MatrixInput,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptLogEntry {
    pub line: usize,
    pub command: String,
    pub ok: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptError {
    pub code: String,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptResponse {
    pub ok: bool,
    pub state: ApiState,
    pub error: Option<ScriptError>,
    pub warning: Option<String>,
    pub transcript: Vec<ScriptLogEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ScriptRunOutcome {
    pub warning: Option<String>,
    pub transcript: Vec<ScriptLogEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ScriptRunFailure {
    pub error: ScriptError,
    pub transcript: Vec<ScriptLogEntry>,
}

#[derive(Debug, Clone, PartialEq)]
struct ScriptToken {
    text: String,
    column: usize,
}

pub(crate) fn run_script_text(
    api: &mut CalculatorApi,
    script: &str,
) -> Result<ScriptRunOutcome, ScriptRunFailure> {
    let mut transcript = Vec::new();
    let mut warnings = Vec::new();

    for (line_index, raw_line) in script.lines().enumerate() {
        let line_number = line_index + 1;
        let tokens = tokenize_line(raw_line, line_number, &transcript)?;
        if tokens.is_empty() {
            continue;
        }
        execute_tokens(api, line_number, &tokens, &mut transcript, &mut warnings)?;
    }

    Ok(ScriptRunOutcome {
        warning: collapse_warnings(&warnings),
        transcript,
    })
}

pub(crate) fn run_script_line_text(
    api: &mut CalculatorApi,
    line: &str,
) -> Result<ScriptRunOutcome, ScriptRunFailure> {
    let tokens = tokenize_line(line, 1, &[])?;
    let mut transcript = Vec::new();
    let mut warnings = Vec::new();
    if !tokens.is_empty() {
        execute_tokens(api, 1, &tokens, &mut transcript, &mut warnings)?;
    }
    Ok(ScriptRunOutcome {
        warning: collapse_warnings(&warnings),
        transcript,
    })
}

fn collapse_warnings(warnings: &[String]) -> Option<String> {
    if warnings.is_empty() {
        return None;
    }
    let mut unique = Vec::new();
    for warning in warnings {
        if !unique.contains(warning) {
            unique.push(warning.clone());
        }
    }
    Some(unique.join(" | "))
}

fn tokenize_line(
    raw_line: &str,
    line_number: usize,
    transcript: &[ScriptLogEntry],
) -> Result<Vec<ScriptToken>, ScriptRunFailure> {
    let mut tokens = Vec::new();
    let chars = raw_line.chars().collect::<Vec<_>>();
    let mut i = 0usize;
    let mut current = String::new();
    let mut start = 0usize;
    let mut bracket_depth = 0i32;
    let mut paren_depth = 0i32;

    while i < chars.len() {
        let ch = chars[i];
        let at_comment = ch == '#' && bracket_depth == 0 && paren_depth == 0;
        let at_slash_comment = ch == '/'
            && i + 1 < chars.len()
            && chars[i + 1] == '/'
            && bracket_depth == 0
            && paren_depth == 0;
        if at_comment || at_slash_comment {
            break;
        }

        if current.is_empty() && !ch.is_whitespace() {
            start = i + 1;
        }

        match ch {
            '[' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                bracket_depth -= 1;
                if bracket_depth < 0 {
                    return Err(parse_failure(
                        "invalid_input",
                        "unmatched ']' in script line",
                        line_number,
                        i + 1,
                        Some("]".to_string()),
                        transcript,
                    ));
                }
                current.push(ch);
            }
            '(' => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' => {
                paren_depth -= 1;
                if paren_depth < 0 {
                    return Err(parse_failure(
                        "invalid_input",
                        "unmatched ')' in script line",
                        line_number,
                        i + 1,
                        Some(")".to_string()),
                        transcript,
                    ));
                }
                current.push(ch);
            }
            _ if ch.is_whitespace() && bracket_depth == 0 && paren_depth == 0 => {
                if !current.trim().is_empty() {
                    tokens.push(ScriptToken {
                        text: current.trim().to_string(),
                        column: start,
                    });
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
        i += 1;
    }

    if bracket_depth != 0 {
        return Err(parse_failure(
            "invalid_input",
            "unterminated matrix literal in script line",
            line_number,
            raw_line.len().max(1),
            None,
            transcript,
        ));
    }
    if paren_depth != 0 {
        return Err(parse_failure(
            "invalid_input",
            "unterminated complex literal in script line",
            line_number,
            raw_line.len().max(1),
            None,
            transcript,
        ));
    }

    if !current.trim().is_empty() {
        tokens.push(ScriptToken {
            text: current.trim().to_string(),
            column: start,
        });
    }

    Ok(tokens)
}

fn execute_tokens(
    api: &mut CalculatorApi,
    line_number: usize,
    tokens: &[ScriptToken],
    transcript: &mut Vec<ScriptLogEntry>,
    warnings: &mut Vec<String>,
) -> Result<(), ScriptRunFailure> {
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];

        if let Some(value) = parse_real_literal(&token.text) {
            let response = api.push_real(value);
            apply_response(transcript, warnings, line_number, token.column, &token.text, response)?;
            index += 1;
            continue;
        }

        if let Some(complex) = parse_complex_literal(&token.text) {
            let response = api.push_complex(complex);
            apply_response(transcript, warnings, line_number, token.column, &token.text, response)?;
            index += 1;
            continue;
        }

        if token.text.starts_with('[') {
            let matrix = parse_matrix_literal(&token.text, line_number, token.column, transcript)?;
            let response = api.push_matrix(matrix);
            apply_response(transcript, warnings, line_number, token.column, &token.text, response)?;
            index += 1;
            continue;
        }

        let command = token.text.to_ascii_lowercase();

        if let Some(response) = execute_zero_arg_command(api, &command) {
            apply_response(transcript, warnings, line_number, token.column, &token.text, response)?;
            index += 1;
            continue;
        }

        match command.as_str() {
            "roll" => {
                let arg = expect_argument(tokens, index, line_number, transcript)?;
                let count = parse_positive_usize(&arg.text, "roll count", line_number, arg.column, transcript)?;
                let response = api.roll(count);
                apply_response(
                    transcript,
                    warnings,
                    line_number,
                    token.column,
                    &format!("{} {}", token.text, arg.text),
                    response,
                )?;
                index += 2;
            }
            "pick" => {
                let arg = expect_argument(tokens, index, line_number, transcript)?;
                let depth = parse_positive_usize(&arg.text, "pick depth", line_number, arg.column, transcript)?;
                let response = api.pick(depth);
                apply_response(
                    transcript,
                    warnings,
                    line_number,
                    token.column,
                    &format!("{} {}", token.text, arg.text),
                    response,
                )?;
                index += 2;
            }
            "memory_store" | "store" => {
                let arg = expect_argument(tokens, index, line_number, transcript)?;
                let register = parse_register(&arg.text, line_number, arg.column, transcript)?;
                let response = api.memory_store(register);
                apply_response(
                    transcript,
                    warnings,
                    line_number,
                    token.column,
                    &format!("store {}", arg.text.to_ascii_uppercase()),
                    response,
                )?;
                index += 2;
            }
            "memory_recall" | "recall" => {
                let arg = expect_argument(tokens, index, line_number, transcript)?;
                let register = parse_register(&arg.text, line_number, arg.column, transcript)?;
                let response = api.memory_recall(register);
                apply_response(
                    transcript,
                    warnings,
                    line_number,
                    token.column,
                    &format!("recall {}", arg.text.to_ascii_uppercase()),
                    response,
                )?;
                index += 2;
            }
            "memory_clear" | "memclear" => {
                let arg = expect_argument(tokens, index, line_number, transcript)?;
                let register = parse_register(&arg.text, line_number, arg.column, transcript)?;
                let response = api.memory_clear(register);
                apply_response(
                    transcript,
                    warnings,
                    line_number,
                    token.column,
                    &format!("memclear {}", arg.text.to_ascii_uppercase()),
                    response,
                )?;
                index += 2;
            }
            "push_identity" | "identity" => {
                let arg = expect_argument(tokens, index, line_number, transcript)?;
                let size = parse_positive_usize(&arg.text, "identity size", line_number, arg.column, transcript)?;
                let response = api.push_identity(size);
                apply_response(
                    transcript,
                    warnings,
                    line_number,
                    token.column,
                    &format!("identity {}", arg.text),
                    response,
                )?;
                index += 2;
            }
            "set_precision" | "precision" => {
                let arg = expect_argument(tokens, index, line_number, transcript)?;
                let precision = parse_precision(&arg.text, line_number, arg.column, transcript)?;
                let response = api.set_precision(precision);
                apply_response(
                    transcript,
                    warnings,
                    line_number,
                    token.column,
                    &format!("precision {}", arg.text),
                    response,
                )?;
                index += 2;
            }
            "set_display_mode" | "display" => {
                let arg = expect_argument(tokens, index, line_number, transcript)?;
                let mode = parse_display_mode(&arg.text, line_number, arg.column, transcript)?;
                let response = api.set_display_mode(mode);
                apply_response(
                    transcript,
                    warnings,
                    line_number,
                    token.column,
                    &format!("display {}", arg.text.to_ascii_lowercase()),
                    response,
                )?;
                index += 2;
            }
            "set_angle_mode" | "angle" => {
                let arg = expect_argument(tokens, index, line_number, transcript)?;
                let mode = parse_angle_mode(&arg.text, line_number, arg.column, transcript)?;
                let response = api.set_angle_mode(mode);
                apply_response(
                    transcript,
                    warnings,
                    line_number,
                    token.column,
                    &format!("angle {}", arg.text.to_ascii_lowercase()),
                    response,
                )?;
                index += 2;
            }
            "entry_set" | "entry" => {
                let arg = expect_argument(tokens, index, line_number, transcript)?;
                let response = api.entry_set(&arg.text);
                apply_response(
                    transcript,
                    warnings,
                    line_number,
                    token.column,
                    &format!("entry {}", arg.text),
                    response,
                )?;
                index += 2;
            }
            "matrix" | "push_matrix" => {
                let arg = expect_argument(tokens, index, line_number, transcript)?;
                let matrix = parse_matrix_literal(&arg.text, line_number, arg.column, transcript)?;
                let response = api.push_matrix(matrix);
                apply_response(
                    transcript,
                    warnings,
                    line_number,
                    token.column,
                    &format!("matrix {}", arg.text),
                    response,
                )?;
                index += 2;
            }
            _ => {
                return Err(parse_failure(
                    "unknown_command",
                    &format!("unknown script command: {}", token.text),
                    line_number,
                    token.column,
                    Some(token.text.clone()),
                    transcript,
                ));
            }
        }
    }

    Ok(())
}

fn execute_zero_arg_command(api: &mut CalculatorApi, command: &str) -> Option<ApiResponse> {
    match command {
        "add" | "+" => Some(api.add()),
        "sub" | "-" => Some(api.sub()),
        "mul" | "*" | "x" => Some(api.mul()),
        "div" | "/" | "\\" => Some(api.div()),
        "drop" => Some(api.drop()),
        "dup" => Some(api.dup()),
        "swap" => Some(api.swap()),
        "rot" | "rotate" => Some(api.rot()),
        "undo" => Some(api.undo()),
        "enter" => Some(api.enter()),
        "clear_entry" => Some(api.clear_entry()),
        "clear_all" => Some(api.clear_all()),
        "pow" => Some(api.pow()),
        "percent" => Some(api.percent()),
        "inv" => Some(api.inv()),
        "square" => Some(api.square()),
        "root" => Some(api.root()),
        "sqrt" => Some(api.sqrt()),
        "ln" => Some(api.ln()),
        "sin" => Some(api.sin()),
        "cos" => Some(api.cos()),
        "tan" => Some(api.tan()),
        "sec" => Some(api.sec()),
        "csc" => Some(api.csc()),
        "cot" => Some(api.cot()),
        "asin" => Some(api.asin()),
        "acos" => Some(api.acos()),
        "asec" => Some(api.asec()),
        "acsc" => Some(api.acsc()),
        "atan" => Some(api.atan()),
        "acot" => Some(api.acot()),
        "sinh" => Some(api.sinh()),
        "cosh" => Some(api.cosh()),
        "tanh" => Some(api.tanh()),
        "sech" => Some(api.sech()),
        "csch" => Some(api.csch()),
        "coth" => Some(api.coth()),
        "asinh" => Some(api.asinh()),
        "acosh" => Some(api.acosh()),
        "asech" => Some(api.asech()),
        "acsch" => Some(api.acsch()),
        "atanh" => Some(api.atanh()),
        "acoth" => Some(api.acoth()),
        "exp" => Some(api.exp()),
        "exp10" => Some(api.exp10()),
        "exp2" => Some(api.exp2()),
        "log10" => Some(api.log10()),
        "log2" => Some(api.log2()),
        "log_y_x" => Some(api.log_y_x()),
        "gamma" => Some(api.gamma()),
        "erf" => Some(api.erf()),
        "erfc" => Some(api.erfc()),
        "bessel" => Some(api.bessel()),
        "mbessel" => Some(api.mbessel()),
        "sinc" => Some(api.sinc()),
        "neg" => Some(api.neg()),
        "signum" => Some(api.signum()),
        "abs" => Some(api.abs()),
        "abs_sq" => Some(api.abs_sq()),
        "arg" => Some(api.arg()),
        "conjugate" => Some(api.conjugate()),
        "real_part" => Some(api.real_part()),
        "imag_part" => Some(api.imag_part()),
        "cart" => Some(api.cart()),
        "pol" => Some(api.pol()),
        "npol" => Some(api.npol()),
        "atan2" => Some(api.atan2()),
        "to_rad" => Some(api.to_rad()),
        "to_deg" => Some(api.to_deg()),
        "factorial" => Some(api.factorial()),
        "ncr" => Some(api.ncr()),
        "npr" => Some(api.npr()),
        "modulo" => Some(api.modulo()),
        "rand_num" | "rand" => Some(api.rand_num()),
        "gcd" => Some(api.gcd()),
        "lcm" => Some(api.lcm()),
        "round" | "round_value" => Some(api.round_value()),
        "floor" | "floor_value" => Some(api.floor_value()),
        "ceil" | "ceil_value" => Some(api.ceil_value()),
        "dec_part" => Some(api.dec_part()),
        "pi" | "push_pi" => Some(api.push_pi()),
        "e" | "push_e" => Some(api.push_e()),
        "determinant" | "det" => Some(api.determinant()),
        "inverse" => Some(api.inverse()),
        "transpose" => Some(api.transpose()),
        "solve_ax_b" => Some(api.solve_ax_b()),
        "solve_lstsq" => Some(api.solve_lstsq()),
        "stack_vec" => Some(api.stack_vec()),
        "dot" => Some(api.dot()),
        "cross" => Some(api.cross()),
        "trace" => Some(api.trace()),
        "norm_p" => Some(api.norm_p()),
        "diag" => Some(api.diag()),
        "toep" | "tpltz" => Some(api.toep()),
        "mat_exp" => Some(api.mat_exp()),
        "hermitian" => Some(api.hermitian()),
        "mat_pow" => Some(api.mat_pow()),
        "qr" => Some(api.qr()),
        "lu" => Some(api.lu()),
        "svd" => Some(api.svd()),
        "evd" => Some(api.evd()),
        "mean" => Some(api.mean()),
        "mode" => Some(api.mode()),
        "variance" => Some(api.variance()),
        "std_dev_p" => Some(api.std_dev_p()),
        "std_dev_s" => Some(api.std_dev_s()),
        "median" => Some(api.median()),
        "quart" => Some(api.quart()),
        "max" | "max_value" => Some(api.max_value()),
        "min" | "min_value" => Some(api.min_value()),
        "hstack" => Some(api.hstack()),
        "vstack" => Some(api.vstack()),
        "ravel" => Some(api.ravel()),
        "hravel" => Some(api.hravel()),
        "vravel" => Some(api.vravel()),
        _ => None,
    }
}

fn expect_argument<'a>(
    tokens: &'a [ScriptToken],
    index: usize,
    line_number: usize,
    transcript: &[ScriptLogEntry],
) -> Result<&'a ScriptToken, ScriptRunFailure> {
    tokens.get(index + 1).ok_or_else(|| {
        parse_failure(
            "invalid_input",
            "script command is missing a required argument",
            line_number,
            tokens[index].column,
            Some(tokens[index].text.clone()),
            transcript,
        )
    })
}

fn apply_response(
    transcript: &mut Vec<ScriptLogEntry>,
    warnings: &mut Vec<String>,
    line_number: usize,
    column: usize,
    command_text: &str,
    response: ApiResponse,
) -> Result<(), ScriptRunFailure> {
    if response.ok {
        if let Some(warning) = response.warning.clone() {
            warnings.push(warning.clone());
        }
        transcript.push(ScriptLogEntry {
            line: line_number,
            command: command_text.to_string(),
            ok: true,
            message: response.warning,
        });
        Ok(())
    } else {
        Err(ScriptRunFailure {
            error: ScriptError {
                code: response
                    .error
                    .as_ref()
                    .map(|error| error.code.clone())
                    .unwrap_or_else(|| "script_error".to_string()),
                message: response
                    .error
                    .as_ref()
                    .map(|error| error.message.clone())
                    .unwrap_or_else(|| "script operation failed".to_string()),
                line: line_number,
                column,
                token: Some(command_text.to_string()),
            },
            transcript: transcript.clone(),
        })
    }
}

fn parse_failure(
    code: &str,
    message: &str,
    line: usize,
    column: usize,
    token: Option<String>,
    transcript: &[ScriptLogEntry],
) -> ScriptRunFailure {
    ScriptRunFailure {
        error: ScriptError {
            code: code.to_string(),
            message: message.to_string(),
            line,
            column,
            token,
        },
        transcript: transcript.to_vec(),
    }
}

fn parse_real_literal(token: &str) -> Option<f64> {
    token.parse::<f64>().ok()
}

fn parse_complex_literal(token: &str) -> Option<ComplexInput> {
    if !token.starts_with('(') || !token.ends_with(')') {
        return None;
    }
    let inner = &token[1..token.len() - 1];
    let parts = inner.split(',').map(|part| part.trim()).collect::<Vec<_>>();
    if parts.len() != 2 {
        return None;
    }
    let re = parts[0].parse::<f64>().ok()?;
    let im = parts[1].parse::<f64>().ok()?;
    Some(ComplexInput { re, im })
}

fn parse_matrix_literal(
    token: &str,
    line_number: usize,
    column: usize,
    transcript: &[ScriptLogEntry],
) -> Result<MatrixInput, ScriptRunFailure> {
    if !token.starts_with('[') || !token.ends_with(']') {
        return Err(parse_failure(
            "invalid_input",
            "matrix literal must be wrapped in [ ... ]",
            line_number,
            column,
            Some(token.to_string()),
            transcript,
        ));
    }

    let inner = token[1..token.len() - 1].trim();
    if inner.is_empty() {
        return Err(parse_failure(
            "invalid_input",
            "matrix literal cannot be empty",
            line_number,
            column,
            Some(token.to_string()),
            transcript,
        ));
    }

    let row_tokens = split_top_level(inner, ';');
    let mut rows = Vec::new();
    for row in row_tokens {
      let entries = split_row_entries(&row);
      if entries.is_empty() {
          return Err(parse_failure(
              "invalid_input",
              "matrix row cannot be empty",
              line_number,
              column,
              Some(token.to_string()),
              transcript,
          ));
      }
      rows.push(entries);
    }

    let col_count = rows[0].len();
    if rows.iter().any(|row| row.len() != col_count) {
        return Err(parse_failure(
            "invalid_input",
            "all matrix rows must have the same number of columns",
            line_number,
            column,
            Some(token.to_string()),
            transcript,
        ));
    }

    let mut data = Vec::new();
    for row in rows {
        for entry in row {
            if let Some(complex) = parse_complex_literal(&entry) {
                data.push(complex);
            } else if let Ok(value) = entry.parse::<f64>() {
                data.push(ComplexInput { re: value, im: 0.0 });
            } else {
                return Err(parse_failure(
                    "invalid_input",
                    &format!("invalid matrix entry: {entry}"),
                    line_number,
                    column,
                    Some(token.to_string()),
                    transcript,
                ));
            }
        }
    }

    Ok(MatrixInput {
        rows: data.len() / col_count,
        cols: col_count,
        data,
    })
}

fn split_top_level(input: &str, delimiter: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0i32;

    for ch in input.chars() {
        match ch {
            '(' => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' => {
                paren_depth -= 1;
                current.push(ch);
            }
            _ if ch == delimiter && paren_depth == 0 => {
                if !current.trim().is_empty() {
                    parts.push(current.trim().to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }

    parts
}

fn split_row_entries(row: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0i32;

    for ch in row.chars() {
        match ch {
            '(' => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' => {
                paren_depth -= 1;
                current.push(ch);
            }
            _ if (ch == ',' || ch.is_whitespace()) && paren_depth == 0 => {
                if !current.trim().is_empty() {
                    entries.push(current.trim().to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if !current.trim().is_empty() {
        entries.push(current.trim().to_string());
    }

    entries
}

fn parse_positive_usize(
    token: &str,
    label: &str,
    line: usize,
    column: usize,
    transcript: &[ScriptLogEntry],
) -> Result<usize, ScriptRunFailure> {
    let value = token.parse::<usize>().map_err(|_| {
        parse_failure(
            "invalid_input",
            &format!("{label} must be a positive integer"),
            line,
            column,
            Some(token.to_string()),
            transcript,
        )
    })?;
    if value == 0 {
        return Err(parse_failure(
            "invalid_input",
            &format!("{label} must be greater than zero"),
            line,
            column,
            Some(token.to_string()),
            transcript,
        ));
    }
    Ok(value)
}

fn parse_precision(
    token: &str,
    line: usize,
    column: usize,
    transcript: &[ScriptLogEntry],
) -> Result<u8, ScriptRunFailure> {
    let value = token.parse::<u8>().map_err(|_| {
        parse_failure(
            "invalid_input",
            "precision must be an integer from 0 to 12",
            line,
            column,
            Some(token.to_string()),
            transcript,
        )
    })?;
    if value > 12 {
        return Err(parse_failure(
            "invalid_input",
            "precision must be an integer from 0 to 12",
            line,
            column,
            Some(token.to_string()),
            transcript,
        ));
    }
    Ok(value)
}

fn parse_register(
    token: &str,
    line: usize,
    column: usize,
    transcript: &[ScriptLogEntry],
) -> Result<usize, ScriptRunFailure> {
    let upper = token.trim().to_ascii_uppercase();
    if upper.len() != 1 {
        return Err(parse_failure(
            "invalid_input",
            "register must be a single letter A-Z",
            line,
            column,
            Some(token.to_string()),
            transcript,
        ));
    }
    let byte = upper.as_bytes()[0];
    if !byte.is_ascii_uppercase() {
        return Err(parse_failure(
            "invalid_input",
            "register must be a single letter A-Z",
            line,
            column,
            Some(token.to_string()),
            transcript,
        ));
    }
    Ok((byte - b'A') as usize)
}

fn parse_display_mode(
    token: &str,
    line: usize,
    column: usize,
    transcript: &[ScriptLogEntry],
) -> Result<ApiDisplayMode, ScriptRunFailure> {
    match token.to_ascii_lowercase().as_str() {
        "fix" => Ok(ApiDisplayMode::Fix),
        "sci" => Ok(ApiDisplayMode::Sci),
        "eng" => Ok(ApiDisplayMode::Eng),
        _ => Err(parse_failure(
            "invalid_input",
            "display mode must be one of: fix, sci, eng",
            line,
            column,
            Some(token.to_string()),
            transcript,
        )),
    }
}

fn parse_angle_mode(
    token: &str,
    line: usize,
    column: usize,
    transcript: &[ScriptLogEntry],
) -> Result<ApiAngleMode, ScriptRunFailure> {
    match token.to_ascii_lowercase().as_str() {
        "deg" => Ok(ApiAngleMode::Deg),
        "rad" => Ok(ApiAngleMode::Rad),
        _ => Err(parse_failure(
            "invalid_input",
            "angle mode must be one of: deg, rad",
            line,
            column,
            Some(token.to_string()),
            transcript,
        )),
    }
}
