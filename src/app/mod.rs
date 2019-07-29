// wengwengweng

mod gl;
pub mod gfx;
pub mod input;
pub mod window;

use crate::*;
use crate::math::*;

pub use gfx::Gfx;
pub use input::Input;
pub use window::Window;

use std::rc::Rc;
use std::collections::HashMap;
use std::thread;
use std::time::Instant;
use std::time::Duration;

use glutin::dpi::*;
use glutin::Api;
use glutin::GlRequest;
use gilrs::Gilrs;

use input::ButtonState;
use input::Key;
use input::Mouse;

use window::Pos;

const MAX_DRAWS: usize = 65536;

const TEMPLATE_2D_VERT: &str = include_str!("../res/2d_template.vert");
const TEMPLATE_2D_FRAG: &str = include_str!("../res/2d_template.frag");

const DEFAULT_2D_VERT: &str = include_str!("../res/2d_default.vert");
const DEFAULT_2D_FRAG: &str = include_str!("../res/2d_default.frag");

const DEFAULT_FONT_IMG: &[u8] = include_bytes!("../res/CP437.png");
const DEFAULT_FONT_COLS: usize = 32;
const DEFAULT_FONT_ROWS: usize = 8;
const DEFAULT_FONT_CHARS: &str = r##" ☺☻♥♦♣♠•◘○◙♂♀♪♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼ !"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~⌂ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáíóúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╢╖╕╣║╗╝╜╛┐└┴┬├─┼╞╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■"##;

/// Manages Ctx
pub struct Ctx {

	// lifecycle
	pub(self) quit: bool,
	pub(self) dt: f32,
	pub(self) time: f32,
	pub(self) fps_cap: u16,
	pub(self) fps_counter: FPSCounter,

	// input
	pub(self) key_state: HashMap<Key, ButtonState>,
	pub(self) mouse_state: HashMap<Mouse, ButtonState>,
	pub(self) mouse_pos: Pos,
	pub(self) mouse_delta: Option<Pos>,
	pub(self) scroll_delta: Option<Pos>,
	pub(self) text_input: Option<String>,

	// window
	pub(self) title: String,
	pub(self) fullscreen: bool,
	pub(self) cursor_hidden: bool,
	pub(self) cursor_locked: bool,
	pub(self) width: i32,
	pub(self) height: i32,

	// gfx
	pub(self) windowed_ctx: glutin::WindowedContext<glutin::PossiblyCurrent>,
	pub(self) events_loop: glutin::EventsLoop,
	pub(self) gamepad_ctx: gilrs::Gilrs,

	pub(self) gl: Rc<gl::Device>,
	pub(self) batched_renderer: gl::BatchedRenderer<gfx::QuadShape>,

	pub(self) cur_tex: Option<gfx::Texture>,
	pub(self) empty_tex: gfx::Texture,

	pub(self) default_shader: gfx::Shader,
	pub(self) cur_shader: gfx::Shader,

	pub(self) default_font: gfx::Font,

	pub(self) draw_calls_last: usize,
	pub(self) draw_calls: usize,

	pub(self) state: gfx::State,
	pub(self) state_stack: Vec<gfx::State>,

}

impl Ctx {

