fn csv_body_template(vars: &str) -> String {
    format!("{{{}}}", vars.replace(",", "},{"))
}

fn json_body_template(vars: &str) -> String {
    vars.split(",")
        .map(|v| format!("\"{0}\":\"{{{0}}}\"", v))
        .collect::<Vec<String>>()
        .join(",")
}

fn plain_body_template(vars: &str) -> String {
    vars.split(",")
        .map(|v| format!("\"{0}\"=\"{{{0}}}\"", v))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn pre_post_templates<'a>(
    vars: &'a str,
    format: &'a str,
) -> (&'a str, &'a str, &'a str, &'a str, &'a str) {
    match format {
        "csv" => (vars, "", "", "", ""),
        "json" => ("[", "  {", "}", ",", "]"),
        &_ => ("", "", "", "", ""),
    }
}

pub fn body_template(vars: &str, format: &str) -> String {
    match format {
        "csv" => csv_body_template(vars),
        "plain" => plain_body_template(vars),
        "json" => json_body_template(vars),
        &_ => "".to_string(),
    }
}
