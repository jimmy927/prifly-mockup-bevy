//! Star Catcher: a tiny 2D arcade scene for the Bevy mockup recipe.
//!
//! Sprites (the ship, falling stars and rocks, a starfield), a `bevy_ui` HUD and a Game over panel, and two
//! `States`. Everything is deterministic: a fixed time step per frame and an LCG instead of randomness, so a
//! re-run looks the same. On wasm the page hash picks the state: `#playing` (default) or `#gameover`.

use bevy::camera::ScalingMode;
use bevy::prelude::*;

const DT: f32 = 1.0 / 60.0;
const HALF_W: f32 = 512.0;
const HALF_H: f32 = 288.0;
const SHIP_Y: f32 = -248.0;
const GOLD: Color = Color::srgb(1.0, 0.82, 0.25);
const ROCK: Color = Color::srgb(0.55, 0.42, 0.38);
const PANEL: Color = Color::srgba(0.04, 0.05, 0.12, 0.88);

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum Phase {
    #[default]
    Playing,
    GameOver,
}

#[derive(Resource, Clone)]
struct Run {
    score: u32,
    best: u32,
    lives: u32,
    tick: u32,
    seed: u32,
}

impl Run {
    /// What the mockup shows mid-game: invented numbers, the same on every load.
    fn demo_playing() -> Self {
        Self { score: 120, best: 410, lives: 2, tick: 0, seed: 7 }
    }
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn demo_game_over() -> Self {
        Self { score: 240, best: 410, lives: 0, tick: 0, seed: 7 }
    }
    fn fresh(best: u32) -> Self {
        Self { score: 0, best, lives: 3, tick: 0, seed: 7 }
    }
    /// A small linear congruential generator in 0..1000.
    fn next(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        (self.seed >> 16) % 1000
    }
}

#[derive(Component)]
struct Ship;
#[derive(Component)]
struct Faller {
    star: bool,
    speed: f32,
}
#[derive(Component)]
struct ScoreText;
#[derive(Component)]
struct BestText;
#[derive(Component)]
struct LifeIcon(u32);
#[derive(Component)]
struct RetryButton;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Star Catcher".into(),
            canvas: Some("#bevy-canvas".into()),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::srgb(0.03, 0.04, 0.1)))
    .insert_resource(Run::demo_playing())
    .init_state::<Phase>()
    .add_systems(Startup, (setup_world, setup_hud))
    .add_systems(OnEnter(Phase::Playing), start_round)
    .add_systems(OnEnter(Phase::GameOver), show_game_over)
    .add_systems(Update, (play, steer, collide).chain().run_if(in_state(Phase::Playing)))
    .add_systems(Update, (update_hud, retry_button));
    #[cfg(target_arch = "wasm32")]
    app.add_systems(Update, (follow_hash, report_frame));
    app.run();
}

fn setup_world(mut commands: Commands, mut run: ResMut<Run>) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical { viewport_height: 2.0 * HALF_H },
            ..OrthographicProjection::default_2d()
        }),
    ));
    // A fixed starfield from the generator.
    for _ in 0..70 {
        let x = run.next() as f32 / 1000.0 * 2.0 * HALF_W - HALF_W;
        let y = run.next() as f32 / 1000.0 * 2.0 * HALF_H - HALF_H;
        let size = 1.0 + (run.next() % 3) as f32;
        let shade = 0.35 + run.next() as f32 / 1800.0;
        commands.spawn((
            Sprite::from_color(Color::srgb(shade, shade, shade + 0.1), Vec2::splat(size)),
            Transform::from_xyz(x, y, -10.0),
        ));
    }
    run.seed = 7;
    // Ground strip and the ship: a hull with a cockpit and an engine glow as children.
    commands.spawn((
        Sprite::from_color(Color::srgb(0.1, 0.13, 0.25), Vec2::new(2.0 * HALF_W, 14.0)),
        Transform::from_xyz(0.0, -HALF_H + 7.0, -1.0),
    ));
    commands
        .spawn((
            Ship,
            Sprite::from_color(Color::srgb(0.2, 0.75, 0.85), Vec2::new(96.0, 20.0)),
            Transform::from_xyz(0.0, SHIP_Y, 1.0),
        ))
        .with_children(|ship| {
            ship.spawn((
                Sprite::from_color(Color::srgb(0.8, 0.95, 1.0), Vec2::new(36.0, 14.0)),
                Transform::from_xyz(0.0, 13.0, 0.1),
            ));
            ship.spawn((
                Sprite::from_color(Color::srgb(1.0, 0.55, 0.2), Vec2::new(18.0, 8.0)),
                Transform::from_xyz(0.0, -13.0, -0.1),
            ));
        });
}

