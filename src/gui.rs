use anyhow::Error;
use chrono::Local;
use clap::{Args, ValueHint};
use colored::Colorize;
use iced::widget::{
    button, checkbox, column, container, mouse_area, operation::focus, pick_list, row, scrollable,
    space::horizontal, text, text::Rich, text_editor, text_input, tooltip,
};
use iced::Element;
use iced::{Color, Length, Task};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use crate::dtypes::{AnekDirectory, AnekDirectoryType};
use crate::run_utils;

#[derive(Args)]
pub struct CliArgs {
    #[arg(default_value = ".", value_hint = ValueHint::DirPath)]
    path: PathBuf,
}

pub fn run(args: CliArgs) -> Result<(), Error> {
    Ok(iced::application(State::new, State::update, State::view)
        .title("TypDoc - A Typst Based Document Format")
        .run()?)
}

struct AnekConfig {
    dir: AnekDirectory,
    variables: Vec<String>,
    inputs: Vec<String>,
    batch: Vec<String>,
    commands: Vec<String>,
    pipelines: Vec<String>,
    loops: Vec<String>,
}

impl AnekConfig {
    fn new() -> Result<Self, Error> {
        let dir = AnekDirectory::from(&PathBuf::from("."))?;
        let list = |dt| crate::variable::list_anek_filenames(&dir.get_directory(dt));
        Ok(Self {
            variables: list(&AnekDirectoryType::Variables)?,
            inputs: list(&AnekDirectoryType::Inputs)?,
            batch: list(&AnekDirectoryType::Batch)?,
            commands: list(&AnekDirectoryType::Commands)?,
            pipelines: list(&AnekDirectoryType::Pipelines)?,
            loops: list(&AnekDirectoryType::Loops)?,
            dir: dir,
        })
    }

    fn at(path: &PathBuf) -> Result<Self, Error> {
        let dir = AnekDirectory::from(path)?;
        let list = |dt| crate::variable::list_anek_filenames(&dir.get_directory(dt));
        Ok(Self {
            variables: list(&AnekDirectoryType::Variables)?,
            inputs: list(&AnekDirectoryType::Inputs)?,
            batch: list(&AnekDirectoryType::Batch)?,
            commands: list(&AnekDirectoryType::Commands)?,
            pipelines: list(&AnekDirectoryType::Pipelines)?,
            loops: list(&AnekDirectoryType::Loops)?,
            dir: dir,
        })
    }

    fn list<'a>(&'a self, ty: &AnekDirectoryType) -> &'a [String] {
        match ty {
            AnekDirectoryType::Variables => self.variables.as_slice(),
            AnekDirectoryType::Inputs => self.inputs.as_slice(),
            AnekDirectoryType::Batch => self.batch.as_slice(),
            AnekDirectoryType::Commands => self.commands.as_slice(),
            AnekDirectoryType::Pipelines => self.pipelines.as_slice(),
            AnekDirectoryType::Loops => self.loops.as_slice(),
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
enum AnekInputType {
    #[default]
    Inputs,
    Batch,
    Loops,
}

impl std::fmt::Display for AnekInputType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AnekInputType::Inputs => write!(f, "Inputs"),
            AnekInputType::Batch => write!(f, "Batch"),
            AnekInputType::Loops => write!(f, "Loops"),
        }
    }
}

impl AnekInputType {
    fn all() -> &'static [Self] {
        &[
            AnekInputType::Inputs,
            AnekInputType::Batch,
            AnekInputType::Loops,
        ]
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
enum AnekTaskType {
    #[default]
    Command,
    Pipeline,
}

impl std::fmt::Display for AnekTaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AnekTaskType::Command => write!(f, "Command"),
            AnekTaskType::Pipeline => write!(f, "Pipeline"),
        }
    }
}

impl AnekTaskType {
    fn all() -> &'static [Self] {
        &[AnekTaskType::Command, AnekTaskType::Pipeline]
    }
}

