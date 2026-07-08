//! Tile grid → "Claymation Dusk" component SVG rendering.
//!
//! Each component is authored here as a small ASCII grid and rendered into
//! one standalone SVG document: merged terrain art (continuous dirt with a
//! single moss surface, smooth slope wedges, water, hazards, props) plus a
//! hidden group of collider markers (`data-t="<tile id>"`) the client turns
//! into Matter bodies. Heavy blocks are marker-only: the client draws them
//! as dynamic sprites. `export components <dir>` dumps the starter library;
//! the committed files in `content/components/` are the source of truth.

use anyhow::{Context, Result, bail};
use rocknrolla_level::{ComponentFacts, GAMEPLAY_CELL_SIZE, content_hash, tile};

pub struct ComponentScene {
    pub cell: u32,
    pub cols: u32,
    pub rows: u32,
    pub tiles: Vec<u8>,
    /// Cap sky-facing solid runs with moss. Off for underground fills.
    pub moss_top: bool,
}

const DIRT: &str = "#8a5443";
const DIRT_DARK: &str = "#6b4034";
const MOSS: &str = "#5f6c40";
const MOSS_LIGHT: &str = "#7d8c53";
const MOSS_DARK: &str = "#454f2c";

const DEFS: &str = concat!(
    "<defs>",
    "<linearGradient id=\"w\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\">",
    "<stop offset=\"0\" stop-color=\"#4f96a4\"/><stop offset=\"1\" stop-color=\"#2a5661\"/>",
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

const ART_DECOR: &str = concat!(
    "<ellipse cx=\"50\" cy=\"88\" rx=\"38\" ry=\"11\" fill=\"#454f2c\"/>",
    "<circle cx=\"32\" cy=\"70\" r=\"19\" fill=\"#5f6c40\"/>",
    "<circle cx=\"60\" cy=\"64\" r=\"23\" fill=\"#5f6c40\"/>",
    "<circle cx=\"76\" cy=\"78\" r=\"15\" fill=\"#5f6c40\"/>",
    "<circle cx=\"27\" cy=\"62\" r=\"7\" fill=\"#7d8c53\" opacity=\".85\"/>",
    "<circle cx=\"54\" cy=\"52\" r=\"9\" fill=\"#7d8c53\" opacity=\".85\"/>",
    "<circle cx=\"72\" cy=\"70\" r=\"5\" fill=\"#7d8c53\" opacity=\".7\"/>"
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

/// Map one authoring character onto a semantic tile id.
fn tile_for(ch: char) -> Result<u8> {
    Ok(match ch {
        '.' => tile::EMPTY,
        '#' => tile::SOLID,
        '/' => tile::SLOPE_UP,
        '\\' => tile::SLOPE_DOWN,
        '^' => tile::LETHAL,
        '~' => tile::WATER,
        'f' => tile::FIRE,
        'H' => tile::HEAVY,
        'd' => tile::DECOR,
        other => bail!("unknown tile character '{other}'"),
    })
}

/// Parse ASCII rows into a [`ComponentScene`].
pub fn parse_scene(cell: u32, moss_top: bool, rows: &[&str]) -> Result<ComponentScene> {
    if rows.is_empty() {
        bail!("component has no rows");
    }
    let cols = rows[0].chars().count() as u32;
    if cols == 0 {
        bail!("component has empty rows");
    }
    let mut tiles = Vec::with_capacity((cols as usize) * rows.len());
    for (y, row) in rows.iter().enumerate() {
        if row.chars().count() as u32 != cols {
            bail!("row {y} has {} tiles, expected {cols}", row.chars().count());
        }
        for (x, ch) in row.chars().enumerate() {
            let id = tile_for(ch).with_context(|| format!("row {y} column {x}"))?;
            tiles.push(id);
        }
    }
    Ok(ComponentScene {
        cell,
        cols,
        rows: rows.len() as u32,
        tiles,
        moss_top,
    })
}

/// Render one component grid into [`ComponentFacts`].
pub fn render_component(slug: &str, scene: &ComponentScene) -> ComponentFacts {
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

    paint_terrain(&mut svg, &grid, c, scene.moss_top);
    paint_water(&mut svg, &grid, c);
    paint_cell_art(&mut svg, &grid, c);

    let mut markers = String::new();
    push_markers(&mut markers, &grid, c);
    if !markers.is_empty() {
        svg.push_str("<g visibility=\"hidden\">");
        svg.push_str(&markers);
        svg.push_str("</g>");
    }
    svg.push_str("</svg>");

    let data = svg.into_bytes();
    ComponentFacts {
        slug: slug.to_string(),
        width_px,
        height_px,
        content_hash: content_hash(width_px, height_px, &data),
        data,
    }
}

/// The bootstrap component library: small ASCII grids plus dedicated
/// painters for the shapes a square grid cannot express.
pub fn starter_library() -> Vec<ComponentFacts> {
    let cell = GAMEPLAY_CELL_SIZE as u32;
    let sources: Vec<(&str, u32, bool, Vec<&str>)> = vec![
        ("ground-flat", cell, true, vec!["########", "########"]),
        ("spikes", cell, true, vec!["^", "#"]),
        ("bush-cluster", cell, true, vec!["d"]),
    ];
    let mut library: Vec<ComponentFacts> = sources
        .into_iter()
        .map(|(slug, cell, moss_top, rows)| {
            let scene = parse_scene(cell, moss_top, &rows).expect("starter library grids are well-formed");
            render_component(slug, &scene)
        })
        .collect();
    library.push(render_fill("dirt-fill", 512, 256));
    library.push(render_fill("dirt-slab", 256, 64));
    // Slope drops are all 512 px; the run sets the angle.
    library.push(render_slope_down("slope-down-60", 288, 512));
    library.push(render_slope_down("slope-down-45", 512, 512));
    library.push(render_slope_down("slope-down-30", 896, 512));
    library.push(render_ramp("launch-ramp", 192, 96));
    library.push(render_bank("bank"));
    library.push(render_water("water", 256, 64));
    library.push(render_fire("fire"));
    library.push(render_heavy("heavy-block"));
    library.push(render_dirt_band("dirt-band"));
    library
}

/// Open one standalone component document.
fn open_svg(width_px: u32, height_px: u32) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width_px}\" height=\"{height_px}\" \
         viewBox=\"0 0 {width_px} {height_px}\">"
    )
}

