//! Turn-based space combat. Each valid order resolves the announced volley.

use serde::{Deserialize, Serialize};

pub const LANES: usize = 3;
pub const MAX_HULL: u8 = 3;
const SHIELDS_PER_WAVE: u8 = 2;
const NO_STRIKE: usize = LANES;
const SAVE_VERSION: u8 = 1;

struct Wave {
    name: &'static str,
    targets: [u8; LANES],
    volleys: &'static [usize],
}

const WAVES: [Wave; 4] = [
    Wave {
        name: "Outer patrol",
        targets: [1, 0, 1],
        volleys: &[NO_STRIKE, 0, 0, 1, 2, 0, 2, 1],
    },
    Wave {
        name: "Crossfire",
        targets: [1, 1, 1],
        volleys: &[2, 0, 2, 1, 0, 1, 2, NO_STRIKE, 0],
    },
    Wave {
        name: "Vanguard",
        targets: [2, 1, 2],
        volleys: &[0, 2, 1, 0, 1, 2, 0, 2, 1, 2, 0, 1],
    },
    Wave {
        name: "Flagship",
        targets: [0, 4, 0],
        volleys: &[1, 0, 1, 2, 1, 0, 2, 1, 0, 1, 2, 0],
    },
];

pub const WAVE_COUNT: usize = WAVES.len();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    Left,
    Right,
    Fire,
    Shield,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum Status {
    Active,
    Won,
    Lost,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum Event {
    Ready,
    Moved,
    Hit,
    Miss,
    ShieldReady,
    Deflected,
    Damaged,
    WaveCleared,
    MissionCleared,
    HullLost,
    TurnLimit,
}

impl Event {
    pub fn text(self) -> &'static str {
        match self {
            Self::Ready => "Read the warning before choosing an action.",
            Self::Moved => "Ship moved.",
            Self::Hit => "Target hit.",
            Self::Miss => "No target in this lane.",
            Self::ShieldReady => "Shield spent. No incoming hit.",
            Self::Deflected => "The shield blocked the hit.",
            Self::Damaged => "Hull damaged.",
            Self::WaveCleared => "Wave cleared. Shields recharged.",
            Self::MissionCleared => "Flagship destroyed. Mission complete.",
            Self::HullLost => "Hull lost. Start a new mission to try again.",
            Self::TurnLimit => "The fleet escaped. Try a shorter route.",
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Game {
    save_version: u8,
    wave: usize,
    turn: usize,
    lane: usize,
    hull: u8,
    shields: u8,
    targets: [u8; LANES],
    points: u16,
    best: u16,
    status: Status,
    event: Event,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            save_version: SAVE_VERSION,
            wave: 0,
            turn: 0,
            lane: 1,
            hull: MAX_HULL,
            shields: SHIELDS_PER_WAVE,
            targets: WAVES[0].targets,
            points: 0,
            best: 0,
            status: Status::Active,
            event: Event::Ready,
        }
    }
}

impl Game {
    pub fn validated(self) -> Option<Self> {
        let wave = WAVES.get(self.wave)?;
        let valid = self.save_version == SAVE_VERSION
            && self.lane < LANES
            && self.hull <= MAX_HULL
            && self.shields <= SHIELDS_PER_WAVE
            && self.turn <= wave.volleys.len()
            && self
                .targets
                .iter()
                .zip(wave.targets)
                .all(|(&hits, max)| hits <= max)
            && self.points <= self.best
            && self.best <= 2400
            && match self.status {
                Status::Active => {
                    self.hull > 0 && self.turn < wave.volleys.len() && self.targets != [0; LANES]
                }
                Status::Won => {
                    self.wave + 1 == WAVE_COUNT && self.targets == [0; LANES] && self.hull > 0
                }
                Status::Lost => self.hull == 0 || self.turn == wave.volleys.len(),
            };
        valid.then_some(self)
    }

    pub fn wave_number(&self) -> usize {
        self.wave + 1
    }
    pub fn wave_name(&self) -> &'static str {
        WAVES[self.wave].name
    }
    pub fn lane(&self) -> usize {
        self.lane
    }
    pub fn hull(&self) -> u8 {
        self.hull
    }
    pub fn shields(&self) -> u8 {
        self.shields
    }
    pub fn targets(&self) -> [u8; LANES] {
        self.targets
    }
    pub fn points(&self) -> u16 {
        self.points
    }
    pub fn best(&self) -> u16 {
        self.best
    }
    pub fn status(&self) -> Status {
        self.status
    }
    pub fn event(&self) -> Event {
        self.event
    }
    pub fn turns_left(&self) -> usize {
        WAVES[self.wave].volleys.len() - self.turn
    }

    pub fn warning(&self) -> Option<usize> {
        if self.status != Status::Active {
            return None;
        }
        let lane = WAVES[self.wave].volleys[self.turn];
        (lane < LANES).then_some(lane)
    }

    pub fn can_order(&self, order: Order) -> bool {
        self.status == Status::Active
            && match order {
                Order::Left => self.lane > 0,
                Order::Right => self.lane + 1 < LANES,
                Order::Fire => true,
                Order::Shield => self.shields > 0,
            }
    }

    pub fn restart(&mut self) {
        *self = Self {
            best: self.best,
            ..Self::default()
        };
    }

    pub fn order(&mut self, order: Order) -> bool {
        if !self.can_order(order) {
            return false;
        }
        let warning = self.warning();
        self.event = match order {
            Order::Left => {
                self.lane -= 1;
                Event::Moved
            }
            Order::Right => {
                self.lane += 1;
                Event::Moved
            }
            Order::Fire => {
                if self.targets[self.lane] == 0 {
                    Event::Miss
                } else {
                    self.targets[self.lane] -= 1;
                    self.add_points(100);
                    if self.targets == [0; LANES] {
                        self.complete_wave();
                        return true;
                    }
                    Event::Hit
                }
            }
            Order::Shield => {
                self.shields -= 1;
                Event::ShieldReady
            }
        };
        if warning == Some(self.lane) {
            if order == Order::Shield {
                self.event = Event::Deflected;
            } else {
                self.hull -= 1;
                self.event = Event::Damaged;
            }
        }
        self.turn += 1;
        if self.hull == 0 {
            self.status = Status::Lost;
            self.event = Event::HullLost;
        } else if self.turns_left() == 0 {
            self.status = Status::Lost;
            self.event = Event::TurnLimit;
        }
        true
    }

    fn add_points(&mut self, points: u16) {
        self.points += points;
        self.best = self.best.max(self.points);
    }

    fn complete_wave(&mut self) {
        self.add_points(250);
        if self.wave + 1 == WAVE_COUNT {
            self.status = Status::Won;
            self.event = Event::MissionCleared;
        } else {
            self.wave += 1;
            self.turn = 0;
            self.lane = 1;
            self.shields = SHIELDS_PER_WAVE;
            self.targets = WAVES[self.wave].targets;
            self.event = Event::WaveCleared;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashSet, VecDeque};

    #[test]
    fn a_shield_blocks_only_the_announced_volley() {
        let mut game = Game::default();
        assert!(game.order(Order::Left));
        assert_eq!(game.warning(), Some(0));
        assert!(game.order(Order::Shield));
        assert_eq!(
            (game.event(), game.hull(), game.shields()),
            (Event::Deflected, 3, 1)
        );
        assert_eq!(game.warning(), Some(0));
        assert!(game.order(Order::Fire));
        assert_eq!(
            (game.event(), game.hull(), game.points()),
            (Event::Damaged, 2, 100)
        );
        assert!(!game.order(Order::Left), "the edge cannot consume a turn");
    }

    #[test]
    fn a_mission_can_be_completed_without_taking_damage() {
        let mut queue = VecDeque::from([Game::default()]);
        let mut seen = HashSet::new();
        while let Some(game) = queue.pop_front() {
            if game.status() == Status::Won {
                assert_eq!(
                    (game.wave_number(), game.hull(), game.points()),
                    (4, MAX_HULL, 2400)
                );
                assert!(game.clone().validated().is_some());
                return;
            }
            if game.status() != Status::Active || game.hull() != MAX_HULL {
                continue;
            }
            let key = (game.wave, game.turn, game.lane, game.shields, game.targets);
            if !seen.insert(key) {
                continue;
            }
            for order in [Order::Left, Order::Right, Order::Fire, Order::Shield] {
                let mut next = game.clone();
                if next.order(order) {
                    queue.push_back(next);
                }
            }
        }
        panic!("the whole mission must have a no-damage route");
    }

    #[test]
    fn restart_keeps_the_best_and_invalid_saves_are_rejected() {
        let mut game = Game::default();
        game.order(Order::Right);
        game.order(Order::Fire);
        game.restart();
        assert_eq!((game.points(), game.best()), (0, 100));
        game.lane = 70;
        assert!(game.validated().is_none());
    }
}
