use libloading::Library;

#[test]
#[cfg_attr(miri, ignore)]
fn smoke_test() {
    let path = test_cdylib::build_current_project();
    let _ = unsafe { Library::new(path) }.unwrap();
}
