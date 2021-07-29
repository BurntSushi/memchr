use crate::vector::Vector;
use core::cmp;

// The number of elements to loop at in one iteration of memchr/memrchr.
const LOOP_AMT: usize = 4;

// The number of elements to loop at in one iteration of memchr2/memrchr2 and
// memchr3/memrchr3. There was no observable difference between 64 and 32 bytes
// in benchmarks. memchr3 in particular only gets a very slight speed up from
// the loop unrolling.
const LOOP_AMT2: usize = 2;

#[inline(always)]
pub(crate) unsafe fn memchr<V: Vector>(
    n1: u8,
    haystack: &[u8],
) -> Option<usize> {
    // What follows is a fast SSE2-only algorithm to detect the position of
    // `n1` in `haystack` if it exists. From what I know, this is the "classic"
    // algorithm. I believe it can be found in places like glibc and Go's
    // standard library. It appears to be well known and is elaborated on in
    // more detail here: https://gms.tf/stdfind-and-memchr-optimizations.html
    //
    // While this routine is very long, the basic idea is actually very simple
    // and can be expressed straight-forwardly in pseudo code:
    //
    //     needle = (n1 << 15) | (n1 << 14) | ... | (n1 << 1) | n1
    //     // Note: shift amount in bytes
    //
    //     while i <= haystack.len() - 16:
    //       // A 16 byte vector. Each byte in chunk corresponds to a byte in
    //       // the haystack.
    //       chunk = haystack[i:i+16]
    //       // Compare bytes in needle with bytes in chunk. The result is a 16
    //       // byte chunk where each byte is 0xFF if the corresponding bytes
    //       // in needle and chunk were equal, or 0x00 otherwise.
    //       eqs = cmpeq(needle, chunk)
    //       // Return a 32 bit integer where the most significant 16 bits
    //       // are always 0 and the lower 16 bits correspond to whether the
    //       // most significant bit in the correspond byte in `eqs` is set.
    //       // In other words, `mask as u16` has bit i set if and only if
    //       // needle[i] == chunk[i].
    //       mask = movemask(eqs)
    //
    //       // Mask is 0 if there is no match, and non-zero otherwise.
    //       if mask != 0:
    //         // trailing_zeros tells us the position of the least significant
    //         // bit that is set.
    //         return i + trailing_zeros(mask)
    //
    //     // haystack length may not be a multiple of 16, so search the rest.
    //     while i < haystack.len():
    //       if haystack[i] == n1:
    //         return i
    //
    //     // No match found.
    //     return NULL
    //
    // In fact, we could loosely translate the above code to Rust line-for-line
    // and it would be a pretty fast algorithm. But, we pull out all the stops
    // to go as fast as possible:
    //
    // 1. We use aligned loads. That is, we do some finagling to make sure our
    //    primary loop not only proceeds in increments of 16 bytes, but that
    //    the address of haystack's pointer that we dereference is aligned to
    //    16 bytes. 16 is a magic number here because it is the size of SSE2
    //    128-bit vector. (For the AVX2 algorithm, 32 is the magic number.)
    //    Therefore, to get aligned loads, our pointer's address must be evenly
    //    divisible by 16.
    // 2. Our primary loop proceeds 64 bytes at a time instead of 16. It's
    //    kind of like loop unrolling, but we combine the equality comparisons
    //    using a vector OR such that we only need to extract a single mask to
    //    determine whether a match exists or not. If so, then we do some
    //    book-keeping to determine the precise location but otherwise mush on.
    // 3. We use our "chunk" comparison routine in as many places as possible,
    //    even if it means using unaligned loads. In particular, if haystack
    //    starts with an unaligned address, then we do an unaligned load to
    //    search the first 16 bytes. We then start our primary loop at the
    //    smallest subsequent aligned address, which will actually overlap with
    //    previously searched bytes. But we're OK with that. We do a similar
    //    dance at the end of our primary loop. Finally, to avoid a
    //    byte-at-a-time loop at the end, we do a final 16 byte unaligned load
    //    that may overlap with a previous load. This is OK because it converts
    //    a loop into a small number of very fast vector instructions.
    //
    // The primary downside of this algorithm is that it's effectively
    // completely unsafe. Therefore, we have to be super careful to avoid
    // undefined behavior:
    //
    // 1. We use raw pointers everywhere. Not only does dereferencing a pointer
    //    require the pointer to be valid, but we actually can't even store the
    //    address of an invalid pointer (unless it's 1 past the end of
    //    haystack) without sacrificing performance.
    // 2. _mm_loadu_si128 is used when you don't care about alignment, and
    //    _mm_load_si128 is used when you do care. You cannot use the latter
    //    on unaligned pointers.
    // 3. We make liberal use of debug_assert! to check assumptions.
    // 4. We make a concerted effort to stick with pointers instead of indices.
    //    Indices are nicer because there's less to worry about with them (see
    //    above about pointer offsets), but I could not get the compiler to
    //    produce as good of code as what the below produces. In any case,
    //    pointers are what we really care about here, and alignment is
    //    expressed a bit more naturally with them.
    //
    // In general, most of the algorithms in this crate have a similar
    // structure to what you see below, so this comment applies fairly well to
    // all of them.

    let vn1 = V::splat(n1);
    let len = haystack.len();
    let loop_size = cmp::min(V::size() * LOOP_AMT, len);
    let start_ptr = haystack.as_ptr();
    let end_ptr = start_ptr.add(haystack.len());
    let mut ptr = start_ptr;

    if haystack.len() < V::size() {
        while ptr < end_ptr {
            if *ptr == n1 {
                return Some(sub(ptr, start_ptr));
            }
            ptr = ptr.offset(1);
        }
        return None;
    }

    if let Some(i) = forward_search1(start_ptr, end_ptr, ptr, vn1) {
        return Some(i);
    }

    ptr = ptr.add(V::size() - (start_ptr as usize & V::align_mask()));
    debug_assert!(ptr > start_ptr && end_ptr.sub(V::size()) >= start_ptr);
    while loop_size == V::size() * LOOP_AMT && ptr <= end_ptr.sub(loop_size) {
        debug_assert_eq!(0, (ptr as usize) % V::size());

        let a = V::load_aligned(ptr);
        let b = V::load_aligned(ptr.add(V::size()));
        let c = V::load_aligned(ptr.add(2 * V::size()));
        let d = V::load_aligned(ptr.add(3 * V::size()));
        let eqa = vn1.cmpeq(a);
        let eqb = vn1.cmpeq(b);
        let eqc = vn1.cmpeq(c);
        let eqd = vn1.cmpeq(d);
        let or1 = eqa.or(eqb);
        let or2 = eqc.or(eqd);
        let or3 = or1.or(or2);
        if or3.movemask() != 0 {
            let mut at = sub(ptr, start_ptr);
            let mask = eqa.movemask();
            if mask != 0 {
                return Some(at + forward_pos(mask));
            }

            at += V::size();
            let mask = eqb.movemask();
            if mask != 0 {
                return Some(at + forward_pos(mask));
            }

            at += V::size();
            let mask = eqc.movemask();
            if mask != 0 {
                return Some(at + forward_pos(mask));
            }

            at += V::size();
            let mask = eqd.movemask();
            debug_assert!(mask != 0);
            return Some(at + forward_pos(mask));
        }
        ptr = ptr.add(loop_size);
    }
    while ptr <= end_ptr.sub(V::size()) {
        debug_assert!(sub(end_ptr, ptr) >= V::size());

        if let Some(i) = forward_search1(start_ptr, end_ptr, ptr, vn1) {
            return Some(i);
        }
        ptr = ptr.add(V::size());
    }
    if ptr < end_ptr {
        debug_assert!(sub(end_ptr, ptr) < V::size());
        ptr = ptr.sub(V::size() - sub(end_ptr, ptr));
        debug_assert_eq!(sub(end_ptr, ptr), V::size());

        return forward_search1(start_ptr, end_ptr, ptr, vn1);
    }
    None
}

