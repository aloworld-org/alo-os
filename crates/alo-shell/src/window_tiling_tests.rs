use super::*;

#[test]
fn window_tiling_halves_cover_even_odd_and_boundary_outputs() -> Result<(), TileGeometryError> {
    for width in [2, 3, 80, 81, 999_999, 1_000_000] {
        for height in [1, 80, 1_000_000] {
            let left = TileGeometry::new((width, height), TileSide::Left, (0, 0), (0, 0))?;
            let right = TileGeometry::new((width, height), TileSide::Right, (0, 0), (0, 0))?;
            assert_eq!(left.requested_size(), (width / 2, height));
            assert_eq!(right.requested_size(), (width - width / 2, height));
            assert_eq!(left.committed_origin(left.requested_size()), Ok((0, 0)));
            assert_eq!(
                right.committed_origin(right.requested_size()),
                Ok((width / 2, 0))
            );
            assert_eq!(left.requested_size().0 + right.requested_size().0, width);
        }
    }
    Ok(())
}

#[test]
fn window_tiling_refuses_invalid_output_and_exact_size_conflicts() {
    for output in [
        (0, 80),
        (1, 80),
        (-1, 80),
        (80, 0),
        (80, -1),
        (1_000_001, 80),
        (80, i32::MAX),
    ] {
        assert_eq!(
            TileGeometry::new(output, TileSide::Left, (0, 0), (0, 0)),
            Err(TileGeometryError::OutputUnavailable)
        );
    }
    for (min, max) in [
        ((41, 0), (0, 0)),
        ((0, 81), (0, 0)),
        ((0, 0), (39, 0)),
        ((0, 0), (0, 79)),
        ((-1, 0), (0, 0)),
        ((0, 0), (0, -1)),
        ((50, 0), (40, 0)),
    ] {
        assert_eq!(
            TileGeometry::new((80, 80), TileSide::Left, min, max),
            Err(TileGeometryError::ClientLimits)
        );
    }
    assert!(TileGeometry::new((80, 80), TileSide::Left, (40, 80), (40, 80)).is_ok());
}

#[test]
fn window_tiling_actual_size_preserves_outside_edge_with_bounded_arithmetic()
-> Result<(), TileGeometryError> {
    for side in [TileSide::Left, TileSide::Right] {
        let tile = TileGeometry::new((81, 80), side, (0, 0), (0, 0))?;
        for size in [(1, 1), (32, 24), (1_000_000, 1_000_000)] {
            let x = if side == TileSide::Left {
                0
            } else {
                81 - size.0
            };
            assert_eq!(tile.committed_origin(size), Ok((x, 0)));
        }
        for size in [
            (0, 1),
            (1, 0),
            (-1, 1),
            (1, -1),
            (i32::MAX, 1),
            (1, 1_000_001),
        ] {
            assert_eq!(tile.committed_origin(size), Err(TileGeometryError::Size));
        }
    }
    Ok(())
}
