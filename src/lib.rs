use std::{
    sync::Mutex,
};

use assets::{
    CAT_ARM, CAT_HAND_CLOSED, CAT_HAND_OPEN, HARDWOOD_FLOOR_PATTERN, HARDWOOD_FLOOR_SPRITES,
    MOUSE_TARGETS,
};
use sync_unsafe_cell::SyncUnsafeCell;
use wasm4::{tone, trace, MOUSE_LEFT, SCREEN_SIZE};
use wasm4_mmio::{MOUSE_BUTTONS, MOUSE_X, MOUSE_Y, PALETTE};

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

#[derive(Debug)]
// Top level struct for game state
struct GameState {
    pub frame: u32,
    pub cat_hand_x: f32,
    pub cat_hand_y: f32,
    pub cat_hand_x_prev: f32,
    pub cat_hand_y_prev: f32,
    pub cat_hand_state: CatHandState,
    pub cat_hand_target_x: f32,
    pub cat_hand_target_y: f32,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
enum CatHandState {
    Ready,
    Attacking,
    Retreating,
}

impl GameState {
    const fn new() -> Self {
        Self {
            frame: 0,
            cat_hand_x: 120.0,
            cat_hand_y: CAT_HAND_WAIT_Y,
            cat_hand_x_prev: 120.0,
            cat_hand_y_prev: CAT_HAND_WAIT_Y,
            cat_hand_state: CatHandState::Ready,
            cat_hand_target_x: 120.0,
            cat_hand_target_y: CAT_HAND_WAIT_Y
        }
    }
}

static GLOBAL_GAME_STATE: SyncUnsafeCell<GameState> = SyncUnsafeCell::new(GameState::new());

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

const CAT_HAND_WAIT_Y: f32 = 155.0;
const CAT_HAND_ACCELERATION: f32 = 0.1;
const CAT_HAND_CATCH_RANGE: f32 = 10.0;

fn update_cat_hand(game_state: &mut GameState) {
    // TODO: Split this out into its own module
    // TODO: This system looks real real bad. The cat hand does not move at all naturally
    //       and does not follow the nice arc you were looking to achieve.
    //       Maybe there's a different way you can achieve a nice arc (e.g.
    //       the only force the cat hand can do is extend/retract, and horizontal
    //       movement is controlled with orbiting around a character instead)
    // set target based on input
    match game_state.cat_hand_state {
        CatHandState::Ready => {
            if MOUSE_BUTTONS.read() & MOUSE_LEFT != 0 {
                game_state.cat_hand_state = CatHandState::Attacking;
                game_state.cat_hand_target_x = MOUSE_X.read() as f32;
                game_state.cat_hand_target_y = MOUSE_Y.read() as f32;
                trace(format!("click {} {}", game_state.cat_hand_target_x, game_state.cat_hand_target_y));
                tone(440, 10, 10, 0);
            } else {
                game_state.cat_hand_target_x = (MOUSE_X.read() as f32 + 160.0) / 2.0;
                game_state.cat_hand_target_y = 155.0;
            }
        },
        _ => ()
    }

    // accelerate towards target
    let x_to_target = game_state.cat_hand_target_x - game_state.cat_hand_x;
    let y_to_target = game_state.cat_hand_target_y - game_state.cat_hand_y;
    let distance_to_target = (x_to_target * x_to_target + y_to_target * y_to_target).sqrt();
    let mut x_acceleration: f32;
    let mut y_acceleration: f32;
    if distance_to_target == 0.0 || CAT_HAND_ACCELERATION < distance_to_target {
        x_acceleration = x_to_target;
        y_acceleration = y_to_target;
    } else {
        x_acceleration = x_to_target * CAT_HAND_ACCELERATION / distance_to_target;
        y_acceleration = y_to_target * CAT_HAND_ACCELERATION / distance_to_target;
    }

    // apply friction
    let friction_coefficient = if game_state.cat_hand_state == CatHandState::Ready { 0.95 } else { 0.5 };
    x_acceleration -= (game_state.cat_hand_x - game_state.cat_hand_x_prev) * friction_coefficient;
    y_acceleration -= (game_state.cat_hand_y - game_state.cat_hand_y_prev) * friction_coefficient;

    // apply verlet integration
    let x_velocity = game_state.cat_hand_x - game_state.cat_hand_x_prev + x_acceleration / 2.0;
    let y_velocity = game_state.cat_hand_y - game_state.cat_hand_y_prev + y_acceleration / 2.0;
    game_state.cat_hand_x_prev = game_state.cat_hand_x;
    game_state.cat_hand_y_prev = game_state.cat_hand_y;
    game_state.cat_hand_x += x_velocity;
    game_state.cat_hand_y += y_velocity;

    // resolve attack states
    match game_state.cat_hand_state {
        CatHandState::Ready => (),
        CatHandState::Attacking => {
            let x_to_target = game_state.cat_hand_target_x - game_state.cat_hand_x;
            let y_to_target = game_state.cat_hand_target_y - game_state.cat_hand_y;
            let distance_to_target = (x_to_target * x_to_target + y_to_target * y_to_target).sqrt();
            if distance_to_target < CAT_HAND_CATCH_RANGE {
                game_state.cat_hand_state = CatHandState::Retreating;
                game_state.cat_hand_target_y = 160.0;
                game_state.cat_hand_target_x = 0.0;
                // TODO: score target here
            }
        },
        CatHandState::Retreating => {
            if game_state.cat_hand_y >= CAT_HAND_WAIT_Y
                && MOUSE_BUTTONS.read() & MOUSE_LEFT == 0 {
                game_state.cat_hand_state = CatHandState::Ready;
                tone(300, 10, 10, 0);
            }
        },
    }
}

fn draw_cat_hand(game_state: &GameState) {
    // TODO: Split this out into its own module
    let cat_hand_sprite = if game_state.cat_hand_state == CatHandState::Attacking {
        &CAT_HAND_OPEN
    } else {
        &CAT_HAND_CLOSED
    };
    cat_hand_sprite.draw(
        game_state.cat_hand_x as i32 - cat_hand_sprite.width as i32 / 2,
        game_state.cat_hand_y as i32 - cat_hand_sprite.height as i32 / 2,
        0,
    );
    let mut cat_arm_y = game_state.cat_hand_y as i32 + cat_hand_sprite.height as i32 / 2;
    while cat_arm_y < SCREEN_SIZE as i32 {
        CAT_ARM.draw(
            game_state.cat_hand_x as i32 - cat_hand_sprite.width as i32 / 2,
            cat_arm_y as i32,
            0,
        );
        cat_arm_y += CAT_ARM.height as i32;
    }
}

#[no_mangle]
fn update() {
    let mut game_state = unsafe { GLOBAL_GAME_STATE.get().as_mut().unwrap() };
    update_cat_hand(&mut game_state);

    draw_floor_pattern();

    let mouse_animation_frame = ((game_state.frame % 12) / 4) as usize;

    let mouse_sprite = &(MOUSE_TARGETS[mouse_animation_frame]);
    mouse_sprite.draw(60, 60, 0);

    draw_cat_hand(&game_state);

    game_state.frame += 1;
}
