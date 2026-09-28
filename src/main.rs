use image::imageops::FilterType;
use macroquad::prelude::*;

const IMAGE_WIDTH: u32 = 64;
const IMAGE_HEIGHT: u32 = 64;

#[derive(Clone, Copy, Debug)]

struct Pixel {
    x: u32,
    y: u32,

    r: u8,
    g: u8,
    b: u8,
}

struct Particle {
    start_x: f32,
    start_y: f32,

    end_x: f32,
    end_y: f32,

    r: u8,
    g: u8,
    b: u8,
}

fn load_pixels(path: &str, width: u32, height: u32) -> image::ImageResult<Vec<Pixel>> {
    let image = image::open(path)?;

    let image = image.resize_exact(width, height, FilterType::Triangle);

    let image = image.to_rgb8();
    let mut pixels = Vec::new();

    for (x, y, color) in image.enumerate_pixels() {
        let [r, g, b] = color.0;

        pixels.push(Pixel { x, y, r, g, b });
    }

    Ok(pixels)
}

fn color_distance_squared(p1: &Pixel, p2: &Pixel) -> u32 {
    let dr = p1.r as i32 - p2.r as i32;
    let dg = p1.g as i32 - p2.g as i32;
    let db = p1.b as i32 - p2.b as i32;

    (dr * dr + dg * dg + db * db) as u32
}

fn find_closest_pixel(source: &[Pixel], used: &[bool], target: &Pixel) -> Option<usize> {
    let mut best_index: Option<usize> = None;
    let mut best_distance = u32::MAX;

    for (index, pixel) in source.iter().enumerate() {
        if used[index] {
            continue;
        }

        let distance = color_distance_squared(pixel, target);

        if distance < best_distance {
            best_distance = distance;
            best_index = Some(index);
        }
    }
    best_index
}

fn create_particles(source: &[Pixel], target: &[Pixel]) -> Vec<Particle> {
    assert_eq!(source.len(), target.len());

    let mut used = vec![false; source.len()];
    let mut particles = Vec::new();

    for target_pixel in target {
        let source_index = find_closest_pixel(source, &used, target_pixel);

        match source_index {
            Some(index) => {
                let source_pixel = source[index];
                used[index] = true;

                particles.push(Particle {
                    start_x: source_pixel.y as f32,
                    start_y: source_pixel.y as f32,

                    end_x: target_pixel.x as f32,
                    end_y: target_pixel.y as f32,

                    r: source_pixel.r,
                    g: source_pixel.g,
                    b: source_pixel.b,
                });
            }
            None => {
                panic!("Ran out of source pixels");
            }
        }
    }
    particles
}

fn linear_interp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

fn ease_in_and_out(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[macroquad::main("Pixel Morph")]
async fn main() {
    let source = match load_pixels("src/source.jpeg", IMAGE_WIDTH, IMAGE_HEIGHT) {
        Ok(pixels) => pixels,
        Err(error) => {
            println!("Failed to load source: {}", error);
            return;
        }
    };

    let target = match load_pixels("src/target.jpeg", IMAGE_WIDTH, IMAGE_HEIGHT) {
        Ok(pixels) => pixels,
        Err(error) => {
            println!("Failed to load target: {}", error);
            return;
        }
    };

    println!("Creating assignments");

    let particles = create_particles(&source, &target);

    println!("Created {} particles", particles.len());

    let start_time = get_time();

    loop {
        clear_background(BLACK);
        let elapsed = (get_time() - start_time) as f32;
        let duration = 5.0;
        let t = (elapsed / duration).clamp(0.0, 1.0);

        let t = ease_in_and_out(t);

        let scale = 8.0;
        for particle in &particles {
            let x = linear_interp(particle.start_x, particle.end_x, t);
            let y = linear_interp(particle.start_y, particle.end_y, t);

            let color = Color::from_rgba(particle.r, particle.g, particle.b, 255);

            draw_rectangle(x * scale, y * scale, scale, scale, color);
        }

        next_frame().await;
    }
}
