//! Deterministic arcade combat. A tick advances the mission independently of steering.

use serde::{Deserialize, Serialize};

pub const COLUMNS: u8 = 13;
pub const PLAYER_ROW: u8 = 15;
pub const MAX_HULL: u8 = 3;

const SAVE_VERSION: u8 = 1;
const FIRE_EVERY: u16 = 3;
const BREAK_TICKS: u8 = 5;
const MAX_TICKS: u16 = 700;
const MAX_SCORE: u16 = 2_750;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Point {
    pub column: u8,
    pub row: u8,
}

impl Point {
    const fn new(column: u8, row: u8) -> Self {
        Self { column, row }
    }

    fn in_field(self) -> bool {
        self.column < COLUMNS && self.row < PLAYER_ROW
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum Ship {
    Scout,
    Guard,
    Flagship,
}

impl Ship {
    pub const fn max_health(self) -> u8 {
        match self {
            Self::Scout => 1,
            Self::Guard => 2,
            Self::Flagship => 8,
        }
    }

    fn value(self) -> u16 {
        match self {
            Self::Scout => 100,
            Self::Guard => 200,
            Self::Flagship => 750,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Enemy {
    pub point: Point,
    pub ship: Ship,
    pub health: u8,
}

impl Enemy {
    const fn new(column: u8, row: u8, ship: Ship) -> Self {
        Self {
            point: Point::new(column, row),
            ship,
            health: ship.max_health(),
        }
    }
}

struct Wave {
    name: &'static str,
    fleet: &'static [Enemy],
    move_every: u16,
    fire_every: u16,
}

const WAVES: &[Wave] = &[
    Wave {
        name: "Outer patrol",
        fleet: &[
            Enemy::new(2, 2, Ship::Scout),
            Enemy::new(5, 1, Ship::Scout),
            Enemy::new(8, 2, Ship::Scout),
        ],
        move_every: 5,
        fire_every: 12,
    },
    Wave {
        name: "Crossfire",
        fleet: &[
            Enemy::new(2, 2, Ship::Scout),
            Enemy::new(4, 0, Ship::Scout),
            Enemy::new(6, 2, Ship::Guard),
            Enemy::new(8, 0, Ship::Scout),
            Enemy::new(10, 2, Ship::Scout),
        ],
        move_every: 4,
        fire_every: 9,
    },
    Wave {
        name: "Vanguard",
        fleet: &[
            Enemy::new(1, 2, Ship::Guard),
            Enemy::new(3, 3, Ship::Scout),
            Enemy::new(5, 1, Ship::Scout),
            Enemy::new(7, 3, Ship::Scout),
            Enemy::new(9, 2, Ship::Guard),
        ],
        move_every: 3,
        fire_every: 7,
    },
    Wave {
        name: "Flagship",
        fleet: &[
            Enemy::new(3, 3, Ship::Guard),
            Enemy::new(6, 1, Ship::Flagship),
            Enemy::new(9, 3, Ship::Guard),
        ],
        move_every: 3,
        fire_every: 6,
    },
];

pub const WAVE_COUNT: usize = WAVES.len();

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum Status {
    Ready,
    Flying,
    WaveClear,
    Won,
    Lost,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Game {
    save_version: u8,
    wave: usize,
    tick: u16,
    intermission: u8,
    column: u8,
    hull: u8,
    right: bool,
    fleet: Vec<Enemy>,
    friendly: Vec<Point>,
    hostile: Vec<Point>,
    score: u16,
    best: u16,
    status: Status,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            save_version: SAVE_VERSION,
            wave: 0,
            tick: 0,
            intermission: 0,
            column: COLUMNS / 2,
            hull: MAX_HULL,
            right: true,
            fleet: WAVES[0].fleet.to_vec(),
            friendly: Vec::new(),
            hostile: Vec::new(),
            score: 0,
            best: 0,
            status: Status::Ready,
        }
    }
}

impl Game {
    pub fn validated(self) -> Option<Self> {
        let wave = WAVES.get(self.wave)?;
        let valid = self.save_version == SAVE_VERSION
            && self.tick <= MAX_TICKS
            && self.column < COLUMNS
            && self.hull <= MAX_HULL
            && self.score <= self.best
            && self.best <= MAX_SCORE
            && self.fleet.len() <= wave.fleet.len()
            && self.fleet.iter().enumerate().all(|(index, enemy)| {
                enemy.point.in_field()
                    && enemy.point.row < PLAYER_ROW - 1
                    && (1..=enemy.ship.max_health()).contains(&enemy.health)
                    && self.fleet[..index]
                        .iter()
                        .all(|other| other.point != enemy.point)
            })
            && self.friendly.len() <= 3
            && self.hostile.len() <= 5
            && self
                .friendly
                .iter()
                .chain(&self.hostile)
                .all(|shot| shot.in_field())
            && [Ship::Scout, Ship::Guard, Ship::Flagship]
                .into_iter()
                .all(|ship| {
                    self.fleet.iter().filter(|enemy| enemy.ship == ship).count()
                        <= wave.fleet.iter().filter(|enemy| enemy.ship == ship).count()
                })
            && self.score
                == WAVES[..=self.wave]
                    .iter()
                    .flat_map(|wave| wave.fleet)
                    .map(|enemy| enemy.ship.value())
                    .sum::<u16>()
                    - self
                        .fleet
                        .iter()
                        .map(|enemy| enemy.ship.value())
                        .sum::<u16>()
            && match self.status {
                Status::Ready => {
                    self == Self {
                        best: self.best,
                        ..Self::default()
                    }
                }
                Status::Flying => self.hull > 0 && !self.fleet.is_empty() && self.intermission == 0,
                Status::WaveClear => {
                    self.hull > 0
                        && self.fleet.is_empty()
                        && self.wave + 1 < WAVE_COUNT
                        && (1..=BREAK_TICKS).contains(&self.intermission)
                }
                Status::Won => {
                    self.hull > 0 && self.fleet.is_empty() && self.wave + 1 == WAVE_COUNT
                }
                Status::Lost => self.hull == 0 && !self.fleet.is_empty(),
            };
        valid.then_some(self)
    }

    pub fn wave_number(&self) -> usize {
        self.wave + 1
    }

    pub fn wave_name(&self) -> &'static str {
        WAVES[self.wave].name
    }

    pub fn column(&self) -> u8 {
        self.column
    }

    pub fn hull(&self) -> u8 {
        self.hull
    }

    pub fn score(&self) -> u16 {
        self.score
    }

    pub fn best(&self) -> u16 {
        self.best
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn fleet(&self) -> &[Enemy] {
        &self.fleet
    }

    pub fn friendly(&self) -> &[Point] {
        &self.friendly
    }

    pub fn hostile(&self) -> &[Point] {
        &self.hostile
    }

    pub fn start(&mut self) {
        if matches!(self.status, Status::Lost | Status::Won) {
            *self = Self {
                best: self.best,
                ..Self::default()
            };
        }
        if self.status == Status::Ready {
            self.status = Status::Flying;
            self.fire();
        }
    }

    pub fn steer(&mut self, column: u8) -> bool {
        if self.status != Status::Flying || column >= COLUMNS || self.column == column {
            return false;
        }
        self.column = column;
        true
    }

    /// Advance one simulation step. Returns whether the visible state changed.
    pub fn advance(&mut self) -> bool {
        match self.status {
            Status::Flying => self.advance_flight(),
            Status::WaveClear => {
                self.intermission -= 1;
                if self.intermission != 0 {
                    return false;
                }
                self.wave += 1;
                self.fleet = WAVES[self.wave].fleet.to_vec();
                self.tick = 0;
                self.right = true;
                self.status = Status::Flying;
                self.fire();
            }
            _ => return false,
        }
        true
    }

    fn advance_flight(&mut self) {
        self.tick += 1;
        let wave = &WAVES[self.wave];
        if self.tick >= MAX_TICKS
            || (self.tick.is_multiple_of(wave.move_every) && self.move_fleet())
        {
            self.lose_hull();
            return;
        }
        self.advance_friendly();
        if self.fleet.is_empty() {
            self.friendly.clear();
            self.hostile.clear();
            self.status = if self.wave + 1 == WAVE_COUNT {
                Status::Won
            } else {
                self.intermission = BREAK_TICKS;
                Status::WaveClear
            };
            return;
        }
        self.advance_hostile();
        if self.hull == 0 {
            self.lose_hull();
            return;
        }

        if self.tick.is_multiple_of(FIRE_EVERY) {
            self.fire();
        }
        if self.tick.is_multiple_of(wave.fire_every) {
            let volley = usize::from(self.tick / wave.fire_every);
            let shooter = self.fleet[(volley - 1) % self.fleet.len()];
            self.hostile
                .push(Point::new(shooter.point.column, shooter.point.row + 1));
        }
    }

    fn fire(&mut self) {
        self.friendly.push(Point::new(self.column, PLAYER_ROW - 1));
    }

    /// Move the formation one column or turn and descend at a wall.
    /// Reaching the flight line ends the mission.
    fn move_fleet(&mut self) -> bool {
        let edge = if self.right { COLUMNS - 1 } else { 0 };
        if self.fleet.iter().any(|ship| ship.point.column == edge) {
            self.right = !self.right;
            if self
                .fleet
                .iter()
                .any(|ship| ship.point.row == PLAYER_ROW - 2)
            {
                return true;
            }
            for ship in &mut self.fleet {
                ship.point.row += 1;
            }
        } else {
            for ship in &mut self.fleet {
                if self.right {
                    ship.point.column += 1;
                } else {
                    ship.point.column -= 1;
                }
            }
        }
        false
    }

    fn advance_friendly(&mut self) {
        self.friendly.retain_mut(|shot| {
            for step in 0..=2 {
                if step != 0 {
                    let Some(row) = shot.row.checked_sub(1) else {
                        return false;
                    };
                    shot.row = row;
                }
                if let Some(index) = self.fleet.iter().position(|enemy| enemy.point == *shot) {
                    let enemy = &mut self.fleet[index];
                    enemy.health -= 1;
                    if enemy.health == 0 {
                        self.score += enemy.ship.value();
                        self.best = self.best.max(self.score);
                        self.fleet.remove(index);
                    }
                    return false;
                }
            }
            true
        });
    }

    fn advance_hostile(&mut self) {
        self.hostile.retain_mut(|shot| {
            shot.row += 1;
            if shot.row == PLAYER_ROW && shot.column == self.column {
                self.hull = self.hull.saturating_sub(1);
            }
            shot.row < PLAYER_ROW
        });
    }

    fn lose_hull(&mut self) {
        self.hull = 0;
        self.status = Status::Lost;
        self.friendly.clear();
        self.hostile.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_fire_must_reach_a_ship_before_it_can_hit() {
        let mut game = Game {
            column: 2,
            ..Game::default()
        };
        game.start();
        assert_eq!(game.friendly(), [Point::new(2, PLAYER_ROW - 1)]);
        assert!(game.advance());
        assert_eq!(game.friendly(), [Point::new(2, PLAYER_ROW - 3)]);
        assert_eq!(game.score(), 0);

        game.fleet = vec![Enemy::new(2, 10, Ship::Guard)];
        game.advance();
        assert_eq!(game.fleet()[0].health, 1);
        assert_eq!(game.score(), 0);
        assert!(game.friendly().is_empty());
    }

    #[test]
    fn steering_dodges_a_travelling_hostile_bolt() {
        let mut game = Game::default();
        game.start();
        game.hostile = vec![Point::new(6, PLAYER_ROW - 2), Point::new(7, PLAYER_ROW - 1)];
        assert!(game.steer(7));
        game.advance();
        assert_eq!(game.hull(), MAX_HULL - 1);
        assert_eq!(game.hostile(), [Point::new(6, PLAYER_ROW - 1)]);
        assert!(!game.steer(COLUMNS));
        game.advance();
        assert_eq!(game.hull(), MAX_HULL - 1);
        assert!(game.hostile().is_empty());
    }

    #[test]
    fn last_hit_clears_the_field_then_starts_the_next_wave() {
        let mut game = Game::default();
        game.start();
        game.fleet = vec![Enemy::new(6, PLAYER_ROW - 3, Ship::Scout)];
        game.hostile.push(Point::new(6, PLAYER_ROW - 1));
        game.advance();
        assert_eq!(game.status(), Status::WaveClear);
        assert_eq!(game.hull(), MAX_HULL);
        assert!(game.friendly().is_empty() && game.hostile().is_empty());
        for _ in 1..BREAK_TICKS {
            assert!(!game.advance());
        }
        assert!(game.advance());
        assert_eq!(game.status(), Status::Flying);
        assert_eq!(game.wave_number(), 2);
        assert_eq!(game.fleet(), WAVES[1].fleet);
        assert_eq!(game.score(), 100);
    }

    #[test]
    fn fleet_turns_and_descends_at_a_wall() {
        let mut game = Game::default();
        game.start();
        game.fleet = vec![Enemy::new(COLUMNS - 1, 4, Ship::Guard)];
        assert!(!game.move_fleet());
        assert_eq!(game.fleet()[0].point, Point::new(COLUMNS - 1, 5));
        assert!(!game.move_fleet());
        assert_eq!(game.fleet()[0].point, Point::new(COLUMNS - 2, 5));
    }

    #[test]
    fn an_aiming_and_dodging_pilot_can_finish_the_mission() {
        use std::cmp::Reverse;

        let mut game = Game::default();
        game.start();
        for _ in 0..MAX_TICKS {
            if game.status() == Status::Flying {
                // Compare short routes without changing the live mission.
                let column = (0..COLUMNS)
                    .max_by_key(|&column| {
                        let mut route = game.clone();
                        route.steer(column);
                        for _ in 0..8 {
                            route.advance();
                        }
                        let fleet_health = route.fleet().iter().map(|ship| ship.health).sum::<u8>();
                        (
                            route.hull(),
                            route.score(),
                            Reverse(fleet_health),
                            column == game.column(),
                        )
                    })
                    .unwrap();
                game.steer(column);
            }
            game.advance();
            assert!(game.clone().validated().is_some());
            if game.status() == Status::Won {
                assert_eq!(game.score(), MAX_SCORE);
                assert_eq!(game.hull(), MAX_HULL);
                assert_eq!(game.wave_number(), WAVE_COUNT);
                return;
            }
            assert_ne!(game.status(), Status::Lost);
        }
        panic!("a pilot must be able to defeat the flagship");
    }

    #[test]
    fn restarting_a_defeat_preserves_best_and_invalid_saves_are_rejected() {
        let mut game = Game::default();
        game.start();
        game.best = 200;
        game.lose_hull();
        assert!(game.clone().validated().is_some());
        game.start();
        assert_eq!((game.score(), game.best(), game.hull()), (0, 200, MAX_HULL));
        assert!(game.clone().validated().is_some());
        game.column = COLUMNS;
        assert!(game.validated().is_none());
    }
}
