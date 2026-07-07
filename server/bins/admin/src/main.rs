//! RocknRolla content administration shell.
//!
//! An interactive shell that validates committed components, level
//! sources, and seed content, then imports them
//! through the owner-only reducers using the locally
//! installed, already authenticated `spacetime` CLI session. This binary
//! never reads or prints the `.env` token; run `task server:login` first.

use anyhow::{Context, Result, bail};
use std::{
    io::{BufRead, Write},
    path::PathBuf,
    process::Command as Process,
};

mod command;
mod componentsrc;
mod levelsrc;
mod seed;
mod svggen;
mod uuid;

use command::Command;

const DEFAULT_DATABASE: &str = "rocknrolladb-dev";
const PRODUCTION_DATABASE: &str = "rocknrolladb";
const DEFAULT_SERVER: &str = "maincloud";
const DEFAULT_LEVELS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../levels/src");
const DEFAULT_COMPONENTS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../levels/components");
const DEFAULT_SEED_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../levels/seed.json");

/// Target and content-path state kept alive across shell commands.
struct Session {
    server: String,
    database: String,
    levels_path: PathBuf,
    components_path: PathBuf,
    seed_path: PathBuf,
}

impl Default for Session {
    fn default() -> Self {
        Session {
            server: DEFAULT_SERVER.to_string(),
            database: DEFAULT_DATABASE.to_string(),
            levels_path: default_path(DEFAULT_LEVELS_PATH),
            components_path: default_path(DEFAULT_COMPONENTS_PATH),
            seed_path: default_path(DEFAULT_SEED_PATH),
        }
    }
}

fn default_path(path: &str) -> PathBuf {
    PathBuf::from(path).canonicalize().unwrap_or_else(|_| PathBuf::from(path))
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
            },
        };
        match command::parse_line(&line) {
            Ok(None) => {},
            Ok(Some(Command::Quit)) => break,
            Ok(Some(parsed)) => {
                if let Err(error) = execute(&mut session, parsed, &mut input) {
                    eprintln!("error: {error:?}");
                }
            },
            Err(error) => eprintln!("error: {error} (type 'help' for usage)"),
        }
    }
}

fn execute(session: &mut Session, parsed: Command, input: &mut impl Iterator<Item = std::io::Result<String>>) -> Result<()> {
    match parsed {
        Command::Quit => {
            bail!("quit is handled by the caller loop and should not reach execute");
        },
        Command::Help => println!("{}", command::USAGE),
        Command::Status => print_status(session),
        Command::SetServer(server) => {
            session.server = server;
            println!("server set to '{}'", session.server);
        },
        Command::SetDatabase(database) => {
            session.database = database;
            println!("database set to '{}'", session.database);
        },
        Command::ValidateLevels(path) => {
            let components = load_components(session, None)?;
            let levels = load_levels(session, path.as_deref(), &components)?;
            println!("{} level(s) valid; nothing imported", levels.len());
        },
        Command::ValidateComponents(path) => {
            let components = load_components(session, path.as_deref())?;
            println!("{} component(s) valid; nothing imported", components.len());
        },
        Command::ValidateSeed(path) => {
            load_seed(session, path.as_deref())?;
            println!("seed content valid; nothing imported");
        },
        Command::ValidateAll => {
            let content = load_seed(session, None)?;
            let components = load_components(session, None)?;
            let levels = load_levels(session, None, &components)?;
            println!(
                "seed content, {} component(s), and {} level(s) valid; nothing imported ({} characters, {} pieces, {} lootboxes)",
                components.len(),
                levels.len(),
                content.characters.len(),
                content.pieces.len(),
                content.lootboxes.len()
            );
        },
        Command::ImportLevels(path) => {
            let components = load_components(session, None)?;
            let levels = load_levels(session, path.as_deref(), &components)?;
            if confirm_import(session, input)? {
                import_levels(session, &levels)?;
            }
        },
        Command::ImportComponents(path) => {
            let components = load_components(session, path.as_deref())?;
            if confirm_import(session, input)? {
                import_components(session, &components)?;
            }
        },
        Command::ImportSeed(path) => {
            let content = load_seed(session, path.as_deref())?;
            if confirm_import(session, input)? {
                import_seed(session, &content)?;
            }
        },
        Command::ExportComponents(dir) => {
            let target = PathBuf::from(&dir);
            std::fs::create_dir_all(&target).with_context(|| format!("cannot create {}", target.display()))?;
            for component in svggen::starter_library() {
                let file = target.join(format!("{}.svg", component.slug));
                std::fs::write(&file, &component.data).with_context(|| format!("cannot write {}", file.display()))?;
                println!("wrote {}", file.display());
            }
        },
        Command::ImportAll => {
            let content = load_seed(session, None)?;
            let components = load_components(session, None)?;
            let levels = load_levels(session, None, &components)?;
            if confirm_import(session, input)? {
                import_seed(session, &content)?;
                import_components(session, &components)?;
                import_levels(session, &levels)?;
            }
        },
    }
    Ok(())
}

