//! Tile grid → "Claymation Dusk" scene SVG rendering.
//!
//! Each layer becomes one standalone SVG document: merged terrain art
//! (continuous dirt with a single moss surface, smooth slope wedges, water,
//! hazards, props) plus — on the gameplay layer — a hidden group of
//! collider markers (`data-t="<tile id>"`) the client turns into Matter
//! bodies. Heavy blocks are marker-only: the client draws them as dynamic
//! sprites.

use rocknrolla_level::{ENCODING_SVG_V1, GAMEPLAY_Z, LayerFacts, content_hash, tile};

pub struct LayerScene {
    pub z: u8,
    pub parallax_x: f32,
    pub parallax_y: f32,
    pub cell: u32,
    pub cols: u32,
    pub rows: u32,
    pub tiles: Vec<u8>,
}

const DIRT: &str = "#b06038";
const DIRT_DARK: &str = "#9a4f2e";
const MOSS: &str = "#6c7a3e";
const MOSS_LIGHT: &str = "#9aa763";
const MOSS_DARK: &str = "#4c5528";

const DEFS: &str = concat!(
    "<defs>",
    "<linearGradient id=\"w\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\">",
    "<stop offset=\"0\" stop-color=\"#5aa6b4\"/><stop offset=\"1\" stop-color=\"#2f6472\"/>",
    "</linearGradient>",
    "<linearGradient id=\"s\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\">",
    "<stop offset=\"0\" stop-color=\"#a8503a\"/><stop offset=\"1\" stop-color=\"#6a2c1e\"/>",
    "</linearGradient>",
    "<radialGradient id=\"f\" cx=\"50%\" cy=\"70%\">",
    "<stop offset=\"0\" stop-color=\"#ffe08a\"/><stop offset=\".5\" stop-color=\"#f2882c\"/>",
    "<stop offset=\"1\" stop-color=\"#d13a1f\"/>",
    "</radialGradient>",
    "</defs>"
);

/// Per-cell art in a local 100×100 space, wrapped by a translate+scale group.
const ART_FIRE: &str = concat!(
    "<ellipse cx=\"50\" cy=\"93\" rx=\"40\" ry=\"8\" fill=\"#d13a1f\" opacity=\".6\"/>",
    "<g transform=\"translate(17,8) scale(1.1)\">",
    "<path d=\"M30 4 C10 30 42 36 30 60 C30 48 18 46 22 30 C16 42 8 52 16 66 ",
    "C22 78 40 78 46 66 C54 50 40 44 44 24 C40 34 36 24 30 4 Z\" fill=\"url(#f)\"/>",
    "<path d=\"M30 34 C22 46 34 50 30 62 C36 54 40 46 34 36 C33 44 32 40 30 34 Z\" fill=\"#ffe08a\"/>",
    "</g>"
);

const ART_LETHAL: &str = concat!(
    "<polygon points=\"2,100 18,38 34,100\" fill=\"url(#s)\"/>",
    "<polygon points=\"34,100 50,28 66,100\" fill=\"url(#s)\"/>",
    "<polygon points=\"66,100 82,38 98,100\" fill=\"url(#s)\"/>",
    "<rect y=\"94\" width=\"100\" height=\"6\" fill=\"#6a2c1e\"/>"
);

const ART_FINISH: &str = concat!(
    "<rect x=\"22\" y=\"6\" width=\"7\" height=\"94\" rx=\"3.5\" fill=\"#241d16\"/>",
    "<rect x=\"29\" y=\"10\" width=\"60\" height=\"36\" fill=\"#f5ecd8\"/>",
    "<rect x=\"29\" y=\"10\" width=\"12\" height=\"12\" fill=\"#241d16\"/>",
    "<rect x=\"53\" y=\"10\" width=\"12\" height=\"12\" fill=\"#241d16\"/>",
    "<rect x=\"77\" y=\"10\" width=\"12\" height=\"12\" fill=\"#241d16\"/>",
    "<rect x=\"41\" y=\"22\" width=\"12\" height=\"12\" fill=\"#241d16\"/>",
    "<rect x=\"65\" y=\"22\" width=\"12\" height=\"12\" fill=\"#241d16\"/>",
    "<rect x=\"29\" y=\"34\" width=\"12\" height=\"12\" fill=\"#241d16\"/>",
    "<rect x=\"53\" y=\"34\" width=\"12\" height=\"12\" fill=\"#241d16\"/>",
    "<rect x=\"77\" y=\"34\" width=\"12\" height=\"12\" fill=\"#241d16\"/>",
    "<circle cx=\"25.5\" cy=\"8\" r=\"5\" fill=\"#ffce5c\"/>"
);

