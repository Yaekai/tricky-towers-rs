//! Mods: a `mods/` folder whose files change the game without touching it.
//!
//! - `mods/<name>.ron`, or `mods/<name>/config.ron`: config changes. Only the
//!   fields a mod writes change; nested sections merge field by field, so
//!   `(race: (finish_height: 20.0))` keeps every other race rule.
//! - `mods/<name>/manifest.ron` with files beside it: an art and sound pack in
//!   the same format as `extracted/manifest.ron` (docs/art-layout.md). Its
//!   entries replace the extracted ones of the same name.
//!
//! Mods apply in name order, so a later one wins. See docs/modding.md.

use std::path::{Path, PathBuf};

use bevy::prelude::*;

use crate::config::GameConfig;

/// Asset source name for the mods folder: `mods://<mod>/<file>`.
pub const SOURCE: &str = "mods";

/// The `mods/` folder: in the working folder, or next to the program or one of
/// its parent folders.
pub fn mods_dir() -> PathBuf {
    let mut candidates = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("mods"));
    }
    if let Ok(exe) = std::env::current_exe() {
        candidates.extend(exe.ancestors().skip(1).map(|d| d.join("mods")));
    }
    candidates
        .into_iter()
        .find(|d| d.is_dir())
        .unwrap_or_else(|| PathBuf::from("mods"))
}

/// One mod found in the folder.
#[derive(Clone, Debug)]
pub struct Mod {
    pub name: String,
    /// Its config changes, if it has any.
    pub config: Option<PathBuf>,
    /// Its art pack's folder, if it has a `manifest.ron`.
    pub art: Option<PathBuf>,
}

/// The mods in `dir`, in the order they apply.
pub fn list_mods(dir: &Path) -> Vec<Mod> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut mods: Vec<Mod> = read
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_stem()?.to_string_lossy().into_owned();
            if path.is_dir() {
                let config = Some(path.join("config.ron")).filter(|p| p.is_file());
                let art = Some(path.clone()).filter(|p| p.join("manifest.ron").is_file());
                (config.is_some() || art.is_some()).then_some(Mod { name, config, art })
            } else if path.extension().is_some_and(|e| e == "ron") {
                Some(Mod {
                    name,
                    config: Some(path),
                    art: None,
                })
            } else {
                None
            }
        })
        .collect();
    mods.sort_by(|a, b| a.name.cmp(&b.name));
    mods
}

/// `config` with every mod's config changes in `dir` applied. A mod that
/// doesn't read is skipped with an error in the log.
pub fn apply_config_mods(config: GameConfig, dir: &Path) -> GameConfig {
    let mut config = config;
    for m in list_mods(dir) {
        let Some(path) = &m.config else { continue };
        let changed = std::fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|text| config.with_changes(&text));
        match changed {
            Ok(next) => {
                info!("mod {}: config applied", m.name);
                config = next.validated();
            }
            Err(err) => error!("mod {}: {err}; skipped", m.name),
        }
    }
    config
}

impl GameConfig {
    /// This config with the fields `text` writes changed (RON, any subset of
    /// the config's fields).
    pub fn with_changes(&self, text: &str) -> Result<GameConfig, String> {
        let base =
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::new().struct_names(false))
                .map_err(|e| e.to_string())?;
        let (extensions, changes) = split_extensions(&strip_comments(text));
        let merged = format!("{extensions}{}", merge_ron(&base, &changes));
        ron::from_str(&merged).map_err(|e| e.to_string())
    }
}

/// Removes `//` and `/* */` comments outside strings.
fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_string = false;
    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if c == '\\' {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match (c, chars.peek()) {
            ('"', _) => {
                in_string = true;
                out.push(c);
            }
            ('/', Some('/')) => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            ('/', Some('*')) => {
                chars.next();
                let mut last = ' ';
                for c in chars.by_ref() {
                    if last == '*' && c == '/' {
                        break;
                    }
                    last = c;
                }
                out.push(' ');
            }
            _ => out.push(c),
        }
    }
    out
}

/// Splits `#![enable(...)]` lines off the front.
fn split_extensions(text: &str) -> (String, String) {
    let mut extensions = String::new();
    let mut rest = text.trim_start();
    while rest.starts_with("#!") {
        let end = rest.find(']').map_or(rest.len(), |i| i + 1);
        extensions.push_str(&rest[..end]);
        extensions.push('\n');
        rest = rest[end..].trim_start();
    }
    (extensions, rest.to_string())
}

