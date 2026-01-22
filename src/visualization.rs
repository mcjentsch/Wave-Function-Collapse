use bevy::prelude::*;
use wfc::{Coord, Map, TileType, VisualEvent, WFCState};

pub struct WFCPlugin;

impl Plugin for WFCPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(Update, step);
    }
}

#[derive(Resource)]
struct WFCVisual {
    state: WFCState,
    timer: Timer,
    done: bool,
}

fn tile_path(tile_type: TileType) -> &'static str {
    match tile_type {
        // Water types
        TileType::DeepWater => "tiles/tile_deep_water.png",
        TileType::ShallowWater => "tiles/tile_shallow_water.png",
        TileType::River => "tiles/tile_river.png",

        // Land types
        TileType::Beach => "tiles/tile_beach.png",
        TileType::Grass => "tiles/tile_grass.png",
        TileType::Forest => "tiles/tile_forest.png",
        TileType::Mountain => "tiles/tile_mountain.png",
        TileType::Snow => "tiles/tile_snow.png",
        TileType::Desert => "tiles/tile_desert.png",

        // Beach-Water transitions (edges)
        TileType::BeachWaterN => "tiles/tile_beach_water_n.png",
        TileType::BeachWaterE => "tiles/tile_beach_water_e.png",
        TileType::BeachWaterS => "tiles/tile_beach_water_s.png",
        TileType::BeachWaterW => "tiles/tile_beach_water_w.png",

        // Beach-Water transitions (corners)
        TileType::BeachWaterNe => "tiles/tile_beach_water_ne.png",
        TileType::BeachWaterNw => "tiles/tile_beach_water_nw.png",
        TileType::BeachWaterSe => "tiles/tile_beach_water_se.png",
        TileType::BeachWaterSw => "tiles/tile_beach_water_sw.png",

        // Grass-Forest transitions
        TileType::GrassForestN => "tiles/tile_grass_forest_n.png",
        TileType::GrassForestE => "tiles/tile_grass_forest_e.png",
        TileType::GrassForestS => "tiles/tile_grass_forest_s.png",
        TileType::GrassForestW => "tiles/tile_grass_forest_w.png",

        // Mountain-Snow transitions
        TileType::MountainSnowN => "tiles/tile_mountain_snow_n.png",
        TileType::MountainSnowE => "tiles/tile_mountain_snow_e.png",
        TileType::MountainSnowS => "tiles/tile_mountain_snow_s.png",
        TileType::MountainSnowW => "tiles/tile_mountain_snow_w.png",
    }
}

fn step(
    mut visual: ResMut<WFCVisual>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(&Coord, &mut Sprite)>,
) {
    visual.timer.tick(time.delta());

    if visual.timer.just_finished() && !visual.done {
        let visual_event = visual.state.next();
        if visual_event.is_none() {
            visual.done = true;
        }
        if let Some(event) = visual_event {
            let (coord, tile_type) = match event {
                VisualEvent::SetTile {
                    tile_type, coord, ..
                } => (coord, Some(tile_type)),
                VisualEvent::UndoTile { coord } => (coord, None),
            };

            for (query_coord, mut sprite) in &mut query {
                if query_coord.row == coord.row && query_coord.col == coord.col {
                    if let Some(tile_type) = tile_type {
                        sprite.image = asset_server.load(tile_path(tile_type));
                        sprite.color = Color::WHITE;
                    } else {
                        sprite.image = Handle::default();
                        sprite.color = Color::srgb(0.1, 0.1, 0.1);
                    }
                    break;
                }
            }
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let grid_width = 20;
    let grid_height = 20;

    let map_data = match Map::new(grid_width, grid_height) {
        Ok(map) => map,
        Err(e) => {
            eprintln!("Failed to create map: {}", e);
            return;
        }
    };

    commands.insert_resource(WFCVisual {
        state: WFCState::new(map_data),
        timer: Timer::from_seconds(0.00001, TimerMode::Repeating),
        done: false,
    });

    let cell_size = 30.0;
    let offset_x = -(grid_width as f32 * cell_size) / 2.0 + cell_size / 2.0;
    let offset_y = -(grid_height as f32 * cell_size) / 2.0 + cell_size / 2.0;

    for y in 0..grid_height {
        for x in 0..grid_width {
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.1, 0.1, 0.1),
                    custom_size: Some(Vec2::new(28.0, 28.0)),
                    ..default()
                },
                Transform::from_xyz(
                    x as f32 * cell_size + offset_x,
                    y as f32 * cell_size + offset_y,
                    0.0,
                ),
                Coord { row: y, col: x },
            ));
        }
    }
}
