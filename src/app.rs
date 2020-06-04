use gio::prelude::*;
use gio::subclass::prelude::*;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use glib;
use glib::translate::*;

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

mod private {
    use super::*;
    pub struct LinesApp;

    impl ObjectSubclass for LinesApp {
        const NAME: &'static str = "LinesApp";
        type ParentType = gtk::Application;
        type Instance = glib::subclass::simple::InstanceStruct<Self>;
        type Class = glib::subclass::simple::ClassStruct<Self>;

        glib_object_subclass!();

        fn new() -> Self {
            Self
        }
    }

    impl ObjectImpl for LinesApp {
        glib_object_impl!();
    }

    impl ApplicationImpl for LinesApp {}

    impl GtkApplicationImpl for LinesApp {}
}

glib_wrapper! {
    pub struct LinesApp(Object<
        private::LinesApp,
        LinesAppClass
    >) @extends gtk::Application, gio::Application;

    match fn {
        get_type => || private::LinesApp::get_type().to_glib(),
    }
}

impl LinesApp {
    pub const DBUS_PATH: &'static str = "net.jzimm.rs-lines";

    pub fn new() -> Result<Self, glib::Error> {
        gtk::init().expect("Failed to initialise GTK");
        let obj = glib::Object::new(Self::static_type(), &[]).expect("Instantiation error");
        let app = obj.downcast::<Self>().expect("LinesApp downcast error");

        app.connect_startup(clone!(@weak app => @default-panic, move |_| app.on_startup()));
        app.connect_activate(clone!(@weak app => @default-panic, move |_| app.on_activate()));

        app.set_application_id(Some(&Self::DBUS_PATH));
        app.set_flags(gio::ApplicationFlags::default());
        app.register(gio::Cancellable::get_current().as_ref())?;

        Ok(app)
    }

    fn on_startup(&self) {
        let app = self.upcast_ref::<gtk::Application>();

        let act_quit = gio::SimpleAction::new("quit", None);
        act_quit.connect_activate(clone!(@weak app => @default-panic, move |_,_| app.quit()));
        app.add_action(&act_quit);

        let act_new_window = gio::SimpleAction::new("new_window", None);
        act_new_window
            .connect_activate(clone!(@weak app => @default-panic, move |_,_| app.activate()));
        app.add_action(&act_new_window);

        let builder = gtk::Builder::new_from_string(APP_MENU);
        let model: gio::MenuModel = builder.get_object("appmenu").unwrap();
        app.set_app_menu(Some(&model));
    }

    fn on_activate(&self) {
        let win = crate::window::LinesWindow::get_new_ptr(self.upcast_ref());
        win.show_all();
    }
}
