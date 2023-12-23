use image::imageops::thumbnail;
use image::{open, Luma};
use imageproc::definitions::{HasBlack, HasWhite};
use imageproc::map::map_pixels;
use std::path::Path;
use std::process::Command;

pub fn take_photo() {
    // 创建 `Command` 实例
    let mut command = Command::new("rpicam-jpeg");

    // 设置参数
    command.arg("-o").arg("test.jpg");
    command.arg("-t").arg("2000");
    command.arg("--width").arg("640");
    command.arg("--height").arg("480");

    // 运行命令
    let mut child = command.spawn().unwrap();

    // 等待命令执行完成
    child.wait().unwrap();
}

pub fn ana() -> (u32, u32) {
    let input_path = Path::new("./test.jpg");
    const SHARP: u32 = 15;
    // 插入图片并灰度化 阈值应该为170
    let input_image = open(input_path)
        .expect(&format!("无法加载图像 {:?}", input_path))
        .to_luma8();
    let input_image = thumbnail(&input_image, SHARP, SHARP);
    let threshold_img = map_pixels(&input_image, |x, _, p| {
        if p.0[0] > 170 {
            Luma::<u8>::black()
        } else {
            // flag += 1;
            // if flag == 10 {
            //     position.push(x);
            //     flag = 0;
            // }
            Luma::<u8>::white()
        }
    });

    //left
    let res = threshold_img.into_vec();
    for (i, x) in res.iter().enumerate() {
        print!(
            "{}",
            match x {
                255 => "⬛",
                _ => "⬜",
            }
        );
        if i as u32 % SHARP == SHARP - 1 {
            println!();
        };
    }
    let mut left = 0;
    for i in 0..SHARP / 2 {
        for row in 0..SHARP {
            if res[(i + row * SHARP) as usize] == 255 {
                left += SHARP - row;
            }
        }
    }
    let mut right = 0;
    for i in SHARP / 2..SHARP {
        for row in 0..SHARP {
            if res[(i + row * SHARP) as usize] == 255 {
                right += SHARP - row;
            }
        }
    }
    println!("{}|{}", left, right);
    (left, right)
}
