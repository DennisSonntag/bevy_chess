#![allow(dead_code, unused)]

use bevy::prelude::*;
use std::collections::HashMap;

use crate::BOARD_SIZE;

#[derive(Resource, Debug)]
pub struct BoardResource {
	pub board: [Piece; 64],
}

pub struct TakeEvent;

pub struct MoveEvent {
	pub row: Option<u8>,
	pub col: Option<u8>,
}

#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct MovedSquare;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pieces {
	None,
	King,
	Queen,
	Bishop,
	Knight,
	Rook,
	Pawn,
}

#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct HighlightSquare;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieceColor {
	White,
	Black,
	None,
}

#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct Piece {
	pub row: Option<u8>,
	pub col: Option<u8>,
	pub piece: Pieces,
	pub color: PieceColor,
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub enum Turn {
	#[default]
	White,
	Black,
}

impl PartialEq<Turn> for PieceColor {
	fn eq(&self, other: &Turn) -> bool {
		matches!(
			(self, other),
			(PieceColor::White, Turn::White) | (PieceColor::Black, Turn::Black)
		)
	}
}

impl FromWorld for BoardResource {
	fn from_world(_: &mut World) -> Self {
		let board = load_position_from_fen(String::from(
			"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
		));

		BoardResource { board }
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
	let mut row: i32 = BOARD_SIZE;

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
			board[(row * BOARD_SIZE + col) as usize] = Piece {
				piece: piece_type,
				color: piece_color,
				row: Some(row as u8),
				col: Some(col as u8),
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
