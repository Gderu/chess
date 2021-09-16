use crate::logic::LogicManager;
pub use bevy::{prelude::*};
use bevy::input::mouse::MouseButtonInput;
use bevy_prototype_lyon::prelude::*;
pub use bevy_kira_audio::{Audio, AudioPlugin, AudioSource};
use std::ops::DerefMut;
use crate::logic::piece::PieceTypes;

pub const SCREEN_LEN: f32 = 650.;
const NUM_SQUARES: f32 = 8.;
const SQUARE_SIZE: f32 = SCREEN_LEN / NUM_SQUARES;
const INVALID_VALUE: usize = 100;

const HIDDEN_LAYER: usize = 0;
const TILES_LAYER: usize = 1;
const SELECTIONS_LAYER: usize = 2;
const HINTS_LAYER: usize = 3;
const PIECES_LAYER: usize = 4;
const PAWN_PROMOTION_BACKGROUND_LAYER: usize = 5;
const PAWN_PROMOTION_PIECES_LAYER: usize = 6;

#[derive(Copy, Clone)]
pub struct Position {
    x: usize,
    y: usize,
    z: usize,
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[derive(Clone)]
pub struct BackgroundColors {
    pub white: Handle<ColorMaterial>,
    pub light: Handle<ColorMaterial>,
    pub dark: Handle<ColorMaterial>,
    pub yellow: Handle<ColorMaterial>,
}

#[derive(Clone)]
pub struct MoveSounds {
    pub move_self: Handle<AudioSource>,
    pub capture: Handle<AudioSource>,
    pub checkmate: Handle<AudioSource>,
    pub check: Handle<AudioSource>,
}

pub struct Turn(pub bool);

pub struct PromotePawnOption {
    pub happened: bool,
    pub new_pos: (i8, i8),
}

pub struct Piece {
    piece_type: PieceTypes,
}

pub struct Hint;
pub struct Selected;
pub struct LastMove;
pub struct Hidden;
pub struct PromotePawn;


pub struct PieceOptionEvent(Vec<(i8, i8)>);
pub struct MoveEvent(((i8, i8), (i8, i8)));
pub struct PawnPromotionEvent((i8, i8));

#[derive(StageLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum StageLabels {
    MouseClicks,
    MoveCalculation,
    PositionCalculation,
}

pub fn setup(
    mut commands: Commands,
    mut server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let bc = BackgroundColors {
        white: materials.add(Color::rgb(1., 1., 1.).into()),
        light: materials.add(Color::rgb(0.71, 0.533, 0.388).into()),
        dark: materials.add(Color::rgb(0.941, 0.851, 0.71).into()),
        yellow: materials.add(Color::rgba(1., 1., 0., 0.4).into()),
    };
    let ms = MoveSounds {
        capture: server.load("capture.mp3"),
        move_self: server.load("move-self.mp3"),
        check: server.load("move-check.mp3"),
        checkmate: server.load("game-end.mp3"),
    };
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(bc.clone());
    commands.insert_resource(ms.clone());
    spawn_board(
        &mut commands,
        &bc,
    );
    spawn_pieces(
        &mut commands,
        &mut server,
        &mut materials,
    )
}

fn spawn_board(
    commands: &mut Commands,
    bc: &BackgroundColors,
) {
    for x in 0..8 {
        for y in 0..8 {
            if (x + y) % 2 == 0 {//white square
                commands
                    .spawn_bundle(SpriteBundle {
                        material: bc.light.clone(),
                        sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                        ..Default::default()
                    })
                    .insert(Position { x, y, z: TILES_LAYER });
                // thread::sleep(time::Duration::from_secs(1));
            } else { //black square
                commands
                    .spawn_bundle(SpriteBundle {
                        material: bc.dark.clone(),
                        sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                        ..Default::default()
                    })
                    .insert(Position { x, y, z: TILES_LAYER });
            }
        }
    }
}

fn spawn_pieces(
    commands: &mut Commands,
    server: &mut ResMut<AssetServer>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    for y in [0, 1, 6, 7] {
        let color = match y {
            0..=3 => "dark",
            _ => "light",
        };

        for x in 0..8 {
            if y == 0 || y == 7 {
                let (p, piece_type) = get_path(x, color);

                let piece = Piece {
                    piece_type,
                };
                commands
                    .spawn_bundle(SpriteBundle {
                        material: materials.add(server.load(&p[..]).into()),
                        sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                        ..Default::default()
                    })
                    .insert(Position {x, y, z: PIECES_LAYER})
                    .insert(piece);
            } else {
                let p = color.to_string() + "_pawn.png";

                let piece = Piece {
                    piece_type: PieceTypes::Pawn,
                };
                commands
                    .spawn_bundle(SpriteBundle {
                        material: materials.add(server.load(&p[..]).into()),
                        sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                        ..Default::default()
                    })
                    .insert(Position {x, y, z: PIECES_LAYER})
                    .insert(piece);
            }
        }
    }
}

