use macroquad::prelude::*;
// Fix imports to avoid ambiguity between macroquad's rand and the rand crate
use ::rand::prelude::*;
use ::rand::thread_rng;

// Game settings
const BLOCK_SIZE: f32 = 30.0;
const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;
const BOARD_POS_X: f32 = 100.0;
const BOARD_POS_Y: f32 = 50.0;
const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 700.0;
const UPDATE_INTERVAL: f64 = 0.5; // seconds between automatic piece drops

// Tetromino definitions
const TETROMINOS: [&[&[(i32, i32)]]; 7] = [
    // I-piece
    &[
        &[(0, 0), (0, 1), (0, 2), (0, 3)],
        &[(0, 0), (1, 0), (2, 0), (3, 0)],
    ],
    // J-piece
    &[
        &[(0, 0), (0, 1), (0, 2), (-1, 2)],
        &[(0, 0), (1, 0), (2, 0), (2, 1)],
        &[(0, 0), (0, 1), (0, 2), (1, 0)],
        &[(0, 0), (0, 1), (1, 1), (2, 1)],
    ],
    // L-piece
    &[
        &[(0, 0), (0, 1), (0, 2), (1, 2)],
        &[(0, 0), (0, 1), (1, 0), (2, 0)],
        &[(0, 0), (1, 0), (1, 1), (1, 2)],
        &[(0, 1), (1, 1), (2, 1), (2, 0)],
    ],
    // O-piece
    &[&[(0, 0), (0, 1), (1, 0), (1, 1)]],
    // S-piece
    &[
        &[(0, 0), (0, 1), (1, 1), (1, 2)],
        &[(0, 1), (1, 1), (1, 0), (2, 0)],
    ],
    // T-piece
    &[
        &[(0, 1), (1, 0), (1, 1), (1, 2)],
        &[(0, 0), (1, 0), (2, 0), (1, 1)],
        &[(0, 0), (0, 1), (0, 2), (1, 1)],
        &[(0, 1), (1, 1), (2, 1), (1, 0)],
    ],
    // Z-piece
    &[
        &[(0, 1), (0, 2), (1, 0), (1, 1)],
        &[(0, 0), (1, 0), (1, 1), (2, 1)],
    ],
];

// Colors for each tetromino
const TETROMINO_COLORS: [Color; 7] = [
    SKYBLUE,  // I-piece
    DARKBLUE, // J-piece
    ORANGE,   // L-piece
    YELLOW,   // O-piece
    GREEN,    // S-piece
    PURPLE,   // T-piece
    RED,      // Z-piece
];

struct Tetromino {
    shapes: &'static [&'static [(i32, i32)]],
    current_rotation: usize,
    position: (i32, i32), // (x, y)
    color: Color,
    type_index: usize,
}

impl Tetromino {
    fn new(tetromino_type: usize) -> Self {
        Self {
            shapes: TETROMINOS[tetromino_type],
            current_rotation: 0,
            position: (BOARD_WIDTH as i32 / 2 - 1, 0),
            color: TETROMINO_COLORS[tetromino_type],
            type_index: tetromino_type,
        }
    }