struct State {
    config: AnekConfig,
    input_type: AnekInputType,
    input: Option<String>,
    subset: String,
    _parallel: bool,
    task_type: AnekTaskType,
    task: Option<String>,
    demo: bool,
    editing: bool,
    dirty: bool,
    dir_type: crate::dtypes::AnekDirectoryType,
    filename: Option<String>,
    content: text_editor::Content,
}

impl State {
    fn run(&self) -> Result<(), Error> {
        let select_inputs = run_utils::filter_index(&self.subset).unwrap();
        let input = self.input.clone().unwrap();
        let (batch, input, r#loop) = match &self.input_type {
            AnekInputType::Inputs => (vec![], vec![input], None),
            AnekInputType::Batch => (vec![input], vec![], None),
            AnekInputType::Loops => (vec![], vec![], Some(input)),
        };
        let inputs = crate::run_utils::Inputs::On(crate::run_utils::InputsArgs {
            select_inputs,
            batch,
            input,
            r#loop,
            overwrite: vec![],
            command_args: vec![],
        });

        let task = self.task.clone().unwrap();
        let commands = match &self.task_type {
            AnekTaskType::Pipeline => run_utils::cmd_from_pipeline(&self.config.dir, &task)?,
            AnekTaskType::Command => {
                vec![self.config.dir.command(&task)?]
            }
        };
        let mut variables: HashSet<&str> = HashSet::new();
        for cmd in &commands {
            cmd.template().parts().iter().for_each(|p| {
                p.variables().iter().for_each(|v| {
                    variables.insert(v);
                })
            });
        }

        let cmd_args = run_utils::command_args(&inputs);
        let overwrite: HashMap<String, String> = run_utils::overwrite_vars(&inputs, &cmd_args)?;

        let input_files = run_utils::inputs(&self.config.dir, &inputs, &variables)?;
        let total = input_files.len();

        for (i, input) in input_files.iter().enumerate() {
            input.eprint_job(i + 1, total);
            let variables = run_utils::variables_from_input(input, &overwrite)?;
            for cmd in &commands {
                cmd.run(&variables, self.demo, false)?;
            }
        }
        Ok(())
    }

    fn new() -> Self {
        Self {
            config: AnekConfig::new().unwrap(),
            input_type: AnekInputType::default(),
            input: None,
            subset: String::new(),
            _parallel: false,
            task_type: AnekTaskType::default(),
            task: None,
            demo: false,
            editing: false,
            dirty: false,
            dir_type: crate::dtypes::AnekDirectoryType::Inputs,
            filename: None,
            content: text_editor::Content::default(),
        }
    }

