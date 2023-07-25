#![allow(
	dead_code,
	unused,
	clippy::cast_sign_loss,
	clippy::cast_precision_loss,
	clippy::needless_pass_by_value
)]

use bevy::{prelude::*, sprite::Anchor, window::PresentMode};
use bevy_prototype_lyon::prelude::*;

use bevy::app::AppExit;
use binary::{BinaryPlugin, FONT_HANDLE, PIECE_HANDLE};
use components::{
	BlackTimer, BoardResource, Coord, GameTimers, HighlightSquare, HoverEvent, HoverSquare,
	LegalMoveEvent, MoveData, MoveEvent, MovedSquare, Piece, PieceColor, Position, SelectedPiece,
	TakeEvent, WhiteTimer,
};

use std::env;

use piece::PiecePlugin;
use sounds::SoundPlugin;

use chrono::Duration;
use num_traits::cast::ToPrimitive;

mod binary;
mod components;
mod piece;
mod sounds;

const WINDOW_SIZE: f32 = 600.;

const SQUARE_SIZE: f32 = WINDOW_SIZE / 8.;

const BOARD_SIZE: i8 = 8;

fn main() {
	#[cfg(not(debug_assertions))]
	env::set_var("RUST_LOG", "");

	App::new()
		.add_plugins(DefaultPlugins.set(WindowPlugin {
			primary_window: Some(Window {
				title: "chess".into(),
				resolution: (WINDOW_SIZE, WINDOW_SIZE + 100.).into(),
				present_mode: PresentMode::AutoVsync,
				fit_canvas_to_parent: true,
				prevent_default_event_handling: false,
				..default()
			}),
			..default()
		}))
		.add_plugins(ShapePlugin)
		.add_plugins(BinaryPlugin)
		.insert_resource(Msaa::Sample8)
		.init_resource::<BoardResource>()
		.init_resource::<SelectedPiece>()
		.init_resource::<MoveData>()
		.init_resource::<GameTimers>()
		.add_state::<PieceColor>()
		.add_event::<MoveEvent>()
		.add_event::<TakeEvent>()
		.add_event::<HoverEvent>()
		.add_event::<LegalMoveEvent>()
		.add_systems(
			Startup,
			(
				setup_camera,
				spawn_board_system,
				spawn_piece_sprites_system,
				spawn_timers_system,
			),
		)
		.add_systems(
			Update,
			(
				update_white_timer_system,
				update_black_timer_system,
				countdown,
			),
		)
		.add_plugins(SoundPlugin)
		.add_plugins(PiecePlugin)
		.run();
}

fn setup_camera(mut commands: Commands, mut countdown: ResMut<GameTimers>) {
	commands.spawn(Camera2dBundle::default());
	countdown.black.pause();
}

fn spawn_board_system(mut commands: Commands) {
	let text_style = TextStyle {
		font: FONT_HANDLE.typed(),
		font_size: 20.0,
		color: Color::BLACK,
	};

	for row in 1..=BOARD_SIZE {
		for col in 1..=BOARD_SIZE {
			let color = if (row + col) % 2 == 0 {
				Color::rgb_u8(162, 110, 91)
			} else {
				Color::rgb_u8(236, 210, 185)
			};

			let color_alternate = if (row + col) % 2 == 0 {
				Color::rgb_u8(236, 210, 185)
			} else {
				Color::rgb_u8(162, 110, 91)
			};

			commands.spawn((SpriteBundle {
				transform: Transform {
					translation: Vec3::new(Coord::to_win(col, -0.5), Coord::to_win(row, -0.5), 0.0),
					scale: Vec3::new(SQUARE_SIZE, SQUARE_SIZE, 0.0),
					..default()
				},
				sprite: Sprite { color, ..default() },
				..default()
			},));

			if row == 1 {
				commands.spawn((Text2dBundle {
					text: Text {
						sections: vec![TextSection::new(
							format!(
								"{}",
								char::from_u32(96 + col as u32)
									.expect("could not cast number to char")
							),
							TextStyle {
								color: color_alternate,
								..text_style.clone()
							},
						)],
						..default()
					},
					transform: Transform::from_translation(Vec3::new(
						Coord::to_win(col, -1.) + 67.,
						Coord::to_win(row, -1.) + 67.,
						1.,
					)),
					text_anchor: Anchor::Center,
					..default()
				},));
			}
			if col == 1 {
				commands.spawn((Text2dBundle {
					text: Text {
						sections: vec![TextSection::new(
							format!("{row}"),
							TextStyle {
								color: color_alternate,
								..text_style.clone()
							},
						)],
						..default()
					},
					transform: Transform::from_translation(Vec3::new(
						Coord::to_win(col, -1.) + 10.,
						Coord::to_win(row, -1.) + 10.,
						1.,
					)),
					text_anchor: Anchor::Center,
					..default()
				},));
			}
		}
	}
}

