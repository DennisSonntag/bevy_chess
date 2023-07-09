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
		BoardResource, HighlightSquare, HoverEvent, HoverSquare, LegalMoveEvent, LegalMoveMarker,
		MoveData, MoveEvent, MovedSquare, Piece, PieceColor, Pieces, Position, SelectedPiece,
		TakeEvent,
	},
	BOARD_SIZE, SQUARE_SIZE, WINDOW_SIZE,
};

pub struct PiecePlugin;

impl Plugin for PiecePlugin {
	fn build(&self, app: &mut App) {
		app.add_system(move_piece_system)
			.add_system(highlight_moved_system)
			.add_system(highlight_selected_system)
			.add_system(highlight_hover_system)
			.add_system(highlight_legal_moves_system);
	}
}

fn get_legal_moves(
	squares_to_edge: &HashMap<usize, Vec<i8>>,
	direction_offsets: [i8; 8],
	board: &[Piece; 64],
	start_square: i8,
	turn_color: &PieceColor,
) -> Vec<i8> {
	let mut moves: Vec<i8> = Vec::new();
	let piece = board[start_square as usize];
	if let Some(piece_type) = piece.piece {
		match piece_type {
			Pieces::Queen | Pieces::Rook | Pieces::Bishop => {
				let start_dir_index = if piece_type == Pieces::Bishop { 4 } else { 0 };

				let end_dir_index = if piece_type == Pieces::Rook { 4 } else { 8 };

				for direction_index in start_dir_index..end_dir_index {
					let direction_index = direction_index as usize;
					for n in 0..squares_to_edge
						.get(&(start_square as usize))
						.expect("squares to edge for start square not found")[direction_index]
					{
						let target_square =
							(start_square + direction_offsets[direction_index] * (n + 1));
						let piece_on_target_square = board[target_square as usize];

						// Blocked by friendly piece, so can't move any further in this direction
						if (piece_on_target_square.color == Some(*turn_color)) {
							break;
						}

						moves.push(target_square);

						// Can't move any further in this directoin after capturing opponent's piece
						if (piece_on_target_square.color != Some(*turn_color)
							&& piece_on_target_square.color != None)
						{
							break;
						}
					}
				}
			}
			Pieces::Knight => {
				let directions: [(i8, i8); 8] = [
					(1, 2),
					(2, 1),
					(-1, 2),
					(-2, 1),
					(1, -2),
					(2, -1),
					(-1, -2),
					(-2, -1),
				];

				let calc_idx = |row: i8, col: i8| (row * BOARD_SIZE + col);
				for &(dx, dy) in &directions {
					if let Some(position) = piece.pos {
						let new_row = position.row + dx;
						let new_col = position.col + dy;

						if (0..8).contains(&new_row) && (0..8).contains(&new_col) {
							moves.push(calc_idx(new_row, new_col));
						}
					}
				}
			}
			Pieces::Pawn => {
				let valid_offsets = if piece.color == Some(PieceColor::White) {
					([8, 16], [9, 7])
				} else {
					([-8, -16], [-9, -7])
				};
				for offset in valid_offsets.0 {
					let target_square = (start_square + offset);
					let piece_on_target_square = board[target_square as usize];

					// Blocked by friendly piece, so can't move any further in this direction
					if (piece_on_target_square.color == Some(*turn_color)) {
						break;
					}
					if offset.abs() == 16 && piece.amount_moved != 0 {
						break;
					}
					if (offset.abs() == 8 || offset.abs() == 16)
						&& piece_on_target_square.color != None
					{
						break;
					}

					moves.push(target_square);

					// Can't move any further in this directoin after capturing opponent's piece
					if (piece_on_target_square.color != Some(*turn_color)
						&& piece_on_target_square.color != None)
					{
						break;
					}
				}
				for offset in valid_offsets.1 {
					let target_square = (start_square + offset);
					let piece_on_target_square = board[target_square as usize];

					// Blocked by friendly piece, so can't move any further in this direction
					if (piece_on_target_square.color == Some(*turn_color)) {
						break;
					}

					if (offset.abs() == 9 || offset.abs() == 7)
						&& piece_on_target_square.color != Some(piece.color.unwrap().not())
					{
						continue;
					}

					moves.push(target_square);

					// Can't move any further in this directoin after capturing opponent's piece
					if (piece_on_target_square.color != Some(*turn_color)
						&& piece_on_target_square.color != None)
					{
						break;
					}
				}
			}
			Pieces::King => {
				for (idx, val) in direction_offsets.iter().enumerate() {
					if squares_to_edge
						.get(&(start_square as usize))
						.expect("could not find squares_to_edge from start_square")[idx]
						> 0
					{
						let target_square = (start_square + val);
						let piece_on_target_square = board[target_square as usize];

						// Blocked by friendly piece, so can't move any further in this direction
						if (piece_on_target_square.color == Some(*turn_color)) {
							continue;
						}
						moves.push(target_square);
					}
				}
			}
		}
	}

	moves
}

