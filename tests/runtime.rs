extern crate rurtle;
mod support;
use support::image;

use std::fs;
use std::io::Read;
use std::path::PathBuf;

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
    let mut count = 0;
    let mut fails = 0;
    for file in files.filter(|path| path.extension() == Some(String::from("rtl").as_ref())) {
        println!("Testing {}...", file.display());
        if let Err(path) = check_file(file) {
            fails += 1;
            println!("    Failed, output saved as {}", path.display())
        }
        count += 1;
    }
    (count, fails)
}

fn check_file(mut name: PathBuf) -> Result<(), PathBuf> {
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
    let shot = screen.screenshot();

    // Open the reference image
    name.set_extension("png");
    let reference = image::open(&name).expect("Can't open reference file");
    let okay = support::image_eq(&shot, &reference);
    if okay {
        return Ok(())
    };

    // Save the wrong output to review it
    fs::create_dir("test-results").unwrap_or(());
    let result_path = PathBuf::from("test-results/").join(name.file_name().unwrap());
    let mut result_file = fs::File::create(&result_path).expect("Can't write test output");
    shot.save(&mut result_file, image::PNG).expect("Can't write test output");
    Err(result_path)
}
