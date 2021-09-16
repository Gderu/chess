use super::piece::{Piece, Board, is_valid_pos, PieceTypes};
use colored::*;
use super::King;
use std::any::Any;

#[derive(Clone)]
pub struct Knight {
    pos: (i8, i8),
    color: bool, //true is white, false is black
    first_move: bool,
}

impl Piece for Knight {
    fn new(pos: (i8, i8), color: bool) -> Option<Box<dyn Piece>> {
        Some(Box::new(Knight { pos, color, first_move: true }))
    }

    fn print(&self) {
        if self.color {
            print!("{}", "n".bright_white());
        } else {
            print!("{}", "n".black());
        }
    }

    fn color(&self) -> bool {
        self.color
    }

    fn move_piece(&mut self, pos: (i8, i8)) {
        self.pos = pos;
        self.first_move = false;
    }

    fn get_possible_moves(&self, board: &Board, en_passant: &Option<(i8, i8)>, king_pos: (i8, i8), already_called: bool) -> Vec<(i8, i8)> {
        let mut possible_moves = vec![];
        let king = board[king_pos.0 as usize][king_pos.1 as usize].as_ref().unwrap().as_any().downcast_ref::<King>().unwrap();
        for pos in [(1, 2), (1, -2), (-1, 2), (-1, -2), (2, 1), (2, -1), (-2, 1), (-2, -1)] {
            let to_check = (self.pos.0 + pos.0, self.pos.1 + pos.1);
            if is_valid_pos(to_check) {
                if let Some(piece) = board[to_check.0 as usize][to_check.1 as usize].as_ref() {
                    if piece.color() != self.color && (already_called || !king.is_check(&board, self.pos, to_check, en_passant, king_pos)) {
                        possible_moves.push(to_check);
                    }
                } else if already_called || !king.is_check(&board, self.pos, to_check, en_passant, king_pos){
                    possible_moves.push(to_check);
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

    fn piece_type(&self) -> PieceTypes {
        PieceTypes::Knight
    }
}