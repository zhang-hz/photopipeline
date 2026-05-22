pub use photopipeline_core::{
    GuiSchema, GuiLayout, GuiSection, GuiCell, GuiRow, PreviewMode, AuxView,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodePanelDefinition {
    pub plugin_id: String,
    pub plugin_name: String,
    pub plugin_category: String,
    pub sections: Vec<PanelSection>,
    pub context_bar: ContextBarConfig,
    pub help_text: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PanelSection {
    pub id: String,
    pub label: String,
    pub widget: PanelWidget,
    pub collapsible: bool,
    pub default_collapsed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum PanelWidget {
    #[serde(rename = "text_input")]
    TextInput {
        param_id: String,
        placeholder: Option<String>,
        max_length: usize,
    },
    #[serde(rename = "number_input")]
    NumberInput {
        param_id: String,
        min: f64,
        max: f64,
        step: f64,
        precision: u8,
        unit: Option<String>,
    },
    #[serde(rename = "slider")]
    Slider {
        param_id: String,
        min: f64,
        max: f64,
        step: f64,
        show_value: bool,
    },
    #[serde(rename = "toggle")]
    Toggle {
        param_id: String,
        label_on: Option<String>,
        label_off: Option<String>,
    },
    #[serde(rename = "dropdown")]
    Dropdown {
        param_id: String,
        options: Vec<DropdownOption>,
    },
    #[serde(rename = "segmented_control")]
    SegmentedControl {
        param_id: String,
        options: Vec<DropdownOption>,
    },
    #[serde(rename = "card_selector")]
    CardSelector {
        param_id: String,
        options: Vec<CardOption>,
    },
    #[serde(rename = "file_picker")]
    FilePicker {
        param_id: String,
        kind: String,
        filters: Vec<(String, String)>,
    },
    #[serde(rename = "color_picker")]
    ColorPicker {
        param_id: String,
        show_alpha: bool,
    },
    #[serde(rename = "coordinate_input")]
    CoordinateInput {
        param_id_lat: String,
        param_id_lon: String,
        param_id_alt: Option<String>,
    },
    #[serde(rename = "combo_slider")]
    ComboSlider {
        param_id: String,
        presets: Vec<(String, f64)>,
        min: f64,
        max: f64,
        unit: Option<String>,
    },
    #[serde(rename = "map_widget")]
    MapWidget {
        param_id_lat: String,
        param_id_lon: String,
        show_track: bool,
        show_photos: bool,
        allow_manual_pin: bool,
    },
    #[serde(rename = "expression_editor")]
    ExpressionEditor {
        param_id: String,
        variables: Vec<String>,
        example: Option<String>,
    },
    #[serde(rename = "before_after_preview")]
    BeforeAfterPreview {
        zoom_levels: Vec<f64>,
        show_histogram: bool,
    },
    #[serde(rename = "nested_fields")]
    NestedFields {
        fields: Vec<PanelSection>,
    },
    #[serde(rename = "label")]
    Label {
        text: String,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DropdownOption {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
    pub recommended: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CardOption {
    pub value: String,
    pub label: String,
    pub description: String,
    pub icon: Option<String>,
    pub tags: Vec<String>,
    pub recommended: bool,
    pub available: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextBarConfig {
    pub show_template_selector: bool,
    pub show_override_selector: bool,
    pub allow_per_image_override: bool,
}
