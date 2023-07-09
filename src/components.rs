#![allow(dead_code, unused, clippy::cast_sign_loss)]

use bevy::prelude::*;
use std::collections::HashMap;

use crate::BOARD_SIZE;

#[derive(Resource, Debug)]
pub struct BoardResource {
	pub board: [Piece; 64],
}

pub struct TakeEvent;

pub struct MoveEvent {
	pub pos: Option<Position>,
}

pub struct LegalMoveEvent(pub Option<Vec<i8>>);

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub struct LegalMoveMarker;

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub struct MovedSquare;

pub struct HoverEvent {
	pub pos: Option<Position>,
}

#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub struct HoverSquare;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Default, Debug, Clone, Copy, Component, PartialEq, Eq)]
pub struct Piece {
	pub pos: Option<Position>,
	pub amount_moved: u32,
	pub piece: Option<Pieces>,
	pub color: Option<PieceColor>,
}

impl FromWorld for BoardResource {
	fn from_world(_: &mut World) -> Self {
		let board =
			load_position_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

		Self { board }
	}
}

fn load_position_from_fen(fen: &str) -> [Piece; 64] {
	let mut board: [Piece; 64] = [Piece::default(); 64];

	let mut piece_type_from_symbol: HashMap<char, Pieces> = HashMap::new();
	piece_type_from_symbol.insert('k', Pieces::King);
	piece_type_from_symbol.insert('p', Pieces::Pawn);
	piece_type_from_symbol.insert('n', Pieces::Knight);
	piece_type_from_symbol.insert('b', Pieces::Bishop);
	piece_type_from_symbol.insert('r', Pieces::Rook);
	piece_type_from_symbol.insert('q', Pieces::Queen);

	let fen_data: Vec<&str> = fen.split(' ').collect();
	let fen_board: Vec<&str> = fen_data[0].split('/').collect();

	let mut col: i8 = 0;
	let mut row: i8 = BOARD_SIZE;

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
				Some(PieceColor::White)
			} else if char.is_lowercase() {
				Some(PieceColor::Black)
			} else {
				None
			};

			let lower_char = &char
				.to_lowercase()
				.to_string()
				.chars()
				.next()
				.expect("could not get first lowercase character");

			let piece_type = if piece_type_from_symbol.contains_key(lower_char) {
				Some(
					*piece_type_from_symbol
						.get(lower_char)
						.expect("value with key lower_char does not exist"),
				)
			} else {
				None
			};

			board[(row * BOARD_SIZE + col) as usize] = Piece {
				piece: piece_type,
				color: piece_color,
				amount_moved: 0,
				pos: Some(Position::new(row, col)),
			}
		}
	}

	board
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
