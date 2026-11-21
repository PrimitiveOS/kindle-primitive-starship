use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
    time::{Duration, Instant},
};

use primitive_app::{App, Change};
use primitive_ui::{
    back_button, header, screen_rect, Canvas, Input, Rect, TextStyle, BLACK, BODY, CAPTION, PAPER,
};
use starship_core::{Enemy, Game, Point, Ship, Status, COLUMNS, MAX_HULL, PLAYER_ROW, WAVE_COUNT};

const STEP_TIME: Duration = Duration::from_millis(420);
const CHECKPOINT_TIME: Duration = Duration::from_secs(15);
const MAX_SAVE_BYTES: u64 = 16 * 1024;
const X_FIRST: i32 = 116;
const X_STEP: i32 = 86;
const Y_FIRST: i32 = 334;
const Y_STEP: i32 = 55;

const SCOUT: &[&str] = &[
    "##.......##",
    ".###...###.",
    "..#######..",
    "...##.##...",
    "...#####...",
    "....###....",
    ".....#.....",
];
const GUARD: &[&str] = &[
    "##.......##",
    "#.##...##.#",
    "##.##.##.##",
    "###########",
    "...##.##...",
    "..###.###..",
    "....#.#....",
];
const FLAGSHIP: &[&str] = &[
    "......#......",
    "..#..###..#..",
    ".##.#####.##.",
    "###.##.##.###",
    "#############",
    "#.#########.#",
    "#.#.#####.#.#",
    "....#...#....",
];
const PILOT: &[&str] = &[
    ".....#.....",
    "....###....",
    "....#.#....",
    "#..##.##..#",
    "###########",
    "###.###.###",
    ".#...#...#.",
];

pub struct View {
    game: Game,
    save_path: Option<PathBuf>,
    next_tick: Option<Instant>,
    last_save: Option<Instant>,
    save_error: bool,
}

impl View {
    pub fn open(path: PathBuf) -> Self {
        let game = File::open(&path)
            .ok()
            .and_then(|file| {
                let mut bytes = Vec::new();
                file.take(MAX_SAVE_BYTES + 1).read_to_end(&mut bytes).ok()?;
                (bytes.len() as u64 <= MAX_SAVE_BYTES).then_some(bytes)
            })
            .and_then(|bytes| serde_json::from_slice::<Game>(&bytes).ok())
            .and_then(Game::validated)
            .unwrap_or_default();
        Self {
            game,
            save_path: Some(path),
            ..Self::preview()
        }
    }

    pub fn preview() -> Self {
        Self {
            game: Game::default(),
            save_path: None,
            next_tick: None,
            last_save: None,
            save_error: false,
        }
    }

    fn flying(&self) -> bool {
        self.next_tick.is_some() && self.game.status() == Status::Flying
    }

    fn steer(&mut self, column: u8) -> Change {
        if self.flying() && self.game.steer(column) {
            Change::Partial
        } else {
            Change::None
        }
    }

    fn nudge(&mut self, offset: i32) -> Change {
        let column = (i32::from(self.game.column()) + offset).clamp(0, i32::from(COLUMNS) - 1);
        self.steer(column as u8)
    }

    fn checkpoint(&mut self, now: Instant) {
        self.last_save = Some(now);
        self.save_error = self.save().is_err();
    }

    fn toggle(&mut self, now: Instant) -> Change {
        if self.next_tick.take().is_none() {
            self.game.start();
            self.next_tick = Some(now + STEP_TIME);
        }
        self.checkpoint(now);
        Change::Partial
    }

    fn tap(&mut self, x: f32, y: f32) -> Change {
        let w = primitive_app::WIDTH as i32;
        if back_button(w).contains(x, y) {
            return Change::Exit;
        }
        let [left, action, right] = controls(w);
        if action.contains(x, y) {
            return self.toggle(Instant::now());
        }
        if left.contains(x, y) {
            return self.nudge(-1);
        }
        if right.contains(x, y) {
            return self.nudge(1);
        }
        let field = board(w);
        if field.contains(x, y) {
            let canonical_x = f64::from(x) * 1264.0 / f64::from(w);
            let column = ((canonical_x - f64::from(X_FIRST)) / f64::from(X_STEP)).round() as i32;
            return self.steer(column.clamp(0, i32::from(COLUMNS) - 1) as u8);
        }
        Change::None
    }