#[inline(always)]
pub(crate) unsafe fn memchr2<V: Vector>(
    n1: u8,
    n2: u8,
    haystack: &[u8],
) -> Option<usize> {
    let vn1 = V::splat(n1);
    let vn2 = V::splat(n2);
    let len = haystack.len();
    let loop_size = cmp::min(LOOP_AMT2 * V::size(), len);
    let start_ptr = haystack.as_ptr();
    let end_ptr = start_ptr.add(haystack.len());
    let mut ptr = start_ptr;

    if haystack.len() < V::size() {
        while ptr < end_ptr {
            if *ptr == n1 || *ptr == n2 {
                return Some(sub(ptr, start_ptr));
            }
            ptr = ptr.offset(1);
        }
        return None;
    }

    if let Some(i) = forward_search2(start_ptr, end_ptr, ptr, vn1, vn2) {
        return Some(i);
    }

    ptr = ptr.add(V::size() - (start_ptr as usize & V::align_mask()));
    debug_assert!(ptr > start_ptr && end_ptr.sub(V::size()) >= start_ptr);
    while loop_size == LOOP_AMT2 * V::size() && ptr <= end_ptr.sub(loop_size) {
        debug_assert_eq!(0, (ptr as usize) % V::size());

        let a = V::load_aligned(ptr);
        let b = V::load_aligned(ptr.add(V::size()));
        let eqa1 = vn1.cmpeq(a);
        let eqb1 = vn1.cmpeq(b);
        let eqa2 = vn2.cmpeq(a);
        let eqb2 = vn2.cmpeq(b);
        let or1 = eqa1.or(eqb1);
        let or2 = eqa2.or(eqb2);
        let or3 = or1.or(or2);
        if or3.movemask() != 0 {
            let mut at = sub(ptr, start_ptr);
            let mask1 = eqa1.movemask();
            let mask2 = eqa2.movemask();
            if mask1 != 0 || mask2 != 0 {
                return Some(at + forward_pos2(mask1, mask2));
            }

            at += V::size();
            let mask1 = eqb1.movemask();
            let mask2 = eqb2.movemask();
            return Some(at + forward_pos2(mask1, mask2));
        }
        ptr = ptr.add(loop_size);
    }
    while ptr <= end_ptr.sub(V::size()) {
        if let Some(i) = forward_search2(start_ptr, end_ptr, ptr, vn1, vn2) {
            return Some(i);
        }
        ptr = ptr.add(V::size());
    }
    if ptr < end_ptr {
        debug_assert!(sub(end_ptr, ptr) < V::size());
        ptr = ptr.sub(V::size() - sub(end_ptr, ptr));
        debug_assert_eq!(sub(end_ptr, ptr), V::size());

        return forward_search2(start_ptr, end_ptr, ptr, vn1, vn2);
    }
    None
}

