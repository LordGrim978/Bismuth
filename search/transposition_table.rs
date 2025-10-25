use crate::move_gen_dir::move_gen::Move;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TranspositionTableEntry {
    pub key: u64,
    pub value: i32,
    pub mv: Option<Move>,
    pub depth: i32,
    pub node_type: NodeType,
}

impl TranspositionTableEntry {
    pub fn new(key: u64, value: i32, depth: i32, node_type: NodeType, mv: Option<Move>) -> Self {
        TranspositionTableEntry {
            key,
            value,
            depth,
            node_type,
            mv,
        }
    }

    pub fn size_of() -> usize {
        size_of::<TranspositionTableEntry>()
    }
}
pub const LOOKUP_FAILED:i32 = -1;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranspositionTable {
    pub entries: Vec<TranspositionTableEntry>,
    pub count: u64,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let entry_size = TranspositionTableEntry::size_of();
        let desired_bytes = size_mb * 1024 * 1024;
        let num_entries = desired_bytes / entry_size;

        TranspositionTable {
            entries: vec![
                TranspositionTableEntry {
                    key: 0,
                    value: 0,
                    depth: 0,
                    node_type: NodeType::Exact,
                    mv: None
                };
                num_entries
            ],
            count: num_entries as u64,
        }
    }

    pub fn clear(&mut self) {
        for e in self.entries.iter_mut() {
            *e = TranspositionTableEntry {
                key: 0,
                value: 0,
                depth: 0,
                node_type: NodeType::Exact,
                mv: None,
            };
        }
    }

    pub fn index(&self, zobrist_key: u64) -> usize {
        (zobrist_key % self.count) as usize
    }

    pub fn try_get_stored_move(&self, zobrist_key: u64) -> Option<Move> {
        self.entries[self.index(zobrist_key)].mv
    }

    pub fn lookup_evaluation(
        &self,
        zobrist_key: u64,
        depth: i32,
        ply_from_root: i32,
        alpha: i32,
        beta: i32,
    ) -> i32 {
        let entry = &self.entries[self.index(zobrist_key)];
        if entry.key == zobrist_key {
            if entry.depth >= depth {
                let corrected_score =
                    Self::correct_retrieved_mate_score(entry.value, ply_from_root);
                match entry.node_type {
                    NodeType::Exact => return corrected_score,
                    NodeType::UpperBound if corrected_score <= alpha => return corrected_score,
                    NodeType::LowerBound if corrected_score >= beta => return corrected_score,
                    _ => {}
                }
            }
        }
        LOOKUP_FAILED
    }

    pub fn store_evaluation(
        &mut self,
        zobrist_key: u64,
        depth: i32,
        ply_from_root: i32,
        eval: i32,
        eval_type: NodeType,
        mv: Option<Move>,
    ) {
        let index = self.index(zobrist_key);
        let corrected = Self::correct_mate_score_for_storage(eval, ply_from_root);
        self.entries[index] = TranspositionTableEntry::new(zobrist_key, corrected, depth, eval_type, mv);
    }

    fn correct_mate_score_for_storage(score: i32, num_ply_searched: i32) -> i32 {
        if Self::is_mate_score(score) {
            let sign = score.signum();
            (score * sign + num_ply_searched) * sign
        } else {
            score
        }
    }

    fn correct_retrieved_mate_score(score: i32, num_ply_searched: i32) -> i32 {
        if Self::is_mate_score(score) {
            let sign = score.signum();
            (score * sign - num_ply_searched) * sign
        } else {
            score
        }
    }

    fn is_mate_score(score: i32) -> bool {
        // Example heuristic (adjust for your engine’s mate score convention)
        score.abs() > 9_000_000
    }

    pub fn get_entry(&self, zobrist_key: u64) -> &TranspositionTableEntry {
        &self.entries[self.index(zobrist_key)]
    }
}