    fn draw_hud(&self, canvas: &mut Canvas, w: i32) {
        let mission = format!("{} / {}", self.game.wave_number(), WAVE_COUNT);
        let score = self.game.score().to_string();
        let hull = format!("{} / {}", self.game.hull(), MAX_HULL);
        for (x, label, value) in [
            (40, "Wave", mission.as_str()),
            (438, "Hull", hull.as_str()),
            (836, "Score", score.as_str()),
        ] {
            canvas.card(screen_rect(w, x, 162, 388, 110));
            canvas.text(screen_rect(w, x + 18, 170, 352, 34), label, CAPTION);
            canvas.text(
                screen_rect(w, x + 18, 209, 352, 48),
                value,
                TextStyle::new(35).bold(),
            );
        }
    }

    fn draw_board(&self, canvas: &mut Canvas, w: i32) {
        canvas.panel(board(w), 22, 2, BLACK);
        for &(x, y, size) in STARS {
            canvas.fill(screen_rect(w, x, y, size, size), 145);
        }
        for column in 0..COLUMNS {
            let x = X_FIRST + i32::from(column) * X_STEP;
            canvas.fill(screen_rect(w, x - 10, 1198, 20, 3), 145);
        }
        for &enemy in self.game.fleet() {
            draw_enemy(canvas, w, enemy);
        }
        for &shot in self.game.friendly() {
            let (x, y) = center(shot);
            canvas.fill(screen_rect(w, x - 3, y - 13, 6, 26), PAPER);
        }
        for &shot in self.game.hostile() {
            let (x, y) = center(shot);
            canvas.fill(screen_rect(w, x - 2, y - 12, 4, 13), PAPER);
            canvas.fill(screen_rect(w, x - 8, y + 1, 16, 8), PAPER);
        }
        if self.game.hull() > 0 {
            let (x, y) = center(Point {
                column: self.game.column(),
                row: PLAYER_ROW,
            });
            sprite(canvas, w, x, y, PILOT, 6);
        }

        let message = match self.game.status() {
            Status::Ready => Some(("Ready to launch", "Destroy the fleet and its flagship.")),
            Status::Flying if !self.flying() => Some(("Paused", "Tap Resume to continue.")),
            Status::WaveClear if self.next_tick.is_none() => {
                Some(("Paused", "Tap Resume for the next wave."))
            }
            Status::WaveClear => Some(("Wave clear", "Next wave approaching.")),
            Status::Won => Some(("Mission complete", "Flagship destroyed.")),
            Status::Lost => Some(("Mission ended", "Tap New mission to try again.")),
            Status::Flying => None,
        };
        if let Some((title, detail)) = message {
            canvas.panel(screen_rect(w, 296, 651, 672, 162), 20, 2, PAPER);
            canvas.text(
                screen_rect(w, 316, 675, 632, 61),
                title,
                TextStyle::new(46).bold().center(),
            );
            canvas.text(screen_rect(w, 316, 743, 632, 44), detail, CAPTION.center());
        }
    }

    fn draw_controls(&self, canvas: &mut Canvas, w: i32) {
        let [left, action, right] = controls(w);
        let label = match self.game.status() {
            Status::Ready => "Start",
            Status::Won | Status::Lost => "New mission",
            Status::Flying | Status::WaveClear if self.next_tick.is_some() => "Pause",
            Status::Flying | Status::WaveClear => "Resume",
        };
        for (column, bounds, label) in [(0, left, "Left"), (COLUMNS - 1, right, "Right")] {
            if self.flying() && self.game.column() != column {
                canvas.button(bounds, label, false, false);
            }
        }
        canvas.button(action, label, !self.flying(), false);

        let hint = if self.save_error {
            "Could not save mission progress."
        } else {
            "Tap or swipe the field to steer. Your ship fires automatically."
        };
        canvas.text(screen_rect(w, 40, 1426, 1184, 49), hint, CAPTION.center());
        canvas.text(
            screen_rect(w, 40, 1503, 820, 66),
            self.game.wave_name(),
            BODY.bold(),
        );
        let best = format!("Best {}", self.game.best());
        canvas.text(screen_rect(w, 884, 1507, 340, 56), &best, CAPTION.right());
    }
}

