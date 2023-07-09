#![allow(dead_code, unused, clippy::cast_sign_loss, clippy::cast_precision_loss)]

use bevy::{
	prelude::*,
	sprite::Anchor,
	window::{ExitCondition, PresentMode},
};
use bevy_prototype_lyon::prelude::*;

use components::{
	BoardResource, HighlightSquare, HoverEvent, HoverSquare, LegalMoveEvent, MoveData, MoveEvent,
	MovedSquare, Piece, Position, SelectedPiece, TakeEvent,Turn
};
use piece::PiecePlugin;
use sounds::SoundPlugin;

mod components;
mod piece;
mod sounds;

const WINDOW_SIZE: f32 = 600.;

const SQUARE_SIZE: f32 = WINDOW_SIZE / 8.;

const BOARD_SIZE: i8 = 8;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins.set(WindowPlugin {
			primary_window: Some(Window {
				title: "chess".into(),
				resolution: (WINDOW_SIZE, WINDOW_SIZE).into(),
				present_mode: PresentMode::AutoVsync,
				fit_canvas_to_parent: true,
				prevent_default_event_handling: false,
				..Default::default()
			}),
			exit_condition: ExitCondition::OnPrimaryClosed,
			..Default::default()
		}))
		.add_plugin(ShapePlugin)
		.insert_resource(Msaa::Sample8)
		.init_resource::<BoardResource>()
		.init_resource::<SelectedPiece>()
		.init_resource::<MoveData>()
		.add_state::<Turn>()
		.add_event::<MoveEvent>()
		.add_event::<TakeEvent>()
		.add_event::<HoverEvent>()
		.add_event::<LegalMoveEvent>()
		.add_startup_system(setup_camera)
		.add_startup_system(spawn_board_system)
		.add_startup_system(spawn_piece_sprites_system)
		.add_plugin(SoundPlugin)
		.add_plugin(PiecePlugin)
		.run();
}

fn setup_camera(mut commands: Commands) {
	commands.spawn(Camera2dBundle::default());
}

#[allow(clippy::needless_pass_by_value)]
fn spawn_board_system(mut commands: Commands, asset_server: Res<AssetServer>) {
	let font = asset_server.load("fonts/Roboto-Bold.ttf");
	let text_style = TextStyle {
		font,
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

			let color1 = if (row + col) % 2 == 0 {
				Color::rgb_u8(236, 210, 185)
			} else {
				Color::rgb_u8(162, 110, 91)
			};

			commands.spawn((SpriteBundle {
				transform: Transform {
					translation: Vec3::new(
						(f32::from(col) - 0.5).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)),
						(f32::from(row) - 0.5).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)),
						0.0,
					),
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
								color: color1,
								..text_style.clone()
							},
						)],
						..Default::default()
					},
					transform: Transform::from_translation(Vec3::new(
						SQUARE_SIZE.mul_add(f32::from(col) - 1., -(WINDOW_SIZE / 2.) + 67.),
						SQUARE_SIZE.mul_add(f32::from(row) - 1., -(WINDOW_SIZE / 2.) + 67.),
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
								color: color1,
								..text_style.clone()
							},
						)],
						..Default::default()
					},
					transform: Transform::from_translation(Vec3::new(
						SQUARE_SIZE.mul_add(f32::from(col) - 1., -(WINDOW_SIZE / 2.) + 10.),
						SQUARE_SIZE.mul_add(f32::from(row) - 1., -(WINDOW_SIZE / 2.) + 10.),
						1.,
					)),
					text_anchor: Anchor::Center,
					..default()
				},));
			}
		}
	}
}

#[allow(clippy::needless_pass_by_value)]
fn spawn_piece_sprites_system(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
	board: ResMut<BoardResource>,
) {
	let texture_handle = asset_server.load("pieces.png");

	for (index, piece) in board.board.iter().enumerate() {
		if piece.piece as i32 != 0 {
			let row = i8::try_from(index).expect("could not cast index to i8") / BOARD_SIZE;
			let col = i8::try_from(index).expect("could not cast index to i8") % BOARD_SIZE;

			let texture_atlas = TextureAtlas::from_grid(
				texture_handle.clone(),
				Vec2::new(333.3, 333.3),
				2,
				1,
				None,
				Some(Vec2::new(
					(piece.piece as i32 - 1) as f32 * 333.3,
					(piece.color as i32) as f32 * 333.3,
				)),
			);
			let texture_atlas_handle = texture_atlases.add(texture_atlas);

			commands
				.spawn(SpriteSheetBundle {
					texture_atlas: texture_atlas_handle,
					transform: Transform {
						translation: Vec3::new(
							f32::from(col).mul_add(SQUARE_SIZE, -WINDOW_SIZE / 2.)
								+ (SQUARE_SIZE / 2.),
							f32::from(row).mul_add(SQUARE_SIZE, -WINDOW_SIZE / 2.)
								+ (SQUARE_SIZE / 2.),
							2.0,
						),
						scale: Vec3::splat(WINDOW_SIZE / 2500.),
						..default()
					},

					..default()
				})
				.insert(Piece {
					piece: piece.piece,
					color: piece.color,
					amount_moved: piece.amount_moved,
					pos: Some(Position::new(row, col)),
				});
		}
	}

	commands
		.spawn((SpriteBundle {
			transform: Transform {
				translation: Vec3::new(
					(0.5f32).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)),
					(0.5f32).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)),
					1.0,
				),
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
				translation: Vec3::new(
					(0.5f32).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)),
					(0.5f32).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)),
					1.0,
				),
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
					translation: Vec3::new(
						(-0.5f32).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)),
						(-0.5f32).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)),
						5.0,
					),
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

// fn print_board(board: [Piece; 64]) {
// 	for row in (0..BOARD_SIZE).rev() {
// 		print!("|");
// 		for col in 0..BOARD_SIZE {
// 			let index = (row * BOARD_SIZE + col) as usize;
// 			match board[index].piece {
// 				Pieces::None => print!(" "),
// 				piece => print!(
// 					"{}",
// 					format!("{:?}", piece)
// 						.chars()
// 						.next()
// 						.unwrap()
// 						.to_lowercase()
// 				),
// 			}
// 			print!("|");
// 		}
// 		println!();
// 	}
// }
