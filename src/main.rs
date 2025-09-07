use pixels::{Error, Pixels, SurfaceTexture};
use rand::Rng;
use std::time::Instant;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const STAR_COUNT: usize = 5000;
const SHOOTING_STAR_GRAVITY: f32 = 30.0;
const STAR_MIN_SIZE: u32 = 1;
const STAR_MAX_SIZE: u32 = 4;
const STAR_MIN_SPEED: f32 = 5.0;
const STAR_MAX_SPEED: f32 = 25.0;

struct ScreenDetails {
    width: u32,
    height: u32,
}

// Common trait for all celestial objects
trait CelestialObject {
    fn update(&mut self, dt: f32, elapsed: f32, rng: &mut impl Rng, screen_details: &ScreenDetails);
    fn draw(&self, frame: &mut [u8], screen_details: &ScreenDetails);
    fn is_alive(&self, screen_details: &ScreenDetails) -> bool;
}

struct Star {
    x: f32,
    y: f32,
    speed: f32,
    twinkle_phase: f32,
    twinkle_speed: f32,
    can_twinkle: bool,
    depth: f32,
    color: (u8, u8, u8),
    size: u32,
}

impl CelestialObject for Star {
    fn update(
        &mut self,
        dt: f32,
        _elapsed: f32,
        rng: &mut impl Rng,
        screen_details: &ScreenDetails,
    ) {
        self.speed *= 0.999_f32.powf(dt * 60.0);
        self.x -= self.speed * self.depth * dt;

        if self.x < 0.0 {
            self.x = screen_details.width as f32;
            self.y = rng.gen_range(0.0..screen_details.height as f32);
            self.depth = rng.gen_range(0.5..2.0);
            self.twinkle_phase = rng.gen_range(0.0..std::f32::consts::TAU);
            self.twinkle_speed = rng.gen_range(0.5..3.14); // Max 1 blink every 2 seconds
            self.speed = rng.gen_range(STAR_MIN_SPEED..STAR_MAX_SPEED);
            self.size = rng.gen_range(STAR_MIN_SIZE..=STAR_MAX_SIZE);
        }
    }

    fn draw(&self, frame: &mut [u8], screen_details: &ScreenDetails) {
        // We need elapsed time for twinkling, but we can calculate it from the phase
        // For now, let's use a simple approach - we'll pass elapsed through context later if needed
        let twinkle = (self.twinkle_phase).sin() * 0.5 + 0.5;
        let intensity = (twinkle * 255.0 / self.depth).min(200.0) as u8;

        let (base_r, base_g, base_b) = self.color;
        let r = ((base_r as f32 * (intensity as f32 / 255.0)).min(255.0)) as u8;
        let g = ((base_g as f32 * (intensity as f32 / 255.0)).min(255.0)) as u8;
        let b = ((base_b as f32 * (intensity as f32 / 255.0)).min(255.0)) as u8;

        for dx in 0..self.size {
            for dy in 0..self.size {
                let ix = self.x as i32 + dx as i32;
                let iy = self.y as i32 + dy as i32;
                if ix >= 0
                    && ix < screen_details.width as i32
                    && iy >= 0
                    && iy < screen_details.height as i32
                {
                    let idx = ((iy as u32 * screen_details.width + ix as u32) * 4) as usize;
                    frame[idx] = r;
                    frame[idx + 1] = g;
                    frame[idx + 2] = b;
                    frame[idx + 3] = 255;
                }
            }
        }
    }

    fn is_alive(&self, _: &ScreenDetails) -> bool {
        true // Stars are always alive, they just wrap around
    }
}

