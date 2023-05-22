#![allow(dead_code)]

use bevy::prelude::*;

use crate::{
	components::{
		BoardResource, HighlightSquare, MoveEvent, MovedSquare, Piece, PieceColor, Pieces,
		SelectedPiece, TakeEvent, Turn,
	},
	BOARD_SIZE, SQUARE_SIZE, WINDOW_SIZE,
};

pub struct PiecePlugin;

impl Plugin for PiecePlugin {
	fn build(&self, app: &mut App) {
		app.add_system(drag_piece_system)
			// .add_system(move_piece_system)
			.add_system(highlight_moved_system)
			.add_system(highlight_selected_system);
	}
}

pub fn drag_piece_system(
	mouse_button_input: Res<Input<MouseButton>>,
	windows: Query<&Window>,
	mut board: ResMut<BoardResource>,
	mut selected_piece: ResMut<SelectedPiece>,
	mut pieces: Query<(&mut Piece, &mut Transform, Entity)>,
	mut next_state: ResMut<NextState<Turn>>,
	current_state: Res<State<Turn>>,
	mut commands: Commands,
	mut ev_move: EventWriter<MoveEvent>,
	mut ev_take: EventWriter<TakeEvent>,
) {
	let window = windows.get_single().unwrap();

	if let Some(position) = window.cursor_position() {
		let col = ((position[0] / 75.).floor()) as u8;
		let row = ((position[1] / 75.).floor()) as u8;

		let index = (row * BOARD_SIZE as u8 + col) as usize;

		let clicked_piece = board.board[index];
		if mouse_button_input.just_pressed(MouseButton::Left) {}
		if mouse_button_input.pressed(MouseButton::Left) {}
		if mouse_button_input.just_released(MouseButton::Left) {}
	}
}

pub fn move_piece_system(
	mouse_button_input: Res<Input<MouseButton>>,
	windows: Query<&Window>,
	mut board: ResMut<BoardResource>,
	mut selected_piece: ResMut<SelectedPiece>,
	mut pieces: Query<(&mut Piece, &mut Transform, Entity)>,
	mut next_state: ResMut<NextState<Turn>>,
	current_state: Res<State<Turn>>,
	mut commands: Commands,
	mut ev_move: EventWriter<MoveEvent>,
	mut ev_take: EventWriter<TakeEvent>,
) {
	let window = windows.get_single().unwrap();

	if let Some(position) = window.cursor_position() {
		if mouse_button_input.just_pressed(MouseButton::Left) {
			let col = ((position[0] / 75.).floor()) as u8;
			let row = ((position[1] / 75.).floor()) as u8;

			let index = (row * BOARD_SIZE as u8 + col) as usize;

			let clicked_piece = board.board[index];

			if Some(clicked_piece) == selected_piece.0 && selected_piece.0.is_some() {
				selected_piece.0 = None;
				ev_move.send(MoveEvent {
					row: None,
					col: None,
				});
			} else if clicked_piece.piece != Pieces::None && clicked_piece.color == current_state.0
			{
				selected_piece.0 = Some(clicked_piece);
				ev_move.send(MoveEvent {
					row: None,
					col: None,
				});
			};

			if selected_piece.0 != Some(clicked_piece)
				&& selected_piece.0.is_some()
				&& clicked_piece.color != current_state.0
			{
				for (mut piece, mut transform, entity) in pieces.iter_mut() {
					if Some(*piece) == selected_piece.0 {
						transform.translation.x =
							col as f32 * SQUARE_SIZE - (WINDOW_SIZE / 2.) + (SQUARE_SIZE / 2.);
						transform.translation.y =
							row as f32 * SQUARE_SIZE - (WINDOW_SIZE / 2.) + (SQUARE_SIZE / 2.);
						piece.row = Some(row);
						piece.col = Some(col);

						let old_index = (piece.row.unwrap_or(0) * BOARD_SIZE as u8
							+ piece.col.unwrap_or(0)) as usize;
						board.board[old_index] = Piece {
							..Default::default()
						};

						board.board[index] = *piece;
						board.board[index].row = Some(row);
						board.board[index].col = Some(col);

						ev_move.send(MoveEvent {
							row: Some(row),
							col: Some(col),
						});
					}
					if *piece.as_ref() == clicked_piece
						&& clicked_piece.color != PieceColor::None
						&& clicked_piece.color != current_state.0
					{
						commands.entity(entity).despawn_recursive();
						ev_take.send(TakeEvent);
					}
				}

				selected_piece.0 = None;
				next_state.set(match current_state.0 {
					Turn::White => Turn::Black,
					Turn::Black => Turn::White,
				});
			}
		}
	}
}

pub fn highlight_selected_system(
	selected: Res<SelectedPiece>,
	mut highlight_square: Query<(&HighlightSquare, &mut Transform)>,
) {
	let mut highlight_square = highlight_square.get_single_mut().unwrap();
	if let Some(selected) = selected.0 {
		highlight_square.1.translation.x = selected.col.unwrap_or(0) as f32 * SQUARE_SIZE
			- (WINDOW_SIZE / 2.)
			+ (SQUARE_SIZE / 2.);
		highlight_square.1.translation.y = selected.row.unwrap_or(0) as f32 * SQUARE_SIZE
			- (WINDOW_SIZE / 2.)
			+ (SQUARE_SIZE / 2.);
	} else {
		highlight_square.1.translation.x =
			-1. * SQUARE_SIZE - (WINDOW_SIZE / 2.) + (SQUARE_SIZE / 2.);
		highlight_square.1.translation.y =
			-1. * SQUARE_SIZE - (WINDOW_SIZE / 2.) + (SQUARE_SIZE / 2.);
	}
}

pub fn highlight_moved_system(
	mut move_square: Query<(&MovedSquare, &mut Transform)>,
	mut ev_move: EventReader<MoveEvent>,
) {
	let mut moved_square = move_square.get_single_mut().unwrap();
	for i in ev_move.iter() {
		if let (Some(row), Some(col)) = (i.row, i.col) {
			moved_square.1.translation.x =
				col as f32 * SQUARE_SIZE - (WINDOW_SIZE / 2.) + (SQUARE_SIZE / 2.);
			moved_square.1.translation.y =
				row as f32 * SQUARE_SIZE - (WINDOW_SIZE / 2.) + (SQUARE_SIZE / 2.);
		} else {
			moved_square.1.translation.x =
				-1. * SQUARE_SIZE - (WINDOW_SIZE / 2.) + (SQUARE_SIZE / 2.);
			moved_square.1.translation.y =
				-1. * SQUARE_SIZE - (WINDOW_SIZE / 2.) + (SQUARE_SIZE / 2.);
		}
	}
}
