fn main() {
    slint_build::compile("ui/main.slint")
        .expect("Could not build slint files before the start of the rust program");
}
