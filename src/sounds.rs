#![allow(dead_code)]

use bevy::prelude::*;

use crate::components::{MoveEvent, TakeEvent};

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
	fn build(&self, app: &mut App) {
		app.add_startup_system(play_start_sound)
			.add_system(play_take_sound_system)
			.add_system(play_move_sound_system);
	}
}

pub fn play_move_sound_system(
	mut ev_move: EventReader<MoveEvent>,
	mut ev_take: EventReader<TakeEvent>,
	asset_server: Res<AssetServer>,
	audio: Res<Audio>,
) {
	for i in ev_move.iter() {
		if i.row.is_some() && ev_take.iter().count() == 0 {
			let music = asset_server.load("sounds/move.ogg");
			audio.play(music);
		}
	}
}

pub fn play_take_sound_system(
	mut ev_take: EventReader<TakeEvent>,
	asset_server: Res<AssetServer>,
	audio: Res<Audio>,
) {
	if ev_take.iter().count() > 0 {
		let music = asset_server.load("sounds/take.ogg");
		audio.play(music);
	}
}

pub fn play_start_sound(asset_server: Res<AssetServer>, audio: Res<Audio>) {
	let music = asset_server.load("sounds/start.ogg");
	audio.play(music);
}
