use std::cell::{Cell, RefCell};
use std::f64::consts;

use cairo;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use glib::subclass::Property;
use glib::translate::*;
use glib::{ParamFlags, ParamSpec};

use rand::prelude::*;

const DEFAULT_STEP: u32 = 10;
const DEFAULT_BLEND: u32 = 16;
const DEFAULT_NUM_LINES: u32 = 64;
const MAX_NUM_LINES: u32 = 256;

static PROPERTIES: [Property; 3] = [
    Property("step", |name| {
        ParamSpec::uint(
            name,
            "Step",
            "Step length between lines",
            0,
            100,
            DEFAULT_STEP,
            ParamFlags::READWRITE,
        )
    }),
    Property("blendsteps", |name| {
        ParamSpec::uint(
            name,
            "Blendsteps",
            "Number of steps to transition to another color",
            1,
            128,
            DEFAULT_BLEND,
            ParamFlags::READWRITE,
        )
    }),
    Property("numlines", |name| {
        ParamSpec::uint(
            name,
            "NumLines",
            "Number of simultaneous lines",
            1,
            MAX_NUM_LINES,
            DEFAULT_NUM_LINES,
            ParamFlags::READWRITE,
        )
    }),
];

#[derive(Copy, Clone)]
pub enum Mode {
    Lines,
    Triangles,
    Curves,
}

impl From<Mode> for &'static str {
    fn from(m: Mode) -> Self {
        use Mode::*;
        match m {
            Lines => "lines",
            Triangles => "triangles",
            Curves => "curves",
        }
    }
}

impl From<&str> for Mode {
    fn from(s: &str) -> Self {
        use Mode::*;
        match s {
            "lines" => Lines,
            "trianges" => Triangles,
            "curves" => Curves,
            _ => unimplemented!(),
        }
    }
}

#[derive(Copy, Clone)]
struct Point {
    x: f64,
    y: f64,

    a: f64,
    da: f64,
}

impl Point {
    fn new(x: f64, y: f64, a: f64) -> Self {
        Self {
            x: x,
            y: y,
            a: a,
            da: 0.0,
        }
    }

    fn random_da(rng: &mut ThreadRng) -> f64 {
        let s: f64 = rng.gen_range(-consts::FRAC_PI_6, consts::FRAC_PI_6);
        s.abs() * s * s * s
    }

    fn step(&mut self, rng: &mut ThreadRng, step: f64, xbound: f64, ybound: f64) {
        let dx = step * self.a.cos();
        let dy = step * self.a.sin();
        self.a += self.da;

        self.x += dx;
        self.y += dy;

        if self.x < 0.0 {
            self.a = consts::PI - self.a;
            self.da = Self::random_da(rng);
            self.x = 0.0;
        } else if self.x > xbound {
            self.a = consts::PI - self.a;
            self.da = Self::random_da(rng);
            self.x = xbound;
        }
        if self.y < 0.0 {
            self.a = -self.a;
            self.da = Self::random_da(rng);
            self.y = 0.0;
        } else if self.y > ybound {
            self.a = -self.a;
            self.da = Self::random_da(rng);
            self.y = ybound;
        }
    }
}

impl Default for Point {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

#[derive(Clone, Copy)]
enum LineItemType {
    Line(f64, f64, f64, f64),
    Triangle(f64, f64, f64, f64, f64, f64),
    Curve(f64, f64, f64, f64, f64, f64, f64, f64),
}

#[derive(Clone, Copy)]
struct LineItem {
    item: LineItemType,

    r: f64,
    g: f64,
    b: f64,
}

impl LineItem {
    fn new_for(item: LineItemType, a: f64, z: f64) -> Self {
        Self {
            item,
            r: z.sin() * a.cos(),
            g: z.sin() * a.sin(),
            b: z.cos(),
        }
    }

    fn line(p0: &Point, p1: &Point, az: (f64, f64)) -> Self {
        Self::new_for(LineItemType::Line(p0.x, p0.y, p1.x, p1.y), az.0, az.1)
    }

