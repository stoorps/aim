#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceKind {
    GitHub,
    GitLab,
    DirectUrl,
    File,
}

impl SourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::GitLab => "gitlab",
            Self::DirectUrl => "direct-url",
            Self::File => "file",
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SourceRef {
    pub kind: SourceKind,
    pub locator: String,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ResolvedRelease {
    pub version: String,
}