/// Close the document and wrap it into [`ComponentFacts`].
fn close_svg(slug: &str, width_px: u32, height_px: u32, mut svg: String, markers: &str) -> ComponentFacts {
    if !markers.is_empty() {
        svg.push_str("<g visibility=\"hidden\">");
        svg.push_str(markers);
        svg.push_str("</g>");
    }
    svg.push_str("</svg>");
    let data = svg.into_bytes();
    ComponentFacts {
        slug: slug.to_string(),
        width_px,
        height_px,
        content_hash: content_hash(width_px, height_px, &data),
        data,
    }
}

/// Deterministic speckles sprinkled inside a dirt mass.
fn push_speckles(svg: &mut String, width_px: u32, height_px: u32, top_at: impl Fn(f64) -> f64) {
    let mut i = 0u32;
    loop {
        let cx = 40.0 + i as f64 * 88.0;
        if cx >= width_px as f64 - 16.0 {
            break;
        }
        let top = top_at(cx);
        let cy = top + (height_px as f64 - top) * if i.is_multiple_of(2) { 0.42 } else { 0.68 };
        if cy < height_px as f64 - 10.0 {
            svg.push_str(&format!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{DIRT_DARK}\" opacity=\".5\"/>",
                n(cx),
                n(cy),
                n(if i.is_multiple_of(3) { 4.0 } else { 3.2 })
            ));
        }
        i += 1;
    }
}

/// A plain underground dirt mass (no moss cap): one merged solid collider.
fn render_fill(slug: &str, width_px: u32, height_px: u32) -> ComponentFacts {
    let mut svg = open_svg(width_px, height_px);
    svg.push_str(&format!(
        "<rect width=\"{width_px}\" height=\"{height_px}\" fill=\"{DIRT}\"/>"
    ));
    push_speckles(&mut svg, width_px, height_px, |_| 0.0);
    let markers = format!(
        "<rect data-t=\"{}\" width=\"{width_px}\" height=\"{height_px}\"/>",
        tile::SOLID
    );
    close_svg(slug, width_px, height_px, svg, &markers)
}

