use std::cell::RefCell;

use gtk::prelude::*;
use gtk::subclass::prelude::*;

use glib;
use glib::translate::*;

use crate::lines_area::*;

const MODES: [&'static str; 3] = ["Lines", "Triangles", "Curves"];
const PERIOD: u32 = 20;

pub struct LinesWindowPrivate {
    lines_area: LinesArea,
    menu: RefCell<Vec<gtk::RadioMenuItem>>,
}

impl ObjectSubclass for LinesWindowPrivate {
    const NAME: &'static str = "LinesWindow";
    type ParentType = gtk::Window;
    type Instance = glib::subclass::simple::InstanceStruct<Self>;
    type Class = glib::subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn new() -> Self {
        Self {
            lines_area: LinesArea::new(),
            menu: RefCell::new(Vec::with_capacity(MODES.len())),
        }
    }
}

impl ObjectImpl for LinesWindowPrivate {
    glib_object_impl!();

    fn constructed(&self, obj: &glib::Object) {
        self.parent_constructed(obj);

        let lw = obj.downcast_ref::<LinesWindow>().unwrap();
        lw.set_default_size(500, 500);
        lw.add(&self.lines_area);
        lw.set_resizable(true);

        let hdr = gtk::HeaderBar::new();
        hdr.set_title(Some("Lines"));
        hdr.set_show_close_button(true);

        let menu = gtk::Menu::new();
        let mut menu_vec = self.menu.borrow_mut();
        menu_vec.extend(
            MODES
                .iter()
                .map(|label| gtk::RadioMenuItem::new_with_label(label)),
        );
        let mut group = None;
        for item in menu_vec.iter() {
            menu.append(item);
            item.show();
            item.connect_toggled(clone!(@weak lw => @default-panic, move |m|
                if m.get_active() {let mode = m.get_label().unwrap();
                    lw.on_switch(&mode);
                }
            ));

            if group.is_some() {
                item.join_group(group);
            } else {
                group = Some(item)
            }
        }

        let mb = gtk::MenuButton::new();
        MenuButtonExt::set_direction(&mb, gtk::ArrowType::None);
        mb.set_popup(Some(&menu));
        hdr.pack_end(&mb);

        lw.set_titlebar(Some(&hdr));

        timeout_add(
            PERIOD,
            clone!(@weak lw => @default-panic, move || lw.on_tick()),
        );
    }
}

impl WidgetImpl for LinesWindowPrivate {}

impl ContainerImpl for LinesWindowPrivate {}

impl BinImpl for LinesWindowPrivate {}

impl WindowImpl for LinesWindowPrivate {}

glib_wrapper! {
    pub struct LinesWindow(Object<LinesWindowPrivate, LinesWindowClass>) @extends gtk::Widget, gtk::Container, gtk::Bin, gtk::Window;

    match fn {
        get_type => || LinesWindowPrivate::get_type().to_glib(),
    }
}

impl LinesWindow {
    pub fn new() -> Self {
        let obj = glib::Object::new(Self::static_type(), &[]).expect("Instantiation error");
        obj.downcast::<Self>().expect("LinesWindow downcast error")
    }

    fn on_tick(&self) -> glib::Continue {
        let private = LinesWindowPrivate::from_instance(self);
        private.lines_area.add_line();
        glib::Continue(true)
    }

    fn on_switch(&self, which: &str) {
        let private = LinesWindowPrivate::from_instance(self);
        private
            .lines_area
            .set_property("mode", &which.to_value())
            .unwrap();
    }
}
