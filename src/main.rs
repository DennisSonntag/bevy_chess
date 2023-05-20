#![allow(dead_code, unused)]

use std::collections::HashMap;

use bevy::{
	prelude::*,
	window::{ExitCondition, PresentMode},
};

const WINDOW_SIZE: f32 = 600.;

const SQUARE_SIZE: f32 = WINDOW_SIZE / 8.;

#[derive(Resource, Debug)]
struct BoardResource {
	board: [Piece; 64],
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Pieces {
	None,
	King,
	Queen,
	Bishop,
	Knight,
	Rook,
	Pawn,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PieceColor {
	White,
	Black,
	None,
}

#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct Piece {
	row: Option<u8>,
	col: Option<u8>,
	piece: Pieces,
	color: PieceColor,
}

impl Default for Piece {
	fn default() -> Self {
		Self {
			row: None,
			col: None,
			color: PieceColor::None,
			piece: Pieces::None,
		}
	}
}

fn load_position_from_fen(fen: String) -> [Piece; 64] {
	let mut board: [Piece; 64] = [Piece {
		piece: Pieces::None,
		color: PieceColor::None,
		row: None,
		col: None,
	}; 64];

	let mut piece_type_from_symbol: HashMap<char, Pieces> = HashMap::new();
	piece_type_from_symbol.insert('k', Pieces::King);
	piece_type_from_symbol.insert('p', Pieces::Pawn);
	piece_type_from_symbol.insert('n', Pieces::Knight);
	piece_type_from_symbol.insert('b', Pieces::Bishop);
	piece_type_from_symbol.insert('r', Pieces::Rook);
	piece_type_from_symbol.insert('q', Pieces::Queen);

	let fen_data: Vec<&str> = fen.split(' ').collect();
	let fen_board: Vec<&str> = fen_data[0].split('/').collect();

	let mut col: i32 = 0;
	let mut row: i32 = 8;

	for row_data in fen_board {
		col = -1;
		row -= 1;
		for i in row_data.chars() {
			if i.is_ascii_digit() {
				col += i as i32;
				if col >= 7 {
					continue;
				}
			} else {
				col += 1;
			}
			let piece_color = if i.is_uppercase() {
				PieceColor::White
			} else if i.is_lowercase() {
				PieceColor::Black
			} else {
				PieceColor::None
			};

			let lower_char = &i.to_lowercase().to_string().chars().next().unwrap();
			let mut piece_type = Pieces::None;
			if piece_type_from_symbol.contains_key(lower_char) {
				piece_type = *piece_type_from_symbol.get(lower_char).unwrap();
			}
			board[(row * 8 + col) as usize] = Piece {
				piece: piece_type,
				color: piece_color,
				row: Some(row as u8),
				col: Some(col as u8),
			}
		}
	}

	board
}

impl FromWorld for BoardResource {
	fn from_world(_: &mut World) -> Self {
		let board = load_position_from_fen(String::from(
			"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
		));

		BoardResource { board }
	}
}

#[derive(Resource, Debug)]
struct SelectedPiece(Option<Piece>);

impl FromWorld for SelectedPiece {
	fn from_world(_: &mut World) -> Self {
		Self(None)
	}
}

fn main() {
	App::new()
		.insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
		.add_plugins(DefaultPlugins.set(WindowPlugin {
			primary_window: Some(Window {
				title: "chess".into(),
				resolution: (WINDOW_SIZE, WINDOW_SIZE).into(),
				present_mode: PresentMode::AutoVsync,
				// Tells wasm to resize the window according to the available canvas
				fit_canvas_to_parent: true,
				// Tells wasm not to override default event handling, like F5, Ctrl+R etc.
				prevent_default_event_handling: false,
				..Default::default()
			}),
			..Default::default()
		}))
		.insert_resource(Msaa::Sample8)
		.init_resource::<BoardResource>()
		.init_resource::<SelectedPiece>()
		.add_startup_system(setup_camera)
		.add_startup_system(spawn_board)
		.add_startup_system(first_draw_board)
		.add_system(move_piece_system)
		.run();
}

fn setup_camera(mut commands: Commands) {
	commands.spawn(Camera2dBundle::default());
}

fn spawn_board(mut commands: Commands) {
	for row in 1..=8 {
		for col in 1..=8 {
			let color = if (row + col) % 2 == 0 {
				Color::rgb_u8(162, 110, 91)
			} else {
				Color::rgb_u8(236, 210, 185)
			};

			commands.spawn((SpriteBundle {
				transform: Transform {
					translation: Vec3::new(
						-(WINDOW_SIZE / 2.) + ((col as f32 - 0.5) * SQUARE_SIZE),
						-(WINDOW_SIZE / 2.) + ((row as f32 - 0.5) * SQUARE_SIZE),
						0.0,
					),
					scale: Vec3::new(SQUARE_SIZE, SQUARE_SIZE, 0.0),
					..default()
				},
				sprite: Sprite { color, ..default() },
				..default()
			},));
		}
	}
}

fn first_draw_board(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
	board: ResMut<BoardResource>,
) {
	let texture_handle = asset_server.load("pieces.png");

	for (i, el) in board.board.iter().enumerate() {
		if el.piece as i32 != 0 {
			let row = i / 8;
			let col = i % 8;

			let texture_atlas = TextureAtlas::from_grid(
				texture_handle.clone(),
				Vec2::new(333.3, 333.3),
				2,
				1,
				None,
				Some(Vec2::new(
					(el.piece as i32 - 1) as f32 * 333.3,
					(el.color as i32) as f32 * 333.3,
				)),
			);
			let texture_atlas_handle = texture_atlases.add(texture_atlas);

			commands
				.spawn(SpriteSheetBundle {
					texture_atlas: texture_atlas_handle,
					transform: Transform {
						translation: Vec3::new(
							col as f32 * SQUARE_SIZE - (WINDOW_SIZE / 2.) + (SQUARE_SIZE / 2.),
							row as f32 * SQUARE_SIZE - (WINDOW_SIZE / 2.) + (SQUARE_SIZE / 2.),
							0.0,
						),
						scale: Vec3::splat(WINDOW_SIZE / 2500.),
						..default()
					},

					..default()
				})
				.insert(Piece {
					piece: el.piece,
					color: el.color,
					row: Some(row as u8),
					col: Some(col as u8),
				});
		}
	}
}

fn move_piece_system(
	mouse_button_input: Res<Input<MouseButton>>,
	windows: Query<&Window>,
	mut board: ResMut<BoardResource>,
	mut selected_piece: ResMut<SelectedPiece>,
	mut pieces: Query<(&Piece, &mut Transform)>,
) {
	let window = windows.get_single().unwrap();

	if let Some(position) = window.cursor_position() {
		if mouse_button_input.just_pressed(MouseButton::Left) {
			let col = ((position[0] / 75.).floor()) as u8;
			let row = ((position[1] / 75.).floor()) as u8;

			let index = (row * 8 + col) as usize;

			let clicked_piece = board.board[index];

			// board.board.iter().for_each(|x| println!("{:?}", x));
			if selected_piece.0.is_none() && clicked_piece.piece != Pieces::None {
				selected_piece.0 = Some(clicked_piece);
			};

			if selected_piece.0 != Some(clicked_piece) && selected_piece.0.is_some() {
				for (piece, mut transform) in pieces.iter_mut() {
					if Some(*piece) == selected_piece.0 {
						transform.translation.x =
							col as f32 * SQUARE_SIZE - (WINDOW_SIZE / 2.) + (SQUARE_SIZE / 2.);
						transform.translation.y =
							row as f32 * SQUARE_SIZE - (WINDOW_SIZE / 2.) + (SQUARE_SIZE / 2.);
					}
				}

				if let Some(piece) = selected_piece.0 {
					let old_index = (piece.row.unwrap_or(0) * 8 + piece.col.unwrap_or(0)) as usize;
					board.board[old_index] = Piece {
						..Default::default()
					};
					board.board[index] = piece;
					board.board[index].row = Some(row);
					board.board[index].col = Some(col);
				}

				selected_piece.0 = None;
			}
		}
	}
}

fn print_board(board: [Piece; 64]) {
	for row in (0..8).rev() {
		print!("|");
		for col in 0..8 {
			let index = (row * 8 + col) as usize;
			match board[index].piece {
				Pieces::None => print!(" "),
				piece => print!(
					"{}",
					format!("{:?}", piece)
						.chars()
						.next()
						.unwrap()
						.to_lowercase()
				),
			}
			print!("|");
		}
		println!();
	}
}
