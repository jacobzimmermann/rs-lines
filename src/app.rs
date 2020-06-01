use gio::prelude::*;
use gtk::prelude::*;

use std::ops::Deref;
use std::rc::Rc;

const APP_MENU: &str = "
        <interface>
            <menu id=\"appmenu\">
                <section>
                    <item>
                        <attribute name=\"label\" translatable=\"yes\">New Window</attribute>
                        <attribute name=\"action\">app.new_window</attribute>
                    </item>
                </section>
                <section>
                    <item>
                        <attribute name=\"label\" translatable=\"yes\">Quit</attribute>
                        <attribute name=\"action\">app.quit</attribute>
                    </item>
                </section>
            </menu>
        </interface>
    ";

pub struct Lines(gtk::Application);
pub type LinesPtr = Rc<Lines>;

impl Deref for Lines {
    type Target = gtk::Application;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

trait ILines {
    fn on_startup(&self);
    fn on_activate(&self);
}

impl ILines for LinesPtr {
    fn on_startup(&self) {
        use gtk::prelude::*;

        let act_quit = gio::SimpleAction::new("quit", None);
        let wk_quit = Rc::downgrade(&self);
        act_quit.connect_activate(move |_, _| {
            wk_quit.upgrade().expect("quit pointer is nil").quit()
        });
        self.add_action(&act_quit);

        let act_new_window = gio::SimpleAction::new("new_window", None);
        let wk_new_window = Rc::downgrade(&self);
        act_new_window.connect_activate(move |_, _| {
            wk_new_window
                .upgrade()
                .expect("new_window pointer is nil")
                .activate()
        });
        self.add_action(&act_new_window);

        let builder = gtk::Builder::new_from_string(APP_MENU);
        let model: gio::MenuModel = builder.get_object("appmenu").unwrap();
        self.set_app_menu(Some(&model));
    }

    fn on_activate(&self) {
        let w = crate::window::LinesWindow::get_new_ptr(&self);
        w.show_all();
    }
}

impl Lines {
    pub const DBUS_PATH: &'static str = "net.jzimm.lines";

    pub fn initialise() -> Result<LinesPtr, glib::BoolError> {
        let gtk_app =
            gtk::Application::new(Some(Self::DBUS_PATH), gio::ApplicationFlags::FLAGS_NONE)?;
        let linesapp = LinesPtr::new(Lines(gtk_app));

        let wk_startup = Rc::downgrade(&linesapp);
        linesapp.connect_startup(move |_| {
            wk_startup
                .upgrade()
                .expect("startup pointer is nil")
                .on_startup()
        });

        let wk_activate = Rc::downgrade(&linesapp);
        linesapp.connect_activate(move |_| {
            wk_activate
                .upgrade()
                .expect("activate pointer is nil")
                .on_activate()
        });

        Ok(linesapp)
    }
}