#[inline(always)]
pub(crate) unsafe fn memchr3<V: Vector>(
    n1: u8,
    n2: u8,
    n3: u8,
    haystack: &[u8],
) -> Option<usize> {
    let vn1 = V::splat(n1);
    let vn2 = V::splat(n2);
    let vn3 = V::splat(n3);
    let len = haystack.len();
    let loop_size = cmp::min(LOOP_AMT2 * V::size(), len);
    let start_ptr = haystack.as_ptr();
    let end_ptr = start_ptr.add(haystack.len());
    let mut ptr = start_ptr;

    if haystack.len() < V::size() {
        while ptr < end_ptr {
            if *ptr == n1 || *ptr == n2 || *ptr == n3 {
                return Some(sub(ptr, start_ptr));
            }
            ptr = ptr.offset(1);
        }
        return None;
    }

    if let Some(i) = forward_search3(start_ptr, end_ptr, ptr, vn1, vn2, vn3) {
        return Some(i);
    }

    ptr = ptr.add(V::size() - (start_ptr as usize & V::align_mask()));
    debug_assert!(ptr > start_ptr && end_ptr.sub(V::size()) >= start_ptr);
    while loop_size == LOOP_AMT2 * V::size() && ptr <= end_ptr.sub(loop_size) {
        debug_assert_eq!(0, (ptr as usize) % V::size());

        let a = V::load_aligned(ptr);
        let b = V::load_aligned(ptr.add(V::size()));
        let eqa1 = vn1.cmpeq(a);
        let eqb1 = vn1.cmpeq(b);
        let eqa2 = vn2.cmpeq(a);
        let eqb2 = vn2.cmpeq(b);
        let eqa3 = vn3.cmpeq(a);
        let eqb3 = vn3.cmpeq(b);
        let or1 = eqa1.or(eqb1);
        let or2 = eqa2.or(eqb2);
        let or3 = eqa3.or(eqb3);
        let or4 = or1.or(or2);
        let or5 = or3.or(or4);
        if or5.movemask() != 0 {
            let mut at = sub(ptr, start_ptr);
            let mask1 = eqa1.movemask();
            let mask2 = eqa2.movemask();
            let mask3 = eqa3.movemask();
            if mask1 != 0 || mask2 != 0 || mask3 != 0 {
                return Some(at + forward_pos3(mask1, mask2, mask3));
            }

            at += V::size();
            let mask1 = eqb1.movemask();
            let mask2 = eqb2.movemask();
            let mask3 = eqb3.movemask();
            return Some(at + forward_pos3(mask1, mask2, mask3));
        }
        ptr = ptr.add(loop_size);
    }
    while ptr <= end_ptr.sub(V::size()) {
        if let Some(i) =
            forward_search3(start_ptr, end_ptr, ptr, vn1, vn2, vn3)
        {
            return Some(i);
        }
        ptr = ptr.add(V::size());
    }
    if ptr < end_ptr {
        debug_assert!(sub(end_ptr, ptr) < V::size());
        ptr = ptr.sub(V::size() - sub(end_ptr, ptr));
        debug_assert_eq!(sub(end_ptr, ptr), V::size());

        return forward_search3(start_ptr, end_ptr, ptr, vn1, vn2, vn3);
    }
    None
}