fn print_status(session: &Session) {
    println!("server:     {}", session.server);
    println!("database:   {}", session.database);
    println!("levels:     {}", session.levels_path.display());
    println!("components: {}", session.components_path.display());
    println!("seed:       {}", session.seed_path.display());
}

/// Show the destination and require explicit confirmation. Importing into the
/// production database demands an additional unmistakable confirmation.
fn confirm_import(session: &Session, input: &mut impl Iterator<Item = std::io::Result<String>>) -> Result<bool> {
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

fn prompt_matches(input: &mut impl Iterator<Item = std::io::Result<String>>, prompt: &str, expected: &str) -> Result<bool> {
    print!("{prompt}");
    let _ = std::io::stdout().flush();
    match input.next() {
        Some(Ok(answer)) => Ok(answer.trim() == expected),
        Some(Err(error)) => Err(error).context("cannot read confirmation"),
        None => Ok(false),
    }
}

fn load_seed(session: &Session, path_override: Option<&str>) -> Result<seed::SeedContent> {
    let path = path_override.map(PathBuf::from).unwrap_or_else(|| session.seed_path.clone());
    let source = std::fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
    let content = seed::parse_seed(&source).with_context(|| path.display().to_string())?;
    println!(
        "validated {}: {} characters, {} pieces, {} lootboxes",
        path.display(),
        content.characters.len(),
        content.pieces.len(),
        content.lootboxes.len()
    );
    Ok(content)
}

fn load_components(session: &Session, path_override: Option<&str>) -> Result<Vec<rocknrolla_level::ComponentFacts>> {
    let path = path_override
        .map(PathBuf::from)
        .unwrap_or_else(|| session.components_path.clone());
    let components = componentsrc::load_components(&path)?;
    for component in &components {
        println!(
            "validated component '{}' ({}x{}, {} SVG bytes, hash {})",
            component.slug,
            component.width_px,
            component.height_px,
            component.data.len(),
            component.content_hash,
        );
    }
    Ok(components)
}

fn load_levels(
    session: &Session,
    path_override: Option<&str>,
    components: &[rocknrolla_level::ComponentFacts],
) -> Result<Vec<levelsrc::ImportedLevel>> {
    let path = path_override
        .map(PathBuf::from)
        .unwrap_or_else(|| session.levels_path.clone());
    let files = collect_json_files(&path)?;
    let mut levels = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file).with_context(|| format!("cannot read {}", file.display()))?;
        let level = levelsrc::parse_level(&source, components).with_context(|| file.display().to_string())?;
        println!(
            "validated {}: level '{}' ({} placements)",
            file.display(),
            level.slug,
            level.placements.len(),
        );
        levels.push(level);
    }
    reject_duplicate_levels(&levels)?;
    Ok(levels)
}

fn reject_duplicate_levels(levels: &[levelsrc::ImportedLevel]) -> Result<()> {
    let mut ids = std::collections::HashSet::new();
    let mut slugs = std::collections::HashSet::new();
    for level in levels {
        if !ids.insert(level.id.to_lowercase()) {
            bail!("duplicate level id '{}'", level.id);
        }
        if !slugs.insert(level.slug.clone()) {
            bail!("duplicate level slug '{}'", level.slug);
        }
    }
    Ok(())
}