    fn get_current_shape(&self) -> &'static [(i32, i32)] {
        self.shapes[self.current_rotation]
    }

    fn rotate(&mut self, board: &Board) {
        let next_rotation = (self.current_rotation + 1) % self.shapes.len();
        let next_shape = self.shapes[next_rotation];

        // Check if the rotation is valid
        if !self.collides(self.position, next_shape, board) {
            self.current_rotation = next_rotation;
        }
    }

    fn move_left(&mut self, board: &Board) {
        let next_position = (self.position.0 - 1, self.position.1);
        if !self.collides(next_position, self.get_current_shape(), board) {
            self.position = next_position;
        }
    }

    fn move_right(&mut self, board: &Board) {
        let next_position = (self.position.0 + 1, self.position.1);
        if !self.collides(next_position, self.get_current_shape(), board) {
            self.position = next_position;
        }
    }

    fn move_down(&mut self, board: &Board) -> bool {
        let next_position = (self.position.0, self.position.1 + 1);
        if !self.collides(next_position, self.get_current_shape(), board) {
            self.position = next_position;
            false // Not landed
        } else {
            true // Landed
        }
    }

    fn hard_drop(&mut self, board: &Board) {
        while !self.move_down(board) {}
    }

    fn collides(&self, position: (i32, i32), shape: &[(i32, i32)], board: &Board) -> bool {
        for &(dx, dy) in shape {
            let x = position.0 + dx;
            let y = position.1 + dy;

            // Check bounds
            if x < 0 || x >= BOARD_WIDTH as i32 || y < 0 || y >= BOARD_HEIGHT as i32 {
                return true;
            }

            // Check collision with placed blocks
            if board.grid[y as usize][x as usize].is_some() {
                return true;
            }
        }
        false
    }

    fn draw(&self) {
        for &(dx, dy) in self.get_current_shape() {
            let x = self.position.0 + dx;
            let y = self.position.1 + dy;

            let screen_x = BOARD_POS_X + (x as f32 * BLOCK_SIZE);
            let screen_y = BOARD_POS_Y + (y as f32 * BLOCK_SIZE);

            draw_rectangle(screen_x, screen_y, BLOCK_SIZE, BLOCK_SIZE, self.color);
            draw_rectangle_lines(screen_x, screen_y, BLOCK_SIZE, BLOCK_SIZE, 1.0, BLACK);
        }
    }

    fn draw_preview(&self, x: f32, y: f32, scale: f32) {
        // Find the bounding box of the piece
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;

        for &(dx, dy) in self.get_current_shape() {
            min_x = min_x.min(dx);
            min_y = min_y.min(dy);
            max_x = max_x.max(dx);
            max_y = max_y.max(dy);
        }

        let width = (max_x - min_x + 1) as f32;
        let height = (max_y - min_y + 1) as f32;
        let block_size = scale.min(80.0 / width.max(height));

        // Center the preview
        let center_x = x + (4.0 * block_size) / 2.0;
        let center_y = y + (4.0 * block_size) / 2.0;

        for &(dx, dy) in self.get_current_shape() {
            let px = center_x + ((dx - min_x) as f32 - width / 2.0 + 0.5) * block_size;
            let py = center_y + ((dy - min_y) as f32 - height / 2.0 + 0.5) * block_size;

            draw_rectangle(px, py, block_size, block_size, self.color);
            draw_rectangle_lines(px, py, block_size, block_size, 1.0, BLACK);
        }
    }
}

struct Board {
    grid: Vec<Vec<Option<Color>>>,
}

impl Board {
    fn new() -> Self {
        Self {
            grid: vec![vec![None; BOARD_WIDTH]; BOARD_HEIGHT],
        }
    }

    fn place_tetromino(&mut self, tetromino: &Tetromino) {
        let (pos_x, pos_y) = tetromino.position;
        for &(dx, dy) in tetromino.get_current_shape() {
            let x = (pos_x + dx) as usize;
            let y = (pos_y + dy) as usize;
            if y < BOARD_HEIGHT && x < BOARD_WIDTH {
                self.grid[y][x] = Some(tetromino.color);
            }
        }
    }

    fn clear_lines(&mut self) -> usize {
        let mut lines_cleared = 0;

        // Check each row from bottom to top
        let mut row = BOARD_HEIGHT;
        while row > 0 {
            row -= 1;

            // Check if the row is full
            if self.grid[row].iter().all(|cell| cell.is_some()) {
                // Move all rows above down by one
                for y in (1..=row).rev() {
                    self.grid[y] = self.grid[y - 1].clone();
                }
                // Clear the top row
                self.grid[0] = vec![None; BOARD_WIDTH];

                lines_cleared += 1;
                // Don't move the row counter as we need to check the same row again
                row += 1;
            }
        }

        lines_cleared
    }

