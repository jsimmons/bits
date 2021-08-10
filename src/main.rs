mod blit;
mod maths;
mod part;
mod raw_table;
mod registry;
mod ring_vec;
mod sparse_vec;
mod virtual_vec;
mod world;

use blit::Blit;
use maths::Vec3;
use part::Part;
use world::{ThingId, World};

use crate::registry::Registry;

struct PosX(f32);
struct PosY(f32);
struct PosZ(f32);
struct Parent(ThingId);

unsafe impl Blit for PosX {}
unsafe impl Blit for PosY {}
unsafe impl Blit for PosZ {}
unsafe impl Blit for Parent {}

impl Part for PosX {}
impl Part for PosY {}
impl Part for PosZ {}
impl Part for Parent {}

// - root
//   - body
//   - left track
//   - right track
//   - main turret
//     - baby turret
fn create_tank(world: &mut World, position: Vec3) {}

fn main() {
    println!("Hello, world!");

    let mut registry = Registry::new();
    registry.register_part::<PosX>();
    registry.register_part::<PosY>();
    registry.register_part::<PosZ>();
    registry.register_part::<Parent>();
    let mut world = World::new(&registry);

    create_tank(&mut world, Vec3::new(0.0, 0.0, 0.0));
    create_tank(&mut world, Vec3::new(10.0, 0.0, 0.0));
    create_tank(&mut world, Vec3::new(0.0, 0.0, 10.0));
}