/// A descending slope: surface from the top-left corner down to `drop` at
/// the right edge, with dirt meat continuing 128 px below the low end. One
/// convex quad collider covers the whole mass.
fn render_slope_down(slug: &str, run: u32, drop: u32) -> ComponentFacts {
    let height_px = drop + 128;
    let (w, d, h) = (run as f64, drop as f64, height_px as f64);
    let mut svg = open_svg(run, height_px);
    svg.push_str(&format!(
        "<polygon points=\"0,0 {},{} {},{} 0,{}\" fill=\"{DIRT}\"/>",
        n(w),
        n(d),
        n(w),
        n(h),
        n(h)
    ));
    // Moss band parallel to the surface, with a lighter top lip.
    svg.push_str(&format!(
        "<polygon points=\"0,0 {},{} {},{} 0,20\" fill=\"{MOSS}\"/>",
        n(w),
        n(d),
        n(w),
        n(d + 20.0)
    ));
    svg.push_str(&format!(
        "<polygon points=\"0,0 {},{} {},{} 0,8\" fill=\"{MOSS_LIGHT}\"/>",
        n(w),
        n(d),
        n(w),
        n(d + 8.0)
    ));
    push_speckles(&mut svg, run, height_px, |x| d * x / w + 24.0);
    let markers = format!(
        "<polygon data-t=\"{}\" points=\"0,0 {},{} {},{} 0,{}\"/>",
        tile::SLOPE_DOWN,
        n(w),
        n(d),
        n(w),
        n(h),
        n(h)
    );
    close_svg(slug, run, height_px, svg, &markers)
}

/// A short launch ramp rising left-to-right; place it on flat ground to
/// throw the player into the air.
fn render_ramp(slug: &str, run: u32, rise: u32) -> ComponentFacts {
    render_wedge(slug, run, rise, 32)
}

/// A pool bank: a one-cell wedge that lets the player roll into and out of
/// recessed water without jumping. Flip it horizontally for the entry side.
fn render_bank(slug: &str) -> ComponentFacts {
    render_wedge(slug, 64, 64, 0)
}

/// A right-rising wedge with `base` px of dirt below the low end.
fn render_wedge(slug: &str, run: u32, rise: u32, base: u32) -> ComponentFacts {
    let height_px = rise + base;
    let (w, h) = (run as f64, height_px as f64);
    let top = h - rise as f64;
    let mut svg = open_svg(run, height_px);
    svg.push_str(&format!(
        "<polygon points=\"0,{} {},{} {},{}\" fill=\"{DIRT}\"/>",
        n(h),
        n(w),
        n(top),
        n(w),
        n(h)
    ));
    svg.push_str(&format!(
        "<polygon points=\"0,{} {},{} {},{} {},{}\" fill=\"{MOSS}\"/>",
        n(h),
        n(w),
        n(top),
        n(w),
        n(top + 22.0),
        n(w * 0.22),
        n(h)
    ));
    svg.push_str(&format!(
        "<polygon points=\"0,{} {},{} {},{} {},{}\" fill=\"{MOSS_LIGHT}\"/>",
        n(h),
        n(w),
        n(top),
        n(w),
        n(top + 9.0),
        n(w * 0.09),
        n(h)
    ));
    let markers = format!(
        "<polygon data-t=\"{}\" points=\"0,{} {},{} {},{}\"/>",
        tile::SLOPE_UP,
        n(h),
        n(w),
        n(top),
        n(w),
        n(h)
    );
    close_svg(slug, run, height_px, svg, &markers)
}

