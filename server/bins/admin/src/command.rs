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
    ValidateSeed(Option<String>),
    ValidateAll,
    ImportLevels(Option<String>),
    ImportSeed(Option<String>),
    ImportAll,
}

pub const USAGE: &str = "\
commands:
  help                     show this help
  status                   show target server, database, and content paths
  set server <name>        change the target server
  set database <name>      change the target database
  validate levels [path]   validate Tiled level exports (no mutation)
  validate seed [path]     validate seed content (no mutation)
  validate all             validate levels and seed together (no mutation)
  import levels [path]     import levels after confirmation
  import seed [path]       import seed content after confirmation
  import all               import seed then levels after confirmation
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
        ["validate", "seed", rest @ ..] => Command::ValidateSeed(optional_path(rest)?),
        ["validate", "all"] => Command::ValidateAll,
        ["import", "levels", rest @ ..] => Command::ImportLevels(optional_path(rest)?),
        ["import", "seed", rest @ ..] => Command::ImportSeed(optional_path(rest)?),
        ["import", "all"] => Command::ImportAll,
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
            parse_line("import levels ../levels/generated").unwrap(),
            Some(Command::ImportLevels(Some("../levels/generated".into())))
        );
        assert_eq!(
            parse_line("import seed custom/seed.json").unwrap(),
            Some(Command::ImportSeed(Some("custom/seed.json".into())))
        );
        assert_eq!(parse_line("import all").unwrap(), Some(Command::ImportAll));
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
        assert!(parse_line("frobnicate").is_err());
    }
}
