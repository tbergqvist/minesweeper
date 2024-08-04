#![feature(let_chains)]

use macroquad::{prelude::*, ui::{root_ui, widgets::Button}};
use miniquad::window::{get_window_position, set_window_position, set_window_size};
use ::rand::{thread_rng, Rng};

struct BoardSetting {
  rows: u8,
  cols: u8,
  mines: u8,
}

impl BoardSetting {
  fn new(rows: u8, cols: u8, mines: u8) -> Self {
    Self {
      rows,
      cols,
      mines,
    }
  }
}

#[derive(Clone, PartialEq)]
enum TileType {
  Empty,
  Mine,
  Number(u32),
}

#[derive(Clone, PartialEq)]
enum TileState {
  Hidden,
  Flagged,
  Revealed,
}

#[derive(Clone)]
struct Tile {
  tile_type: TileType,
  state: TileState,
}

type Board = Vec<Vec<Tile>>;

const BUTTON_SIZE: f32 = 26.;
const BUTTON_HALF_SIZE: f32 = BUTTON_SIZE / 2.;
const PADDING: f32 = BUTTON_SIZE + 0.;

fn generate_board(settings: &BoardSetting) -> Board {
  let mut board = vec![vec![
    Tile {
      tile_type: TileType::Empty,
      state: TileState::Hidden,
    }; settings.cols as usize]; settings.rows as usize];

  let mut rng = thread_rng();

  let mut mines_placed = 0;
  while mines_placed < settings.mines {
    let row = rng.gen_range(0..settings.rows);
    let col = rng.gen_range(0..settings.cols);

    let tile = &mut board[row as usize][col as usize];
    if tile.tile_type != TileType::Mine {
      tile.tile_type = TileType::Mine;
      mines_placed += 1;
    }
  }

  for row in 0..settings.rows as usize {
    for col in 0..settings.cols as usize {
      if board[row][col].tile_type == TileType::Mine {
        continue;
      }

      let mine_count = loop_surrounding_tiles(&board, col, row)
        .filter(|(x, y)| board[*y][*x].tile_type == TileType::Mine)
        .count();

      if mine_count > 0 {
        board[row as usize][col as usize].tile_type = TileType::Number(mine_count as u32);
      }
    }
  }

  board
}

fn resize_window(settings: &BoardSetting) {
  let original_screen_size = (screen_width(), screen_height());
  let new_screen_size = (settings.cols as f32 * BUTTON_SIZE, settings.rows as f32 * BUTTON_SIZE);
  let original_window_position = get_window_position();
  let screen_size_diff = (original_screen_size.0 - new_screen_size.0, original_screen_size.1 - new_screen_size.1);
  set_window_size(new_screen_size.0 as u32, new_screen_size.1 as u32);
  set_window_position((original_window_position.0 as f32 + screen_size_diff.0 / 2.) as u32, (original_window_position.1 as f32 + screen_size_diff.1 / 2.) as u32);
}

#[macroquad::main("Minesweeper")]
async fn main() {
  let settings = get_settings().await;
  let mut board = generate_board(&settings);

  resize_window(&settings);

  loop {
    handle_click(&mut board);

    clear_background(WHITE);
    
    let fps_text = format!("FPS: {}", macroquad::time::get_fps());
    draw_text(&fps_text, 20.0, 30.0, 30.0, BLACK);
    
    draw_board(&board);

    next_frame().await
  }
}

fn draw_board(board: &Board) {
  let iter = board.iter()
    .enumerate()
    .flat_map(|(row, tiles)| tiles.iter().enumerate().map(move |(col, tile)| (col, row, tile)));

  for (x, y, tile) in iter {
    let position = Vec2::new(x as f32 * PADDING, y as f32 * PADDING);

    draw_rectangle(position.x, position.y, BUTTON_SIZE, BUTTON_SIZE, 
      match tile.state {
        TileState::Hidden => GRAY,
        TileState::Flagged => RED,
        TileState::Revealed => WHITE,
      }
    );

    if tile.state == TileState::Revealed {
      match &tile.tile_type {
        TileType::Empty => {},
        TileType::Mine => {
          draw_circle(position.x + BUTTON_HALF_SIZE, position.y + BUTTON_HALF_SIZE, 10., RED);
        },
        TileType::Number(number) => {
          draw_text(&number.to_string(), position.x + BUTTON_HALF_SIZE, position.y + BUTTON_HALF_SIZE, 20., BLACK);
        },
      }
    }
  }
}

