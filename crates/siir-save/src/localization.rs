use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn languages(locale_dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(locale_dir) else {
        return Vec::new();
    };
    let mut found: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_ok_and(|kind| kind.is_dir())
                && entry.path().join("local.sii").is_file()
        })
        .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
        .collect();
    found.sort();
    found
}

pub(crate) fn read(locale_dir: &Path, language: &str) -> BTreeMap<String, String> {
    let mut texts = BTreeMap::new();
    let mut seen = Vec::new();
    let folder = locale_dir.join(language);
    load(&folder.join("local.sii"), locale_dir, &mut texts, &mut seen);
    load(
        &folder.join("local.override.sii"),
        locale_dir,
        &mut texts,
        &mut seen,
    );
    texts
}

fn load(
    file: &Path,
    locale_dir: &Path,
    texts: &mut BTreeMap<String, String>,
    seen: &mut Vec<PathBuf>,
) {
    if seen.iter().any(|seen| seen == file) {
        return;
    }
    seen.push(file.to_path_buf());
    let Ok(source) = fs::read_to_string(file) else {
        return;
    };
    let mut key: Option<String> = None;
    for line in source.lines() {
        let line = line.trim_start_matches('\u{feff}').trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(include) = quoted_after(line, "@include") {
            let target = match include.strip_prefix("/locale/") {
                Some(rest) => locale_dir.join(rest),
                None if include.starts_with('/') => {
                    locale_dir.join(include.trim_start_matches('/'))
                }
                None => file.parent().unwrap_or(locale_dir).join(include),
            };
            load(&target, locale_dir, texts, seen);
        } else if let Some(found) = quoted_after(line, "key[]:") {
            key = Some(found);
        } else if let Some(found) = quoted_after(line, "val[]:")
            && let Some(key) = key.take()
        {
            texts.insert(key, found);
        }
    }
}

fn quoted_after(line: &str, prefix: &str) -> Option<String> {
    let rest = line.strip_prefix(prefix)?.trim_start();
    let start = rest.find('"')? + 1;
    let end = rest.rfind('"')?;
    if end <= start {
        return None;
    }
    Some(unescape(&rest[start..end]))
}

fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        match chars.next() {
            Some(quoted @ ('"' | '\\')) => out.push(quoted),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locale() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/locale")
    }

    #[test]
    fn resolves_includes_quotes_and_language_overrides() {
        let english = read(&locale(), "en_us");
        assert_eq!(english.get("brand_test").map(String::as_str), Some("Test"));
        assert_eq!(
            english.get("series_test_truck").map(String::as_str),
            Some("T 900")
        );
        assert_eq!(english.get("nested").map(String::as_str), Some("Nested"));
        assert_eq!(
            english.get("shared_quote").map(String::as_str),
            Some("A \"quoted\" value")
        );

        let portuguese = read(&locale(), "pt_br");
        assert_eq!(
            portuguese.get("brand_test").map(String::as_str),
            Some("Test")
        );
        assert_eq!(
            portuguese.get("shared_quote").map(String::as_str),
            Some("Um valor \"entre aspas\"")
        );
    }

    #[test]
    fn override_file_wins_over_local_texts() {
        let english = read(&locale(), "en_us");
        assert_eq!(
            english.get("lvl_name1").map(String::as_str),
            Some("Workhorse")
        );
        assert_eq!(english.get("lvl_name0").map(String::as_str), Some("Newbie"));
        let portuguese = read(&locale(), "pt_br");
        assert_eq!(
            portuguese.get("lvl_name1").map(String::as_str),
            Some("Entusiasta")
        );
    }
}
