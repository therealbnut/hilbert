pub fn xy2d(x: u32, y: u32) -> u64 {
    let mut xy = (x, y);
    let mut rx: u32;
    let mut ry: u32;
    let mut s: u32 = 1 << 31;
    let mut d: u64 = 0;
    while s > 0 {
        rx = if (xy.0 & s) > 0 { 1 } else { 0 };
        ry = if (xy.1 & s) > 0 { 1 } else { 0 };
        d += (s as u64) * (s as u64) * ((3 * rx) ^ ry) as u64;
        xy = rot_b(xy, (rx, ry));
        s /= 2;
    }
    return d;
}

#[inline]
fn rot_b((mut x, mut y): (u32, u32), (rx, ry): (u32, u32)) -> (u32, u32) {
    if ry == 0 {
        if rx == 1 {
            x = !x;
            y = !y;
        }
        std::mem::swap(&mut x, &mut y);
    }
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hilbert::HilbertPrecompute;

    #[test]
    fn test_brute_distance() {
        let n = 32;

        for x0 in 0..n {
            for y0 in 0..n {
                let d0 = xy2d(x0, y0);
                let precompute = HilbertPrecompute::new(x0, y0);

                assert_eq!(d0, precompute.distance());
            }
        }
    }

    #[test]
    fn test_brute_compare() {
        let n = 32;

        for x0 in 0..n {
            for y0 in 0..n {
                let d0 = xy2d(x0, y0);
                let precompute = HilbertPrecompute::new(x0, y0);

                for x1 in 0..n {
                    for y1 in 0..n {
                        let d1 = xy2d(x1, y1);
                        assert_eq!(d0.partial_cmp(&d1), precompute.partial_cmp(&(x1, y1)));
                    }
                }
            }
        }
    }
}
