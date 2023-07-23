#![allow(
	dead_code,
	unused,
	clippy::suspicious_operation_groupings,
	clippy::module_name_repetitions,
	clippy::cast_sign_loss,
	clippy::too_many_arguments,
	clippy::too_many_lines,
	clippy::needless_pass_by_value
)]

use std::collections::HashMap;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
	components::{
		BoardResource, Coord, GameTimers, HighlightSquare, HoverEvent, HoverSquare, LegalMoveEvent,
		LegalMoveMarker, MoveData, MoveEvent, MovedSquare, Piece, PieceColor, Pieces, Position,
		SelectedPiece, TakeEvent,
	},
	BOARD_SIZE, SQUARE_SIZE, WINDOW_SIZE,
};

pub struct PiecePlugin;

impl Plugin for PiecePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			(
				move_piece_system,
				highlight_moved_system,
				highlight_selected_system,
				highlight_hover_system,
				highlight_legal_moves_system,
			),
		);
	}
}

struct LegalMoveGen<'a> {
	piece: Piece,
	start_square: i8,
	squares_to_edge: &'a HashMap<usize, Vec<i8>>,
	direction_offsets: [i8; 8],
	board: &'a [Option<Piece>; 64],
	turn_color: PieceColor,
	moves: Vec<i8>,
}

impl LegalMoveGen<'_> {
	fn sliding_moves(&mut self) {
		let start_dir_index = if self.piece.piece_type == Pieces::Bishop {
			4
		} else {
			0
		};

		let end_dir_index = if self.piece.piece_type == Pieces::Rook {
			4
		} else {
			8
		};

		for direction_index in start_dir_index..end_dir_index {
			let direction_index = direction_index as usize;
			for n in 0..self
				.squares_to_edge
				.get(&(self.start_square as usize))
				.expect("squares to edge for start square not found")[direction_index]
			{
				let target_square =
					(self.start_square + self.direction_offsets[direction_index] * (n + 1));
				let piece_on_target_square = self.board[target_square as usize];

				// Blocked by friendly piece, so can't move any further in this direction
				if piece_on_target_square.is_some_and(|x| x.color == self.turn_color) {
					break;
				}

				self.moves.push(target_square);

				// Can't move any further in this directoin after capturing opponent's piece
				if piece_on_target_square.is_some_and(|x| x.color != self.turn_color) {
					break;
				}
			}
		}
	}

	fn pawn_moves(&mut self) {
		let valid_offsets = if self.piece.color == PieceColor::White {
			([8, 16], [9, 7])
		} else {
			([-8, -16], [-9, -7])
		};
		for offset in valid_offsets.0 {
			let target_square = (self.start_square + offset);
			let piece_on_target_square = self.board[target_square as usize];

			if offset.abs() == 8
				&& piece_on_target_square.is_some_and(|x| x.color != self.turn_color)
			{
				break;
			}

			// Blocked by friendly piece, so can't move any further in this direction
			if piece_on_target_square.is_some_and(|x| x.color == self.turn_color) {
				break;
			}
			if offset.abs() == 16 && self.piece.amount_moved != 0 {
				break;
			}

			self.moves.push(target_square);

			// Can't move any further in this directoin after capturing opponent's piece
			if piece_on_target_square.is_some_and(|x| x.color != self.turn_color) {
				break;
			}
		}
		for offset in valid_offsets.1 {
			let target_square = (self.start_square + offset);
			let piece_on_target_square = self.board[target_square as usize];

			// Blocked by friendly piece, so can't move any further in this direction
			if piece_on_target_square.is_some_and(|x| x.color == self.turn_color) {
				break;
			}

			if piece_on_target_square.is_none()
				|| piece_on_target_square.is_some_and(|x| x.color != self.piece.color.not())
			{
				continue;
			}

			self.moves.push(target_square);

			// Can't move any further in this directoin after capturing opponent's piece
			if piece_on_target_square.is_some_and(|x| x.color != self.turn_color) {
				break;
			}
		}
	}

	fn knight_moves(&mut self) {
		let directions = [
			(1, 2),
			(2, 1),
			(-1, 2),
			(-2, 1),
			(1, -2),
			(2, -1),
			(-1, -2),
			(-2, -1),
		];

		let calculate_index = |row: i8, col: i8| (row * BOARD_SIZE + col);
		for &(dir_x, dir_y) in &directions {
			let new_row = self.piece.pos.row + dir_x;
			let new_col = self.piece.pos.col + dir_y;

			if (0..8).contains(&new_row) && (0..8).contains(&new_col) {
				self.moves.push(calculate_index(new_row, new_col));
			}
		}
	}
	fn king_moves(&mut self) {
		for (index, offset) in self.direction_offsets.iter().enumerate() {
			if self
				.squares_to_edge
				.get(&(self.start_square as usize))
				.expect("could not find squares_to_edge from start_square")[index]
				> 0
			{
				let target_square = (self.start_square + offset);
				let piece_on_target_square = self.board[target_square as usize];

				// Blocked by friendly piece, so can't move any further in this direction
				if piece_on_target_square.is_some_and(|x| x.color == self.turn_color) {
					continue;
				}
				self.moves.push(target_square);
			}
		}
	}
}

