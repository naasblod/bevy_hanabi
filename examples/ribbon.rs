//! Draws a trail and connects the trails using a ribbon.

use bevy::color::palettes::css::{BLACK, YELLOW};
use bevy::math::vec4;
use bevy::prelude::*;
use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    math::vec3,
};
use bevy_hanabi::prelude::*;

mod utils;
use rand::{thread_rng, Rng};
use utils::*;

// These determine the shape of the Spirograph:
// https://en.wikipedia.org/wiki/Spirograph#Mathematical_basis
const K: f32 = 0.64;
const L: f32 = 0.384;

const TIME_SCALE: f32 = 10.0;
const SHAPE_SCALE: f32 = 25.0;
const LIFETIME: f32 = 1.5;
const TRAIL_SPAWN_RATE: f32 = 256.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_exit = utils::make_test_app("ribbon")
        .add_systems(Startup, setup)
        .add_systems(Update, move_particle_effect)
        .run();
    app_exit.into_result()
}

fn setup(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., 50.)),
            camera: Camera {
                hdr: true,
                clear_color: Color::BLACK.into(),
                ..default()
            },
            tonemapping: Tonemapping::None,
            ..default()
        },
        BloomSettings::default(),
    ));

    let writer = ExprWriter::new();

    let init_position_attr = SetAttributeModifier {
        attribute: Attribute::POSITION,
        value: writer.lit(Vec3::ZERO).expr(),
    };

    let init_velocity_attr = SetAttributeModifier {
        attribute: Attribute::VELOCITY,
        value: writer.lit(Vec3::ZERO).expr(),
    };

    let init_age_attr = SetAttributeModifier {
        attribute: Attribute::AGE,
        value: writer.lit(0.0).expr(),
    };

    let init_lifetime_attr = SetAttributeModifier {
        attribute: Attribute::LIFETIME,
        value: writer.lit(999999.0).expr(),
    };

    let init_size_attr = SetAttributeModifier {
        attribute: Attribute::SIZE,
        value: writer.lit(0.5).expr(),
    };

    let clone_modifier = CloneModifier::new(1.0 / TRAIL_SPAWN_RATE, 1);

    let pos = writer.add_property("head_pos", Vec3::ZERO.into());
    let pos = writer.prop(pos);

    let move_modifier = SetAttributeModifier {
        attribute: Attribute::POSITION,
        value: pos.expr(),
    };

    let update_lifetime_attr = SetAttributeModifier {
        attribute: Attribute::LIFETIME,
        value: writer.lit(LIFETIME).expr(),
    };

    let render_color = ColorOverLifetimeModifier {
        gradient: Gradient::linear(vec4(3.0, 0.0, 0.0, 1.0), vec4(3.0, 0.0, 0.0, 0.0)),
    };

    let effect = EffectAsset::new(
        vec![256, 32768],
        Spawner::once(1.0.into(), true),
        writer.finish(),
    )
    .with_name("ribbon")
    .with_simulation_space(SimulationSpace::Global)
    .init(init_position_attr)
    .init(init_velocity_attr)
    .init(init_age_attr)
    .init(init_lifetime_attr)
    .init(init_size_attr)
    .update_groups(move_modifier, ParticleGroupSet::single(0))
    .update_groups(clone_modifier, ParticleGroupSet::single(0))
    .update_groups(update_lifetime_attr, ParticleGroupSet::single(1))
    .render(SizeOverLifetimeModifier {
        gradient: Gradient::linear(Vec2::ONE, Vec2::ZERO),
        ..default()
    })
    .render(RibbonModifier)
    .render_groups(render_color, ParticleGroupSet::single(1));

    let effect = effects.add(effect);

    commands
        .spawn(ParticleEffectBundle {
            effect: ParticleEffect::new(effect),
            transform: Transform::IDENTITY,
            ..default()
        })
        .insert(Name::new("ribbon"));
}

fn move_particle_effect(
    mut gizmos: Gizmos,
    mut query: Query<&mut Transform, With<ParticleEffect>>,
    mut effect: Query<(&mut EffectProperties, &mut EffectSpawner)>,
    timer: Res<Time>,
) {
    let theta = timer.elapsed_seconds() * 3.0;
    let Ok((mut properties, mut spawner)) = effect.get_single_mut() else {
        return;
    };
    for mut transform in query.iter_mut() {
        let time = timer.elapsed_seconds() * TIME_SCALE;
        let pos = vec3(
            (1.0 - K) * (time.clone().cos()) + (L * K) * (((1.0 - K) / K) * time.clone()).cos(),
            (1.0 - K) * (time.clone().sin()) - (L * K) * (((1.0 - K) / K) * time.clone()).sin(),
            0.0,
        ) * SHAPE_SCALE;

        //let pos = vec3(f32::cos(theta), f32::sin(theta), 0.0) * 5.0;
        properties.set("head_pos", (pos).into());
        gizmos.sphere(pos, Quat::IDENTITY, 1.0, YELLOW);
        transform.translation = pos;
    }
}
