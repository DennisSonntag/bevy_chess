#![allow(clippy::needless_pass_by_value)]
use bevy::{audio::Volume, prelude::*};

use crate::components::{MoveEvent, TakeEvent};

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, (play_take_sound_system, play_move_sound_system))
			.add_systems(Startup, play_start_sound);
	}
}

#[derive(Component)]
struct MyMusic;

pub fn play_move_sound_system(
	mut ev_move: EventReader<MoveEvent>,
	mut ev_take: EventReader<TakeEvent>,
	asset_server: Res<AssetServer>,
	mut commands: Commands,
) {
	for event in ev_move.iter() {
		if event.pos.is_some() && ev_take.iter().count() == 0 {
			commands.spawn(AudioBundle {
				source: asset_server.load("sounds/move.ogg"),
				settings: PlaybackSettings::ONCE.with_volume(Volume::new_absolute(0.5)),
			});
		}
	}
}

pub fn play_take_sound_system(
	mut ev_take: EventReader<TakeEvent>,
	asset_server: Res<AssetServer>,
	mut commands: Commands,
) {
	if ev_take.iter().count() > 0 {
		commands.spawn(AudioBundle {
			source: asset_server.load("sounds/take.ogg"),
			settings: PlaybackSettings::ONCE.with_volume(Volume::new_absolute(0.5)),
		});
	}
}

pub fn play_start_sound(asset_server: Res<AssetServer>, mut commands: Commands) {
	commands.spawn(AudioBundle {
		source: asset_server.load("sounds/start.ogg"),
		settings: PlaybackSettings::ONCE.with_volume(Volume::new_absolute(0.5)),
	});
}
