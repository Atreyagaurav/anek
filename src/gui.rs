use crate::dtypes::AnekDirectoryType;
use crate::run_utils::InputsArgs;
use crate::variable;
use crate::{dtypes, run_utils};
use colored::Colorize;
use gtk::gio::Settings;
use gtk::{gio, glib, prelude::*, Align, Label};
use gtk::{gio::ApplicationFlags, Application, StringList, StringObject};
use gtk::{AlertDialog, FileFilter};
use itertools::Itertools;
use std::path::PathBuf;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

const APP_ID: &str = "org.zero.AnekGui";

macro_rules! return_if_none {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => return,
        }
    };
}

pub fn run() -> anyhow::Result<()> {
    let app = Application::builder()
        .flags(ApplicationFlags::HANDLES_OPEN)
        .application_id(APP_ID)
        .build();
    app.connect_activate(build_ui);

    // Run the application, the args complication is because it things there are cli args.
    app.connect_open(|_, _, _| ());
    app.run_with_args(&[""]);
    Ok(())
}

pub fn build_ui(application: &gtk::Application) {
    let ui_src = include_str!("../resources/window.ui");
    let builder = gtk::Builder::from_string(ui_src);

    macro_rules! load_ui {
        ($l:ident, $t:ty) => {
            let $l = builder
                .object::<$t>(stringify!($l))
                .expect(concat!("couldn't get: ", stringify!($l)));
        };
    }

    let window = builder
        .object::<gtk::ApplicationWindow>("window")
        .expect("Couldn't get window");
    window.set_application(Some(application));

    load_ui!(btn_browse, gtk::Button);
    load_ui!(txt_browse, gtk::Text);

    load_ui!(nb_task, gtk::Notebook);
    // command tab
    load_ui!(cb_pipeline, gtk::CheckButton);
    load_ui!(dd_command, gtk::DropDown);

    // export tab
    load_ui!(lb_variable, gtk::ListBox);
    load_ui!(cb_variable_safe, gtk::CheckButton);
    load_ui!(cb_variable_all, gtk::CheckButton);
    load_ui!(dd_export_type, gtk::DropDown);
    load_ui!(cb_export_file, gtk::CheckButton);
    load_ui!(txt_export_file, gtk::Text);
    load_ui!(btn_export_file, gtk::Button);

    // Render tab
    load_ui!(dd_template, gtk::DropDown);
    load_ui!(tv_template, gtk::TextView);
    load_ui!(cb_template, gtk::CheckButton);
    load_ui!(txt_template, gtk::Text);
    load_ui!(btn_template, gtk::Button);

    // input notebook
    load_ui!(nb_input, gtk::Notebook);
    load_ui!(dd_input, gtk::DropDown);
    load_ui!(tv_input, gtk::TextView);
    load_ui!(dd_batch, gtk::DropDown);
    load_ui!(tv_batch, gtk::TextView);
    load_ui!(dd_loop, gtk::DropDown);
    load_ui!(tv_loop, gtk::TextView);

    load_ui!(btn_execute, gtk::Button);
    load_ui!(txt_command, gtk::Text);
    load_ui!(tv_command, gtk::TextView);

    btn_browse.connect_clicked(glib::clone!(@weak window, @weak txt_browse => move |_| {

        let dialog = gtk::FileDialog::builder()
            .title("Anek Project Location")
            .accept_label("Open Folder")
            .initial_folder(&gio::File::for_path(txt_browse.text()))
            .build();

        dialog.select_folder(Some(&window), gio::Cancellable::NONE, move |file| {
            if let Ok(file) = file {
                let filename = file.path().expect("Couldn't get file path");
        txt_browse.set_text(&filename.to_string_lossy());
            }
        });
    }));

    cb_pipeline.connect_active_notify(
        glib::clone!(@weak window, @weak txt_browse, @weak dd_command, @weak cb_pipeline => move |_| {
            let wd = PathBuf::from(txt_browse.text());
            if let Ok(anek) = dtypes::AnekDirectory::from(&wd){
		let tasks = variable::list_anek_filenames(
		    &anek.get_directory(
			if cb_pipeline.is_active() {
			    &dtypes::AnekDirectoryType::Pipelines
			} else {
			    &dtypes::AnekDirectoryType::Commands
			})).unwrap();
        let tasks: StringList = tasks.into_iter().collect();
		dd_command.set_model(Some(&tasks));
		dd_command.set_sensitive(true);
	    } else {
		dd_command.set_sensitive(false);
	    }}));

    txt_browse.connect_changed(
        glib::clone!(@weak window, @weak txt_browse, @weak dd_template, @weak dd_command, @weak dd_input, @weak dd_batch, @weak dd_loop, @weak cb_pipeline, @weak lb_variable => move |_| {
                let wd = PathBuf::from(txt_browse.text());
            if let Ok(anek) = dtypes::AnekDirectory::from(&wd){
		// dd_commands and pipelines can be filled from there.
		cb_pipeline.notify("active");
            let variables = variable::list_anek_filenames(
                &anek.get_directory(&dtypes::AnekDirectoryType::Variables)
            ).unwrap();
		variables.iter().for_each(|v| {
		    let l = Label::new(Some(v));
		    l.set_halign(Align::Start);
		    lb_variable.append(&l);
		});
		    // lb_variable.set_model(Some(&variables));

            let templates: StringList = variable::list_anek_filenames(
                &anek.get_directory(&dtypes::AnekDirectoryType::Templates)
            ).unwrap().into_iter().collect();
		dd_template.set_model(Some(&templates));

            let inputs: StringList = variable::list_anek_filenames(
                &anek.get_directory(&dtypes::AnekDirectoryType::Inputs)
            ).unwrap().into_iter().collect();
		    dd_input.set_model(Some(&inputs));

            let batches: StringList = variable::list_anek_filenames(
                &anek.get_directory(&dtypes::AnekDirectoryType::Batch)
            ).unwrap().into_iter().collect();
		    dd_batch.set_model(Some(&batches));

            let loops: StringList = variable::list_anek_filenames(
                &anek.get_directory(&dtypes::AnekDirectoryType::Loops)
            ).unwrap().into_iter().collect();
		    dd_loop.set_model(Some(&loops));

            dd_template.set_sensitive(true);
            dd_input.set_sensitive(true);
            dd_batch.set_sensitive(true);
            dd_loop.set_sensitive(true);
            lb_variable.set_sensitive(true);
                } else {
            dd_template.set_sensitive(false);
            dd_input.set_sensitive(false);
            dd_batch.set_sensitive(false);
            dd_loop.set_sensitive(false);
            lb_variable.set_sensitive(false);
            }
            }),
    );
    btn_execute.connect_clicked(
        glib::clone!(@weak window, @weak dd_command, @weak txt_browse, @weak nb_task, @weak nb_input, @weak dd_input, @weak lb_variable, @weak dd_export_type, @weak dd_batch, @weak dd_loop, @weak cb_pipeline, @weak cb_export_file, @weak txt_export_file, @weak txt_command, @weak dd_template, @weak cb_template, @weak txt_template => move |_| {
            let wd = PathBuf::from(txt_browse.text());
            if let Ok(anek) = dtypes::AnekDirectory::from(&wd){
		let inputs: InputsArgs = match nb_input.current_page() {
			    Some(0) => InputsArgs::input(vec![return_if_none!(dd_input.selected_item()).downcast::<StringObject>().unwrap().string().to_string()]),
			    Some(1) => InputsArgs::batch(vec![return_if_none!(dd_batch.selected_item()).downcast::<StringObject>().unwrap().string().to_string()]),
			    Some(2) => InputsArgs::r#loop(return_if_none!(dd_loop.selected_item()).downcast::<StringObject>().unwrap().string().to_string()),
			    _ => panic!("Only 3 tabs coded."),
		};
		let task_res: anyhow::Result<()> = match return_if_none!(nb_task.current_page()) {
		    0 => {
			let args = crate::run::CliArgs::from_gui(
			    cb_pipeline.is_active(),
			    return_if_none!(dd_command.selected_item()).downcast::<StringObject>().unwrap().string().to_string(),
			    run_utils::Inputs::On(inputs)
			);
			crate::run::run_command(args, anek)
		    }
		    1 => {
			let safe = cb_variable_safe.is_active();
			let variables: Vec<String> = lb_variable.selected_rows().iter().map(|r| {
			    r.child().unwrap().downcast::<Label>().unwrap().label().to_string()}).map(|l| if safe {format!("{l}?")
			} else {l}).collect();
			let output = if cb_export_file.is_active() {
				    if txt_export_file.text().is_empty(){
					alert_diag(&window, "Empty Output File");
					return;
				    }
				    Some(PathBuf::from(txt_export_file.text()))
				}else{
				    None
				};
			let args = crate::export::CliArgs::from_gui(
			    return_if_none!(dd_export_type.selected_item()).downcast::<StringObject>().unwrap().string().to_string().to_ascii_lowercase(),
			    variables,
			    run_utils::Inputs::On(inputs),
			    output
				);
			crate::export::run_command(args, anek)
		    },
			    2 => {
			let args = crate::render::CliArgs::from_gui(
			    return_if_none!(dd_template.selected_item()).downcast::<StringObject>().unwrap().string().to_string(),
			    if cb_template.is_active(){
				if txt_template.text().is_empty(){
				    alert_diag(&window, "Output File Empty!");
				    return;
				}
				Some(PathBuf::from(txt_template.text()))
			    } else {None},
			    run_utils::Inputs::On(inputs)
			);
			crate::render::run_command(args, anek)
		    },
			    _ => panic!("Only 3 tabs coded."),
		};
		match task_res {
		    Ok(_) => println!("{}", "-".repeat(20).blue()),
		    Err(e) => {
			println!("{}", "-".repeat(20).red());
			println!("{}: {:?}", "Error".on_red(), e);
			alert_diag(&window, &e.to_string());
			println!("{}", "-".repeat(20).red());
		    }
		}
        } else {
        alert_diag(&window, "Invalid Anek Projecect Directory!");
        }
        }),
    );

    dd_input.connect_selected_item_notify(
        glib::clone!(@weak txt_browse, @weak dd_input, @weak tv_input => move |_| {
            let wd = PathBuf::from(txt_browse.text());
            if let Ok(anek) = dtypes::AnekDirectory::from(&wd){
		let path = anek.get_file(
		    &AnekDirectoryType::Inputs,
		    &return_if_none!(dd_input.selected_item()).downcast::<gtk::StringObject>().unwrap().string()
		);
		let inputs = dtypes::CommandInputs::from_files(0, "".to_string(), vec![path]);
		let contents = inputs.read_files().unwrap().variables().iter().map(|(k,v)| format!("{k}={v}")).sorted().join("\n");
                tv_input.buffer().set_text(&contents);
	    }}
	));

    dd_batch.connect_selected_item_notify(
        glib::clone!(@weak txt_browse, @weak dd_batch, @weak tv_batch => move |_| {
                let wd = PathBuf::from(txt_browse.text());
                if let Ok(anek) = dtypes::AnekDirectory::from(&wd){
            let path = anek.get_file(
                &AnekDirectoryType::Batch,
                &return_if_none!(dd_batch.selected_item()).downcast::<gtk::StringObject>().unwrap().string()
            );
            let file = File::open(path).expect("Couldn't open file");
                    let mut reader = BufReader::new(file);
                    let mut contents = String::new();
                    let _ = reader.read_to_string(&mut contents);
                    tv_batch.buffer().set_text(&contents);
            }}
        ),
    );

    dd_loop.connect_selected_item_notify(
        glib::clone!(@weak txt_browse, @weak dd_loop, @weak tv_loop => move |_| {
                let wd = PathBuf::from(txt_browse.text());
                if let Ok(anek) = dtypes::AnekDirectory::from(&wd){
            let path = anek.get_file(
                &AnekDirectoryType::Loops,
                &return_if_none!(
            dd_loop.selected_item()
        ).downcast::<gtk::StringObject>().unwrap().string()
            ).with_extension("d");
            let loop_vars = variable::loop_inputs(&path).unwrap();
            let contents = loop_vars.into_iter().multi_cartesian_product().map(
        |inp| inp.iter().map(|dtypes::LoopVariable {name, index, value}| format!("{name}[{index}]={value}")).join("; ")
            ).join("\n");
                    tv_loop.buffer().set_text(&contents);
            }}
        ),
    );

    dd_command.connect_selected_item_notify(
        glib::clone!(@weak txt_browse, @weak dd_command, @weak tv_command => move |_| {
                let wd = PathBuf::from(txt_browse.text());
                if let Ok(anek) = dtypes::AnekDirectory::from(&wd){
		    let path = anek.get_file(
			if cb_pipeline.is_active() {
			    &dtypes::AnekDirectoryType::Pipelines
			} else {
			    &dtypes::AnekDirectoryType::Commands
			},
                &return_if_none!(dd_command.selected_item()).downcast::<gtk::StringObject>().unwrap().string()
            );
            let file = File::open(path).expect("Couldn't open file");
                    let mut reader = BufReader::new(file);
                    let mut contents = String::new();
                    let _ = reader.read_to_string(&mut contents);
                    tv_command.buffer().set_text(&contents);
            }}
        ),
    );

    cb_variable_all.connect_active_notify(glib::clone!(@weak lb_variable => move |cv_variable| {
        if cv_variable.is_active() {
            lb_variable.select_all();
        }else{
            lb_variable.unselect_all();
        }
    }));

    btn_export_file.connect_clicked(
        glib::clone!(@weak window, @weak txt_browse, @weak dd_export_type, @weak txt_export_file, @weak cb_export_file => move |_| {
	    let mime = format!("text/{}", return_if_none!(dd_export_type.selected_item()).downcast::<StringObject>().unwrap().string().to_string().to_ascii_lowercase());

	    let filter = FileFilter::new();
	    filter.add_mime_type(&mime);

            let dialog = gtk::FileDialog::builder()
                .title("Export File")
                .accept_label("Save")
		.default_filter(&filter)
                .initial_folder(&gio::File::for_path(txt_browse.text()))
                .build();
            dialog.save(Some(&window), gio::Cancellable::NONE, move |file| {
                if let Ok(file) = file {
                    let filename = file.path().expect("Couldn't get file path");
		    txt_export_file.set_text(&filename.to_string_lossy());
		    cb_export_file.set_active(true);
                }
            });
        }),
    );

    dd_template.connect_selected_item_notify(
        glib::clone!(@weak txt_browse, @weak dd_template, @weak tv_template => move |_| {
                    let wd = PathBuf::from(txt_browse.text());
                    if let Ok(anek) = dtypes::AnekDirectory::from(&wd){
                let path = anek.get_file(&dtypes::AnekDirectoryType::Templates,
                    &return_if_none!(dd_template.selected_item()).downcast::<gtk::StringObject>().unwrap().string()
                );
                let file = File::open(path).expect("Couldn't open file");
                        let mut reader = BufReader::new(file);
                        let mut contents = String::new();
                        let _ = reader.read_to_string(&mut contents);
                        tv_template.buffer().set_text(&contents);
            }}
            ),
    );

    btn_template.connect_clicked(
        glib::clone!(@weak window, @weak txt_browse, @weak txt_template, @weak cb_template => move |_| {
	    let filter = FileFilter::new();
	    filter.add_mime_type("text/plain");

            let dialog = gtk::FileDialog::builder()
                .title("Save Rendered File")
                .accept_label("Save")
		.default_filter(&filter)
                .initial_folder(&gio::File::for_path(txt_browse.text()))
                .build();
            dialog.save(Some(&window), gio::Cancellable::NONE, move |file| {
                if let Ok(file) = file {
                    let filename = file.path().expect("Couldn't get file path");
		    txt_template.set_text(&filename.to_string_lossy());
		    cb_template.set_active(true);
                }
            });
        }),
    );

    let settings = Settings::new(crate::editor::APP_ID);
    settings.bind("project-path", &txt_browse, "text").build();
    let curr_path = PathBuf::from(settings.string("project-path"));
    let curr_path = match dunce::canonicalize(&curr_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        _ => curr_path.to_string_lossy().to_string(),
    };
    txt_browse.set_text(&curr_path);
    window.present();
}

fn alert_diag(window: &gtk::ApplicationWindow, msg: &str) {
    let diag = AlertDialog::builder().modal(false).message(msg).build();
    diag.choose(Some(window), gio::Cancellable::NONE, |_| ());
}
