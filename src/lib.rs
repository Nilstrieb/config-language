use codespan::Span;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
};

pub mod parser;

#[derive(Debug)]
pub enum Error {
    Parse(parser::Error),
}

impl Error {
    fn into_msg_and_span(self) -> (String, Span) {
        match self {
            Error::Parse(e) => e.into_msg_and_span(),
        }
    }
}

pub fn evaluate(input: &str) -> Result<(), Error> {
    let expr = parser::parse(input).map_err(Error::Parse)?;
    dbg!(expr);
    todo!()
}

pub fn render_error(input: &str, error: Error) {
    let mut files = SimpleFiles::new();
    let file_id = files.add("example.nc", input);
    let (msg, span) = error.into_msg_and_span();
    let diagnostic = Diagnostic::error()
        .with_message(msg)
        .with_labels(vec![Label::primary(
            file_id,
            (span.start().to_usize())..span.end().to_usize(),
        )]);

    let writer = codespan_reporting::term::termcolor::StandardStream::stderr(
        codespan_reporting::term::termcolor::ColorChoice::Always,
    );
    let config = codespan_reporting::term::Config::default();

    codespan_reporting::term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
}
