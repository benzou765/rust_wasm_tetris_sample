// rustc lib.rs --crate-type=cdylib -O --target=wasm32-unknown -o test.wasm

const WIDTH: i32 = 640;
const HEIGHT: i32 = 480;
const RGBA: i32 = 4;
const BLOCK_SIZE: i32 = 22;
const FIELD_X: i32 = 10;
const FIELD_Y: i32 = 20;
const BLOCK_TYPE_NUM: i32 = 7;

// イメージを保存するメモリ
static mut IMAGE_BUFFER: &mut [u8] = &mut [0; (WIDTH * HEIGHT * RGBA) as usize];
// フィールドの状態を保存するメモリ
static mut FIELD: &mut [u8] = &mut [0; (FIELD_X * FIELD_Y) as usize];
// 経過時間を表す値
static mut ELAPSED_TIME: i32 = 0;

// 現在所有しているブロックの種類
static mut BLOCK_TYPE: i32 = 0;
static mut X: i32 = 5;
static mut Y: i32 = 0;

// テトリスのブロック構成
const T_BLOCK: [u8; (5 * 5)] = [
    0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const O_BLOCK: [u8; (5 * 5)] = [
    0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const N_BLOCK: [u8; (5 * 5)] = [
    0, 0, 0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const RN_BLOCK: [u8; (5 * 5)] = [
    0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 0, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const L_BLOCK: [u8; (5 * 5)] = [
    0, 0, 0, 0, 0, 0, 6, 6, 6, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const RL_BLOCK: [u8; (5 * 5)] = [
    0, 0, 0, 0, 0, 0, 5, 5, 5, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const I_BLOCK: [u8; (5 * 5)] = [
    0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
];

// js側のメソッド
extern "C" {
    fn js_console_log(ptr: *const u8, size: usize);
    fn random() -> f64;
}

fn console_log(message: &str) {
    unsafe { js_console_log(message.as_ptr(), message.len()) }
}

struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

unsafe fn draw_pixel(x: i32, y: i32, color: &Color) {
    IMAGE_BUFFER[((y * WIDTH + x) * RGBA) as usize] = color.red;
    IMAGE_BUFFER[((y * WIDTH + x) * RGBA + 1) as usize] = color.green;
    IMAGE_BUFFER[((y * WIDTH + x) * RGBA + 2) as usize] = color.blue;
    IMAGE_BUFFER[((y * WIDTH + x) * RGBA + 3) as usize] = color.alpha;
}

unsafe fn draw_rect(sx: i32, sy: i32, dx: i32, dy: i32, color: &Color) {
    for x in sx..dx {
        // top
        draw_pixel(x, sy, &color);
        // bottom
        draw_pixel(x, dy, &color);
    }
    for y in sy..dy {
        // left
        draw_pixel(sx, y, &color);
        // right
        draw_pixel(dx, y, &color);
    }
}

unsafe fn draw_back_ground(x: i32, y: i32) {
    let back_color = Color {
        red: 238,
        green: 238,
        blue: 238,
        alpha: 255,
    };

    draw_rect(
        (x * BLOCK_SIZE) + 20, // 開始位置 * ブロックサイズ + 初期位置
        (y * BLOCK_SIZE) + 20,
        (x * BLOCK_SIZE) + BLOCK_SIZE + 20,
        (y * BLOCK_SIZE) + BLOCK_SIZE + 20,
        &back_color,
    )
}

/// 1ブロックを描画
/// # Arguments
/// * x, y はフィールドのx, y座標
unsafe fn draw_one_block(x: i32, y: i32, color: &Color) {
    // draw frame
    let gray = Color {
        red: 160,
        green: 160,
        blue: 160,
        alpha: 255,
    };
    draw_rect(
        (x * BLOCK_SIZE) + 20, // 開始位置 * ブロックサイズ + 初期位置
        (y * BLOCK_SIZE) + 20,
        (x * BLOCK_SIZE) + BLOCK_SIZE + 19,
        (y * BLOCK_SIZE) + BLOCK_SIZE + 19,
        &gray,
    );

    // draw inner
    for j in ((y * BLOCK_SIZE) + 21)..((y * BLOCK_SIZE) + BLOCK_SIZE + 19) {
        for i in ((x * BLOCK_SIZE) + 21)..((x * BLOCK_SIZE) + BLOCK_SIZE + 19) {
            draw_pixel(i, j, color);
        }
    }
}

unsafe fn draw_frame() {
    let black = Color {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 255,
    };
    draw_rect(19, 19, 240, 460, &black);
}

unsafe fn draw_block() {
    // I_BLOCK
    let cyan = Color {
        red: 0,
        green: 255,
        blue: 255,
        alpha: 255,
    };
    // O_BLOCK
    let yellow = Color {
        red: 255,
        green: 255,
        blue: 0,
        alpha: 255,
    };
    // RN_BLOCK
    let green = Color {
        red: 0,
        green: 255,
        blue: 0,
        alpha: 255,
    };
    // N_BLOCK
    let red = Color {
        red: 255,
        green: 0,
        blue: 0,
        alpha: 255,
    };
    // RL_BLOCK
    let blue = Color {
        red: 0,
        green: 0,
        blue: 255,
        alpha: 255,
    };
    // L_BLOCK
    let orange = Color {
        red: 255,
        green: 128,
        blue: 0,
        alpha: 255,
    };
    // T_BLOCK
    let magenta = Color {
        red: 255,
        green: 0,
        blue: 255,
        alpha: 255,
    };
    // ユーザーのブロックを描画
    let mut blocks: &[u8] = &O_BLOCK;
    match BLOCK_TYPE {
        1 => blocks = &O_BLOCK,
        2 => blocks = &N_BLOCK,
        3 => blocks = &RN_BLOCK,
        4 => blocks = &L_BLOCK,
        5 => blocks = &RL_BLOCK,
        6 => blocks = &I_BLOCK,
        _ => blocks = &T_BLOCK,
    }

    let mut y = 0;
    for y2 in (Y - 2)..(Y + 3) {
        let mut x = 0;
        if 0 <= y2 && y2 < FIELD_Y {
            for x2 in (X - 2)..(X + 3) {
                if 0 <= x2 && x2 < FIELD_X {
                    match blocks[((y * 5) + x) as usize] {
                        1 => draw_one_block(x2, y2, &cyan),
                        2 => draw_one_block(x2, y2, &yellow),
                        3 => draw_one_block(x2, y2, &green),
                        4 => draw_one_block(x2, y2, &red),
                        5 => draw_one_block(x2, y2, &blue),
                        6 => draw_one_block(x2, y2, &orange),
                        7 => draw_one_block(x2, y2, &magenta),
                        _ => draw_back_ground(x2, y2),
                    }
                }
                x = x + 1;
            }
        }
        y = y + 1;
    }

    // フィールド上のブロックを描画
    for j in 0..(FIELD_Y) {
        for i in 0..(FIELD_X) {
            match FIELD[((j * FIELD_X) + i) as usize] {
                1 => draw_one_block(i, j, &cyan),
                2 => draw_one_block(i, j, &yellow),
                3 => draw_one_block(i, j, &green),
                4 => draw_one_block(i, j, &red),
                5 => draw_one_block(i, j, &blue),
                6 => draw_one_block(i, j, &orange),
                7 => draw_one_block(i, j, &magenta),
                _ => {}
            }
        }
    }
}

unsafe fn create_block() {
    let val = (random() * BLOCK_TYPE_NUM as f64).floor() as i32;
    match val {
        1 => BLOCK_TYPE = 1,
        2 => BLOCK_TYPE = 2,
        3 => BLOCK_TYPE = 3,
        4 => BLOCK_TYPE = 4,
        5 => BLOCK_TYPE = 5,
        6 => BLOCK_TYPE = 6,
        _ => BLOCK_TYPE = 0,
    }
    // 座標の初期化
    X = 5;
    Y = 0;
}

unsafe fn down_block() {
    Y = Y + 1;
}

#[no_mangle]
pub unsafe extern "C" fn getPixelAddress() -> *const u8 {
    &IMAGE_BUFFER[0]
}

// jsから呼び出すメソッドの場合は、no_mangleが必要
/// "init" is called before the first frame update
#[no_mangle]
pub unsafe extern "C" fn init() {
    console_log("called init method");
    // draw first view
    draw_frame();
    create_block();
}

/// "update" is called once per frame
#[no_mangle]
pub unsafe extern "C" fn update() {
    ELAPSED_TIME = ELAPSED_TIME + 1;
    if ELAPSED_TIME % 60 == 0 {
        down_block();
    }
    draw_frame();
    draw_block();
}
