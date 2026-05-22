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
    ExifEq {
        tag: String,
        value: String,
    },
    ExifGte {
        tag: String,
        value: f64,
    },
    ExifLte {
        tag: String,
        value: f64,
    },
    GpsNear {
        lat: f64,
        lon: f64,
        radius_km: f64,
    },
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
            "focal_length" => Ok(exif
                .focal_length
                .clone()
                .unwrap_or_else(|| "0".to_string())),
            "make" => Ok(exif
                .make
                .clone()
                .unwrap_or_default()),
            "model" => Ok(exif
                .model
                .clone()
                .unwrap_or_default()),
            "lens" => Ok(exif
                .lens_model
                .clone()
                .unwrap_or_default()),
            _ => Err(format!("unknown exif field '{}'", field)),
        }
    }

    fn resolve_image_var(
        &self,
        field: &str,
        image_info: &ImageInfo,
    ) -> Result<String, String> {
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
            expr_engine: ExpressionEngine::default(),
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

    pub fn set_image_override(
        &mut self,
        image_id: ImageId,
        node_id: NodeId,
        params: ParameterSet,
    ) {
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

        for (condition, node_params) in &self.group_overrides {
            if self.evaluate_condition(condition, metadata, image_info) {
                if let Some(group_params) = node_params.get(&node_id) {
                    result.merge(group_params);
                }
            }
        }

        if let Some(image_params) = self.image_overrides.get(&(image_id, node_id)) {
            result.merge(image_params);
        }

        self.resolve_expressions(&mut result, metadata, image_info);

        result
    }

    pub fn resolve_single(
        &self,
        node_id: NodeId,
        schema: &ParameterSchema,
    ) -> ParameterSet {
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
                .map(|v| v.as_bool().unwrap_or(false))
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
            + lat1.to_radians().cos()
                * lat2.to_radians().cos()
                * (dlon / 2.0).sin().powi(2);
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
            if let Some(serde_json::Value::String(expr_str)) = params.values.get(&key) {
                if expr_str.contains("${") {
                    match self.expr_engine.evaluate(expr_str, metadata, image_info) {
                        Ok(value) => {
                            params.values.insert(key, value);
                        }
                        Err(_) => {}
                    }
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
