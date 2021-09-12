use crate::logic::piece::{Board};

pub fn print_board_ascii(board: &Board) {
    for row in board {
        for sqr in row {
            match &sqr {
                Some(p) => p.print(),
                None => print!("{}", "."),
            }
        }
        println!();
    }
    println!();

}