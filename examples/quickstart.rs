//! Executable tour of the primary ZLID API.

use std::cmp::Ordering;

use zlid::{bytes_from_hex, Inspection, InspectionKind, Profile, Zlid};

fn main() -> zlid::Result<()> {
    let ordered = Zlid::next_with_partition(42)?;
    let ordered_info = ordered.inspect();
    assert_eq!(ordered_info.kind(), InspectionKind::Ordered);
    if let Inspection::Ordered { partition, .. } = ordered_info {
        assert_eq!(partition, 42);
    } else {
        unreachable!("ordered ZLID inspected as non-ordered");
    }

    let mut generator = Zlid::generator(Profile::HighThroughput, 42);
    assert_eq!(generator.profile(), Profile::HighThroughput);
    let generated = generator.next()?;
    let generated_info = generated.inspect();
    assert_eq!(generated_info.kind(), InspectionKind::Ordered);
    if let Inspection::Ordered {
        profile, partition, ..
    } = generated_info
    {
        assert_eq!(profile, Profile::HighThroughput);
        assert_eq!(partition, 42);
    } else {
        unreachable!("generated ZLID inspected as non-ordered");
    }

    let random = Zlid::random()?;
    assert_eq!(random.inspect().kind(), InspectionKind::Random);

    let alias_key = [0, 1, 2, 3];
    let alias = ordered.alias_str(&alias_key, "users|prod")?;
    let source = alias.unalias_str(&alias_key, "users|prod")?;
    assert_eq!(source, ordered);

    let canonical = Zlid::parse("01k2r7-kfwe58 07000000000001")?;
    assert_eq!(canonical.text(), "01K2R7KFWE5807000000000001");

    let partition_key = bytes_from_hex("000102030405060708090A0B0C0D0E0F")?;
    let partition = Zlid::partition_str("tenant:acme", Some(&partition_key))?;
    assert_eq!(partition, 17);

    assert_eq!(
        Zlid::compare(
            &Zlid::parse("01K2R7KFWE5807000000000001")?,
            &Zlid::parse("01K2R7KFWE5809000000000003")?
        ),
        Ordering::Less
    );
    assert_eq!(Zlid::NIL.inspect().kind(), InspectionKind::Sentinel);
    assert_eq!(Zlid::MAX.inspect().kind(), InspectionKind::Sentinel);

    println!("{}", ordered.text());
    println!("{}", random.text());
    println!("{}", alias.text());
    println!("{}", source == ordered);
    println!("{}", canonical.text());
    println!("{partition}");

    Ok(())
}
