// wengwengweng

//! Window & Graphics

use glutin::dpi::*;
use derive_more::*;

use super::*;
use crate::math::*;
use crate::*;

pub trait Window {

	fn set_fullscreen(&mut self, b: bool);
	fn is_fullscreen(&self) -> bool;
	fn toggle_fullscreen(&mut self);
	fn set_cursor_hidden(&mut self, b: bool);
	fn is_cursor_hidden(&self) -> bool;
	fn toggle_cursor_hidden(&mut self);
	fn set_cursor_locked(&mut self, b: bool) -> Result<()>;
	fn is_cursor_locked(&self) -> bool;
	fn toggle_cursor_locked(&mut self);
	fn set_title(&mut self, t: &str);
	fn width(&self) -> i32;
	fn height(&self) -> i32;

}

impl Window for Ctx {

	fn set_fullscreen(&mut self, b: bool) {

		let window = self.windowed_ctx.window();

		if b {
			window.set_fullscreen(Some(window.get_current_monitor()));
			self.fullscreen = true;
		} else {
			window.set_fullscreen(None);
			self.fullscreen = false;
		}

	}

	fn is_fullscreen(&self) -> bool {
		return self.fullscreen;
	}

	fn toggle_fullscreen(&mut self) {
		self.set_fullscreen(!self.is_fullscreen());
	}

	fn set_cursor_hidden(&mut self, b: bool) {
		self.windowed_ctx.window().hide_cursor(b);
		self.cursor_hidden = b;
	}

	fn is_cursor_hidden(&self) -> bool {
		return self.cursor_hidden;
	}

	fn toggle_cursor_hidden(&mut self) {
		self.set_cursor_hidden(!self.is_cursor_hidden());
	}

	fn set_cursor_locked(&mut self, b: bool) -> Result<()> {
		self.windowed_ctx.window().grab_cursor(b)?;
		self.cursor_locked = b;
		return Ok(());
	}

	fn is_cursor_locked(&self) -> bool {
		return self.cursor_locked;
	}

	fn toggle_cursor_locked(&mut self) {
		self.set_cursor_locked(!self.is_cursor_locked());
	}

	fn set_title(&mut self, t: &str) {
		self.windowed_ctx.window().set_title(t);
	}

	fn width(&self) -> i32 {
		return self.width;
	}

	fn height(&self) -> i32 {
		return self.height;
	}
}

pub(super) fn swap(ctx: &window::Ctx) -> Result<()> {
	return Ok(ctx.windowed_ctx.swap_buffers()?);
}

#[derive(Copy, Clone, PartialEq, Debug, Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign, DivAssign, From, Into)]
pub struct Pos {
	pub x: i32,
	pub y: i32,
}

impl Pos {
	pub(super) fn new(x: i32, y: i32) -> Self {
		return Self {
			x: x,
			y: y,
		};
	}
}

impl From<Pos> for Vec2 {
	fn from(mpos: Pos) -> Self {
		return vec2!(mpos.x, mpos.y);
	}
}

impl From<LogicalPosition> for Pos {
	fn from(pos: LogicalPosition) -> Self {
		let (x, y): (i32, i32) = pos.into();
		return Self {
			x: x,
			y: y,
		};
	}
}

impl From<Pos> for LogicalPosition {
	fn from(pos: Pos) -> Self {
		return Self {
			x: pos.x as f64,
			y: pos.y as f64,
		};
	}
}

impl From<glutin::MouseScrollDelta> for Pos {
	fn from(delta: glutin::MouseScrollDelta) -> Self {
		use glutin::MouseScrollDelta;
		match delta {
			MouseScrollDelta::PixelDelta(pos) => {
				let (x, y): (i32, i32) = pos.into();
				return Self {
					x: x,
					y: y,
				};
			},
			MouseScrollDelta::LineDelta(x, y) => {
				return Self {
					x: x as i32,
					y: y as i32,
				};
			}
		};
	}
}

impl From<Vec2> for LogicalPosition {
	fn from(pos: Vec2) -> Self {
		return Self {
			x: pos.x as f64,
			y: pos.y as f64,
		};
	}
}

impl From<LogicalPosition> for Vec2 {
	fn from(pos: LogicalPosition) -> Self {
		return Self {
			x: pos.x as f32,
			y: pos.y as f32,
		};
	}
}