pub fn move_piece_system(
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
) {
	let window = windows.get_single().expect("could not get window");

	#[allow(clippy::cast_possible_truncation)]
	if let Some(position) = window.cursor_position() {
		let col = ((position[0] / 75.).floor()) as i8;
		let row = ((position[1] / 75.).floor()) as i8;

		let index = (row * BOARD_SIZE + col) as usize;

		let clicked_piece = board.board[index];
		if mouse_button_input.just_pressed(MouseButton::Left) {
			if Some(clicked_piece) == selected_piece.0 && selected_piece.0.is_some() {
				// if piece is already selected deselect it
				selected_piece.0 = None;
				ev_legal.send(LegalMoveEvent(None));
				ev_move.send(MoveEvent { pos: None });
			} else if clicked_piece.piece != None && clicked_piece.color == Some(current_state.0) {
				//if piece isnt selected select it
				selected_piece.0 = Some(clicked_piece);
				if let Some(selected) = selected_piece.0 {
					if let Some(position) = selected.pos {
						let selected_index = position.row * BOARD_SIZE + position.col;

						let legal_moves = get_legal_moves(
							&move_info.clone().num_squares_to_edge,
							move_info.direction_offsets,
							&board.board,
							selected_index,
							&current_state.0.clone(),
						);
						ev_legal.send(LegalMoveEvent(Some(legal_moves)));
					}
				}
				ev_move.send(MoveEvent { pos: None });
				ev_hover.send(HoverEvent { pos: None });
			};
		}
		if mouse_button_input.pressed(MouseButton::Left) {
			if let Some(selected) = selected_piece.0 {
				if let Some(position) = selected.pos {
					let selected_index = position.row * BOARD_SIZE + position.col;

					let legal_moves = get_legal_moves(
						&move_info.clone().num_squares_to_edge,
						move_info.direction_offsets,
						&board.board,
						selected_index,
						&current_state.0.clone(),
					);

					ev_legal.send(LegalMoveEvent(Some(legal_moves.clone())));
					let clicked_index = row * BOARD_SIZE + col;

					if clicked_piece.color != Some(current_state.0)
						&& legal_moves.contains(&(clicked_index))
					{
						ev_hover.send(HoverEvent {
							pos: Some(Position::new(row, col)),
						});
					} else if clicked_piece.color == Some(current_state.0) {
						ev_hover.send(HoverEvent { pos: None });
					}
				}
				for (piece, mut transform, _) in pieces.iter_mut() {
					if Some(*piece) == selected_piece.0 {
						transform.translation.x = position.x - (WINDOW_SIZE / 2.);
						transform.translation.y = position.y - (WINDOW_SIZE / 2.);
						transform.translation.z = 30.;
					}
				}
			}
		}
		if mouse_button_input.just_released(MouseButton::Left) {
			ev_hover.send(HoverEvent { pos: None });

			if let Some(selected) = selected_piece.0 {
				if let Some(mut position) = selected.pos {
					let selected_index = position.row * BOARD_SIZE + position.col;

					let legal_moves = get_legal_moves(
						&move_info.clone().num_squares_to_edge,
						move_info.direction_offsets,
						&board.board,
						selected_index,
						&current_state.0.clone(),
					);

					let clicked_index = row * BOARD_SIZE + col;
					if selected != clicked_piece
						&& clicked_piece.color != Some(current_state.0)
						&& legal_moves.contains(&{ clicked_index })
					{
						for (mut piece, mut transform, entity) in pieces.iter_mut() {
							if piece.as_ref() == &selected {
								transform.translation.x = f32::from(col)
									.mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.))
									+ (SQUARE_SIZE / 2.);
								transform.translation.y = f32::from(row)
									.mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.))
									+ (SQUARE_SIZE / 2.);

								transform.translation.z = 2.;
								piece.amount_moved += 1;
								let new_position = Position::new(row, col);
								piece.pos = Some(new_position);

								let old_index = (position.row * BOARD_SIZE + position.col) as usize;

								board.board[old_index] = Piece::default();

								board.board[index] = *piece;
								board.board[index].pos = Some(new_position);

								ev_move.send(MoveEvent {
									pos: Some(new_position),
								});
							}
							if *piece.as_ref() == clicked_piece
								&& clicked_piece.color != None && clicked_piece.color
								!= Some(current_state.0)
							{
								commands.entity(entity).despawn_recursive();
								ev_take.send(TakeEvent);
							}
						}
						selected_piece.0 = None;
						ev_legal.send(LegalMoveEvent(None));
						next_state.set(current_state.0.not());
					} else if selected == clicked_piece
						|| clicked_piece.color == Some(current_state.0)
						|| !legal_moves.contains(&{ clicked_index })
					{
						for (piece, mut transform, _) in pieces.iter_mut() {
							if piece.as_ref() == &selected {
								if let Some(position) = piece.pos {
									transform.translation.x = f32::from(position.col)
										.mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.))
										+ (SQUARE_SIZE / 2.);
									transform.translation.y = f32::from(position.row)
										.mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.))
										+ (SQUARE_SIZE / 2.);
									transform.translation.z = 2.;
								}
							}
						}
					}
				}
			}
		}
	}
}

