use std::collections::HashMap;

use photopipeline_core::{ImageId, ImageInfo, NodeId};
use photopipeline_plugin::{ParameterSchema, ParameterSet};
use regex::Regex;

#[derive(Debug)]
pub struct ParameterResolver {
    pub template_params: HashMap<NodeId, ParameterSet>,
    pub group_overrides: Vec<(GroupCondition, HashMap<NodeId, ParameterSet>)>,
    pub image_overrides: HashMap<(ImageId, NodeId), ParameterSet>,
    pub expr_engine: ExpressionEngine,
}

#[derive(Debug, Clone)]
pub enum GroupCondition {
    ExifEq { tag: String, value: String },
    ExifGte { tag: String, value: f64 },
    ExifLte { tag: String, value: f64 },
    GpsNear { lat: f64, lon: f64, radius_km: f64 },
    Always,
    And(Vec<GroupCondition>),
    Or(Vec<GroupCondition>),
    Expression(String),
}

#[derive(Debug, Clone, Default)]
pub struct ExpressionEngine;

impl ExpressionEngine {
    pub fn evaluate(
        &self,
        expr: &str,
        metadata: &photopipeline_core::Metadata,
        image_info: &ImageInfo,
    ) -> Result<serde_json::Value, String> {
        let re = Regex::new(r"\$\{([^}]+)\}").map_err(|e| e.to_string())?;
        let mut result = expr.to_string();

        for cap in re.captures_iter(expr) {
            let inner = &cap[1];
            let full_match = &cap[0];
            let resolved = self.evaluate_inner(inner, metadata, image_info)?;
            result = result.replace(full_match, &resolved);
        }

        Ok(serde_json::Value::String(result))
    }

    fn evaluate_inner(
        &self,
        expr: &str,
        metadata: &photopipeline_core::Metadata,
        image_info: &ImageInfo,
    ) -> Result<String, String> {
        if let Some(ternary_pos) = Self::find_ternary(expr) {
            let cond_str = &expr[..ternary_pos].trim();
            let rest = &expr[ternary_pos + 1..];
            let colon_pos = Self::find_matching_colon(rest)?;
            let true_str = &rest[..colon_pos].trim();
            let false_str = &rest[colon_pos + 1..].trim();

            let cond = self.eval_comparison(cond_str, metadata, image_info)?;
            if cond {
                self.evaluate_inner(true_str, metadata, image_info)
            } else {
                self.evaluate_inner(false_str, metadata, image_info)
            }
        } else if Self::is_comparison(expr) {
            let result = self.eval_comparison(expr, metadata, image_info)?;
            Ok(result.to_string())
        } else {
            self.resolve_variable(expr.trim(), metadata, image_info)
        }
    }