fn get_legal_moves(
	squares_to_edge: &HashMap<usize, Vec<i8>>,
	direction_offsets: [i8; 8],
	board: &[Option<Piece>; 64],
	start_square: i8,
	turn_color: PieceColor,
) -> Vec<i8> {
	let mut moves = Vec::new();
	let piece = board[start_square as usize].unwrap();
	let mut legal_move_gen = LegalMoveGen {
		piece,
		start_square,
		squares_to_edge,
		direction_offsets,
		board,
		turn_color,
		moves,
	};
	match piece.piece_type {
		Pieces::Queen | Pieces::Rook | Pieces::Bishop => legal_move_gen.sliding_moves(),
		Pieces::Knight => legal_move_gen.knight_moves(),
		Pieces::Pawn => legal_move_gen.pawn_moves(),
		Pieces::King => legal_move_gen.king_moves(),
	}

	legal_move_gen.moves
}

fn move_piece_system(
	mouse_button_input: Res<Input<MouseButton>>,
	windows: Query<&Window>,
	mut board: ResMut<BoardResource>,
	mut selected_piece: ResMut<SelectedPiece>,
	mut pieces: Query<(&mut Piece, &mut Transform, Entity)>,
	mut next_state: ResMut<NextState<PieceColor>>,
	current_state: Res<State<PieceColor>>,
	mut commands: Commands,
	mut ev_move: EventWriter<MoveEvent>,
	mut ev_hover: EventWriter<HoverEvent>,
	mut ev_take: EventWriter<TakeEvent>,
	mut ev_legal: EventWriter<LegalMoveEvent>,
	move_info: Res<MoveData>,
	mut timers: ResMut<GameTimers>,
) {
	let window = windows.get_single().expect("could not get window");

	#[allow(clippy::cast_possible_truncation)]
	if let Some(position) = window.cursor_position() {
		let x = position[0];
		let y = position[1] - 50.;

		let col = ((x / 75.).floor()) as i8;
		let row = 7 - ((y / 75.).floor()) as i8;
		if (0. ..600.).contains(&y) && (0. ..600.).contains(&x) {
			let turn_color = *current_state.get();
			let index = (row * BOARD_SIZE + col) as usize;

			let clicked_piece = board.0[index];
			if mouse_button_input.just_pressed(MouseButton::Left) {
				if clicked_piece == selected_piece.0 && selected_piece.0.is_some() {
					// if piece is already selected deselect it
					selected_piece.0 = None;
					ev_legal.send(LegalMoveEvent::default());
					ev_move.send(MoveEvent::default());
				} else if clicked_piece.is_some_and(|x| x.color == turn_color) {
					//if piece isnt selected select it
					selected_piece.0 = clicked_piece;
					if let Some(selected) = selected_piece.0 {
						let selected_index = selected.pos.row * BOARD_SIZE + selected.pos.col;

						let legal_moves = get_legal_moves(
							&move_info.clone().num_squares_to_edge,
							move_info.direction_offsets,
							&board.0,
							selected_index,
							turn_color,
						);
						ev_legal.send(LegalMoveEvent(Some(legal_moves)));
					}
					ev_move.send(MoveEvent::default());
					ev_hover.send(HoverEvent::default());
				};
			}
			if mouse_button_input.pressed(MouseButton::Left) {
				if let Some(selected) = selected_piece.0 {
					let selected_index = selected.pos.row * BOARD_SIZE + selected.pos.col;

					let legal_moves = get_legal_moves(
						&move_info.clone().num_squares_to_edge,
						move_info.direction_offsets,
						&board.0,
						selected_index,
						turn_color,
					);

					ev_legal.send(LegalMoveEvent(Some(legal_moves.clone())));
					let clicked_index = row * BOARD_SIZE + col;

					if (clicked_piece.is_none()
						|| clicked_piece.is_some_and(|x| x.color != turn_color))
						&& legal_moves.contains(&(clicked_index))
					{
						ev_hover.send(HoverEvent(Some(Position::new(row, col))));
					} else if clicked_piece.is_some_and(|x| x.color == turn_color) {
						ev_hover.send(HoverEvent::default());
					}
					for (piece, mut transform, _) in pieces.iter_mut() {
						if Some(*piece) == selected_piece.0 {
							transform.translation.x = position.x - (WINDOW_SIZE / 2.);
							transform.translation.y = -position.y + (WINDOW_SIZE / 2.) + 50.;
							transform.translation.z = 30.;
						}
					}
				}
			}
			if mouse_button_input.just_released(MouseButton::Left) {
				ev_hover.send(HoverEvent::default());

				if let Some(selected) = selected_piece.0 {
					let selected_index = selected.pos.row * BOARD_SIZE + selected.pos.col;

					let legal_moves = get_legal_moves(
						&move_info.clone().num_squares_to_edge,
						move_info.direction_offsets,
						&board.0,
						selected_index,
						turn_color,
					);

					let clicked_index = row * BOARD_SIZE + col;
					if Some(selected) != clicked_piece
						&& (clicked_piece.is_none()
							|| clicked_piece.is_some_and(|x| x.color != turn_color))
						&& legal_moves.contains(&{ clicked_index })
					{
						for (mut piece, mut transform, entity) in pieces.iter_mut() {
							if piece.as_ref() == &selected {
								transform.translation.x = Coord::to_win_piece(col);
								transform.translation.y = Coord::to_win_piece(row);

								transform.translation.z = 2.;
								piece.amount_moved += 1;
								let new_position = Position::new(row, col);
								piece.pos = new_position;

								let old_index =
									(selected.pos.row * BOARD_SIZE + selected.pos.col) as usize;

								board.0[old_index] = None;

								board.0[index] = Some(*piece);
								board.0[index].map(|mut x| new_position);

								ev_move.send(MoveEvent(Some(new_position)));
							}
							if clicked_piece.is_some_and(|x| x == *piece.as_ref())
								&& clicked_piece.is_some_and(|x| x.color != turn_color)
							{
								commands.entity(entity).despawn_recursive();
								ev_take.send(TakeEvent);
							}
						}
						selected_piece.0 = None;
						ev_legal.send(LegalMoveEvent::default());
						next_state.set(turn_color.not());
						match turn_color {
							PieceColor::White => {
								timers.black.unpause();
								timers.white.pause();
							}
							PieceColor::Black => {
								timers.white.unpause();
								timers.black.pause();
							}
						}
					} else if Some(selected) == clicked_piece
						|| clicked_piece.is_some_and(|x| x.color == turn_color)
						|| !legal_moves.contains(&{ clicked_index })
					{
						for (piece, mut transform, _) in pieces.iter_mut() {
							if piece.as_ref() == &selected {
								transform.translation.x = Coord::to_win_piece(piece.pos.col);
								transform.translation.y = Coord::to_win_piece(piece.pos.row);
								transform.translation.z = 2.;
							}
						}
					}
				}
			}
		}
	}
}