#[inline(always)]
pub(crate) unsafe fn memrchr<V: Vector>(
    n1: u8,
    haystack: &[u8],
) -> Option<usize> {
    let vn1 = V::splat(n1);
    let len = haystack.len();
    let loop_size = cmp::min(LOOP_AMT * V::size(), len);
    let start_ptr = haystack.as_ptr();
    let end_ptr = start_ptr.add(haystack.len());
    let mut ptr = end_ptr;

    if haystack.len() < V::size() {
        while ptr > start_ptr {
            ptr = ptr.offset(-1);
            if *ptr == n1 {
                return Some(sub(ptr, start_ptr));
            }
        }
        return None;
    }

    ptr = ptr.sub(V::size());
    if let Some(i) = reverse_search1(start_ptr, end_ptr, ptr, vn1) {
        return Some(i);
    }

    ptr = (end_ptr as usize & !V::align_mask()) as *const u8;
    debug_assert!(start_ptr <= ptr && ptr <= end_ptr);
    while loop_size == LOOP_AMT * V::size() && ptr >= start_ptr.add(loop_size)
    {
        debug_assert_eq!(0, (ptr as usize) % V::size());

        ptr = ptr.sub(loop_size);
        let a = V::load_aligned(ptr);
        let b = V::load_aligned(ptr.add(V::size()));
        let c = V::load_aligned(ptr.add(2 * V::size()));
        let d = V::load_aligned(ptr.add(3 * V::size()));
        let eqa = vn1.cmpeq(a);
        let eqb = vn1.cmpeq(b);
        let eqc = vn1.cmpeq(c);
        let eqd = vn1.cmpeq(d);
        let or1 = eqa.or(eqb);
        let or2 = eqc.or(eqd);
        let or3 = or1.or(or2);
        if or3.movemask() != 0 {
            let mut at = sub(ptr.add(3 * V::size()), start_ptr);
            let mask = eqd.movemask();
            if mask != 0 {
                return Some(at + reverse_pos::<V>(mask));
            }

            at -= V::size();
            let mask = eqc.movemask();
            if mask != 0 {
                return Some(at + reverse_pos::<V>(mask));
            }

            at -= V::size();
            let mask = eqb.movemask();
            if mask != 0 {
                return Some(at + reverse_pos::<V>(mask));
            }

            at -= V::size();
            let mask = eqa.movemask();
            debug_assert!(mask != 0);
            return Some(at + reverse_pos::<V>(mask));
        }
    }
    while ptr >= start_ptr.add(V::size()) {
        ptr = ptr.sub(V::size());
        if let Some(i) = reverse_search1(start_ptr, end_ptr, ptr, vn1) {
            return Some(i);
        }
    }
    if ptr > start_ptr {
        debug_assert!(sub(ptr, start_ptr) < V::size());
        return reverse_search1(start_ptr, end_ptr, start_ptr, vn1);
    }
    None
}

