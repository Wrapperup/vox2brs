use std::fs::File;
use std::path::PathBuf;
use brickadia::{
    save::{BrickOwner, SaveData},
    write::SaveWriter,
};
use brickadia::save::{User};
use create_vox::{VoxFile};
use clap::{Parser};
use vox2brs::{vox2brs, BrickOutputMode};

fn valid_brs_path(string: &str) -> Result<PathBuf, &'static str> {
    if !string.ends_with(".brs") {
        return Err("Invalid path to brs.");
    }
    Ok(string.into())
}

fn valid_vox_path(string: &str) -> Result<PathBuf, &'static str> {
    if !string.ends_with(".vox") {
        return Err("Invalid path to vox.");
    }
    let path: PathBuf = string.into();
    if path.exists() {
        return Ok(path);
    }
    Err("Input file doesn't exist.")
}

/// Convert MagicaVoxel models into a BRS file.
#[derive(Parser, Debug)]
struct Args {
    /// Input path to .vox file.
    #[clap(required = true, parse(try_from_str = valid_vox_path))]
    input: PathBuf,

    /// Output directory of the converted .brs file.
    #[clap(required = true, parse(try_from_str = valid_brs_path))]
    output: PathBuf,

    /// How voxels are interpreted.
    #[clap(arg_enum, default_value_t = BrickOutputMode::Brick)]
    mode: BrickOutputMode,

    /// Width of the output brick.
    width: Option<u32>,

    /// Height of the output brick.
    height: Option<u32>,

    /// Should we run the simplifier?
    #[clap(short, long)]
    simplify: bool,

    /// Run rampifier?
    #[clap(short, long)]
    rampify: bool,
}

fn main() -> Result<(), &'static str> {
    let args = Args::parse();

    let public = User {
        name: "vox2brs".into(),
        id: "a8033bee-6c37-4118-b4a6-cecc1d966133".parse().unwrap(),
    };

    let mut save = SaveData::default();

    // set the first header
    save.header1.author = public.clone();
    save.header1.host = Some(public.clone());
    save.header1.description = "Converted .vox file.".into();

    // set the second header
    save.header2
        .brick_owners
        .push(BrickOwner::from_user_bricks(public.clone(), 100));

    save.header2.brick_assets =
        vec![
            "PB_DefaultBrick".into(),
            "PB_DefaultMicroBrick".into(),
            "PB_DefaultRamp".into(),
            "PB_DefaultWedge".into(),
        ];

    // In case this changes in the future... it should already be empty.
    save.header2.colors.clear();

    let vox_data = VoxFile::load(&args.input.into_os_string().into_string().unwrap());

    let result = vox2brs(vox_data, save, args.mode, args.width, args.height, args.simplify, args.rampify, 0, 1, 2, 3);

    match result {
        Ok(out_save) => {
            println!("\nWriting save file...");
            let file = File::create(&args.output);

            match file {
                Ok(file) => {
                    SaveWriter::new(file, out_save)
                        .write()
                        .unwrap();

                    println!("Save written to {}", args.output.into_os_string().into_string().unwrap());
                },
                Err(error) => {
                    println!("Could not write to {}, {}", args.output.into_os_string().into_string().unwrap(), error.to_string())
                }
            }

            Ok(())
        },
        Err(_) => {
            Err("Could not convert vox to brs.")
        }
    }
}