    fn triangle(p0: &Point, p1: &Point, p2: &Point, az: (f64, f64)) -> Self {
        Self::new_for(
            LineItemType::Triangle(p0.x, p0.y, p1.x, p1.y, p2.x, p2.y),
            az.0,
            az.1,
        )
    }

    fn curve(p0: &Point, p1: &Point, p2: &Point, p3: &Point, az: (f64, f64)) -> Self {
        Self::new_for(
            LineItemType::Curve(p0.x, p0.y, p1.x, p1.y, p2.x, p2.y, p3.x, p3.y),
            az.0,
            az.1,
        )
    }

    fn draw(&self, cr: &cairo::Context) {
        cr.set_source_rgb(self.r, self.g, self.b);
        match self.item {
            LineItemType::Line(x0, y0, x1, y1) => {
                cr.move_to(x0, y0);
                cr.line_to(x1, y1);
            }
            LineItemType::Triangle(x0, y0, x1, y1, x2, y2) => {
                cr.move_to(x0, y0);
                cr.line_to(x1, y1);
                cr.line_to(x2, y2);
                cr.close_path();
            }
            LineItemType::Curve(x0, y0, x1, y1, x2, y2, x3, y3) => {
                cr.move_to(x0, y0);
                cr.curve_to(x1, y1, x2, y2, x3, y3);
            }
        }
        cr.stroke();
    }
}

pub struct LinesAreaPrivate {
    step: Cell<f64>,
    blend: Cell<f64>,
    numlines: Cell<usize>,
    mode: Cell<Mode>,

    col_az: Cell<(f64, f64)>,
    dcol_az: Cell<(f64, f64)>,
    col_step: Cell<usize>,

    lines: RefCell<Vec<LineItem>>,
    pts: RefCell<[Point; 4]>,
    ix: Cell<usize>,

    rng: RefCell<ThreadRng>,
}

impl ObjectSubclass for LinesAreaPrivate {
    const NAME: &'static str = "LinesArea";
    type ParentType = gtk::DrawingArea;
    type Instance = glib::subclass::simple::InstanceStruct<Self>;
    type Class = glib::subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES)
    }

    fn new() -> Self {
        let mut rng = thread_rng();
        let a = rng.gen_range(0.0, consts::FRAC_PI_2);
        let z = rng.gen_range(0.0, consts::PI);

        Self {
            step: Cell::new(DEFAULT_STEP as f64),
            blend: Cell::new(DEFAULT_BLEND as f64),
            numlines: Cell::new(DEFAULT_NUM_LINES as usize),
            mode: Cell::new(Mode::Lines),

            col_az: Cell::new((a, z)),
            dcol_az: Cell::new((0.0, 0.0)),
            col_step: Cell::new(0),

            lines: RefCell::new(Vec::with_capacity(MAX_NUM_LINES as usize)),
            pts: RefCell::new([Point::default(); 4]),
            ix: Cell::new(0),

            rng: RefCell::new(rng),
        }
    }
}

impl ObjectImpl for LinesAreaPrivate {
    glib_object_impl!();

    fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
        match PROPERTIES[id] {
            Property("step", ..) => {
                let step: u32 = value
                    .get()
                    .expect("Object::set_property() type check for step")
                    .unwrap();
                self.step.set(step as f64);
            }
            Property("blendsteps", ..) => {
                let blend: u32 = value
                    .get()
                    .expect("Object::set_property() type check for blendsteps")
                    .unwrap();
                self.blend.set(blend as f64);
            }
            Property("numlines", ..) => {
                let numlines: u32 = value
                    .get()
                    .expect("Object::set_property() type check for numlines")
                    .unwrap();
                self.numlines.set(numlines as usize);
            }
            Property("mode", ..) => {
                let mode: &str = value
                    .get()
                    .expect("Object::set_property() type check for mode")
                    .unwrap();
                self.mode.set(From::from(mode));
            }
            _ => unimplemented!("Unsupported property"),
        }
    }

    fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
        match PROPERTIES[id] {
            Property("step", ..) => Ok((self.step.get() as f64).to_value()),
            Property("blendsteps", ..) => Ok((self.blend.get() as f64).to_value()),
            Property("numlines", ..) => Ok((self.numlines.get() as u32).to_value()),
            Property("mode", ..) => Ok(<&str>::from(self.mode.get()).to_value()),
            _ => unimplemented!("Unsupported property"),
        }
    }

    fn constructed(&self, obj: &glib::Object) {
        self.parent_constructed(obj);

        let la = obj.downcast_ref::<LinesArea>().unwrap();
        la.connect_realize(clone!(@weak la => @default-panic, move |_| la.on_realize()));
    }
}

