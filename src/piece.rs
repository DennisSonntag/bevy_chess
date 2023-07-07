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
		MoveData, MoveEvent, MovedSquare, Piece, PieceColor, Pieces, SelectedPiece, TakeEvent,
		Turn,
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
	turn_color: &Turn,
) -> Vec<i8> {
	let mut moves: Vec<i8> = Vec::new();
	let piece = board[start_square as usize];
	match piece.piece {
		Pieces::Queen | Pieces::Rook | Pieces::Bishop => {
			let start_dir_index = if piece.piece == Pieces::Bishop { 4 } else { 0 };

			let end_dir_index = if piece.piece == Pieces::Rook { 4 } else { 8 };

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
					if (&piece_on_target_square.color == turn_color) {
						break;
					}

					moves.push(target_square);

					// Can't move any further in this directoin after capturing opponent's piece
					if (&piece_on_target_square.color != turn_color
						&& piece_on_target_square.color != PieceColor::None)
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
				let new_row = piece.row.unwrap_or(0) + dx;
				let new_col = piece.col.unwrap_or(0) + dy;

				if (0..8).contains(&new_row) && (0..8).contains(&new_col) {
					moves.push(calc_idx(new_row, new_col));
				}
			}
		}
		Pieces::Pawn => {
			let valid_moves = if piece.color == PieceColor::White {
				([8, 16], [9, 7])
			} else {
				([-8, -16], [-9, -7])
			};
			for i in valid_moves.0 {
				let target_square = (start_square + i);
				let piece_on_target_square = board[target_square as usize];

				// Blocked by friendly piece, so can't move any further in this direction
				if (&piece_on_target_square.color == turn_color) {
					break;
				}
				if i.abs() == 16 && piece.amount_moved != 0 {
				    break;
				}
				if (i.abs() == 8 || i.abs() == 16)
					&& piece_on_target_square.color != PieceColor::None
				{
					break;
				}

				moves.push(target_square);

				// Can't move any further in this directoin after capturing opponent's piece
				if (&piece_on_target_square.color != turn_color
					&& piece_on_target_square.color != PieceColor::None)
				{
					break;
				}
			}
			for i in valid_moves.1 {
				let target_square = (start_square + i);
				let piece_on_target_square = board[target_square as usize];

				// Blocked by friendly piece, so can't move any further in this direction
				if (&piece_on_target_square.color == turn_color) {
					break;
				}

				if (i.abs() == 9 || i.abs() == 7)
					&& piece_on_target_square.color != piece.color.not()
				{
					continue;
				}

				moves.push(target_square);

				// Can't move any further in this directoin after capturing opponent's piece
				if (&piece_on_target_square.color != turn_color
					&& piece_on_target_square.color != PieceColor::None)
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
					if (&piece_on_target_square.color == turn_color) {
						continue;
					}
					moves.push(target_square);
				}
			}
		}
		Pieces::None => unreachable!(),
	}

	moves
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
				ev_move.send(MoveEvent {
					row: None,
					col: None,
				});
			} else if clicked_piece.piece != Pieces::None && clicked_piece.color == current_state.0
			{
				//if piece isnt selected select it
				selected_piece.0 = Some(clicked_piece);
				if let Some(selected) = selected_piece.0 {
					let selected_index =
						selected.row.unwrap_or(0) * BOARD_SIZE + selected.col.unwrap_or(0);

					let legal_moves = get_legal_moves(
						&move_info.clone().num_squares_to_edge,
						move_info.direction_offsets,
						&board.board,
						selected_index,
						&current_state.0.clone(),
					);
					ev_legal.send(LegalMoveEvent(Some(legal_moves)));
				}
				ev_move.send(MoveEvent {
					row: None,
					col: None,
				});
				ev_hover.send(HoverEvent {
					row: None,
					col: None,
				});
			};
		}
		if mouse_button_input.pressed(MouseButton::Left) {
			if let Some(selected) = selected_piece.0 {
				let selected_index =
					selected.row.unwrap_or(0) * BOARD_SIZE + selected.col.unwrap_or(0);

				let legal_moves = get_legal_moves(
					&move_info.clone().num_squares_to_edge,
					move_info.direction_offsets,
					&board.board,
					selected_index,
					&current_state.0.clone(),
				);

				ev_legal.send(LegalMoveEvent(Some(legal_moves.clone())));
				let clicked_index = row * BOARD_SIZE + col;

				if clicked_piece.color != current_state.0 && legal_moves.contains(&(clicked_index))
				{
					ev_hover.send(HoverEvent {
						row: Some(row),
						col: Some(col),
					});
				} else if clicked_piece.color == current_state.0 {
					ev_hover.send(HoverEvent {
						row: None,
						col: None,
					});
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
		if mouse_button_input.just_released(MouseButton::Left) {
			ev_hover.send(HoverEvent {
				row: None,
				col: None,
			});

			if let Some(selected) = selected_piece.0 {
				let selected_index =
					selected.row.unwrap_or(0) * BOARD_SIZE + selected.col.unwrap_or(0);

				let legal_moves = get_legal_moves(
					&move_info.clone().num_squares_to_edge,
					move_info.direction_offsets,
					&board.board,
					selected_index,
					&current_state.0.clone(),
				);

				let clicked_index = row * BOARD_SIZE + col;
				if selected != clicked_piece
					&& clicked_piece.color != current_state.0
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
							piece.row = Some(row);
							piece.amount_moved += 1;
							piece.col = Some(col);

							let old_index = (selected.row.unwrap_or(0) * BOARD_SIZE
								+ selected.col.unwrap_or(0)) as usize;

							board.board[old_index] = Piece {
								color: PieceColor::None,
								piece: Pieces::None,
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
					ev_legal.send(LegalMoveEvent(None));
					next_state.set(match current_state.0 {
						Turn::White => Turn::Black,
						Turn::Black => Turn::White,
					});
				} else if selected == clicked_piece
					|| clicked_piece.color == current_state.0
					|| !legal_moves.contains(&{ clicked_index })
				{
					for (piece, mut transform, _) in pieces.iter_mut() {
						if piece.as_ref() == &selected {
							if let (Some(row), Some(col)) = (piece.row, piece.col) {
								transform.translation.x = f32::from(col)
									.mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.))
									+ (SQUARE_SIZE / 2.);
								transform.translation.y = f32::from(row)
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

pub fn highlight_selected_system(
	selected: Res<SelectedPiece>,
	mut highlight_square: Query<(&HighlightSquare, &mut Transform)>,
) {
	let mut highlight_square = highlight_square
		.get_single_mut()
		.expect("failed to get highlight_square");
	if let Some(selected) = selected.0 {
		highlight_square.1.translation.x = f32::from(selected.col.unwrap_or(0))
			.mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.))
			+ (SQUARE_SIZE / 2.);

		highlight_square.1.translation.y = f32::from(selected.row.unwrap_or(0))
			.mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.))
			+ (SQUARE_SIZE / 2.);
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
	for i in ev_move.iter() {
		if let (Some(row), Some(col)) = (i.row, i.col) {
			moved_square.1.translation.x =
				f32::from(col).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)) + (SQUARE_SIZE / 2.);
			moved_square.1.translation.y =
				f32::from(row).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)) + (SQUARE_SIZE / 2.);
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
	for i in ev_move.iter() {
		if let (Some(row), Some(col)) = (i.row, i.col) {
			hover_square.1.translation.x =
				f32::from(col).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)) + (SQUARE_SIZE / 2.);
			hover_square.1.translation.y =
				f32::from(row).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)) + (SQUARE_SIZE / 2.);
		} else {
			hover_square.1.translation.x =
				(-1.0f32).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)) + (SQUARE_SIZE / 2.);
			hover_square.1.translation.y =
				(-1.0f32).mul_add(SQUARE_SIZE, -(WINDOW_SIZE / 2.)) + (SQUARE_SIZE / 2.);
		}
	}
}

pub fn highlight_legal_moves_system(
	mut ev_move: EventReader<LegalMoveEvent>,
	mut commands: Commands,
	mut materials: ResMut<Assets<ColorMaterial>>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut markers: Query<(&LegalMoveMarker, Entity)>,
) {
	for event in ev_move.iter() {
		if let Some(moves) = &event.0 {
			for (_, entity) in markers.iter_mut() {
				commands.entity(entity).despawn_recursive();
			}
			for i in moves.iter() {
				let row = *i / BOARD_SIZE;
				let col = *i % BOARD_SIZE;
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
