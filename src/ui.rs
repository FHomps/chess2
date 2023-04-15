use bevy::{prelude::*, window::WindowResized, transform::TransformSystem};
use crate::sets::*;
use crate::board::*;
use crate::logic::*;

const BG_TEX_SIZE: Vec2 = Vec2::new(2560., 1587.);
const PIECE_TEX_SIZE: f32 = 256.;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_ui.in_set(GameSet::UISetup))
            .add_system(update_transform_cache.in_base_set(CoreSet::PostUpdate).after(TransformSystem::TransformPropagate))
            .add_system(select_piece)
            .add_system(handle_window_resize);
    }
}

#[derive(Component)]
struct Background;

#[derive(Component)]
struct PlayArea;

// Used to compute where the cursor is relative to a transformed entity
#[derive(Component)]
struct InverseGTransformCache {
    matrix: Mat4
}

impl Default for InverseGTransformCache {
    fn default() -> Self {
        Self { matrix: Mat4::IDENTITY }
    }
}

#[derive(Component)]
struct Square;

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct Marker;

#[derive(Resource)]
struct TextureHandles {
    marker: Handle<Image>
}



fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    board: Res<Board>,
) {
    let (bw, bh) = board.spaces.dim();
    let (bw, bh) = (bw as f32, bh as f32);

    commands.spawn(Camera2dBundle::default());
    
    commands.spawn((
        Background,
        SpriteBundle {
            texture: asset_server.load("bg.png"),
            ..default()
        }
    ));

    let pieces_atlas = atlases.add(TextureAtlas::from_grid(
        asset_server.load("pieces.png"), Vec2::splat(PIECE_TEX_SIZE), 6, 2, None, None
    ));

    commands.insert_resource(
        TextureHandles {
            marker: asset_server.load("marker.png")
        }
    );

    commands.spawn((
        PlayArea,
        TransformBundle::default(),
        InverseGTransformCache::default(),
        VisibilityBundle::default()
    )).with_children(|parent| {
        for ((x, y), space) in board.spaces.indexed_iter() {
            if *space != Space::Hole {
                parent.spawn(
                    SpriteBundle {
                        sprite: Sprite {
                            color: if (x + y) % 2 == 0 { Color::rgb(0.2, 0.3, 0.4) }
                                   else { Color::rgb(0.8, 0.8, 0.8) },
                            custom_size: Some(Vec2::ONE),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::new(
                            x as f32 - (bw - 1.) / 2.,
                            y as f32 - (bh - 1.) / 2.,
                            1.
                        )),
                        ..default()
                    }
                );
            }

            if let Space::Square { slot: Some(piece), .. } = space {
                parent.spawn((
                    *piece,
                    Coords { x: x as isize, y: y as isize },
                    SpriteSheetBundle {
                        texture_atlas: pieces_atlas.clone(),
                        sprite: TextureAtlasSprite {
                            index: piece.texture_index(),
                            custom_size: Some(Vec2::ONE),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::new(
                            x as f32 - (bw - 1.) / 2.,
                            y as f32 - (bh - 1.) / 2.,
                            2.
                        )),
                        ..default()
                    }
                ));
            }
        }
    });
}

fn select_piece(
    mut commands: Commands,
    windows: Query<&Window>,
    buttons: Res<Input<MouseButton>>,
    board: Res<Board>,
    play_area: Query<(Entity, &InverseGTransformCache), With<PlayArea>>,
    mut pieces: Query<(Entity, &mut Transform, &Coords, Option<&Selected>), With<Piece>>,
    markers: Query<Entity, With<Marker>>,
    possible_moves: Res<PossibleMoves>,
    texture_handles: Res<TextureHandles>
) {
    let (bw, bh) = board.spaces.dim();
    let (bw, bh) = (bw as f32, bh as f32);
    let Ok(window) = windows.get_single() else { eprintln!("select_piece: Could not fetch window"); return };
    let Some(mut pos) = window.cursor_position() else { return };
    pos.x -= window.width() / 2.;
    pos.y -= window.height() / 2.;

    let Ok((pa_entity, InverseGTransformCache { matrix })) = play_area.get_single() else { return };
    let local_pos = matrix.transform_point3(pos.extend(0.));

    if buttons.just_pressed(MouseButton::Left) {
        let local_coords = Coords {
            x: (local_pos.x + bw / 2.).floor() as isize,
            y: (local_pos.y + bh / 2.).floor() as isize
        };

        for (entity, mut transform, coords, maybe_selected) in pieces.iter_mut() {
            if maybe_selected.is_some() {
                commands.entity(entity).remove::<Selected>();
                transform.translation = Vec3::new(
                    coords.x as f32 - (bw - 1.) / 2.,
                    coords.y as f32 - (bh - 1.) / 2.,
                    2.
                );
                transform.scale = Vec3::ONE;

                for marker in markers.iter() {
                    commands.entity(marker).despawn();
                }
            }
            else if local_coords == *coords {
                commands.entity(entity).insert(Selected);
                transform.translation = local_pos.truncate().extend(3.);
                transform.scale = Vec2::splat(1.2).extend(1.);
                
                if let Some(moves) = possible_moves.0.get(coords) {
                    commands.entity(pa_entity).with_children(|parent| {
                        for move_ in moves {
                            parent.spawn((
                                Marker,
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::rgb(0.2, 0.6, 0.3),
                                        custom_size: Some(Vec2::ONE),
                                        ..default()
                                    },
                                    transform: Transform::from_translation(Vec3::new(
                                        move_.target.x as f32 - (bw - 1.) / 2.,
                                        move_.target.y as f32 - (bh - 1.) / 2.,
                                        1.
                                    )),
                                    texture: texture_handles.marker.clone(),
                                    ..default()
                                }
                            ));
                        }
                    });
                }
            }
        }
    }
    else {
        for (_, mut transform, _, maybe_selected) in pieces.iter_mut() {
            if maybe_selected.is_some() {
                transform.translation = local_pos.truncate().extend(3.);
            }
        }
    }   
}

// This system should be run in PostUpdate after transform propagation
fn update_transform_cache(
    mut query: Query<(&mut InverseGTransformCache, &GlobalTransform), Changed<GlobalTransform>>
) {
    let Ok((mut cache, transform)) = query.get_single_mut() else { return };
    cache.matrix = transform.compute_matrix().inverse();
}

fn handle_window_resize(
    mut events: EventReader<WindowResized>,
    mut set: ParamSet<(
        Query<&mut Transform, With<Background>>,
        Query<&mut Transform, With<PlayArea>>
    )>,
    board: Res<Board>
) {
    for event in events.iter() {
        let (ww, wh) = (event.width as f32, event.height as f32);
        let (bw, bh) = board.spaces.dim();
        let (bw, bh) = (bw as f32, bh as f32);

        // Resize the background so that it always fully covers the window
        if let Ok(mut transform) = set.p0().get_single_mut() {
            transform.scale = Vec2::splat(f32::max(ww / BG_TEX_SIZE.x, wh / BG_TEX_SIZE.y)).extend(1.);
        }

        // Resize the play area so that it is always fully visible
        if let Ok(mut transform) = set.p1().get_single_mut() {
            transform.scale = Vec2::splat(f32::min(ww / (bw + 2.), wh / (bh + 2.))).extend(1.);
        }
    }
}
