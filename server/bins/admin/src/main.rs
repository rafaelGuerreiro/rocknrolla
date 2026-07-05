//! RocknRolla content administration shell.
//!
//! An interactive shell that validates committed Tiled JSON and seed content,
//! then imports them through the owner-only reducers using the locally
//! installed, already authenticated `spacetime` CLI session. This binary
//! never reads or prints the `.env` token; run `task server:login` first.

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::Command as Process;

mod command;
mod seed;
mod tiled;
mod uuid;

use command::Command;

const DEFAULT_DATABASE: &str = "rocknrolladb-dev";
const PRODUCTION_DATABASE: &str = "rocknrolladb";
const DEFAULT_SERVER: &str = "maincloud";
const DEFAULT_LEVELS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../levels/generated");
const DEFAULT_SEED_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../levels/seed.json");

/// Target and content-path state kept alive across shell commands.
struct Session {
    server: String,
    database: String,
    levels_path: PathBuf,
    seed_path: PathBuf,
}

impl Default for Session {
    fn default() -> Self {
        Session {
            server: DEFAULT_SERVER.to_string(),
            database: DEFAULT_DATABASE.to_string(),
            levels_path: default_path(DEFAULT_LEVELS_PATH),
            seed_path: default_path(DEFAULT_SEED_PATH),
        }
    }
}

fn default_path(path: &str) -> PathBuf {
    PathBuf::from(path)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(path))
}

fn main() {
    let stdin = std::io::stdin();
    let mut input = stdin.lock().lines();
    let mut session = Session::default();
    println!("RocknRolla admin shell. Type 'help' for commands.");
    loop {
        print!("admin> ");
        let _ = std::io::stdout().flush();
        let Some(line) = input.next() else {
            println!();
            break;
        };
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("error: cannot read input: {error}");
                break;
            }
        };
        match command::parse_line(&line) {
            Ok(None) => {}
            Ok(Some(Command::Quit)) => break,
            Ok(Some(parsed)) => {
                if let Err(error) = execute(&mut session, parsed, &mut input) {
                    eprintln!("error: {error}");
                }
            }
            Err(error) => eprintln!("error: {error} (type 'help' for usage)"),
        }
    }
}

fn execute(
    session: &mut Session,
    parsed: Command,
    input: &mut impl Iterator<Item = std::io::Result<String>>,
) -> Result<(), String> {
    match parsed {
        Command::Quit => unreachable!("quit is handled by the loop"),
        Command::Help => println!("{}", command::USAGE),
        Command::Status => print_status(session),
        Command::SetServer(server) => {
            session.server = server;
            println!("server set to '{}'", session.server);
        }
        Command::SetDatabase(database) => {
            session.database = database;
            println!("database set to '{}'", session.database);
        }
        Command::ValidateLevels(path) => {
            let levels = load_levels(session, path.as_deref())?;
            println!("{} level(s) valid; nothing imported", levels.len());
        }
        Command::ValidateSeed(path) => {
            load_seed(session, path.as_deref())?;
            println!("seed content valid; nothing imported");
        }
        Command::ValidateAll => {
            let content = load_seed(session, None)?;
            let levels = load_levels(session, None)?;
            println!(
                "seed content and {} level(s) valid; nothing imported ({} characters, {} pieces, {} lootboxes)",
                levels.len(),
                content.characters.len(),
                content.pieces.len(),
                content.lootboxes.len()
            );
        }
        Command::ImportLevels(path) => {
            let levels = load_levels(session, path.as_deref())?;
            if confirm_import(session, input)? {
                import_levels(session, &levels)?;
            }
        }
        Command::ImportSeed(path) => {
            let content = load_seed(session, path.as_deref())?;
            if confirm_import(session, input)? {
                import_seed(session, &content)?;
            }
        }
        Command::ImportAll => {
            let content = load_seed(session, None)?;
            let levels = load_levels(session, None)?;
            if confirm_import(session, input)? {
                import_seed(session, &content)?;
                import_levels(session, &levels)?;
            }
        }
    }
    Ok(())
}

fn print_status(session: &Session) {
    println!("server:   {}", session.server);
    println!("database: {}", session.database);
    println!("levels:   {}", session.levels_path.display());
    println!("seed:     {}", session.seed_path.display());
}

/// Show the destination and require explicit confirmation. Importing into the
/// production database demands an additional unmistakable confirmation.
fn confirm_import(
    session: &Session,
    input: &mut impl Iterator<Item = std::io::Result<String>>,
) -> Result<bool, String> {
    println!(
        "about to import into database '{}' on server '{}'",
        session.database, session.server
    );
    if !prompt_matches(input, "type 'yes' to continue: ", "yes")? {
        println!("cancelled; nothing was imported");
        return Ok(false);
    }
    if session.database == PRODUCTION_DATABASE {
        println!("'{PRODUCTION_DATABASE}' is the PRODUCTION database");
        let prompt = format!("type the database name '{PRODUCTION_DATABASE}' to confirm: ");
        if !prompt_matches(input, &prompt, PRODUCTION_DATABASE)? {
            println!("cancelled; nothing was imported");
            return Ok(false);
        }
    }
    Ok(true)
}

fn prompt_matches(
    input: &mut impl Iterator<Item = std::io::Result<String>>,
    prompt: &str,
    expected: &str,
) -> Result<bool, String> {
    print!("{prompt}");
    let _ = std::io::stdout().flush();
    match input.next() {
        Some(Ok(answer)) => Ok(answer.trim() == expected),
        Some(Err(error)) => Err(format!("cannot read confirmation: {error}")),
        None => Ok(false),
    }
}

