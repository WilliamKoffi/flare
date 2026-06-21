use super::Ledger;
use chrono::{TimeZone, Utc};
use std::collections::HashMap;

#[test]
fn counts_entries_for_date() {
    let entries = HashMap::from([
        (
            "a".into(),
            Utc.with_ymd_and_hms(2026, 6, 20, 10, 0, 0).unwrap(),
        ),
        (
            "b".into(),
            Utc.with_ymd_and_hms(2026, 6, 19, 23, 59, 0).unwrap(),
        ),
    ]);
    let ledger = Ledger {
        path: String::new(),
        entries,
    };

    assert_eq!(
        ledger.count(chrono::NaiveDate::from_ymd_opt(2026, 6, 20).unwrap()),
        1
    );
}
