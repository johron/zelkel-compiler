use std::collections::HashMap;
use crate::error;
use crate::lexer::{Token, TokenPos, TokenValue};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum StatementKind {
    VariableDeclaration(VariableDeclaration),
    FunctionDeclaration(FunctionDeclaration),
    ExpressionStatement(ExpressionStatement),
}

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    name: String,
    typ: ValueType,
    expr: Expression,
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    name: String,
    typ: ValueType,
    args: Vec<VariableDeclaration>,
    body: Vec<Statement>,
}

#[derive(Debug, Clone)]
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
    nested: Option<Box<Expression>>,
}

#[derive(Clone, Debug)]
pub struct UnaryExpression {
    left: PrimaryExpression,
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

#[derive(Debug, Clone)]
pub struct VariableOptions {
    pub mutable: bool,
    pub typ: ValueType,
}

#[derive(Clone, Debug)]
pub struct Scope {
    variables: HashMap<String, VariableOptions>,
    functions: Vec<String>,
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

fn enter_scope(scope: &mut Vec<Scope>) -> Vec<Scope> {
    let parent_scope = scope.last().cloned().unwrap_or(Scope {
        variables: HashMap::new(),
        functions: Vec::new(),
    });
    scope.push(parent_scope);
    scope.clone()
}

fn exit_scope(scope: &mut Vec<Scope>) -> Vec<Scope> {
    scope.pop();
    scope.clone()
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

fn parse_primary_expression(i: &usize, toks: &Vec<Token>) -> Result<(PrimaryExpression, usize), String> {
    let mut i = *i;
    let t = toks[i].clone();
    i += 1;
    match t.value {
        TokenValue::String(_) => {
            Ok((PrimaryExpression {
                value: t.value.clone(),
                typ: ValueType::String,
                nested: None,
            }, i))
        },
        TokenValue::Integer(_) => {
            Ok((PrimaryExpression {
                value: t.value.clone(),
                typ: ValueType::Integer,
                nested: None,
            }, i))
        },
        TokenValue::Float(_) => {
            Ok((PrimaryExpression {
                value: t.value.clone(),
                typ: ValueType::Float,
                nested: None,
            }, i))
        },
        TokenValue::Bool(_) => {
            Ok((PrimaryExpression {
                value: t.value.clone(),
                typ: ValueType::Bool,
                nested: None,
            }, i))
        },
        TokenValue::Punctuation(ref p) if p == "(" => {
            let (expr, j) = parse_expression(&i, &toks)?;
            i = j;
            expect(&i, &toks, TokenValue::Punctuation(")".to_string()))?;
            i += 1;
            Ok((PrimaryExpression {
                value: TokenValue::Nested,
                typ: expr.clone().typ,
                nested: Some(Box::new(expr)),
            }, i))
        },
        TokenValue::Identifier(_) => {
            todo!()
        }
        _ => Err(error("Unexpected token found while parsing primary expression".to_string(), t.pos)),
    }
}

fn parse_unary_expression(i: &usize, toks: &Vec<Token>) -> Result<(UnaryExpression, usize), String> {
    let mut i = *i;
    let t = toks[i].clone();
    if t.value == TokenValue::Arithmetic("-".to_string()) || t.value == TokenValue::Arithmetic("+".to_string()) {
        i += 1;
        let (right, j) = parse_unary_expression(&i, &toks)?;

        Ok((UnaryExpression {
            left: PrimaryExpression {
                value: right.clone().left.value,
                typ: right.clone().left.typ,
                nested: right.clone().left.nested,
            },
            typ: right.typ.clone(),
            op: Some(t.clone()),
        }, j))
    } else {
        let (left, j) = parse_primary_expression(&i, &toks)?;
        Ok((UnaryExpression {
            left: left.clone(),
            typ: left.typ,
            op: None,
        }, j))
    }
}

fn parse_term_expression(i: &usize, toks: &Vec<Token>) -> Result<(Option<TermExpression>, usize), String> {
    let mut i = *i;
    let mut expr: Option<TermExpression> = None;
    let (left, j) = parse_unary_expression(&i, &toks)?;
    i = j;
    while
        i < toks.len() &&
            (toks[i].value == TokenValue::Arithmetic("*".to_string()) ||
                toks[i].value == TokenValue::Arithmetic("/".to_string()) ||
                toks[i].value == TokenValue::Arithmetic("%".to_string())) &&
            toks[i + 1].value != TokenValue::Punctuation(";".to_string()) {
        let op = expect(&i, &toks, TokenValue::Arithmetic("".to_string()))?;
        i += 1;
        let (right, k) = parse_unary_expression(&i, &toks)?;
        i = k;
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
        return Ok((Some(TermExpression {
            left: Some(left.clone()),
            right: None,
            typ: left.typ.clone(),
            op: None,
        }), i));
    }

    Ok((expr, i))
}

fn parse_comparison_expression(i: &usize, toks: &Vec<Token>) -> Result<(Option<ComparisonExpression>, usize), String> {
    let mut i = *i;
    let mut expr: Option<ComparisonExpression> = None;
    let (left, j) = parse_term_expression(&i, &toks)?;
    i = j;
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
        let (right, k) = parse_term_expression(&i, &toks)?;
        i = k;
        if left.clone().unwrap().typ != right.clone().unwrap().typ {
            return Err(error("Type mismatch".to_string(), toks[i].pos.clone()));
        }
        expr = Some(ComparisonExpression {
            left: left.clone(),
            right: right.clone(),
            typ: expr.clone().unwrap().typ.clone(),
            op: Some(op),
        });
    }

    if expr.is_none() {
        return Ok((Some(ComparisonExpression {
            left: left.clone(),
            right: None,
            typ: left.clone().unwrap().typ,
            op: None,
        }), i));
    }

    Ok((expr, i))
}

fn parse_expression(i: &usize, toks: &Vec<Token>) -> Result<(Expression, usize), String> {
    let mut i = *i;
    let mut expr: Option<Expression> = None;
    let (left, j) = parse_comparison_expression(&i, &toks)?;
    i = j;
    while i < toks.len() {
        if toks[i].value == TokenValue::Punctuation("(".to_string()) {
            i += 1;
            let (nested_expr, k) = parse_expression(&i, &toks)?;
            i = k;
            expect(&i, &toks, TokenValue::Punctuation(")".to_string()))?;
            i += 1;
            expr = Some(nested_expr);
        } else if toks[i].value == TokenValue::Arithmetic("+".to_string()) || toks[i].value == TokenValue::Arithmetic("-".to_string()) {
            let op = expect(&i, &toks, TokenValue::Arithmetic("".to_string()))?;
            i += 1;
            let (right, k) = parse_comparison_expression(&i, &toks)?;
            i = k;
            if left.clone().unwrap().typ != right.clone().unwrap().typ {
                return Err(error("Type mismatch".to_string(), toks[i].pos.clone()));
            }
            expr = Some(Expression {
                kind: ExpressionKind::Binary(BinaryExpression {
                    left: Some(left.clone().unwrap()),
                    right: Some(right.clone().unwrap()),
                    typ: left.clone().unwrap().typ.clone(),
                    op: Some(op),
                }),
                typ: left.clone().unwrap().typ.clone(),
            });
        } else {
            break;
        }
    }

    if expr.is_none() {
        return Ok((Expression {
            kind: ExpressionKind::Comparison(left.clone().unwrap()),
            typ: left.clone().unwrap().typ,
        }, i));
    }

    Ok((expr.unwrap(), i))
}

fn parse_class_declaration(i: &usize, toks: &Vec<Token>) -> Result<(Statement, usize), String> {
    let mut i = *i;
    i += 1;
    let name = expect(&i, &toks, TokenValue::empty("identifier")?)?.value.as_string();
    i += 1;
    expect(&i, &toks, TokenValue::Punctuation("{".to_string()))?;
    i += 1;
    todo!("parse class body, should only be function declarations");
    expect(&i, &toks, TokenValue::Punctuation("}".to_string()))?;
    i += 1;
    todo!("do rest");
}

fn parse_function_declaration(i: &usize, toks: &Vec<Token>, global_scope: &mut Vec<Scope>) -> Result<(Statement, usize, Vec<Scope>), String> {
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
            args: todo!("function arguments"),
            typ: return_type,
            body: todo!("return function body"),
        }),
        pos: toks[i].pos.clone(),
    }, i, todo!("return scope")))
}