    fn update(&mut self, msg: Message) {
        match msg {
            Message::InputTypeChanged(v) => {
                self.input_type = v;
                self.input = None;
            }
            Message::InputChanged(v) => {
                self.input = Some(v);
            }
            Message::TaskTypeChanged(v) => {
                self.task_type = v;
                self.task = None;
            }
            Message::TaskChanged(v) => {
                self.task = Some(v);
            }
            Message::SubsetChanged(v) => {
                self.subset = v;
            }
            Message::DemoChecked(v) => {
                self.demo = v;
            }
            Message::Run => {
                let start_time = Local::now().format("%Y-%m-%d %H:%M:%S");
                let start = Instant::now();
                let action_result = self.run();
                let duration = start.elapsed();
                if let Err(e) = action_result {
                    eprintln!("{}: {}", "Error".bright_red(), e);
                }
                eprintln!("{:12}: {}", "Started at".bright_blue().bold(), start_time);
                eprintln!("{:12}: {:?}", "Time Elapsed".bright_blue().bold(), duration);
            }
            Message::EditAnekFiles => {
                self.editing = true;
                self.dirty = false;
            }
            Message::FinishEditing => {
                // TODO check for dirty: save ? not save?
                self.editing = false;
                self.dirty = false;
            }
            Message::DirTypeChanged(v) => {
                self.dir_type = v;
                self.filename = None;
                self.content = text_editor::Content::new();
            }
            Message::EditFileChanged(v) => {
                let path = self.config.dir.get_file(&self.dir_type, &v);
                let txt = std::fs::read_to_string(path).unwrap_or_default();
                self.content = text_editor::Content::with_text(&txt);
                self.dirty = false;
                self.filename = Some(v);
            }
            Message::Edit(a) => {
                if a.is_edit() {
                    self.dirty = true;
                }
                self.content.perform(a);
            }
            Message::RevertEdit => {
                if let Some(v) = &self.filename {
                    let path = self.config.dir.get_file(&self.dir_type, v);
                    let txt = std::fs::read_to_string(path).unwrap_or_default();
                    self.content = text_editor::Content::with_text(&txt);
                } else {
                    self.content = text_editor::Content::new();
                }
                self.dirty = false;
            }
            Message::SaveEdit => {
                if let Some(v) = &self.filename {
                    let path = self.config.dir.get_file(&self.dir_type, v);
                    let _ = std::fs::write(path, self.content.text());
                    self.dirty = false;
                } else {
                    // ask filename? how ? maybe add a text_input for filename when it's new file
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let main = if self.editing {
            self.view_edit()
        } else {
            self.view_run()
        };
        column![main, row![horizontal(), text("© 2026 Gaurav Atreya")]]
            .spacing(10)
            .into()
    }

    fn view_edit(&self) -> Element<'_, Message> {
        column![
            row![
                text("Editing"),
                pick_list(
                    crate::dtypes::anekdirtype(),
                    Some(&self.dir_type),
                    Message::DirTypeChanged
                ), // no way to disable?
                pick_list(
                    self.config.list(&self.dir_type),
                    self.filename.as_ref(),
                    Message::EditFileChanged
                ),
                button("Revert").on_press(Message::RevertEdit),
                horizontal(),
                button("Save").on_press(Message::SaveEdit)
            ]
            .spacing(5),
            text_editor(&self.content)
                .on_action(Message::Edit)
                .height(Length::Fill),
            button("Done").on_press(Message::FinishEditing)
        ]
        .spacing(10)
        .into()
    }

    fn view_run(&self) -> Element<'_, Message> {
        column![
            row![
                text("Inputs: "),
                pick_list(
                    AnekInputType::all(),
                    Some(&self.input_type),
                    Message::InputTypeChanged
                ),
                pick_list(
                    match &self.input_type {
                        AnekInputType::Inputs => self.config.inputs.as_slice(),
                        AnekInputType::Batch => self.config.batch.as_slice(),
                        AnekInputType::Loops => self.config.loops.as_slice(),
                    },
                    self.input.as_ref(),
                    Message::InputChanged
                )
            ]
            .spacing(5),
            row![
                text("Tasks: "),
                pick_list(
                    AnekTaskType::all(),
                    Some(&self.task_type),
                    Message::TaskTypeChanged
                ),
                pick_list(
                    match &self.task_type {
                        AnekTaskType::Command => self.config.commands.as_slice(),
                        AnekTaskType::Pipeline => self.config.pipelines.as_slice(),
                    },
                    self.task.as_ref(),
                    Message::TaskChanged
                )
            ]
            .spacing(5),
            row![
                button("Edit").on_press(Message::EditAnekFiles),
                horizontal(),
                text_input("All", &self.subset)
                    .on_input(Message::SubsetChanged)
                    .width(100),
                checkbox(self.demo)
                    .label("Demo Run")
                    .on_toggle(Message::DemoChecked),
                button("Run").on_press(Message::Run)
            ]
        ]
        .spacing(10)
        .into()
    }
}

#[derive(Clone)]
enum Message {
    InputTypeChanged(AnekInputType),
    InputChanged(String),
    TaskTypeChanged(AnekTaskType),
    TaskChanged(String),
    SubsetChanged(String),
    Run,
    DemoChecked(bool),
    EditAnekFiles,
    DirTypeChanged(AnekDirectoryType),
    EditFileChanged(String),
    Edit(text_editor::Action),
    SaveEdit,
    RevertEdit,
    FinishEditing,
}
