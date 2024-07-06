fn main() {
    let input = std::fs::read_to_string("example.nc").unwrap();
    match config_language::evaluate(&input) {
        Ok(()) => {}
        Err(e) => config_language::render_error(&input, e),
    }
}
