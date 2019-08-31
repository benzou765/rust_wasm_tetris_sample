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
static mut USER_BLOCK: [u8; 25] = T_BLOCK;
static mut X: i32 = 5;
static mut Y: i32 = 0;

// テトリスのブロック構成
const T_BLOCK: [u8; 25] = [
    0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 7, 7, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const O_BLOCK: [u8; 25] = [
    0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const N_BLOCK: [u8; 25] = [
    0, 0, 0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const RN_BLOCK: [u8; 25] = [
    0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 0, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const L_BLOCK: [u8; 25] = [
    0, 0, 0, 0, 0, 0, 6, 6, 6, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const RL_BLOCK: [u8; 25] = [
    0, 0, 0, 0, 0, 0, 5, 5, 5, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const I_BLOCK: [u8; 25] = [
    0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
];

// ブロックの回転。回転行列で計算
const TURN_RIGHT: [usize; 25] = [
    4, 9, 14, 19, 24, 3, 8, 13, 18, 23, 2, 7, 12, 17, 22, 1, 6, 11, 16, 21, 0, 5, 10, 15, 20,
];
const TURN_LEFT: [usize; 25] = [
    20, 15, 10, 5, 0, 21, 16, 11, 6, 1, 22, 17, 12, 7, 2, 23, 18, 13, 8, 3, 24, 19, 14, 9, 4,
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

    for j in ((y * BLOCK_SIZE) + 20)..((y * BLOCK_SIZE) + BLOCK_SIZE + 20) {
        for i in ((x * BLOCK_SIZE) + 20)..((x * BLOCK_SIZE) + BLOCK_SIZE + 20) {
            draw_pixel(i, j, &back_color);
        }
    }
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
    for j in 0..FIELD_Y {
        for i in 0..FIELD_X {
            draw_back_ground(i, j);
        }
    }
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
    let mut y = 0;
    for y2 in (Y - 2)..(Y + 3) {
        let mut x = 0;
        if 0 <= y2 && y2 < FIELD_Y {
            for x2 in (X - 2)..(X + 3) {
                if 0 <= x2 && x2 < FIELD_X {
                    match USER_BLOCK[((y * 5) + x) as usize] {
                        1 => draw_one_block(x2, y2, &cyan),
                        2 => draw_one_block(x2, y2, &yellow),
                        3 => draw_one_block(x2, y2, &green),
                        4 => draw_one_block(x2, y2, &red),
                        5 => draw_one_block(x2, y2, &blue),
                        6 => draw_one_block(x2, y2, &orange),
                        7 => draw_one_block(x2, y2, &magenta),
                        _ => {}
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

// 初期ブロックの生成
unsafe fn create_block() {
    let val = (random() * BLOCK_TYPE_NUM as f64).floor() as i32;
    match val {
        1 => USER_BLOCK = O_BLOCK,
        2 => USER_BLOCK = N_BLOCK,
        3 => USER_BLOCK = RN_BLOCK,
        4 => USER_BLOCK = L_BLOCK,
        5 => USER_BLOCK = RL_BLOCK,
        6 => USER_BLOCK = I_BLOCK,
        _ => USER_BLOCK = T_BLOCK,
    }
    // 座標の初期化
    X = 5;
    Y = 0;
}

// 指定先に移動可能か
// x: 移動先のフィールド上のX座標
// y: 移動先のフィールド上のY座標
unsafe fn can_move(dist_x: i32, dist_y: i32) -> bool {
    for y in 0..5 {
        for x in 0..5 {
            if USER_BLOCK[(y * 5 + x) as usize] > 0 {
                // フィールドの下端に重なる部分はないか
                if (dist_y + y - 2) == FIELD_Y {
                    return false;
                }
                // フィールドの左端に重なる部分はないか
                if (dist_x + x - 2) == -1 {
                    return false;
                }
                // フィールドの右端に重なる部分はないか
                if (dist_x + x - 2) == FIELD_X {
                    return false;
                }
                // フィールドブロックと重なる場所はないか
                let field_px = dist_x + x - 2;
                let field_py = dist_y + y - 2;
                if 0 <= field_px && field_px < FIELD_X && 0 <= field_py && field_py < FIELD_Y {
                    if FIELD[(field_py * FIELD_X + field_px) as usize] > 0 {
                        return false;
                    }
                }
            }
        }
    }
    true
}

// ブロックを固定する
unsafe fn fix_block() {
    for y in 0..5 {
        for x in 0..5 {
            if USER_BLOCK[y * 5 + x] > 0 {
                let px = X + (x as i32) - 2;
                let py = Y + (y as i32) - 2;
                FIELD[(py * FIELD_X + px) as usize] = USER_BLOCK[y * 5 + x];
            }
        }
    }
}

// ブロックを1行削除して、1段落とす
unsafe fn clear_block(line: i32) {
    // 該当行のクリア
    for x in 0..FIELD_X {
        FIELD[(line * FIELD_X + x) as usize] = 0;
    }
    // 1段下げる
    let mut index = line - 1;
    while index >= 0 {
        for x in 0..FIELD_X {
            FIELD[((index + 1) * FIELD_X + x) as usize] = FIELD[(index * FIELD_X + x) as usize];
        }
        index = index - 1;
    }
}

// 消せるブロックがないか確認
// return 消した行数
unsafe fn check_line() -> i32 {
    let mut clear_num = 0;
    for line in 0..FIELD_Y {
        let mut x = 0;
        let mut block_num = 0;
        while x < FIELD_X {
            if FIELD[(line * FIELD_X + x) as usize] > 0 {
                block_num = block_num + 1;
            }
            x = x + 1;
        }
        if block_num == FIELD_X {
            clear_block(line);
            clear_num = clear_num + 1;
        }
    }
    clear_num
}

// ブロックが画面外に出ていないか確認
unsafe fn check_over_block() -> bool {
    false
}

// ブロックの落下
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
        if can_move(X, Y + 1) {
            down_block();
        } else {
            if check_over_block() {
                console_log("GAME OVER")
            } else {
                fix_block();
                check_line();
                create_block();
            }
        }
    }
    draw_frame();
    draw_block();
}

/// block move left
#[no_mangle]
pub unsafe extern "C" fn move_left() {
    if can_move(X - 1, Y) {
        X = X - 1
    }
}

/// block move right
#[no_mangle]
pub unsafe extern "C" fn move_right() {
    // フィールドの右端に接している
    if can_move(X + 1, Y) {
        X = X + 1
    }
}

/// block move down
#[no_mangle]
pub unsafe extern "C" fn move_down() {
    if can_move(X, Y + 1) {
        down_block();
    } else {
        if check_over_block() {
            console_log("GAME OVER")
        } else {
            fix_block();
            check_line();
            create_block();
        }
    }
}

/// block turn left
#[no_mangle]
pub unsafe extern "C" fn turn_left() {
    let clone_block = USER_BLOCK.clone();
    let mut i = 0;
    let mut moved_block = [0; 25];
    while i < 25 {
        moved_block[TURN_LEFT[i]] = clone_block[i];
        i = i + 1;
    }
    // 回転後の位置に移動可能かチェック
    let mut can_turn = true;
    for y in 0..5 {
        for x in 0..5 {
            if moved_block[(y * 5 + x) as usize] > 0 {
                // フィールドの下端に重なる部分はないか
                if (Y + y - 2) == FIELD_Y {
                    can_turn = false;
                }
                // フィールドの左端に重なる部分はないか
                if (X + x - 2) == -1 {
                    can_turn = false;
                }
                // フィールドの右端に重なる部分はないか
                if (X + x - 2) == FIELD_X {
                    can_turn = false;
                }
                // フィールドブロックと重なる場所はないか
                let field_px = X + x - 2;
                let field_py = Y + y - 2;
                if 0 <= field_px && field_px < FIELD_X && 0 <= field_py && field_py < FIELD_Y {
                    if FIELD[(field_py * FIELD_X + field_px) as usize] > 0 {
                        can_turn = false;
                    }
                }
            }
        }
    }
    if can_turn {
        i = 0;
        while i < 25 {
            USER_BLOCK[TURN_LEFT[i]] = clone_block[i];
            i = i + 1;
        }
    }
}

/// block turn right
#[no_mangle]
pub unsafe extern "C" fn turn_right() {
    let clone_block = USER_BLOCK.clone();
    let mut moved_block = [0; 25];
    let mut i = 0;
    while i < 25 {
        moved_block[TURN_RIGHT[i]] = clone_block[i];
        i = i + 1;
    }
    // 回転後の位置に移動可能かチェック
    let mut can_turn = true;
    for y in 0..5 {
        for x in 0..5 {
            if moved_block[(y * 5 + x) as usize] > 0 {
                // フィールドの下端に重なる部分はないか
                if (Y + y - 2) == FIELD_Y {
                    can_turn = false;
                }
                // フィールドの左端に重なる部分はないか
                if (X + x - 2) == -1 {
                    can_turn = false;
                }
                // フィールドの右端に重なる部分はないか
                if (X + x - 2) == FIELD_X {
                    can_turn = false;
                }
                // フィールドブロックと重なる場所はないか
                let field_px = X + x - 2;
                let field_py = Y + y - 2;
                if 0 <= field_px && field_px < FIELD_X && 0 <= field_py && field_py < FIELD_Y {
                    if FIELD[(field_py * FIELD_X + field_px) as usize] > 0 {
                        can_turn = false;
                    }
                }
            }
        }
    }
    if can_turn {
        i = 0;
        while i < 25 {
            USER_BLOCK[TURN_RIGHT[i]] = clone_block[i];
            i = i + 1;
        }
    }
}
