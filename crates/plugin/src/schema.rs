use photopipeline_core::{
    ColorMode, EnumDisplay, FilePathKind, FloatWidget, IntegerWidget, SliderOrientation,
    SliderStyle,
};
use serde::{Deserialize, Serialize};

pub use photopipeline_core::{
    AuxView, GuiCell, GuiLayout, GuiRow, GuiSchema, GuiSection, LabelPosition, PreviewMode,
    RowHeight, SectionStyle, SplitOrientation,
};

// ---- Parameter Schema ----
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    pub version: u32,
    pub sections: Vec<ParameterSection>,
}

impl ParameterSchema {
    pub fn empty() -> Self {
        Self {
            version: 1,
            sections: vec![],
        }
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

fn default_true() -> bool {
    true
}

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
    Expression { variables: Vec<VariableDef> },
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
    Separator { label: Option<String> },
    #[serde(rename = "section")]
    Section { fields: Vec<ParameterField> },
}

fn default_step() -> f64 {
    1.0
}

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
        Self {
            values: Default::default(),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_schema() -> ParameterSchema {
        ParameterSchema {
            version: 1,
            sections: vec![
                ParameterSection {
                    id: "basic".into(),
                    label: "Basic".into(),
                    description: None,
                    icon: None,
                    collapsible: false,
                    default_collapsed: false,
                    fields: vec![
                        ParameterField {
                            id: "brightness".into(),
                            label: "Brightness".into(),
                            description: None,
                            help_url: None,
                            field_type: ParameterType::Float {
                                min: -1.0,
                                max: 1.0,
                                step: 0.01,
                                precision: 2,
                                unit: None,
                                logarithmic: false,
                                style: FloatWidget::default(),
                            },
                            default: serde_json::json!(0.0),
                            required: false,
                            advanced: false,
                            allow_override: true,
                            supports_expression: false,
                        },
                        ParameterField {
                            id: "enabled".into(),
                            label: "Enabled".into(),
                            description: None,
                            help_url: None,
                            field_type: ParameterType::Boolean {
                                label_true: None,
                                label_false: None,
                            },
                            default: serde_json::json!(true),
                            required: false,
                            advanced: false,
                            allow_override: true,
                            supports_expression: false,
                        },
                    ],
                },
                ParameterSection {
                    id: "advanced".into(),
                    label: "Advanced".into(),
                    description: None,
                    icon: None,
                    collapsible: true,
                    default_collapsed: true,
                    fields: vec![ParameterField {
                        id: "threshold".into(),
                        label: "Threshold".into(),
                        description: None,
                        help_url: None,
                        field_type: ParameterType::Integer {
                            min: 0,
                            max: 255,
                            step: 1,
                            unit: None,
                            style: IntegerWidget::default(),
                        },
                        default: serde_json::json!(128),
                        required: false,
                        advanced: true,
                        allow_override: true,
                        supports_expression: false,
                    }],
                },
            ],
        }
    }

    #[test]
    fn parameter_schema_defaults_fills_all_fields() {
        let schema = make_schema();
        let defaults = schema.defaults();
        assert!(defaults.get("brightness").is_some());
        assert!(defaults.get("enabled").is_some());
        assert!(defaults.get("threshold").is_some());
        assert_eq!(defaults.get_f64("brightness"), Some(0.0));
        assert_eq!(defaults.get_bool("enabled"), Some(true));
        assert_eq!(defaults.get_i64("threshold"), Some(128));
    }

    #[test]
    fn parameter_schema_field_lookup() {
        let schema = make_schema();
        let field = schema.field("basic", "brightness");
        assert!(field.is_some());
        assert_eq!(field.unwrap().label, "Brightness");

        let field = schema.field("advanced", "threshold");
        assert!(field.is_some());
        assert_eq!(field.unwrap().label, "Threshold");

        let field = schema.field("basic", "nonexistent");
        assert!(field.is_none());

        let field = schema.field("nonexistent", "brightness");
        assert!(field.is_none());
    }

    #[test]
    fn parameter_schema_field_multiple_sections() {
        let schema = make_schema();
        let field = schema.field("basic", "enabled");
        assert!(field.is_some());
        let field = schema.field("advanced", "threshold");
        assert!(field.is_some());
    }

    #[test]
    fn parameter_set_insert_and_get() {
        let mut ps = ParameterSet::new();
        ps.insert("key1".into(), serde_json::json!("value1"));
        ps.insert("key2".into(), serde_json::json!(42));

        assert_eq!(ps.get_str("key1"), Some("value1"));
        assert_eq!(ps.get_i64("key2"), Some(42));
        assert!(ps.get("key3").is_none());
    }

    #[test]
    fn parameter_set_get_str() {
        let mut ps = ParameterSet::new();
        ps.insert("name".into(), serde_json::json!("hello"));
        assert_eq!(ps.get_str("name"), Some("hello"));
        assert_eq!(ps.get_str("missing"), None);
    }

    #[test]
    fn parameter_set_get_i64() {
        let mut ps = ParameterSet::new();
        ps.insert("count".into(), serde_json::json!(100));
        assert_eq!(ps.get_i64("count"), Some(100));
        assert_eq!(ps.get_i64("missing"), None);
    }

    #[test]
    fn parameter_set_get_f64() {
        let mut ps = ParameterSet::new();
        ps.insert("ratio".into(), serde_json::json!(1.5));
        assert!((ps.get_f64("ratio").unwrap() - 1.5).abs() < 0.001);
        assert_eq!(ps.get_f64("missing"), None);
    }

    #[test]
    fn parameter_set_get_bool() {
        let mut ps = ParameterSet::new();
        ps.insert("flag".into(), serde_json::json!(true));
        assert_eq!(ps.get_bool("flag"), Some(true));
        ps.insert("flag2".into(), serde_json::json!(false));
        assert_eq!(ps.get_bool("flag2"), Some(false));
        assert_eq!(ps.get_bool("missing"), None);
    }

    #[test]
    fn parameter_set_get_wrong_type_returns_none() {
        let mut ps = ParameterSet::new();
        ps.insert("key".into(), serde_json::json!("not_a_number"));
        assert_eq!(ps.get_i64("key"), None);
        assert_eq!(ps.get_f64("key"), None);
        assert_eq!(ps.get_bool("key"), None);
    }

    #[test]
    fn parameter_set_merge_shallow_override() {
        let mut base = ParameterSet::new();
        base.insert("a".into(), serde_json::json!(1));
        base.insert("b".into(), serde_json::json!(2));

        let mut overrides = ParameterSet::new();
        overrides.insert("b".into(), serde_json::json!(99));
        overrides.insert("c".into(), serde_json::json!(3));

        base.merge(&overrides);

        assert_eq!(base.get_i64("a"), Some(1));
        assert_eq!(base.get_i64("b"), Some(99));
        assert_eq!(base.get_i64("c"), Some(3));
    }

    #[test]
    fn parameter_set_merge_does_not_remove() {
        let mut base = ParameterSet::new();
        base.insert("keep".into(), serde_json::json!("value"));
        let other = ParameterSet::new();
        base.merge(&other);
        assert_eq!(base.get_str("keep"), Some("value"));
    }

    #[test]
    fn parameter_set_iter() {
        let mut ps = ParameterSet::new();
        ps.insert("x".into(), serde_json::json!(1));
        ps.insert("y".into(), serde_json::json!(2));
        let mut keys: Vec<&String> = ps.iter().map(|(k, _)| k).collect();
        keys.sort();
        assert_eq!(keys, vec!["x", "y"]);
    }

    #[test]
    fn parameter_schema_empty() {
        let schema = ParameterSchema::empty();
        assert_eq!(schema.version, 1);
        assert!(schema.sections.is_empty());
        assert!(schema.defaults().values.is_empty());
        assert!(schema.all_fields().is_empty());
    }

    #[test]
    fn parameter_schema_all_fields() {
        let schema = make_schema();
        let fields = schema.all_fields();
        assert_eq!(fields.len(), 3);
    }

    #[test]
    fn parameter_type_string_serialization_roundtrip() {
        let pt = ParameterType::String {
            max_length: 256,
            pattern: Some("[a-z]+".into()),
            placeholder: None,
        };
        let json = serde_json::to_string(&pt).unwrap();
        let pt2: ParameterType = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&pt2).unwrap();
        assert_eq!(json, json2);
    }

