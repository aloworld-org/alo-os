//! **What this model learned from, written down so it can be answered later.**
//!
//! *What has my model learned from?* is a question a person asks a year after
//! the fine-tune, when the model says something that surprises them. It is
//! answerable only if the machine wrote it down at the time, so this is the
//! statement a record entry carries: the folders, the grant they were read
//! under, when, how many files, and what was left out.
//!
//! **Revoking the grant afterwards does not unlearn any of it** — ADR 0041 is
//! the sentence a person reads about that, and this statement is what makes the
//! question answerable in the first place.

use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::dataset::Dataset;

/// **What one fine-tune was trained on.**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedFrom {
    /// The folders, as the person granted them.
    pub folders: Vec<PathBuf>,
    /// Who the grant was made to, as `alo-capability` names a grantee.
    pub under_the_grant_to: String,
    /// When the list of files was taken — the moment the dataset stopped
    /// changing.
    pub taken_at: SystemTime,
    /// How many files it learned from.
    pub learned_from: usize,
    /// How many were found and left out, each of which was named to the person
    /// at the time.
    pub left_out: usize,
}

impl LearnedFrom {
    /// The statement, from the dataset that was trained on.
    #[must_use]
    pub fn of(dataset: &Dataset, grantee: &str) -> Self {
        Self {
            folders: dataset.folders().to_vec(),
            under_the_grant_to: grantee.to_owned(),
            taken_at: dataset.taken_at(),
            learned_from: dataset.how_many(),
            left_out: dataset.not_trained_on().len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Everything the question needs is in the statement**: which folders,
    /// whose grant, when, and how much.
    #[test]
    fn what_a_model_learned_from_answers_the_question_a_year_later() {
        let (dataset, folder) = crate::dataset::tests_support::one_file_dataset();
        let learned = LearnedFrom::of(&dataset, "@adapting");
        assert_eq!(learned.folders, vec![folder]);
        assert_eq!(learned.under_the_grant_to, "@adapting");
        assert_eq!(learned.learned_from, 1);
        assert_eq!(learned.left_out, 0);
        assert_eq!(learned.taken_at, SystemTime::UNIX_EPOCH);
    }
}
