use anyhow::{Context, Error};
use clap::{ArgGroup, Args, ValueHint};
use colored::Colorize;
use number_range::NumberRangeOptions;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use string_template_plus::Template;
use subprocess::Exec;

use crate::dtypes::{AnekDirectory, AnekDirectoryType};
use crate::export;
use crate::variable;
use itertools::Itertools;

fn csv_head_template(vars: &Vec<String>) -> String {
    format!(
        "{}\n",
        vars.iter().map(|v| v.trim_end_matches("?")).join(",")
    )
}

fn csv_body_template(vars: &Vec<String>) -> String {
    format!("{{{}}}", vars.join("},{"))
}

fn json_body_template(vars: &Vec<String>) -> String {
    vars.iter()
        .map(|v| format!("\"{}\":\"{{{}}}\"", v.trim_end_matches("?"), v))
        .collect::<Vec<String>>()
        .join(",")
}

fn plain_body_template(vars: &Vec<String>) -> String {
    vars.iter()
        .map(|v| format!("{}={{{}}}", v.trim_end_matches("?"), v))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn pre_post_templates<'a>(
    vars: &Vec<String>,
    format: &'a str,
) -> (String, &'a str, &'a str, &'a str, &'a str) {
    match format {
        // pre_everything, pre_line, post_line, between_lines, post_everything
        "csv" => (csv_head_template(vars), "", "", "", ""),
        "json" => ("[\n".to_string(), "  {", "}", ",", "]\n"),
        &_ => ("".to_string(), "", "", "", ""),
    }
}

pub fn body_template(vars: &Vec<String>, format: &str) -> String {
    match format {
        "csv" => csv_body_template(vars),
        "plain" => plain_body_template(vars),
        "json" => json_body_template(vars),
        &_ => "".to_string(),
    }
}
