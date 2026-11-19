use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use primitive_app::{App, Change};
use primitive_ui::{
    back_button, header, screen_rect, Canvas, Input, Rect, TextStyle, BLACK, BODY, CAPTION, PAPER,
};
use starship_core::{Event, Game, Order, Status, MAX_HULL, WAVE_COUNT};

const ORDERS: [Order; 4] = [Order::Left, Order::Right, Order::Fire, Order::Shield];
const SCOUT: &[&str] = &[
    "##.......##",
    ".###...###.",
    "..#######..",
    "...##.##...",
    "...#####...",
    "....###....",
    ".....#.....",
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
    pub game: Game,
    save_path: Option<PathBuf>,
    confirm: bool,
    save_error: bool,
}

impl View {
    pub fn open(path: PathBuf) -> Self {
        let game = File::open(&path)
            .ok()
            .and_then(|file| {
                let mut bytes = Vec::new();
                file.take(4097).read_to_end(&mut bytes).ok()?;
                (bytes.len() <= 4096).then_some(bytes)
            })
            .and_then(|bytes| serde_json::from_slice::<Game>(&bytes).ok())
            .and_then(Game::validated)
            .unwrap_or_default();
        Self {
            game,
            save_path: Some(path),
            confirm: false,
            save_error: false,
        }
    }

    pub fn preview() -> Self {
        Self {
            game: Game::default(),
            save_path: None,
            confirm: false,
            save_error: false,
        }
    }

    fn save_turn(&mut self) {
        self.save_error = self.save().is_err();
    }

    fn apply_order(&mut self, order: Order) -> Change {
        if self.game.order(order) {
            self.save_turn();
            Change::Partial
        } else {
            Change::None
        }
    }

    fn tap(&mut self, width: i32, x: f32, y: f32) -> Change {
        if self.confirm {
            let [cancel, confirm] = confirm_buttons(width);
            if confirm.contains(x, y) {
                self.game.restart();
                self.save_turn();
            } else if !cancel.contains(x, y) && dialog(width).contains(x, y) {
                return Change::None;
            }
            self.confirm = false;
            return Change::Full;
        }
        if back_button(width).contains(x, y) {
            return Change::Exit;
        }
        if new_button(width).contains(x, y) {
            if self.game.status() == Status::Active && self.game.event() != Event::Ready {
                self.confirm = true;
            } else {
                self.game.restart();
                self.save_turn();
            }
            return Change::Full;
        }
        match ORDERS
            .into_iter()
            .enumerate()
            .find(|(i, _)| order_button(width, *i).contains(x, y))
        {
            Some((_, order)) => self.apply_order(order),
            None => Change::None,
        }
    }

