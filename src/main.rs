use glam::{Mat4, Vec2, Vec3, Vec4};
use minifb::{Key, Window, WindowOptions};
use std::time::Instant;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

const GREEN: u32 = 0x00_ff_00;
const BLACK: u32 = 0x00_00_00;

macro_rules! time_it {
  ($($body:tt)*) => {{
      let start = std::time::Instant::now();
      let result = { $($body)* }; // execute the block
      let elapsed = start.elapsed();
      println!("Time: {} µs", elapsed.as_micros());
      result
  }};
}

const VERTICES: [Vec3; 8] = [
    Vec3::new(-0.5, -0.5, -0.5), // 0
    Vec3::new(0.5, -0.5, -0.5),  // 1
    Vec3::new(0.5, 0.5, -0.5),   // 2
    Vec3::new(-0.5, 0.5, -0.5),  // 3
    Vec3::new(-0.5, -0.5, 0.5),  // 4
    Vec3::new(0.5, -0.5, 0.5),   // 5
    Vec3::new(0.5, 0.5, 0.5),    // 6
    Vec3::new(-0.5, 0.5, 0.5),   // 7
];

const TRIS: [[u32; 3]; 12] = [
    [0, 1, 2],
    [0, 2, 3],
    [4, 6, 5],
    [4, 7, 6],
    [0, 3, 7],
    [0, 7, 4],
    [1, 5, 6],
    [1, 6, 2],
    [0, 4, 5],
    [0, 5, 1],
    [3, 2, 6],
    [3, 6, 7],
];

#[inline]
fn clip_check(v: &Vec4) -> bool {
    let w = v.w;
    if w <= 0.0 {
        return false;
    }
    v.truncate().abs().cmple(Vec3::splat(w)).all()
}

fn ndc_to_screen(v: Vec3) -> Vec3 {
    let x = (v.x + 1.0) * 0.5 * WIDTH as f32;
    let y = (1.0 - v.y) * 0.5 * HEIGHT as f32;
    Vec3::new(x, y, v.z)
}

fn draw_line(a: Vec2, b: Vec2, buffer: &mut [u32], thickness: i32) {
    let dx = b.x - a.x;
    let dy = b.y - a.y;

    let steps = dx.abs().max(dy.abs()) as i32;
    if steps == 0 {
        return;
    }

    let x_inc = dx / steps as f32;
    let y_inc = dy / steps as f32;

    let mut x = a.x;
    let mut y = a.y;

    let half = thickness / 2;

    for _ in 0..=steps {
        let ix = x.round() as i32;
        let iy = y.round() as i32;

        let min_x = (ix - half).max(0) as usize;
        let max_x = (ix + half).min(WIDTH as i32 - 1) as usize;
        let min_y = (iy - half).max(0) as usize;
        let max_y = (iy + half).min(HEIGHT as i32 - 1) as usize;

        for py in min_y..=max_y {
            let row = py * WIDTH;
            buffer[row + min_x..=row + max_x].fill(GREEN);
        }

        x += x_inc;
        y += y_inc;
    }
}

fn draw(buffer: &mut [u32], dt: f32, total_time: f32) {
    println!("dt: {:.2}ms total_time: {:.2}s", dt * 1000.0, total_time);
    let model_mat = Mat4::from_rotation_y(total_time);
    let view_mat = Mat4::look_to_rh((0.0, 0.0, 2.0).into(), (0.0, 0.0, -1.0).into(), Vec3::Y);
    let proj_mat = Mat4::perspective_rh(
        90.0_f32.to_radians(),
        WIDTH as f32 / HEIGHT as f32,
        0.1,
        100.0,
    );

    let mvp_mat: Mat4 = proj_mat * view_mat * model_mat;

    for tri in TRIS.iter() {
        let mut clip = [Vec4::ZERO; 3];
        let mut visible = true;
        for (i, &index) in tri.iter().enumerate() {
            let vertex = VERTICES[index as usize];
            let clip_vertex = mvp_mat * vertex.extend(1.0);

            if !clip_check(&clip_vertex) {
                visible = false;
                break;
            }

            clip[i] = clip_vertex;
        }
        if !visible {
            continue;
        }

        let ndc = [
            clip[0].truncate() / clip[0].w,
            clip[1].truncate() / clip[1].w,
            clip[2].truncate() / clip[2].w,
        ];

        let screen = [
            ndc_to_screen(ndc[0]),
            ndc_to_screen(ndc[1]),
            ndc_to_screen(ndc[2]),
        ];

        draw_line(screen[0].truncate(), screen[1].truncate(), buffer, 2);
        draw_line(screen[1].truncate(), screen[2].truncate(), buffer, 2);
        draw_line(screen[2].truncate(), screen[0].truncate(), buffer, 2);
    }
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window =
        Window::new("Minifb Example", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    window.set_target_fps(usize::MAX);

    let mut last_time = Instant::now();
    let mut total_time: f32 = 0.0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = now.duration_since(last_time).as_secs_f32();
        last_time = now;
        total_time += dt;
        time_it! {
          for i in buffer.iter_mut() {
              *i = BLACK;
          }
          draw(&mut buffer, dt, total_time);
        };
        time_it! {
          window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        };
    }
}
