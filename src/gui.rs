use crate::logic::LogicManager;
pub use bevy::{prelude::*};
use bevy::input::mouse::MouseButtonInput;
use bevy_prototype_lyon::prelude::*;
pub use bevy_kira_audio::{Audio, AudioPlugin, AudioSource};
use crate::logic::piece::PieceTypes;

pub use bevy::{
    prelude::*,
    render::{
        camera::{ActiveCameras, Camera},
        pass::*,
        render_graph::{
            base::MainPass, CameraNode, PassNode, RenderGraph, WindowSwapChainNode,
            WindowTextureNode,
        },
        texture::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage},
    },
    window::{CreateWindow, WindowDescriptor, WindowId},
};

pub const SCREEN_HEIGHT: f32 = 650.;
pub const SCREEN_WIDTH: f32 = 800.;
const NUM_SQUARES: f32 = 8.;
const SQUARE_SIZE: f32 = SCREEN_HEIGHT / NUM_SQUARES;

const HIDDEN_LAYER: usize = 0;
const TILES_LAYER: usize = 1;
const SELECTIONS_LAYER: usize = 2;
const HINTS_LAYER: usize = 3;
const PIECES_LAYER: usize = 4;
const PAWN_PROMOTION_BACKGROUND_LAYER: usize = 5;
const PAWN_PROMOTION_PIECES_LAYER: usize = 6;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    CreateWindow,
    Setup,
    Done,
}

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
    pub game_end: Handle<AudioSource>,
    pub check: Handle<AudioSource>,
}

pub struct Turn(pub bool);
pub struct Moved(pub bool);
pub struct Capture(pub bool);

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
    AfterTurnUpdates,
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
        game_end: server.load("game-end.mp3"),
    };
    commands.spawn_bundle(UiCameraBundle::default());
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

