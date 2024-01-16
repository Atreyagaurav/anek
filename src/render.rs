use anyhow::{Context, Error};
use clap::{Args, ValueHint};
use number_range::NumberRangeOptions;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use string_template_plus::{Render, RenderOptions, Template};

use crate::dtypes::{AnekDirectory, AnekDirectoryType};
use crate::run_utils::{self, variables_from_input};

#[derive(Args, Default)]
pub struct CliArgs {
    /// Render the given template like command templates
    ///
    /// Renders a template. You pass template from the CLI.
    #[arg(short, long, group="action", value_hint = ValueHint::Other)]
    template: bool,
    /// Render the given file like command templates
    ///
    /// Renders a template from file. You pass template file from
    /// outside instead of anek template.
    #[arg(short, long, group="action", value_hint = ValueHint::Other)]
    input_file: bool,
    /// Render the given file like command templates
    ///
    /// Renders a file. Just write the file anywhere in your path with
    /// templates for input variables in the same way you write
    /// command templates. It's useful for markdown files, as the
    /// curly braces syntax won't be used for anything else that
    /// way. Do be careful about that. And the program will replace
    /// those templates with their values when you run it with inputs.
    ///
    /// For batch files, it'll repeat the same contents for each
    /// file. If you want only a portion of the file repeated for all
    /// inputs in a batch file inclose them with lines with `---8<---`
    /// on both start and the end. The lines containing the clip
    /// syntax will be ignored, ideally you can put them in comments.
    ///
    /// You can also use `---file:<filename>[::line_range]` syntax to
    /// include a file, the line_range syntax, if present, should be
    /// in the form of `start[:increment]:end`, you can exclude start
    /// or end to denote the line 1 or last line (e.g. `:5` is 1:5,
    /// and `3:` is from line 3 to the end)
    #[arg(value_hint = ValueHint::FilePath)]
    file: String,

    #[command(subcommand)]
    inputs: run_utils::Inputs,
}

enum RenderFileContentsType {
    Include(PathBuf, String),
    Literal(String),
    Snippet(Template, Option<String>),
}

struct RenderFileContents {
    contents: Vec<RenderFileContentsType>,
}

fn insert_till_now(
    snippet: bool,
    lines: &mut String,
    batch: Option<String>,
    filecontents: &mut RenderFileContents,
) -> Result<(), Error> {
    let p = if snippet {
        RenderFileContentsType::Snippet(Template::parse_template(&lines)?, batch)
    } else {
        RenderFileContentsType::Literal(lines.clone())
    };
    filecontents.contents.push(p);
    lines.clear();
    Ok(())
}

impl RenderFileContents {
    fn read_file(filename: &Path) -> Result<Self, Error> {
        let file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => {
                return Err(Error::msg(format!(
                    "Couldn't open input file: {:?}\n{:?}",
                    filename.to_string_lossy(),
                    e
                )))
            }
        };
        let reader_lines = BufReader::new(file).lines();

        let mut snippet = false;
        let mut filecontents = RenderFileContents {
            contents: Vec::new(),
        };
        let mut lines = String::new();
        let mut batch: Option<String> = None;
        for line in reader_lines {
            let l = line.unwrap();
            if l.contains("---8<---") {
                insert_till_now(snippet, &mut lines, batch.clone(), &mut filecontents)?;
                batch = l.split_once(':').map(|s| s.1.to_string());
                snippet = !snippet;
            } else if l.contains("---file:") {
                if snippet {
                    return Err(Error::msg("Cannot have file in render snippet"));
                }
                insert_till_now(snippet, &mut lines, None, &mut filecontents)?;
                let (_, fname) = l.split_once(':').unwrap();
                let (fname, lines) = fname.split_once("::").unwrap_or((fname, ":"));
                filecontents.contents.push(RenderFileContentsType::Include(
                    PathBuf::from(filename).parent().unwrap().join(fname.trim()),
                    lines.to_string(),
                ))
            } else {
                lines.push_str(&l);
                lines.push('\n');
            }
        }
        if filecontents.contents.is_empty() {
            // if there is no ---8<--- in file, consider the whole
            // file as snippet
            snippet = true;
        }
        if !lines.is_empty() {
            insert_till_now(snippet, &mut lines, batch, &mut filecontents)?;
        }
        Ok(filecontents)
    }

    fn snippet(templ: &str, batch: Option<String>) -> Result<Self, Error> {
        Ok(Self {
            contents: vec![RenderFileContentsType::Snippet(
                Template::parse_template(templ)?,
                batch,
            )],
        })
    }

    fn print_render(
        &self,
        inputs: Vec<HashMap<String, String>>,
        overwrite: &HashMap<String, String>,
    ) -> Result<(), Error> {
        for part in &self.contents {
            match part {
                RenderFileContentsType::Include(filename, lines) => {
                    let file = File::open(&filename)
                        .with_context(|| format!("File {filename:?} not found"))?;
                    let reader_lines: Vec<String> =
                        BufReader::new(file)
                            .lines()
                            .collect::<Result<Vec<String>, std::io::Error>>()?;
                    let lines = NumberRangeOptions::default()
                        .with_default_start(1)
                        .with_default_end(reader_lines.len())
                        .parse(lines)?;
                    for l in lines {
                        println!("{}", reader_lines[l - 1]);
                    }
                }
                RenderFileContentsType::Literal(s) => print!("{}", s),
                RenderFileContentsType::Snippet(templ, batch) => {
                    if let Some(batch) = batch {
                        let ad = AnekDirectory::from(&PathBuf::default())?;
                        let inputs =
                            run_utils::input_files(&ad, &vec![batch.to_string()], &HashSet::new())?;
                        for inp in &inputs {
                            let input = variables_from_input(&inp, &overwrite)?;
                            let renderops = RenderOptions {
                                variables: input,
                                wd: PathBuf::default(),
                                shell_commands: true,
                            };
                            print!("{}", templ.render(&renderops)?);
                        }
                    } else {
                        for input in &inputs {
                            let renderops = RenderOptions {
                                variables: input.clone(),
                                wd: PathBuf::default(),
                                shell_commands: true,
                            };
                            print!("{}", templ.render(&renderops)?);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn run_command(args: CliArgs) -> Result<(), Error> {
    let anek_dir = AnekDirectory::from_pwd()?;
    let template = if args.template {
        RenderFileContents::snippet(&args.file, None)?
    } else if args.input_file {
        RenderFileContents::read_file(args.file.as_ref())?
    } else {
        RenderFileContents::read_file(
            anek_dir
                .get_file(&AnekDirectoryType::Templates, &args.file)
                .as_ref(),
        )?
    };

    let cmd_args = run_utils::command_args(&args.inputs);
    let overwrite: HashMap<String, String> = run_utils::overwrite_vars(&args.inputs, &cmd_args)?;

    let input_files = run_utils::inputs(&anek_dir, &args.inputs, &HashSet::new())?;
    let variables = input_files
        .iter()
        .map(|inp| -> Result<_, Error> { run_utils::variables_from_input(inp, &overwrite) })
        .collect::<Result<Vec<_>, Error>>()?;
    template.print_render(variables, &overwrite)?;
    Ok(())
}
