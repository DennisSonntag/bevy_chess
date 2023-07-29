#![allow(dead_code, unused, clippy::cast_sign_loss)]

use anyhow::Result;
use bevy::prelude::*;
use std::{collections::HashMap, string::ToString};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::{BOARD_SIZE, SQUARE_SIZE, WINDOW_SIZE};

pub struct Coord;

impl Coord {
	pub fn to_win<T: Into<f32>>(pos: T, min: f32) -> f32 {
		let pos: f32 = pos.into();
		(pos + min).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.))
	}
	pub fn to_win_piece<T: Into<f32>>(pos: T) -> f32 {
		let pos: f32 = pos.into();
		(pos).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)) + (SQUARE_SIZE / 2.)
	}
}

#[derive(Resource)]
pub struct GameTimers {
	pub white: Timer,
	pub black: Timer,
}

impl GameTimers {
	pub fn new() -> Self {
		Self {
			white: Timer::from_seconds(5. * 60., TimerMode::Once),
			black: Timer::from_seconds(5. * 60., TimerMode::Once),
		}
	}
}

impl Default for GameTimers {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Resource, Debug)]
pub struct BoardResource(pub [Option<Piece>; 64]);

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub struct WhiteTimer;

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub struct BlackTimer;

#[derive(Event)]
pub struct TakeEvent;

#[derive(Default, Event)]
pub struct MoveEvent(pub Option<Position>);

#[derive(Default, Event)]
pub struct LegalMoveEvent(pub Option<Vec<i8>>);

#[derive(Default, Event)]
pub struct HoverEvent(pub Option<Position>);

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub struct LegalMoveMarker;

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub struct MovedSquare;

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub struct HoverSquare;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Display)]
pub enum Pieces {
	King,
	Queen,
	Bishop,
	Knight,
	Rook,
	Pawn,
}

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub struct HighlightSquare;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum PieceColor {
	#[default]
	White,
	Black,
}

impl PieceColor {
	pub const fn not(self) -> Self {
		match self {
			Self::Black => Self::White,
			Self::White => Self::Black,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
	pub row: i8,
	pub col: i8,
}

impl Position {
	pub const fn new(row: i8, col: i8) -> Self {
		Self { row, col }
	}
}

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub struct Piece {
	pub pos: Position,
	pub amount_moved: u32,
	pub piece_type: Pieces,
	pub color: PieceColor,
}

impl FromWorld for BoardResource {
	fn from_world(_: &mut World) -> Self {
		load_position_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
			.map_or(Self([None; 64]), Self)
	}
}

fn load_position_from_fen(fen: &str) -> Option<[Option<Piece>; 64]> {
	let mut board = [None; 64];

	let piece_type_from_symbol: HashMap<char, Pieces> = Pieces::iter()
		.map(|x| {
			(
				match x {
					Pieces::Knight => 'n',
					_ => x
						.to_string()
						.chars()
						.next()
						.unwrap()
						.to_lowercase()
						.next()
						.unwrap(),
				},
				x,
			)
		})
		.collect();

	let fen_data: Vec<&str> = fen.split(' ').collect();
	let fen_board: Vec<&str> = fen_data[0].split('/').collect();

	let mut col = 0;
	let mut row = BOARD_SIZE;

	for row_data in fen_board {
		col = -1;
		row -= 1;
		for char in row_data.chars() {
			if char.is_ascii_digit() {
				col += char as i8;
				if col >= 7 {
					continue;
				}
			} else {
				col += 1;
			}
			let piece_color = if char.is_uppercase() {
				PieceColor::White
			} else {
				PieceColor::Black
			};

			let lower_char = &char.to_lowercase().to_string().chars().next()?;
			if piece_type_from_symbol.contains_key(lower_char) {
				let piece_type = *piece_type_from_symbol.get(lower_char)?;

				board[(row * BOARD_SIZE + col) as usize] = Some(Piece {
					piece_type,
					color: piece_color,
					amount_moved: 0,
					pos: Position::new(row, col),
				});
			};
		}
	}

	Some(board)
}

#[derive(Resource, Debug)]
pub struct SelectedPiece(pub Option<Piece>);

impl FromWorld for SelectedPiece {
	fn from_world(_: &mut World) -> Self {
		Self(None)
	}
}

#[derive(Resource, Debug, Clone)]
pub struct MoveData {
	pub num_squares_to_edge: HashMap<usize, Vec<i8>>,
	pub direction_offsets: [i8; 8],
}

impl FromWorld for MoveData {
	fn from_world(_: &mut World) -> Self {
		let mut num_squares_to_edge = HashMap::new();
		for row in 0..BOARD_SIZE {
			for col in 0..BOARD_SIZE {
				let num_north = 7 - col;
				let num_south = col;
				let num_west = row;
				let num_east = 7 - row;

				let index = (col * BOARD_SIZE + row) as usize;

				num_squares_to_edge.insert(
					index,
					vec![
						num_north,
						num_south,
						num_west,
						num_east,
						num_north.min(num_west),
						num_south.min(num_east),
						num_north.min(num_east),
						num_south.min(num_west),
					],
				);
			}
		}

		Self {
			num_squares_to_edge,
			direction_offsets: [8, -8, -1, 1, 7, -7, 9, -9],
		}
	}
}
