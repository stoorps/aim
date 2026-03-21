#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DistroFamily {
    Debian,
    Fedora,
    Arch,
    OpenSuse,
    Alpine,
    Nix,
    Immutable,
    Generic,
}

pub fn detect_distro_family(os_release: &str) -> DistroFamily {
    let id = lookup_field(os_release, "ID");
    let id_like = lookup_field(os_release, "ID_LIKE");
    let variant_id = lookup_field(os_release, "VARIANT_ID");

    if matches_any(id, id_like, &["nixos"]) {
        return DistroFamily::Nix;
    }

    if matches_field(variant_id, &["silverblue", "kinoite", "coreos", "aurora"])
        || matches_any(
            id,
            id_like,
            &["silverblue", "kinoite", "ublue", "fedora-immutable"],
        )
    {
        return DistroFamily::Immutable;
    }

    if matches_any(id, id_like, &["fedora", "rhel", "centos"]) {
        return DistroFamily::Fedora;
    }

    if matches_any(id, id_like, &["debian", "ubuntu"]) {
        return DistroFamily::Debian;
    }

    if matches_any(id, id_like, &["arch", "manjaro"]) {
        return DistroFamily::Arch;
    }

    if matches_any(id, id_like, &["opensuse", "suse", "sles"]) {
        return DistroFamily::OpenSuse;
    }

    if matches_any(id, id_like, &["alpine"]) {
        return DistroFamily::Alpine;
    }

    DistroFamily::Generic
}

fn lookup_field<'a>(os_release: &'a str, key: &str) -> Option<&'a str> {
    os_release
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}=")))
        .map(trim_value)
}

fn trim_value(value: &str) -> &str {
    value.trim().trim_matches('"')
}

fn matches_any(id: Option<&str>, id_like: Option<&str>, needles: &[&str]) -> bool {
    matches_field(id, needles) || matches_field(id_like, needles)
}

fn matches_field(field: Option<&str>, needles: &[&str]) -> bool {
    field
        .into_iter()
        .flat_map(|value| value.split_ascii_whitespace())
        .any(|candidate| needles.contains(&candidate))
}
