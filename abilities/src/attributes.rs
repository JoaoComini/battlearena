use bevy::{ecs::component::Mutable, prelude::*};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

pub trait Attribute: Component<Mutability = Mutable> {
    fn apply_modifier(&mut self, value: f32, modifier: Modifier);
}

#[derive(Message)]
pub struct EffectEvent<A: Attribute> {
    pub target: Entity,
    pub value: f32,
    pub modifier: Modifier,
    _marker: PhantomData<A>,
}

impl<A: Attribute> EffectEvent<A> {
    pub fn new(target: Entity, value: f32, modifier: Modifier) -> Self {
        Self {
            target,
            value,
            modifier,
            _marker: PhantomData,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Modifier {
    Add,
    Mul,
}

#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MovementSpeed(pub f32);

impl Attribute for MovementSpeed {
    fn apply_modifier(&mut self, value: f32, modifier: Modifier) {
        match modifier {
            Modifier::Add => self.0 += value,
            Modifier::Mul => self.0 *= value,
        }
    }
}

#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Attribute for Health {
    fn apply_modifier(&mut self, value: f32, modifier: Modifier) {
        match modifier {
            Modifier::Add => self.current = (self.current + value).clamp(0.0, self.max),
            Modifier::Mul => self.current = (self.current * value).clamp(0.0, self.max),
        }
    }
}

impl Health {
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

pub fn lerp_health(start: Health, end: Health, t: f32) -> Health {
    Health {
        current: start.current + (end.current - start.current) * t,
        max: end.max,
    }
}

#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Energy {
    pub current: f32,
    pub max: f32,
}

impl Energy {
    pub fn new(max: f32) -> Self {
        Self { current: 0.0, max }
    }
}

impl Default for Energy {
    fn default() -> Self {
        Self::new(100.0)
    }
}

impl Attribute for Energy {
    fn apply_modifier(&mut self, value: f32, modifier: Modifier) {
        match modifier {
            Modifier::Add => self.current = (self.current + value).clamp(0.0, self.max),
            Modifier::Mul => self.current = (self.current * value).clamp(0.0, self.max),
        }
    }
}
