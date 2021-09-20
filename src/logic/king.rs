use super::{Piece, Board, piece::is_valid_pos, PieceTypes};
use colored::*;
use std::any::Any;

#[derive(Clone)]
pub struct King {
    pos: (i8, i8),
    color: bool, //true is white, false is black
    first_move: bool,
}


impl Piece for King {
    fn new(pos: (i8, i8), color: bool) -> Option<Box<dyn Piece>> {
        Some(Box::new(King { pos, color, first_move: true }))
    }

    fn print(&self) {
        if self.color {
            print!("{}", "k".bright_white());
        } else {
            print!("{}", "k".black());
        }
    }

    fn color(&self) -> bool {
        self.color
    }

    fn move_piece(&mut self, pos: (i8, i8)) {
        self.pos = pos;
        self.first_move = false;
    }

    fn get_possible_moves(&self, board: &Board, en_passant: &Option<(i8, i8)>, _king_pos: (i8, i8), already_called: bool) -> Vec<(i8, i8)> {
        let mut possible_moves = vec![];
        for dir in [(1, 1), (-1, 1), (1, -1), (-1, -1), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            let to_check = (self.pos.0 + dir.0, self.pos.1 + dir.1);
            if !is_valid_pos(to_check) {
                continue;
            }
            if already_called || !self.is_check(&board, self.pos, to_check, en_passant, to_check) {
                if let Some(piece) = board[to_check.0 as usize][to_check.1 as usize].as_ref() {
                    if piece.color() != self.color {
                        possible_moves.push(to_check);
                    }
                } else {
                    possible_moves.push(to_check);
                }
            }
        }
        if self.first_move && !already_called {
            //Kingside castle
            if board[self.pos.0 as usize][5].is_none() &&
                board[self.pos.0 as usize][6].is_none() &&
                !self.is_check(&board, self.pos, (self.pos.0, 5), en_passant, (self.pos.0, 5)) &&
                !self.is_check(&board, self.pos, (self.pos.0, 6), en_passant, (self.pos.0, 6)) {

                if let Some(piece) = board[self.pos.0 as usize][7].as_ref() {
                    if piece.is_first_move() {
                        possible_moves.push((self.pos.0, 6));
                    }
                }
            }

            //Queenside castle
            if board[self.pos.0 as usize][3].is_none() &&
                board[self.pos.0 as usize][2].is_none() &&
                board[self.pos.0 as usize][1].is_none() &&
                !self.is_check(&board, self.pos, (self.pos.0, 3), en_passant, (self.pos.0, 3)) &&
                !self.is_check(&board, self.pos, (self.pos.0, 2), en_passant, (self.pos.0, 2)) &&
                !self.is_check(&board, self.pos, (self.pos.0, 1), en_passant, (self.pos.0, 1)) {

                if let Some(piece) = board[self.pos.0 as usize][0].as_ref() {
                    if piece.is_first_move() {
                        possible_moves.push((self.pos.0, 2));
                    }
                }
            }
        }
        possible_moves
    }

    fn box_clone(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_first_move(&self) -> bool {
        self.first_move
    }

    fn piece_type(&self) -> PieceTypes {
        PieceTypes::King
    }
}

impl King {
    pub fn is_check(&self, board: &Board, orig_pos: (i8, i8), dest_pos: (i8, i8), en_passant: &Option<(i8, i8)>, king_pos: (i8, i8)) -> bool {
        let mut board_copy = board.clone();
        if orig_pos != dest_pos {
            board_copy[dest_pos.0 as usize][dest_pos.1 as usize] = board_copy[orig_pos.0 as usize][orig_pos.1 as usize].clone();
            board_copy[orig_pos.0 as usize][orig_pos.1 as usize] = None;
        }
        for row in &board_copy {
            for sqr in row {
                if let Some(piece) = sqr {
                    if piece.color() != self.color && piece.get_possible_moves(&board_copy, en_passant, king_pos, true).contains(&king_pos) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn is_checkmate(&self, board: &Board, en_passant: &Option<(i8, i8)>, other_king_pos: (i8, i8)) -> bool {
        for row in board {
            for sqr in row {
                if let Some(piece) = sqr {
                    if piece.color() == self.color && piece.get_possible_moves(&board, en_passant, other_king_pos, false).len() != 0 {
                        return false;
                    }
                }
            }
        }
        true
    }
}