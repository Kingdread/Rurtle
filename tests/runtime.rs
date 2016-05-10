extern crate rurtle;
mod support;
use support::image;

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::ffi::OsStr;

const RESULT_DIR: &'static str = "test-results";

#[test]
fn runtime_tests() {
    let files = fs::read_dir("tests/runtime").expect("Couldn't read source dir");
    // Turn the DirEntries into PathBufs
    let files = files.map(|entry| entry.expect("Couldn't get DirEntry").path());
    let (count, fails) = check_files(files);
    println!("Ran {} runtime tests.", count);
    assert!(fails == 0, "{} runtime tests failed.", fails);
}

fn check_files<I: Iterator<Item=PathBuf>>(files: I) -> (u32, u32) {
    fs::create_dir(RESULT_DIR).unwrap_or(());
    let report_path = PathBuf::from(RESULT_DIR).join(String::from("test-result"));
    let mut report = File::create(report_path).unwrap();
    let mut count = 0;
    let mut fails = 0;
    for file in files.filter(|path| path.extension() == Some(String::from("rtl").as_ref())) {
        print!("Testing {} ... ", file.display());
        write!(&mut report, "{}/",
               file.file_name().and_then(OsStr::to_str).unwrap()).unwrap();
        let result_path;
        match check_file(file) {
            Ok(path) => {
                println!("Passed");
                write!(&mut report, "ok/").unwrap();
                result_path = path;
            },
            Err(path) => {
                fails += 1;
                println!("       Failed, output saved as {}", path.display());
                write!(&mut report, "fail/").unwrap();
                result_path = path;
            },
        }
        writeln!(&mut report, "{}",
                 result_path.file_name().and_then(OsStr::to_str).unwrap()).unwrap();
        count += 1;
    }
    (count, fails)
}

fn check_file(mut name: PathBuf) -> Result<PathBuf, PathBuf> {
    // Open and read the test source file
    let mut file = fs::File::open(&name).expect("Can't open file to test");
    let mut source = String::new();
    file.read_to_string(&mut source).expect("Can't read file");

    // Evaluate the source in a new environment
    let mut environ = support::environ();
    environ.get_turtle().hide();
    environ.eval_source(&source).expect("Rurtle runtime error");

    // Get the screenshot
    let mut screen = environ.get_turtle().get_screen();
    screen.draw_and_update();
    let mut shot = screen.screenshot();

    // Open the reference image
    name.set_extension("png");
    let mut reference = image::open(&name).expect("Can't open reference file");
    let okay = support::image_eq(&mut shot, &mut reference);

    // Save the output to review it
    let result_path = PathBuf::from(RESULT_DIR).join(name.file_name().unwrap());
    let mut result_file = fs::File::create(&result_path).expect("Can't write test output");
    shot.save(&mut result_file, image::PNG).expect("Can't write test output");

    if okay {
        Ok(result_path)
    } else {
        Err(result_path)
    }
}
