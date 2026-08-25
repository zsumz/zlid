//! Executable tour of the primary ZLID API.

use std::cmp::Ordering;

use zlid::{bytes_from_hex, Inspection, InspectionKind, Profile, ZLID};

fn main() -> zlid::Result<()> {
    let ordered = ZLID::next_with_partition(42)?;
    let ordered_info = ordered.inspect();
    assert_eq!(ordered_info.kind(), InspectionKind::Ordered);
    if let Inspection::Ordered { partition, .. } = ordered_info {
        assert_eq!(partition, 42);
    } else {
        unreachable!("ordered ZLID inspected as non-ordered");
    }

    let mut generator = ZLID::generator(Profile::HighThroughput, 42);
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

    let random = ZLID::random()?;
    assert_eq!(random.inspect().kind(), InspectionKind::Random);

    let alias_key = [0x42; 32]; // Demo only; production keys belong in secret storage.
    let alias = ordered.alias_str(&alias_key, "users|prod")?;
    let source = alias.unalias_str(&alias_key, "users|prod")?;
    assert_eq!(source, ordered);

    let canonical = ZLID::parse("01k2r7-kfwe58 07000000000001")?;
    assert_eq!(canonical.text(), "01K2R7KFWE5807000000000001");

    let partition_key = bytes_from_hex("000102030405060708090A0B0C0D0E0F")?;
    let partition = ZLID::partition_str("tenant:acme", Some(&partition_key))?;
    assert_eq!(partition, 17);

    assert_eq!(
        ZLID::compare(
            &ZLID::parse("01K2R7KFWE5807000000000001")?,
            &ZLID::parse("01K2R7KFWE5809000000000003")?
        ),
        Ordering::Less
    );
    assert_eq!(ZLID::NIL.inspect().kind(), InspectionKind::Sentinel);
    assert_eq!(ZLID::MAX.inspect().kind(), InspectionKind::Sentinel);

    println!("{}", ordered.text());
    println!("{}", random.text());
    println!("{}", alias.text());
    println!("{}", source == ordered);
    println!("{}", canonical.text());
    println!("{partition}");

    Ok(())
}
