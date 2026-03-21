/// A `select_biased!` macro that works on both std (via `futures::select_biased!`)
/// and embassy (via `embassy_futures::select`).
///
/// Supports 2, 3, and 4-branch variants. The first branch has highest priority.
///
/// Usage:
/// ```ignore
/// pal::select_biased! {
///     result = some_future => { handle(result); }
///     other = another_future => { handle(other); }
/// }
/// ```

// =============================================================================
// std backend — delegates to futures::select_biased!
// =============================================================================

#[cfg(feature = "std")]
#[macro_export]
macro_rules! select_biased {
    // 2-branch variant
    ($p1:pat = $f1:expr => $h1:expr, $p2:pat = $f2:expr => $h2:expr $(,)?) => {{
        use $crate::__reexport_futures::FutureExt;
        $crate::__reexport_futures::select_biased! {
            $p1 = ($f1).fuse() => $h1,
            $p2 = ($f2).fuse() => $h2,
        }
    }};

    // 3-branch variant
    ($p1:pat = $f1:expr => $h1:expr, $p2:pat = $f2:expr => $h2:expr, $p3:pat = $f3:expr => $h3:expr $(,)?) => {{
        use $crate::__reexport_futures::FutureExt;
        $crate::__reexport_futures::select_biased! {
            $p1 = ($f1).fuse() => $h1,
            $p2 = ($f2).fuse() => $h2,
            $p3 = ($f3).fuse() => $h3,
        }
    }};

    // 4-branch variant
    ($p1:pat = $f1:expr => $h1:expr, $p2:pat = $f2:expr => $h2:expr, $p3:pat = $f3:expr => $h3:expr, $p4:pat = $f4:expr => $h4:expr $(,)?) => {{
        use $crate::__reexport_futures::FutureExt;
        $crate::__reexport_futures::select_biased! {
            $p1 = ($f1).fuse() => $h1,
            $p2 = ($f2).fuse() => $h2,
            $p3 = ($f3).fuse() => $h3,
            $p4 = ($f4).fuse() => $h4,
        }
    }};
}

// =============================================================================
// embassy backend — delegates to embassy_futures::select
// =============================================================================

#[cfg(feature = "embassy")]
#[macro_export]
macro_rules! select_biased {
    // 2-branch variant
    ($p1:pat = $f1:expr => $h1:expr, $p2:pat = $f2:expr => $h2:expr $(,)?) => {{
        match $crate::__reexport_embassy_futures::select::select($f1, $f2).await {
            $crate::__reexport_embassy_futures::select::Either::First($p1) => $h1,
            $crate::__reexport_embassy_futures::select::Either::Second($p2) => $h2,
        }
    }};

    // 3-branch variant
    ($p1:pat = $f1:expr => $h1:expr, $p2:pat = $f2:expr => $h2:expr, $p3:pat = $f3:expr => $h3:expr $(,)?) => {{
        match $crate::__reexport_embassy_futures::select::select3($f1, $f2, $f3).await {
            $crate::__reexport_embassy_futures::select::Either3::First($p1) => $h1,
            $crate::__reexport_embassy_futures::select::Either3::Second($p2) => $h2,
            $crate::__reexport_embassy_futures::select::Either3::Third($p3) => $h3,
        }
    }};

    // 4-branch variant
    ($p1:pat = $f1:expr => $h1:expr, $p2:pat = $f2:expr => $h2:expr, $p3:pat = $f3:expr => $h3:expr, $p4:pat = $f4:expr => $h4:expr $(,)?) => {{
        match $crate::__reexport_embassy_futures::select::select4($f1, $f2, $f3, $f4).await {
            $crate::__reexport_embassy_futures::select::Either4::First($p1) => $h1,
            $crate::__reexport_embassy_futures::select::Either4::Second($p2) => $h2,
            $crate::__reexport_embassy_futures::select::Either4::Third($p3) => $h3,
            $crate::__reexport_embassy_futures::select::Either4::Fourth($p4) => $h4,
        }
    }};
}