	fn new(conf: &app::Conf) -> Result<Self> {

		let events_loop = glutin::EventsLoop::new();

		let mut window_builder = glutin::WindowBuilder::new()
			.with_title(conf.title.to_owned())
			.with_resizable(conf.resizable)
			.with_transparency(conf.transparent)
			.with_decorations(!conf.borderless)
			.with_always_on_top(conf.always_on_top)
			.with_dimensions(LogicalSize::new(conf.width as f64, conf.height as f64))
			.with_multitouch();

		if conf.fullscreen {
			window_builder = window_builder
				.with_fullscreen(Some(events_loop.get_primary_monitor()));
		}

		if cfg!(target_os = "macos") {

			use glutin::os::macos::WindowBuilderExt;

			window_builder = window_builder
				.with_titlebar_buttons_hidden(conf.hide_titlebar_buttons)
				.with_title_hidden(conf.hide_title)
				.with_titlebar_transparent(conf.titlebar_transparent)
				.with_fullsize_content_view(conf.fullsize_content);
// 				.with_disallow_hidpi(!conf.hidpi);

		}

		let windowed_ctx = glutin::ContextBuilder::new()
			.with_vsync(conf.vsync)
			.with_gl(GlRequest::Specific(Api::OpenGl, (2, 1)))
			.build_windowed(window_builder, &events_loop)?;

		let windowed_ctx = unsafe { windowed_ctx.make_current()? };

		let gl = gl::Device::from_loader(|s| {
			windowed_ctx.get_proc_address(s) as *const _
		});

		gl.enable(gl::Capability::Blend);
		gl.blend_func_sep(gl::BlendFunc::SrcAlpha, gl::BlendFunc::OneMinusSrcAlpha, gl::BlendFunc::One, gl::BlendFunc::OneMinusSrcAlpha);
		gl.clear_color(conf.clear_color);
		gl.clear();

		let batched_renderer = gl::BatchedRenderer::<gfx::QuadShape>::new(&gl, MAX_DRAWS)?;

		let empty_tex = gl::Texture::new(&gl, 1, 1)?;
		empty_tex.data(&[255, 255, 255, 255]);
		let empty_tex = gfx::Texture::from_handle(empty_tex);

		let vert_src = TEMPLATE_2D_VERT.replace("###REPLACE###", DEFAULT_2D_VERT);
		let frag_src = TEMPLATE_2D_FRAG.replace("###REPLACE###", DEFAULT_2D_FRAG);

		let shader = gfx::Shader::from_handle(gl::Program::new(&gl, &vert_src, &frag_src)?);
		let proj = gfx::Origin::TopLeft.to_ortho(conf.width, conf.height);

		shader.send("projection", proj);

		let font_img = img::Image::from_bytes(DEFAULT_FONT_IMG)?;
		let font_tex = gl::Texture::new(&gl, font_img.width() as i32, font_img.height() as i32)?;
		font_tex.data(&font_img.into_raw());
		let font_tex = gfx::Texture::from_handle(font_tex);

		let font = gfx::Font::from_tex(
			font_tex,
			DEFAULT_FONT_COLS,
			DEFAULT_FONT_ROWS,
			DEFAULT_FONT_CHARS,
		)?;

		let mut ctx = Self {

			quit: false,
			dt: 0.0,
			time: 0.0,
			fps_cap: 60,
			fps_counter: FPSCounter::new(16),

			key_state: HashMap::new(),
			mouse_state: HashMap::new(),
			mouse_pos: Pos::new(0, 0),
			mouse_delta: None,
			scroll_delta: None,
			text_input: None,
			fullscreen: conf.fullscreen,
			cursor_hidden: conf.cursor_hidden,
			cursor_locked: conf.cursor_locked,
			title: conf.title.to_owned(),
			width: conf.width,
			height: conf.height,

			events_loop: events_loop,
			windowed_ctx: windowed_ctx,
			gamepad_ctx: Gilrs::new()?,

			gl: Rc::new(gl),
			batched_renderer: batched_renderer,

			cur_tex: None,
			empty_tex: empty_tex,

			default_shader: shader.clone(),
			cur_shader: shader,

			default_font: font,

			draw_calls: 0,
			draw_calls_last: 0,

			state: gfx::State::default(),
			state_stack: Vec::with_capacity(16),

		};

		if conf.cursor_hidden {
			ctx.set_cursor_hidden(true);
		}

		if conf.cursor_locked {
			ctx.set_cursor_locked(true)?;
		}

		return Ok(ctx);

	}

}

pub trait App {
	fn quit(&mut self);
	fn dt(&self) -> f32;
	fn fps(&self) -> u16;
	fn time(&self) -> f32;
}

impl App for Ctx {

	fn quit(&mut self) {
		self.quit = true;
	}

	fn dt(&self) -> f32 {
		return self.dt;
	}

	fn fps(&self) -> u16 {
		return self.fps_counter.get_avg();
	}

	fn time(&self) -> f32 {
		return self.time;
	}
}

pub fn run<S: State>() -> Result<()> {
	return Launcher::new().run::<S>();
}

#[derive(Default)]
pub struct Launcher {
	conf: Conf,
}

impl Launcher {

	pub fn new() -> Self {
		return Self::default();
	}

	pub fn run<S: State>(self) -> Result<()> {

		let mut ctx = Ctx::new(&self.conf)?;
		let mut s = S::init(&mut ctx)?;

		loop {

			let start_time = Instant::now();

			input::poll(&mut ctx)?;

			gfx::begin(&mut ctx);
			s.run(&mut ctx)?;
			gfx::end(&mut ctx);
			window::swap(&mut ctx)?;

			if ctx.quit {
				break;
			}

			let real_dt = start_time.elapsed().as_millis();
			let expected_dt = (1000.0 / ctx.fps_cap as f32) as u128;

			if real_dt < expected_dt {
				thread::sleep(Duration::from_millis((expected_dt - real_dt) as u64));
			}

			ctx.dt = start_time.elapsed().as_millis() as f32 / 1000.0;
			ctx.time += ctx.dt;
			ctx.fps_counter.push((1.0 / ctx.dt) as u16);

		}

		return Ok(());

	}

