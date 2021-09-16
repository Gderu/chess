pub mod logic;
pub mod gui;

use logic::piece::{Board};
use logic::LogicManager;
use gui::*;
use bevy_prototype_lyon::plugin::ShapePlugin;
use bevy::audio::AudioPlugin;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Chess".to_string(),
            width: SCREEN_LEN,
            height: SCREEN_LEN,
            ..Default::default()
        })
        .insert_resource(LogicManager::new())
        .insert_resource(Turn(true))
        .insert_resource(PromotePawnOption {
            happened: false,
            new_pos: (-1, -1),
        })
        .add_startup_system(setup.system())
        .add_stage_after(CoreStage::Update, StageLabels::MouseClicks, SystemStage::single_threaded())
        .add_stage_after(StageLabels::MouseClicks, StageLabels::MoveCalculation, SystemStage::single_threaded())
        .add_stage_after(StageLabels::MoveCalculation, StageLabels::PositionCalculation, SystemStage::single_threaded())
        .add_system_to_stage(StageLabels::MouseClicks, mouse_clicks.system())
        .add_system_to_stage(StageLabels::MoveCalculation, piece_options.system())
        .add_system_to_stage(StageLabels::MoveCalculation, move_piece.system())
        .add_system_to_stage(StageLabels::MoveCalculation, promote_pawn_choice.system())
        .add_system_to_stage(StageLabels::PositionCalculation, position_translation.system())
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(bevy_kira_audio::AudioPlugin)
        .add_event::<PieceOptionEvent>()
        .add_event::<MoveEvent>()
        .add_event::<PawnPromotionEvent>()
        .run();
}

fn cli_chess() {
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

fn print_board_ascii(board: &Board) {
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