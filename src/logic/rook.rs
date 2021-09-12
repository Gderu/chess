use super::piece::{Piece, Board, is_valid_pos};
use colored::*;
use super::King;
use std::any::Any;

#[derive(Clone)]
pub struct Rook {
    pos: (i8, i8),
    color: bool, //true is white, false is black
    first_move: bool,
}

impl Piece for Rook {
    fn new(pos: (i8, i8), color: bool) -> Option<Box<dyn Piece>> {
        Some(Box::new(Rook { pos, color, first_move: true }))
    }

    fn print(&self) {
        if self.color {
            print!("{}", "r".bright_white());
        } else {
            print!("{}", "r".black());
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
        for dir in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let mut to_check = (self.pos.0 + dir.0, self.pos.1 + dir.1);
            while is_valid_pos(to_check) {
                if let Some(piece) = board[to_check.0 as usize][to_check.1 as usize].as_ref() {
                    if piece.color() != self.color && (already_called || !king.is_check(&board, self.pos, to_check, en_passant, king_pos)) {
                        possible_moves.push(to_check);
                    }
                    break;
                } else if already_called || !king.is_check(&board, self.pos, to_check, en_passant, king_pos){
                    possible_moves.push(to_check);
                }
                to_check = (to_check.0 + dir.0, to_check.1 + dir.1);
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
}