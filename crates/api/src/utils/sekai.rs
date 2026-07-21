#![allow(unused)]

//! 星界变换与逆变换实现
//!
//! 提供对 64 位和 32 位无符号整数进行可逆位运算变换的函数。
//!
//! 包含星界变换（sekai_transform）、逆星界变换（isekai_transform）、
//! 半星界变换（half_sekai_transform）和半逆星界变换（half_isekai_transform）。
//!
//! 这些变换通过复杂的位运算操作实现数据的混淆和还原，具有可逆性和雪崩效应。

/// 星界变换：对 64 位无符号整数进行可逆的位运算变换
///
/// 通过 64 轮复杂的位运算操作对输入值进行混淆变换。
///
/// # 参数
///
/// * `value` - 待变换的 64 位无符号整数
///
/// # 返回值
///
/// 返回经过星界变换后的 64 位无符号整数
///
/// # 示例
///
/// ```rust
/// let original = 0x1234567890ABCDEF;
/// let transformed = sekai_transform(original);
/// let recovered = isekai_transform(transformed);
/// assert_eq!(original, recovered);
/// ```
pub fn sekai_transform(value: u64) -> u64 {
    let mut n = value;
    for _ in 0..64 {
        n = n.rotate_left(1);
        n ^= 0x0000_0000_0000_1fff;
        let f7 = n & 0x007f;
        let e7 = n & 0xfe00_0000_0000_0000;

        n = (n & !0xfe00_0000_0000_007f) | (f7 << 57) | (e7 >> 57);
        if (n >> 63) ^ (n & 1) == 1 {
            // Acid Salt 7sRef DC1
            n ^= 0xAc1d_5a17_75Fef_dc1;
        } else {
            n ^= 0xE235_8620_40AF_5033;
        }
    }

    n
}

/// 逆星界变换：对经过星界变换的 64 位无符号整数进行还原
///
/// 通过与星界变换相反的位运算操作对输入值进行还原。
///
/// # 参数
///
/// * `value` - 待还原的 64 位无符号整数
///
/// # 返回值
///
/// 返回经过逆星界变换后的 64 位无符号整数
///
/// # 示例
///
/// ```rust
/// let original = 0x1234567890ABCDEF;
/// let transformed = sekai_transform(original);
/// let recovered = isekai_transform(transformed);
/// assert_eq!(original, recovered);
/// ```
pub fn isekai_transform(value: u64) -> u64 {
    let mut n = value;
    for _ in 0..64 {
        if (n >> 63) ^ (n & 1) == 1 {
            n ^= 0xAc1d_5a17_75Fef_dc1;
        } else {
            n ^= 0xE235_8620_40AF_5033;
        }

        let f7 = n & 0x007f;
        let e7 = n & 0xfe00_0000_0000_0000;

        n = (n & !0xfe00_0000_0000_007f) | (f7 << 57) | (e7 >> 57);
        n ^= 0x0000_0000_0000_1fff;
        n = n.rotate_right(1);
    }

    n
}

/// 半星界变换：对 32 位无符号整数进行可逆的位运算变换
///
/// 通过 32 轮复杂的位运算操作对输入值进行混淆变换。
///
/// # 参数
///
/// * `value` - 待变换的 32 位无符号整数
///
/// # 返回值
///
/// 返回经过半星界变换后的 32 位无符号整数
///
/// # 示例
///
/// ```rust
/// let original = 0x12345678;
/// let transformed = half_sekai_transform(original);
/// let recovered = half_isekai_transform(transformed);
/// assert_eq!(original, recovered);
/// ```
pub fn half_sekai_transform(value: u32) -> u32 {
    let mut n = value;
    for _ in 0..32 {
        n = n.rotate_left(1);
        n ^= 0x0000_1fff;
        let f7 = n & 0x007f;
        let e7 = n & 0xfe00_0000;

        n = (n & !0xfe00_007f) | (f7 << 25) | (e7 >> 25);
        if (n >> 31) ^ (n & 1) == 1 {
            // Acid Salt 7sRef DC1
            n ^= 0xAc1d_5a17;
        } else {
            n ^= 0xE235_8620;
        }
    }

    n
}

/// 半逆星界变换：对经过半星界变换的 32 位无符号整数进行还原
///
/// 通过与半星界变换相反的位运算操作对输入值进行还原。
///
/// # 参数
///
/// * `value` - 待还原的 32 位无符号整数
///
/// # 返回值
///
/// 返回经过半逆星界变换后的 32 位无符号整数
///
/// # 示例
///
/// ```rust
/// let original = 0x12345678;
/// let transformed = half_sekai_transform(original);
/// let recovered = half_isekai_transform(transformed);
/// assert_eq!(original, recovered);
/// ```

pub fn half_isekai_transform(value: u32) -> u32 {
    let mut n = value;
    for _ in 0..32 {
        if (n >> 31) ^ (n & 1) == 1 {
            n ^= 0xAc1d_5a17;
        } else {
            n ^= 0xE235_8620;
        }

        let f7 = n & 0x007f;
        let e7 = n & 0xfe00_0000;

        n = (n & !0xfe00_007f) | (f7 << 25) | (e7 >> 25);
        n = n.rotate_right(1);
    }

    n
}

mod test {
    use crate::utils::sekai::*;
    #[test]
    fn test1() {
        let original = 0x1234567890ABCDEFu64;
        let transformed = sekai_transform(original);
        let recovered = isekai_transform(transformed);
        assert_eq!(original, recovered);
        assert_ne!(transformed, original);
    }
}