const ART_SPAWN: &str = concat!(
    "<rect x=\"46\" y=\"38\" width=\"8\" height=\"62\" fill=\"#6a3c22\"/>",
    "<rect x=\"16\" y=\"14\" width=\"68\" height=\"36\" rx=\"9\" fill=\"#8a5a34\" ",
    "stroke=\"#6a3c22\" stroke-width=\"4\"/>",
    "<polygon points=\"40,24 40,40 62,32\" fill=\"#f5ecd8\"/>"
);

const ART_DECOR: &str = concat!(
    "<ellipse cx=\"50\" cy=\"88\" rx=\"38\" ry=\"11\" fill=\"#4c5528\"/>",
    "<circle cx=\"32\" cy=\"70\" r=\"19\" fill=\"#6c7a3e\"/>",
    "<circle cx=\"60\" cy=\"64\" r=\"23\" fill=\"#6c7a3e\"/>",
    "<circle cx=\"76\" cy=\"78\" r=\"15\" fill=\"#6c7a3e\"/>",
    "<circle cx=\"27\" cy=\"62\" r=\"7\" fill=\"#9aa763\" opacity=\".85\"/>",
    "<circle cx=\"54\" cy=\"52\" r=\"9\" fill=\"#9aa763\" opacity=\".85\"/>",
    "<circle cx=\"72\" cy=\"70\" r=\"5\" fill=\"#9aa763\" opacity=\".7\"/>"
);

/// Format a pixel coordinate, trimming float noise.
fn n(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if rounded.fract() == 0.0 {
        format!("{}", rounded as i64)
    } else {
        format!("{rounded}")
    }
}

struct Grid<'a> {
    tiles: &'a [u8],
    cols: u32,
    rows: u32,
}

impl Grid<'_> {
    fn at(&self, x: u32, y: u32) -> u8 {
        self.tiles[(y * self.cols + x) as usize]
    }

    /// Terrain (solid or slope) occupies the cell above, so no moss cap.
    fn terrain_above(&self, x: u32, y: u32) -> bool {
        y > 0 && matches!(self.at(x, y - 1), tile::SOLID | tile::SLOPE_UP | tile::SLOPE_DOWN)
    }

    /// Horizontal runs of cells matching a predicate, per row.
    fn runs(&self, matches: impl Fn(u32, u32) -> bool) -> Vec<(u32, u32, u32)> {
        let mut out = Vec::new();
        for y in 0..self.rows {
            let mut start: Option<u32> = None;
            for x in 0..=self.cols {
                let hit = x < self.cols && matches(x, y);
                match (hit, start) {
                    (true, None) => start = Some(x),
                    (false, Some(s)) => {
                        out.push((s, y, x - s));
                        start = None;
                    },
                    _ => {},
                }
            }
        }
        out
    }
}

/// Render one layer's grid into `svg-v1` [`LayerFacts`].
pub fn render_layer(scene: &LayerScene) -> LayerFacts {
    let c = scene.cell as f64;
    let grid = Grid {
        tiles: &scene.tiles,
        cols: scene.cols,
        rows: scene.rows,
    };
    let width_px = scene.cols * scene.cell;
    let height_px = scene.rows * scene.cell;

    let mut svg = String::with_capacity(16 * 1024);
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width_px}\" height=\"{height_px}\" \
         viewBox=\"0 0 {width_px} {height_px}\">"
    ));
    svg.push_str(DEFS);

    paint_terrain(&mut svg, &grid, c);
    paint_water(&mut svg, &grid, c);
    paint_cell_art(&mut svg, &grid, c);

    if scene.z == GAMEPLAY_Z {
        svg.push_str("<g visibility=\"hidden\">");
        push_markers(&mut svg, &grid, c);
        svg.push_str("</g>");
    }
    svg.push_str("</svg>");

    let data = svg.into_bytes();
    LayerFacts {
        z: scene.z,
        width_px,
        height_px,
        parallax_x: scene.parallax_x,
        parallax_y: scene.parallax_y,
        encoding: ENCODING_SVG_V1.to_string(),
        content_hash: content_hash(width_px, height_px, &data),
        data,
    }
}

