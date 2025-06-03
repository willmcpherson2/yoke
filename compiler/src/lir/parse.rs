use crate::lir::{Block, Case, Global, Op, Program};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, digit1, multispace0, multispace1},
    combinator::{map, map_res},
    multi::{many0, separated_list0},
    sequence::{delimited, preceded},
    IResult, Parser,
};

pub fn parse(input: &str) -> Result<Program, String> {
    match program(input) {
        Ok((remaining, prog)) => {
            if remaining.trim().is_empty() {
                Ok(prog)
            } else {
                Err(format!("Unexpected input: {}", remaining))
            }
        }
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}

fn program(input: &str) -> IResult<&str, Program> {
    map_res(many0(ws(global)), |globals| {
        let (main, globals) = get_main(globals);
        if let Some(main) = main {
            Ok(Program { main, globals })
        } else {
            Err("No main function defined")
        }
    })
    .parse(input)
}

fn get_main(globals: Vec<Global>) -> (Option<Block>, Vec<Global>) {
    globals
        .into_iter()
        .fold((None, vec![]), |(main, mut globals), global| match global {
            Global::Fun { name, block, .. } if name == "main" => (Some(block), globals),
            global => {
                globals.push(global);
                (main, globals)
            }
        })
}

fn global(input: &str) -> IResult<&str, Global> {
    alt((
        map(
            (
                tag("const"),
                multispace1,
                var,
                multispace1,
                int16,
                multispace1,
                int32,
            ),
            |(_, _, name, _, arity, _, symbol)| Global::Const {
                name,
                arity,
                symbol,
            },
        ),
        map(
            (
                tag("fun"),
                multispace1,
                var,
                multispace1,
                int16,
                multispace1,
                delimited(ws(tag("{")), many0(ws(op)), ws(tag("}"))),
            ),
            |(_, _, name, _, arity, _, ops)| Global::Fun {
                name,
                symbol: 0,
                arity,
                block: Block(ops),
            },
        ),
    ))
    .parse(input)
}

fn op(input: &str) -> IResult<&str, Op> {
    alt((
        map(
            (tag("load_global"), multispace1, var, multispace1, var),
            |(_, _, name, _, global)| Op::LoadGlobal { name, global },
        ),
        map(
            (
                tag("load_arg"),
                multispace1,
                var,
                multispace1,
                var,
                multispace1,
                int64,
            ),
            |(_, _, name, _, var, _, index)| Op::LoadArg { name, var, index },
        ),
        map(
            (
                tag("new_app"),
                multispace1,
                var,
                multispace1,
                var,
                multispace1,
                vars,
            ),
            |(_, _, name, _, var, _, args)| Op::NewApp { name, var, args },
        ),
        map(
            (
                tag("new_partial"),
                multispace1,
                var,
                multispace1,
                var,
                multispace1,
                vars,
            ),
            |(_, _, name, _, var, _, args)| Op::NewPartial { name, var, args },
        ),
        map(
            (
                tag("apply_partial"),
                multispace1,
                var,
                multispace1,
                var,
                multispace1,
                vars,
            ),
            |(_, _, name, _, var, _, args)| Op::ApplyPartial { name, var, args },
        ),
        map(
            (tag("copy"), multispace1, var, multispace1, var),
            |(_, _, name, _, var)| Op::Copy { name, var },
        ),
        map(
            (tag("eval"), multispace1, var, multispace1, var),
            |(_, _, name, _, var)| Op::Eval { name, var },
        ),
        map(preceded((tag("free_args"), multispace1), var), |var| {
            Op::FreeArgs { var }
        }),
        map(preceded((tag("free_term"), multispace1), var), |var| {
            Op::FreeTerm { var }
        }),
        map(preceded((tag("return"), multispace1), var), |var| {
            Op::Return { var }
        }),
        map(preceded((tag("return_symbol"), multispace1), var), |var| {
            Op::ReturnSymbol { var }
        }),
        map(
            (
                tag("switch"),
                multispace1,
                var,
                multispace1,
                delimited(ws(tag("{")), many0(ws(case)), ws(tag("}"))),
            ),
            |(_, _, var, _, cases)| Op::Switch { var, cases },
        ),
        map(tag("abort"), |_| Op::Abort),
    ))
    .parse(input)
}

fn case(input: &str) -> IResult<&str, Case> {
    map(
        (
            int32,
            multispace1,
            delimited(ws(tag("{")), many0(ws(op)), ws(tag("}"))),
        ),
        |(symbol, _, ops)| Case {
            symbol,
            block: Block(ops),
        },
    )
    .parse(input)
}

fn vars(input: &str) -> IResult<&str, Vec<String>> {
    delimited(
        ws(tag("{")),
        separated_list0(multispace1, var),
        ws(tag("}")),
    )
    .parse(input)
}

fn var(input: &str) -> IResult<&str, String> {
    map(alphanumeric1, |s: &str| s.to_string()).parse(input)
}

fn int16(input: &str) -> IResult<&str, u16> {
    map_res(digit1, |s: &str| s.parse::<u16>()).parse(input)
}

fn int32(input: &str) -> IResult<&str, u32> {
    map_res(digit1, |s: &str| s.parse::<u32>()).parse(input)
}

fn int64(input: &str) -> IResult<&str, u64> {
    map_res(digit1, |s: &str| s.parse::<u64>()).parse(input)
}

fn ws<'a, F, O>(inner: F) -> impl Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    delimited(multispace0, inner, multispace0)
}
