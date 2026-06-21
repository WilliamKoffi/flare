use super::Warmup;

fn schedule() -> Warmup {
    Warmup {
        tier: vec![
            super::Tier {
                day: 1,
                capacity: 2,
            },
            super::Tier {
                day: 4,
                capacity: 5,
            },
            super::Tier {
                day: 8,
                capacity: 10,
            },
            super::Tier {
                day: 15,
                capacity: 20,
            },
        ],
    }
}

#[test]
fn resolves_first_tier() {
    assert_eq!(schedule().resolve(1), 2);
    assert_eq!(schedule().resolve(3), 2);
}

#[test]
fn resolves_middle_tier() {
    assert_eq!(schedule().resolve(4), 5);
    assert_eq!(schedule().resolve(7), 5);
    assert_eq!(schedule().resolve(8), 10);
    assert_eq!(schedule().resolve(14), 10);
}

#[test]
fn resolves_final_tier() {
    assert_eq!(schedule().resolve(15), 20);
    assert_eq!(schedule().resolve(100), 20);
}

#[test]
fn resolves_before_first_tier() {
    let warmup = Warmup {
        tier: vec![super::Tier {
            day: 5,
            capacity: 10,
        }],
    };
    // Day 1 is before the first tier (day 5), falls back to first tier's capacity
    assert_eq!(warmup.resolve(1), 10);
}