/// Merged dirt mass, one moss surface along the sky line, slope wedges.
fn paint_terrain(svg: &mut String, grid: &Grid, c: f64) {
    for (x, y, len) in grid.runs(|x, y| grid.at(x, y) == tile::SOLID) {
        svg.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{DIRT}\"/>",
            n(x as f64 * c),
            n(y as f64 * c),
            n(len as f64 * c),
            n(c)
        ));
    }
    // Sparse deterministic speckles inside the dirt.
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            if grid.at(x, y) != tile::SOLID {
                continue;
            }
            let seed = (x * 31 + y * 17) % 3;
            let (fx, fy, r) = match seed {
                0 => (0.28, 0.58, 0.05),
                1 => (0.66, 0.74, 0.06),
                _ => continue,
            };
            svg.push_str(&format!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{DIRT_DARK}\" opacity=\".5\"/>",
                n((x as f64 + fx) * c),
                n((y as f64 + fy) * c),
                n(r * c)
            ));
        }
    }
    // Moss caps where solid ground meets the sky, merged into runs.
    for (x, y, len) in grid.runs(|x, y| grid.at(x, y) == tile::SOLID && !grid.terrain_above(x, y)) {
        let px = x as f64 * c;
        let py = y as f64 * c;
        let w = len as f64 * c;
        svg.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{MOSS}\"/>",
            n(px),
            n(py),
            n(w),
            n(0.30 * c)
        ));
        svg.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{MOSS_LIGHT}\"/>",
            n(px),
            n(py),
            n(w),
            n(0.11 * c)
        ));
        for i in 0..len {
            let cx = px + i as f64 * c;
            for (fx, r) in [(0.16, 0.07), (0.42, 0.09), (0.68, 0.07), (0.88, 0.08)] {
                svg.push_str(&format!(
                    "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{MOSS}\"/>",
                    n(cx + fx * c),
                    n(py + 0.30 * c),
                    n(r * c)
                ));
            }
        }
        svg.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{MOSS_DARK}\" opacity=\".25\"/>",
            n(px),
            n(py + 0.26 * c),
            n(w),
            n(0.04 * c)
        ));
    }
    // Slope wedges with a moss band along the hypotenuse.
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            let t = grid.at(x, y);
            if t != tile::SLOPE_UP && t != tile::SLOPE_DOWN {
                continue;
            }
            let x0 = x as f64 * c;
            let y0 = y as f64 * c;
            let x1 = x0 + c;
            let y1 = y0 + c;
            let (dirt, moss, light) = if t == tile::SLOPE_UP {
                (
                    format!("{},{} {},{} {},{}", n(x0), n(y1), n(x1), n(y1), n(x1), n(y0)),
                    format!(
                        "{},{} {},{} {},{} {},{}",
                        n(x0),
                        n(y1),
                        n(x1),
                        n(y0),
                        n(x1),
                        n(y0 + 0.26 * c),
                        n(x0 + 0.20 * c),
                        n(y1)
                    ),
                    format!(
                        "{},{} {},{} {},{} {},{}",
                        n(x0),
                        n(y1),
                        n(x1),
                        n(y0),
                        n(x1),
                        n(y0 + 0.10 * c),
                        n(x0 + 0.08 * c),
                        n(y1)
                    ),
                )
            } else {
                (
                    format!("{},{} {},{} {},{}", n(x0), n(y0), n(x1), n(y1), n(x0), n(y1)),
                    format!(
                        "{},{} {},{} {},{} {},{}",
                        n(x0),
                        n(y0),
                        n(x1),
                        n(y1),
                        n(x0 + 0.80 * c),
                        n(y1),
                        n(x0),
                        n(y0 + 0.26 * c)
                    ),
                    format!(
                        "{},{} {},{} {},{} {},{}",
                        n(x0),
                        n(y0),
                        n(x1),
                        n(y1),
                        n(x0 + 0.92 * c),
                        n(y1),
                        n(x0),
                        n(y0 + 0.10 * c)
                    ),
                )
            };
            svg.push_str(&format!("<polygon points=\"{dirt}\" fill=\"{DIRT}\"/>"));
            svg.push_str(&format!("<polygon points=\"{moss}\" fill=\"{MOSS}\"/>"));
            svg.push_str(&format!("<polygon points=\"{light}\" fill=\"{MOSS_LIGHT}\"/>"));
        }
    }
}

fn paint_water(svg: &mut String, grid: &Grid, c: f64) {
    for (x, y, len) in grid.runs(|x, y| grid.at(x, y) == tile::WATER) {
        let px = x as f64 * c;
        let py = y as f64 * c;
        let w = len as f64 * c;
        svg.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"url(#w)\" opacity=\".92\"/>",
            n(px),
            n(py),
            n(w),
            n(c)
        ));
        // Crest only where the water surface meets the sky.
        if y == 0 || grid.at(x, y - 1) != tile::WATER {
            let step = 0.125 * c;
            let cy = py + 0.09 * c;
            let mut cursor = px + step;
            let mut path = format!(
                "M{} {} Q{} {} {} {}",
                n(px),
                n(cy),
                n(px + step / 2.0),
                n(py + 0.03 * c),
                n(cursor),
                n(cy)
            );
            let segments = (w / step).round() as u32;
            for _ in 1..segments {
                cursor += step;
                path.push_str(&format!(" T{} {}", n(cursor), n(cy)));
            }
            svg.push_str(&format!(
                "<path d=\"{path}\" fill=\"none\" stroke=\"#cfeef2\" stroke-width=\"{}\" opacity=\".75\"/>",
                n(0.04 * c)
            ));
        }
        for i in 0..len {
            let (fx, fy) = if (x + i + y) % 2 == 0 { (0.30, 0.46) } else { (0.68, 0.72) };
            svg.push_str(&format!(
                "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"#cfeef2\" opacity=\".2\"/>",
                n(px + (i as f64 + fx) * c),
                n(py + fy * c),
                n(0.10 * c),
                n(0.03 * c)
            ));
        }
    }
}