fn setup_hud(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            padding: UiRect::axes(px(24), px(16)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                Node { flex_direction: FlexDirection::Column, row_gap: px(2), ..default() },
                children![
                    (
                        Text::new("SCORE 0"),
                        TextFont { font_size: FontSize::Px(34.0), ..default() },
                        TextColor(GOLD),
                        ScoreText
                    ),
                    (
                        Text::new("BEST 0"),
                        TextFont { font_size: FontSize::Px(18.0), ..default() },
                        TextColor(Color::srgb(0.6, 0.65, 0.8)),
                        BestText
                    ),
                ]
            ),
            (
                Node { column_gap: px(8), ..default() },
                children![life(0), life(1), life(2)]
            ),
        ],
    ));
}

fn life(index: u32) -> impl Bundle {
    (
        Node { width: px(26), height: px(26), border_radius: BorderRadius::all(px(13)), ..default() },
        BackgroundColor(Color::srgb(0.9, 0.25, 0.3)),
        LifeIcon(index),
    )
}

fn start_round(mut commands: Commands, old: Query<Entity, With<Faller>>, mut run: ResMut<Run>) {
    for entity in &old {
        commands.entity(entity).despawn();
    }
    run.tick = 0;
    // Start with objects already in flight, so the first frame is not an empty sky.
    for row in 0..7 {
        spawn_faller(&mut commands, &mut run, HALF_H - 40.0 - row as f32 * 62.0);
    }
}

fn spawn_faller(commands: &mut Commands, run: &mut Run, y: f32) {
    let x = (run.next() as f32 / 1000.0 * 2.0 - 1.0) * (HALF_W - 40.0);
    let star = run.next() % 3 != 0;
    let speed = 150.0 + run.next() as f32 / 10.0;
    let (color, size, turn) = if star { (GOLD, 24.0, 0.785) } else { (ROCK, 34.0, 0.3) };
    commands
        .spawn((
            Faller { star, speed },
            Sprite::from_color(color, Vec2::splat(size)),
            Transform::from_xyz(x, y, 0.5).with_rotation(Quat::from_rotation_z(turn)),
        ))
        .with_children(|faller| {
            let core = if star { Color::srgb(1.0, 0.96, 0.7) } else { Color::srgb(0.38, 0.28, 0.26) };
            faller.spawn((Sprite::from_color(core, Vec2::splat(size * 0.45)), Transform::from_xyz(0.0, 0.0, 0.1)));
        });
}

fn play(mut commands: Commands, mut run: ResMut<Run>, mut fallers: Query<(Entity, &Faller, &mut Transform)>) {
    run.tick += 1;
    if run.tick % 38 == 0 {
        spawn_faller(&mut commands, &mut run, HALF_H + 20.0);
    }
    for (entity, faller, mut transform) in &mut fallers {
        transform.translation.y -= faller.speed * DT;
        transform.rotate_z(if faller.star { 1.5 * DT } else { -0.8 * DT });
        if transform.translation.y < -HALF_H - 30.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Arrow keys or A/D steer; with no key held an autopilot chases the lowest star.
fn steer(
    keys: Res<ButtonInput<KeyCode>>,
    fallers: Query<(&Faller, &Transform), Without<Ship>>,
    mut ship: Single<&mut Transform, With<Ship>>,
) {
    let left = keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]);
    let right = keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]);
    let dir = if left || right {
        (right as i8 - left as i8) as f32
    } else {
        let target = fallers
            .iter()
            .filter(|(f, t)| f.star && t.translation.y > SHIP_Y + 30.0)
            .min_by(|a, b| a.1.translation.y.total_cmp(&b.1.translation.y))
            .map_or(ship.translation.x, |(_, t)| t.translation.x);
        (target - ship.translation.x).clamp(-6.0, 6.0) / 6.0
    };
    ship.translation.x = (ship.translation.x + dir * 420.0 * DT).clamp(-HALF_W + 48.0, HALF_W - 48.0);
}

