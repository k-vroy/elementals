use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, DiagnosticsStore};

#[derive(Component)]
pub struct FpsText;

pub fn setup_fps_counter(mut commands: Commands) {
    commands.spawn((
        Text::new("FPS: --"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        FpsText,
    ));
}

pub fn update_fps_counter(
    diagnostics: Res<DiagnosticsStore>,
    mut fps_text_query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut fps_text_query {
        if let Some(fps) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            text.0 = format!("FPS: {:.1}", fps);
        }
    }
}