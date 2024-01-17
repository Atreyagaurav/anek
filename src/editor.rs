use crate::dtypes;
use crate::dtypes::AnekDirectory;
use gtk::gdk::Key;
use gtk::gdk::ModifierType;
use gtk::gio::Settings;
use gtk::AlertDialog;
use gtk::DropDown;
use gtk::EventControllerKey;
use gtk::{gio::ApplicationFlags, Application, StringList, StringObject};
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

use std::path::PathBuf;

use gtk::{gio, glib, prelude::*};

pub const APP_ID: &str = "org.anek.AnekEditor";

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
    let ui_src = include_str!("../resources/editor.ui");
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

    load_ui!(dd_filetype, gtk::DropDown);
    load_ui!(btn_savenew, gtk::Button);
    load_ui!(txt_newfile, gtk::Entry);
    load_ui!(cb_newfile_sub, gtk::CheckButton);
    load_ui!(txt_newfile_sub, gtk::Entry);

    load_ui!(dd_file, gtk::DropDown);
    load_ui!(btn_save, gtk::Button);
    load_ui!(swt_save, gtk::Switch);
    load_ui!(txt_file, gtk::TextView);

    btn_browse.connect_clicked(glib::clone!(@weak window, @weak txt_browse => move |_| {

        let dialog = gtk::FileDialog::builder()
            .title("Anek Project Location")
            .accept_label("Open Folder")
            .initial_folder(&gio::File::for_path(txt_browse.text()))
            .build();

        dialog.select_folder(Some(&window), gio::Cancellable::NONE, move |file| {
            if let Ok(file) = file {
                let filename = file.path().expect("Couldn't get file path");
        let name = filename.to_string_lossy();
        txt_browse.set_text(&name);
            }
        });
    }));

    txt_browse.connect_changed(
        glib::clone!(@weak window, @weak dd_file, @weak txt_file, @weak btn_save, @weak btn_savenew => move |txt_browse| {
            let wd = PathBuf::from(txt_browse.text());
            if  let Ok(anek) = dtypes::AnekDirectory::from(&wd){
		load_anek_files(&dd_file, &anek);
        dd_file.set_sensitive(true);
        txt_file.set_sensitive(true);
        btn_savenew.set_sensitive(true);
        btn_save.set_sensitive(true);
            } else {
        dd_file.set_sensitive(false);
        txt_file.set_sensitive(false);
        btn_savenew.set_sensitive(false);
        btn_save.set_sensitive(false);
        }
        }),
    );

    dd_file.connect_selected_item_notify(
        glib::clone!(@weak window, @weak txt_browse, @weak txt_file, @weak btn_save => move |dd_file| {
            if let Some(path) = dd_file.selected_item().map( |i| i.downcast::<StringObject>().unwrap()){
            let wd = PathBuf::from(txt_browse.text());
            if  let Ok(anek) = dtypes::AnekDirectory::from(&wd){
		let filename = anek.get_file_global(&path.string());
		let file = File::open(filename).expect("Couldn't open file");

                let mut reader = BufReader::new(file);
                let mut contents = String::new();
                let _ = reader.read_to_string(&mut contents);
                txt_file.buffer().set_text(&contents);
		txt_file.buffer().set_modified(false);
	    }
	    }
        }),
    );

    btn_savenew.connect_clicked(glib::clone!(@weak window, @weak txt_browse, @weak dd_filetype, @weak dd_file, @weak txt_newfile, @weak cb_newfile_sub, @weak txt_newfile_sub => move |_| {
            let wd = PathBuf::from(txt_browse.text());
            if  let Ok(anek) = dtypes::AnekDirectory::from(&wd){
	let dt: String = dd_filetype.selected_item().unwrap().downcast::<StringObject>().unwrap().string().into();
	let dt = match dt.as_str() {
	    "Variable" => dtypes::AnekDirectoryType::Variables,
	    "Command" => dtypes::AnekDirectoryType::Commands,
	    "Pipeline" => dtypes::AnekDirectoryType::Pipelines,
	    "Template" => dtypes::AnekDirectoryType::Templates,
	    "Input" => dtypes::AnekDirectoryType::Inputs,
	    "Batch" => dtypes::AnekDirectoryType::Batch,
	    "Loop" => dtypes::AnekDirectoryType::Loops,
	    _ => panic!("Invalid Choice for filetype"),
	};
		// check for subfolders
		match &dt {
		    &dtypes::AnekDirectoryType::Loops => {
		if !cb_newfile_sub.is_active(){
		    alert_diag(&window, "Loop requires subfolders!");
		    return;
		}},
		    &dtypes::AnekDirectoryType::Variables | &dtypes::AnekDirectoryType::Templates => {
			if cb_newfile_sub.is_active(){
		    alert_diag(&window, "Subfolders not supported!");
		    return;
			}},
			_ =>  if cb_newfile_sub.is_active(){
		    if txt_newfile_sub.text().is_empty(){
			alert_diag(&window, "Cannot have Empty subfolder name!");
			return;
		    } else {
			txt_newfile.set_text(&format!("{}/{}", txt_newfile.text(), txt_newfile_sub.text()));
			cb_newfile_sub.set_active(false);
		    }
		},
		}
		if txt_newfile.text().is_empty(){
		    alert_diag(&window, "Empty Filename!");
		    return;
		}
		let filename = if cb_newfile_sub.is_active() {
		    if txt_newfile_sub.text().is_empty(){
			alert_diag(&window, "Cannot have Empty subfolder name!");
			return;
		    }
		    format!("{}.d/{}",
			    txt_newfile.text(),
			    txt_newfile_sub.text())
		} else {
		    txt_newfile.text().to_string()
		};
		let path = anek.get_file(&dt, &filename);
		if path.exists(){
		    alert_diag(&window, "File Already Exists.");
		} else {
		    File::create(path).unwrap();
		    load_anek_files(&dd_file, &anek);
		}
		let filename = format!("{}/{}", dt.dir_name(), filename);
		let mut m: u32 = 0;
		let model = dd_file.model().unwrap();
		for i in 0..(model.n_items()){
		    let p = model.item(i).unwrap().downcast::<StringObject>().unwrap().string().to_string();
		    if p == filename{
			m = i;
			break;
		    }
		}
		dd_file.set_selected(m);
	    }
    }));

    swt_save.connect_active_notify(glib::clone!(@weak btn_save, @weak swt_save => move |_| {
        btn_save.emit_by_name::<()>("clicked", &[]);
    if swt_save.is_active() {
        btn_save.set_label("Auto Save");
        btn_save.set_sensitive(false);
    }else {
        btn_save.set_label("Save");
        btn_save.set_sensitive(true);
    }
    }));

    btn_save.connect_clicked(
        glib::clone!(@weak window, @weak btn_save, @weak txt_browse, @weak txt_file, @weak dd_file => move |_| {
            if let Some(path) = dd_file.selected_item().map( |i| i.downcast::<StringObject>().unwrap()){
            let wd = PathBuf::from(txt_browse.text());
            if  let Ok(anek) = dtypes::AnekDirectory::from(&wd){
		let filename = anek.get_file_global(&path.string());
		let buf = txt_file.buffer();
		let contents = buf.text(&buf.start_iter(), &buf.end_iter(), true).to_string();
		let mut file = File::create(filename).expect("Couldn't open file");
		let _ = file.write_all(contents.into_bytes().as_slice());
		txt_file.buffer().set_modified(false);
	    }
	    }
        }),
    );

    txt_file.buffer().connect_modified_changed(
        glib::clone!(@weak btn_save, @weak txt_file, @weak swt_save=> move |_| {
        if swt_save.is_active(){
        return;
        }
        if txt_file.buffer().is_modified() {
        btn_save.set_label("*Save*");
        } else {
        btn_save.set_label("Save");
        }
        }
        ),
    );

    txt_file.connect_has_focus_notify(
        glib::clone!(@weak btn_save, @weak swt_save, @weak txt_file => move |_| {
        if txt_file.buffer().is_modified() && swt_save.is_active() {
            btn_save.emit_by_name::<()>("clicked", &[]);
        }
        }),
    );

    let event_ctrl = EventControllerKey::new();
    event_ctrl.connect_key_released(
        glib::clone!(@weak window, @weak btn_save, @weak dd_file, @weak swt_save  => move|_, key, _, state| {
        match (state, key) {
            (_, Key::Escape) => std::process::exit(0),
            (_, Key::Page_Down) => {
		if swt_save.is_active() {
		    btn_save.emit_by_name::<()>("clicked", &[]);
		}
        dd_file.set_selected((dd_file.selected() + 1)% dd_file.model().unwrap().n_items());
        },
            (_, Key::Page_Up) => {
		if swt_save.is_active() {
		    btn_save.emit_by_name::<()>("clicked", &[]);
		}
        dd_file.set_selected((dd_file.selected() - 1)% dd_file.model().unwrap().n_items());},
            (ModifierType::CONTROL_MASK, Key::S) => {
            btn_save.emit_by_name::<()>("clicked", &[]);
            },
        _ => ()
        }
        // for pressed
        // glib::Propagation::Proceed
        }),
    );
    window.add_controller(event_ctrl);

    // Initialize settings and always load the last project
    let settings = Settings::new(APP_ID);
    settings.bind("project-path", &txt_browse, "text").build();
    let curr_path = PathBuf::from(settings.string("project-path"));
    let curr_path = match curr_path.canonicalize() {
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

fn load_anek_files(dd_file: &DropDown, anek: &AnekDirectory) {
    let files = anek.list_all_files();
    let files: StringList = files.into_iter().collect();
    dd_file.set_model(Some(&files));
}
