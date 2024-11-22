use std::sync::Arc;
use std::future::Future;
use futures_signals::signal::{SignalExt};
use dominator::clone;

use crate::grid::{EXPLOSION_ANIMATION_TIME, UNIT_MOVE_TIME, Grid, Coord};
use crate::grid::unit::{Unit, UnitAnimation};
use crate::grid::explosion::{Explosion, ExplosionAnimation};


#[derive(Debug, Clone, Copy)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

impl MoveDirection {
    fn end(self, mut start: Coord, length: f32) -> Coord {
        match self {
            Self::Up => start.y -= length,
            Self::Down => start.y += length,
            Self::Left => start.x -= length,
            Self::Right => start.x += length,
        }

        start
    }

    fn animation(self) -> UnitAnimation {
        match self {
            Self::Up => UnitAnimation::Up,
            Self::Down => UnitAnimation::Down,
            Self::Left => UnitAnimation::Left,
            Self::Right => UnitAnimation::Right,
        }
    }
}


impl Grid {
    pub fn wait(self: &Arc<Self>, duration: f64) -> impl Future<Output = ()> + Send {
        let timer = self.timer(duration);

        async move {
            timer.for_each(move |_| async {}).await;
        }
    }


    pub fn move_unit(self: &Arc<Self>, unit: &Arc<Unit>, direction: MoveDirection, length: f32) -> impl Future<Output = ()> + Send {
        let grid = self.clone();
        let unit = unit.clone();

        async move {
            let start = unit.coord.get();
            let end = direction.end(start, length);

            unit.animation.set_neq(direction.animation());

            grid.timer((length as f64) * UNIT_MOVE_TIME)
                .for_each(clone!(unit => move |percent| {
                    unit.coord.set(start.lerp(end, percent as f32));
                    async {}
                })).await;

            unit.animation.set_neq(UnitAnimation::Idle);
        }
    }


    pub fn explosion(self: &Arc<Self>, animation: ExplosionAnimation, coord: Coord) -> impl Future<Output = ()> + Send {
        let grid = self.clone();

        async move {
            let explosion = Explosion::new(coord, animation);

            grid.explosions.insert(explosion.clone());

            grid.timer(EXPLOSION_ANIMATION_TIME)
                .for_each(clone!(explosion => move |percent| {
                    explosion.percent.set(percent as f32);
                    async {}
                })).await;

            grid.explosions.remove(&explosion);
        }
    }


    pub fn hide_unit(self: &Arc<Self>, unit: &Arc<Unit>, time: f64) -> impl Future<Output = ()> + Send {
        let grid = self.clone();
        let unit = unit.clone();

        async move {
            grid.timer(time)
                .for_each(move |percent| {
                    unit.alpha.set((1.0 - percent) as f32);
                    async {}
                }).await;
        }
    }


    pub fn show_unit(self: &Arc<Self>, unit: &Arc<Unit>, time: f64) -> impl Future<Output = ()> + Send {
        let grid = self.clone();
        let unit = unit.clone();

        async move {
            grid.timer(time)
                .for_each(move |percent| {
                    unit.alpha.set(percent as f32);
                    async {}
                }).await;
        }
    }


    pub fn destroy_unit(self: &Arc<Self>, unit: &Arc<Unit>) -> impl Future<Output = ()> + Send {
        let grid = self.clone();
        let unit = unit.clone();

        async move {
            let coord = unit.coord.get();

            let explosion = Explosion::new(coord, unit.class.explosion_animation());

            grid.explosions.insert(explosion.clone());

            grid.units.remove(&unit);

            grid.timer(EXPLOSION_ANIMATION_TIME)
                .for_each(clone!(explosion => move |percent| {
                    explosion.percent.set(percent as f32);
                    async {}
                })).await;

            grid.explosions.remove(&explosion);
        }
    }
}
