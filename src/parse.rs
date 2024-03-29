use chumsky::prelude::*;

use crate::Error;

#[derive(Clone, Debug)]
pub enum Ast {
    _Invalid, // その他文字
    PtrInc,   // >
    PtrDec,   // <
    Inc,      // +
    Dec,      // -
    Read,     // ,
    Write,    // .
    Loop(Vec<Self>),
}

pub fn parse(code: &str) -> Result<Vec<Ast>, Error> {
    let ast = bf_parser()
        .parse(code)
        .into_result()
        .map_err(|_| Error::InvalidSyntax {
            msg: "The brackets are not corresponding.",
        })?;

    Ok(ast)
}

fn bf_parser<'a>() -> impl Parser<'a, &'a str, Vec<Ast>, extra::Err<EmptyErr>> {
    use Ast::*;

    let bf_chars = "+-><.,[]";
    let is_other_char = |c: &char| !bf_chars.contains(*c);

    recursive(|bf| {
        choice((
            just('<').to(PtrDec),
            just('>').to(PtrInc),
            just('+').to(Inc),
            just('-').to(Dec),
            just(',').to(Read),
            just('.').to(Write),
            bf.delimited_by(just('['), just(']')).map(Loop),
        ))
        .padded_by(any().filter(is_other_char).repeated())
        .recover_with(via_parser(nested_delimiters('[', ']', [], |_| _Invalid)))
        .repeated()
        .collect()
    })
    .then_ignore(end())
}