fn get_path(x: usize, color: &str) -> (String, PieceTypes) {
    let color_bool = match color {
        "dark" => false,
        _ => true,
    };
    if x == 0 {
        (color.to_string() + "_rook.png", PieceTypes::Rook)
    } else if x == 1 {
        (color.to_string() + "_knight.png", PieceTypes::Knight)
    } else if x == 2 {
        (color.to_string() + "_bishop.png", PieceTypes::Bishop)
    } else if x == 3 {
        (color.to_string() + "_queen.png", PieceTypes::Queen)
    } else if x == 4 {
        (color.to_string() + "_king.png", PieceTypes::King)
    } else if x == 5 {
        (color.to_string() + "_bishop.png", PieceTypes::Bishop)
    } else if x == 6 {
        (color.to_string() + "_knight.png", PieceTypes::Knight)
    } else {
        (color.to_string() + "_rook.png", PieceTypes::Rook)
    }
}

pub fn position_translation(mut q: Query<(&Position, &mut Transform)>) {
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            pos.x as f32 / NUM_SQUARES * SCREEN_LEN - (SCREEN_LEN / 2.) + (SQUARE_SIZE / 2.),
            (SCREEN_LEN / 2.) - pos.y as f32 / NUM_SQUARES * SCREEN_LEN - (SQUARE_SIZE / 2.),
            pos.z as f32,
        );
    }
}

