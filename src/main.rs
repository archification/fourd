use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::rect::{Point};

type Vec3 = [f64; 3];
type Vec4 = [f64; 4];
type Mat4x4 = [Vec4; 4];

const BACKGROUND: Color = Color::RGB(18, 18, 18);
const FOREGROUND: Color = Color::RGB(255, 150, 150);
const DISTANCE: f64 = 5.0;

fn dot_4d_vv(v1: Vec4, v2: Vec4) -> f64 {
    let mut result = 0.0;
    for i in 0..4 {
        result += v1[i] * v2[i];
    }
    result
}

fn dot_4d_mv(m: Mat4x4, v: Vec4) -> Vec4 {
    let mut result: Vec4 = [0.0, 0.0, 0.0, 0.0];
    for i in 0..4 {
        result[i] = dot_4d_vv(m[i], v);
    }
    result
}

fn transpose_mat4x4(m: Mat4x4) -> Mat4x4 {
    let mut result: Mat4x4 = [[0.0; 4];4];
    for i in 0..4 {
        for j in 0..4 {
            result[i][j] = m[j][i];
        }
    }
    result
}

fn dot_4d_mm(m1: Mat4x4, m2: Mat4x4) -> Mat4x4 {
    let mut result: Mat4x4 = [[0.0; 4]; 4];
    for (i, v1) in m1.iter().enumerate() {
        for (j, v2) in transpose_mat4x4(m2).iter().enumerate() {
            result[i][j] = dot_4d_vv(*v1, *v2);
        }
    }
    result
}

fn project_3d_to_2d_persp([x, y, z]: Vec3, r: f64) -> [f64; 2] {
    return [x * r / z, y / z];
}

fn project_4d_to_3d_ortho([x, y, z, u]: Vec4) -> Vec3 {
    return [x, y, z + u];
}

fn to_screen([x0, y0]: [f64; 2], w: f64, h: f64) -> [f64; 2] {
    let half_w = w * 0.5;
    let half_h = h * 0.5;
    let x = x0 * half_w + half_w;
    let y = y0 * half_h + half_h;
    [x, y]
}

fn translate_4d([x0, y0, z0, u0]: Vec4, [x1, y1, z1, u1]: Vec4) -> Vec4 {
    return [x0 + x1, y0 + y1, z0 + z1, u0 + u1];
}

fn rotmat_4d_xz(theta: f64) -> Mat4x4 {
    [
        [f64::cos(theta) , 0.0, -f64::sin(theta), 0.0],
        [0.0             , 1.0, 0.0, 0.0],
        [f64::sin(theta) , 0.0, f64::cos(theta), 0.0],
        [0.0             , 0.0, 0.0, 1.0],
    ]
}

fn rotmat_4d_zu(theta: f64) -> Mat4x4 {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, f64::cos(theta), -f64::sin(theta)],
        [0.0, 0.0, f64::sin(theta), f64::cos(theta)],
    ]
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem.window("4D", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let mut timer_subsystem = sdl_context.timer()?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    let mut theta: f64 = 0.0;
    'running: loop {
        let begin = timer_subsystem.ticks();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },
                _ => {}
            }

        }

        canvas.set_draw_color(BACKGROUND);
        canvas.clear();
        let (w, h) = canvas.window().size();
        const LOW_RANGE: f64 = -1.0;
        const HIGH_RANGE: f64 = 1.0;
        const N: u32 = 2;
        const D: usize = 4;
        const DS: f64 = (HIGH_RANGE - LOW_RANGE) / (N - 1) as f64;
        // TODO: make D affect the amount of nested loops and stuff
        for ix in 0..N {
            for iy in 0..N {
                for iz in 0..N {
                    for iw in 0..N {
                        let p1 = [LOW_RANGE + ix as f64 * DS,
                                  LOW_RANGE + iy as f64 * DS,
                                  LOW_RANGE + iz as f64 * DS,
                                  LOW_RANGE + iw as f64 * DS];

                        for id in 0..D {
                            let p2 = {
                                let mut t = p1.clone();
                                t[id] += DS;
                                t
                            };

                            if p2[id] <= HIGH_RANGE {
                                let r = h as f64 / w as f64;
                                let rotmat =
                                    dot_4d_mm(
                                        rotmat_4d_xz(-0.5),
                                        rotmat_4d_zu(theta));
                                let ps1 = to_screen(
                                    project_3d_to_2d_persp(
                                        project_4d_to_3d_ortho(
                                            translate_4d(
                                                dot_4d_mv(rotmat, p1),
                                                [0.0, 0.0, DISTANCE, 0.0])),
                                        r),
                                    w as f64, h as f64);
                                let ps2 = to_screen(
                                    project_3d_to_2d_persp(
                                        project_4d_to_3d_ortho(
                                            translate_4d(
                                                dot_4d_mv(rotmat, p2),
                                                [0.0, 0.0, DISTANCE, 0.0])), r),
                                    w as f64, h as f64);

                                canvas.set_draw_color(FOREGROUND);
                                canvas.draw_line(Point::new(ps1[0] as i32, ps1[1] as i32),
                                                 Point::new(ps2[0] as i32, ps2[1] as i32))?;
                            }
                        }
                    }
                }
            }
        }
        canvas.present();

        let dt = (timer_subsystem.ticks() - begin) as f64 / 1000.0;
        theta = (theta + 2.0 * dt) % (2.0 * std::f64::consts::PI);
    }

    Ok(())
}
