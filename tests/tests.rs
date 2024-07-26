use libloading::{Library, Symbol};

#[test]
fn smoke_test() {
    let path = test_cdylib::build_current_project();
    let _ = unsafe { Library::new(path) }.unwrap();
}