fn collide(
    mut commands: Commands,
    mut run: ResMut<Run>,
    mut next: ResMut<NextState<Phase>>,
    ship: Single<&Transform, With<Ship>>,
    fallers: Query<(Entity, &Faller, &Transform)>,
) {
    for (entity, faller, transform) in &fallers {
        let d = transform.translation.truncate() - ship.translation.truncate();
        if d.x.abs() < 56.0 && d.y.abs() < 26.0 {
            commands.entity(entity).despawn();
            if faller.star {
                run.score += 10;
                run.best = run.best.max(run.score);
            } else {
                run.lives = run.lives.saturating_sub(1);
                if run.lives == 0 {
                    next.set(Phase::GameOver);
                }
            }
        }
    }
}

fn update_hud(
    run: Res<Run>,
    mut score: Single<&mut Text, (With<ScoreText>, Without<BestText>)>,
    mut best: Single<&mut Text, (With<BestText>, Without<ScoreText>)>,
    mut icons: Query<(&LifeIcon, &mut BackgroundColor)>,
) {
    if !run.is_changed() {
        return;
    }
    score.0 = format!("SCORE {}", run.score);
    best.0 = format!("BEST {}", run.best);
    for (icon, mut color) in &mut icons {
        color.0 = if icon.0 < run.lives { Color::srgb(0.9, 0.25, 0.3) } else { Color::srgb(0.2, 0.22, 0.32) };
    }
}

fn show_game_over(mut commands: Commands, run: Res<Run>) {
    commands.spawn((
        DespawnOnExit(Phase::GameOver),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(14),
                padding: UiRect::axes(px(64), px(36)),
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(14)),
                ..default()
            },
            BackgroundColor(PANEL),
            BorderColor::all(Color::srgb(0.9, 0.25, 0.3)),
            children![
                (Text::new("GAME OVER"), TextFont { font_size: FontSize::Px(56.0), ..default() }, TextColor(Color::srgb(0.95, 0.35, 0.4))),
                (Text::new(format!("You caught {} stars", run.score / 10)), TextFont { font_size: FontSize::Px(24.0), ..default() }, TextColor(Color::WHITE)),
                (Text::new(format!("Score {}   Best {}", run.score, run.best)), TextFont { font_size: FontSize::Px(24.0), ..default() }, TextColor(GOLD)),
                (
                    Button,
                    RetryButton,
                    Node {
                        width: px(180),
                        height: px(54),
                        margin: UiRect::top(px(10)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(px(10)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.75, 0.85)),
                    children![(Text::new("Retry"), TextFont { font_size: FontSize::Px(28.0), ..default() }, TextColor(Color::srgb(0.03, 0.06, 0.1)))]
                ),
            ]
        )],
    ));
}

fn retry_button(
    mut run: ResMut<Run>,
    mut next: ResMut<NextState<Phase>>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<RetryButton>)>,
) {
    if buttons.iter().any(|i| *i == Interaction::Pressed) {
        *run = Run::fresh(run.best);
        next.set(Phase::Playing);
    }
}

#[cfg(target_arch = "wasm32")]
fn page_hash() -> String {
    web_sys::window().and_then(|w| w.location().hash().ok()).unwrap_or_default()
}

/// The page hash picks the state (`#playing`, `#gameover`); a change applies the matching demo numbers.
#[cfg(target_arch = "wasm32")]
fn follow_hash(mut seen: Local<Option<String>>, mut run: ResMut<Run>, mut next: ResMut<NextState<Phase>>) {
    let hash = page_hash();
    if seen.as_deref() == Some(hash.as_str()) {
        return;
    }
    let first = seen.is_none();
    *seen = Some(hash.clone());
    if hash == "#gameover" {
        *run = Run::demo_game_over();
        next.set(Phase::GameOver);
    } else if !first {
        *run = Run::demo_playing();
        next.set(Phase::Playing);
    }
}

/// `window.__bevyFrames` counts drawn frames; the page flips `__mockupReady` once a few have been drawn.
#[cfg(target_arch = "wasm32")]
fn report_frame(mut frames: Local<u32>) {
    *frames += 1;
    if let Some(window) = web_sys::window() {
        let _ = js_sys::Reflect::set(&window, &"__bevyFrames".into(), &(*frames).into());
    }
}
