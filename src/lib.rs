// use chrono::{DateTime, Utc};

#[derive(::rkyv::Archive, ::rkyv::Deserialize, ::rkyv::Serialize, Debug, PartialEq)]
// #[archive(compare(PartialEq), check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Meta {
    pub name: String,
    pub one: String,
    pub two: String,
    pub three: String,
    pub age: u8,
    // blocked on chrono PR (support for PartialEq/PartialOrd)
    // pub finalized: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{Read, Write},
    };

    use const_format::formatcp;
    use fake::{Fake, Faker};
    use memmap2::Mmap;
    use rkyv::{
        archived_root,
        ser::{serializers::AllocSerializer, Serializer},
    };

    use crate::Meta;

    const DIR: &str = env!("CARGO_MANIFEST_DIR");
    const SRC: &str = formatcp!("{}{}{}", DIR, '/', "archive.zip");
    const DST: &str = formatcp!("{}{}{}", DIR, '/', "archive.cust");

    const NAME: &str = "Jean-Guy";
    const AGE: u8 = 35;

    const SIZE_LEN: usize = 4;

    #[allow(unused_variables)]
    fn setup() -> Vec<u8> {
        let mut file = File::open(SRC).expect("open zip file");
        let mut blob = Vec::new();
        file.read_to_end(&mut blob).expect("read zip file to bytes");
        dbg!(&blob.len());

        let meta = Meta {
            name: NAME.to_owned(),
            one: Faker.fake(),
            two: Faker.fake(),
            three: Faker.fake(),
            age: AGE,
        };
        let mut serializer = AllocSerializer::<1024>::default();
        serializer
            .serialize_value(&meta)
            .expect("failed to archive meta");
        let bytes = serializer.into_serializer().into_inner();
        let mut file = File::create(DST).expect("open cust file");
        file.write(&(bytes.len() as u32).to_le_bytes())
            .expect("write cust pos bytes");
        file.write(bytes.as_slice()).expect("write cust meta bytes");
        let written = file.write(blob.as_slice()).expect("write cust zip bytes");
        dbg!(written);
        blob
    }

    #[test]
    fn mmap_partial() {
        let blob = setup();

        let file = File::open(DST).expect("open cust file");
        let mmap = unsafe { Mmap::map(&file).expect("map file in memory") };
        let header_len: [u8; SIZE_LEN] = mmap[..SIZE_LEN].try_into().expect("read pos");
        let header_len = u32::from_le_bytes(header_len) as usize;
        dbg!(&header_len);
        let archived = unsafe { archived_root::<Meta>(&mmap[SIZE_LEN..SIZE_LEN + header_len]) };
        assert_eq!(archived.name, NAME);
        assert_eq!(archived.age, AGE);
        dbg!(&archived.one, &archived.two, &archived.three);
        let zip = &mmap[SIZE_LEN + header_len..];
        dbg!(zip[..].len());
        assert_eq!(zip.len(), blob.len());
        assert_eq!(zip[..], blob[..]);
    }
}