fn parse_variable_declaration(i: &usize, toks: &Vec<Token>, global_scope: &mut Vec<Scope>) -> Result<(Statement, usize, Vec<Scope>), String> {
    let mut i = *i;
    let mut global_scope = global_scope.clone();
    i += 1;
    let name = expect(&i, &toks, TokenValue::empty("identifier")?)?.value.as_string();

    if global_scope.last().unwrap().variables.iter().any(|v| v.0 == &name) {
        return Err(error(format!("Variable '{}' already declared", name), toks[i].pos.clone()));
    }

    i += 1;
    expect(&i, &toks, TokenValue::Punctuation(":".to_string()))?;
    i += 1;
    let type_ident = expect(&i, &toks, TokenValue::empty("identifier")?)?;
    let typ = parse_type(&type_ident)?;
    i += 1;
    expect(&i, &toks, TokenValue::Punctuation("=".to_string()))?;
    i += 1;
    let (expr, j) = parse_expression(&i, toks)?;
    if typ != expr.typ {
        return Err(error(format!("Type mismatch: expected {:?}, but found {:?}", typ, expr.typ), toks[i].pos.clone()));
    }

    i = j;
    expect(&i, &toks, TokenValue::Punctuation(";".to_string()))?;

    global_scope.last_mut().unwrap().variables.insert(name.clone(), VariableOptions {
        mutable: false,
        typ: typ.clone(),
    });

    Ok((Statement {
        kind: StatementKind::VariableDeclaration(VariableDeclaration {
            name,
            typ,
            expr,
        }),
        pos: toks[i].pos.clone(),
    }, i + 1, global_scope))
}

