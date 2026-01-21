mod visualization;

use bevy::prelude::*;
use visualization::WFCPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WFCPlugin)
        .run();
}
