use crate::domain::source::SourceKind;
use crate::domain::source::SourceRef;

pub fn resolve_query(query: &str) -> Result<SourceRef, ResolveQueryError> {
    if query.starts_with("file://") {
        return Ok(SourceRef {
            kind: SourceKind::File,
            locator: query.to_owned(),
        });
    }

    if query.starts_with("https://gitlab.com/") || query.starts_with("http://gitlab.com/") {
        return Ok(SourceRef {
            kind: SourceKind::GitLab,
            locator: query.to_owned(),
        });
    }

    if query.starts_with("https://") || query.starts_with("http://") {
        return Ok(SourceRef {
            kind: SourceKind::DirectUrl,
            locator: query.to_owned(),
        });
    }

    if is_github_shorthand(query) {
        return Ok(SourceRef {
            kind: SourceKind::GitHub,
            locator: query.to_owned(),
        });
    }

    Err(ResolveQueryError::Unsupported)
}

#[derive(Debug, Eq, PartialEq)]
pub enum ResolveQueryError {
    Unsupported,
}

fn is_github_shorthand(query: &str) -> bool {
    let mut parts = query.split('/');
    let Some(owner) = parts.next() else {
        return false;
    };
    let Some(repo) = parts.next() else {
        return false;
    };

    if parts.next().is_some() {
        return false;
    }

    !owner.is_empty() && !repo.is_empty() && !owner.contains(':') && !repo.contains(':')
}
