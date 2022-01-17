use std::cmp::min;
use std::fs::File;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;
use brickadia::{
    save::{BrickOwner, SaveData},
    write::SaveWriter,
};
use clap::{ArgEnum};
use brickadia::save::{Brick, BrickColor, Color, Size, User};
use create_vox::{Model, VoxFile};
use rampifier::{Rampifier, RampifierConfig};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum BrickOutputMode {
    /// Default 1x1 brick.
    Brick,

    /// Default 1x1f plate brick.
    Plate,

    /// Default 1x1x1 micro brick.
    MicroBrick,
}

fn gamma_correction(r: u8, g: u8, b: u8) -> (u8, u8, u8) {

    let r = (r as f32) / 255.0;
    let g = (g as f32) / 255.0;
    let b = (b as f32) / 255.0;

    let r = r.powf(2.2);
    let g = g.powf(2.2);
    let b = b.powf(2.2);

    let r = (r * 255.0) as u8;
    let g = (g * 255.0) as u8;
    let b = (b * 255.0) as u8;

    (r, g, b)
}

pub fn vox2brs(
    in_vox_data: VoxFile,
    mut brs_save: SaveData,
    mode: BrickOutputMode,
    width: Option<u32>,
    height: Option<u32>,
    simplify: bool,
    rampify: bool,
    brick_asset_index: u32,
    microbrick_asset_index: u32,
    ramp_asset_index: u32,
    wedge_asset_index: u32,
) -> Result<SaveData, ()> {
    let now = Instant::now();

    println!("Running vox2brs...");
    println!("Loading colors...");

    // Add voxel colors to brickadia save color palette.
    for vox_color in in_vox_data.palette {
        // Color correction
        let rgb = gamma_correction(vox_color.r, vox_color.g, vox_color.b);

        let brs_color = Color {
            r: rgb.0,
            g: rgb.1,
            b: rgb.2,
            a: 255,
        };

        brs_save.header2.colors.push(brs_color);
    }

    println!(" - Done\n");

    let (brick_size, brick_asset): ((u32, u32), u32) = match mode {
        BrickOutputMode::Brick => {
            let w = width.unwrap_or(1) * 5;
            let h = height.unwrap_or(3) * 6;
            ((w, h), brick_asset_index)
        },
        BrickOutputMode::Plate => {
            let w = width.unwrap_or(1) * 5;
            let h = height.unwrap_or(1) * 2;
            ((w, h), brick_asset_index)
        },
        BrickOutputMode::MicroBrick => {
            let w = width.unwrap_or(1);
            let h = height.unwrap_or(1);
            ((w, h), microbrick_asset_index)
        },
    };

    println!("Converting voxels into bricks...");

    let models_len = in_vox_data.models.len() + in_vox_data.copies.len();

    fn row_major_rotation(pos: (i32, i32, i32), rotation_byte: u8, debug: bool) -> (i32, i32, i32) {
        let (x, y, z) = pos;

        let r1_i = (rotation_byte >> 0) & 0b11;
        let r2_i = (rotation_byte >> 2) & 0b11;

        let r1_sign = if (rotation_byte >> 4) & 0b1 == 1 { -1 } else { 1 };
        let r2_sign = if (rotation_byte >> 5) & 0b1 == 1 { -1 } else { 1 };
        let r3_sign = if (rotation_byte >> 6) & 0b1 == 1 { -1 } else { 1 };

        let mut m = (
            [0, 0, 0],
            [0, 0, 0],
            [r3_sign, r3_sign, r3_sign],
        );

        m.0[r1_i as usize] = r1_sign;
        m.1[r2_i as usize] = r2_sign;
        m.2[r1_i as usize] = 0;
        m.2[r2_i as usize] = 0;

        // Gotta do some funky stuff here because the coordinate system is different in brs...
        let x_prime =  x * m.0[0] +  y * m.0[1] +  z * m.0[2];
        let y_prime =  x * m.1[0] +  y * m.1[1] +  z * m.1[2];
        let z_prime =  x * m.2[0] +  y * m.2[1] +  z * m.2[2];

        (x_prime, y_prime, z_prime)
    }

    let model_to_bricks = |model: &Model, pos: (i32, i32, i32), rot_option: Option<u8>, bricks: &mut Vec<Brick>| {
        let size = (model.size.0 as i32, model.size.1 as i32, model.size.2 as i32);

        if let Some(rot) = rot_option {
            row_major_rotation(pos, rot, true);
            println!("model rotation: {:#016b}", rot);
        }

        for voxel in model.voxels.iter() {

            let mut vox_pos = (
                voxel.position.0 as i32 - size.0 / 2,
                voxel.position.1 as i32 - size.1 / 2,
                voxel.position.2 as i32 - size.2 / 2
            );

            if let Some(rot) = rot_option {
                vox_pos = row_major_rotation(vox_pos, rot, false);
            }

            let pos = (
                vox_pos.0 + pos.0,
                vox_pos.1 + pos.1,
                vox_pos.2 + pos.2,
            );

            let mut brick = Brick::default();
            brick.size = Size::Procedural(brick_size.0, brick_size.0, brick_size.1);
            brick.asset_name_index = brick_asset;

            brick.position = (
                pos.0 * brick_size.0 as i32 * 2 + brick_size.0 as i32,
                -pos.1 * brick_size.0 as i32 * 2 + brick_size.0 as i32,
                pos.2 * brick_size.1 as i32 * 2 + brick_size.1 as i32,
            );

            brick.color = BrickColor::Index(voxel.color_index as u32 - 1);

            brick.owner_index = 1;

            bricks.push(brick);
        }
    };

    for model in in_vox_data.models.iter() {
        let pos = model.position.unwrap_or((0, 0, 0));
        model_to_bricks(model, pos, model.rotation, &mut brs_save.bricks);
    }

    for model_copy in in_vox_data.copies.iter() {
        if let Some(model) = in_vox_data.get_model_by_id(model_copy.model_id) {
            let pos = model_copy.position.unwrap_or((0, 0, 0));
            model_to_bricks(model, pos, model_copy.rotation, &mut brs_save.bricks);
        }
    }

    println!(" - Read {} models.", models_len);

    // I ripped this from rampifier because I'm lazy. Too bad!
    if simplify || rampify {
        // Move brick vector so we can re-write the optimized version into the save.
        let bricks = brs_save.bricks;
        brs_save.bricks = vec![];

        println!("\nSimplifying BRS...");

        let brick_size = if rampify {
            (5, 2)
        }
        else {
            (brick_size.0 as i32, brick_size.1 as i32)
        };

        let fix_brick_pos = |brick: &Brick| -> (i32, i32, i32) {
            let (mut x, mut  y, mut  z) = brick.position;
            if let Size::Procedural(w_half, l_half, h_half) = brick.size {
                x -= w_half as i32;
                y -= l_half as i32;
                z -= h_half as i32;

                x /= brick_size.0 * 2;
                y /= brick_size.0 * 2;
                z /= brick_size.1 * 2;

                return (x, y, z);
            }

            (0, 0, 0)
        };

        // Find bounds for bricks.
        let mut min_bounds = (i32::MAX, i32::MAX, i32::MAX);
        let mut max_bounds = (i32::MIN, i32::MIN, i32::MIN);

        for brick in &bricks {
            if let Size::Procedural(w_half, l_half, h_half) = brick.size {
                let w = w_half as i32 / brick_size.0;
                let l = l_half as i32 / brick_size.0;
                let h = h_half as i32 / brick_size.1;

                let pos = fix_brick_pos(&brick);

                min_bounds.0 = min_bounds.0.min(pos.0);
                min_bounds.1 = min_bounds.1.min(pos.1);
                min_bounds.2 = min_bounds.2.min(pos.2);

                let pos = (
                    pos.0 + w + 1,
                    pos.1 + l + 1,
                    pos.2 + h + 1,
                );

                max_bounds.0 = max_bounds.0.max(pos.0);
                max_bounds.1 = max_bounds.1.max(pos.1);
                max_bounds.2 = max_bounds.2.max(pos.2);
            }
        }

        let grid_size = (
            (max_bounds.0 - min_bounds.0) as usize,
            (max_bounds.1 - min_bounds.1) as usize,
            (max_bounds.2 - min_bounds.2) as usize,
        );

        let get_index = |pos: (usize, usize, usize)| -> usize {
            pos.0 + pos.1 * grid_size.0 + pos.2 * grid_size.0 * grid_size.1
        };

        let mut grid: Vec<Option<u8>> = vec![None; grid_size.0 * grid_size.1 * grid_size.2];

        for brick in &bricks {
            if let Size::Procedural(w_half, l_half, h_half) = brick.size {
                let pos = fix_brick_pos(&brick);
                let pos = (
                    (pos.0 - min_bounds.0) as usize,
                    (pos.1 - min_bounds.1) as usize,
                    (pos.2 - min_bounds.2) as usize,
                );

                let w = w_half as usize / brick_size.0 as usize;
                let l = l_half as usize / brick_size.0 as usize;
                let h = h_half as usize / brick_size.1 as usize;

                for i in 0..w {
                    for j in 0..l {
                        for k in 0..h {
                            let pos = (pos.0 + i, pos.1 + j, pos.2 + k);

                            if let BrickColor::Index(index) = brick.color {
                                grid[get_index(pos)] = Some(index as u8);
                            }
                        }
                    }
                }
            }
        }

        if rampify {
            println!("Generating ramps...");

            let vox_count = grid.len();

            let rampifier_config = RampifierConfig {
                ramp_index: ramp_asset_index,
                wedge_index: wedge_asset_index,
                ..RampifierConfig::default()
            };

            let mut rampifier = Rampifier::new(
                grid_size,
                grid,
                rampifier_config
            );

            let now = Instant::now();

            // Generate ramps for floor and ceiling.
            let ramps = &mut rampifier.generate_ramps(true);
            let ramps2 = &mut rampifier.generate_ramps(false);

            let ramp_count = ramps.len();
            let ramp2_count = ramps2.len();

            brs_save.bricks.append(ramps);
            brs_save.bricks.append(ramps2);

            println!(" - Processed {} voxels", vox_count);
            println!(" - Generated {} ramps in {}s\n", ramp_count + ramp2_count, now.elapsed().as_millis() as f64 / 1000.0);

            // Sets the voxels occupied by ramps to empty.
            rampifier.remove_occupied_voxels();

            // Move grid back out of the rampifier to do further processing.
            grid = rampifier.move_grid();
        }

        let box_remove = |g: &mut Vec<Option<u8>>, pos: &(usize, usize, usize), size: &(usize, usize, usize)| {
            let &(x, y, z) = pos;
            let &(w, l, h) = size;

            for i in 0..w {
                for j in 0..l {
                    for k in 0..h {
                        let p = (x + i, y + j, z + k);

                        g[get_index((p.0, p.1, p.2))] = None;
                    }
                }
            }
        };

        let can_box = |g: &Vec<Option<u8>>, value: u8, pos: &(usize, usize, usize), size: &(usize, usize, usize)| -> bool {
            let &(w, l, h) = size;

            if pos.0 + w > grid_size.0 {
                return false;
            }
            if pos.1 + l > grid_size.1 {
                return false;
            }
            if pos.2 + h > grid_size.2 {
                return false;
            }

            for i in 0..w {
                for j in 0..l {
                    for k in 0..h {
                        let pos = (pos.0 + i, pos.1 + j, pos.2 + k);
                        if g[get_index((pos.0, pos.1, pos.2))] != Some(value) {
                            return false;
                        }
                    }
                }
            }

            return true;
        };

        println!("\nFilling Gaps...");

        for x in 0..grid_size.0 {
            for y in 0..grid_size.1 {
                for z in 0..grid_size.2 {
                    let mut brick = Brick::default();

                    if let Some(val) = grid[get_index((x, y, z))] {
                        let mut w = 1;
                        let mut l = 1;
                        let mut h = 1;

                        while can_box(&grid, val, &(x, y, z), &(w, l, h)) && h <= 64 {
                            h += 1;
                        }

                        h -= 1;

                        if h > 0 {
                            while can_box(&grid, val, &(x, y, z), &(w, l, h)) && w <= 64 {
                                w += 1;
                            }

                            w -= 1;

                            if w > 0 {
                                while can_box(&grid, val, &(x, y, z), &(w, l, h)) && l <= 64 {
                                    l += 1;
                                }

                                l -= 1;

                                if l > 0 {
                                    box_remove(&mut grid, &(x, y, z), &(w, l, h));

                                    let size = (w as u32 * brick_size.0 as u32, l as u32 * brick_size.0 as u32, h as u32 * brick_size.1 as u32);
                                    {
                                        let (x, y, z) = (x as i32 * brick_size.0 * 2, y as i32 * brick_size.0 * 2, z as i32 * brick_size.1 * 2);

                                        brick.position = (
                                            x + size.0 as i32,
                                            y + size.1 as i32,
                                            z + size.2 as i32
                                        );

                                        brick.size = Size::Procedural(size.0, size.1, size.2);
                                    }

                                    brick.color = BrickColor::Index(val as u32);
                                    brick.asset_name_index = brick_asset;
                                    brs_save.bricks.push(brick);
                                }
                            }
                        }
                    }
                }
            }
        }

        for mut brick in &mut brs_save.bricks {
            brick.position.0 += min_bounds.0 * brick_size.0 * 2;
            brick.position.1 += min_bounds.1 * brick_size.0 * 2;
            brick.position.2 += min_bounds.2 * brick_size.1 * 2;
        }

        println!(" - Gaps filled.");
    }

    println!("\nFinished vox2brs in {}s.", now.elapsed().as_millis() as f64 / 1000.0);
    println!(" - Created {} bricks.", brs_save.bricks.len());

    Ok(brs_save)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
