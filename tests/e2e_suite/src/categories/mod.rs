mod cat01_single_plugin;
mod cat02_param_variants;
mod cat03_two_plugin_chain;
mod cat04_three_plugin_chain;
mod cat05_four_plugin_workflow;
mod cat06_pipeline_topology;
mod cat07_format_structure;
mod cat08_param_boundary;
mod cat09_error_paths;
mod cat10_quality_content;
mod cat11_metadata_preservation;
mod cat12_cli_commands;
mod cat13_roundtrip;
mod cat14_known_regressions;

use crate::common::TestCaseSpec;

pub fn load(category: &str) -> Option<Vec<TestCaseSpec>> {
    match category {
        "cat01" => Some(cat01_single_plugin::specs()),
        "cat02" => Some(cat02_param_variants::specs()),
        "cat03" => Some(cat03_two_plugin_chain::specs()),
        "cat04" => Some(cat04_three_plugin_chain::specs()),
        "cat05" => Some(cat05_four_plugin_workflow::specs()),
        "cat06" => Some(cat06_pipeline_topology::specs()),
        "cat07" => Some(cat07_format_structure::specs()),
        "cat08" => Some(cat08_param_boundary::specs()),
        "cat09" => Some(cat09_error_paths::specs()),
        "cat10" => Some(cat10_quality_content::specs()),
        "cat11" => Some(cat11_metadata_preservation::specs()),
        "cat12" => Some(cat12_cli_commands::specs()),
        "cat13" => Some(cat13_roundtrip::specs()),
        "cat14" => Some(cat14_known_regressions::specs()),
        _ => None,
    }
}
