use codeprints_analyzer::Merger;
use codeprints_analyzer::Timeline;
use std::fs;

#[test]
fn test_intesity() {
    let input = fs::read_to_string("fixtures/mre_raw_2020_from_api.json").unwrap();
    let timeline: Timeline = serde_json::from_str(&input).unwrap();
    let mut merger = Merger::new();
    merger.merge_timeline(&timeline).unwrap();

    let new_timeline = merger.timeline().unwrap();

    for year in new_timeline.years {
        let orig_year = timeline.years.iter().find(|y| y.year == year.year).unwrap();
        assert_eq!(year.total, orig_year.total);
    }

    for contribution in new_timeline.contributions {
        let orig_contribution = timeline
            .contributions
            .iter()
            .find(|c| c.date == contribution.date)
            .unwrap();

        println!("contribution: {:?}", contribution);
        println!("orig: {:?}", orig_contribution);

        assert_eq!(contribution.count, orig_contribution.count);
        assert_eq!(contribution.color, orig_contribution.color);
        assert_eq!(contribution.intensity, orig_contribution.intensity);
    }
}
