use std::{
    cell::{Cell, RefCell},
    sync::Mutex,
};

use assets::{
    CAT_ARM, CAT_HAND_CLOSED, CAT_HAND_OPEN, HARDWOOD_FLOOR_PATTERN, HARDWOOD_FLOOR_SPRITES,
    MOUSE_TARGETS,
};
use wasm4::SCREEN_SIZE;
use wasm4_mmio::PALETTE;

#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;

// A module with all our assets in it
mod assets;

// Custom libraries I wrote to make asset management easier
// and deal less with unsafe
mod sprite;
mod wasm4_mmio;

// Palettes from https://itch.io/jam/gbpixelartjam24
//const PALETTE_BLOOD_TIDE: [u32; 4] = [0x652121, 0x394a5a, 0x7a968d, 0xfffeea];
//const PALETTE_FORGOTTEN_SWAMP: [u32; 4] = [0x3b252e, 0x593a5f, 0x4d7d65, 0xd1ada1];
const PALETTE_HOMEWORK: [u32; 4] = [0x12121b, 0x45568d, 0x878c9d, 0xe1d8d4];
//const PALETTE_MANGAVANIA: [u32; 4] = [0x6e1a4b, 0xe64ca4, 0x4aedff, 0xffffff];

// Top level struct for game state
struct GameState {
    pub frame: u32,
}

// TODO: If I can use SyncUnsafeCell here instead, that'd be great, since
//       we don't actually have mutliple threads running through here
static GLOBAL_GAME_STATE: Mutex<GameState> = Mutex::new(GameState { frame: 0 });

#[no_mangle]
fn start() {
    PALETTE.write(PALETTE_HOMEWORK);
}

fn draw_floor_pattern() {
    let mut x = -32;
    let mut y = -32;
    for pattern_byte in HARDWOOD_FLOOR_PATTERN {
        if y > SCREEN_SIZE as i32 + 32 {
            break;
        }
        let mut b = pattern_byte;
        for i in 0..3 {
            let floor_index = (b & 0x03) as usize;
            let floor_sprite = &HARDWOOD_FLOOR_SPRITES[floor_index];
            floor_sprite.draw(x, y, 0);
            x += floor_sprite.width as i32;
            if x > SCREEN_SIZE as i32 + 32 {
                y += 8;
                x -= SCREEN_SIZE as i32 + 64;
                if (x > -(SCREEN_SIZE as i32) - 32) {
                    floor_sprite.draw(x - floor_sprite.width as i32, y, 0);
                }
            }
            b = b >> 2;
        }
    }
}

#[no_mangle]
fn update() {
    let mut game_state = GLOBAL_GAME_STATE.lock().unwrap();

    draw_floor_pattern();

    let mouse_animation_frame = ((game_state.frame % 12) / 4) as usize;

    let mouse_sprite = &(MOUSE_TARGETS[mouse_animation_frame]);
    mouse_sprite.draw(60, 60, 0);

    // TODO: This is just a demo. For the real thing, draw the hand swiping up from
    //       the bottom of the screen, nabbing a target, and coming back down.
    //       You would ideally like the hand to go up and down in an arc, and it should
    //       also be tracking a moving game entity, so that you can reward a player
    //       for the instant they click on a target rather than attempting to do collision
    //       when the hand arrives.
    let cat_hand_animation_frame = ((game_state.frame % 32) / 16) as usize;
    let cat_hand_sprite = &(if cat_hand_animation_frame == 0 {
        CAT_HAND_OPEN
    } else {
        CAT_HAND_CLOSED
    });
    cat_hand_sprite.draw(120, 70, 0);
    let mut cat_arm_y = 70 + cat_hand_sprite.height;
    while cat_arm_y < SCREEN_SIZE {
        CAT_ARM.draw(120, cat_arm_y as i32, 0);
        cat_arm_y += CAT_ARM.height;
    }

    game_state.frame += 1;
}