fn collect_json_files(path: &PathBuf) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.clone()]);
    }
    if !path.is_dir() {
        bail!("path not found: {}", path.display());
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(path)
        .with_context(|| format!("cannot read directory {}", path.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    if files.is_empty() {
        bail!("no JSON files found in {}", path.display());
    }
    Ok(files)
}

fn call_reducer(session: &Session, reducer: &str, args: &[String]) -> Result<()> {
    let output = Process::new("spacetime")
        .arg("call")
        .arg("--server")
        .arg(&session.server)
        .arg("--yes")
        .arg(&session.database)
        .arg(reducer)
        .args(args)
        .output()
        .context("failed to run the spacetime CLI")?;
    if output.status.success() {
        Ok(())
    } else {
        bail!(
            "spacetime call {reducer} failed: {}{}",
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
}

fn placement_to_json(placement: &levelsrc::ImportedPlacement) -> serde_json::Value {
    serde_json::json!({
        "component_slug": placement.component_slug,
        "position": {
            "x": placement.position.x,
            "y": placement.position.y,
            "z": placement.position.z,
        },
        "flip_x": placement.flip_x,
        "scale": placement.scale,
    })
}

fn point_json(point: rocknrolla_level::Vec2) -> String {
    serde_json::json!({ "x": point.x, "y": point.y }).to_string()
}

fn import_components(session: &Session, components: &[rocknrolla_level::ComponentFacts]) -> Result<()> {
    for component in components {
        let arg = serde_json::json!({
            "slug": component.slug,
            "width_px": component.width_px,
            "height_px": component.height_px,
            "content_hash": component.content_hash,
            "data": component.data,
        })
        .to_string();
        call_reducer(session, "import_component", &[arg])?;
        println!("imported component '{}' into {}", component.slug, session.database);
    }
    Ok(())
}

fn import_levels(session: &Session, levels: &[levelsrc::ImportedLevel]) -> Result<()> {
    for level in levels {
        let args = vec![
            crate::uuid::uuid_arg(&level.id)?,
            serde_json::json!(level.slug).to_string(),
            serde_json::json!(level.name).to_string(),
            level.is_starting.to_string(),
            level.active.to_string(),
            crate::uuid::uuid_opt_arg(level.reward_lootbox_id.as_deref())?,
            crate::uuid::uuid_vec_arg(&level.successors)?,
            point_json(level.spawn),
            point_json(level.finish),
            serde_json::Value::Array(level.placements.iter().map(placement_to_json).collect()).to_string(),
        ];
        call_reducer(session, "import_level", &args)?;
        println!("imported level '{}' into {}", level.slug, session.database);
    }
    Ok(())
}

fn import_seed(session: &Session, content: &seed::SeedContent) -> Result<()> {
    for character in &content.characters {
        let args = vec![
            crate::uuid::uuid_arg(&character.id)?,
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
            crate::uuid::uuid_arg(&piece.id)?,
            serde_json::json!(piece.name).to_string(),
            crate::uuid::uuid_arg(&piece.character_id)?,
        ];
        call_reducer(session, "import_piece", &args)?;
        println!("imported piece '{}'", piece.name);
    }
    for lootbox in &content.lootboxes {
        let drops = lootbox
            .drops
            .iter()
            .map(|d| -> Result<String> {
                Ok(format!(
                    r#"{{"piece_id":{},"weight":{}}}"#,
                    crate::uuid::uuid_arg(&d.piece_id)?,
                    d.weight
                ))
            })
            .collect::<Result<Vec<String>>>()?;
        let args = vec![
            crate::uuid::uuid_arg(&lootbox.id)?,
            serde_json::json!(lootbox.name).to_string(),
            format!("[{}]", drops.join(",")),
        ];
        call_reducer(session, "import_lootbox", &args)?;
        println!("imported lootbox '{}'", lootbox.name);
    }
    Ok(())
}
