pub mod piece;
mod pawn;
mod knight;
mod rook;
mod bishop;
mod queen;
mod king;

use piece::{Piece, Board, PieceTypes};
use pawn::Pawn;
use knight::Knight;
use rook::Rook;
use bishop::Bishop;
use queen::Queen;
use king::King;


pub struct LogicManager {
    board: Board,
    curr_selected: (i8, i8),
    possible_moves: Vec<(i8, i8)>,
    en_passant: Option<(i8, i8)>,
    black_king: (i8, i8),
    white_king: (i8, i8),
    past_positions: Vec<(Vec<Vec<String>>, i8)>,
    turns_since_capture: i8,
    stop: bool,
}

impl LogicManager {
    //creates a new LogicManager instance and return it
    pub fn new() -> LogicManager {
        let mut board = Vec::with_capacity(8);
        board.push(LogicManager::create_back_line(false));
        for i in 1..7 {
            let mut row = Vec::with_capacity(8);
            for j in 0..8 {
                if i == 1 {
                    row.push(Pawn::new((i, j), false));
                }
                else if i == 6 {
                    row.push(Pawn::new((i, j), true));
                }
                else {
                    row.push(None);
                }
            }
            board.push(row);
        }
        board.push(LogicManager::create_back_line(true));

        let mut res = LogicManager {
            board,
            curr_selected: (-1, -1),
            possible_moves: vec![],
            en_passant: None,
            black_king: (0, 4),
            white_king: (7, 4),
            past_positions: vec![],
            turns_since_capture: 0,
            stop: false,
        };
        res.add_board_to_list();
        res

    }

    //gets a reference to the board
    pub fn get_board(&self) -> &Board {
        &self.board
    }

    //gets all possible moves for a piece at pos. Returns None if there is no piece there. Must be called before moving
    pub fn get_possible_moves(&mut self, pos: (i8, i8)) -> Option<&Vec<(i8, i8)>> {
        if self.stop {
            return None;
        }
        if self.curr_selected != (-1, -1) {
            self.possible_moves.clear();
        }
        if let Some(piece) = self.board[pos.0 as usize][pos.1 as usize].as_ref() {
            self.curr_selected = pos;
            let king_pos = match piece.color() {
                false => self.black_king,
                true => self.white_king,
            };
            self.possible_moves = piece.get_possible_moves(&self.board, &self.en_passant, king_pos, false);
            return Some(&self.possible_moves);
        }
        None
    }

    pub fn promote_pawn(&mut self, new_pos: (i8, i8), into: PieceTypes) {
        let color = self.get_piece_color(self.curr_selected).unwrap();
        self.board[self.curr_selected.0 as usize][self.curr_selected.1 as usize] = None;//moving the piece on the board
        self.board[new_pos.0 as usize][new_pos.1 as usize] = match into {
            PieceTypes::Queen => Queen::new(new_pos, color),
            PieceTypes::Rook => Rook::new(new_pos, color),
            PieceTypes::Bishop => Bishop::new(new_pos, color),
            PieceTypes::Knight => Knight::new(new_pos, color),
            _ => panic!("Promoted pawn to illegal piece"),
        };
    }

    //moves a piece to new_pos. Must be called after get_possible_moves. Returns other piece to move if necessary.
    pub fn move_piece(&mut self, new_pos: (i8, i8)) -> Option<((i8, i8), (i8, i8))> {
        let mut to_return = None;
        let first_move = self.get_piece(self.curr_selected).is_first_move();
        let used_en_passant = self.get_piece(self.curr_selected).took_using_en_passant(new_pos, &self.board);
        if used_en_passant {
            if let Some(piece_taken) = self.board[(new_pos.0 + 1) as usize][new_pos.1 as usize].as_ref() {
                if !piece_taken.possible_en_passant().is_none() && piece_taken.color() == false {
                    self.board[(new_pos.0 + 1) as usize][new_pos.1 as usize] = None;
                    to_return = Some((((new_pos.0 + 1), new_pos.1), (-1, -1)));
                }
            }
            if let Some(piece_taken) = self.board[(new_pos.0 - 1) as usize][new_pos.1 as usize].as_ref() {
                if !piece_taken.possible_en_passant().is_none() && piece_taken.color() == true {
                    self.board[(new_pos.0 - 1) as usize][new_pos.1 as usize] = None;
                    to_return = Some((((new_pos.0 - 1), new_pos.1), (-1, -1)));
                }
            }
        } else if self.get_piece(self.curr_selected).piece_type() == PieceTypes::Pawn && [0, 7].contains(&new_pos.0) {
            return None;
        }

        self.get_mut_piece(self.curr_selected).move_piece(new_pos);//telling the piece it has moved
        if let Some(pos) =
            self.get_piece(self.curr_selected).possible_en_passant() {//if en passant occurred, mark it on board
            self.en_passant = Some(pos);
        } else {
            self.en_passant = None;
        }

        if self.get_piece(self.curr_selected).piece_type() == PieceTypes::King {
            let pos = match self.get_piece(self.curr_selected).color() {
                true => self.white_king,
                false => self.black_king,
            };
            if self.get_piece(self.curr_selected).color() {
                self.white_king = new_pos;
            } else {
                self.black_king = new_pos;
            }
            if first_move && new_pos == (pos.0, 6) {
                self.get_mut_piece((pos.0, 7)).move_piece((pos.0, 5));//telling the piece it has moved
                self.board[pos.0 as usize][5] =
                    Some(self.board[pos.0 as usize][7].as_ref().unwrap().clone());
                self.board[pos.0 as usize][7] = None;//moving the piece on the board
                to_return = Some(((pos.0, 7), (pos.0, 5)));
            } else if first_move && new_pos == (pos.0, 2) {
                self.get_mut_piece((pos.0, 0)).move_piece((pos.0, 3));//telling the piece it has moved
                self.board[pos.0 as usize][3] =
                    Some(self.board[pos.0 as usize][0].as_ref().unwrap().clone());
                self.board[pos.0 as usize][0] = None;//moving the piece on the board
                to_return = Some(((pos.0, 0), (pos.0, 3)));
            }
        }

        let was_piece_taken =  self.board[new_pos.0 as usize][new_pos.1 as usize].is_some();

        self.board[new_pos.0 as usize][new_pos.1 as usize] =
            Some(self.board[self.curr_selected.0 as usize][self.curr_selected.1 as usize].as_ref().unwrap().clone());
        self.board[self.curr_selected.0 as usize][self.curr_selected.1 as usize] = None;//moving the piece on the board
        self.curr_selected = (-1, -1);

        if was_piece_taken {
            self.turns_since_capture = 0;
            self.past_positions.clear();
        } else {
            self.turns_since_capture += 1;
            self.add_board_to_list();
        }
        to_return
    }