/// Splits `text` at commas that aren't inside brackets or strings.
fn split_top(text: &str, at: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let (mut depth, mut start, mut in_string, mut escaped) = (0i32, 0, false, false);
    for (i, c) in text.char_indices() {
        if in_string {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            c if c == at && depth == 0 => {
                parts.push(&text[start..i]);
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&text[start..]);
    parts
}

/// The `name: value` fields of a RON struct written as `(a: 1, b: 2)` or
/// `Name(a: 1)`, or `None` for anything else (a tuple, a list, an enum value).
fn struct_fields(text: &str) -> Option<Vec<(String, String)>> {
    let text = text.trim();
    let open = text.find('(')?;
    let name = &text[..open];
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') || !text.ends_with(')') {
        return None;
    }
    // A named value with fields could also be an enum variant; only plain
    // `( ... )` and the outermost struct are merged field by field.
    let inner = &text[open + 1..text.len() - 1];
    let mut fields = Vec::new();
    for part in split_top(inner, ',') {
        if part.trim().is_empty() {
            continue;
        }
        let pieces = split_top(part, ':');
        let key = pieces[0].trim();
        if pieces.len() < 2
            || key.is_empty()
            || !key.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            return None;
        }
        let value = part[part.find(':')? + 1..].trim();
        fields.push((key.to_string(), value.to_string()));
    }
    (!fields.is_empty()).then_some(fields)
}

/// `base` with `changes` written over it: struct fields merge one by one, and
/// anything else is replaced whole.
pub fn merge_ron(base: &str, changes: &str) -> String {
    let base_is_plain = base.trim_start().starts_with('(');
    let changes_is_plain = changes.trim_start().starts_with('(');
    match (struct_fields(base), struct_fields(changes)) {
        (Some(mut fields), Some(over)) if base_is_plain && changes_is_plain => {
            for (key, value) in over {
                match fields.iter_mut().find(|(k, _)| *k == key) {
                    Some((_, old)) => *old = merge_ron(old, &value),
                    None => fields.push((key, value)),
                }
            }
            let body: Vec<String> = fields.iter().map(|(k, v)| format!("{k}: {v}")).collect();
            format!("({})", body.join(", "))
        }
        _ => changes.trim().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ModeKind;

    #[test]
    fn a_mod_changes_only_the_fields_it_writes() {
        let base = GameConfig::default();
        let text = "// faster races\n#![enable(implicit_some)]\n(\n  mode: Puzzle, /* note */\n  race: (finish_height: 20.0),\n  gravity: 30.0,\n)";
        let modded = base.with_changes(text).unwrap();
        assert_eq!(modded.mode, ModeKind::Puzzle);
        assert_eq!(modded.race.finish_height, 20.0);
        assert_eq!(modded.race.hold_seconds, base.race.hold_seconds);
        assert_eq!(modded.gravity, 30.0);
        assert_eq!(modded.pieces.len(), base.pieces.len());
        assert_eq!(modded.survival.lives, base.survival.lives);
    }

    #[test]
    fn tuples_and_lists_are_replaced_whole() {
        assert_eq!(
            merge_ron("(a: (1, 2), b: [1])", "(a: (3, 4))"),
            "(a: (3, 4), b: [1])"
        );
        assert_eq!(merge_ron("(a: [1, 2])", "(a: [])"), "(a: [])");
        assert_eq!(merge_ron("(a: Some(1))", "(a: None)"), "(a: None)");
        assert_eq!(
            merge_ron("(s: \"a, b: c\")", "(t: 1)"),
            "(s: \"a, b: c\", t: 1)"
        );
    }

    #[test]
    fn mods_apply_in_name_order() {
        let dir = std::env::temp_dir().join(format!("tt_mods_{}", std::process::id()));
        let _ = std::fs::create_dir_all(dir.join("b_pack"));
        std::fs::write(dir.join("a.ron"), "(gravity: 10.0, cell_size: 40.0)").unwrap();
        std::fs::write(dir.join("b_pack/config.ron"), "(gravity: 12.0)").unwrap();
        std::fs::write(dir.join("c.ron"), "(gravity: oops").unwrap();
        let config = apply_config_mods(GameConfig::default(), &dir);
        assert_eq!((config.gravity, config.cell_size), (12.0, 40.0));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
