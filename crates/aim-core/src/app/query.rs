use crate::domain::source::SourceRef;
use crate::source::input::classify_input;

pub fn resolve_query(query: &str) -> Result<SourceRef, ResolveQueryError> {
    classify_input(query)
        .map(|input| input.into_source_ref())
        .map_err(|_| ResolveQueryError::Unsupported)
}

#[derive(Debug, Eq, PartialEq)]
pub enum ResolveQueryError {
    Unsupported,
}
