use crate::error;
use crate::lexer::{Token, TokenPos, TokenValue};

#[derive(Debug)]
pub struct Statement {
    kind: StatementKind,
    pos: TokenPos,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Integer,
    Float,
    String,
    Bool,
}

#[derive(Debug)]
pub enum StatementKind {
    VariableDeclaration(VariableDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    ExpressionStatement(ExpressionStatement),
}

#[derive(Debug)]
pub struct VariableDeclaration {
    name: String,
    typ: ValueType,
    expr: Expression,
}

#[derive(Debug)]
pub struct FunctionDeclaration {
    name: String,
    typ: ValueType,
    body: Vec<Statement>,
}

#[derive(Debug)]
pub struct ExpressionStatement {
    typ: ValueType,
    expr: Expression,
}

#[derive(Debug, Clone)]
pub struct Expression {
    kind: ExpressionKind,
    typ: ValueType
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    Primary(PrimaryExpression),
    Unary(UnaryExpression),
    Term(TermExpression),
    Comparison(ComparisonExpression),
    Binary(BinaryExpression),
}

#[derive(Debug, Clone)]
pub struct PrimaryExpression {
    value: TokenValue,
    typ: ValueType,
}

#[derive(Clone, Debug)]
pub struct UnaryExpression {
    left: Option<PrimaryExpression>,
    typ: ValueType,
    op: Option<Token>,
}

#[derive(Clone, Debug)]
pub struct TermExpression {
    right: Option<UnaryExpression>,
    left: Option<UnaryExpression>,
    typ: ValueType,
    op: Option<Token>,
}

#[derive(Clone, Debug)]
pub struct ComparisonExpression {
    right: Option<TermExpression>,
    left: Option<TermExpression>,
    typ: ValueType,
    op: Option<Token>,
}

#[derive(Clone, Debug)]
pub struct BinaryExpression {
    right: Option<ComparisonExpression>,
    left: Option<ComparisonExpression>,
    typ: ValueType,
    op: Option<Token>,
}

fn expect(i: &usize, toks: &Vec<Token>, value: TokenValue) -> Result<Token, String> {
    if i >= &toks.len() {
        return Err(error("Unexpected end of file".to_string(), toks[*i].pos.clone()));
    }

    if toks[*i].value == value {
        return Ok(toks[*i].clone());
    } else if let TokenValue::Identifier(_) | TokenValue::String(_) | TokenValue::Arithmetic(_) | TokenValue::Punctuation(_) = value {
        return Ok(toks[*i].clone());
    }

    Err(error(format!("Expected {:?} but got {:?}", value, toks[*i].value), toks[*i].pos.clone()))
}

fn parse_type(tok: &Token) -> Result<ValueType, String> {
    match tok.value {
        TokenValue::Identifier(ref s) => match s.as_str() {
            "int" => Ok(ValueType::Integer),
            "str" => Ok(ValueType::String),
            "float" => Ok(ValueType::Float),
            "bool" => Ok(ValueType::Bool),
            _ => Err(error(format!("Unknown type: '{}'", s), tok.pos.clone())),
        },
        _ => Err(error("Expected an identifier while parsing type".to_string(), tok.pos.clone())),
    }
}

fn parse_primary_expression(i: &usize, toks: &Vec<Token>) -> Result<Option<PrimaryExpression>, String> {
    let mut i = *i;
    let t = toks[i].clone();
    match t.value {
        TokenValue::String(_) | TokenValue::Integer(_) | TokenValue::Float(_) | TokenValue::Bool(_) => {
            Ok(Some(PrimaryExpression {
                value: t.value.clone(),
                typ: ValueType::Integer,
            }))
        },
        TokenValue::Identifier(_) => {
            todo!()
        }
        _ => Err(error("Unexpected token found while parsing primary expression".to_string(), t.pos)),
    }
}

fn parse_unary_expression(i: &usize, toks: &Vec<Token>) -> Result<Option<UnaryExpression>, String> {
    let mut i = *i;
    let t = toks[i].clone();
    if t.value == TokenValue::Arithmetic("-".to_string()) || t.value == TokenValue::Arithmetic("+".to_string()) {
        i += 1;
        let right = parse_unary_expression(&i, &toks)?.unwrap();

        Ok(Some(UnaryExpression {
            left: Some(PrimaryExpression {
                value: right.clone().left.unwrap().value,
                typ: right.clone().left.unwrap().typ,
            }),
            typ: right.typ.clone(),
            op: Some(t.clone()),
        }))
    } else {
        let left = parse_primary_expression(&i, &toks)?;
        Ok(Some(UnaryExpression {
            left: left.clone(),
            typ: left.unwrap().typ,
            op: None,
        }))
    }
}

fn parse_term_expression(i: &usize, toks: &Vec<Token>) -> Result<Option<TermExpression>, String> {
    let mut i = *i;
    let mut expr: Option<TermExpression> = None;
    let left = parse_unary_expression(&i, &toks)?.unwrap();
    while
        i < toks.len() &&
            (toks[i].value == TokenValue::Arithmetic("*".to_string()) ||
                toks[i].value == TokenValue::Arithmetic("/".to_string()) ||
                toks[i].value == TokenValue::Arithmetic("%".to_string())) &&
            toks[i + 1].value != TokenValue::Punctuation(";".to_string()) {
        let op = expect(&i, &toks, TokenValue::Arithmetic("".to_string()))?;
        i += 1;
        let right = parse_unary_expression(&i, &toks)?.unwrap();
        if left.typ != right.typ {
            return Err(error("Type mismatch".to_string(), toks[i].pos.clone()));
        }
        expr = Some(TermExpression {
            left: Some(left.clone()),
            right: Some(right.clone()),
            typ: expr.clone().unwrap().typ.clone(),
            op: Some(op),
        });
    }

    if expr.is_none() {
        return Ok(Some(TermExpression {
            left: Some(left.clone()),
            right: None,
            typ: left.typ.clone(),
            op: None,
        }));
    }

    Ok(expr)
}

fn parse_comparison_expression(i: &usize, toks: &Vec<Token>) -> Result<Option<ComparisonExpression>, String> {
    let mut i = *i;
    let mut expr: Option<ComparisonExpression> = None;
    let left = parse_term_expression(&i, &toks)?.unwrap();
    while
        i < toks.len() &&
            (toks[i].value == TokenValue::Arithmetic("==".to_string()) ||
                toks[i].value == TokenValue::Arithmetic("!=".to_string()) ||
                toks[i].value == TokenValue::Arithmetic(">=".to_string()) ||
                toks[i].value == TokenValue::Arithmetic("<=".to_string()) ||
                toks[i].value == TokenValue::Arithmetic(">".to_string()) ||
                toks[i].value == TokenValue::Arithmetic("<".to_string())) &&
            toks[i + 1].value != TokenValue::Punctuation(";".to_string()) {
        let op = expect(&i, &toks, TokenValue::Arithmetic("".to_string()))?;
        i += 1;
        let right = parse_term_expression(&i, &toks)?.unwrap();
        if left.typ != right.typ {
            return Err(error("Type mismatch".to_string(), toks[i].pos.clone()));
        }
        expr = Some(ComparisonExpression {
            left: Some(left.clone()),
            right: Some(right.clone()),
            typ: expr.clone().unwrap().typ.clone(),
            op: Some(op),
        });
    }

    if expr.is_none() {
        return Ok(Some(ComparisonExpression {
            left: Some(left.clone()),
            right: None,
            typ: left.typ.clone(),
            op: None,
        }));
    }

    Ok(expr)
}

fn parse_expression(i: &usize, toks: &Vec<Token>) -> Result<(Expression, usize), String> {
    let mut i = *i;
    let mut expr: Option<Expression> = None;
    let left = parse_comparison_expression(&i, &toks)?.unwrap();
    while
        i < toks.len() &&
            (toks[i].value == TokenValue::Arithmetic("+".to_string()) ||
                toks[i].value == TokenValue::Arithmetic("-".to_string())) &&
            toks[i + 1].value != TokenValue::Punctuation(";".to_string()) {
        let op = expect(&i, &toks, TokenValue::Arithmetic("".to_string()))?;
        i += 1;
        let right = parse_comparison_expression(&i, &toks)?.unwrap();
        if left.typ != right.typ {
            return Err(error("Type mismatch".to_string(), toks[i].pos.clone()));
        }
        expr = Some(Expression {
            kind: ExpressionKind::Binary(BinaryExpression {
                left: Some(left.clone()),
                right: Some(right.clone()),
                typ: expr.clone().unwrap().typ.clone(),
                op: Some(op),
            }),
            typ: expr.unwrap().typ.clone(),
        });
    }

    if expr.is_none() {
        return Ok((Expression {
            kind: ExpressionKind::Comparison(left.clone()),
            typ: left.clone().typ.clone(),
        }, i));
    }

    Ok((expr.unwrap(), i))
}

fn parse_function_declaration(i: &usize, toks: &Vec<Token>) -> Result<(Statement, usize), String> {
    let mut i = *i;
    i += 1;
    let name = expect(&i, &toks, TokenValue::empty("identifier")?)?.value.as_string();
    i += 1;
    expect(&i, &toks, TokenValue::Punctuation("(".to_string()))?;
    todo!("parse function arguments");
    i += 1;
    expect(&i, &toks, TokenValue::Punctuation(")".to_string()))?;
    i += 1;
    expect(&i, &toks, TokenValue::Punctuation("->".to_string()))?;
    i += 1;
    let return_type = parse_type(&expect(&i, &toks, TokenValue::empty("identifier")?)?)?;
    i += 1;
    expect(&i, &toks, TokenValue::Punctuation("{".to_string()))?;
    i += 1;
    todo!("parse function body");
    expect(&i, &toks, TokenValue::Punctuation("}".to_string()))?;

    Ok((Statement {
        kind: StatementKind::FunctionDeclaration(FunctionDeclaration {
            name,
            typ: return_type,
            body: todo!("return function body"),
        }),
        pos: toks[i].pos.clone(),
    }, i))
}

fn parse_variable_declaration(i: &usize, toks: &Vec<Token>) -> Result<(Statement, usize), String> {
    println!("parse variable declaration");
    let mut i = *i;
    i += 1;
    let name = expect(&i, &toks, TokenValue::empty("identifier")?)?.value.as_string();
    i += 1;
    expect(&i, &toks, TokenValue::Punctuation(":".to_string()))?;
    i += 1;
    let typ = parse_type(&expect(&i, &toks, TokenValue::empty("identifier")?)?)?;
    i += 1;
    expect(&i, &toks, TokenValue::Punctuation("=".to_string()))?;
    i += 1;
    let (expr, j) = parse_expression(&i, toks)?;
    println!("{:?}", expr);
    i = j;
    expect(&i, &toks, TokenValue::Punctuation(";".to_string()))?;

    Ok((Statement {
        kind: StatementKind::VariableDeclaration(VariableDeclaration {
            name,
            typ,
            expr,
        }),
        pos: toks[i].pos.clone(),
    }, j))
}

fn parse_expression_statement(i: &usize, toks: &Vec<Token>) -> Result<(Statement, usize), String> {
    let mut i = *i;
    println!("{:?}", toks[i]);
    let (expr, j) = parse_expression(&i, toks)?;
    i = j;
    expect(&i, &toks, TokenValue::Punctuation(";".to_string()))?;

    Ok((Statement {
        kind: StatementKind::ExpressionStatement(ExpressionStatement {
            typ: expr.clone().typ,
            expr,
        }),
        pos: toks[i].pos.clone(),
    }, j))
}

fn parse_identifier(i: &usize, toks: &Vec<Token>) -> Result<(Statement, usize), String> {
    let mut i = *i;
    let t = toks[i].clone();
    let val = t.value;

    let stmt: Result<(Statement, usize), String> = match val {
        TokenValue::Identifier(ref s) => match s.as_str() {
            "fn" => parse_function_declaration(&i, toks),
            "let" => parse_variable_declaration(&i, toks),
            _ => Err(error(format!("Unknown identifier: '{}'", s), t.pos)),
        },
        _ => Err(error("Expected an identifier while parsing identifier".to_string(), t.pos)),
    };

    stmt
}

fn parse_statement(i: &usize, toks: &Vec<Token>) -> Result<(Statement, usize), String> {
    let mut i = *i;
    let pos = toks[i].pos.clone();

    while i < toks.len() {
        return match toks[i].value {
            TokenValue::Identifier(_) => Ok(parse_identifier(&i, toks)?),
            _ => Ok(parse_expression_statement(&i, toks)?),
        }
    }

    Err(error("Unexpected end of file".to_string(), pos))
}

pub fn parse(toks: Vec<Token>) -> Result<Vec<Statement>, String> {
    let mut ast: Vec<Statement> = Vec::new();
    let mut i = 0;

    while i < toks.len() {
        let (stmt, j) = parse_statement(&i, &toks)?;
        ast.push(stmt);
        i = j;
    }

    Ok(ast)
}