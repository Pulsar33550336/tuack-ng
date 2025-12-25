use nom::{
    IResult,
    Parser, // 需要导入这个
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alpha1, alphanumeric1, char},
    combinator::{opt, recognize},
    multi::{many0, separated_list1},
    sequence::{delimited, pair},
};

// 单个标识符
fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(alpha1, many0(alt((alphanumeric1, tag("_")))))).parse(input)
}

// 解析 a.b.c 部分
fn dot_path(input: &str) -> IResult<&str, Vec<&str>> {
    separated_list1(tag("."), identifier).parse(input)
}

// 解析括号内的内容（不含括号）
fn parentheses_content(input: &str) -> IResult<&str, &str> {
    delimited(char('('), take_until(")"), char(')')).parse(input)
}

// 返回结构体的版本
#[derive(Debug)]
pub struct DotCall {
    pub path: Vec<String>,
    pub arguments: Option<String>,
}

pub fn parse_dot_call(input: &str) -> IResult<&str, DotCall> {
    let (input, (path_parts, args)) = (dot_path, opt(parentheses_content)).parse(input)?;

    let dot_call = DotCall {
        path: path_parts.into_iter().map(String::from).collect(),
        arguments: args.map(String::from),
    };

    Ok((input, dot_call))
}