#[inline(always)]
pub(crate) unsafe fn memrchr2<V: Vector>(
    n1: u8,
    n2: u8,
    haystack: &[u8],
) -> Option<usize> {
    let vn1 = V::splat(n1);
    let vn2 = V::splat(n2);
    let len = haystack.len();
    let loop_size = cmp::min(LOOP_AMT2 * V::size(), len);
    let start_ptr = haystack.as_ptr();
    let end_ptr = start_ptr.add(haystack.len());
    let mut ptr = end_ptr;

    if haystack.len() < V::size() {
        while ptr > start_ptr {
            ptr = ptr.offset(-1);
            if *ptr == n1 || *ptr == n2 {
                return Some(sub(ptr, start_ptr));
            }
        }
        return None;
    }

    ptr = ptr.sub(V::size());
    if let Some(i) = reverse_search2(start_ptr, end_ptr, ptr, vn1, vn2) {
        return Some(i);
    }

    ptr = (end_ptr as usize & !V::align_mask()) as *const u8;
    debug_assert!(start_ptr <= ptr && ptr <= end_ptr);
    while loop_size == LOOP_AMT2 * V::size() && ptr >= start_ptr.add(loop_size)
    {
        debug_assert_eq!(0, (ptr as usize) % V::size());

        ptr = ptr.sub(loop_size);
        let a = V::load_aligned(ptr);
        let b = V::load_aligned(ptr.add(V::size()));
        let eqa1 = vn1.cmpeq(a);
        let eqb1 = vn1.cmpeq(b);
        let eqa2 = vn2.cmpeq(a);
        let eqb2 = vn2.cmpeq(b);
        let or1 = eqa1.or(eqb1);
        let or2 = eqa2.or(eqb2);
        let or3 = or1.or(or2);
        if or3.movemask() != 0 {
            let mut at = sub(ptr.add(V::size()), start_ptr);
            let mask1 = eqb1.movemask();
            let mask2 = eqb2.movemask();
            if mask1 != 0 || mask2 != 0 {
                return Some(at + reverse_pos2::<V>(mask1, mask2));
            }

            at -= V::size();
            let mask1 = eqa1.movemask();
            let mask2 = eqa2.movemask();
            return Some(at + reverse_pos2::<V>(mask1, mask2));
        }
    }
    while ptr >= start_ptr.add(V::size()) {
        ptr = ptr.sub(V::size());
        if let Some(i) = reverse_search2(start_ptr, end_ptr, ptr, vn1, vn2) {
            return Some(i);
        }
    }
    if ptr > start_ptr {
        debug_assert!(sub(ptr, start_ptr) < V::size());
        return reverse_search2(start_ptr, end_ptr, start_ptr, vn1, vn2);
    }
    None
}

