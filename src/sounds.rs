use bevy::{audio::Volume, prelude::*};

use crate::{
	binary::{MOVE_SOUND_HANDLE, START_SOUND_HANDLE, TAKE_SOUND_HANDLE},
	components::{MoveEvent, TakeEvent}
};

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, (play_take_sound_system, play_move_sound_system))
			.add_systems(Startup, play_start_sound);
	}
}

fn play_move_sound_system(
	mut ev_move: EventReader<MoveEvent>,
	mut ev_take: EventReader<TakeEvent>,
	asset_server: Res<AssetServer>,
	mut commands: Commands
) {
	for event in ev_move.iter() {
		if event.0.is_some() && ev_take.iter().count() == 0 {
			commands.spawn(AudioBundle {
				source: MOVE_SOUND_HANDLE.typed(),
				settings: PlaybackSettings::ONCE.with_volume(Volume::new_absolute(0.5))
			});
		}
	}
}

fn play_take_sound_system(
	mut ev_take: EventReader<TakeEvent>,
	asset_server: Res<AssetServer>,
	mut commands: Commands
) {
	if ev_take.iter().count() > 0 {
		commands.spawn(AudioBundle {
			source: TAKE_SOUND_HANDLE.typed(),
			settings: PlaybackSettings::ONCE.with_volume(Volume::new_absolute(0.5))
		});
	}
}

fn play_start_sound(asset_server: Res<AssetServer>, mut commands: Commands) {
	commands.spawn(AudioBundle {
		source: START_SOUND_HANDLE.typed(),
		settings: PlaybackSettings::ONCE.with_volume(Volume::new_absolute(0.5))
	});
}