impl Star {
    fn new(rng: &mut impl Rng, width: u32, height: u32) -> Self {
        let palette = [
            (180, 200, 255), // blue
            (255, 255, 255), // white
            (255, 255, 200), // yellow
            (255, 220, 180), // orange
            (255, 180, 180), // red
        ];
        let color = palette[rng.gen_range(0..palette.len())];

        Self {
            x: rng.gen_range(0.0..width as f32),
            y: rng.gen_range(0.0..height as f32),
            speed: rng.gen_range(STAR_MIN_SPEED..STAR_MAX_SPEED),
            can_twinkle: rng.gen_bool(0.15),
            twinkle_phase: rng.gen_range(0.0..std::f32::consts::TAU),
            twinkle_speed: rng.gen_range(0.5..3.14), // Max 1 blink every 2 seconds
            depth: rng.gen_range(0.5..4.0),
            color,
            size: rng.gen_range(STAR_MIN_SIZE..=STAR_MAX_SIZE),
        }
    }

    fn update_twinkle(&mut self, elapsed: f32) {
        if self.can_twinkle {
            self.twinkle_phase = elapsed * self.twinkle_speed + self.twinkle_phase;
        }
    }
}

struct ShootingStar {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    trail: Vec<(f32, f32)>,
    trail_max_len: usize,
}

impl CelestialObject for ShootingStar {
    fn update(&mut self, dt: f32, _elapsed: f32, _rng: &mut impl Rng, _: &ScreenDetails) {
        // Store current position in trail
        self.trail.push((self.x, self.y));
        if self.trail.len() > self.trail_max_len {
            self.trail.remove(0);
        }

        // Update physics
        self.x += self.vx * dt;
        self.vy += SHOOTING_STAR_GRAVITY * dt;
        self.y += self.vy * dt;
        self.life += dt;
    }

    fn draw(&self, frame: &mut [u8], _: &ScreenDetails) {
        let alpha = (1.0 - self.life / self.max_life).clamp(0.0, 1.0);

        // Draw trail using stored positions
        for (i, &(tx, ty)) in self.trail.iter().enumerate() {
            let trail_progress = i as f32 / self.trail.len() as f32;
            let trail_alpha = alpha * trail_progress * trail_progress; // Quadratic falloff

            if trail_alpha < 0.01 {
                continue; // Skip nearly invisible segments
            }

            // Color gradient: white/yellow at head to orange/red at tail
            let r = (255.0 * (0.8 + 0.2 * trail_progress)) as u8;
            let g = (255.0 * (0.6 + 0.4 * trail_progress)) as u8;
            let b = (100.0 + 155.0 * (1.0 - trail_progress)) as u8;

            // Variable width: thicker at head, thinner at tail
            let width = (1.0 + 3.0 * trail_progress) as i32;

            self.draw_point(frame, tx, ty, r, g, b, trail_alpha, width);
        }

        // Draw bright head
        if alpha > 0.01 {
            let head_size = 6;
            self.draw_point(frame, self.x, self.y, 255, 255, 220, alpha, head_size);
        }
    }

    fn is_alive(&self, screen_details: &ScreenDetails) -> bool {
        self.life < self.max_life
            && self.x > -200.0
            && self.x < screen_details.width as f32 + 200.0
            && self.y > -200.0
            && self.y < screen_details.height as f32 + 200.0
    }
}

impl ShootingStar {
    fn new(start_x: f32, start_y: f32, vx: f32, vy: f32) -> Self {
        let max_life = 3.0;
        Self {
            x: start_x,
            y: start_y,
            vx,
            vy,
            life: 0.0,
            max_life,
            trail: Vec::new(),
            trail_max_len: 80,
        }
    }

