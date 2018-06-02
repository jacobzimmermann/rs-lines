extern crate gio;
extern crate gtk;
use gio::*;

extern crate failure;
#[macro_use]
extern crate lazy_static;

const DEFAULT_WIDTH: usize = 600;
const DEFAULT_HEIGHT: usize = 600;
const PERIOD: u32 = 20;
const STEP: f64 = 10.0;
const NUM_LINES: usize = 64;
const BLEND_STEPS: usize = 30;

mod lines_area {
	use super::gio::signal;
	use super::gtk::{Allocation, DrawingArea, WidgetExt};

	extern crate cairo;
	extern crate rand;
	use self::rand::Rng;

	use std::cell::{Cell, RefCell};
	use std::f64::consts;
	use std::ops::Deref;
	use std::rc::Rc;

	struct Point {
		x: f64,
		y: f64,

		a: f64,
		da: f64,

		rng: self::rand::ThreadRng,
	}

	impl Point {
		fn new(x: f64, y: f64, a: f64) -> Point {
			let rng = self::rand::thread_rng();
			Point {
				x: x,
				y: y,
				a: a,
				da: 0.0,
				rng: rng,
			}
		}

		fn get_random_da(&mut self) -> f64 {
			let s: f64 = self.rng.gen_range(-consts::FRAC_PI_6, consts::FRAC_PI_6);
			s.abs() * s * s * s
		}

		fn step(&mut self, xbound: f64, ybound: f64) {
			let dx = super::STEP * self.a.cos();
			let dy = super::STEP * self.a.sin();
			self.a += self.da;

			self.x += dx;
			self.y += dy;

			if self.x < 0.0 {
				self.a = consts::PI - self.a;
				self.da = self.get_random_da();
				self.x = 0.0;
			} else if self.x > xbound {
				self.a = consts::PI - self.a;
				self.da = self.get_random_da();
				self.x = xbound;
			}
			if self.y < 0.0 {
				self.a = -self.a;
				self.da = self.get_random_da();
				self.y = 0.0;
			} else if self.y > ybound {
				self.a = -self.a;
				self.da = self.get_random_da();
				self.y = ybound;
			}
		}
	}

	#[derive(Clone, Copy)]
	enum LinesObjType {
		Line(f64, f64, f64, f64),
		Triangle(f64, f64, f64, f64, f64, f64),
		Curve(f64, f64, f64, f64, f64, f64, f64, f64),
	}

	#[derive(Clone, Copy)]
	struct LinesObj {
		obj: LinesObjType,

		r: f64,
		g: f64,
		b: f64,
	}

	impl LinesObj {
		fn new_for(obj: LinesObjType, a: f64, z: f64) -> LinesObj {
			LinesObj {
				obj: obj,
				r: z.sin() * a.cos(),
				g: z.sin() * a.sin(),
				b: z.cos(),
			}
		}

		fn line(p0: &Point, p1: &Point, a: f64, z: f64) -> LinesObj {
			Self::new_for(LinesObjType::Line(p0.x, p0.y, p1.x, p1.y), a, z)
		}

		fn triangle(p0: &Point, p1: &Point, p2: &Point, a: f64, z: f64) -> LinesObj {
			Self::new_for(
				LinesObjType::Triangle(p0.x, p0.y, p1.x, p1.y, p2.x, p2.y),
				a,
				z,
			)
		}

		fn curve(p0: &Point, p1: &Point, p2: &Point, p3: &Point, a: f64, z: f64) -> LinesObj {
			Self::new_for(
				LinesObjType::Curve(p0.x, p0.y, p1.x, p1.y, p2.x, p2.y, p3.x, p3.y),
				a,
				z,
			)
		}
		fn draw(&self, cr: &cairo::Context) {
			cr.set_source_rgb(self.r, self.g, self.b);
			match self.obj {
				LinesObjType::Line(x0, y0, x1, y1) => {
					cr.move_to(x0, y0);
					cr.line_to(x1, y1);
				}
				LinesObjType::Triangle(x0, y0, x1, y1, x2, y2) => {
					cr.move_to(x0, y0);
					cr.line_to(x1, y1);
					cr.line_to(x2, y2);
					cr.close_path();
				}
				LinesObjType::Curve(x0, y0, x1, y1, x2, y2, x3, y3) => {
					cr.move_to(x0, y0);
					cr.curve_to(x1, y1, x2, y2, x3, y3);
				}
			}
			cr.stroke();
		}
	}

	#[derive(Clone, Copy)]
	pub enum Mode {
		Lines,
		Triangles,
		Curves,
	}

