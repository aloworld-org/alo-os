//! Exhaustive edge mathematics and refusal boundaries.
use super::*;

fn geometry(edge: ResizeEdge) -> ResizeGeometry {
    ResizeGeometry {
        origin: (100, 200),
        size: (80, 60),
        min: (0, 0),
        max: (0, 0),
        edge,
    }
}

#[test]
fn every_edge_preserves_its_opposite_for_the_actual_client_size() {
    use ResizeEdge::*;
    for (edge, requested, committed) in [
        (Top, (80, 40), (100, 190)),
        (Bottom, (80, 80), (100, 200)),
        (Left, (70, 60), (90, 200)),
        (Right, (90, 60), (100, 200)),
        (TopLeft, (70, 40), (90, 190)),
        (TopRight, (90, 40), (100, 190)),
        (BottomLeft, (70, 80), (90, 200)),
        (BottomRight, (90, 80), (100, 200)),
    ] {
        let g = geometry(edge);
        assert_eq!(g.requested_size((10.0, 20.0)), Ok(requested), "{edge:?}");
        // A client can choose 90x70 despite receiving a different suggestion.
        assert_eq!(g.committed_origin((90, 70)), Ok(committed), "{edge:?}");
        assert_eq!(g.committed_origin(g.size), Ok(g.origin));
    }
}

#[test]
fn crossing_limits_and_rounding_never_flip_edges_or_accumulate_drift() {
    let mut g = geometry(ResizeEdge::TopLeft);
    assert_eq!(g.requested_size((100.0, 100.0)), Ok((1, 1)));
    assert_eq!(g.requested_size((-0.5, -1.5)), Ok((81, 62)));
    assert_eq!(g.requested_size((0.49, 0.49)), Ok((80, 60)));
    g.min = (20, 30);
    g.max = (100, 90);
    assert_eq!(g.requested_size((100.0, 100.0)), Ok((20, 30)));
    assert_eq!(g.requested_size((-100.0, -100.0)), Ok((100, 90)));
    assert_eq!(g.requested_size((0.0, 0.0)), Ok((80, 60)));
    g.max = (0, 0);
    assert_eq!(
        g.requested_size((-1_000_000.0, -1_000_000.0)),
        Ok((LIMIT, LIMIT))
    );
}

#[test]
fn invalid_deltas_sizes_origins_and_unsatisfiable_limits_refuse() {
    let mut g = geometry(ResizeEdge::Left);
    for bad in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        1_000_001.0,
        -1_000_001.0,
    ] {
        assert_eq!(
            g.requested_size((bad, 0.0)),
            Err(ResizeGeometryError::Delta)
        );
        assert_eq!(
            g.requested_size((0.0, bad)),
            Err(ResizeGeometryError::Delta)
        );
    }
    for bad in [0, -1, i32::MIN, LIMIT + 1, i32::MAX] {
        assert_eq!(g.committed_origin((bad, 1)), Err(ResizeGeometryError::Size));
        assert_eq!(g.committed_origin((1, bad)), Err(ResizeGeometryError::Size));
    }
    g.origin.0 = -LIMIT;
    assert_eq!(
        g.committed_origin((81, 60)),
        Err(ResizeGeometryError::Geometry)
    );
    assert_eq!(g.committed_origin((80, 60)), Ok((-LIMIT, 200)));
    g.origin.0 = LIMIT;
    assert_eq!(
        g.committed_origin((79, 60)),
        Err(ResizeGeometryError::Geometry)
    );
    g.min.0 = LIMIT + 1;
    assert_eq!(
        g.requested_size((0.0, 0.0)),
        Err(ResizeGeometryError::ClientLimits)
    );
    g.min = (0, 61); // A horizontal resize cannot fix an invalid unchanged height.
    assert_eq!(
        g.requested_size((0.0, 0.0)),
        Err(ResizeGeometryError::ClientLimits)
    );
    g.min = (100, 0);
    g.max = (90, 0);
    assert_eq!(
        g.requested_size((0.0, 0.0)),
        Err(ResizeGeometryError::ClientLimits)
    );
}
