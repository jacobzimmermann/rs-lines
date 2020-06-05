use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use gtk::prelude::*;

use crate::lines_area::*;
/*
lazy_static! {
    static ref MODES: HashMap<&'static str, Mode> = {
        let mut hash = HashMap::new();
        hash.insert("Lines", Mode::Lines);
        hash.insert("Triangles", Mode::Triangles);
        hash.insert("Curves", Mode::Curves);
        hash
    };
}
*/
pub struct LinesWindow {
    window: gtk::ApplicationWindow,
    lines_area: LinesArea,
    modes_menu: RefCell<HashMap<&'static str, gtk::RadioMenuItem>>,
}
type LinesWindowPtr = Rc<LinesWindow>;

impl Deref for LinesWindow {
    type Target = gtk::ApplicationWindow;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

trait ILinesWindow {
    fn set_mode(&self, mode: &str);
    fn on_switch(&self, mode: &str);
}

impl ILinesWindow for LinesWindowPtr {
    fn on_switch(&self, mode: &str) {
        /*      if let Some(&m) = MODES.get(mode) {
            self.lines_area.set_mode(m);
        }*/
    }

    fn set_mode(&self, mode: &str) {
        if let Some(m_item) = self.modes_menu.borrow().get(mode) {
            m_item.set_active(true);
        }
        self.on_switch(mode);
    }
}

impl LinesWindow {
    fn new(app: &gtk::Application) -> Self {
        LinesWindow {
            window: gtk::ApplicationWindow::new(app),
            lines_area: LinesArea::new(),
            modes_menu: RefCell::new(HashMap::new()),
        }
    }
    /*
        fn build_menu(lwin: &LinesWindowPtr) -> gtk::Menu {
            let menu = gtk::Menu::new();
            let mut modes_menu = lwin.modes_menu.borrow_mut();

            for k in MODES.keys() {
                let m_item = gtk::RadioMenuItem::new_with_label(k);
                m_item.show();
                menu.append(&m_item);
                let wk_switch = Rc::downgrade(&lwin);
                m_item.connect_toggled(move |m| {
                    if m.get_active() {
                        let mode = m.get_label().unwrap();
                        wk_switch
                            .upgrade()
                            .expect("switch pointer is nil")
                            .on_switch(&mode)
                    }
                });
                modes_menu.insert(k, m_item);
            }
            for m in modes_menu.values() {
                m.join_group(modes_menu.get("Lines"));
            }

            menu
        }
    */
    pub fn get_new_ptr(app: &gtk::Application) -> LinesWindowPtr {
        let lwin = LinesWindowPtr::new(Self::new(app));
        lwin.set_default_size(500, 500);
        lwin.add(&lwin.lines_area);
        let time_lwin = Rc::clone(&lwin);
        timeout_add(30, move || {
            time_lwin.lines_area.add_line();
            glib::Continue(true)
        });
        /*
              lines_area::DEFAULT_WIDTH as i32,
              lines_area::DEFAULT_HEIGHT as i32,
          );
          lwin.set_resizable(true);

          let hb = gtk::HeaderBar::new();
          hb.set_title(Some("Lines"));
          hb.set_show_close_button(true);

          let mb = gtk::MenuButton::new();
          MenuButtonExt::set_direction(&mb, gtk::ArrowType::None);
          let menu = Self::build_menu(&lwin);
          mb.set_popup(Some(&menu));

          hb.pack_end(&mb);
          lwin.set_titlebar(Some(&hb));
        //  lwin.add(&lwin.lines_area as &gtk::DrawingArea);

          let time_lwin = Rc::clone(&lwin);
          timeout_add(lines_area::PERIOD, move || {
              time_lwin.lines_area.add_line();
              glib::Continue(true)
          });*/

        lwin
    }
}