    fn draw(&self) {
        // Draw the game board background
        let board_width = BOARD_WIDTH as f32 * BLOCK_SIZE;
        let board_height = BOARD_HEIGHT as f32 * BLOCK_SIZE;
        draw_rectangle(
            BOARD_POS_X,
            BOARD_POS_Y,
            board_width,
            board_height,
            LIGHTGRAY,
        );

        // Draw grid lines
        for i in 0..=BOARD_WIDTH {
            let x = BOARD_POS_X + (i as f32 * BLOCK_SIZE);
            draw_line(x, BOARD_POS_Y, x, BOARD_POS_Y + board_height, 1.0, DARKGRAY);
        }

        for i in 0..=BOARD_HEIGHT {
            let y = BOARD_POS_Y + (i as f32 * BLOCK_SIZE);
            draw_line(BOARD_POS_X, y, BOARD_POS_X + board_width, y, 1.0, DARKGRAY);
        }

        // Draw placed blocks
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                if let Some(color) = self.grid[y][x] {
                    let screen_x = BOARD_POS_X + (x as f32 * BLOCK_SIZE);
                    let screen_y = BOARD_POS_Y + (y as f32 * BLOCK_SIZE);

                    draw_rectangle(screen_x, screen_y, BLOCK_SIZE, BLOCK_SIZE, color);
                    draw_rectangle_lines(screen_x, screen_y, BLOCK_SIZE, BLOCK_SIZE, 1.0, BLACK);
                }
            }
        }
    }
}

struct Game {
    board: Board,
    current_tetromino: Tetromino,
    next_tetromino: Tetromino,
    score: usize,
    level: usize,
    lines_cleared: usize,
    game_over: bool,
    last_update_time: f64,
    paused: bool,
}

impl Game {
    fn new() -> Self {
        let mut rng = thread_rng();
        let first_type = *((0..7).collect::<Vec<_>>().choose(&mut rng).unwrap());
        let next_type = *((0..7).collect::<Vec<_>>().choose(&mut rng).unwrap());

        Self {
            board: Board::new(),
            current_tetromino: Tetromino::new(first_type),
            next_tetromino: Tetromino::new(next_type),
            score: 0,
            level: 1,
            lines_cleared: 0,
            game_over: false,
            last_update_time: get_time(),
            paused: false,
        }
    }

    fn reset(&mut self) {
        let mut rng = thread_rng();
        let first_type = *((0..7).collect::<Vec<_>>().choose(&mut rng).unwrap());
        let next_type = *((0..7).collect::<Vec<_>>().choose(&mut rng).unwrap());

        self.board = Board::new();
        self.current_tetromino = Tetromino::new(first_type);
        self.next_tetromino = Tetromino::new(next_type);
        self.score = 0;
        self.level = 1;
        self.lines_cleared = 0;
        self.game_over = false;
        self.last_update_time = get_time();
        self.paused = false;
    }

    fn generate_next_tetromino(&mut self) {
        let mut rng = thread_rng();

        // Current tetromino becomes the next tetromino
        self.current_tetromino = Tetromino::new(self.next_tetromino.type_index);

        // Generate next tetromino
        let next_type = *((0..7).collect::<Vec<_>>().choose(&mut rng).unwrap());
        self.next_tetromino = Tetromino::new(next_type);

        // Check if the new tetromino collides immediately (game over)
        if self.current_tetromino.collides(
            self.current_tetromino.position,
            self.current_tetromino.get_current_shape(),
            &self.board,
        ) {
            self.game_over = true;
        }
    }

    fn update(&mut self) -> bool {
        if self.game_over || self.paused {
            return false;
        }

        // Handle automatic drop based on level
        let current_time = get_time();
        let drop_interval = UPDATE_INTERVAL / (1.0 + (self.level as f64 * 0.2));

        if current_time - self.last_update_time >= drop_interval {
            self.last_update_time = current_time;

            // Move current tetromino down
            let landed = self.current_tetromino.move_down(&self.board);

            if landed {
                // Place the tetromino on the board
                self.board.place_tetromino(&self.current_tetromino);

                // Clear completed lines and update score
                let lines = self.board.clear_lines();
                self.lines_cleared += lines;

                // Update score based on lines cleared
                self.score += match lines {
                    1 => 100 * self.level,
                    2 => 300 * self.level,
                    3 => 500 * self.level,
                    4 => 800 * self.level, // Tetris!
                    _ => 0,
                };

                // Update level
                self.level = (self.lines_cleared / 10) + 1;

                // Generate next tetromino
                self.generate_next_tetromino();

                return true; // Piece has landed
            }
        }

        false // Piece is still falling
    }

