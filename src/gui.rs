use crate::logic::LogicManager;
pub use bevy::{prelude::*};
use bevy::input::mouse::MouseButtonInput;
use bevy_prototype_lyon::prelude::*;
pub use bevy_kira_audio::{Audio, AudioPlugin, AudioSource};
use std::ops::DerefMut;

pub const SCREEN_LEN: f32 = 650.;
const NUM_SQUARES: f32 = 8.;
const SQUARE_SIZE: f32 = SCREEN_LEN / NUM_SQUARES;
const INVALID_VALUE: usize = 100;


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
    pub black: Handle<ColorMaterial>,
    pub yellow: Handle<ColorMaterial>,
}

#[derive(Clone)]
pub struct MoveSounds {
    pub move_self: Handle<AudioSource>,
    pub capture: Handle<AudioSource>,
    pub checkmate: Handle<AudioSource>,
    pub check: Handle<AudioSource>,
}

pub enum Pieces {
    King(bool),
    Queen(bool),
    Bishop(bool),
    Knight(bool),
    Rook(bool),
    Pawn(bool),
}

pub struct Turn(pub bool);

pub struct Piece {
    piece_type: Pieces,
    material: Handle<ColorMaterial>,
}

pub struct Hint;
pub struct Selected;
pub struct OldMove;

pub struct PieceOption(Vec<(i8, i8)>);
pub struct Move(((i8, i8), (i8, i8)));

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
        white: materials.add(Color::rgb(0.71, 0.533, 0.388).into()),
        black: materials.add(Color::rgb(0.941, 0.851, 0.71).into()),
        yellow: materials.add(Color::rgba(1., 1., 0., 0.55).into()),
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
                        material: bc.white.clone(),
                        sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                        ..Default::default()
                    })
                    .insert(Position { x, y, z: 0 });
                // thread::sleep(time::Duration::from_secs(1));
            } else { //black square
                commands
                    .spawn_bundle(SpriteBundle {
                        material: bc.black.clone(),
                        sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                        ..Default::default()
                    })
                    .insert(Position { x, y, z: 0 });
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
                    material: materials.add(server.load(&p[..]).into()),
                };
                commands
                    .spawn_bundle(SpriteBundle {
                        material: piece.material.clone(),
                        sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                        ..Default::default()
                    })
                    .insert(Position {x, y, z: 2})
                    .insert(piece);
            } else {
                let p = "..\\assets\\".to_string() + color + "_pawn.png";

                let piece = Piece {
                    piece_type: Pieces::King(true),
                    material: materials.add(server.load(&p[..]).into()),
                };
                commands
                    .spawn_bundle(SpriteBundle {
                        material: piece.material.clone(),
                        sprite: Sprite::new(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                        ..Default::default()
                    })
                    .insert(Position {x, y, z: 2})
                    .insert(piece);
            }
        }
    }
}

fn get_path(x: usize, color: &str) -> (String, Pieces) {
    let color_bool = match color {
        "dark" => false,
        _ => true,
    };
    if x == 0 {
        ("..\\assets\\".to_string() + color + "_rook.png", Pieces::Rook(color_bool))
    } else if x == 1 {
        ("..\\assets\\".to_string() + color + "_knight.png", Pieces::Knight(color_bool))
    } else if x == 2 {
        ("..\\assets\\".to_string() + color + "_bishop.png", Pieces::Bishop(color_bool))
    } else if x == 3 {
        ("..\\assets\\".to_string() + color + "_queen.png", Pieces::Queen(color_bool))
    } else if x == 4 {
        ("..\\assets\\".to_string() + color + "_king.png", Pieces::King(color_bool))
    } else if x == 5 {
        ("..\\assets\\".to_string() + color + "_bishop.png", Pieces::Bishop(color_bool))
    } else if x == 6 {
        ("..\\assets\\".to_string() + color + "_knight.png", Pieces::Knight(color_bool))
    } else {
        ("..\\assets\\".to_string() + color + "_rook.png", Pieces::Rook(color_bool))
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
    mut move_writer: EventWriter<Move>,
    mut piece_option_writer: EventWriter<PieceOption>,
    mut turn: Res<Turn>,
    query_hint: Query<(&Position, Entity), With<Hint>>,
    query_selected: Query<(&Position, Entity), With<Selected>>,
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
                if lm.can_move() && hint_positions.contains(&pos) {
                    move_writer.send(Move((*selected_position.iter().next().unwrap(), pos)));
                } else {
                    lm.clear_selection();
                    if !selected_position.contains(&pos) {
                        if let Some(piece_color) = lm.get_piece_color(pos) {
                            if piece_color == turn.0 {
                                if let Some(squares) = lm.get_possible_moves(pos) {
                                    let mut v = (*squares).clone();
                                    v.insert(0, pos);
                                    piece_option_writer.send(PieceOption(v));
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
    mut piece_option_reader: EventReader<PieceOption>,
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
            .insert(Position {x: pos.1 as usize, y: pos.0 as usize, z: 1})
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
                    .insert(Position {x: hint_pos.1 as usize, y: hint_pos.0 as usize, z: 1})
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
                    .insert(Position {x: hint_pos.1 as usize, y: hint_pos.0 as usize, z: 1})
                    .insert(Hint);
            }
        }
    }
}

pub fn move_piece(
    mut commands: Commands,
    mut move_reader: EventReader<Move>,
    mut query_pieces: Query<(&mut Position, Entity), With<Piece>>,
    mut lm: ResMut<LogicManager>,
    mut turn: ResMut<Turn>,
    audio: Res<Audio>,
    move_sounds: Res<MoveSounds>,
    asset_server: Res<AssetServer>,

) {
    if let Some(move_event) = move_reader.iter().next() {
        turn.0 = !turn.0;

        let (prev_pos, new_pos) = move_event.0;
        let res = lm.move_piece(new_pos);
        let mut to_play = move_sounds.move_self.clone();


        for (mut pos, e) in query_pieces.iter_mut() {
            if (pos.y as i8, pos.x as i8) == prev_pos {
                pos.x = new_pos.1 as usize;
                pos.y = new_pos.0 as usize;

            } else if (pos.y as i8, pos.x as i8) == new_pos {
                commands.entity(e).despawn();
                to_play = move_sounds.capture.clone();

            } else if let Some((other_prev_pos, other_new_pos)) = res {
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
        if lm.is_check(!turn.0) {
            to_play = move_sounds.check.clone();
        }
        if lm.is_checkmate(!turn.0) {
            to_play = move_sounds.checkmate.clone();
        }
        audio.play(to_play);
    }
}