impl App for View {
    fn draw(&self, canvas: &mut Canvas) {
        let width = canvas.bounds().w;
        header(canvas, "Starship", "Exit");
        self.draw_hud(canvas, width);
        self.draw_board(canvas, width);
        self.draw_controls(canvas, width);
    }

    fn input(&mut self, input: Input) -> Change {
        match input {
            Input::Tap(x, y) => self.tap(x, y),
            Input::Swipe { dx, dy, start_y }
                if (board(primitive_app::WIDTH as i32).y as f32
                    ..board(primitive_app::WIDTH as i32).bottom() as f32)
                    .contains(&start_y)
                    && dx.abs() > dy.abs() * 1.5
                    && dx.abs() >= 48.0 =>
            {
                let steps = (dx.abs() / X_STEP as f32)
                    .round()
                    .clamp(1.0, f32::from(COLUMNS)) as i32;
                self.nudge(steps * dx.signum() as i32)
            }
            Input::PageBack => self.nudge(-1),
            Input::PageForward => self.nudge(1),
            Input::Sleep => {
                self.next_tick = None;
                Change::None
            }
            _ => Change::None,
        }
    }

    fn deadline(&self) -> Option<Instant> {
        self.next_tick
    }

    fn tick(&mut self, now: Instant) -> Change {
        if self.next_tick.is_none_or(|deadline| now < deadline) {
            return Change::None;
        }
        let old_status = self.game.status();
        let changed = self.game.advance();
        let new_status = self.game.status();
        self.next_tick = match new_status {
            Status::Flying | Status::WaveClear => Some(now + STEP_TIME),
            _ => None,
        };
        if old_status != new_status
            || self
                .last_save
                .is_none_or(|last| now.saturating_duration_since(last) >= CHECKPOINT_TIME)
        {
            self.checkpoint(now);
        }
        if changed {
            Change::Partial
        } else {
            Change::None
        }
    }

    fn save(&mut self) -> io::Result<()> {
        if let Some(path) = &self.save_path {
            primitive_app::save(path, &serde_json::to_vec(&self.game)?)?;
        }
        Ok(())
    }

    fn storage_returned(&mut self) -> io::Result<()> {
        self.next_tick = None;
        let saved = self.save();
        self.save_error = saved.is_err();
        saved
    }
}

fn draw_enemy(canvas: &mut Canvas, width: i32, enemy: Enemy) {
    let (x, y) = center(enemy.point);
    let (shape, pixel) = match enemy.ship {
        Ship::Scout => (SCOUT, 4),
        Ship::Guard => (GUARD, 5),
        Ship::Flagship => (FLAGSHIP, 6),
    };
    sprite(canvas, width, x, y, shape, pixel);
    if enemy.ship != Ship::Scout {
        let bar = screen_rect(width, x - 28, y + 29, 56, 5);
        canvas.fill(bar, 104);
        let max = i32::from(enemy.ship.max_health());
        canvas.fill(
            screen_rect(width, x - 28, y + 29, 56 * i32::from(enemy.health) / max, 5),
            PAPER,
        );
    }
}

fn board(w: i32) -> Rect {
    screen_rect(w, 40, 294, 1184, 934)
}

fn controls(w: i32) -> [Rect; 3] {
    [
        screen_rect(w, 40, 1283, 306, 112),
        screen_rect(w, 370, 1283, 524, 112),
        screen_rect(w, 918, 1283, 306, 112),
    ]
}

fn center(point: Point) -> (i32, i32) {
    (
        X_FIRST + i32::from(point.column) * X_STEP,
        Y_FIRST + i32::from(point.row) * Y_STEP,
    )
}

