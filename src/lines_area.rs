use gtk::*;
use rand::prelude::*;

use std::cell::{Cell, RefCell};
use std::f64::consts;
use std::ops::Deref;
use std::rc::Rc;

pub const DEFAULT_WIDTH: usize = 600;
pub const DEFAULT_HEIGHT: usize = 600;
pub const PERIOD: u32 = 20;
const STEP: f64 = 10.0;
const NUM_LINES: usize = 64;
const BLEND_STEPS: usize = 30;

struct Point {
    x: f64,
    y: f64,

    a: f64,
    da: f64,

    rng: ThreadRng,
}

impl Point {
    fn new(x: f64, y: f64, a: f64) -> Self {
        let rng = rand::thread_rng();
        Point {
            x,
            y,
            a,
            da: 0.0,
            rng,
        }
    }

    fn get_random_da(&mut self) -> f64 {
        let s: f64 = self.rng.gen_range(-consts::FRAC_PI_6, consts::FRAC_PI_6);
        s.abs() * s * s * s
    }

    fn step(&mut self, xbound: f64, ybound: f64) {
        let dx = STEP * self.a.cos();
        let dy = STEP * self.a.sin();
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
    fn new_for(obj: LinesObjType, a: f64, z: f64) -> Self {
        LinesObj {
            obj,
            r: z.sin() * a.cos(),
            g: z.sin() * a.sin(),
            b: z.cos(),
        }
    }

    fn line(p0: &Point, p1: &Point, a: f64, z: f64) -> Self {
        Self::new_for(LinesObjType::Line(p0.x, p0.y, p1.x, p1.y), a, z)
    }

    fn triangle(p0: &Point, p1: &Point, p2: &Point, a: f64, z: f64) -> Self {
        Self::new_for(
            LinesObjType::Triangle(p0.x, p0.y, p1.x, p1.y, p2.x, p2.y),
            a,
            z,
        )
    }

    fn curve(p0: &Point, p1: &Point, p2: &Point, p3: &Point, a: f64, z: f64) -> Self {
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
    lines: RefCell<[Option<LinesObj>; NUM_LINES]>,
    ix: Cell<usize>,
    pts: RefCell<(Point, Point, Point, Point)>,

    rng: RefCell<ThreadRng>,

    col_az: Cell<(f64, f64)>,
    dcol_az: Cell<(f64, f64)>,
    col_step: Cell<usize>,

    size: Cell<(f64, f64)>,
}
pub type LinesAreaPtr = Rc<LinesArea>;

impl LinesArea {
    fn new(mode: Mode) -> Self {
        let w = DEFAULT_WIDTH as f64;
        let h = DEFAULT_HEIGHT as f64;
        let mut rng = rand::thread_rng();

        let a = rng.gen_range(0.0, consts::FRAC_PI_2);
        let z = rng.gen_range(0.0, consts::PI);

        LinesArea {
            drawing_area: DrawingArea::new(),

            mode: Cell::new(mode),
            lines: RefCell::new([None; NUM_LINES]),
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
        }
    }
    pub fn get_new_ptr(mode: Mode) -> LinesAreaPtr {
        let la = LinesAreaPtr::new(Self::new(mode));

        let wk_draw = Rc::downgrade(&la);
        la.connect_draw(move |_, cr| wk_draw.upgrade().expect("draw pointer is nil").on_draw(cr));

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
            let nda = (na - az.0) / BLEND_STEPS as f64;
            let ndz = (nz - az.1) / BLEND_STEPS as f64;
            let ndaz = (nda, ndz);
            self.col_step.set(BLEND_STEPS);
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
        self.ix.set((ix + 1) & (NUM_LINES - 1));
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
    fn on_draw(&self, cr: &cairo::Context) -> glib::signal::Inhibit;
    fn on_size_allocate(&self, size: &Allocation);
}

impl ILinesArea for LinesAreaPtr {
    fn on_draw(&self, cr: &cairo::Context) -> glib::signal::Inhibit {
        cr.set_line_width(1.0);
        let lines = self.lines.borrow();
        let mut ix = self.ix.get();
        let last = ix;
        ix = (ix + 1) & (NUM_LINES - 1);
        while ix != last {
            if let Some(l) = lines[ix] {
                l.draw(cr);
            }
            ix = (ix + 1) & (NUM_LINES - 1);
        }

        glib::signal::Inhibit(false)
    }

    fn on_size_allocate(&self, size: &Allocation) {
        self.size
            .set((f64::from(size.width), f64::from(size.height)));
    }
}
