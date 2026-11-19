//! Game `config.cfg` and `cams.txt` helpers.

use std::fs;
use std::path::Path;

use crate::error::Error;
use crate::profiles::replace_file;

/// Developer, console, and convoy settings from `config.cfg`.
#[derive(Debug)]
pub struct Config {
    /// `g_developer` is enabled.
    pub developer: bool,
    /// `g_console` is enabled.
    pub console: bool,
    /// `g_max_convoy_size` holds the large-convoy value.
    pub max_convoy: bool,
}

/// A camera placement from the last line of `cams.txt`.
#[derive(Debug)]
pub struct Camera {
    /// Location vector text.
    pub location: String,
    /// Rotation vector text.
    pub rotation: String,
}

/// Reads the developer, console, and convoy settings.
pub fn read(config: &Path) -> Result<Config, Error> {
    let (_, lines) = read_lines(config)?;
    let mut developer = None;
    let mut console = None;
    let mut max_convoy = None;
    for line in &lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            continue;
        }
        match parts[1] {
            "g_developer" => developer = Some(parts[2].trim_matches('"') == "1"),
            "g_console" => console = Some(parts[2].trim_matches('"') == "1"),
            "g_max_convoy_size" => max_convoy = Some(parts[2].trim_matches('"') == "128"),
            _ => {}
        }
    }
    Ok(Config {
        developer: developer.ok_or(Error::MissingConfig {
            key: "g_developer".to_owned(),
        })?,
        console: console.ok_or(Error::MissingConfig {
            key: "g_console".to_owned(),
        })?,
        max_convoy: max_convoy.unwrap_or(false),
    })
}

/// The display currency `g_currency` selects: an index into
/// [`Catalog::currencies`](crate::catalog::Catalog::currencies).
pub fn currency(config: &Path) -> Result<usize, Error> {
    let (_, lines) = read_lines(config)?;
    let missing = || Error::MissingConfig {
        key: "g_currency".to_owned(),
    };
    lines
        .iter()
        .find_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            (parts.len() == 3 && parts[1] == "g_currency").then(|| parts[2].trim_matches('"'))
        })
        .ok_or_else(missing)?
        .parse()
        .map_err(|_| missing())
}

/// Enables or disables developer mode and the console.
pub fn set_developer(config: &Path, enabled: bool) -> Result<(), Error> {
    let value = if enabled { "1" } else { "0" };
    update(config, &[("g_developer", value), ("g_console", value)])
}

/// Enables the large convoy size or restores the default.
pub fn set_convoy_size(config: &Path, enabled: bool) -> Result<(), Error> {
    update(
        config,
        &[("g_max_convoy_size", if enabled { "128" } else { "0" })],
    )
}

/// Reads the last placement written to `cams.txt`.
pub fn read_camera(path: &Path) -> Result<Camera, Error> {
    let text = fs::read_to_string(path)?;
    let line = text
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .ok_or(Error::InvalidCamera)?;
    let line = line.replace(' ', "");
    let parts: Vec<&str> = line.split(';').collect();
    if parts.len() < 8 {
        return Err(Error::InvalidCamera);
    }
    for part in &parts[..8] {
        part.parse::<f32>().map_err(|_| Error::InvalidCamera)?;
    }
    Ok(Camera {
        location: format!("({},{},{})", parts[1], parts[2], parts[3]),
        rotation: format!("({};{},{},{})", parts[4], parts[5], parts[6], parts[7]),
    })
}

fn update(config: &Path, changes: &[(&str, &str)]) -> Result<(), Error> {
    let (newline, mut lines) = read_lines(config)?;
    let mut seen = vec![false; changes.len()];
    for line in &mut lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            continue;
        }
        if let Some(position) = changes.iter().position(|(key, _)| *key == parts[1]) {
            *line = format!("uset {} \"{}\"", changes[position].0, changes[position].1);
            seen[position] = true;
        }
    }
    if let Some(position) = seen.iter().position(|found| !found) {
        return Err(Error::MissingConfig {
            key: changes[position].0.to_owned(),
        });
    }
    let name = config
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "config.cfg".to_owned());
    let tmp = config.with_file_name(format!("{name}.{}.tmp", std::process::id()));
    let result = (|| -> Result<(), Error> {
        fs::write(&tmp, lines.join(&newline))?;
        replace_file(&tmp, config)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

fn read_lines(path: &Path) -> Result<(String, Vec<String>), Error> {
    let text = fs::read_to_string(path)?;
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let lines = text.split(newline).map(str::to_owned).collect();
    Ok((newline.to_owned(), lines))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> std::path::PathBuf {
        let path =
            std::env::temp_dir().join(format!("siir-save-config-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn reads_and_updates_developer_and_convoy() {
        let path = temp("config").join("config.cfg");
        fs::write(
            &path,
            "uset g_developer \"0\"\r\nuset g_console \"0\"\r\nuset g_max_convoy_size \"0\"\r\n",
        )
        .unwrap();

        let config = read(&path).unwrap();
        assert!(!config.developer);
        assert!(!config.console);
        assert!(!config.max_convoy);

        set_developer(&path, true).unwrap();
        set_convoy_size(&path, true).unwrap();
        let config = read(&path).unwrap();
        assert!(config.developer);
        assert!(config.console);
        assert!(config.max_convoy);
        assert!(fs::read_to_string(&path).unwrap().contains("\r\n"));

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn defaults_missing_convoy_to_false() {
        let path = temp("no-convoy").join("config.cfg");
        fs::write(&path, "uset g_developer \"1\"\nuset g_console \"1\"\n").unwrap();
        let config = read(&path).unwrap();
        assert!(config.developer);
        assert!(config.console);
        assert!(!config.max_convoy);

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn reads_the_display_currency() {
        let path = temp("currency").join("config.cfg");
        fs::write(&path, "uset g_lang \"pt_br\"\r\nuset g_currency \"3\"\r\n").unwrap();
        assert_eq!(currency(&path).unwrap(), 3);
        fs::write(&path, "uset g_lang \"pt_br\"\n").unwrap();
        assert!(currency(&path).is_err());
        fs::write(&path, "uset g_currency \"x\"\n").unwrap();
        assert!(currency(&path).is_err());

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn reads_the_last_camera_placement() {
        let path = temp("cams").join("cams.txt");
        fs::write(&path, "1;9;9;9;1;9;9;9\n0;1.0;2.0;3.0;1.0;0.0;0.0;0.0\n").unwrap();
        let camera = read_camera(&path).unwrap();
        assert_eq!(camera.location, "(1.0,2.0,3.0)");
        assert_eq!(camera.rotation, "(1.0;0.0,0.0,0.0)");

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn rejects_a_non_numeric_camera() {
        let path = temp("bad-numbers").join("cams.txt");
        fs::write(&path, "0;a;b;c;1;0;0;0\n").unwrap();
        assert_eq!(
            read_camera(&path).unwrap_err().to_string(),
            "cams.txt has no parseable placement"
        );

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn reports_missing_settings_and_bad_camera() {
        let path = temp("bad").join("config.cfg");
        fs::write(&path, "uset g_developer \"0\"\n").unwrap();
        let error = read(&path).unwrap_err();
        assert_eq!(error.to_string(), "config setting \"g_console\" is missing");

        let cams = path.with_file_name("cams.txt");
        fs::write(&cams, "bad\n").unwrap();
        assert_eq!(
            read_camera(&cams).unwrap_err().to_string(),
            "cams.txt has no parseable placement"
        );

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }
}