    fn handle_input(&mut self) {
        if self.game_over {
            // Only check for restart in game over state
            if is_key_pressed(KeyCode::R) {
                self.reset();
            }
            return;
        }

        // Toggle pause
        if is_key_pressed(KeyCode::P) {
            self.paused = !self.paused;
            return;
        }

        if self.paused {
            return;
        }

        // Movement controls
        if is_key_pressed(KeyCode::Left) || is_key_down(KeyCode::Left) && (get_time() % 0.1 < 0.05)
        {
            self.current_tetromino.move_left(&self.board);
        }

        if is_key_pressed(KeyCode::Right)
            || is_key_down(KeyCode::Right) && (get_time() % 0.1 < 0.05)
        {
            self.current_tetromino.move_right(&self.board);
        }

        if is_key_pressed(KeyCode::Down) || is_key_down(KeyCode::Down) && (get_time() % 0.1 < 0.05)
        {
            self.current_tetromino.move_down(&self.board);
        }

        if is_key_pressed(KeyCode::Up) {
            self.current_tetromino.rotate(&self.board);
        }

        if is_key_pressed(KeyCode::Space) {
            self.current_tetromino.hard_drop(&self.board);
            // Force an immediate update after hard drop
            self.last_update_time = get_time() - UPDATE_INTERVAL;
        }
    }

    fn draw(&self) {
        clear_background(WHITE);

        // Draw game board and placed blocks
        self.board.draw();

        // Draw current tetromino if game is not over
        if !self.game_over {
            self.current_tetromino.draw();
        }

        // Draw game info
        let info_x = BOARD_POS_X + (BOARD_WIDTH as f32 * BLOCK_SIZE) + 50.0;
        let info_y = BOARD_POS_Y;

        // Next piece preview
        draw_text("NEXT:", info_x, info_y, 30.0, BLACK);
        self.next_tetromino
            .draw_preview(info_x, info_y + 40.0, 20.0);

        // Score
        draw_text(
            &format!("SCORE: {}", self.score),
            info_x,
            info_y + 150.0,
            30.0,
            BLACK,
        );

        // Level
        draw_text(
            &format!("LEVEL: {}", self.level),
            info_x,
            info_y + 190.0,
            30.0,
            BLACK,
        );

        // Lines cleared
        draw_text(
            &format!("LINES: {}", self.lines_cleared),
            info_x,
            info_y + 230.0,
            30.0,
            BLACK,
        );

        // Controls
        draw_text("CONTROLS:", info_x, info_y + 290.0, 24.0, BLACK);
        draw_text("← → : Move", info_x, info_y + 320.0, 20.0, DARKGRAY);
        draw_text("↑ : Rotate", info_x, info_y + 350.0, 20.0, DARKGRAY);
        draw_text("↓ : Soft Drop", info_x, info_y + 380.0, 20.0, DARKGRAY);
        draw_text("SPACE : Hard Drop", info_x, info_y + 410.0, 20.0, DARKGRAY);
        draw_text("P : Pause", info_x, info_y + 440.0, 20.0, DARKGRAY);

        // Game over message
        if self.game_over {
            let x = SCREEN_WIDTH / 2.0 - 150.0;
            let y = SCREEN_HEIGHT / 2.0;

            draw_rectangle(
                x - 20.0,
                y - 50.0,
                340.0,
                100.0,
                Color::new(0.0, 0.0, 0.0, 0.7),
            );
            draw_text("GAME OVER", x, y, 50.0, RED);
            draw_text("Press 'R' to restart", x + 20.0, y + 40.0, 25.0, WHITE);
        }

        // Pause message
        if self.paused {
            let x = SCREEN_WIDTH / 2.0 - 100.0;
            let y = SCREEN_HEIGHT / 2.0;

            draw_rectangle(
                x - 20.0,
                y - 50.0,
                240.0,
                100.0,
                Color::new(0.0, 0.0, 0.0, 0.7),
            );
            draw_text("PAUSED", x, y, 50.0, YELLOW);
            draw_text("Press 'P' to resume", x, y + 40.0, 25.0, WHITE);
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Tetris in Rust".to_owned(),
        window_width: SCREEN_WIDTH as i32,
        window_height: SCREEN_HEIGHT as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();

    loop {
        game.handle_input();
        game.update();
        game.draw();

        next_frame().await;
    }
}