	pub struct LinesArea {
		drawing_area: DrawingArea,

		mode: Cell<Mode>,
		lines: RefCell<[Option<LinesObj>; super::NUM_LINES]>,
		ix: Cell<usize>,
		pts: RefCell<(Point, Point, Point, Point)>,

		rng: RefCell<self::rand::ThreadRng>,

		col_az: Cell<(f64, f64)>,
		dcol_az: Cell<(f64, f64)>,
		col_step: Cell<usize>,

		size: Cell<(f64, f64)>,
	}
	pub type LinesAreaPtr = Rc<LinesArea>;

	impl LinesArea {
		pub fn new(mode: Mode) -> LinesAreaPtr {
			let w = super::DEFAULT_WIDTH as f64;
			let h = super::DEFAULT_HEIGHT as f64;
			let mut rng = self::rand::thread_rng();

			let a = rng.gen_range(0.0, consts::FRAC_PI_2);
			let z = rng.gen_range(0.0, consts::PI);

			let la = LinesAreaPtr::new(LinesArea {
				drawing_area: DrawingArea::new(),

				mode: Cell::new(mode),
				lines: RefCell::new([None; super::NUM_LINES]),
				ix: Cell::new(0),
				pts: RefCell::new((
					Point::new(0.0, h / 3.0, 0.1),
					Point::new(w, h / 6.0, -0.22),
					Point::new(w / 3.0, 0.0, 0.2),
					Point::new(w / 3.0, h, -0.2),
				)),

				rng: RefCell::new(rng),

				col_az: Cell::new((a, z)),
				dcol_az: Cell::new((0.0, 0.0)),
				col_step: Cell::new(0),

				size: Cell::new((w, h)),
			});

			let wk_draw = Rc::downgrade(&la);
			la.connect_draw(move |_, cr| {
				wk_draw.upgrade().expect("draw pointer is nil").on_draw(cr)
			});

			let wk_size_allocate = Rc::downgrade(&la);
			la.connect_size_allocate(move |_, s| {
				wk_size_allocate
					.upgrade()
					.expect("size allocate pointer is nil")
					.on_size_allocate(s)
			});

			la
		}

		pub fn add_line(&self) {
			let mut pts = self.pts.borrow_mut();

			let (w, h) = self.size.get();

			let mut az = self.col_az.get();
			let s = self.col_step.get();

			let daz = if s > 0 {
				self.col_step.set(s - 1);
				self.dcol_az.get()
			} else {
				let mut rng = self.rng.borrow_mut();
				let na = rng.gen_range(0.0, consts::FRAC_PI_2);
				let nz = rng.gen_range(0.0, consts::PI);
				let nda = (na - az.0) / super::BLEND_STEPS as f64;
				let ndz = (nz - az.1) / super::BLEND_STEPS as f64;
				let ndaz = (nda, ndz);
				self.col_step.set(super::BLEND_STEPS);
				self.dcol_az.set(ndaz);
				ndaz
			};
			az.0 += daz.0;
			az.1 += daz.1;
			self.col_az.set(az);

			pts.0.step(w, h);
			pts.1.step(w, h);
			pts.2.step(w, h);
			pts.3.step(w, h);

			let ix = self.ix.get();
			self.ix.set((ix + 1) & (super::NUM_LINES - 1));
			self.lines.borrow_mut()[ix] = Some(match self.mode.get() {
				Mode::Lines => LinesObj::line(&pts.0, &pts.1, az.0, az.1),
				Mode::Triangles => LinesObj::triangle(&pts.0, &pts.1, &pts.2, az.0, az.1),
				Mode::Curves => LinesObj::curve(&pts.0, &pts.1, &pts.2, &pts.3, az.0, az.1),
			});
			self.queue_draw();
		}

		pub fn set_mode(&self, mode: Mode) {
			self.mode.set(mode)
		}
	}

	impl Deref for LinesArea {
		type Target = DrawingArea;

		fn deref(&self) -> &Self::Target {
			&self.drawing_area
		}
	}

	trait ILinesArea {
		fn on_draw(&self, cr: &cairo::Context) -> signal::Inhibit;
		fn on_size_allocate(&self, size: &Allocation);
	}

	impl ILinesArea for LinesAreaPtr {
		fn on_draw(&self, cr: &cairo::Context) -> signal::Inhibit {
			cr.set_line_width(1.0);
			let lines = self.lines.borrow();
			let mut ix = self.ix.get();
			let last = ix;
			ix = (ix + 1) & (super::NUM_LINES - 1);
			while ix != last {
				lines[ix].map(|l| l.draw(cr));
				ix = (ix + 1) & (super::NUM_LINES - 1);
			}

			signal::Inhibit(false)
		}

