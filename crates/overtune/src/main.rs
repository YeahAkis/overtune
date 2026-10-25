use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, glib};

const APP_ID: &str = "io.github.yeahakis.Overtune";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        ApplicationWindow::builder()
            .application(app)
            .title("Overtune")
            .default_width(1200)
            .default_height(760)
            .build()
            .present();
    });
    app.run()
}
