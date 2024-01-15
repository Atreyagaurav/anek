use crate::dtypes;
use crate::dtypes::AnekDirectoryType;
use crate::variable;
use colored::Colorize;
use gtk::gio::Settings;
use gtk::AlertDialog;
use gtk::{gio, glib, prelude::*};
use gtk::{gio::ApplicationFlags, Application, StringList, StringObject};
use itertools::Itertools;
use std::path::PathBuf;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

const APP_ID: &str = "org.zero.AnekGui";

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

    // task notebook
    load_ui!(nb_task, gtk::Notebook);
    load_ui!(cb_pipeline, gtk::CheckButton);
    load_ui!(dd_command, gtk::DropDown);
    load_ui!(dd_variable, gtk::DropDown);

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
        dd_command.set_enable_search(true);
		dd_command.set_model(Some(&tasks));
		dd_command.set_sensitive(true);
	    } else {
		dd_command.set_sensitive(false);
	    }}));

    txt_browse.connect_changed(
        glib::clone!(@weak window, @weak txt_browse, @weak dd_command, @weak dd_input, @weak dd_batch, @weak dd_loop, @weak cb_pipeline => move |_| {
                let wd = PathBuf::from(txt_browse.text());
            if let Ok(anek) = dtypes::AnekDirectory::from(&wd){
		// dd_commands and pipelines can be filled from there.
		cb_pipeline.notify("active");
            let variables = variable::list_anek_filenames(
                &anek.get_directory(&dtypes::AnekDirectoryType::Variables)
            ).unwrap();
            let variables: StringList = variables.into_iter().collect();
            dd_variable.set_enable_search(true);
		    dd_variable.set_model(Some(&variables));

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

            dd_input.set_enable_search(true);
            dd_batch.set_enable_search(true);
            dd_loop.set_enable_search(true);
            dd_variable.set_enable_search(true);
            dd_input.set_sensitive(true);
            dd_batch.set_sensitive(true);
            dd_loop.set_sensitive(true);
            dd_variable.set_sensitive(true);
                } else {
            dd_input.set_sensitive(false);
            dd_batch.set_sensitive(false);
            dd_loop.set_sensitive(false);
            dd_variable.set_sensitive(false);
            }
            }),
    );
    btn_execute.connect_clicked(
        glib::clone!(@weak window, @weak dd_command, @weak txt_browse, @weak nb_task, @weak nb_input, @weak dd_input, @weak dd_batch, @weak dd_loop, @weak cb_pipeline, @weak txt_command => move |_| {
            let wd = PathBuf::from(txt_browse.text());
            if let Ok(anek) = dtypes::AnekDirectory::from(&wd){
		let cmd: String =
		    format!(
			"anek -q {} {} {} on {}",
			match nb_task.current_page() {
			    Some(0) => "run",
			    Some(1) => "export",
			    Some(2) => "render",
			    _ => panic!("Only 3 tabs coded."),
			},
			// tasks
			if cb_pipeline.is_active() {"-p"} else {""},
			dd_command.selected_item().map(|i| i.downcast::<StringObject>().unwrap().string()).unwrap(),
			// inputs
			match nb_input.current_page() {
			    Some(0) => format!("-i {}", dd_input.selected_item().map( |i| i.downcast::<StringObject>().unwrap().string()).unwrap()),
			    Some(1) => format!("-b {}", dd_batch.selected_item().map( |i| i.downcast::<StringObject>().unwrap().string()).unwrap()),
			    Some(2) => format!("-l {}", dd_loop.selected_item().map( |i| i.downcast::<StringObject>().unwrap().string()).unwrap()),
			    _ => panic!("Only 3 tabs coded."),
			},
		    );
		txt_command.set_text(&cmd);
		println!("{} {:?}", "Subprocess Result:".on_blue(), subprocess::Exec::shell(cmd).cwd(anek.proj_root).join());
        } else {
        invalid_anek_dir_warning(&window);
        }
        }),
    );

    dd_input.connect_selected_item_notify(
        glib::clone!(@weak txt_browse, @weak dd_input, @weak tv_input => move |_| {
            let wd = PathBuf::from(txt_browse.text());
            if let Ok(anek) = dtypes::AnekDirectory::from(&wd){
		let path = anek.get_file(
		    &AnekDirectoryType::Inputs,
		    &dd_input.selected_item().unwrap().downcast::<gtk::StringObject>().unwrap().string()
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
                &dd_batch.selected_item().unwrap().downcast::<gtk::StringObject>().unwrap().string()
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
                &dd_loop.selected_item().unwrap().downcast::<gtk::StringObject>().unwrap().string()
            ).with_extension("d");
            let loop_vars = variable::loop_inputs(&path).unwrap();
            let contents = loop_vars.into_iter().multi_cartesian_product().map(
            |inp| inp.iter().map(|(var, i, val)| format!("{var}[{i}]={val}")).join("; ")
            ).join("\n");
                    tv_loop.buffer().set_text(&contents);
            }}
        ),
    );

    let settings = Settings::new(crate::editor::APP_ID);
    settings.bind("project-path", &txt_browse, "text").build();
    let curr_path = PathBuf::from(settings.string("project-path"));
    let curr_path = match curr_path.canonicalize() {
        Ok(p) => p.to_string_lossy().to_string(),
        _ => curr_path.to_string_lossy().to_string(),
    };
    txt_browse.set_text(&curr_path);
    window.present();
}

fn invalid_anek_dir_warning(window: &gtk::ApplicationWindow) {
    let diag = AlertDialog::builder()
        .modal(false)
        .message("Not a Valid Anek Project.")
        .build();
    diag.choose(Some(window), gio::Cancellable::NONE, |_| ());
}