fn highlight_selected_system(
	selected: Res<SelectedPiece>,
	mut highlight_square: Query<(&HighlightSquare, &mut Transform)>,
) {
	let mut highlight_square = highlight_square
		.get_single_mut()
		.expect("failed to get highlight_square");
	if let Some(selected) = selected.0 {
		highlight_square.1.translation.x = Coord::to_win_piece(selected.pos.col);
		highlight_square.1.translation.y = Coord::to_win_piece(selected.pos.row);
	} else {
		highlight_square.1.translation.x = Coord::to_win_piece(-1.);
		highlight_square.1.translation.y = Coord::to_win_piece(-1.);
	}
}

fn highlight_moved_system(
	mut moved_square: Query<(&MovedSquare, &mut Transform)>,
	mut ev_move: EventReader<MoveEvent>,
) {
	let mut moved_square = moved_square
		.get_single_mut()
		.expect("failed to get moved_square");
	for event in ev_move.iter() {
		if let Some(position) = event.0 {
			moved_square.1.translation.x = Coord::to_win_piece(position.col);
			moved_square.1.translation.y = Coord::to_win_piece(position.row);
		} else {
			moved_square.1.translation.x = Coord::to_win_piece(-1.);
			moved_square.1.translation.y = Coord::to_win_piece(-1.);
		}
	}
}

