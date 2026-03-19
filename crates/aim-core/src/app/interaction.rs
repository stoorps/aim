#[derive(Debug, Eq, PartialEq)]
pub enum InteractionRequest {
    SelectRegisteredApp { query: String, matches: Vec<String> },
}