pub fn mouse_clicks (
    mut commands: Commands,
    mut evr_mousebtn: EventReader<MouseButtonInput>,
    windows: Res<Windows>,
    mut lm: ResMut<LogicManager>,
    mut move_writer: EventWriter<MoveEvent>,
    mut piece_option_writer: EventWriter<PieceOptionEvent>,
    mut pawn_promotion_writer: EventWriter<PawnPromotionEvent>,
    mut turn: Res<Turn>,
    query_hint: Query<(&Position, Entity), With<Hint>>,
    query_selected: Query<(&Position, Entity), With<Selected>>,
    mut promote_pawn_option: ResMut<PromotePawnOption>,
) {
    let window = windows.get_primary().unwrap();
    for ev in evr_mousebtn.iter() {
        if ev.state.is_pressed() {
            if let Some(position) = window.cursor_position() {
                let pos = get_sqr(position);
                let hint_positions = query_hint.iter().map(|(p, _e)| (p.y as i8, p.x as i8)).collect::<Vec<(i8, i8)>>();
                let selected_position = query_selected.iter().map(|(p, _e)| (p.y as i8, p.x as i8)).collect::<Vec<(i8, i8)>>();
                for (_p, e) in query_hint.iter() {
                    commands
                        .entity(e).despawn();
                }
                for (_p, e) in query_selected.iter() {
                    commands
                        .entity(e).despawn();
                }
                if promote_pawn_option.happened {
                    pawn_promotion_writer.send(PawnPromotionEvent(pos));
                } else if lm.can_move() && hint_positions.contains(&pos) {
                    move_writer.send(MoveEvent((*selected_position.iter().next().unwrap(), pos)));
                } else {
                    lm.clear_selection();
                    if !selected_position.contains(&pos) {
                        if let Some(piece_color) = lm.get_piece_color(pos) {
                            if piece_color == turn.0 {
                                if let Some(squares) = lm.get_possible_moves(pos) {
                                    let mut v = (*squares).clone();
                                    v.insert(0, pos);
                                    piece_option_writer.send(PieceOptionEvent(v));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_sqr(pos: Vec2) -> (i8, i8) {
    (7 - (pos.y * NUM_SQUARES / SCREEN_LEN).floor() as i8, (pos.x * NUM_SQUARES / SCREEN_LEN).floor() as i8)
}

pub fn piece_options(
    mut commands: Commands,
    mut piece_option_reader: EventReader<PieceOptionEvent>,
    background_colors: Res<BackgroundColors>,
    lm: Res<LogicManager>,
) {
    if let Some(piece_option_event) = piece_option_reader.iter().next() {
        let mut it = piece_option_event.0.iter();
        let pos = it.next().unwrap();
        commands
            .spawn_bundle(SpriteBundle {
                material: background_colors.yellow.clone(),
                sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                ..Default::default()
            })
            .insert(Position {x: pos.1 as usize, y: pos.0 as usize, z: SELECTIONS_LAYER})
            .insert(Selected);
        for hint_pos in it {
            if let Some(_color) = lm.get_piece_color(*hint_pos) {//if there is a piece at the hint location
                let circle = shapes::Circle {
                    radius: 30.,
                    ..Default::default()
                };
                commands
                    .spawn_bundle(GeometryBuilder::build_as(
                        &circle,
                        ShapeColors::outlined(Color::rgba(0., 0., 0., 0.), Color::rgba(0., 0., 0., 0.3)),
                        DrawMode::Outlined{
                            fill_options: FillOptions::default(),
                            outline_options: StrokeOptions::default().with_line_width(10.)},
                        Transform::default(),
                    ))
                    .insert(Position {x: hint_pos.1 as usize, y: hint_pos.0 as usize, z: HINTS_LAYER})
                    .insert(Hint);
            } else {//if the hint location is empty
                let circle = shapes::Circle {
                    radius: 15.,
                    ..Default::default()
                };
                commands
                    .spawn_bundle(GeometryBuilder::build_as(
                        &circle,
                        ShapeColors::new(Color::rgba(0., 0., 0., 0.3)),
                        DrawMode::Fill(FillOptions::default()),
                        Transform::default(),
                    ))
                    .insert(Position {x: hint_pos.1 as usize, y: hint_pos.0 as usize, z: HINTS_LAYER})
                    .insert(Hint);
            }
        }
    }
}

pub fn move_piece(
    mut commands: Commands,
    mut move_reader: EventReader<MoveEvent>,
    mut query_pieces: Query<(&mut Position, Entity, &Piece)>,
    mut query_last_move: Query<Entity, With<LastMove>>,
    mut lm: ResMut<LogicManager>,
    mut turn: ResMut<Turn>,
    audio: Res<Audio>,
    move_sounds: Res<MoveSounds>,
    bc: Res<BackgroundColors>,
    server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut promote_pawn_option: ResMut<PromotePawnOption>,
) {
    if let Some(move_event) = move_reader.iter().next() {
        let (prev_pos, new_pos) = move_event.0;
        let mut to_play = move_sounds.move_self.clone();

        for (mut pos, e, piece) in query_pieces.iter_mut() {
            if (pos.y as i8, pos.x as i8) == prev_pos && piece.piece_type == PieceTypes::Pawn && [0, 7].contains(&new_pos.0) {
                promote_pawn_show_options(&mut commands, turn.0, new_pos, &bc, &server, &mut materials);
                promote_pawn_option.happened = true;
                promote_pawn_option.new_pos = new_pos;
                commands.entity(e).insert(Hidden);
                pos.z = HIDDEN_LAYER;
                return;
            }
            if (pos.y as i8, pos.x as i8) == new_pos {
                commands.entity(e).despawn();
                to_play = move_sounds.capture.clone();
            }
        }

        let res = lm.move_piece(new_pos);
        for (mut pos, e, _piece) in query_pieces.iter_mut() {
            if (pos.y as i8, pos.x as i8) == new_pos {
                commands.entity(e).despawn();
                to_play = move_sounds.capture.clone();
            }
            if let Some((other_prev_pos, other_new_pos)) = res {
                if (pos.y as i8, pos.x as i8) == other_prev_pos {
                    if other_new_pos == (-1, -1) {
                        commands.entity(e).despawn();
                        to_play = move_sounds.capture.clone();
                    } else {
                        pos.x = other_new_pos.1 as usize;
                        pos.y = other_new_pos.0 as usize;
                    }
                }
            }
        }

        for (mut pos, _e, _piece) in query_pieces.iter_mut() {
            if (pos.y as i8, pos.x as i8) == prev_pos {
                pos.x = new_pos.1 as usize;
                pos.y = new_pos.0 as usize;
            }
        }
        if lm.is_check(turn.0) {
            to_play = move_sounds.check.clone();
        }
        if lm.is_checkmate(turn.0) {
            to_play = move_sounds.checkmate.clone();
        }
        audio.play(to_play);
        turn.0 = !turn.0;

        for e in query_last_move.iter() {
            commands.entity(e).despawn();
        }
        commands
            .spawn_bundle(SpriteBundle {
                material: bc.yellow.clone(),
                sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                ..Default::default()
            })
            .insert(Position {x: new_pos.1 as usize, y: new_pos.0 as usize, z: SELECTIONS_LAYER})
            .insert(LastMove);

        commands
            .spawn_bundle(SpriteBundle {
                material: bc.yellow.clone(),
                sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                ..Default::default()
            })
            .insert(Position {x: prev_pos.1 as usize, y: prev_pos.0 as usize, z: SELECTIONS_LAYER})
            .insert(LastMove);

    }
}

fn promote_pawn_show_options(
    commands: &mut Commands,
    turn: bool,
    new_pos: (i8, i8),
    bc: &BackgroundColors,
    server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let range = match turn {
        false => (4..8).rev().collect::<Vec<_>>(),
        true => (0..4).collect::<Vec<_>>(),
    };
    let piece_nums = [3, 1, 0, 2];
    for (y, piece_type) in range.iter().zip(piece_nums) {
        let (p, _piece_type) = get_path(piece_type, match turn {
            true => "light",
            false => "dark",
        });

        commands
            .spawn_bundle(SpriteBundle {
                material: bc.white.clone(),
                sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                ..Default::default()
            })
            .insert(Position {x: new_pos.1 as usize, y: (*y) as usize, z: PAWN_PROMOTION_BACKGROUND_LAYER})
            .insert(PromotePawn);


        commands
            .spawn_bundle(SpriteBundle {
                material: materials.add(server.load(&p[..]).into()),
                sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                ..Default::default()
            })
            .insert(Position {x: new_pos.1 as usize, y: (*y) as usize, z: PAWN_PROMOTION_PIECES_LAYER})
            .insert(PromotePawn);
    }
}

pub fn promote_pawn_choice(
    mut commands: Commands,
    mut lm: ResMut<LogicManager>,
    mut pawn_promotion_reader: EventReader<PawnPromotionEvent>,
    query_pawn_promotion: Query<Entity, With<PromotePawn>>,
    query_last_move: Query<Entity, With<LastMove>>,
    mut query_hidden: Query<(&mut Position, Entity), With<Hidden>>,
    mut query_pieces: Query<(&Position, Entity), (With<Piece>, Without<Hidden>)>,
    mut turn: ResMut<Turn>,
    mut promote_pawn_option: ResMut<PromotePawnOption>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut server: ResMut<AssetServer>,
    bc: Res<BackgroundColors>,
) {
    if let Some(pawn_promotion_event) = pawn_promotion_reader.iter().next() {
        let pos_clicked = pawn_promotion_event.0;
        let (mut p_hidden, e_hidden) = query_hidden.iter_mut().next().unwrap();
        let range = match turn.0 {
            false => (4..8).rev().collect::<Vec<_>>(),
            true => (0..4).collect::<Vec<_>>(),
        };
        let new_pos = promote_pawn_option.new_pos;
        let piece_types = vec![3, 1, 0,2];
        let mut promoted = false;
        if new_pos.1 == pos_clicked.1 {
            if let Some(index) = range.iter().position(|r| *r == pos_clicked.0) {
                println!("{}", index);
                promoted = true;
                let piece_type_usize: usize = *piece_types.get(index).unwrap();
                let piece_type = match piece_type_usize {
                    3 => PieceTypes::Queen,
                    2 => PieceTypes::Bishop,
                    1 => PieceTypes::Knight,
                    0 => PieceTypes::Rook,
                    _ => panic!("If this ever happens, something went very wrong"),
                };
                lm.promote_pawn(new_pos, piece_type);
                commands.entity(e_hidden).despawn();
                let (p, _piece_type) = get_path(piece_type_usize, match turn.0 {
                    true => "light",
                    false => "dark",
                });
                commands
                    .spawn_bundle(SpriteBundle {
                        material: materials.add(server.load(&p[..]).into()),
                        sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                        ..Default::default()
                    })
                    .insert(Position {x: new_pos.1 as usize, y: new_pos.0 as usize, z: PIECES_LAYER})
                    .insert(Piece {
                        piece_type,
                    });
                turn.0 = !turn.0;

                for e in query_last_move.iter() {
                    commands.entity(e).despawn();
                }

                for (pos, e) in query_pieces.iter() {
                    if pos.x == new_pos.1 as usize && pos.y == new_pos.0 as usize {
                        commands.entity(e).despawn();
                    }
                }
                commands
                    .spawn_bundle(SpriteBundle {
                        material: bc.yellow.clone(),
                        sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                        ..Default::default()
                    })
                    .insert(Position {x: new_pos.1 as usize, y: new_pos.0 as usize, z: SELECTIONS_LAYER})
                    .insert(LastMove);

                commands
                    .spawn_bundle(SpriteBundle {
                        material: bc.yellow.clone(),
                        sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                        ..Default::default()
                    })
                    .insert(Position {x: p_hidden.x as usize, y: p_hidden.y as usize, z: SELECTIONS_LAYER})
                    .insert(LastMove);
            }
        }
        if promoted == false {
            p_hidden.z = PIECES_LAYER;
            commands.entity(e_hidden).remove::<Hidden>();
            lm.clear_selection();
        }
        for e in query_pawn_promotion.iter() {
            commands.entity(e).despawn();
        }
        promote_pawn_option.happened = false;
    }
}