#[inline(always)]
pub(crate) unsafe fn memrchr3<V: Vector>(
    n1: u8,
    n2: u8,
    n3: u8,
    haystack: &[u8],
) -> Option<usize> {
    let vn1 = V::splat(n1);
    let vn2 = V::splat(n2);
    let vn3 = V::splat(n3);
    let len = haystack.len();
    let loop_size = cmp::min(LOOP_AMT2 * V::size(), len);
    let start_ptr = haystack.as_ptr();
    let end_ptr = start_ptr.add(haystack.len());
    let mut ptr = end_ptr;

    if haystack.len() < V::size() {
        while ptr > start_ptr {
            ptr = ptr.offset(-1);
            if *ptr == n1 || *ptr == n2 || *ptr == n3 {
                return Some(sub(ptr, start_ptr));
            }
        }
        return None;
    }

    ptr = ptr.sub(V::size());
    if let Some(i) = reverse_search3(start_ptr, end_ptr, ptr, vn1, vn2, vn3) {
        return Some(i);
    }

    ptr = (end_ptr as usize & !V::align_mask()) as *const u8;
    debug_assert!(start_ptr <= ptr && ptr <= end_ptr);
    while loop_size == LOOP_AMT2 * V::size() && ptr >= start_ptr.add(loop_size)
    {
        debug_assert_eq!(0, (ptr as usize) % V::size());

        ptr = ptr.sub(loop_size);
        let a = V::load_aligned(ptr);
        let b = V::load_aligned(ptr.add(V::size()));
        let eqa1 = vn1.cmpeq(a);
        let eqb1 = vn1.cmpeq(b);
        let eqa2 = vn2.cmpeq(a);
        let eqb2 = vn2.cmpeq(b);
        let eqa3 = vn3.cmpeq(a);
        let eqb3 = vn3.cmpeq(b);
        let or1 = eqa1.or(eqb1);
        let or2 = eqa2.or(eqb2);
        let or3 = eqa3.or(eqb3);
        let or4 = or1.or(or2);
        let or5 = or3.or(or4);
        if or5.movemask() != 0 {
            let mut at = sub(ptr.add(V::size()), start_ptr);
            let mask1 = eqb1.movemask();
            let mask2 = eqb2.movemask();
            let mask3 = eqb3.movemask();
            if mask1 != 0 || mask2 != 0 || mask3 != 0 {
                return Some(at + reverse_pos3::<V>(mask1, mask2, mask3));
            }

            at -= V::size();
            let mask1 = eqa1.movemask();
            let mask2 = eqa2.movemask();
            let mask3 = eqa3.movemask();
            return Some(at + reverse_pos3::<V>(mask1, mask2, mask3));
        }
    }
    while ptr >= start_ptr.add(V::size()) {
        ptr = ptr.sub(V::size());
        if let Some(i) =
            reverse_search3(start_ptr, end_ptr, ptr, vn1, vn2, vn3)
        {
            return Some(i);
        }
    }
    if ptr > start_ptr {
        debug_assert!(sub(ptr, start_ptr) < V::size());
        return reverse_search3(start_ptr, end_ptr, start_ptr, vn1, vn2, vn3);
    }
    None
}

#[inline(always)]
pub(crate) unsafe fn forward_search1<V: Vector>(
    start_ptr: *const u8,
    end_ptr: *const u8,
    ptr: *const u8,
    vn1: V,
) -> Option<usize> {
    debug_assert!(sub(end_ptr, start_ptr) >= V::size());
    debug_assert!(start_ptr <= ptr);
    debug_assert!(ptr <= end_ptr.sub(V::size()));

    let chunk = V::load_unaligned(ptr);
    let mask = chunk.cmpeq(vn1).movemask();
    if mask != 0 {
        Some(sub(ptr, start_ptr) + forward_pos(mask))
    } else {
        None
    }
}

