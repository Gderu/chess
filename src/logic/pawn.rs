use super::piece::{Piece, Board, is_valid_pos};
use super::King;
use colored::*;
use std::any::Any;

#[derive(Clone)]
pub struct Pawn {
    pos: (i8, i8),
    color: bool, //true is white, false is black
    first_move: bool,
    en_passant: Option<(i8, i8)>,
}

impl Piece for Pawn {
    fn new(pos: (i8, i8), color: bool) -> Option<Box<dyn Piece>> {
        Some(Box::new(Pawn { pos, color, first_move: true, en_passant: None}))
    }

    fn print(&self) {
        if self.color {
            print!("{}", "p".bright_white());
        } else {
            print!("{}", "p".black());
        }
    }

    fn color(&self) -> bool {
        self.color
    }

    fn move_piece(&mut self, pos: (i8, i8)) {
        if (pos.0 - self.pos.0).abs() == 2 {
            self.en_passant = Some(((pos.0 + self.pos.0) / 2, pos.1));
        } else {
            self.en_passant = None;
        }
        self.first_move = false;
        self.pos = pos;
    }

    fn get_possible_moves(&self, board: &Board, en_passant: &Option<(i8, i8)>, king_pos: (i8, i8), already_called: bool) -> Vec<(i8, i8)> {
        let dir = match self.color {
            true => -1,
            false => 1,
        };
        let king = board[king_pos.0 as usize][king_pos.1 as usize].as_ref().unwrap().as_any().downcast_ref::<King>().unwrap();
        let mut possible_moves = vec![];
        if is_valid_pos((self.pos.0 + dir, self.pos.1)) && board[(self.pos.0 + dir) as usize][self.pos.1 as usize].is_none() && (already_called || !king.is_check(&board, self.pos, (self.pos.0 + dir, self.pos.1), en_passant, king_pos)) {
            possible_moves.push((self.pos.0 + dir, self.pos.1));
            if is_valid_pos((self.pos.0 + 2 * dir, self.pos.1)) && self.first_move && board[(self.pos.0 + 2 * dir) as usize][self.pos.1 as usize].is_none() && (already_called || !king.is_check(&board, self.pos, (self.pos.0 + 2 * dir, self.pos.1), en_passant, king_pos)) {
                possible_moves.push((self.pos.0 + 2 * dir, self.pos.1));
            }
        }
        for i in [-1, 1] {
            if is_valid_pos((self.pos.0 + dir, self.pos.1 + i)) {
                if let Some(piece) = &board[(self.pos.0 + dir) as usize][(self.pos.1 + i) as usize] {
                    if piece.color() != self.color && (already_called || !king.is_check(&board, self.pos, (self.pos.0 + dir, self.pos.1 + i), en_passant, king_pos)) {
                        possible_moves.push((self.pos.0 + dir, self.pos.1 + i));
                    }
                }
                if en_passant.eq(&Some((self.pos.0 + dir, self.pos.1 + i))) && (already_called || !king.is_check(&board, self.pos, (self.pos.0 + dir, self.pos.1 + i), en_passant, king_pos)) {
                    possible_moves.push((self.pos.0 + dir, self.pos.1 + i));
                }
            }
        }
        possible_moves
    }

    fn box_clone(&self) -> Box<dyn Piece> {
        Box::new(self.clone())
    }

    fn possible_en_passant(&self) -> Option<(i8, i8)> {
        self.en_passant.clone()
    }

    fn took_using_en_passant(&self, new_pos: (i8, i8), board: &Board) -> bool {
        board[new_pos.0 as usize][new_pos.1 as usize].is_none()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_first_move(&self) -> bool {
        self.first_move
    }
}