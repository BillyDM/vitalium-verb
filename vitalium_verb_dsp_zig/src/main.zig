const std = @import("std");

export fn add(a: c_int, b: c_int) callconv(.C) c_int {
    return a + b;
}
