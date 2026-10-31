// Collect nearby player UUIDs and positions for Simple Voice Chat proximity.
// Kept inside remote.rs so private entity maps are accessed legitimately.
pub(crate) fn snapshot(
    remote: &super::Remote,
    listener: [f64; 3],
) -> Vec<([u8; 16], [f64; 3])> {
    let mut players = Vec::new();
    for (entity_id, uuid) in remote.level_players.iter().take(512) {
        if let Some(entity) = remote.entities.get(entity_id) {
            let xyz = entity.position;
            if xyz.iter().all(|x| x.is_finite()) {
                let dist = (xyz[0] - listener[0]).powi(2)
                    + (xyz[1] - listener[1]).powi(2)
                    + (xyz[2] - listener[2]).powi(2);
                if dist.is_finite() {
                    players.push((dist, uuid.to_be_bytes(), xyz));
                }
            }
        }
    }
    players.sort_by(|a, b| a.0.total_cmp(&b.0));
    players.into_iter().take(24).map(|(_, uuid, xyz)| (uuid, xyz)).collect()
}
