use crate::dtypes;
use crate::variable;
use gtk::{gio::ApplicationFlags, Application, StringList, StringObject};
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

use std::path::PathBuf;

use gtk::{gio, glib, prelude::*};

const APP_ID: &str = "org.anek.AnekEditor";

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
    let cmb_file = builder
        .object::<gtk::DropDown>("cmb_file")
        .expect("Couldn't get builder");
    let btn_save = builder
        .object::<gtk::Button>("btn_save")
        .expect("Couldn't get builder");
    let txt_file = builder
        .object::<gtk::TextView>("txt_file")
        .expect("Couldn't get text_view");

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
        glib::clone!(@weak window, @weak txt_browse, @weak cmb_file => move |_| {
            let wd = PathBuf::from(txt_browse.text());
            if  let Ok(anek) = dtypes::AnekDirectory::from(&wd){
        let files = variable::list_filenames(&anek.root).unwrap();
        let files: StringList = files.into_iter().collect();
        cmb_file.set_enable_search(true);
        cmb_file.set_model(Some(&files));
            } else {
        cmb_file.set_model(None::<StringList>.as_ref());
        }
        }),
    );

    cmb_file.connect_selected_item_notify(
        glib::clone!(@weak window, @weak txt_browse, @weak txt_file, @weak cmb_file => move |_| {
            let path: StringObject = cmb_file.selected_item().unwrap().downcast::<StringObject>().unwrap();
            let wd = PathBuf::from(txt_browse.text());
            if  let Ok(anek) = dtypes::AnekDirectory::from(&wd){
		let filename = anek.root.join(&path.string());
		let file = File::open(filename).expect("Couldn't open file");

                let mut reader = BufReader::new(file);
                let mut contents = String::new();
                let _ = reader.read_to_string(&mut contents);
                txt_file.buffer().set_text(&contents);
	    }
        }),
    );

    btn_save.connect_clicked(
        glib::clone!(@weak window, @weak txt_browse, @weak txt_file, @weak cmb_file => move |_| {
            let path: StringObject = cmb_file.selected_item().unwrap().downcast::<StringObject>().unwrap();
            let wd = PathBuf::from(txt_browse.text());
            if  let Ok(anek) = dtypes::AnekDirectory::from(&wd){
		let filename = anek.root.join(&path.string());
		let buf = txt_file.buffer();
		let contents = buf.text(&buf.start_iter(), &buf.end_iter(), true).to_string();
		let mut file = File::create(filename).expect("Couldn't open file");
		let _ = file.write_all(contents.into_bytes().as_slice());
	    }
        }),
    );

    txt_browse.set_text(&PathBuf::from(".").canonicalize().unwrap().to_string_lossy());
    window.present();
}