    fn draw_status(&self, canvas: &mut Canvas, w: i32) {
        let wave = format!("{} of {}", self.game.wave_number(), WAVE_COUNT);
        let score = self.game.points().to_string();
        let remaining = self.game.turns_left().to_string();
        for (x, label, value) in [
            (40, "Wave", wave.as_str()),
            (438, "Turns left", remaining.as_str()),
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
        canvas.panel(screen_rect(w, 40, 294, 1184, 780), 22, 2, BLACK);
        for &(x, y, size) in STARS {
            canvas.fill(screen_rect(w, x, y, size, size), PAPER);
        }
        for x in [434, 829] {
            for y in (355..1040).step_by(34) {
                canvas.fill(screen_rect(w, x, y, 2, 13), 145);
            }
        }

        for (i, hits) in self.game.targets().into_iter().enumerate() {
            let center = lane_center(i);
            canvas.text(
                screen_rect(w, center - 152, 322, 304, 45),
                ["Left", "Centre", "Right"][i],
                CAPTION.center().ink(PAPER),
            );
            if hits > 0 {
                let flagship = self.game.wave_number() == WAVE_COUNT;
                sprite(
                    canvas,
                    w,
                    center,
                    537,
                    if flagship { FLAGSHIP } else { SCOUT },
                    if flagship { 19 } else { 15 },
                );
                for hit in 0..hits {
                    let x = center - (hits as i32 * 28 - 8) / 2 + hit as i32 * 28;
                    canvas.fill(screen_rect(w, x, 645, 20, 10), PAPER);
                }
            }
            if self.game.warning() == Some(i) {
                canvas.panel(screen_rect(w, center - 126, 715, 252, 63), 14, 2, PAPER);
                canvas.text(
                    screen_rect(w, center - 126, 728, 252, 38),
                    "Incoming fire",
                    TextStyle::new(28).bold().center(),
                );
                for y in [808, 837, 866] {
                    canvas.fill(screen_rect(w, center - 3, y, 6, 15), PAPER);
                }
            }
        }
        if self.game.hull() > 0 {
            sprite(canvas, w, lane_center(self.game.lane()), 941, PILOT, 14);
        }
        let hull = self.game.hull();
        for slot in 0..MAX_HULL {
            let marker = screen_rect(w, 460 + i32::from(slot) * 52, 996, 44, 16);
            canvas.fill(marker, PAPER);
            if slot >= hull {
                canvas.fill(marker.inset(canvas.px(3)), BLACK);
            }
        }
        canvas.text(
            screen_rect(w, 627, 980, 156, 48),
            "Hull",
            CAPTION.ink(PAPER),
        );
    }

    fn draw_controls(&self, canvas: &mut Canvas, w: i32) {
        let hint = match self.game.status() {
            Status::Won => "Mission complete",
            Status::Lost => "Mission ended",
            Status::Active => self.game.wave_name(),
        };
        canvas.text(
            screen_rect(w, 40, 1094, 1184, 54),
            hint,
            BODY.bold().center(),
        );
        let message = if self.save_error {
            "Could not save this turn."
        } else {
            self.game.event().text()
        };
        canvas.text(
            screen_rect(w, 40, 1155, 1184, 52),
            message,
            CAPTION.center(),
        );

        let shield = format!("Shield ({})", self.game.shields());
        for (index, (order, label)) in ORDERS
            .into_iter()
            .zip(["Move left", "Move right", "Fire", &shield])
            .enumerate()
        {
            let bounds = order_button(w, index);
            if self.game.can_order(order) {
                canvas.button(bounds, label, order == Order::Fire, false);
            } else {
                canvas.panel(bounds, 16, 2, PAPER);
                let top = canvas.px(29);
                let text = Rect::new(bounds.x, bounds.y + top, bounds.w, bounds.h - top);
                canvas.text(text, label, TextStyle::new(36).center().ink(145));
            }
        }
        canvas.text(
            screen_rect(w, 40, 1370, 1184, 48),
            "Fire hits the target in your lane. Each action takes one turn.",
            CAPTION.center(),
        );
        canvas.text(
            screen_rect(w, 40, 1535, 430, 48),
            &format!("Best score {}", self.game.best()),
            CAPTION,
        );
        canvas.button(
            new_button(w),
            "New mission",
            self.game.status() != Status::Active,
            false,
        );
    }
}

impl App for View {
    fn draw(&self, canvas: &mut Canvas) {
        let width = canvas.bounds().w;
        header(canvas, "Starship", "Done");
        self.draw_status(canvas, width);
        self.draw_board(canvas, width);
        self.draw_controls(canvas, width);
        if self.confirm {
            canvas.popup(dialog(width));
            canvas.text(
                screen_rect(width, 246, 620, 772, 70),
                "Start a new mission?",
                TextStyle::new(50).bold(),
            );
            canvas.text(
                screen_rect(width, 246, 711, 772, 62),
                "Your best score stays saved.",
                CAPTION,
            );
            let [cancel, confirm] = confirm_buttons(width);
            canvas.button(cancel, "Cancel", false, false);
            canvas.button(confirm, "New mission", true, false);
        }
    }

    fn input(&mut self, input: Input) -> Change {
        match input {
            Input::Tap(x, y) => self.tap(primitive_app::WIDTH as i32, x, y),
            Input::PageForward if !self.confirm => self.apply_order(Order::Fire),
            Input::PageBack if self.confirm => {
                self.confirm = false;
                Change::Full
            }
            Input::PageBack => Change::Exit,
            _ => Change::None,
        }
    }

    fn save(&mut self) -> io::Result<()> {
        if let Some(path) = &self.save_path {
            primitive_app::save(path, &serde_json::to_vec(&self.game)?)?;
        }
        Ok(())
    }
}

fn lane_center(lane: usize) -> i32 {
    235 + lane as i32 * 397
}

fn order_button(w: i32, index: usize) -> Rect {
    screen_rect(w, 40 + index as i32 * 299, 1228, 287, 110)
}

fn new_button(w: i32) -> Rect {
    screen_rect(w, 800, 1498, 424, 110)
}

fn dialog(w: i32) -> Rect {
    screen_rect(w, 194, 570, 876, 382)
}

fn confirm_buttons(w: i32) -> [Rect; 2] {
    [
        screen_rect(w, 246, 810, 358, 100),
        screen_rect(w, 660, 810, 358, 100),
    ]
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
    (504, 454, 5),
    (740, 448, 6),
    (546, 658, 5),
    (779, 605, 4),
    (508, 783, 5),
    (771, 875, 5),
    (527, 944, 5),
    (753, 934, 4),
    (884, 449, 5),
    (1144, 421, 6),
    (934, 598, 4),
    (1150, 643, 4),
    (895, 714, 5),
    (1148, 795, 5),
    (924, 877, 6),
    (1181, 963, 5),
];
