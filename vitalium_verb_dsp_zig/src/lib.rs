extern crate libc;
use libc::c_int;

#[link(name = "vitalium_verb_dsp_zig", kind = "static")]
extern "C" {
    fn add(a: c_int, b: c_int) -> c_int;
}

pub fn add_it(a: i32, b: i32) -> i32 {
    unsafe { add(a as c_int, b as c_int) as i32 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add_it(2, 4);
        assert_eq!(result, 6);
    }
}
