pub mod ui;
pub mod input;

use crate::task::Status;
use crate::todolist::TodoList;

// ---- NEW: path data type for NYC map (GeoJSON→paths conversion output) ----
#[derive(Debug, serde::Deserialize, Clone)]
pub struct MapPaths(pub Vec<Vec<Vec<[f64; 2]>>>); // features -> rings -> [lon,lat]

// Minimal GeoJSON structs (only what we need)
#[derive(Debug, serde::Deserialize)]
struct GeoFeatureCollection {
    #[serde(rename = "type")]
    typ: String,
    features: Vec<GeoFeature>,
}
#[derive(Debug, serde::Deserialize)]
struct GeoFeature {
    geometry: GeoGeometry,
}
#[derive(Debug, serde::Deserialize)]
struct GeoGeometry {
    #[serde(rename = "type")]
    typ: String,
    coordinates: serde_json::Value,
}

// Tabs
#[derive(Debug)]
pub struct Tabs {
    pub titles: Vec<&'static str>,
    pub index: usize,
}
impl Tabs {
    pub fn new(titles: Vec<&'static str>) -> Self {
        Self { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }
    pub fn prev(&mut self) {
        self.index = if self.index == 0 { self.titles.len() - 1 } else { self.index - 1 };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode { Normal, Insert }

// Focusable fields in Insert mode (Tab cycles through these)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertField { Title, Notes, Time, Priority }

// Map view selector in the World tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapView { World, NYC }

/// Central TUI state
pub struct App {
    pub list: TodoList,
    pub selected: usize,
    pub input_mode: InputMode,

    // insert-mode drafts
    pub insert_field: InsertField,
    pub draft_title: String,
    pub draft_priority: i8,
    pub draft_notes: String,
    pub draft_timeframe: String,

    pub status_line: String,
    pub dirty: bool,

    // visuals/animation
    pub tabs: Tabs,
    pub show_chart: bool,
    pub enhanced_graphics: bool,
    pub progress: f64, // 0..1 wave
    pub pulse: f64,    // 0..tau loop
    pub spark_points: Vec<u64>,

    // inline expansion in Todos tab
    pub expanded: bool,

    // world tab view
    pub map_view: MapView,

    // ---- NYC map data loaded once at startup (optional) ----
    pub nyc_paths: Option<MapPaths>,
    pub nyc_bbox: Option<([f64; 2], [f64; 2])>, // ([min_lon, max_lon], [min_lat, max_lat])
}

impl App {
    pub fn new(list: TodoList) -> Self {
        // Try to load NYC data at startup (supports both compact paths JSON and raw GeoJSON)
        let (nyc_paths, nyc_bbox) = load_nyc_paths_and_bbox();

        Self {
            list,
            selected: 0,
            input_mode: InputMode::Normal,
            insert_field: InsertField::Title,
            draft_title: String::new(),
            draft_priority: 1,
            draft_notes: String::new(),
            draft_timeframe: String::new(),
            status_line: String::new(),
            dirty: false,

            tabs: Tabs::new(vec!["Todos", "Dash", "World"]),
            show_chart: true,
            enhanced_graphics: true,

            progress: 0.0,
            pulse: 0.0,
            spark_points: vec![0; 60],

            expanded: false,
            map_view: MapView::World,

            nyc_paths,
            nyc_bbox,
        }
    }

    pub fn visible_indices(&self) -> Vec<usize> {
        (0..self.list.items.len()).collect()
    }
    pub fn select_next(&mut self) {
        let len = self.visible_indices().len();
        if len > 0 && self.selected + 1 < len { self.selected += 1; }
    }
    pub fn select_prev(&mut self) {
        if self.selected > 0 { self.selected -= 1; }
    }
    pub fn clamp_selection(&mut self) {
        let len = self.visible_indices().len();
        if len == 0 { self.selected = 0; }
        else if self.selected >= len { self.selected = len - 1; }
    }

    // metrics
    pub fn percent_done(&self) -> f64 {
        let total = self.list.items.len() as f64;
        if total == 0.0 { 0.0 } else {
            let done = self.list.items.iter().filter(|t| t.status == Status::Done).count() as f64;
            done / total
        }
    }
    pub fn counts_by_priority(&self) -> [u64; 5] {
        let mut c = [0u64; 5];
        for t in &self.list.items {
            let p = t.priority.clamp(1, 5) as usize;
            c[p - 1] += 1;
        }
        c
    }

    // animation tick
    pub fn on_tick(&mut self) {
        self.progress += 0.01;
        if self.progress > 1.0 { self.progress = 0.0; }

        let base = (self.percent_done() * 100.0) as u64;
        let wobble = ((self.pulse.sin() * 20.0) + 20.0) as u64;
        self.spark_points.remove(0);
        self.spark_points.push(base + wobble);

        const TAU: f64 = std::f64::consts::PI * 2.0;
        self.pulse += 0.07;
        if self.pulse > TAU { self.pulse -= TAU; }
    }
}

// ------------------------ NYC paths loading & conversion ------------------------

fn load_nyc_paths_and_bbox() -> (Option<MapPaths>, Option<([f64; 2], [f64; 2])>) {
    use std::env;
    use std::fs;

    // Try in this order: ENV, local assets, absolute /root path
    let candidates = [
        env::var("NYC_PATHS").ok(),
        Some("assets/nyc_paths.json".to_string()),
        Some("/root/assets/nyc_paths.json".to_string()),
    ];

    for c in candidates.into_iter().flatten() {
        if let Ok(raw) = fs::read_to_string(&c) {
            // 1) Try compact MapPaths first
            if let Ok(paths) = serde_json::from_str::<MapPaths>(&raw) {
                let bbox = compute_bbox(&paths);
                return (Some(paths), Some(bbox));
            }
            // 2) Try GeoJSON and convert
            if let Ok(fc) = serde_json::from_str::<GeoFeatureCollection>(&raw) {
                if let Some(paths) = geojson_to_paths(&fc) {
                    let bbox = compute_bbox(&paths);
                    return (Some(paths), Some(bbox));
                }
            }
            // 3) Otherwise keep looking
        }
    }
    (None, None)
}

fn compute_bbox(paths: &MapPaths) -> ([f64; 2], [f64; 2]) {
    let mut min_lon = f64::INFINITY;
    let mut max_lon = f64::NEG_INFINITY;
    let mut min_lat = f64::INFINITY;
    let mut max_lat = f64::NEG_INFINITY;

    for feature in &paths.0 {
        for ring in feature {
            for pt in ring {
                let lon = pt[0];
                let lat = pt[1];
                if lon < min_lon { min_lon = lon; }
                if lon > max_lon { max_lon = lon; }
                if lat < min_lat { min_lat = lat; }
                if lat > max_lat { max_lat = lat; }
            }
        }
    }
    ([min_lon, max_lon], [min_lat, max_lat])
}

fn geojson_to_paths(fc: &GeoFeatureCollection) -> Option<MapPaths> {
    if fc.typ != "FeatureCollection" { return None; }
    let mut out: Vec<Vec<Vec<[f64; 2]>>> = Vec::new();

    for feat in &fc.features {
        match feat.geometry.typ.as_str() {
            "Polygon" => {
                if let Some(poly) = parse_polygon_coords(&feat.geometry.coordinates) {
                    out.push(poly);
                }
            }
            "MultiPolygon" => {
                if let Some(multi) = parse_multipolygon_coords(&feat.geometry.coordinates) {
                    for poly in multi {
                        out.push(poly);
                    }
                }
            }
            _ => { /* ignore other geometry types */ }
        }
    }

    if out.is_empty() { None } else { Some(MapPaths(out)) }
}

fn parse_polygon_coords(v: &serde_json::Value) -> Option<Vec<Vec<[f64; 2]>>> {
    // coordinates: [ [ [lon,lat], ... ] , [hole...], ... ]
    let rings = v.as_array()?;
    let mut out: Vec<Vec<[f64; 2]>> = Vec::new();
    for ring in rings {
        let pts = ring.as_array()?;
        let mut one: Vec<[f64; 2]> = Vec::with_capacity(pts.len());
        for p in pts {
            let arr = p.as_array()?;
            if arr.len() < 2 { return None; }
            let lon = arr[0].as_f64()?;
            let lat = arr[1].as_f64()?;
            one.push([lon, lat]);
        }
        // Drop last if it repeats the first
        if one.len() > 1 && one.first() == one.last() {
            one.pop();
        }
        out.push(one);
    }
    Some(out)
}

fn parse_multipolygon_coords(v: &serde_json::Value) -> Option<Vec<Vec<Vec<[f64; 2]>>>> {
    // coordinates: [ [ [ [lon,lat], ... ], [hole...], ... ],  ... ]
    let polys = v.as_array()?;
    let mut out: Vec<Vec<Vec<[f64; 2]>>> = Vec::new();
    for poly in polys {
        if let Some(p) = parse_polygon_coords(poly) {
            out.push(p);
        }
    }
    Some(out)
}
