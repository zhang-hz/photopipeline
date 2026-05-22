use serde::{Deserialize, Serialize};
use photopipeline_core::{
    ColorMode, FilePathKind, SliderOrientation, SliderStyle, FloatWidget,
    IntegerWidget, EnumDisplay,
};

pub use photopipeline_core::{
    GuiSchema, GuiLayout, GuiSection, GuiCell, GuiRow,
    PreviewMode, AuxView, SectionStyle, RowHeight, LabelPosition, SplitOrientation,
};

// ---- Parameter Schema ----
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    pub version: u32,
    pub sections: Vec<ParameterSection>,
}

impl ParameterSchema {
    pub fn empty() -> Self {
        Self { version: 1, sections: vec![] }
    }

    pub fn field(&self, section_id: &str, field_id: &str) -> Option<&ParameterField> {
        for sec in &self.sections {
            if sec.id == section_id {
                return sec.fields.iter().find(|f| f.id == field_id);
            }
        }
        None
    }

    pub fn defaults(&self) -> ParameterSet {
        let mut params = ParameterSet::new();
        for sec in &self.sections {
            for field in &sec.fields {
                params.insert(field.id.clone(), field.default.clone());
            }
        }
        params
    }

    pub fn all_fields(&self) -> Vec<&ParameterField> {
        self.sections.iter().flat_map(|s| s.fields.iter()).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSection {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub collapsible: bool,
    pub default_collapsed: bool,
    pub fields: Vec<ParameterField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterField {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub help_url: Option<String>,
    #[serde(flatten)]
    pub field_type: ParameterType,
    pub default: serde_json::Value,
    pub required: bool,
    pub advanced: bool,
    #[serde(default = "default_true")]
    pub allow_override: bool,
    #[serde(default)]
    pub supports_expression: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ParameterType {
    #[serde(rename = "string")]
    String {
        max_length: usize,
        #[serde(default)]
        pattern: Option<String>,
        placeholder: Option<String>,
    },
    #[serde(rename = "integer")]
    Integer {
        min: i64,
        max: i64,
        step: i64,
        unit: Option<String>,
        #[serde(default)]
        style: IntegerWidget,
    },
    #[serde(rename = "float")]
    Float {
        min: f64,
        max: f64,
        step: f64,
        precision: u8,
        unit: Option<String>,
        #[serde(default)]
        logarithmic: bool,
        #[serde(default)]
        style: FloatWidget,
    },
    #[serde(rename = "boolean")]
    Boolean {
        label_true: Option<String>,
        label_false: Option<String>,
    },
    #[serde(rename = "enum")]
    Enum {
        options: Vec<EnumOption>,
        #[serde(default)]
        display: EnumDisplay,
    },
    #[serde(rename = "color")]
    Color {
        #[serde(default)]
        mode: ColorMode,
        #[serde(default)]
        show_alpha: bool,
    },
    #[serde(rename = "file_path")]
    FilePath {
        #[serde(default)]
        kind: FilePathKind,
        #[serde(default)]
        filters: Vec<(String, String)>,
        #[serde(default)]
        must_exist: bool,
    },
    #[serde(rename = "coordinate")]
    Coordinate {
        #[serde(default)]
        alt_required: bool,
        #[serde(default)]
        direction_required: bool,
    },
    #[serde(rename = "slider")]
    Slider {
        min: f64,
        max: f64,
        #[serde(default = "default_step")]
        step: f64,
        #[serde(default)]
        show_ticks: bool,
        ticks: Option<Vec<f64>>,
        #[serde(default = "default_true")]
        show_value: bool,
        #[serde(default)]
        orientation: SliderOrientation,
        #[serde(default)]
        style: SliderStyle,
    },
    #[serde(rename = "combo_slider")]
    ComboSlider {
        min: f64,
        max: f64,
        #[serde(default = "default_step")]
        step: f64,
        presets: Vec<(String, f64)>,
        unit: Option<String>,
    },
    #[serde(rename = "expression")]
    Expression {
        variables: Vec<VariableDef>,
    },
    #[serde(rename = "preset")]
    Preset {
        preset_schema_ref: String,
        builtin_presets: Vec<NamedPreset>,
        #[serde(default)]
        allow_custom: bool,
        #[serde(default)]
        allow_import: bool,
    },
    #[serde(rename = "array")]
    Array {
        element: Box<ParameterField>,
        min_items: usize,
        max_items: Option<usize>,
    },
    #[serde(rename = "map_widget")]
    MapWidget {
        #[serde(default)]
        show_track: bool,
        #[serde(default)]
        show_photos: bool,
        #[serde(default)]
        allow_manual_pin: bool,
    },
    #[serde(rename = "before_after")]
    BeforeAfter {
        zoom_levels: Vec<f64>,
        #[serde(default)]
        show_histogram: bool,
    },
    #[serde(rename = "separator")]
    Separator {
        label: Option<String>,
    },
    #[serde(rename = "section")]
    Section {
        fields: Vec<ParameterField>,
    },
}

fn default_step() -> f64 { 1.0 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumOption {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub recommended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDef {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub var_type: String,
    pub example: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedPreset {
    pub name: String,
    pub description: Option<String>,
    pub params: std::collections::HashMap<String, serde_json::Value>,
}

// ---- Parameter Set ----
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParameterSet {
    pub values: std::collections::HashMap<String, serde_json::Value>,
}

impl ParameterSet {
    pub fn new() -> Self {
        Self { values: Default::default() }
    }

    pub fn insert(&mut self, key: String, value: serde_json::Value) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.values.get(key)
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.values.get(key)?.as_str()
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.values.get(key)?.as_i64()
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.values.get(key)?.as_f64()
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.values.get(key)?.as_bool()
    }

    pub fn merge(&mut self, other: &ParameterSet) {
        for (k, v) in &other.values {
            self.values.insert(k.clone(), v.clone());
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &serde_json::Value)> {
        self.values.iter()
    }
}
