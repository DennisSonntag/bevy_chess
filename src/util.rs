use anyhow::Result;
use bevy::prelude::In;

pub const WINDOW_SIZE: f32 = 600.;
pub const SQUARE_SIZE: f32 = WINDOW_SIZE / 8.;
pub const BOARD_SIZE: i8 = 8;

pub mod macros {
	macro_rules! spawn_sprite_bundle {
		($commands:ident , $color: expr, $size: expr) => {
			$commands.spawn((SpriteBundle {
				transform: Transform {
					translation: $size,
					scale: Vec3::new(SQUARE_SIZE, SQUARE_SIZE, 0.0),
					..default()
				},
				sprite: Sprite {
					color: $color,
					..default()
				},
				..default()
			},));
		};
		($commands:expr, $color: expr, $size: expr, $component:expr) => {
			$commands
				.spawn((SpriteBundle {
					transform: Transform {
						translation: $size,
						scale: Vec3::new(SQUARE_SIZE, SQUARE_SIZE, 0.0),
						..default()
					},
					sprite: Sprite {
						color: $color,
						..default()
					},
					..default()
				},))
				.insert($component);
		};
	}
	macro_rules! spawn_text_bundle {
		($commands:expr, $text: expr, $color: expr, $size: expr,  $style: expr,$component:expr) => {
			$commands.spawn((
				Text2dBundle {
					text: Text {
						sections: vec![TextSection::new(
							String::from($text),
							TextStyle {
								color: $color,
								..$style
							},
						)],
						..default()
					},
					transform: Transform::from_translation($size),
					text_anchor: Anchor::Center,
					..default()
				},
				$component,
			));
		};
		($commands:expr, $text: expr, $color: expr, $size: expr,  $style: expr) => {
			$commands.spawn(Text2dBundle {
				text: Text {
					sections: vec![TextSection::new(
						$text,
						TextStyle {
							color: $color,
							..$style
						},
					)],
					..default()
				},
				transform: Transform::from_translation($size),
				text_anchor: Anchor::Center,
				..default()
			});
		};
	}
	pub(crate) use spawn_sprite_bundle;
	pub(crate) use spawn_text_bundle;
}

pub fn error_handler(In(result): In<Result<()>>) {
	if let Err(err) = result {
		println!("encountered an error {err:?}");
	}
}

pub fn option_handler(In(result): In<Option<()>>) {
	if let Some(err) = result {
		println!("encountered an None {err:?}");
	}
}
