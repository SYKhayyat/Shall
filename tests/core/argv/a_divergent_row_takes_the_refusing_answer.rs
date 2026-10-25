use super::*;

/// A row whose hosts disagree takes the answer that sends no terminator.
///
/// The exemption `Divergent` buys is *the probe stops calling one host's reading a
/// disagreement* — and it is only sound in the safe direction. A divergent row claiming to
/// terminate would be Shall emitting a `--` that some platform reads as a package name,
/// which is precisely the failure this table exists to prevent, with an exemption on top
/// of it. Unrepresentable is better than documented, and this is as close as a const table
/// gets.
#[test]
fn a_divergent_row_takes_the_refusing_answer() {
    let claiming: Vec<&str> = TERMINATORS
        .iter()
        .filter(|t| matches!(t.evidence, Evidence::Divergent(_)) && t.terminates)
        .map(|t| t.binary)
        .collect();
    assert!(
        claiming.is_empty(),
        "{claiming:?} say their answer differs by host AND that they terminate. The whole \
         point of recording divergence is to take the refusing answer; a divergent `true` \
         is an exemption laid over the defect the table exists to prevent."
    );
}