    pub fn is_in_possible_moves(&self, pos: (i8, i8)) -> bool {
        self.possible_moves.contains(&pos)
    }

    pub fn is_check(&self, color: bool) -> bool {
        let other_king_pos = match color {
            true => self.black_king,
            false => self.white_king,
        };
        self.get_piece(other_king_pos)
            .as_any().downcast_ref::<King>().unwrap()
            .is_check(&self.board, (-1, -1), (-1, -1), &self.en_passant, other_king_pos)
    }

    pub fn is_checkmate(&self, color: bool) -> bool {
        let other_king_pos = match color {
            true => self.black_king,
            false => self.white_king,
        };
        self.get_piece(other_king_pos)
            .as_any().downcast_ref::<King>().unwrap()
            .is_checkmate(&self.board, &self.en_passant, other_king_pos)
    }

    pub fn is_draw(&self) -> bool {
        if self.turns_since_capture >= 100 {
            true
        } else if self.past_positions.iter().find(|(_board, n)| *n >= 3).is_some() {
            true
        } else {
            false
        }
    }

    pub fn can_move(&self) -> bool {
        self.curr_selected != (-1, -1)
    }

    pub fn clear_selection(&mut self) {
        self.possible_moves.clear();
        self.curr_selected = (-1, -1);
    }

    pub fn get_piece_color(&self, pos: (i8, i8)) -> Option<bool> {
        if let Some(_piece) = self.board[pos.0 as usize][pos.1 as usize].as_ref() {
            Some(self.get_piece(pos).color())
        } else {
            None
        }
    }

    pub fn stop(&mut self) {
        self.stop = true;
    }

    pub fn is_stop(&self) -> bool {
        self.stop
    }

    fn get_piece(&self, pos: (i8, i8)) -> &Box<dyn Piece> {
        if let Some(piece) = self.board[pos.0 as usize][pos.1 as usize].as_ref() {
            piece
        } else{
            panic!("Called get piece on empty square!");
        }
    }

    fn get_mut_piece(&mut self, pos: (i8, i8)) -> &mut Box<dyn Piece> {
        if let Some(piece) = self.board[pos.0 as usize][pos.1 as usize].as_mut() {
            piece
        } else{
            panic!("Called get piece on empty square!");
        }
    }

    fn create_back_line(color: bool) -> Vec<Option<Box<dyn Piece>>> {
        let i = match color { false => 0, true => 7};
        vec![Rook::new((i, 0), color), Knight::new((i, 1), color), Bishop::new((i, 2), color), Queen::new((i, 3), color), King::new((i, 4), color), Bishop::new((i, 5), color), Knight::new((i, 6), color), Rook::new((i, 7), color)]
    }

    fn add_board_to_list(&mut self) {
        let mut simplified_board = vec![];
        for row in &self.board {
            let mut simplified_row = vec![];
            for sqr in row {
                let mut to_add;
                if let Some(piece) = sqr {
                    to_add = match piece.piece_type() {
                        PieceTypes::King => "k",
                        PieceTypes::Queen => "q",
                        PieceTypes::Rook => "r",
                        PieceTypes::Bishop => "b",
                        PieceTypes::Knight => "n",
                        PieceTypes::Pawn => "p",
                    }.to_string();
                    to_add += match piece.color() {
                        true => "w",
                        false => "b",
                    };
                } else {
                    to_add = " ".to_string();
                }
                simplified_row.push(to_add);
            }
            simplified_board.push(simplified_row);
        }
        let pos = self.past_positions.iter().position(|(b, _i)| b == &simplified_board);
        if let Some(pos) = pos {
            self.past_positions[pos].1 += 1;
        } else {
            self.past_positions.push((simplified_board, 1));
        }
    }
}

unsafe impl Send for LogicManager {}
unsafe impl Sync for LogicManager {}