		fn on_size_allocate(&self, size: &Allocation) {
			self.size.set((size.width as f64, size.height as f64));
		}
	}
}

mod app {
	use super::failure;
	use super::gio::{ApplicationExt, ApplicationFlags};
	use super::gtk::*;
	use super::lines_area::*;

	use std::cell::RefCell;
	use std::collections::HashMap;
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

	lazy_static! {
		static ref MODES: HashMap<&'static str, Mode> = {
			let mut hash = HashMap::new();
			hash.insert("Lines", Mode::Lines);
			hash.insert("Triangles", Mode::Triangles);
			hash.insert("Curves", Mode::Curves);
			hash
		};
	}

	struct LinesWindow {
		window: ApplicationWindow,
		lines_area: LinesAreaPtr,
		modes_menu: RefCell<HashMap<&'static str, RadioMenuItem>>,
	}
	type LinesWindowPtr = Rc<LinesWindow>;

	impl Deref for LinesWindow {
		type Target = ApplicationWindow;

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
			MODES.get(mode).map(|&m| self.lines_area.set_mode(m));
		}

		fn set_mode(&self, mode: &str) {
			self.modes_menu
				.borrow()
				.get(mode)
				.map(|m_item| m_item.set_active(true));
			self.on_switch(mode);
		}
	}

	impl LinesWindow {
		fn new(app: &super::gtk::Application) -> LinesWindowPtr {
			let lwin = LinesWindowPtr::new(LinesWindow {
				window: ApplicationWindow::new(app),
				lines_area: LinesArea::new(Mode::Lines),
				modes_menu: RefCell::new(HashMap::new()),
			});

			lwin.set_default_size(super::DEFAULT_WIDTH as i32, super::DEFAULT_HEIGHT as i32);
			lwin.set_resizable(true);

			let hb = HeaderBar::new();
			hb.set_title("Lines");
			hb.set_show_close_button(true);

			let mb = MenuButton::new();
			MenuButtonExt::set_direction(&mb, ArrowType::None);
			let menu = Menu::new();

			// Here we borrow modes_menu mutably
			{
				let mut modes_menu = lwin.modes_menu.borrow_mut();

				for k in MODES.keys() {
					let m_item = RadioMenuItem::new_with_label(k);
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
			} // End of mutable borrowing of modes_menu

			mb.set_popup(&menu);
			hb.pack_end(&mb);
			lwin.set_titlebar(Some(&hb));
			lwin.add(&lwin.lines_area as &DrawingArea);

			let time_lwin = Rc::clone(&lwin);
			timeout_add(super::PERIOD, move || {
				time_lwin.lines_area.add_line();
				Continue(true)
			});

			lwin
		}
	}

	pub struct Lines(super::gtk::Application);
	pub type LinesPtr = Rc<Lines>;

	impl Deref for Lines {
		type Target = super::gtk::Application;

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
			use super::gio::{ActionMapExt, MenuModel, SimpleAction, SimpleActionExt};

			let act_quit = SimpleAction::new("quit", None);
			let wk_quit = Rc::downgrade(&self);
			act_quit.connect_activate(move |_, _| {
				wk_quit.upgrade().expect("quit pointer is nil").quit()
			});
			self.add_action(&act_quit);

			let act_new_window = SimpleAction::new("new_window", None);
			let wk_new_window = Rc::downgrade(&self);
			act_new_window.connect_activate(move |_, _| {
				wk_new_window
					.upgrade()
					.expect("new_window pointer is nil")
					.activate()
			});
			self.add_action(&act_new_window);

			let builder = Builder::new_from_string(APP_MENU);
			let model: MenuModel = builder.get_object("appmenu").unwrap();
			self.set_app_menu(&model);
		}

		fn on_activate(&self) {
			let w = LinesWindow::new(&self);
			w.show_all();
		}
	}

	impl Lines {
		pub const DBUS_PATH: &'static str = "net.jzimm.lines";

		pub fn new() -> Result<LinesPtr, failure::Error> {
			let gtk_app =
				super::gtk::Application::new(Self::DBUS_PATH, ApplicationFlags::FLAGS_NONE)?;
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
}

fn main() -> Result<(), failure::Error> {
	use std::env;

	app::Lines::new().map(|app| {
		let args: Vec<String> = env::args().collect();
		app.run(&args);
	})
}
