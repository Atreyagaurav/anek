use chrono::Local;
use clap::{Args, ValueHint};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::dtypes::{anekdirtype_iter, AnekDirectory};
use crate::variable;

#[derive(Args)]
pub struct CliArgs {
    /// Filename for the report, without the extension
    #[arg(short, long, default_value = "anek_report", value_hint=ValueHint::FilePath)]
    filename: PathBuf,
    /// Anek Directory Path
    #[arg(default_value = ".", value_hint=ValueHint::DirPath)]
    path: PathBuf,
}

fn capitalize(s: &str) -> String {
    let mut v: Vec<char> = s.chars().collect();
    v[0] = v[0].to_uppercase().nth(0).unwrap();
    v.into_iter().collect()
}

pub fn generate_report(anekdir: AnekDirectory) -> Result<String, String> {
    let mut report = String::new();
    report.push_str("Anek Configuration File\n\n");
    report.push_str(&format!(
        "Generated at: {}\n\n",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    let mut toc = String::new();
    let mut contents = String::new();

    for (i, adt) in anekdirtype_iter().enumerate() {
        let dirname = capitalize(adt.dir_name());
        toc.push_str(&format!("{}. {}\n", i + 1, &dirname));
        contents.push_str(&format!("\n# {}\n", dirname));

        for (j, filename) in variable::list_filenames(&anekdir.get_directory(adt))?
            .iter()
            .enumerate()
        {
            toc.push_str(&format!("   {}.{}. {}\n", i + 1, j + 1, &filename));
            contents.push_str(&format!("## {}\n", &filename));

            let file_contents =
                fs::read_to_string(&anekdir.get_file(adt, &filename)).map_err(|e| e.to_string())?;
            contents.push_str("```\n");
            contents.push_str(&file_contents);
            contents.push_str("```\n");
        }
    }
    report.push_str("# Table of Contents\n");
    report.extend(toc.chars());
    report.extend(contents.chars());
    Ok(report)
}

pub fn save_report(args: CliArgs) -> Result<(), String> {
    let filepath = AnekDirectory::from(&args.path);
    if !filepath.root.exists() {
        return Err(format!(
            "The path {:?} doesn't have anek configuration",
            args.path
        ));
    } else {
        if !filepath.root.is_dir() {
            return Err(format!(
                "{:?} file exists which is not a anek configuration",
                filepath.root
            ));
        } else {
            // make report
            let report_fname = args.filename.with_extension("md");
            eprintln!(
                "Generating report {:?} for {:?}",
                report_fname, filepath.root
            );
            let report_content = generate_report(filepath)?;
            let mut file = fs::File::create(std::env::current_dir().unwrap().join(report_fname))
                .map_err(|e| e.to_string())?;
            file.write_all(report_content.as_bytes())
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
