//! RocknRolla level and seed-content administration CLI.
//!
//! Validates committed Tiled JSON and seed content, then imports them through
//! the owner-only reducers using the locally installed, already authenticated
//! `spacetime` CLI session. This binary never reads or prints the `.env`
//! token; run `task server:login` first.

use std::path::PathBuf;
use std::process::Command;

mod seed;
mod tiled;

const DEFAULT_DATABASE: &str = "rocknrolladb-dev";
const DEFAULT_SERVER: &str = "maincloud";

const USAGE: &str = "\
rocknrolla-admin - RocknRolla content importer

USAGE:
  rocknrolla-admin levels [--dry-run] [--database NAME] [--server NAME] <file-or-dir>...
  rocknrolla-admin seed   [--dry-run] [--database NAME] [--server NAME] <seed.json>

Levels are Tiled JSON exports; directories are scanned for *.json.
--dry-run performs every validation without touching SpacetimeDB.
--database defaults to rocknrolladb-dev (never the production database).
--server defaults to maincloud.";

struct Options {
    dry_run: bool,
    database: String,
    server: String,
    paths: Vec<PathBuf>,
}

fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        dry_run: false,
        database: DEFAULT_DATABASE.to_string(),
        server: DEFAULT_SERVER.to_string(),
        paths: Vec::new(),
    };
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--dry-run" => options.dry_run = true,
            "--database" => {
                options.database = iter.next().ok_or("--database needs a value")?.clone();
            }
            "--server" => {
                options.server = iter.next().ok_or("--server needs a value")?.clone();
            }
            other if other.starts_with("--") => return Err(format!("unknown flag '{other}'")),
            path => options.paths.push(PathBuf::from(path)),
        }
    }
    if options.paths.is_empty() {
        return Err("no input paths given".to_string());
    }
    Ok(options)
}

fn collect_json_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            let mut entries: Vec<PathBuf> = std::fs::read_dir(path)
                .map_err(|e| format!("cannot read directory {}: {e}", path.display()))?
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
                .collect();
            entries.sort();
            files.extend(entries);
        } else if path.is_file() {
            files.push(path.clone());
        } else {
            return Err(format!("path not found: {}", path.display()));
        }
    }
    if files.is_empty() {
        return Err("no JSON files found in the given paths".to_string());
    }
    Ok(files)
}

fn call_reducer(options: &Options, reducer: &str, args: &[String]) -> Result<(), String> {
    let mut command = Command::new("spacetime");
    command
        .arg("call")
        .arg("--server")
        .arg(&options.server)
        .arg("--yes")
        .arg(&options.database)
        .arg(reducer)
        .args(args);
    let output = command
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

fn run_levels(options: &Options) -> Result<(), String> {
    let files = collect_json_files(&options.paths)?;
    let mut levels = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file)
            .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
        let level = tiled::parse_level(&source).map_err(|e| format!("{}: {e}", file.display()))?;
        println!(
            "validated {}: level '{}' ({} layers, {} compressed bytes, gameplay hash {})",
            file.display(),
            level.id,
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
    if options.dry_run {
        println!("dry run: {} level(s) valid; nothing imported", levels.len());
        return Ok(());
    }
    for level in &levels {
        let args = vec![
            serde_json::Value::String(level.id.clone()).to_string(),
            serde_json::Value::String(level.name.clone()).to_string(),
            level.is_starting.to_string(),
            level.active.to_string(),
            serde_json::Value::String(level.reward_lootbox_id.clone()).to_string(),
            serde_json::json!(level.successors).to_string(),
            serde_json::Value::Array(level.layers.iter().map(layer_to_json).collect()).to_string(),
        ];
        call_reducer(options, "import_level", &args)?;
        println!("imported level '{}' into {}", level.id, options.database);
    }
    Ok(())
}

fn run_seed(options: &Options) -> Result<(), String> {
    let [path] = options.paths.as_slice() else {
        return Err("seed takes exactly one seed.json file".to_string());
    };
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let content = seed::parse_seed(&source).map_err(|e| format!("{}: {e}", path.display()))?;
    println!(
        "validated {}: {} characters, {} pieces, {} lootboxes",
        path.display(),
        content.characters.len(),
        content.pieces.len(),
        content.lootboxes.len()
    );
    if options.dry_run {
        println!("dry run: seed content valid; nothing imported");
        return Ok(());
    }
    for character in &content.characters {
        let args = vec![
            serde_json::Value::String(character.id.clone()).to_string(),
            serde_json::Value::String(character.name.clone()).to_string(),
            serde_json::Value::String(character.style.clone()).to_string(),
            character.rarity_weight.to_string(),
            character.density.to_string(),
            character.jump_speed.to_string(),
            character.flight_time_ms.to_string(),
            character.buoyancy.to_string(),
            character.fire_resistance.to_string(),
            character.starter.to_string(),
        ];
        call_reducer(options, "import_character", &args)?;
        println!("imported character '{}'", character.id);
    }
    for piece in &content.pieces {
        let args = vec![
            serde_json::Value::String(piece.id.clone()).to_string(),
            serde_json::Value::String(piece.name.clone()).to_string(),
            serde_json::Value::String(piece.character_id.clone()).to_string(),
        ];
        call_reducer(options, "import_piece", &args)?;
        println!("imported piece '{}'", piece.id);
    }
    for lootbox in &content.lootboxes {
        let drops: Vec<serde_json::Value> = lootbox
            .drops
            .iter()
            .map(|d| serde_json::json!({ "piece_id": d.piece_id, "weight": d.weight }))
            .collect();
        let args = vec![
            serde_json::Value::String(lootbox.id.clone()).to_string(),
            serde_json::Value::String(lootbox.name.clone()).to_string(),
            serde_json::Value::Array(drops).to_string(),
        ];
        call_reducer(options, "import_lootbox", &args)?;
        println!("imported lootbox '{}'", lootbox.id);
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.split_first() {
        Some((command, rest)) if command == "levels" => {
            parse_options(rest).and_then(|options| run_levels(&options))
        }
        Some((command, rest)) if command == "seed" => {
            parse_options(rest).and_then(|options| run_seed(&options))
        }
        _ => {
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    };
    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