fn highlight_hover_system(
	mut hover_square: Query<(&HoverSquare, &mut Transform)>,
	mut ev_move: EventReader<HoverEvent>,
) {
	let mut hover_square = hover_square
		.get_single_mut()
		.expect("failed to get hover_square");
	for event in ev_move.iter() {
		if let Some(position) = event.0 {
			hover_square.1.translation.x = Coord::to_win_piece(position.col);
			hover_square.1.translation.y = Coord::to_win_piece(position.row);
		} else {
			hover_square.1.translation.x = Coord::to_win_piece(-1.);
			hover_square.1.translation.y = Coord::to_win_piece(-1.);
		}
	}
}

fn highlight_legal_moves_system(
	mut ev_legal: EventReader<LegalMoveEvent>,
	mut commands: Commands,
	mut materials: ResMut<Assets<ColorMaterial>>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut markers: Query<(&LegalMoveMarker, Entity)>,
) {
	for event in ev_legal.iter() {
		if let Some(legal_moves) = &event.0 {
			for (_, entity) in markers.iter_mut() {
				commands.entity(entity).despawn_recursive();
			}
			for legal_move in legal_moves.iter() {
				let row = *legal_move / BOARD_SIZE;
				let col = *legal_move % BOARD_SIZE;
				commands
					.spawn(MaterialMesh2dBundle {
						mesh: meshes.add(shape::Circle::new(50.).into()).into(),
						material: materials.add(ColorMaterial::from(Color::rgba_u8(0, 0, 0, 100))),
						transform: Transform {
							translation: Vec3::new(
								Coord::to_win(col + 1, -0.5),
								Coord::to_win(row + 1, -0.5),
								10.0,
							),
							scale: Vec3::new(0.25, 0.25, 0.0),
							..default()
						},
						..default()
					})
					.insert(LegalMoveMarker);
			}
		} else {
			for (_, entity) in markers.iter_mut() {
				commands.entity(entity).despawn_recursive();
			}
		}
	}
}