/// A standalone water surface, one cell deep, no basin — the level supplies
/// the floor (e.g. `dirt-slab`).
fn render_water(slug: &str, width_px: u32, height_px: u32) -> ComponentFacts {
    let (w, h) = (width_px as f64, height_px as f64);
    let mut svg = open_svg(width_px, height_px);
    svg.push_str(concat!(
        "<defs><linearGradient id=\"w\" x1=\"0\" y1=\"0\" x2=\"0\" y2=\"1\">",
        "<stop offset=\"0\" stop-color=\"#4f96a4\"/><stop offset=\"1\" stop-color=\"#2a5661\"/>",
        "</linearGradient></defs>"
    ));
    svg.push_str(&format!(
        "<rect width=\"{width_px}\" height=\"{height_px}\" fill=\"url(#w)\" opacity=\".92\"/>"
    ));
    // Wavy crest along the surface.
    let step = h / 8.0;
    let cy = h * 0.09;
    let mut cursor = step;
    let mut path = format!("M0 {} Q{} {} {} {}", n(cy), n(step / 2.0), n(h * 0.03), n(cursor), n(cy));
    while cursor + step <= w {
        cursor += step;
        path.push_str(&format!(" T{} {}", n(cursor), n(cy)));
    }
    svg.push_str(&format!(
        "<path d=\"{path}\" fill=\"none\" stroke=\"#c2e0e4\" stroke-width=\"{}\" opacity=\".75\"/>",
        n(h * 0.04)
    ));
    for i in 0..(width_px / 64) {
        let (fx, fy) = if i.is_multiple_of(2) { (0.30, 0.46) } else { (0.68, 0.72) };
        svg.push_str(&format!(
            "<ellipse cx=\"{}\" cy=\"{}\" rx=\"6.4\" ry=\"1.9\" fill=\"#c2e0e4\" opacity=\".2\"/>",
            n((i as f64 + fx) * 64.0),
            n(fy * h)
        ));
    }
    let markers = format!(
        "<rect data-t=\"{}\" width=\"{width_px}\" height=\"{height_px}\"/>",
        tile::WATER
    );
    close_svg(slug, width_px, height_px, svg, &markers)
}

/// A single standalone flame (64×64), no ground.
fn render_fire(slug: &str) -> ComponentFacts {
    let mut svg = open_svg(64, 64);
    svg.push_str(concat!(
        "<defs><radialGradient id=\"f\" cx=\"50%\" cy=\"70%\">",
        "<stop offset=\"0\" stop-color=\"#ffe08a\"/><stop offset=\".5\" stop-color=\"#f2882c\"/>",
        "<stop offset=\"1\" stop-color=\"#d13a1f\"/>",
        "</radialGradient></defs>"
    ));
    svg.push_str(&format!("<g transform=\"scale(0.64)\">{ART_FIRE}</g>"));
    let markers = format!("<rect data-t=\"{}\" width=\"64\" height=\"64\"/>", tile::FIRE);
    close_svg(slug, 64, 64, svg, &markers)
}

/// The heavy pushable block (64×64). The client excludes it from the
/// composed plane and reuses this art as the dynamic sprite's texture.
fn render_heavy(slug: &str) -> ComponentFacts {
    let mut svg = open_svg(64, 64);
    svg.push_str(concat!(
        "<rect x=\"2\" y=\"2\" width=\"60\" height=\"60\" rx=\"9\" fill=\"#6e6257\" stroke=\"#4a4038\" stroke-width=\"3\"/>",
        "<rect x=\"8\" y=\"8\" width=\"48\" height=\"21\" rx=\"6\" fill=\"#8a7d70\" opacity=\".55\"/>",
        "<circle cx=\"11\" cy=\"11\" r=\"2.5\" fill=\"#3c342d\"/><circle cx=\"53\" cy=\"11\" r=\"2.5\" fill=\"#3c342d\"/>",
        "<circle cx=\"11\" cy=\"53\" r=\"2.5\" fill=\"#3c342d\"/><circle cx=\"53\" cy=\"53\" r=\"2.5\" fill=\"#3c342d\"/>",
        "<path d=\"M24 40 h16 M32 32 v16\" stroke=\"#4a4038\" stroke-width=\"4\" stroke-linecap=\"round\"/>"
    ));
    let markers = format!("<rect data-t=\"{}\" width=\"64\" height=\"64\"/>", tile::HEAVY);
    close_svg(slug, 64, 64, svg, &markers)
}