    #[test]
    fn parameter_type_integer_serialization_roundtrip() {
        let pt = ParameterType::Integer {
            min: 0,
            max: 100,
            step: 1,
            unit: Some("px".into()),
            style: Default::default(),
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"integer\""));
    }

    #[test]
    fn parameter_type_float_serialization_roundtrip() {
        let pt = ParameterType::Float {
            min: 0.0,
            max: 1.0,
            step: 0.1,
            precision: 2,
            unit: None,
            logarithmic: true,
            style: Default::default(),
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"float\""));
    }

    #[test]
    fn parameter_type_boolean_serialization_roundtrip() {
        let pt = ParameterType::Boolean {
            label_true: Some("On".into()),
            label_false: None,
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"boolean\""));
    }

    #[test]
    fn parameter_type_enum_serialization_roundtrip() {
        let pt = ParameterType::Enum {
            options: vec![EnumOption {
                value: "a".into(),
                label: "A".into(),
                description: None,
                icon: None,
                tags: vec![],
                recommended: true,
            }],
            display: Default::default(),
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"enum\""));
    }

    #[test]
    fn parameter_type_color_serialization_roundtrip() {
        let pt = ParameterType::Color {
            mode: ColorMode::RGB,
            show_alpha: true,
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"color\""));
    }

    #[test]
    fn parameter_type_file_path_serialization_roundtrip() {
        let pt = ParameterType::FilePath {
            kind: FilePathKind::File,
            filters: vec![],
            must_exist: true,
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"file_path\""));
    }

    #[test]
    fn parameter_type_coordinate_serialization_roundtrip() {
        let pt = ParameterType::Coordinate {
            alt_required: true,
            direction_required: false,
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"coordinate\""));
    }

    #[test]
    fn parameter_type_slider_serialization_roundtrip() {
        let pt = ParameterType::Slider {
            min: 0.0,
            max: 100.0,
            step: 5.0,
            show_ticks: true,
            ticks: Some(vec![0.0, 50.0, 100.0]),
            show_value: true,
            orientation: Default::default(),
            style: Default::default(),
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"slider\""));
    }

    #[test]
    fn parameter_type_combo_slider_serialization_roundtrip() {
        let pt = ParameterType::ComboSlider {
            min: 0.0,
            max: 10.0,
            step: 0.5,
            presets: vec![("Low".into(), 1.0), ("High".into(), 9.0)],
            unit: Some("dB".into()),
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"combo_slider\""));
    }

    #[test]
    fn parameter_type_expression_serialization_roundtrip() {
        let pt = ParameterType::Expression {
            variables: vec![VariableDef {
                name: "iso".into(),
                description: "ISO value".into(),
                var_type: "number".into(),
                example: Some("400".into()),
            }],
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"expression\""));
    }

    #[test]
    fn parameter_type_preset_serialization_roundtrip() {
        let pt = ParameterType::Preset {
            preset_schema_ref: "lut_schema".into(),
            builtin_presets: vec![NamedPreset {
                name: "warm".into(),
                description: None,
                params: Default::default(),
            }],
            allow_custom: true,
            allow_import: false,
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"preset\""));
    }

    #[test]
    fn parameter_type_array_serialization_roundtrip() {
        let inner = ParameterField {
            id: "elem".into(),
            label: "Elem".into(),
            description: None,
            help_url: None,
            field_type: ParameterType::String {
                max_length: 100,
                pattern: None,
                placeholder: None,
            },
            default: serde_json::json!(""),
            required: false,
            advanced: false,
            allow_override: true,
            supports_expression: false,
        };
        let pt = ParameterType::Array {
            element: Box::new(inner),
            min_items: 0,
            max_items: Some(10),
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"array\""));
    }

    #[test]
    fn parameter_type_map_widget_serialization_roundtrip() {
        let pt = ParameterType::MapWidget {
            show_track: true,
            show_photos: false,
            allow_manual_pin: true,
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"map_widget\""));
    }

    #[test]
    fn parameter_type_before_after_serialization_roundtrip() {
        let pt = ParameterType::BeforeAfter {
            zoom_levels: vec![1.0, 2.0, 4.0],
            show_histogram: true,
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"before_after\""));
    }

    #[test]
    fn parameter_type_separator_serialization_roundtrip() {
        let pt = ParameterType::Separator {
            label: Some("divider".into()),
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"separator\""));
    }

    #[test]
    fn parameter_type_section_serialization_roundtrip() {
        let inner = ParameterField {
            id: "sub".into(),
            label: "Sub".into(),
            description: None,
            help_url: None,
            field_type: ParameterType::Boolean {
                label_true: None,
                label_false: None,
            },
            default: serde_json::json!(false),
            required: false,
            advanced: false,
            allow_override: true,
            supports_expression: false,
        };
        let pt = ParameterType::Section {
            fields: vec![inner],
        };
        let json = serde_json::to_string(&pt).unwrap();
        let _pt2: ParameterType = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"section\""));
    }

    #[test]
    fn parameter_field_default_allow_override() {
        let field = ParameterField {
            id: "f".into(),
            label: "F".into(),
            description: None,
            help_url: None,
            field_type: ParameterType::Boolean {
                label_true: None,
                label_false: None,
            },
            default: serde_json::json!(true),
            required: false,
            advanced: false,
            allow_override: true,
            supports_expression: false,
        };
        assert!(field.allow_override);
    }

    #[test]
    fn parameter_field_default_supports_expression() {
        let field = ParameterField {
            id: "f2".into(),
            label: "F2".into(),
            description: None,
            help_url: None,
            field_type: ParameterType::String {
                max_length: 100,
                pattern: None,
                placeholder: None,
            },
            default: serde_json::json!(""),
            required: false,
            advanced: false,
            allow_override: true,
            supports_expression: false,
        };
        assert!(!field.supports_expression);
    }

    #[test]
    fn parameter_field_required_true() {
        let field = ParameterField {
            id: "req".into(),
            label: "Req".into(),
            description: None,
            help_url: None,
            field_type: ParameterType::String {
                max_length: 10,
                pattern: None,
                placeholder: None,
            },
            default: serde_json::json!(""),
            required: true,
            advanced: false,
            allow_override: true,
            supports_expression: false,
        };
        assert!(field.required);
    }

    #[test]
    fn parameter_field_advanced_true() {
        let field = ParameterField {
            id: "adv".into(),
            label: "Adv".into(),
            description: None,
            help_url: None,
            field_type: ParameterType::Integer {
                min: 0,
                max: 10,
                step: 1,
                unit: None,
                style: Default::default(),
            },
            default: serde_json::json!(0),
            required: false,
            advanced: true,
            allow_override: true,
            supports_expression: false,
        };
        assert!(field.advanced);
    }

    #[test]
    fn parameter_set_get_str_on_non_string_returns_none() {
        let mut ps = ParameterSet::new();
        ps.insert("num".into(), serde_json::json!(42));
        assert_eq!(ps.get_str("num"), None);
    }

    #[test]
    fn parameter_set_get_i64_on_float_returns_none() {
        let mut ps = ParameterSet::new();
        ps.insert("fl".into(), serde_json::json!(3.14));
        assert_eq!(ps.get_i64("fl"), None);
    }

    #[test]
    fn parameter_set_get_f64_on_integer_returns_some() {
        let mut ps = ParameterSet::new();
        ps.insert("int".into(), serde_json::json!(42));
        assert!(ps.get_f64("int").is_some());
    }

    #[test]
    fn parameter_set_merge_with_empty_no_change() {
        let mut base = ParameterSet::new();
        base.insert("k".into(), serde_json::json!(1));
        let empty = ParameterSet::new();
        base.merge(&empty);
        assert_eq!(base.get_i64("k"), Some(1));
    }

    #[test]
    fn parameter_set_merge_no_overlap() {
        let mut base = ParameterSet::new();
        base.insert("a".into(), serde_json::json!(1));
        let mut other = ParameterSet::new();
        other.insert("b".into(), serde_json::json!(2));
        base.merge(&other);
        assert_eq!(base.get_i64("a"), Some(1));
        assert_eq!(base.get_i64("b"), Some(2));
    }

    #[test]
    fn parameter_set_iter_after_merge() {
        let mut base = ParameterSet::new();
        base.insert("x".into(), serde_json::json!(1));
        let mut other = ParameterSet::new();
        other.insert("y".into(), serde_json::json!(2));
        base.merge(&other);
        let count = base.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn parameter_schema_field_invalid_section() {
        let schema = make_schema();
        assert!(schema.field("nonexistent", "brightness").is_none());
    }

    #[test]
    fn parameter_schema_field_invalid_field() {
        let schema = make_schema();
        assert!(schema.field("basic", "nonexistent").is_none());
    }

    #[test]
    fn parameter_schema_defaults_with_empty_schema() {
        let schema = ParameterSchema::empty();
        let defaults = schema.defaults();
        assert!(defaults.values.is_empty());
    }

    #[test]
    fn enum_option_default_recommended_false() {
        let opt = EnumOption {
            value: "v".into(),
            label: "L".into(),
            description: None,
            icon: None,
            tags: vec![],
            recommended: false,
        };
        assert!(!opt.recommended);
    }

    #[test]
    fn named_preset_serde_roundtrip() {
        let mut params = std::collections::HashMap::new();
        params.insert("brightness".into(), serde_json::json!(0.5));
        let preset = NamedPreset {
            name: "vivid".into(),
            description: Some("Vivid colors".into()),
            params,
        };
        let json = serde_json::to_string(&preset).unwrap();
        let preset2: NamedPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(preset2.name, "vivid");
        assert_eq!(
            preset2.params.get("brightness").and_then(|v| v.as_f64()),
            Some(0.5)
        );
    }
}