fn parse_expression_statement(i: &usize, toks: &Vec<Token>, global_scope: &mut Vec<Scope>) -> Result<(Statement, usize, Vec<Scope>), String> {
    let mut i = *i;
    let mut global_scope = global_scope.clone();
    let (expr, j) = parse_expression(&i, toks)?;
    i = j;
    expect(&i, &toks, TokenValue::Punctuation(";".to_string()))?;

    Ok((Statement {
        kind: StatementKind::ExpressionStatement(ExpressionStatement {
            typ: expr.clone().typ,
            expr,
        }),
        pos: toks[i].pos.clone(),
    }, j, global_scope))
}

fn parse_identifier(i: &usize, toks: &Vec<Token>, global_scope: &mut Vec<Scope>) -> Result<(Statement, usize, Vec<Scope>), String> {
    let mut i = *i;
    let t = toks[i].clone();
    let val = t.value;

    let stmt: Result<(Statement, usize, Vec<Scope>), String> = match val {
        TokenValue::Identifier(ref s) => match s.as_str() {
            "fn" => parse_function_declaration(&i, toks, global_scope),
            "let" => parse_variable_declaration(&i, toks, global_scope),
            _ => Err(error(format!("Unknown identifier: '{}'", s), t.pos)),
        },
        _ => Err(error("Expected an identifier while parsing identifier".to_string(), t.pos)),
    };

    stmt
}

fn parse_statement(i: &usize, toks: &Vec<Token>, global_scope: &mut Vec<Scope>) -> Result<(Statement, usize, Vec<Scope>), String> {
    let mut i = *i;
    let pos = toks[i].pos.clone();

    while i < toks.len() {
        return match toks[i].value {
            TokenValue::Identifier(_) => Ok(parse_identifier(&i, &toks, global_scope)?),
            _ => Ok(parse_expression_statement(&i, &toks, global_scope)?),
        }
    }

    Err(error("Unexpected end of file".to_string(), pos))
}

pub fn parse(toks: Vec<Token>) -> Result<Vec<Statement>, String> {
    let mut ast: Vec<Statement> = Vec::new();
    let mut i = 0;

    let mut global_scope: Vec<Scope> = Vec::new();
    global_scope.push(Scope { variables: HashMap::new(), functions: Vec::new() });

    while i < toks.len() {
        let (stmt, j, scope) = parse_statement(&i, &toks, &mut global_scope)?;
        global_scope = scope;
        ast.push(stmt);
        i = j;
    }

    Ok(ast)
}