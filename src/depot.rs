use crate::registry::Registry;

#[derive(Copy, Clone, Debug)]
pub struct Link {
    hash: u128,
}

pub struct Depot<'registry> {
    registry: &'registry Registry,
}

impl<'registry> Depot<'registry> {
    pub fn new(registry: &'registry Registry) -> Self {
        Self { registry }
    }
}
