use itertools::Itertools;

fn csv_head_template(vars: &Vec<String>) -> String {
    vars.iter().map(|v| v.trim_end_matches("?")).join(",")
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
        .map(|v| format!("{0}={{{0}}}", v))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn pre_post_templates<'a>(
    vars: &Vec<String>,
    format: &'a str,
) -> (String, &'a str, &'a str, &'a str, &'a str) {
    match format {
        "csv" => (csv_head_template(vars), "", "", "", ""),
        "json" => ("[".to_string(), "  {", "}", ",", "]"),
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