#[inline(always)]
unsafe fn forward_search2<V: Vector>(
    start_ptr: *const u8,
    end_ptr: *const u8,
    ptr: *const u8,
    vn1: V,
    vn2: V,
) -> Option<usize> {
    debug_assert!(sub(end_ptr, start_ptr) >= V::size());
    debug_assert!(start_ptr <= ptr);
    debug_assert!(ptr <= end_ptr.sub(V::size()));

    let chunk = V::load_unaligned(ptr);
    let eq1 = chunk.cmpeq(vn1);
    let eq2 = chunk.cmpeq(vn2);
    if eq1.or(eq2).movemask() != 0 {
        let mask1 = eq1.movemask();
        let mask2 = eq2.movemask();
        Some(sub(ptr, start_ptr) + forward_pos2(mask1, mask2))
    } else {
        None
    }
}

#[inline(always)]
pub(crate) unsafe fn forward_search3<V: Vector>(
    start_ptr: *const u8,
    end_ptr: *const u8,
    ptr: *const u8,
    vn1: V,
    vn2: V,
    vn3: V,
) -> Option<usize> {
    debug_assert!(sub(end_ptr, start_ptr) >= V::size());
    debug_assert!(start_ptr <= ptr);
    debug_assert!(ptr <= end_ptr.sub(V::size()));

    let chunk = V::load_unaligned(ptr);
    let eq1 = chunk.cmpeq(vn1);
    let eq2 = chunk.cmpeq(vn2);
    let eq3 = chunk.cmpeq(vn3);
    let or = eq1.or(eq2);
    if or.or(eq3).movemask() != 0 {
        let mask1 = eq1.movemask();
        let mask2 = eq2.movemask();
        let mask3 = eq3.movemask();
        Some(sub(ptr, start_ptr) + forward_pos3(mask1, mask2, mask3))
    } else {
        None
    }
}

#[inline(always)]
unsafe fn reverse_search1<V: Vector>(
    start_ptr: *const u8,
    end_ptr: *const u8,
    ptr: *const u8,
    vn1: V,
) -> Option<usize> {
    debug_assert!(sub(end_ptr, start_ptr) >= V::size());
    debug_assert!(start_ptr <= ptr);
    debug_assert!(ptr <= end_ptr.sub(V::size()));

    let chunk = V::load_unaligned(ptr);
    let mask = vn1.cmpeq(chunk).movemask();
    if mask != 0 {
        Some(sub(ptr, start_ptr) + reverse_pos::<V>(mask))
    } else {
        None
    }
}

#[inline(always)]
unsafe fn reverse_search2<V: Vector>(
    start_ptr: *const u8,
    end_ptr: *const u8,
    ptr: *const u8,
    vn1: V,
    vn2: V,
) -> Option<usize> {
    debug_assert!(sub(end_ptr, start_ptr) >= V::size());
    debug_assert!(start_ptr <= ptr);
    debug_assert!(ptr <= end_ptr.sub(V::size()));

    let chunk = V::load_unaligned(ptr);
    let eq1 = chunk.cmpeq(vn1);
    let eq2 = chunk.cmpeq(vn2);
    if eq1.or(eq2).movemask() != 0 {
        let mask1 = eq1.movemask();
        let mask2 = eq2.movemask();
        Some(sub(ptr, start_ptr) + reverse_pos2::<V>(mask1, mask2))
    } else {
        None
    }
}

#[inline(always)]
unsafe fn reverse_search3<V: Vector>(
    start_ptr: *const u8,
    end_ptr: *const u8,
    ptr: *const u8,
    vn1: V,
    vn2: V,
    vn3: V,
) -> Option<usize> {
    debug_assert!(sub(end_ptr, start_ptr) >= V::size());
    debug_assert!(start_ptr <= ptr);
    debug_assert!(ptr <= end_ptr.sub(V::size()));

    let chunk = V::load_unaligned(ptr);
    let eq1 = chunk.cmpeq(vn1);
    let eq2 = chunk.cmpeq(vn2);
    let eq3 = chunk.cmpeq(vn3);
    let or = eq1.or(eq2);
    if or.or(eq3).movemask() != 0 {
        let mask1 = eq1.movemask();
        let mask2 = eq2.movemask();
        let mask3 = eq3.movemask();
        Some(sub(ptr, start_ptr) + reverse_pos3::<V>(mask1, mask2, mask3))
    } else {
        None
    }
}