	pub fn conf(mut self, c: Conf) -> Self {
		self.conf = c;
		return self;
	}

	pub fn size(mut self, w: i32, h: i32) -> Self {
		self.conf.width = w;
		self.conf.height = h;
		return self;
	}

	pub fn title(mut self, t: &str) -> Self {
		self.conf.title = t.to_owned();
		return self;
	}

	pub fn hidpi(mut self, b: bool) -> Self {
		self.conf.hidpi = b;
		return self;
	}

	pub fn resizable(mut self, b: bool) -> Self {
		self.conf.resizable = b;
		return self;
	}

	pub fn fullscreen(mut self, b: bool) -> Self {
		self.conf.fullscreen = b;
		return self;
	}

	pub fn vsync(mut self, b: bool) -> Self {
		self.conf.vsync = b;
		return self;
	}

	pub fn cursor_hidden(mut self, b: bool) -> Self {
		self.conf.cursor_hidden = b;
		return self;
	}

	pub fn cursor_locked(mut self, b: bool) -> Self {
		self.conf.cursor_locked = b;
		return self;
	}

	pub fn hide_title(mut self, b: bool) -> Self {
		self.conf.hide_title = b;
		return self;
	}

	pub fn hide_titlebar_buttons(mut self, b: bool) -> Self {
		self.conf.hide_titlebar_buttons = b;
		return self;
	}

	pub fn transparent(mut self, b: bool) -> Self {
		self.conf.transparent = b;
		return self;
	}

	pub fn always_on_top(mut self, b: bool) -> Self {
		self.conf.always_on_top = b;
		return self;
	}

	pub fn clear_color(mut self, c: Color) -> Self {
		self.conf.clear_color = c;
		return self;
	}

}

#[derive(Clone, Debug)]
pub struct Conf {
	pub width: i32,
	pub height: i32,
	pub title: String,
	pub hidpi: bool,
	pub resizable: bool,
	pub fullscreen: bool,
	pub always_on_top: bool,
	pub borderless: bool,
	pub transparent: bool,
	pub vsync: bool,
	pub hide_title: bool,
	pub hide_titlebar_buttons: bool,
	pub fullsize_content: bool,
	pub titlebar_transparent: bool,
	pub cursor_hidden: bool,
	pub cursor_locked: bool,
	pub clear_color: Color,
}

impl Conf {

	pub fn basic(title: &str, width: i32, height: i32) -> Self {
		return Self {
			title: String::from(title),
			width: width,
			height: height,
			..Default::default()
		};
	}

}

impl Default for Conf {

	fn default() -> Self {
		return Self {
			width: 640,
			height: 480,
			title: String::new(),
			hidpi: true,
			resizable: false,
			fullscreen: false,
			always_on_top: false,
			borderless: false,
			transparent: false,
			vsync: false,
			fullsize_content: false,
			hide_title: false,
			hide_titlebar_buttons: false,
			titlebar_transparent: false,
			cursor_hidden: false,
			cursor_locked: false,
			clear_color: color!(0, 0, 0, 1),
		};
	}

}

pub trait State {

	fn init(_: &mut Ctx) -> Result<Self> where Self: Sized;

	fn run(&mut self, _: &mut Ctx) -> Result<()> {
		return Ok(());
	}

	fn quit(&mut self, _: &mut Ctx) -> Result<()> {
		return Ok(());
	}

}

pub(super) struct FPSCounter {
	buffer: Vec<u16>,
}

impl FPSCounter {

	fn new(max: usize) -> Self {
		return Self {
			buffer: Vec::with_capacity(max),
		}
	}

	fn push(&mut self, fps: u16) {
		if self.buffer.len() == self.buffer.capacity() {
			self.buffer.remove(0);
		}
		self.buffer.push(fps);
	}

	fn get_avg(&self) -> u16 {

		if self.buffer.is_empty() {
			return 0;
		}

		let sum: u16 = self.buffer.iter().sum();
		return sum / self.buffer.len() as u16;

	}

}
