pub type Board = Vec<Vec<Option<Box<dyn Piece>>>>;
use std::any::Any;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PieceTypes {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

pub fn is_valid_pos(pos: (i8, i8)) -> bool {
    pos.0 < 8 && pos.0 >= 0 && pos.1 < 8 && pos.1 >= 0
}

pub trait Piece {
    //Gets position and color of piece, returns piece object
    fn new(pos: (i8, i8), color: bool) -> Option<Box<dyn Piece>> where Self: Sized;
    //checks if the piece is a king
    fn piece_type(&self) -> PieceTypes;
    //prints the piece in ascii
    fn print(&self);
    //returns the color of the piece, true is white, false is black
    fn color(&self) -> bool;
    //tells the piece to change its position data to pos. Must be called after get_possible_moves
    fn move_piece(&mut self, pos: (i8, i8));
    //gets all possible current moves and returns them. Must be called before move_piece
    fn get_possible_moves(&self, board: &Board, en_passant: &Option<(i8, i8)>, king_pos: (i8, i8), already_called: bool) -> Vec<(i8, i8)>;
    //must be implemented for cloning of Piece
    fn box_clone(&self) -> Box<dyn Piece>;
    //Returns None if no en passant occurred, otherwise returns location of en passant possible
    fn possible_en_passant(&self) -> Option<(i8, i8)> {
        None
    }
    //true if last move was en passant
    fn took_using_en_passant(&self, _new_pos: (i8, i8), _board: &Board) -> bool { false }

    fn as_any(&self) -> &dyn Any;

    fn is_first_move(&self) -> bool {
        false
    }
}

impl Clone for Box<dyn Piece> {
    fn clone(&self) -> Box<dyn Piece> {
        self.box_clone()
    }
}