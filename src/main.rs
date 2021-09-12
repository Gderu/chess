use chess::logic::LogicManager;
use chess::gui::print_board_ascii;

fn main() {
    let mut lm = LogicManager::new();
    let mut buf = String::new();
    let mut turn = true;
    loop {
        print_board_ascii(lm.get_board());
        println!("What would you like to move?");
        buf.clear();
        std::io::stdin().read_line(&mut buf);
        let poss_moves = lm.get_possible_moves((56 - buf.as_bytes()[1] as i8, buf.as_bytes()[0] as i8 - 97));
        println!("{:?}", poss_moves);
        println!("where would you like to move?");
        buf.clear();
        std::io::stdin().read_line(&mut buf);
        if buf.as_bytes().len() < 2 || !poss_moves.unwrap().contains(&(56 - buf.as_bytes()[1] as i8, buf.as_bytes()[0] as i8 - 97)) {
            println!("not possible move");
            continue;
        }
        lm.move_piece((56 - buf.as_bytes()[1] as i8, buf.as_bytes()[0] as i8 - 97));
        println!("is check: {}\nis checkmate: {}", lm.is_check(turn), lm.is_checkmate(turn));
        turn = !turn;
    }
}
