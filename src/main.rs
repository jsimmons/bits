mod blit;
mod depot;
mod maths;
mod raw_table;
mod registry;
mod ring_buf;
mod sparse_vec;
mod virtual_vec;
mod world;

use std::{thread::sleep, time::Duration};

use blit::Blit;
use depot::Link;
use maths::{Quat, Vec3};
use world::{ThingId, World};

use crate::{depot::Depot, registry::Registry};

#[repr(C)]
#[derive(Copy, Clone)]
struct PosX(f32);

#[repr(C)]
#[derive(Copy, Clone)]
struct PosY(f32);

#[repr(C)]
#[derive(Copy, Clone)]
struct PosZ(f32);

#[repr(C)]
#[derive(Copy, Clone)]
struct Orient(Quat);

#[repr(C)]
#[derive(Copy, Clone)]
struct Mesh(Link);

#[repr(C)]
#[derive(Copy, Clone)]
struct Parent(ThingId);

#[derive(Default)]
struct Turret {
    azimuth: f32,
    azimuth_max: f32,
    azimuth_min: f32,
    elevation: f32,
    elevation_min: f32,
    elevation_max: f32,
}

unsafe impl Blit for PosX {}
unsafe impl Blit for PosY {}
unsafe impl Blit for PosZ {}
unsafe impl Blit for Orient {}
unsafe impl Blit for Parent {}
unsafe impl Blit for Turret {}

fn main() {
    println!("Hello, world!");

    let mut registry = Registry::new();
    registry.register_part::<PosX>();
    registry.register_part::<PosY>();
    registry.register_part::<PosZ>();
    registry.register_part::<Orient>();
    registry.register_part::<Parent>();
    registry.register_part::<Turret>();

    let _depot = Depot::new(&registry);
    let _world = World::new(&registry);

    // let mut factory = world.factory();
    // let body = factory
    //     .thing()
    //     .add_shared_part(Mesh)
    //     .add_part(PosX(10.0))
    //     .add_part(PosY(0.0))
    //     .add_part(PosZ(0.0))
    //     .finish();

    // let left_track = factory()
    //     .thing()
    //     .add_part(PosZ(-1.0))
    //     .add_part(Parent(body))
    //     .finish();

    // let right_track = factory()
    //     .thing()
    //     .add_part(PosZ(1.0))
    //     .add_part(Parent(body))
    //     .finish();

    // let main_turret = factory()
    //     .thing()
    //     .add_part(PosY(0.25))
    //     .add_part(Parent(body))
    //     .add_part(Turret::default())
    //     .finish();

    // let machinegun = factory()
    //     .thing()
    //     .add_part(PosY(0.1))
    //     .add_part(PosZ(0.1))
    //     .add_part(Parent(body))
    //     .add_part(Turret::default())
    //     .finish();

    loop {
        sleep(Duration::from_secs(1))
    }
}