/// Muted distant-terrain palette so background bands recede against the
/// dusk sky instead of competing with the gameplay plane.
const BAND_MOSS: &str = "#55603a";
const BAND_DIRT: &str = "#7c4a3a";
const BAND_DIRT_DEEP: &str = "#5f3a30";
const BAND_BUSH: &str = "#4c5528";

/// A background band: a rolling dirt silhouette with a muted moss crest and
/// sparse bush blobs. Pure scenery — no collider markers.
fn render_dirt_band(slug: &str) -> ComponentFacts {
    const WIDTH: u32 = 2048;
    const HEIGHT: u32 = 384;
    /// Crest heights (px below the top edge) sampled every 128 px.
    const CRESTS: [f64; 17] = [
        150.0, 96.0, 60.0, 84.0, 140.0, 110.0, 52.0, 76.0, 128.0, 156.0, 100.0, 64.0, 92.0, 138.0, 88.0, 120.0, 150.0,
    ];
    let step = WIDTH as f64 / (CRESTS.len() - 1) as f64;

    // One smooth path through the crest points (quadratics through segment
    // midpoints), closed along the bottom edge.
    let silhouette = |offset_y: f64| -> String {
        let mut d = format!("M0 {}", n(HEIGHT as f64));
        d.push_str(&format!(" L0 {}", n(CRESTS[0] + offset_y)));
        for i in 0..CRESTS.len() - 1 {
            let x0 = i as f64 * step;
            let x1 = (i + 1) as f64 * step;
            let mid_x = (x0 + x1) / 2.0;
            let mid_y = (CRESTS[i] + CRESTS[i + 1]) / 2.0 + offset_y;
            d.push_str(&format!(
                " Q{} {} {} {}",
                n(x0 + step / 2.0),
                n(CRESTS[i] + offset_y),
                n(mid_x),
                n(mid_y)
            ));
        }
        d.push_str(&format!(
            " Q{} {} {} {}",
            n(WIDTH as f64 - step / 4.0),
            n(CRESTS[CRESTS.len() - 1] + offset_y),
            n(WIDTH as f64),
            n(CRESTS[CRESTS.len() - 1] + offset_y)
        ));
        d.push_str(&format!(" L{} {} Z", n(WIDTH as f64), n(HEIGHT as f64)));
        d
    };

    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{WIDTH}\" height=\"{HEIGHT}\" \
         viewBox=\"0 0 {WIDTH} {HEIGHT}\">"
    );
    // Moss crest first, then the dirt mass 16 px lower leaves a mossy lip.
    svg.push_str(&format!("<path d=\"{}\" fill=\"{BAND_MOSS}\"/>", silhouette(0.0)));
    svg.push_str(&format!("<path d=\"{}\" fill=\"{BAND_DIRT}\"/>", silhouette(16.0)));
    // A deeper shadow band along the bottom grounds the strip.
    svg.push_str(&format!("<path d=\"{}\" fill=\"{BAND_DIRT_DEEP}\"/>", silhouette(200.0)));
    // Sparse muted bush blobs riding the crest.
    for (i, &(index, spread)) in [(1usize, 26.0), (4, 34.0), (7, 22.0), (10, 30.0), (13, 24.0), (15, 28.0)]
        .iter()
        .enumerate()
    {
        let cx = index as f64 * step + if i % 2 == 0 { 34.0 } else { -28.0 };
        let cy = CRESTS[index] + 4.0;
        svg.push_str(&format!(
            "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"{BAND_BUSH}\"/>\
             <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{BAND_BUSH}\"/>\
             <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{BAND_BUSH}\"/>",
            n(cx),
            n(cy),
            n(spread),
            n(spread * 0.35),
            n(cx - spread * 0.45),
            n(cy - spread * 0.28),
            n(spread * 0.42),
            n(cx + spread * 0.35),
            n(cy - spread * 0.34),
            n(spread * 0.5)
        ));
    }
    svg.push_str("</svg>");

    let data = svg.into_bytes();
    ComponentFacts {
        slug: slug.to_string(),
        width_px: WIDTH,
        height_px: HEIGHT,
        content_hash: content_hash(WIDTH, HEIGHT, &data),
        data,
    }
}

