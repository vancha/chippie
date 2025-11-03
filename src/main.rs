use chip8_emulator::constants::{CYCLES_PER_FRAME, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use chip8_emulator::cpu::Cpu;
use chip8_emulator::rombuffer::RomBuffer;
use macroquad::prelude::*;

///This line creates a macroquad application window with the title "chip 8 interpreter \ chippie\"
#[macroquad::main("Chip 8 interpreter \"Chippie\" ")]
async fn main() {
    //creating a chip8 cpu object with a rom loaded
    let mut c = Cpu::new(RomBuffer::new("tests/ibmlogo.ch8"));

    //used for displaying the screen of the chip-8 to the user
    let mut image = Image::gen_image_color(DISPLAY_WIDTH as u16, DISPLAY_HEIGHT as u16, WHITE);
    let mut buffer = vec![false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);

    let mut running = true;

    while running {
        if is_key_pressed(KeyCode::Escape) {
            running = false;
            continue;
        }

        //runs a bunch of cycles to keep everything running at a reasonable speed
        for _ in 0..=CYCLES_PER_FRAME {
            c.cycle();
        }

        //do the audio stuff, if the sound timer is non-zero, BEEP! Make sure that, if thats not
        //the case already, the audio timer (and delay timer) should be decremented at 60hz. 60HZ!!

        //@TODO: the actual audio stuff

        //some graphics specific things, fill the image variable with data
        clear_background(WHITE);
        for y in 0..DISPLAY_HEIGHT as i32 {
            for x in 0..DISPLAY_WIDTH as i32 {
                buffer[y as usize * DISPLAY_WIDTH + x as usize] =
                    c.get_display_contents()[y as usize][x as usize];
            }
        }
        //println!("{:?}",c.get_display_contents());

        for (i, _) in buffer.iter().enumerate() {
            image.set_pixel(
                (i % DISPLAY_WIDTH) as u32,
                (i / DISPLAY_WIDTH) as u32,
                match buffer[i] {
                    true => BLACK,
                    false => WHITE,
                },
            );
        }
        //add the image to a texture
        texture.update(&image);

        //show the texture to the user
        draw_texture_ex(
            &texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        //work on the next frame to display to the user
        next_frame().await;
    }
}