/// Compute the position of the first matching byte from the given mask. The
/// position returned is always in the range [0, V::size()).
///
/// The mask given is expected to be the result of _mm_movemask_epi8.
fn forward_pos(mask: u32) -> usize {
    // We are dealing with little endian here, where the most significant byte
    // is at a higher address. That means the least significant bit that is set
    // corresponds to the position of our first matching byte. That position
    // corresponds to the number of zeros after the least significant bit.
    assert!(cfg!(target_endian = "little"));
    mask.trailing_zeros() as usize
}

/// Compute the position of the first matching byte from the given masks. The
/// position returned is always in the range [0, V::size()). Each mask corresponds to
/// the equality comparison of a single byte.
///
/// The masks given are expected to be the result of _mm_movemask_epi8, where
/// at least one of the masks is non-zero (i.e., indicates a match).
fn forward_pos2(mask1: u32, mask2: u32) -> usize {
    debug_assert!(mask1 != 0 || mask2 != 0);

    forward_pos(mask1 | mask2)
}

/// Compute the position of the first matching byte from the given masks. The
/// position returned is always in the range [0, V::size()). Each mask corresponds to
/// the equality comparison of a single byte.
///
/// The masks given are expected to be the result of _mm_movemask_epi8, where
/// at least one of the masks is non-zero (i.e., indicates a match).
fn forward_pos3(mask1: u32, mask2: u32, mask3: u32) -> usize {
    debug_assert!(mask1 != 0 || mask2 != 0 || mask3 != 0);

    forward_pos(mask1 | mask2 | mask3)
}

/// Compute the position of the last matching byte from the given mask. The
/// position returned is always in the range [0, V::size()).
///
/// The mask given is expected to be the result of _mm_movemask_epi8.
fn reverse_pos<V: Vector>(mask: u32) -> usize {
    // We are dealing with little endian here, where the most significant byte
    // is at a higher address. That means the most significant bit that is set
    // corresponds to the position of our last matching byte. The position from
    // the end of the mask is therefore the number of leading zeros in a 32
    // bit integer, and the position from the start of the mask is therefore
    // size - (leading zeros) - 1.
    let r = 31 - mask.leading_zeros() as usize;
    return r;
    // let r = V::size() - mask.leading_zeros() as usize - 1;
    // return r;
}

/// Compute the position of the last matching byte from the given masks. The
/// position returned is always in the range [0, 15]. Each mask corresponds to
/// the equality comparison of a single byte.
///
/// The masks given are expected to be the result of _mm_movemask_epi8, where
/// at least one of the masks is non-zero (i.e., indicates a match).
fn reverse_pos2<V: Vector>(mask1: u32, mask2: u32) -> usize {
    debug_assert!(mask1 != 0 || mask2 != 0);

    reverse_pos::<V>(mask1 | mask2)
}

/// Compute the position of the last matching byte from the given masks. The
/// position returned is always in the range [0, 15]. Each mask corresponds to
/// the equality comparison of a single byte.
///
/// The masks given are expected to be the result of _mm_movemask_epi8, where
/// at least one of the masks is non-zero (i.e., indicates a match).
fn reverse_pos3<V: Vector>(mask1: u32, mask2: u32, mask3: u32) -> usize {
    debug_assert!(mask1 != 0 || mask2 != 0 || mask3 != 0);

    reverse_pos::<V>(mask1 | mask2 | mask3)
}

/// Subtract `b` from `a` and return the difference. `a` should be greater than
/// or equal to `b`.
fn sub(a: *const u8, b: *const u8) -> usize {
    debug_assert!(a >= b);
    (a as usize) - (b as usize)
}