/// Merged dirt mass, one moss surface along the sky line, slope wedges.
fn paint_terrain(svg: &mut String, grid: &Grid, c: f64, moss_top: bool) {
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
    if moss_top {
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
                "<path d=\"{path}\" fill=\"none\" stroke=\"#c2e0e4\" stroke-width=\"{}\" opacity=\".75\"/>",
                n(0.04 * c)
            ));
        }
        for i in 0..len {
            let (fx, fy) = if (x + i + y) % 2 == 0 { (0.30, 0.46) } else { (0.68, 0.72) };
            svg.push_str(&format!(
                "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"#c2e0e4\" opacity=\".2\"/>",
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
                tile::LETHAL | tile::WATER | tile::FIRE | tile::HEAVY => {
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
    use rocknrolla_level::validate_component;

    fn scene(rows: &[&str]) -> ComponentScene {
        parse_scene(64, true, rows).unwrap()
    }

    #[test]
    fn renders_markers_and_merged_terrain() {
        let facts = render_component("test", &scene(&["..~H", "##~."]));
        assert_eq!(facts.width_px, 256);
        assert_eq!(facts.height_px, 128);
        let svg = String::from_utf8(facts.data).unwrap();
        // Two adjacent solid cells merge into one 128-wide marker rect.
        assert!(svg.contains("data-t=\"1\" x=\"0\" y=\"64\" width=\"128\""), "{svg}");
        assert!(svg.contains("data-t=\"7\""));
        // Heavy blocks are marker-only: no visible art beyond the marker.
        assert_eq!(svg.matches("data-t=\"9\"").count(), 1);
    }

    #[test]
    fn decor_only_components_have_no_marker_group() {
        let facts = render_component("bush", &scene(&["d"]));
        let svg = String::from_utf8(facts.data).unwrap();
        assert!(!svg.contains("visibility=\"hidden\""), "{svg}");
    }

    #[test]
    fn moss_top_flag_controls_the_sky_cap() {
        let capped = String::from_utf8(render_component("a", &scene(&["##"])).data).unwrap();
        assert!(capped.contains(MOSS), "{capped}");
        let bare_scene = parse_scene(64, false, &["##"]).unwrap();
        let bare = String::from_utf8(render_component("a", &bare_scene).data).unwrap();
        assert!(!bare.contains(MOSS), "{bare}");
    }

    #[test]
    fn starter_library_components_validate() {
        let library = starter_library();
        assert!(library.len() >= 12);
        for component in &library {
            validate_component(component).unwrap();
        }
        let slope = library.iter().find(|c| c.slug == "slope-down-45").unwrap();
        assert_eq!((slope.width_px, slope.height_px), (512, 640));
        let svg = std::str::from_utf8(&slope.data).unwrap();
        assert!(svg.contains("data-t=\"3\""), "slope marker missing");
    }

    #[test]
    fn dedicated_painters_emit_their_markers_without_ground() {
        let library = starter_library();
        let find = |slug: &str| library.iter().find(|c| c.slug == slug).unwrap();

        let water = find("water");
        let svg = std::str::from_utf8(&water.data).unwrap();
        assert!(svg.contains("data-t=\"7\""), "{svg}");
        assert!(!svg.contains("data-t=\"1\""), "water must not carry ground: {svg}");

        let fire = find("fire");
        let svg = std::str::from_utf8(&fire.data).unwrap();
        assert_eq!((fire.width_px, fire.height_px), (64, 64));
        assert_eq!(svg.matches("data-t=\"8\"").count(), 1, "one flame only: {svg}");
        assert!(!svg.contains("data-t=\"1\""), "fire must not carry ground: {svg}");

        let heavy = find("heavy-block");
        let svg = std::str::from_utf8(&heavy.data).unwrap();
        assert!(svg.contains("data-t=\"9\""), "{svg}");
        assert!(svg.contains("<rect x=\"2\""), "heavy block must have visible art: {svg}");

        let ramp = find("launch-ramp");
        let svg = std::str::from_utf8(&ramp.data).unwrap();
        assert!(svg.contains("data-t=\"2\""), "ramp needs a slope-up collider: {svg}");
    }
}
