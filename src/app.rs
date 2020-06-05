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

const DBUS_PATH: &'static str = "net.jzimm.rs-lines";

pub struct LinesAppPrivate;

impl ObjectSubclass for LinesAppPrivate {
    const NAME: &'static str = "LinesApp";
    type ParentType = gtk::Application;
    type Instance = glib::subclass::simple::InstanceStruct<Self>;
    type Class = glib::subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        Self
    }
}

impl ObjectImpl for LinesAppPrivate {
    glib_object_impl!();

    fn constructed(&self, obj: &glib::Object) {
        self.parent_constructed(obj);

        let app = obj.downcast_ref::<LinesApp>().unwrap();
        app.connect_startup(clone!(@weak app => @default-panic, move |_| app.on_startup()));
        app.connect_activate(clone!(@weak app => @default-panic, move |_| app.on_activate()));

        app.set_application_id(Some(&DBUS_PATH));
        app.set_flags(gio::ApplicationFlags::default());
        app.register(gio::Cancellable::get_current().as_ref())
            .expect("LinesApp registration failed");
    }
}

impl ApplicationImpl for LinesAppPrivate {}

impl GtkApplicationImpl for LinesAppPrivate {}

glib_wrapper! {
    pub struct LinesApp(Object<
        LinesAppPrivate,
        LinesAppPrivateClass
    >) @extends gtk::Application, gio::Application;

    match fn {
        get_type => || LinesAppPrivate::get_type().to_glib(),
    }
}

impl LinesApp {
    pub fn new() -> Self {
        let obj =
            glib::Object::new(Self::static_type(), &[]).expect("LinesApp instantiation error");
        obj.downcast::<Self>().expect("LinesApp downcast error")
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
        use crate::window::*;
        
        let win = LinesWindow::new();
        self.add_window(&win);
        win.show_all();
    }
}
