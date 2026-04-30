use crate::trajectory::*;
use crate::types::Hash32;

fn mock_transition(from: StateNode, to: StateNode, delta: u128) -> Transition {
    Transition {
        from,
        to,
        delta,
        delta_hat: delta + 2, // d_hat >= d
        projection_hash: Hash32([0; 32]),
        certificate_hash: Hash32([0; 32]),
        step_type: Some("mock".to_string()),
    }
}

#[test]
fn test_v3_distance_triangle_inequality() {
    let mut engine = TrajectoryEngine::new();

    let x = StateNode {
        hash: Hash32([1; 32]),
        potential: 100,
    };
    let y = StateNode {
        hash: Hash32([2; 32]),
        potential: 80,
    };
    let z = StateNode {
        hash: Hash32([3; 32]),
        potential: 60,
    };

    // x -> y with delta 10, delta_hat 12
    engine.add_transition(mock_transition(x.clone(), y.clone(), 10));

    // y -> z with delta 5, delta_hat 7
    engine.add_transition(mock_transition(y.clone(), z.clone(), 5));

    // Direct x -> z with delta 20, delta_hat 22 (suboptimal)
    engine.add_transition(mock_transition(x.clone(), z.clone(), 20));

    let d_xy = engine.compute_distance(x.hash, y.hash).unwrap();
    let d_yz = engine.compute_distance(y.hash, z.hash).unwrap();
    let d_xz = engine.compute_distance(x.hash, z.hash).unwrap();

    assert_eq!(d_xy, 12); // Uses delta_hat
    assert_eq!(d_yz, 7);
    assert_eq!(d_xz, 19); // 12 + 7

    // Triangle Inequality: d(x, z) <= d(x, y) + d(y, z)
    assert!(d_xz <= d_xy + d_yz);
}

#[test]
fn test_compute_geodesic() {
    let mut engine = TrajectoryEngine::new();
    let x = StateNode { hash: Hash32([1; 32]), potential: 100 };
    let y = StateNode { hash: Hash32([2; 32]), potential: 80 };
    let z = StateNode { hash: Hash32([3; 32]), potential: 60 };

    engine.add_transition(mock_transition(x.clone(), y.clone(), 10));
    engine.add_transition(mock_transition(y.clone(), z.clone(), 5));

    let trajectory = engine.compute_geodesic(x.hash, z.hash).unwrap();
    assert_eq!(trajectory.steps.len(), 2);
    assert_eq!(trajectory.total_certified_defect(), 19);
    
    let projection = trajectory.project();
    assert_eq!(projection.projections.len(), 2);
    assert_eq!(projection.total_delta_hat, 19);
}