    fn find_ternary(s: &str) -> Option<usize> {
        let mut depth = 0i32;
        for (i, ch) in s.char_indices() {
            match ch {
                '?' if depth == 0 => return Some(i),
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
        }
        None
    }

    fn find_matching_colon(s: &str) -> Result<usize, String> {
        let mut depth = 0i32;
        for (i, ch) in s.char_indices() {
            match ch {
                '?' => depth += 1,
                ':' if depth == 0 => return Ok(i),
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
        }
        Err("unmatched ternary colon".into())
    }

    fn is_comparison(s: &str) -> bool {
        let s = s.trim();
        s.contains(" >= ")
            || s.contains(" <= ")
            || s.contains(" != ")
            || s.contains(" == ")
            || s.contains(" > ")
            || s.contains(" < ")
    }

    fn eval_comparison(
        &self,
        expr: &str,
        metadata: &photopipeline_core::Metadata,
        image_info: &ImageInfo,
    ) -> Result<bool, String> {
        let expr = expr.trim();

        let ops = [">=", "<=", "!=", "==", ">", "<"];
        for op in &ops {
            if let Some(pos) = expr.find(op) {
                let left_str = expr[..pos].trim();
                let right_str = expr[pos + op.len()..].trim();

                let left = self.resolve_variable(left_str, metadata, image_info)?;
                let right = self.resolve_variable(right_str, metadata, image_info)?;

                return Ok(Self::compare_values(&left, &right, op));
            }
        }

        Err(format!("no comparison operator found in '{}'", expr))
    }

    fn compare_values(left: &str, right: &str, op: &str) -> bool {
        let left_num = left.parse::<f64>();
        let right_num = right.parse::<f64>();

        match (left_num, right_num) {
            (Ok(l), Ok(r)) => match op {
                ">=" => l >= r,
                "<=" => l <= r,
                "!=" => (l - r).abs() > f64::EPSILON,
                "==" => (l - r).abs() < f64::EPSILON,
                ">" => l > r,
                "<" => l < r,
                _ => false,
            },
            _ => match op {
                "==" => left == right,
                "!=" => left != right,
                ">" | "<" | ">=" | "<=" => false,
                _ => false,
            },
        }
    }

    fn resolve_variable(
        &self,
        var: &str,
        metadata: &photopipeline_core::Metadata,
        image_info: &ImageInfo,
    ) -> Result<String, String> {
        let var = var.trim();

        if var.starts_with("exif.") {
            self.resolve_exif_var(&var[5..], metadata)
        } else if var.starts_with("image.") {
            self.resolve_image_var(&var[6..], image_info)
        } else if let Ok(n) = var.parse::<f64>() {
            Ok(n.to_string())
        } else if var.starts_with('"') && var.ends_with('"') {
            Ok(var[1..var.len() - 1].to_string())
        } else if var.starts_with('\'') && var.ends_with('\'') {
            Ok(var[1..var.len() - 1].to_string())
        } else {
            Err(format!("unknown variable '{}'", var))
        }
    }

    fn resolve_exif_var(
        &self,
        field: &str,
        metadata: &photopipeline_core::Metadata,
    ) -> Result<String, String> {
        let exif = metadata
            .exif
            .as_ref()
            .ok_or_else(|| "no exif data available".to_string())?;

        match field {
            "iso" => Ok(exif
                .iso
                .map(|v| v.to_string())
                .unwrap_or_else(|| "0".to_string())),
            "aperture" => Ok(exif
                .aperture_value
                .clone()
                .or_else(|| exif.f_number.clone())
                .unwrap_or_else(|| "0".to_string())),
            "shutter" => Ok(exif
                .shutter_speed_value
                .clone()
                .or_else(|| exif.exposure_time.clone())
                .unwrap_or_else(|| "0".to_string())),
            "focal_length" => Ok(exif.focal_length.clone().unwrap_or_else(|| "0".to_string())),
            "make" => Ok(exif.make.clone().unwrap_or_default()),
            "model" => Ok(exif.model.clone().unwrap_or_default()),
            "lens" => Ok(exif.lens_model.clone().unwrap_or_default()),
            _ => Err(format!("unknown exif field '{}'", field)),
        }
    }

    fn resolve_image_var(&self, field: &str, image_info: &ImageInfo) -> Result<String, String> {
        match field {
            "filename" => Ok(image_info.filename.clone()),
            "width" => Ok(image_info.width.to_string()),
            "height" => Ok(image_info.height.to_string()),
            "filesize" => Ok(image_info.file_size_bytes.to_string()),
            _ => Err(format!("unknown image field '{}'", field)),
        }
    }
}

impl ParameterResolver {
    pub fn new() -> Self {
        Self {
            template_params: HashMap::new(),
            group_overrides: Vec::new(),
            image_overrides: HashMap::new(),
            expr_engine: ExpressionEngine,
        }
    }

    pub fn set_template_params(&mut self, node_id: NodeId, params: ParameterSet) {
        self.template_params.insert(node_id, params);
    }

    pub fn add_group_override(
        &mut self,
        condition: GroupCondition,
        params: HashMap<NodeId, ParameterSet>,
    ) {
        self.group_overrides.push((condition, params));
    }

    pub fn set_image_override(&mut self, image_id: ImageId, node_id: NodeId, params: ParameterSet) {
        self.image_overrides.insert((image_id, node_id), params);
    }

    pub fn resolve(
        &self,
        node_id: NodeId,
        image_id: ImageId,
        schema: &ParameterSchema,
        metadata: &photopipeline_core::Metadata,
        image_info: &ImageInfo,
    ) -> ParameterSet {
        let mut result = self.resolve_plugin_defaults(schema);

        if let Some(template_params) = self.template_params.get(&node_id) {
            result.merge(template_params);
        }

        let template_snapshot: std::collections::HashMap<String, serde_json::Value> = schema
            .all_fields()
            .into_iter()
            .filter(|f| !f.allow_override)
            .filter_map(|f| result.values.get(&f.id).map(|v| (f.id.clone(), v.clone())))
            .collect();

        for (condition, node_params) in &self.group_overrides {
            if self.evaluate_condition(condition, metadata, image_info)
                && let Some(group_params) = node_params.get(&node_id)
            {
                result.merge(group_params);
            }
        }

        if let Some(image_params) = self.image_overrides.get(&(image_id, node_id)) {
            result.merge(image_params);
        }

        for (key, value) in template_snapshot {
            result.values.insert(key, value);
        }

        self.resolve_expressions(&mut result, metadata, image_info);

        result
    }

    pub fn resolve_single(&self, node_id: NodeId, schema: &ParameterSchema) -> ParameterSet {
        let mut result = self.resolve_plugin_defaults(schema);
        if let Some(template_params) = self.template_params.get(&node_id) {
            result.merge(template_params);
        }
        result
    }

    fn resolve_plugin_defaults(&self, schema: &ParameterSchema) -> ParameterSet {
        schema.defaults()
    }

    fn evaluate_condition(
        &self,
        condition: &GroupCondition,
        metadata: &photopipeline_core::Metadata,
        image_info: &ImageInfo,
    ) -> bool {
        match condition {
            GroupCondition::Always => true,
            GroupCondition::And(conditions) => conditions
                .iter()
                .all(|c| self.evaluate_condition(c, metadata, image_info)),
            GroupCondition::Or(conditions) => conditions
                .iter()
                .any(|c| self.evaluate_condition(c, metadata, image_info)),
            GroupCondition::ExifEq { tag, value } => {
                let exif = match &metadata.exif {
                    Some(e) => e,
                    None => return false,
                };
                let tag_value = Self::get_exif_tag(exif, tag);
                tag_value.as_deref() == Some(value.as_str())
            }
            GroupCondition::ExifGte { tag, value } => {
                let exif = match &metadata.exif {
                    Some(e) => e,
                    None => return false,
                };
                let tag_value = Self::get_exif_tag(exif, tag);
                tag_value
                    .and_then(|v| v.parse::<f64>().ok())
                    .map(|v| v >= *value)
                    .unwrap_or(false)
            }
            GroupCondition::ExifLte { tag, value } => {
                let exif = match &metadata.exif {
                    Some(e) => e,
                    None => return false,
                };
                let tag_value = Self::get_exif_tag(exif, tag);
                tag_value
                    .and_then(|v| v.parse::<f64>().ok())
                    .map(|v| v <= *value)
                    .unwrap_or(false)
            }
            GroupCondition::GpsNear {
                lat,
                lon,
                radius_km,
            } => {
                let gps = match &metadata.gps {
                    Some(g) => g,
                    None => return false,
                };
                let (gps_lat, gps_lon) = match gps.coordinate_tuple() {
                    Some(c) => c,
                    None => return false,
                };
                Self::haversine_km(gps_lat, gps_lon, *lat, *lon) <= *radius_km
            }
            GroupCondition::Expression(expr) => self
                .expr_engine
                .evaluate(expr, metadata, image_info)
                .map(|v| {
                    v.as_bool()
                        .or_else(|| v.as_str().and_then(|s| s.parse::<bool>().ok()))
                        .unwrap_or(false)
                })
                .unwrap_or(false),
        }
    }

    fn get_exif_tag(exif: &photopipeline_core::ExifData, tag: &str) -> Option<String> {
        match tag {
            "iso" => exif.iso.map(|v| v.to_string()),
            "make" => exif.make.clone(),
            "model" => exif.model.clone(),
            "lens" => exif.lens_model.clone(),
            "focal_length" => exif.focal_length.clone(),
            "aperture" => exif
                .aperture_value
                .clone()
                .or_else(|| exif.f_number.clone()),
            "shutter" => exif
                .shutter_speed_value
                .clone()
                .or_else(|| exif.exposure_time.clone()),
            _ => None,
        }
    }

    fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let r = 6371.0;
        let dlat = (lat2 - lat1).to_radians();
        let dlon = (lon2 - lon1).to_radians();
        let a = (dlat / 2.0).sin().powi(2)
            + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        r * c
    }

    fn resolve_expressions(
        &self,
        params: &mut ParameterSet,
        metadata: &photopipeline_core::Metadata,
        image_info: &ImageInfo,
    ) {
        let keys: Vec<String> = params.values.keys().cloned().collect();
        for key in keys {
            if let Some(serde_json::Value::String(expr_str)) = params.values.get(&key)
                && expr_str.contains("${")
            {
                if let Ok(value) = self.expr_engine.evaluate(expr_str, metadata, image_info) {
                    params.values.insert(key, value);
                }
            }
        }
    }
}

impl Default for ParameterResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use photopipeline_core::{
        ColorSpace, ExifData, ImageFormat, IntegerWidget, Metadata, PixelFormat,
    };
    use photopipeline_plugin::{ParameterField, ParameterSchema, ParameterSection, ParameterType};
    use uuid::Uuid;

