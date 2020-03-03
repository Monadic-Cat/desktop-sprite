/* Copyright 2020 Alexander Hill

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License. */

extern crate mlua;
extern crate png;
extern crate sdl2;

use mlua::prelude::*;
use mlua::{Function, StdLib};

use png::{ColorType, Decoder};

use sdl2::VideoSubsystem;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{Canvas, Texture, TextureAccess};
use sdl2::video::{Window, WindowPos};

use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Duration;

struct Sprite<'a> {
	canvas: Canvas<Window>,
	state: Lua,
	textures: Vec<Texture<'a>>
}

impl Sprite<'_> {
	pub fn new(video: &VideoSubsystem, path: String) -> Result<Self, Box<dyn std::error::Error>> {
		let mut window = video.window("Desktop Sprite", 1, 1)
			.borderless()
			.build()?;
		window.set_opacity(0.0)?;
		let mut canvas = window.into_canvas().build()?;
		canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
		let state = Lua::new_with(StdLib::MATH | StdLib::TABLE | StdLib::STRING);
		state.load(&fs::read_to_string(path)?).exec()?;
		let texture_creator = Box::leak(Box::new(canvas.texture_creator())); // TODO: fix stupid
		let mut textures = Vec::new();
		{
			let globals = state.globals();
			let init: Function = globals.get("init")?;
			let (list, width, height) = init.call::<_, (Vec<String>, u32, u32)>(())?;
			canvas.window_mut().set_size(width, height)?;
			for path in list {
				let decoder = Decoder::new(File::open(path)?);
				let (info, mut reader) = decoder.read_info()?;
				let mut buffer = vec![0; info.buffer_size()];
				reader.next_frame(&mut buffer)?;
				let mut texture = texture_creator.create_texture(
					Some(match info.color_type {
						ColorType::RGB => PixelFormatEnum::RGB888,
						ColorType::RGBA => PixelFormatEnum::ARGB8888,
						_ => PixelFormatEnum::Unknown
					}),
					TextureAccess::Static,
					info.width,
					info.height
				)?;
				texture.update(None, &buffer, info.line_size)?;
				textures.push(texture);
			}
		}
		Ok(Sprite {
			canvas,
			state,
			textures
		})
	}

	pub fn tick(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		let window = self.canvas.window_mut();
		let globals = self.state.globals();
		let subsystem = window.subsystem();
		let tick: Function = globals.get("tick")?;
		let bounds = subsystem.display_bounds(window.display_index()?)?;
		let (texture, x, y) = tick.call::<_, (usize, i32, i32)>((bounds.width(), bounds.height()))?;
		window.set_position(WindowPos::Positioned(x), WindowPos::Positioned(y));
		self.canvas.clear();
		self.canvas.copy(&self.textures[texture], None, None)?;
		self.canvas.present();
		Ok(())
	}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let list = File::open("sprites.txt")?;
	let reader = BufReader::new(list);
	let context = sdl2::init()?;
	let video = context.video()?;
	let mut sprites: Vec<Sprite> = Vec::new();
	for path in reader.lines() {
		sprites.push(Sprite::new(&video, path?)?);
	}
	loop {
		for sprite in sprites.iter_mut() {
			sprite.tick()?;
		}
		thread::sleep(Duration::new(0, 1000000000u32 / 60));
	}
}