fn spawn_piece_sprites_system(
	mut commands: Commands,
	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
	board: ResMut<BoardResource>,
) {
	let texture_handle = PIECE_HANDLE.typed();

	for (index, piece) in board.0.iter().enumerate() {
		if let Some(piece) = piece {
			let row = i8::try_from(index).expect("could not cast index to i8") / BOARD_SIZE;
			let col = i8::try_from(index).expect("could not cast index to i8") % BOARD_SIZE;

			let texture_atlas = TextureAtlas::from_grid(
				texture_handle.clone(),
				Vec2::new(333.3, 333.3),
				2,
				1,
				None,
				Some(Vec2::new(
					(piece.piece_type as i32) as f32 * 333.3,
					(piece.color as i32) as f32 * 333.3,
				)),
			);
			let texture_atlas_handle = texture_atlases.add(texture_atlas);

			commands
				.spawn(SpriteSheetBundle {
					texture_atlas: texture_atlas_handle,
					transform: Transform {
						translation: Vec3::new(
							Coord::to_win_piece(col),
							Coord::to_win_piece(row),
							2.0,
						),
						scale: Vec3::splat(WINDOW_SIZE / 2500.),
						..default()
					},

					..default()
				})
				.insert(Piece {
					piece_type: piece.piece_type,
					color: piece.color,
					amount_moved: piece.amount_moved,
					pos: Position::new(row, col),
				});
		}
	}

	commands
		.spawn((SpriteBundle {
			transform: Transform {
				translation: Vec3::new(Coord::to_win(0.5, 0.), Coord::to_win(0.5, 0.), 1.0),
				scale: Vec3::new(SQUARE_SIZE, SQUARE_SIZE, 0.0),
				..default()
			},
			sprite: Sprite {
				color: Color::rgba_u8(255, 255, 0, 100),
				..default()
			},
			..default()
		},))
		.insert(HighlightSquare);

	commands
		.spawn((SpriteBundle {
			transform: Transform {
				translation: Vec3::new(Coord::to_win(-0.5, 0.), Coord::to_win(-0.5, 0.), 1.0),
				scale: Vec3::new(SQUARE_SIZE, SQUARE_SIZE, 0.0),
				..default()
			},
			sprite: Sprite {
				color: Color::rgba_u8(200, 115, 0, 100),
				..default()
			},
			..default()
		},))
		.insert(MovedSquare);

	// Hover ------------------------------------------------
	let shape = shapes::RegularPolygon {
		sides: 4,
		..shapes::RegularPolygon::default()
	};

	commands
		.spawn((
			ShapeBundle {
				path: GeometryBuilder::build_as(&shape),
				transform: Transform {
					translation: Vec3::new(Coord::to_win(-0.5, 0.), Coord::to_win(-0.5, 0.), 5.0),
					scale: Vec3::new(SQUARE_SIZE * 0.66, SQUARE_SIZE * 0.66, 0.0),
					..default()
				},
				..default()
			},
			Fill::color(Color::NONE),
			Stroke::new(Color::WHITE, 0.09),
		))
		.insert(HoverSquare);
}

fn spawn_timers_system(mut commands: Commands, asset_server: Res<AssetServer>) {
	let text_style = TextStyle {
		font: FONT_HANDLE.typed(),
		font_size: 20.0,
		color: Color::BLACK,
	};
	commands.spawn((
		Text2dBundle {
			text: Text {
				sections: vec![TextSection::new(
					String::from("00:00"),
					TextStyle {
						color: Color::WHITE,
						..text_style.clone()
					},
				)],
				..default()
			},
			transform: Transform::from_translation(Vec3::new(
				50. - (WINDOW_SIZE / 2.),
				-30. - (WINDOW_SIZE / 2.),
				2.,
			)),
			text_anchor: Anchor::Center,
			..default()
		},
		WhiteTimer,
	));
	commands.spawn((
		Text2dBundle {
			text: Text {
				sections: vec![TextSection::new(
					String::from("00:00"),
					TextStyle {
						color: Color::WHITE,
						..text_style
					},
				)],
				..default()
			},
			transform: Transform::from_translation(Vec3::new(
				50. - (WINDOW_SIZE / 2.),
				30. + (WINDOW_SIZE / 2.),
				2.,
			)),
			text_anchor: Anchor::Center,
			..default()
		},
		BlackTimer,
	));
}

fn format_elapsed_time(seconds: u64) -> String {
	let duration = Duration::seconds(seconds.to_i64().unwrap());
	let minutes = duration.num_minutes();
	let remaining_seconds = duration.num_seconds() % 60;

	format!("{minutes:02}:{remaining_seconds:02}")
}

fn update_white_timer_system(
	timers: Res<GameTimers>,
	mut white_timer: Query<&mut Text, With<WhiteTimer>>,
) {
	let mut text = white_timer.get_single_mut().unwrap();
	let seconds = timers.white.duration().as_secs() - timers.white.elapsed().as_secs();

	text.sections[0].value = format_elapsed_time(seconds);
}

fn update_black_timer_system(
	timers: Res<GameTimers>,
	mut black_timer: Query<&mut Text, With<BlackTimer>>,
) {
	let mut text = black_timer.get_single_mut().unwrap();
	let seconds = timers.black.duration().as_secs() - timers.black.elapsed().as_secs();

	text.sections[0].value = format_elapsed_time(seconds);
}

fn countdown(
	time: Res<Time>,
	mut countdown: ResMut<GameTimers>,
	mut ev_exit: EventWriter<AppExit>,
) {
	if countdown.white.finished() {
		println!("Black WINS!!!");
		ev_exit.send(AppExit);
	}
	if countdown.black.finished() {
		println!("White WINS!!!");
		ev_exit.send(AppExit);
	}

	countdown.white.tick(time.delta());
	countdown.black.tick(time.delta());
}
