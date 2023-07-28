pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }

    // read app_num
    unsafe { (_num_app as *const usize).read_volatile() }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    extern "C" {
        fn _num_app();
    }

    let num_app_ptr = _num_app as *const usize;
    let num_app = get_num_app();

    let app_info_raw =
        unsafe { core::slice::from_raw_parts(num_app_ptr.add(1) as *const usize, num_app + 1) };

    assert!(app_id < num_app);

    unsafe {
        core::slice::from_raw_parts(
            app_info_raw[app_id] as *const u8,
            app_info_raw[app_id + 1] - app_info_raw[app_id],
        )
    }
}