fn handle_click(board: &mut Board) {
  let left_click = is_mouse_button_released(MouseButton::Left);
  let right_click = is_mouse_button_released(MouseButton::Right);
  if !left_click && !right_click {
    return;
  }

  let (mouse_x, mouse_y) = mouse_position();
  let x = (mouse_x / PADDING) as usize;
  let y = (mouse_y / PADDING) as usize;
  let both_down = (is_mouse_button_down(MouseButton::Left) || is_mouse_button_released(MouseButton::Left)) && (is_mouse_button_down(MouseButton::Right) || is_mouse_button_released(MouseButton::Right));

  if left_click {
    reveal_tile(board, x, y, both_down);
  } else if right_click {
    if let Some(tile) = board.get_mut(y).and_then(|row| row.get_mut(x)) {
      if tile.state == TileState::Revealed {
        reveal_tile(board, x, y, both_down);
      } else {
        tile.state = if tile.state == TileState::Hidden { TileState::Flagged } else { TileState::Hidden };
      }
    }
  }
}

fn reveal_tile(board: &mut Board, x: usize, y: usize, both_down: bool) {
  if let Some(tile) = board.get_mut(y).and_then(|row| row.get_mut(x)) {
    if tile.state == TileState::Flagged {
      return;
    }

    if tile.state == TileState::Revealed {
      if !both_down {
        return;
      }

      if let TileType::Number(surrounding_mines) = tile.tile_type {
        let flags_around = loop_surrounding_tiles(board, x, y).filter(|(x, y)| board[*y][*x].state == TileState::Flagged).count();
        if surrounding_mines <= flags_around as u32 {
          loop_surrounding_tiles(board, x, y)
          .for_each(|(x, y)| reveal_tile(board, x, y, false));
          return;
        }
      }

      return;
    }

    tile.state = TileState::Revealed;

    if tile.tile_type == TileType::Empty {
      loop_surrounding_tiles(board, x, y).for_each(|(x, y)| reveal_tile(board, x, y, false));
    }
  }
}

fn loop_surrounding_tiles(board: &Board, x: usize, y: usize) -> impl Iterator<Item = (usize, usize)>{
  let mut tile_pos = Vec::new();
  let width = board[0].len();
  let height = board.len();
  for i in -1..=1 {
    for j in -1..=1 {
      let new_x = x as i32 + i;
      let new_y = y as i32 + j;

      if new_x >= 0 && new_x < width as i32 && new_y >= 0 && new_y < height as i32 {
        tile_pos.push((new_x as usize, new_y as usize));
      }
    }
  }

  tile_pos.into_iter()
}

fn draw_difficulty_button(label: &str, y_pos: f32) -> bool {
  let button_x = screen_width() / 2. - 50.;
  let button_y = screen_height() / 2. - 125. + y_pos;
  let button_size = Vec2::new(100., 50.);

  Button::new(label)
    .position(Vec2::new(button_x, button_y))
    .size(button_size)
    .ui(&mut root_ui())
}

async fn get_settings() -> BoardSetting {
  loop {
    clear_background(WHITE);
    
    if draw_difficulty_button("Noob", 0.) {
      return BoardSetting::new(9, 9, 10);
    }

    if draw_difficulty_button("Okish", 100.) {
      return BoardSetting::new(16, 16, 40);
    }

    if draw_difficulty_button("Pro", 200.) {
      return BoardSetting::new(16, 30, 99);
    }

    next_frame().await;
  }
}