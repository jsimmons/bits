mod blit;
mod id_table;
mod part;
mod vec;
mod virtual_vec;
mod world;

use vec::Vec3;
use world::World;

fn create_tank(world: &mut World, position: Vec3) {}

fn main() {
    println!("Hello, world!");

    let mut world = World::new();

    create_tank(&mut world, Vec3::new(0.0, 0.0, 0.0));
    create_tank(&mut world, Vec3::new(10.0, 0.0, 0.0));
    create_tank(&mut world, Vec3::new(0.0, 0.0, 10.0));
}