fn sprite(canvas: &mut Canvas, width: i32, x: i32, y: i32, rows: &[&str], pixel: i32) {
    let start_x = x - rows[0].len() as i32 * pixel / 2;
    let start_y = y - rows.len() as i32 * pixel / 2;
    for (row, line) in rows.iter().enumerate() {
        for (column, marker) in line.bytes().enumerate() {
            if marker == b'#' {
                canvas.fill(
                    screen_rect(
                        width,
                        start_x + column as i32 * pixel,
                        start_y + row as i32 * pixel,
                        pixel,
                        pixel,
                    ),
                    PAPER,
                );
            }
        }
    }
}

const STARS: &[(i32, i32, i32)] = &[
    (88, 404, 6),
    (142, 481, 4),
    (297, 414, 5),
    (377, 494, 6),
    (96, 650, 5),
    (149, 779, 4),
    (365, 696, 6),
    (330, 865, 5),
    (92, 938, 4),
    (398, 989, 4),
    (228, 1124, 5),
    (109, 1053, 4),
    (504, 454, 5),
    (740, 448, 6),
    (546, 658, 5),
    (779, 605, 4),
    (508, 783, 5),
    (771, 875, 5),
    (527, 944, 5),
    (753, 934, 4),
    (654, 1092, 4),
    (575, 1034, 5),
    (812, 1116, 6),
    (337, 1072, 4),
    (884, 449, 5),
    (1144, 421, 6),
    (934, 598, 4),
    (1150, 643, 4),
    (895, 714, 5),
    (1148, 795, 5),
    (924, 877, 6),
    (1181, 963, 5),
    (956, 1074, 4),
    (1174, 1127, 5),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sleep_and_usb_leave_the_mission_paused() {
        let mut app = View::preview();
        let now = Instant::now();
        app.toggle(now);
        let first = app.deadline().unwrap();
        assert_eq!(app.tick(first - Duration::from_millis(1)), Change::None);
        assert_eq!(app.tick(first + Duration::from_secs(30)), Change::Partial);
        assert_eq!(app.game.friendly()[0].row, PLAYER_ROW - 3);
        let saved = app.game.clone();
        assert_eq!(app.input(Input::Sleep), Change::None);
        assert_eq!(app.input(Input::Wake), Change::None);
        assert!(app.deadline().is_none());
        assert_eq!(app.tick(first + Duration::from_secs(300)), Change::None);
        assert_eq!(app.game, saved);
        app.toggle(now);
        app.storage_returned().unwrap();
        assert!(app.deadline().is_none());
        assert_eq!(app.game, saved);
    }

    #[test]
    fn reopening_loads_the_arcade_mission_paused_and_leaves_legacy_data() {
        let dir = std::env::temp_dir().join(format!("starship-preview-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let legacy_path = dir.join("starship.json");
        let legacy_data = b"turn-based save";
        std::fs::write(&legacy_path, legacy_data).unwrap();
        let path = dir.join("starship-arcade.json");

        let mut app = View::open(path.clone());
        app.toggle(Instant::now());
        assert_eq!(app.steer(4), Change::Partial);
        let mut lifecycle = primitive_app::Lifecycle::default();
        assert_eq!(
            lifecycle.input(&mut app, Input::Tap(1120.0, 92.0), |error| panic!(
                "{error}"
            )),
            Change::Exit
        );

        let reopened = View::open(path);
        assert_eq!(reopened.game, app.game);
        assert_eq!(reopened.deadline(), None);
        assert_eq!(std::fs::read(&legacy_path).unwrap(), legacy_data);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn touch_steers_with_no_timer_step_and_done_exits() {
        let mut app = View::preview();
        assert_eq!(app.tap(633.0, 1341.0), Change::Partial);
        assert_eq!(app.tap(202.0, 1100.0), Change::Partial);
        let deadline = app.deadline();
        assert_eq!(app.game.column(), 1);
        assert_eq!(
            app.input(Input::Swipe {
                dx: 260.0,
                dy: 25.0,
                start_y: 1010.0
            }),
            Change::Partial
        );
        assert_eq!(app.game.column(), 4);
        assert_eq!(app.deadline(), deadline);
        assert_eq!(app.tap(1120.0, 92.0), Change::Exit);
    }
}
