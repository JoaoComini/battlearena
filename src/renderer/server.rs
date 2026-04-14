use bevy::prelude::*;

pub struct ExampleServerRendererPlugin {
    /// The name of the example, which must also match the edgegap application name.
    pub name: String,
}

impl ExampleServerRendererPlugin {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Resource)]
struct GameName(String);

impl Plugin for ExampleServerRendererPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameName(self.name.clone()));
        app.insert_resource(ClearColor::default());
        app.add_systems(Startup, set_window_title);
    }
}

fn set_window_title(mut window: Query<&mut Window>, game_name: Res<GameName>) {
    let mut window = window.single_mut().unwrap();
    window.title = format!("Lightyear Example: {}", game_name.0);
}