    fn make_simple_schema() -> ParameterSchema {
        ParameterSchema {
            version: 1,
            sections: vec![ParameterSection {
                id: "main".into(),
                label: "Main".into(),
                description: None,
                icon: None,
                collapsible: false,
                default_collapsed: false,
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
                    advanced: false,
                    allow_override: true,
                    supports_expression: false,
                }],
            }],
        }
    }

    fn make_test_metadata(iso: u32) -> Metadata {
        Metadata {
            exif: Some(ExifData {
                iso: Some(iso),
                make: Some("Canon".into()),
                model: Some("EOS R5".into()),
                lens_model: Some("24-70mm".into()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn make_test_image_info() -> ImageInfo {
        ImageInfo {
            id: Uuid::new_v4(),
            path: "/tmp/test.jpg".into(),
            filename: "test.jpg".into(),
            format: ImageFormat::JPEG,
            width: 1920,
            height: 1080,
            file_size_bytes: 102400,
            pixel_format: PixelFormat::U8,
            color_space: ColorSpace::default(),
        }
    }

    #[test]
    fn parameter_resolver_default_is_empty() {
        let resolver = ParameterResolver::new();
        assert!(resolver.template_params.is_empty());
        assert!(resolver.group_overrides.is_empty());
        assert!(resolver.image_overrides.is_empty());
    }

    #[test]
    fn parameter_resolver_default_trait() {
        let resolver = ParameterResolver::default();
        assert!(resolver.template_params.is_empty());
    }

    #[test]
    fn resolve_uses_plugin_defaults() {
        let resolver = ParameterResolver::new();
        let schema = make_simple_schema();
        let result = resolver.resolve_single(Uuid::new_v4(), &schema);
        assert_eq!(result.get_i64("threshold"), Some(128));
    }

    #[test]
    fn resolve_merges_template_over_defaults() {
        let mut resolver = ParameterResolver::new();
        let schema = make_simple_schema();
        let node_id = Uuid::new_v4();

        let mut template_params = ParameterSet::new();
        template_params.insert("threshold".into(), serde_json::json!(200));
        resolver.set_template_params(node_id, template_params);

        let result = resolver.resolve_single(node_id, &schema);
        assert_eq!(result.get_i64("threshold"), Some(200));
    }

    #[test]
    fn resolve_with_image_override() {
        let mut resolver = ParameterResolver::new();
        let schema = make_simple_schema();
        let node_id = Uuid::new_v4();
        let image_id = Uuid::new_v4();

        let mut override_params = ParameterSet::new();
        override_params.insert("threshold".into(), serde_json::json!(50));
        resolver.set_image_override(image_id, node_id, override_params);

        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
        assert_eq!(result.get_i64("threshold"), Some(50));
    }

    #[test]
    fn condition_always_true() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        assert!(resolver.evaluate_condition(&GroupCondition::Always, &metadata, &image_info));
    }

    #[test]
    fn condition_exif_eq_match() {
        let resolver = ParameterResolver::new();
        let metadata = make_test_metadata(800);
        let image_info = make_test_image_info();
        let cond = GroupCondition::ExifEq {
            tag: "make".into(),
            value: "Canon".into(),
        };
        assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_exif_eq_no_match() {
        let resolver = ParameterResolver::new();
        let metadata = make_test_metadata(800);
        let image_info = make_test_image_info();
        let cond = GroupCondition::ExifEq {
            tag: "make".into(),
            value: "Nikon".into(),
        };
        assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_exif_gte_match() {
        let resolver = ParameterResolver::new();
        let metadata = make_test_metadata(800);
        let image_info = make_test_image_info();
        let cond = GroupCondition::ExifGte {
            tag: "iso".into(),
            value: 400.0,
        };
        assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_exif_gte_below() {
        let resolver = ParameterResolver::new();
        let metadata = make_test_metadata(100);
        let image_info = make_test_image_info();
        let cond = GroupCondition::ExifGte {
            tag: "iso".into(),
            value: 400.0,
        };
        assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_exif_lte_match() {
        let resolver = ParameterResolver::new();
        let metadata = make_test_metadata(200);
        let image_info = make_test_image_info();
        let cond = GroupCondition::ExifLte {
            tag: "iso".into(),
            value: 400.0,
        };
        assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_exif_lte_above() {
        let resolver = ParameterResolver::new();
        let metadata = make_test_metadata(800);
        let image_info = make_test_image_info();
        let cond = GroupCondition::ExifLte {
            tag: "iso".into(),
            value: 400.0,
        };
        assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_exif_no_exif_data() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let cond = GroupCondition::ExifGte {
            tag: "iso".into(),
            value: 100.0,
        };
        assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_gps_near_within_radius() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata {
            gps: Some(photopipeline_core::GpsData {
                latitude: Some(34.0522),
                longitude: Some(-118.2437),
                ..Default::default()
            }),
            ..Default::default()
        };
        let image_info = make_test_image_info();
        let cond = GroupCondition::GpsNear {
            lat: 34.05,
            lon: -118.24,
            radius_km: 10.0,
        };
        assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_gps_near_outside_radius() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata {
            gps: Some(photopipeline_core::GpsData {
                latitude: Some(34.0522),
                longitude: Some(-118.2437),
                ..Default::default()
            }),
            ..Default::default()
        };
        let image_info = make_test_image_info();
        let cond = GroupCondition::GpsNear {
            lat: 34.05,
            lon: -118.24,
            radius_km: 0.001,
        };
        assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_gps_near_no_gps_data() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let cond = GroupCondition::GpsNear {
            lat: 34.0,
            lon: -118.0,
            radius_km: 10.0,
        };
        assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_and_all_true() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let cond = GroupCondition::And(vec![GroupCondition::Always, GroupCondition::Always]);
        assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_and_one_false() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let cond = GroupCondition::And(vec![
            GroupCondition::Always,
            GroupCondition::ExifEq {
                tag: "make".into(),
                value: "Canon".into(),
            },
        ]);
        assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_or_one_true() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let cond = GroupCondition::Or(vec![
            GroupCondition::Always,
            GroupCondition::ExifEq {
                tag: "make".into(),
                value: "Canon".into(),
            },
        ]);
        assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_or_all_false() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let cond = GroupCondition::Or(vec![
            GroupCondition::ExifEq {
                tag: "make".into(),
                value: "Canon".into(),
            },
            GroupCondition::ExifEq {
                tag: "make".into(),
                value: "Nikon".into(),
            },
        ]);
        assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn expression_engine_simple_variable() {
        let engine = ExpressionEngine::default();
        let metadata = make_test_metadata(400);
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("${exif.make}", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::json!("Canon"));
    }

    #[test]
    fn expression_engine_image_var() {
        let engine = ExpressionEngine::default();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("${image.width}", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("1920".into()));
    }

    #[test]
    fn expression_engine_comparison_eq_true() {
        let engine = ExpressionEngine::default();
        let metadata = make_test_metadata(400);
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("${exif.iso == 400}", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("true".into()));
    }

    #[test]
    fn expression_engine_comparison_gt_true() {
        let engine = ExpressionEngine::default();
        let metadata = make_test_metadata(800);
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("${exif.iso > 400}", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("true".into()));
    }

    #[test]
    fn expression_engine_comparison_lte() {
        let engine = ExpressionEngine::default();
        let metadata = make_test_metadata(400);
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("${exif.iso <= 400}", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("true".into()));
    }

    #[test]
    fn expression_engine_ternary_true() {
        let engine = ExpressionEngine::default();
        let metadata = make_test_metadata(800);
        let image_info = make_test_image_info();
        let result = engine
            .evaluate(
                "${exif.iso >= 400 ? 'high' : 'low'}",
                &metadata,
                &image_info,
            )
            .unwrap();
        assert_eq!(result, serde_json::Value::String("high".into()));
    }

    #[test]
    fn expression_engine_ternary_false() {
        let engine = ExpressionEngine::default();
        let metadata = make_test_metadata(100);
        let image_info = make_test_image_info();
        let result = engine
            .evaluate(
                "${exif.iso >= 400 ? 'high' : 'low'}",
                &metadata,
                &image_info,
            )
            .unwrap();
        assert_eq!(result, serde_json::Value::String("low".into()));
    }

    #[test]
    fn expression_engine_literal_number() {
        let engine = ExpressionEngine::default();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let result = engine.evaluate("${3.14}", &metadata, &image_info).unwrap();
        assert_eq!(result, serde_json::Value::String("3.14".into()));
    }

    #[test]
    fn expression_engine_quoted_string() {
        let engine = ExpressionEngine::default();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("${\"hello\"}", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("hello".into()));
    }

    #[test]
    fn expression_engine_string_eq() {
        let engine = ExpressionEngine::default();
        let metadata = make_test_metadata(400);
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("${exif.make == \"Canon\"}", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("true".into()));
    }

    #[test]
    fn expression_engine_string_neq() {
        let engine = ExpressionEngine::default();
        let metadata = make_test_metadata(400);
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("${exif.make != \"Nikon\"}", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("true".into()));
    }

    #[test]
    fn expression_engine_unknown_var_errors() {
        let engine = ExpressionEngine::default();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        assert!(
            engine
                .evaluate("${unknown.var}", &metadata, &image_info)
                .is_err()
        );
    }

    #[test]
    fn condition_expression_true() {
        let resolver = ParameterResolver::new();
        let metadata = make_test_metadata(800);
        let image_info = make_test_image_info();
        let cond = GroupCondition::Expression("${exif.iso > 400}".into());
        assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_expression_false() {
        let resolver = ParameterResolver::new();
        let metadata = make_test_metadata(100);
        let image_info = make_test_image_info();
        let cond = GroupCondition::Expression("${exif.iso > 400}".into());
        assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn resolve_with_group_override_match() {
        let mut resolver = ParameterResolver::new();
        let schema = make_simple_schema();
        let node_id = Uuid::new_v4();
        let image_id = Uuid::new_v4();

        let mut group_params = ParameterSet::new();
        group_params.insert("threshold".into(), serde_json::json!(255));
        let mut node_map = std::collections::HashMap::new();
        node_map.insert(node_id, group_params);
        resolver.add_group_override(
            GroupCondition::ExifEq {
                tag: "make".into(),
                value: "Canon".into(),
            },
            node_map,
        );

        let metadata = make_test_metadata(400);
        let image_info = make_test_image_info();
        let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
        assert_eq!(result.get_i64("threshold"), Some(255));
    }

    #[test]
    fn resolve_group_override_no_match() {
        let mut resolver = ParameterResolver::new();
        let schema = make_simple_schema();
        let node_id = Uuid::new_v4();
        let image_id = Uuid::new_v4();

        let mut group_params = ParameterSet::new();
        group_params.insert("threshold".into(), serde_json::json!(255));
        let mut node_map = std::collections::HashMap::new();
        node_map.insert(node_id, group_params);
        resolver.add_group_override(
            GroupCondition::ExifEq {
                tag: "make".into(),
                value: "Nikon".into(),
            },
            node_map,
        );

        let metadata = make_test_metadata(400);
        let image_info = make_test_image_info();
        let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
        assert_eq!(result.get_i64("threshold"), Some(128));
    }

    #[test]
    fn resolve_priority_image_overrides_all() {
        let mut resolver = ParameterResolver::new();
        let schema = make_simple_schema();
        let node_id = Uuid::new_v4();
        let image_id = Uuid::new_v4();

        let mut template_params = ParameterSet::new();
        template_params.insert("threshold".into(), serde_json::json!(200));
        resolver.set_template_params(node_id, template_params);

        let mut override_params = ParameterSet::new();
        override_params.insert("threshold".into(), serde_json::json!(1));
        resolver.set_image_override(image_id, node_id, override_params);

        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
        assert_eq!(result.get_i64("threshold"), Some(1));
    }

    #[test]
    fn resolve_fallthrough_missing_image_override_uses_group() {
        let mut resolver = ParameterResolver::new();
        let schema = make_simple_schema();
        let node_id = Uuid::new_v4();
        let image_id = Uuid::new_v4();

        let mut group_params = ParameterSet::new();
        group_params.insert("threshold".into(), serde_json::json!(77));
        let mut node_map = std::collections::HashMap::new();
        node_map.insert(node_id, group_params);
        resolver.add_group_override(GroupCondition::Always, node_map);

        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
        assert_eq!(result.get_i64("threshold"), Some(77));
    }

    #[test]
    fn resolve_fallthrough_missing_group_uses_template() {
        let mut resolver = ParameterResolver::new();
        let schema = make_simple_schema();
        let node_id = Uuid::new_v4();
        let image_id = Uuid::new_v4();

        let mut template_params = ParameterSet::new();
        template_params.insert("threshold".into(), serde_json::json!(42));
        resolver.set_template_params(node_id, template_params);

        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
        assert_eq!(result.get_i64("threshold"), Some(42));
    }

    #[test]
    fn resolve_missing_all_uses_plugin_default() {
        let resolver = ParameterResolver::new();
        let schema = make_simple_schema();
        let node_id = Uuid::new_v4();
        let image_id = Uuid::new_v4();

        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
        assert_eq!(result.get_i64("threshold"), Some(128));
    }

    #[test]
    fn resolve_with_all_four_levels() {
        let mut resolver = ParameterResolver::new();
        let schema = make_simple_schema();
        let node_id = Uuid::new_v4();
        let image_id = Uuid::new_v4();

        let mut template_params = ParameterSet::new();
        template_params.insert("threshold".into(), serde_json::json!(200));
        resolver.set_template_params(node_id, template_params);

        let mut group_params = ParameterSet::new();
        group_params.insert("threshold".into(), serde_json::json!(150));
        let mut node_map = std::collections::HashMap::new();
        node_map.insert(node_id, group_params);
        resolver.add_group_override(GroupCondition::Always, node_map);

        let mut override_params = ParameterSet::new();
        override_params.insert("threshold".into(), serde_json::json!(50));
        resolver.set_image_override(image_id, node_id, override_params);

        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let result = resolver.resolve(node_id, image_id, &schema, &metadata, &image_info);
        assert_eq!(result.get_i64("threshold"), Some(50));
    }

    #[test]
    fn condition_and_empty_returns_true() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        assert!(resolver.evaluate_condition(&GroupCondition::And(vec![]), &metadata, &image_info));
    }

    #[test]
    fn condition_or_empty_returns_false() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        assert!(!resolver.evaluate_condition(&GroupCondition::Or(vec![]), &metadata, &image_info));
    }

    #[test]
    fn condition_and_all_false() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let cond = GroupCondition::And(vec![
            GroupCondition::ExifEq {
                tag: "make".into(),
                value: "Canon".into(),
            },
            GroupCondition::ExifEq {
                tag: "make".into(),
                value: "Nikon".into(),
            },
        ]);
        assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_gps_near_at_radius_boundary() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata {
            gps: Some(photopipeline_core::GpsData {
                latitude: Some(48.8566),
                longitude: Some(2.3522),
                ..Default::default()
            }),
            ..Default::default()
        };
        let image_info = make_test_image_info();
        let cond = GroupCondition::GpsNear {
            lat: 51.5074,
            lon: -0.1278,
            radius_km: 400.0,
        };
        assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_gps_near_london_paris() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata {
            gps: Some(photopipeline_core::GpsData {
                latitude: Some(48.8566),
                longitude: Some(2.3522),
                ..Default::default()
            }),
            ..Default::default()
        };
        let image_info = make_test_image_info();
        let cond = GroupCondition::GpsNear {
            lat: 51.5074,
            lon: -0.1278,
            radius_km: 350.0,
        };
        assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_gps_near_nyc_la() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata {
            gps: Some(photopipeline_core::GpsData {
                latitude: Some(40.7128),
                longitude: Some(-74.0060),
                ..Default::default()
            }),
            ..Default::default()
        };
        let image_info = make_test_image_info();
        let cond = GroupCondition::GpsNear {
            lat: 34.0522,
            lon: -118.2437,
            radius_km: 4000.0,
        };
        assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_exif_gte_exact_boundary() {
        let resolver = ParameterResolver::new();
        let metadata = make_test_metadata(400);
        let image_info = make_test_image_info();
        let cond = GroupCondition::ExifGte {
            tag: "iso".into(),
            value: 400.0,
        };
        assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_exif_lte_exact_boundary() {
        let resolver = ParameterResolver::new();
        let metadata = make_test_metadata(400);
        let image_info = make_test_image_info();
        let cond = GroupCondition::ExifLte {
            tag: "iso".into(),
            value: 400.0,
        };
        assert!(resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_exif_eq_tag_not_found() {
        let resolver = ParameterResolver::new();
        let metadata = make_test_metadata(400);
        let image_info = make_test_image_info();
        let cond = GroupCondition::ExifEq {
            tag: "unknown_tag".into(),
            value: "x".into(),
        };
        assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_exif_gte_no_exif() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let cond = GroupCondition::ExifGte {
            tag: "iso".into(),
            value: 100.0,
        };
        assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn condition_exif_lte_no_exif() {
        let resolver = ParameterResolver::new();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let cond = GroupCondition::ExifLte {
            tag: "iso".into(),
            value: 6400.0,
        };
        assert!(!resolver.evaluate_condition(&cond, &metadata, &image_info));
    }

    #[test]
    fn expression_engine_multiple_substitutions() {
        let engine = ExpressionEngine::default();
        let metadata = make_test_metadata(400);
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("${exif.make} ${exif.model}", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("Canon EOS R5".into()));
    }

    #[test]
    fn expression_engine_numeric_literal_comparison() {
        let engine = ExpressionEngine::default();
        let metadata = make_test_metadata(800);
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("${exif.iso > 400 ? 100 : 200}", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("100".into()));
    }

    #[test]
    fn expression_engine_no_dollar_brace_returns_literal() {
        let engine = ExpressionEngine::default();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("plain text", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("plain text".into()));
    }

    #[test]
    fn expression_engine_malformed_missing_close_brace() {
        let engine = ExpressionEngine::default();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let result = engine.evaluate("${exif.make", &metadata, &image_info);
        assert!(result.is_ok());
    }

    #[test]
    fn expression_engine_string_ne() {
        let engine = ExpressionEngine::default();
        let metadata = make_test_metadata(400);
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("${exif.make != \"Sony\"}", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("true".into()));
    }

    #[test]
    fn expression_engine_comparison_lt() {
        let engine = ExpressionEngine::default();
        let metadata = make_test_metadata(100);
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("${exif.iso < 400}", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("true".into()));
    }

    #[test]
    fn expression_engine_comparison_ge() {
        let engine = ExpressionEngine::default();
        let metadata = make_test_metadata(400);
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("${exif.iso >= 400}", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("true".into()));
    }

    #[test]
    fn expression_engine_single_quoted_string() {
        let engine = ExpressionEngine::default();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("${'hello world'}", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("hello world".into()));
    }

    #[test]
    fn expression_engine_unknown_image_field() {
        let engine = ExpressionEngine::default();
        let metadata = Metadata::default();
        let image_info = make_test_image_info();
        assert!(
            engine
                .evaluate("${image.unknown}", &metadata, &image_info)
                .is_err()
        );
    }

    #[test]
    fn expression_engine_unknown_exif_field() {
        let engine = ExpressionEngine::default();
        let metadata = make_test_metadata(100);
        let image_info = make_test_image_info();
        assert!(
            engine
                .evaluate("${exif.unknown}", &metadata, &image_info)
                .is_err()
        );
    }

    #[test]
    fn expression_engine_exif_lens() {
        let engine = ExpressionEngine::default();
        let metadata = make_test_metadata(100);
        let image_info = make_test_image_info();
        let result = engine
            .evaluate("${exif.lens}", &metadata, &image_info)
            .unwrap();
        assert_eq!(result, serde_json::Value::String("24-70mm".into()));
    }

    #[test]
    fn haversine_same_point_zero() {
        let dist = ParameterResolver::haversine_km(0.0, 0.0, 0.0, 0.0);
        assert!((dist - 0.0).abs() < 0.001);
    }

    #[test]
    fn haversine_london_to_paris_approx() {
        let dist = ParameterResolver::haversine_km(51.5074, -0.1278, 48.8566, 2.3522);
        assert!(dist > 330.0 && dist < 350.0);
    }
}