pub fn highlight_selected_system(
	selected: Res<SelectedPiece>,
	mut highlight_square: Query<(&HighlightSquare, &mut Transform)>,
) {
	let mut highlight_square = highlight_square
		.get_single_mut()
		.expect("failed to get highlight_square");
	if let Some(selected) = selected.0 {
		if let Some(position) = selected.pos {
			highlight_square.1.translation.x = f32::from(position.col)
				.mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.))
				+ (SQUARE_SIZE / 2.);

			highlight_square.1.translation.y = f32::from(position.row)
				.mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.))
				+ (SQUARE_SIZE / 2.);
		}
	} else {
		highlight_square.1.translation.x =
			(-1.0f32).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)) + (SQUARE_SIZE / 2.);
		highlight_square.1.translation.y =
			(-1.0f32).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)) + (SQUARE_SIZE / 2.);
	}
}

pub fn highlight_moved_system(
	mut moved_square: Query<(&MovedSquare, &mut Transform)>,
	mut ev_move: EventReader<MoveEvent>,
) {
	let mut moved_square = moved_square
		.get_single_mut()
		.expect("failed to get moved_square");
	for event in ev_move.iter() {
		if let Some(position) = event.pos {
			moved_square.1.translation.x = f32::from(position.col)
				.mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.))
				+ (SQUARE_SIZE / 2.);
			moved_square.1.translation.y = f32::from(position.row)
				.mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.))
				+ (SQUARE_SIZE / 2.);
		} else {
			moved_square.1.translation.x =
				(-1.0f32).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)) + (SQUARE_SIZE / 2.);
			moved_square.1.translation.y =
				(-1.0f32).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)) + (SQUARE_SIZE / 2.);
		}
	}
}

pub fn highlight_hover_system(
	mut hover_square: Query<(&HoverSquare, &mut Transform)>,
	mut ev_move: EventReader<HoverEvent>,
) {
	let mut hover_square = hover_square
		.get_single_mut()
		.expect("failed to get hover_square");
	for event in ev_move.iter() {
		if let Some(position) = event.pos {
			hover_square.1.translation.x = f32::from(position.col)
				.mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.))
				+ (SQUARE_SIZE / 2.);
			hover_square.1.translation.y = f32::from(position.row)
				.mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.))
				+ (SQUARE_SIZE / 2.);
		} else {
			hover_square.1.translation.x =
				(-1.0f32).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)) + (SQUARE_SIZE / 2.);
			hover_square.1.translation.y =
				(-1.0f32).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)) + (SQUARE_SIZE / 2.);
		}
	}
}

pub fn highlight_legal_moves_system(
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
								(f32::from(col + 1) - 0.5)
									.mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)),
								(f32::from(row + 1) - 0.5)
									.mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)),
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
