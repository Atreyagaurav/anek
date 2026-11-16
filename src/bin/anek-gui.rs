use anyhow::Error;
use chrono::Local;
use clap::{Args, ValueHint};
use colored::Colorize;
use iced::Element;
use iced::widget::{
    button, checkbox, column, container, mouse_area, operation::focus, pick_list, row, scrollable,
    space::horizontal, text, text::Rich, text_editor, text_input, tooltip,
};
use iced::{Color, Length, Task};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use anek::dtypes::{AnekDirectory, AnekDirectoryType};
use anek::run_utils;

pub fn main() -> Result<(), Error> {
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
        let path = if let Some(p) = std::env::args().nth(1) {
            PathBuf::from(p)
        } else {
            PathBuf::from(".")
        };
        let dir = AnekDirectory::from(&path)?;
        let list = |dt| anek::variable::list_anek_filenames(&dir.get_directory(dt));
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

    fn maybe_one<'a>(&'a self, ty: &AnekDirectoryType) -> Option<&'a String> {
        match self.list(ty) {
            [one] => Some(one),
            _ => None,
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

    fn anek_dir_type(&self) -> &AnekDirectoryType {
        match self {
            AnekInputType::Inputs => &AnekDirectoryType::Inputs,
            AnekInputType::Batch => &AnekDirectoryType::Batch,
            AnekInputType::Loops => &AnekDirectoryType::Loops,
        }
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

    fn anek_dir_type(&self) -> &AnekDirectoryType {
        match self {
            AnekTaskType::Command => &AnekDirectoryType::Commands,
            AnekTaskType::Pipeline => &AnekDirectoryType::Pipelines,
        }
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
    preview: (text_editor::Content, text_editor::Content),
    demo: bool,
    editing: bool,
    dirty: bool,
    dir_type: AnekDirectoryType,
    filename: Option<String>,
    new_file: Option<String>,
    content: text_editor::Content,
    error: bool,
    status: String,
}

impl State {
    fn run(&self) -> Result<(), Error> {
        if self.input.is_none() || self.task.is_none() {
            // do nothing; maybe give error later?
            return Ok(());
        }
        let select_inputs = run_utils::filter_index(&self.subset)?;
        let input = self.input.clone().unwrap();
        let (batch, input, r#loop) = match &self.input_type {
            AnekInputType::Inputs => (vec![], vec![input], None),
            AnekInputType::Batch => (vec![input], vec![], None),
            AnekInputType::Loops => (vec![], vec![], Some(input)),
        };
        let inputs = run_utils::Inputs::On(run_utils::InputsArgs {
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
            preview: (
                text_editor::Content::default(),
                text_editor::Content::default(),
            ),
            demo: false,
            editing: false,
            dirty: false,
            dir_type: AnekDirectoryType::Inputs,
            filename: None,
            new_file: None,
            content: text_editor::Content::default(),
            error: false,
            status: "Select a command and input, then press run (check output in terminal)".into(),
        }
    }

    fn update(&mut self, msg: Message) {
        match msg {
            Message::InputTypeChanged(v) => {
                self.input_type = v;
                self.input = self
                    .config
                    .maybe_one(self.input_type.anek_dir_type())
                    .cloned();
                self.preview.1 = text_editor::Content::new();
            }
            Message::InputChanged(v) => {
                if self.input_type == AnekInputType::Loops {
                    self.preview.1 = text_editor::Content::with_text(
                        "# preview not supported for loop inputs\nuse Demo run to see",
                    );
                } else {
                    let path = self
                        .config
                        .dir
                        .get_file(self.input_type.anek_dir_type(), &v);
                    let txt = std::fs::read_to_string(path).unwrap_or_default();
                    self.preview.1 = text_editor::Content::with_text(&txt);
                }
                self.input = Some(v);
            }
            Message::TaskTypeChanged(v) => {
                self.task_type = v;
                self.task = self
                    .config
                    .maybe_one(self.task_type.anek_dir_type())
                    .cloned();
                self.preview.0 = text_editor::Content::new();
            }
            Message::TaskChanged(v) => {
                let path = self.config.dir.get_file(self.task_type.anek_dir_type(), &v);
                let txt = std::fs::read_to_string(path).unwrap_or_default();
                self.preview.0 = text_editor::Content::with_text(&txt);
                self.task = Some(v);
            }
            Message::SubsetChanged(v) => {
                self.subset = v;
            }
            Message::DemoChecked(v) => {
                self.demo = v;
                self.status = if v {
                    "Running a demo only shows the commands without executing them"
                } else {
                    "Run button will execute the actual commands in the terminal"
                }
                .into();
                self.error = false;
            }
            Message::Run => {
                let start_time = Local::now().format("%Y-%m-%d %H:%M:%S");
                let start = Instant::now();
                // maybe we need to do async execution for this with cancel option for later?
                let action_result = self.run();
                let duration = start.elapsed();
                if let Err(e) = action_result {
                    eprintln!("{}: {}", "Error".bright_red(), e);
                    self.error = true;
                    self.status = format!("Error: {e}; check terminal for more details");
                } else {
                    self.error = false;
                    self.status = "Completed; Check terminal for output".into();
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
                if let Some(v) = &self.filename
                    && self.new_file.is_none()
                {
                    let path = self.config.dir.get_file(&self.dir_type, v);
                    let txt = std::fs::read_to_string(path).unwrap_or_default();
                    self.content = text_editor::Content::with_text(&txt);
                } else {
                    self.content = text_editor::Content::new();
                }
                self.dirty = false;
            }
            Message::SaveEdit => {
                if let Some(v) = &self.new_file {
                    let path = self.config.dir.get_file(&self.dir_type, v);
                    let _ = std::fs::write(path, self.content.text());
                    self.dirty = false;
                } else if let Some(v) = &self.filename {
                    let path = self.config.dir.get_file(&self.dir_type, v);
                    let _ = std::fs::write(path, self.content.text());
                    self.dirty = false;
                } else {
                    // Save button disabled when both are none, so this should not happen
                }
            }
            Message::NewFileToggle(v) => {
                self.new_file = v.then(|| String::new());
            }
            Message::NewFileName(v) => {
                if let Some(n) = &mut self.new_file {
                    *n = v;
                }
                // The message should not be sent when new_file is none from UI
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
            .padding(20)
            .into()
    }

    fn view_edit(&self) -> Element<'_, Message> {
        let filename: Element<_> = match &self.new_file {
            Some(name) => text_input("FileName", name)
                .on_input(Message::NewFileName)
                .into(),
            None => pick_list(
                self.config.list(&self.dir_type),
                self.filename.as_ref(),
                Message::EditFileChanged,
            )
            .into(),
        };
        column![
            row![
                checkbox(self.new_file.is_some())
                    .label("New File")
                    .on_toggle(Message::NewFileToggle),
                text("Editing"),
                pick_list(
                    anek::dtypes::anekdirtype(),
                    Some(&self.dir_type),
                    Message::DirTypeChanged
                ), // no way to disable?
                filename,
                button("Revert").on_press(Message::RevertEdit),
                horizontal(),
                button("Save").on_press_maybe(
                    (self.new_file.is_some() || self.filename.is_some()).then(|| Message::SaveEdit)
                )
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
                text("Run: ").size(20),
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
                ),
                horizontal(),
                text("On").size(20),
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
                ),
            ]
            .spacing(5),
            row![
                text_editor(&self.preview.0).height(Length::Fill),
                text_editor(&self.preview.1).height(Length::Fill)
            ],
            row![text(&self.status).style(if self.error {
                text::danger
            } else {
                text::primary
            })],
            row![
                button("Edit Anek Config").on_press(Message::EditAnekFiles),
                horizontal(),
                text("Selection: ").size(20),
                text_input("All", &self.subset)
                    .on_input(Message::SubsetChanged)
                    .width(100),
                horizontal(),
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
    NewFileToggle(bool),
    NewFileName(String),
    RevertEdit,
    FinishEditing,
}