pub fn position_translation(
    mut q: Query<(&Position, &mut Transform)>,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            pos.x as f32 / NUM_SQUARES * window.height() - (window.width() / 2.) + (SQUARE_SIZE / 2.),
            (window.height() / 2.) - pos.y as f32 / NUM_SQUARES * window.height() - (SQUARE_SIZE / 2.),
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
    turn: Res<Turn>,
    query_hint: Query<(&Position, Entity), With<Hint>>,
    query_selected: Query<(&Position, Entity), With<Selected>>,
    promote_pawn_option: Res<PromotePawnOption>,
) {
    let window = windows.get_primary().unwrap();
    for ev in evr_mousebtn.iter() {
        if ev.state.is_pressed() {
            if let Some(position) = window.cursor_position() {
                let pos = get_sqr(position);
                if pos.0 > 7 || pos.0 < 0 || pos.1 > 7 || pos.0 < 0 {
                    return;
                }
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
    (7 - (pos.y * NUM_SQUARES / SCREEN_HEIGHT).floor() as i8, (pos.x * NUM_SQUARES / SCREEN_HEIGHT).floor() as i8)
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
    query_last_move: Query<Entity, With<LastMove>>,
    mut lm: ResMut<LogicManager>,
    turn: Res<Turn>,
    bc: Res<BackgroundColors>,
    server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut promote_pawn_option: ResMut<PromotePawnOption>,
    mut capture: ResMut<Capture>,
    mut moved: ResMut<Moved>,
) {
    if let Some(move_event) = move_reader.iter().next() {
        let (prev_pos, new_pos) = move_event.0;

        for (mut pos, e, piece) in query_pieces.iter_mut() {
            if (pos.y as i8, pos.x as i8) == prev_pos && piece.piece_type == PieceTypes::Pawn && [0, 7].contains(&new_pos.0) {
                promote_pawn_show_options(&mut commands, turn.0, new_pos, &bc, &server, &mut materials);
                promote_pawn_option.happened = true;
                promote_pawn_option.new_pos = new_pos;
                commands.entity(e).insert(Hidden);
                pos.z = HIDDEN_LAYER;
                return;
            }
        }

        let res = lm.move_piece(new_pos);
        moved.0 = true;
        for (mut pos, e, _piece) in query_pieces.iter_mut() {
            if (pos.y as i8, pos.x as i8) == new_pos {
                commands.entity(e).despawn();
                capture.0 = true;
            }
            if let Some((other_prev_pos, other_new_pos)) = res {
                if (pos.y as i8, pos.x as i8) == other_prev_pos {
                    if other_new_pos == (-1, -1) {
                        commands.entity(e).despawn();
                        capture.0 = true;
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
    query_pieces: Query<(&Position, Entity), (With<Piece>, Without<Hidden>)>,
    turn: Res<Turn>,
    mut promote_pawn_option: ResMut<PromotePawnOption>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    server: Res<AssetServer>,
    bc: Res<BackgroundColors>,
    mut capture: ResMut<Capture>,
    mut moved: ResMut<Moved>,
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
                moved.0 = true;
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

                for e in query_last_move.iter() {
                    commands.entity(e).despawn();
                }

                for (pos, e) in query_pieces.iter() {
                    if pos.x == new_pos.1 as usize && pos.y == new_pos.0 as usize {
                        commands.entity(e).despawn();
                        capture.0 = true;
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

pub fn after_turn_updates(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut turn: ResMut<Turn>,
    move_sounds: Res<MoveSounds>,
    mut lm: ResMut<LogicManager>,
    mut capture: ResMut<Capture>,
    mut moved: ResMut<Moved>,
    audio: Res<Audio>,
    mut windows: ResMut<Windows>,
){
    if moved.0 {
        let mut to_play = match capture.0 {
            true => move_sounds.capture.clone(),
            false => move_sounds.move_self.clone(),
        };
        let is_draw = lm.is_draw();
        if lm.is_check(turn.0) {
            to_play = move_sounds.check.clone();
        }
        if lm.is_checkmate(turn.0) || is_draw {
            lm.stop();
            let window = windows.get_primary_mut().unwrap();
            window.set_resolution(SCREEN_WIDTH, SCREEN_HEIGHT);
            to_play = move_sounds.game_end.clone();
            let to_display;
            if is_draw {
                to_display = "Draw";
            } else if turn.0 == true {
                to_display = "White\nwins";
            } else {
                to_display = "Black\nwins";
            }
            commands
                .spawn_bundle(TextBundle {
                    style: Style {
                        align_self: AlignSelf::FlexEnd,
                        position_type: PositionType::Absolute,
                        position: Rect {
                            bottom: Val::Px(SCREEN_HEIGHT / 2. - 50.),
                            right: Val::Px(25.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    // Use the `Text::with_section` constructor
                    text: Text::with_section(
                        // Accepts a `String` or any type that converts into a `String`, such as `&str`
                        to_display,
                        TextStyle {
                            font: asset_server.load("FiraSans-Bold.ttf"),
                            font_size: 50.0,
                            color: Color::WHITE,
                        },
                        // Note: You can use `Default::default()` in place of the `TextAlignment`
                        TextAlignment {
                            horizontal: HorizontalAlign::Right,
                            ..Default::default()
                        },
                    ),
                    ..Default::default()
                });
        }
        audio.play(to_play);
        turn.0 = !turn.0;
        capture.0 = false;
        moved.0 = false;
    }
}

pub fn create_result_window(
    mut create_window_events: EventWriter<CreateWindow>,
    mut app_state: ResMut<State<AppState>>,
) {
    let window_id = WindowId::new();
    // sends out a "CreateWindow" event, which will be received by the windowing backend
    create_window_events.send(CreateWindow {
        id: window_id,
        descriptor: WindowDescriptor {
            width: 1000.,
            height: 1000.,
            vsync: false,
            title: "second window".to_string(),
            ..Default::default()
        },
    });
    app_state.set(AppState::Setup).unwrap();
}

pub fn setup_result_window(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut active_cameras: ResMut<ActiveCameras>,
    mut render_graph: ResMut<RenderGraph>,
    msaa: Res<Msaa>,
    windows: Res<Windows>,
    mut app_state: ResMut<State<AppState>>,
) {

    // let to_display = match draw {
    //     true => "Draw",
    //     false => match turn {
    //         true => "White wins",
    //         false => "Black wins",
    //     }
    // };
    let to_display = "Its working!";
    // get the non-default window id
    // get the non-default window id
    let window_id_old = windows
        .iter()
        .find(|w| w.id() != WindowId::default())
        .map(|w| w.id());
    let window_id;
    window_id = match window_id_old {
        Some(window_id_old) => window_id_old,
        None => return,
    };


    // here we setup our render graph to draw our second camera to the new window's swap chain

    // add a swapchain node for our new window1
    render_graph.add_node(
        "second_window_swap_chain",
        WindowSwapChainNode::new(window_id),
    );

    // add a new depth texture node for our new window
    render_graph.add_node(
        "second_window_depth_texture",
        WindowTextureNode::new(
            window_id,
            TextureDescriptor {
                format: TextureFormat::Depth32Float,
                usage: TextureUsage::OUTPUT_ATTACHMENT,
                sample_count: msaa.samples,
                ..Default::default()
            },
        ),
    );

    // add a new camera node for our new window
    render_graph.add_system_node("secondary_camera", CameraNode::new("Secondary"));

    // add a new render pass for our new window / camera
    let mut second_window_pass = PassNode::<&MainPass>::new(PassDescriptor {
        color_attachments: vec![msaa.color_attachment_descriptor(
            TextureAttachment::Input("color_attachment".to_string()),
            TextureAttachment::Input("color_resolve_target".to_string()),
            Operations {
                load: LoadOp::Clear(Color::rgb(1., 1., 1.)),
                store: true,
            },
        )],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
            attachment: TextureAttachment::Input("depth".to_string()),
            depth_ops: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        }),
        sample_count: msaa.samples,
    });

    second_window_pass.add_camera("Secondary");
    active_cameras.add("Secondary");

    render_graph.add_node("second_window_pass", second_window_pass);

    render_graph
        .add_slot_edge(
            "second_window_swap_chain",
            WindowSwapChainNode::OUT_TEXTURE,
            "second_window_pass",
            if msaa.samples > 1 {
                "color_resolve_target"
            } else {
                "color_attachment"
            },
        )
        .unwrap();

    render_graph
        .add_slot_edge(
            "second_window_depth_texture",
            WindowTextureNode::OUT_TEXTURE,
            "second_window_pass",
            "depth",
        )
        .unwrap();

    render_graph
        .add_node_edge("secondary_camera", "second_window_pass")
        .unwrap();

    if msaa.samples > 1 {
        render_graph.add_node(
            "second_multi_sampled_color_attachment",
            WindowTextureNode::new(
                window_id,
                TextureDescriptor::default(),
            ),
        );

        render_graph
            .add_slot_edge(
                "second_multi_sampled_color_attachment",
                WindowSwapChainNode::OUT_TEXTURE,
                "second_window_pass",
                "color_attachment",
            )
            .unwrap();
    }
    commands.spawn_bundle(UiCameraBundle {
        camera: Camera {
            name: Some("Secondary".to_string()),
            window: window_id,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 1000. - 0.1),
        ..Default::default()
    });
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            // Use the `Text::with_section` constructor
            text: Text::with_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "hello\nbevy!",
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 100.0,
                    color: Color::BLACK,
                },
                // Note: You can use `Default::default()` in place of the `TextAlignment`
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    ..Default::default()
                },
            ),
            ..Default::default()
        });
    app_state.set(AppState::Done).unwrap();
}