/// Per-cell props drawn in their local 100×100 art space.
fn paint_cell_art(svg: &mut String, grid: &Grid, c: f64) {
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            let art = match grid.at(x, y) {
                tile::FIRE => ART_FIRE,
                tile::LETHAL => ART_LETHAL,
                tile::FINISH => ART_FINISH,
                tile::SPAWN => ART_SPAWN,
                tile::DECOR => ART_DECOR,
                _ => continue,
            };
            svg.push_str(&format!(
                "<g transform=\"translate({},{}) scale({})\">{art}</g>",
                n(x as f64 * c),
                n(y as f64 * c),
                n(c / 100.0)
            ));
        }
    }
}

/// Hidden collider markers: merged solid runs, slope triangles, and
/// per-cell sensor/marker rects, all tagged with semantic tile ids.
fn push_markers(svg: &mut String, grid: &Grid, c: f64) {
    for (x, y, len) in grid.runs(|x, y| grid.at(x, y) == tile::SOLID) {
        svg.push_str(&format!(
            "<rect data-t=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>",
            tile::SOLID,
            n(x as f64 * c),
            n(y as f64 * c),
            n(len as f64 * c),
            n(c)
        ));
    }
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            let t = grid.at(x, y);
            let x0 = x as f64 * c;
            let y0 = y as f64 * c;
            let x1 = x0 + c;
            let y1 = y0 + c;
            match t {
                tile::SLOPE_UP => svg.push_str(&format!(
                    "<polygon data-t=\"{t}\" points=\"{},{} {},{} {},{}\"/>",
                    n(x0),
                    n(y1),
                    n(x1),
                    n(y1),
                    n(x1),
                    n(y0)
                )),
                tile::SLOPE_DOWN => svg.push_str(&format!(
                    "<polygon data-t=\"{t}\" points=\"{},{} {},{} {},{}\"/>",
                    n(x0),
                    n(y0),
                    n(x1),
                    n(y1),
                    n(x0),
                    n(y1)
                )),
                tile::SPAWN | tile::FINISH | tile::LETHAL | tile::WATER | tile::FIRE | tile::HEAVY => {
                    svg.push_str(&format!(
                        "<rect data-t=\"{t}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>",
                        n(x0),
                        n(y0),
                        n(c),
                        n(c)
                    ));
                },
                _ => {},
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene(rows: &[&str]) -> LayerScene {
        let cols = rows[0].len() as u32;
        let tiles: Vec<u8> = rows
            .iter()
            .flat_map(|row| {
                row.chars().map(|ch| match ch {
                    '#' => tile::SOLID,
                    '/' => tile::SLOPE_UP,
                    'S' => tile::SPAWN,
                    'F' => tile::FINISH,
                    '~' => tile::WATER,
                    'H' => tile::HEAVY,
                    _ => tile::EMPTY,
                })
            })
            .collect();
        LayerScene {
            z: GAMEPLAY_Z,
            parallax_x: 1.0,
            parallax_y: 1.0,
            cell: 64,
            cols,
            rows: rows.len() as u32,
            tiles,
        }
    }

    #[test]
    fn renders_markers_and_merged_terrain() {
        let facts = render_layer(&scene(&["S..F", "##~H"]));
        assert_eq!(facts.width_px, 256);
        assert_eq!(facts.height_px, 128);
        let svg = String::from_utf8(facts.data).unwrap();
        // Two adjacent solid cells merge into one 128-wide marker rect.
        assert!(svg.contains("data-t=\"1\" x=\"0\" y=\"64\" width=\"128\""), "{svg}");
        assert!(svg.contains("data-t=\"4\""));
        assert!(svg.contains("data-t=\"5\""));
        assert!(svg.contains("data-t=\"7\""));
        assert!(svg.contains("data-t=\"9\""));
        // Heavy blocks are marker-only: no visible art beyond the marker.
        assert_eq!(svg.matches("data-t=\"9\"").count(), 1);
    }

    #[test]
    fn hash_matches_and_validates() {
        let facts = render_layer(&scene(&["S/F", "###"]));
        assert_eq!(facts.content_hash, content_hash(facts.width_px, facts.height_px, &facts.data));
        rocknrolla_level::validate_layers(&[facts]).unwrap();
    }
}