impl WidgetImpl for LinesAreaPrivate {
    fn draw(&self, _w: &gtk::Widget, cr: &cairo::Context) -> gtk::Inhibit {
        cr.set_line_width(1.0);

        let lines = self.lines.borrow();
        let numlines = self.numlines.get();

        if lines.len() < numlines {
            for li in lines.iter() {
                li.draw(cr)
            }
        } else {
            let oldest_ix = (self.ix.get() + 1) % numlines;

            for li in lines[oldest_ix..].iter().chain(lines.iter()).take(numlines) {
                li.draw(cr)
            }
        }
        Inhibit(true)
    }
}

impl DrawingAreaImpl for LinesAreaPrivate {}

glib_wrapper! {
    pub struct LinesArea(Object<
        LinesAreaPrivate,
        LinesAreaPrivateClass
    >) @extends gtk::DrawingArea, gtk::Widget;

    match fn {
        get_type => || LinesAreaPrivate::get_type().to_glib(),
    }
}

impl LinesArea {
    pub fn new() -> Self {
        let obj = glib::Object::new(Self::static_type(), &[]).expect("Instantiation error");
        obj.downcast::<Self>().expect("LinesArea downcast error")
    }

    fn on_realize(&self) {
        let alloc = self.get_allocation();
        let (w, h) = (alloc.width as f64, alloc.height as f64);

        let private = LinesAreaPrivate::from_instance(self);
        let mut pts = private.pts.borrow_mut();
        pts[0] = Point::new(0.0, h / 3.0, 0.1);
        pts[1] = Point::new(w, h / 6.0, -0.22);
        pts[2] = Point::new(w / 3.0, 0.0, 0.2);
        pts[3] = Point::new(w / 3.0, h, -0.2);
    }

    pub fn add_line(&self) {
        use Mode::*;

        let private = LinesAreaPrivate::from_instance(self);

        let mut rng = private.rng.borrow_mut();
        let mut pts = private.pts.borrow_mut();
        let mut az = private.col_az.get();
        let col_step = private.col_step.get();

        let alloc = self.get_allocation();
        let (w, h) = (alloc.width as f64, alloc.height as f64);

        let daz = if col_step > 0 {
            // We are progressing through a colour blend
            private.col_step.set(col_step - 1);
            private.dcol_az.get()
        } else {
            // We have reached a colour blend target, set a new target
            let blend = private.blend.get();

            let na = rng.gen_range(0.0, consts::FRAC_PI_2);
            let nz = rng.gen_range(0.0, consts::PI);
            let nda = (na - az.0) / blend;
            let ndz = (nz - az.1) / blend;
            let ndaz = (nda, ndz);

            private.col_step.set(blend as usize);
            private.dcol_az.set(ndaz);
            ndaz
        };

        az.0 += daz.0;
        az.1 += daz.1;
        private.col_az.set(az);

        // Move the points
        let step = private.step.get();
        for p in pts.iter_mut() {
            p.step(&mut rng, step, w, h);
        }

        let new_line = match private.mode.get() {
            Lines => LineItem::line(&pts[0], &pts[1], az),
            Triangles => LineItem::triangle(&pts[0], &pts[1], &pts[2], az),
            Curves => LineItem::curve(&pts[0], &pts[1], &pts[2], &pts[3], az),
        };

        let mut lines = private.lines.borrow_mut();
        let numlines = private.numlines.get();

        if lines.len() < numlines {
            lines.push(new_line);
        } else {
            let ix = private.ix.get();
            lines[ix] = new_line;
            private.ix.set((ix + 1) % numlines);
        }

        self.queue_draw();
    }
}