    fn draw_point(
        &self,
        frame: &mut [u8],
        x: f32,
        y: f32,
        r: u8,
        g: u8,
        b: u8,
        alpha: f32,
        size: i32,
    ) {
        let center_x = x as i32;
        let center_y = y as i32;

        for dx in -size / 2..=size / 2 {
            for dy in -size / 2..=size / 2 {
                let px = center_x + dx;
                let py = center_y + dy;

                if px >= 0 && px < WIDTH as i32 && py >= 0 && py < HEIGHT as i32 {
                    let idx = ((py as u32 * WIDTH + px as u32) * 4) as usize;

                    // Soft circular falloff
                    let dist = ((dx * dx + dy * dy) as f32).sqrt();
                    let radius = size as f32 / 2.0;
                    let falloff = (1.0 - (dist / radius).clamp(0.0, 1.0)).powf(2.0);
                    let final_alpha = (alpha * falloff).clamp(0.0, 1.0);

                    // Proper alpha blending
                    let old_r = frame[idx] as f32 / 255.0;
                    let old_g = frame[idx + 1] as f32 / 255.0;
                    let old_b = frame[idx + 2] as f32 / 255.0;

                    let new_r = r as f32 / 255.0;
                    let new_g = g as f32 / 255.0;
                    let new_b = b as f32 / 255.0;

                    frame[idx] =
                        ((old_r * (1.0 - final_alpha) + new_r * final_alpha) * 255.0) as u8;
                    frame[idx + 1] =
                        ((old_g * (1.0 - final_alpha) + new_g * final_alpha) * 255.0) as u8;
                    frame[idx + 2] =
                        ((old_b * (1.0 - final_alpha) + new_b * final_alpha) * 255.0) as u8;
                    frame[idx + 3] = 255;
                }
            }
        }
    }
}

// Helper function to update and draw celestial objects
fn update_and_draw_objects<T: CelestialObject>(
    objects: &mut Vec<T>,
    dt: f32,
    elapsed: f32,
    frame: &mut [u8],
    rng: &mut impl Rng,
    screen_details: &ScreenDetails,
) {
    objects.retain_mut(|obj| {
        obj.update(dt, elapsed, rng, screen_details);
        obj.draw(frame, screen_details);
        obj.is_alive(screen_details)
    });
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("wl-starfield")
        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        .build(&event_loop)
        .unwrap();

    // Get monitor resolution at startup
    let size = window
        .current_monitor()
        .map(|m| m.size())
        .unwrap_or(PhysicalSize::new(WIDTH, HEIGHT));

    let screen_details = ScreenDetails {
        width: size.width,
        height: size.height,
    };

    let surface_texture = SurfaceTexture::new(screen_details.width, screen_details.height, &window);
    let mut pixels = Pixels::new(screen_details.width, screen_details.height, surface_texture)?;

    let mut rng = rand::thread_rng();
    let mut stars: Vec<Star> = (0..STAR_COUNT)
        .map(|_| Star::new(&mut rng, screen_details.width, screen_details.height))
        .collect();
    let mut shooting_stars: Vec<ShootingStar> = Vec::new();
    let start = Instant::now();
    let mut last_frame = start;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                let now = Instant::now();
                let dt = (now - last_frame).as_secs_f32();
                last_frame = now;

                let elapsed = start.elapsed().as_secs_f32();
                let frame = pixels.frame_mut();
                frame.fill(0);

                // Update stars with special handling for twinkling
                for star in &mut stars {
                    star.update(dt, elapsed, &mut rng, &screen_details);
                    star.update_twinkle(elapsed);
                    star.draw(frame, &screen_details);
                }

                // Spawn shooting stars less frequently but more predictably
                if rng.gen_bool(dt as f64 * 0.3) {
                    // About 1 every 3-4 seconds
                    let start_x = screen_details.width as f32 + 50.0; // Start off-screen
                    let start_y = rng.gen_range(50.0..screen_details.height as f32 * 0.4);
                    let vx = -rng.gen_range(200.0..400.0); // Faster horizontal speed
                    let vy = rng.gen_range(10.0..50.0); // Moderate downward speed

                    shooting_stars.push(ShootingStar::new(start_x, start_y, vx, vy));
                }

                // Update and draw shooting stars using the trait
                update_and_draw_objects(
                    &mut shooting_stars,
                    dt,
                    elapsed,
                    frame,
                    &mut rng,
                    &screen_details,
                );

                if pixels.render().is_err() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } = event
                {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}
