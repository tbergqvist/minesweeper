use macroquad::prelude::*;

#[macroquad::main("Minesweeper")]
async fn main() {

  loop {
    let fps_text = format!("FPS: {}", macroquad::time::get_fps());
    clear_background(WHITE);

    draw_text(&fps_text, 20.0, 30.0, 30.0, BLACK);

    next_frame().await
  }
}