use crate::dtypes;
use crate::variable;
use gtk::{gio::ApplicationFlags, Application, StringList, StringObject};
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

use std::path::PathBuf;

use gtk::{gio, glib, prelude::*};

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

    let window = builder
        .object::<gtk::ApplicationWindow>("window")
        .expect("Couldn't get window");
    window.set_application(Some(application));
    let btn_browse = builder
        .object::<gtk::Button>("btn_browse")
        .expect("Couldn't get builder");
    let txt_browse = builder
        .object::<gtk::Text>("txt_browse")
        .expect("Couldn't get builder");
    let dd_command = builder
        .object::<gtk::DropDown>("dd_command")
        .expect("Couldn't get builder");
    let dd_task = builder
        .object::<gtk::DropDown>("dd_task")
        .expect("Couldn't get builder");
    let dd_input = builder
        .object::<gtk::DropDown>("dd_input")
        .expect("Couldn't get builder");
    let btn_execute = builder
        .object::<gtk::Button>("btn_execute")
        .expect("Couldn't get builder");

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

    txt_browse.connect_changed(
        glib::clone!(@weak window, @weak txt_browse, @weak dd_task, @weak dd_command, @weak dd_input => move |_| {
            let wd = PathBuf::from(txt_browse.text());
            if let Ok(anek) = dtypes::AnekDirectory::from(&wd){
		let cmd = dd_command.selected_item().unwrap().downcast::<StringObject>().unwrap().string().to_string();
		let tasks = match cmd.as_str() {
		    "run" => variable::list_anek_filenames(&anek.get_directory(&dtypes::AnekDirectoryType::Commands)),
		    "export" => variable::list_anek_filenames(&anek.get_directory(&dtypes::AnekDirectoryType::Variables)),
		    _ => Ok(vec![]),
		}.unwrap();
		
        let tasks: StringList = tasks.into_iter().collect();
        dd_task.set_enable_search(true);
		dd_task.set_model(Some(&tasks));

		let inputs: StringList = variable::list_anek_filenames(&anek.get_directory(&dtypes::AnekDirectoryType::Inputs)).unwrap().into_iter().collect();

        dd_input.set_enable_search(true);
		dd_input.set_model(Some(&inputs));
		
            } else {
        dd_task.set_model(None::<StringList>.as_ref());
        }
        }),
    );

    dd_command.connect_selected_item_notify(
        glib::clone!(@weak window, @weak txt_browse => move |_| {
	    txt_browse.emit_by_name::<()>("changed", &[]);
        }),
    );

    btn_execute.connect_clicked(
        glib::clone!(@weak window, @weak txt_browse => move |_| {
            
	    todo!()
        }),
    );

    txt_browse.set_text(&PathBuf::from(".").canonicalize().unwrap().to_string_lossy());
    window.present();
}
