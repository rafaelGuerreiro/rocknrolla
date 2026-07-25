//! Shell line parsing, kept separate from execution so commands are testable
//! without an interactive terminal.

use anyhow::{Result, bail};

/// One parsed shell command.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Status,
    Quit,
    SetDatabase(String),
    SetServer(String),
    ValidateLevels(Option<String>),
    ValidateComponents(Option<String>),
    ValidateCharacters(Option<String>),
    ValidateFaces(Option<String>),
    ValidateBackdrops(Option<String>),
    ValidateSeed(Option<String>),
    ValidateAll,
    ImportLevels(Option<String>),
    ImportComponents(Option<String>),
    ImportCharacters(Option<String>),
    ImportFaces(Option<String>),
    ImportBackdrops(Option<String>),
    ExportComponents(String),
    ImportSeed(Option<String>),
    ImportAll,
}

pub const USAGE: &str = "\
commands:
  help                     show this help
  status                   show target server, database, and content paths
  set server <name>        change the target server
  set database <name>      change the target database
  validate components [path]  validate authored components (no mutation)
  validate characters [path]  validate authored character art (no mutation)
  validate faces [path]    validate authored face expressions (no mutation)
  validate backdrops [path]   validate authored backdrops (no mutation)
  validate levels [path]   validate authored level sources (no mutation)
  validate seed [path]     validate seed content (no mutation)
  validate all             validate every authored content type (no mutation)
  import components [path] import components after confirmation
  import characters [path] import character art after confirmation
  import faces [path]      import face expressions after confirmation
  import backdrops [path]  import backdrops after confirmation
  import levels [path]     import levels after confirmation
  import seed [path]       import seed content after confirmation
  import all               import seed, components, character art, faces, backdrops, then levels after confirmation
  export components <dir>  render the starter component library to SVG files
  quit | exit              leave the shell (EOF also exits)";

/// Parse one input line. `Ok(None)` means an empty line to ignore.
pub fn parse_line(line: &str) -> Result<Option<Command>> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let command = match words.as_slice() {
        [] => return Ok(None),
        ["help"] => Command::Help,
        ["status"] => Command::Status,
        ["quit"] | ["exit"] => Command::Quit,
        ["set", "server", name] => Command::SetServer(name.to_string()),
        ["set", "database", name] => Command::SetDatabase(name.to_string()),
        ["set", "server"] | ["set", "database"] => {
            bail!("'{}' needs a value", line.trim());
        },
        ["validate", "levels", rest @ ..] => Command::ValidateLevels(optional_path(rest)?),
        ["validate", "components", rest @ ..] => Command::ValidateComponents(optional_path(rest)?),
        ["validate", "characters", rest @ ..] => Command::ValidateCharacters(optional_path(rest)?),
        ["validate", "faces", rest @ ..] => Command::ValidateFaces(optional_path(rest)?),
        ["validate", "backdrops", rest @ ..] => Command::ValidateBackdrops(optional_path(rest)?),
        ["validate", "seed", rest @ ..] => Command::ValidateSeed(optional_path(rest)?),
        ["validate", "all"] => Command::ValidateAll,
        ["import", "levels", rest @ ..] => Command::ImportLevels(optional_path(rest)?),
        ["import", "components", rest @ ..] => Command::ImportComponents(optional_path(rest)?),
        ["import", "characters", rest @ ..] => Command::ImportCharacters(optional_path(rest)?),
        ["import", "faces", rest @ ..] => Command::ImportFaces(optional_path(rest)?),
        ["import", "backdrops", rest @ ..] => Command::ImportBackdrops(optional_path(rest)?),
        ["import", "seed", rest @ ..] => Command::ImportSeed(optional_path(rest)?),
        ["import", "all"] => Command::ImportAll,
        ["export", "components", dir] => Command::ExportComponents(dir.to_string()),
        ["export", "components"] => bail!("'export components' needs a target directory"),
        _ => bail!("unknown command '{}'", line.trim()),
    };
    Ok(Some(command))
}

fn optional_path(rest: &[&str]) -> Result<Option<String>> {
    match rest {
        [] => Ok(None),
        [path] => Ok(Some(path.to_string())),
        _ => bail!("expected at most one path"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_commands() {
        assert_eq!(parse_line("help").unwrap(), Some(Command::Help));
        assert_eq!(parse_line(" status ").unwrap(), Some(Command::Status));
        assert_eq!(parse_line("validate all").unwrap(), Some(Command::ValidateAll));
        assert_eq!(parse_line("validate levels").unwrap(), Some(Command::ValidateLevels(None)));
        assert_eq!(
            parse_line("import levels ../content/generated").unwrap(),
            Some(Command::ImportLevels(Some("../content/generated".into())))
        );
        assert_eq!(
            parse_line("import seed custom/seed.json").unwrap(),
            Some(Command::ImportSeed(Some("custom/seed.json".into())))
        );
        assert_eq!(parse_line("import all").unwrap(), Some(Command::ImportAll));
        assert_eq!(
            parse_line("validate components ../content/components").unwrap(),
            Some(Command::ValidateComponents(Some("../content/components".into())))
        );
        assert_eq!(
            parse_line("import components").unwrap(),
            Some(Command::ImportComponents(None))
        );
        assert_eq!(
            parse_line("export components /tmp/out").unwrap(),
            Some(Command::ExportComponents("/tmp/out".into()))
        );
    }

    #[test]
    fn parses_new_content_commands() {
        assert_eq!(
            parse_line("validate characters").unwrap(),
            Some(Command::ValidateCharacters(None))
        );
        assert_eq!(parse_line("validate faces").unwrap(), Some(Command::ValidateFaces(None)));
        assert_eq!(
            parse_line("validate backdrops ../content/backdrops").unwrap(),
            Some(Command::ValidateBackdrops(Some("../content/backdrops".into())))
        );
        assert_eq!(
            parse_line("import characters").unwrap(),
            Some(Command::ImportCharacters(None))
        );
        assert_eq!(
            parse_line("import faces custom/faces").unwrap(),
            Some(Command::ImportFaces(Some("custom/faces".into())))
        );
        assert_eq!(parse_line("import backdrops").unwrap(), Some(Command::ImportBackdrops(None)));
    }

    #[test]
    fn parses_configuration_commands() {
        assert_eq!(
            parse_line("set server maincloud").unwrap(),
            Some(Command::SetServer("maincloud".into()))
        );
        assert_eq!(
            parse_line("set database rocknrolladb").unwrap(),
            Some(Command::SetDatabase("rocknrolladb".into()))
        );
    }

    #[test]
    fn parses_exit_commands_and_blank_lines() {
        assert_eq!(parse_line("quit").unwrap(), Some(Command::Quit));
        assert_eq!(parse_line("exit").unwrap(), Some(Command::Quit));
        assert_eq!(parse_line("   ").unwrap(), None);
        assert_eq!(parse_line("").unwrap(), None);
    }

    #[test]
    fn rejects_malformed_commands() {
        assert!(parse_line("import").is_err());
        assert!(parse_line("set server").is_err());
        assert!(parse_line("set database").is_err());
        assert!(parse_line("validate levels a b").is_err());
        assert!(parse_line("validate characters a b").is_err());
        assert!(parse_line("frobnicate").is_err());
        assert!(parse_line("export components").is_err());
        assert!(parse_line("export levels /tmp/out").is_err());
    }
}