fn load_seed(session: &Session, path_override: Option<&str>) -> Result<seed::SeedContent, String> {
    let path = path_override
        .map(PathBuf::from)
        .unwrap_or_else(|| session.seed_path.clone());
    let source = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let content = seed::parse_seed(&source).map_err(|e| format!("{}: {e}", path.display()))?;
    println!(
        "validated {}: {} characters, {} pieces, {} lootboxes",
        path.display(),
        content.characters.len(),
        content.pieces.len(),
        content.lootboxes.len()
    );
    Ok(content)
}

fn load_levels(
    session: &Session,
    path_override: Option<&str>,
) -> Result<Vec<tiled::ImportedLevel>, String> {
    let path = path_override
        .map(PathBuf::from)
        .unwrap_or_else(|| session.levels_path.clone());
    let files = collect_json_files(&path)?;
    let mut levels = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file)
            .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
        let level = tiled::parse_level(&source).map_err(|e| format!("{}: {e}", file.display()))?;
        println!(
            "validated {}: level '{}' ({} layers, {} compressed bytes, gameplay hash {})",
            file.display(),
            level.slug,
            level.layers.len(),
            level.layers.iter().map(|l| l.data.len()).sum::<usize>(),
            level
                .layers
                .iter()
                .find(|l| l.z == rocknrolla_level::GAMEPLAY_Z)
                .map(|l| l.content_hash.as_str())
                .unwrap_or("-"),
        );
        levels.push(level);
    }
    reject_duplicate_levels(&levels)?;
    Ok(levels)
}

fn reject_duplicate_levels(levels: &[tiled::ImportedLevel]) -> Result<(), String> {
    let mut ids = std::collections::HashSet::new();
    let mut slugs = std::collections::HashSet::new();
    for level in levels {
        if !ids.insert(level.id.to_lowercase()) {
            return Err(format!("duplicate level id '{}'", level.id));
        }
        if !slugs.insert(level.slug.clone()) {
            return Err(format!("duplicate level slug '{}'", level.slug));
        }
    }
    Ok(())
}

fn collect_json_files(path: &PathBuf) -> Result<Vec<PathBuf>, String> {
    if path.is_file() {
        return Ok(vec![path.clone()]);
    }
    if !path.is_dir() {
        return Err(format!("path not found: {}", path.display()));
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(path)
        .map_err(|e| format!("cannot read directory {}: {e}", path.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    if files.is_empty() {
        return Err(format!("no JSON files found in {}", path.display()));
    }
    Ok(files)
}

fn call_reducer(session: &Session, reducer: &str, args: &[String]) -> Result<(), String> {
    let output = Process::new("spacetime")
        .arg("call")
        .arg("--server")
        .arg(&session.server)
        .arg("--yes")
        .arg(&session.database)
        .arg(reducer)
        .args(args)
        .output()
        .map_err(|e| format!("failed to run the spacetime CLI: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "spacetime call {reducer} failed: {}{}",
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn layer_to_json(layer: &rocknrolla_level::LayerFacts) -> serde_json::Value {
    serde_json::json!({
        "z": layer.z,
        "width": layer.width,
        "height": layer.height,
        "cell_width": layer.cell_width,
        "cell_height": layer.cell_height,
        "parallax_x": layer.parallax_x,
        "parallax_y": layer.parallax_y,
        "encoding": layer.encoding,
        "content_hash": layer.content_hash,
        "data": layer.data,
    })
}

fn import_levels(session: &Session, levels: &[tiled::ImportedLevel]) -> Result<(), String> {
    for level in levels {
        let args = vec![
            crate::uuid::uuid_arg(&level.id),
            serde_json::json!(level.slug).to_string(),
            serde_json::json!(level.name).to_string(),
            level.is_starting.to_string(),
            level.active.to_string(),
            crate::uuid::uuid_opt_arg(level.reward_lootbox_id.as_deref()),
            crate::uuid::uuid_vec_arg(&level.successors),
            serde_json::Value::Array(level.layers.iter().map(layer_to_json).collect()).to_string(),
        ];
        call_reducer(session, "import_level", &args)?;
        println!("imported level '{}' into {}", level.slug, session.database);
    }
    Ok(())
}

fn import_seed(session: &Session, content: &seed::SeedContent) -> Result<(), String> {
    for character in &content.characters {
        let args = vec![
            crate::uuid::uuid_arg(&character.id),
            serde_json::json!(character.name).to_string(),
            serde_json::json!(character.style).to_string(),
            character.rarity_weight.to_string(),
            character.density.to_string(),
            character.jump_speed.to_string(),
            character.flight_time_ms.to_string(),
            character.buoyancy.to_string(),
            character.fire_resistance.to_string(),
            character.starter.to_string(),
        ];
        call_reducer(session, "import_character", &args)?;
        println!("imported character '{}'", character.name);
    }
    for piece in &content.pieces {
        let args = vec![
            crate::uuid::uuid_arg(&piece.id),
            serde_json::json!(piece.name).to_string(),
            crate::uuid::uuid_arg(&piece.character_id),
        ];
        call_reducer(session, "import_piece", &args)?;
        println!("imported piece '{}'", piece.name);
    }
    for lootbox in &content.lootboxes {
        let drops: Vec<String> = lootbox
            .drops
            .iter()
            .map(|d| format!(r#"{{"piece_id":{},"weight":{}}}"#, crate::uuid::uuid_arg(&d.piece_id), d.weight))
            .collect();
        let args = vec![
            crate::uuid::uuid_arg(&lootbox.id),
            serde_json::json!(lootbox.name).to_string(),
            format!("[{}]", drops.join(",")),
        ];
        call_reducer(session, "import_lootbox", &args)?;
        println!("imported lootbox '{}'", lootbox.name);
    }
    Ok(())
}
