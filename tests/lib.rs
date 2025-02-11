use std::path::Path;
use std::{fs::File, path::PathBuf};
use tiled::{
    error::TiledError, layers::LayerData, map::Map, properties::PropertyValue, tileset::Tileset,
};

fn parse_map_without_source(p: impl AsRef<Path>) -> Result<Map, TiledError> {
    let file = File::open(p).unwrap();
    return Map::parse_reader(file, None);
}

#[test]
fn test_gzip_and_zlib_encoded_and_raw_are_the_same() {
    let z = Map::parse_file("assets/tiled_base64_zlib.tmx").unwrap();
    let g = Map::parse_file("assets/tiled_base64_gzip.tmx").unwrap();
    let r = Map::parse_file("assets/tiled_base64.tmx").unwrap();
    let zstd = Map::parse_file("assets/tiled_base64_zstandard.tmx").unwrap();
    let c = Map::parse_file("assets/tiled_csv.tmx").unwrap();
    assert_eq!(z, g);
    assert_eq!(z, r);
    assert_eq!(z, c);
    assert_eq!(z, zstd);

    if let LayerData::Finite(tiles) = &c.layers[0].tiles {
        assert_eq!(tiles.len(), 100);
        assert_eq!(tiles[0].len(), 100);
        assert_eq!(tiles[99].len(), 100);
        assert_eq!(tiles[0][0].gid, 35);
        assert_eq!(tiles[1][0].gid, 17);
        assert_eq!(tiles[2][0].gid, 0);
        assert_eq!(tiles[2][1].gid, 17);
        assert!(tiles[99].iter().map(|t| t.gid).all(|g| g == 0));
    } else {
        assert!(false, "It is wrongly recognised as an infinite map");
    }
}

#[test]
fn test_external_tileset() {
    let r = Map::parse_file("assets/tiled_base64.tmx").unwrap();
    let mut e = Map::parse_file("assets/tiled_base64_external.tmx").unwrap();
    e.tilesets[0].source = None;
    assert_eq!(r, e);
}

#[test]
fn test_sources() {
    let e = Map::parse_file("assets/tiled_base64_external.tmx").unwrap();
    assert_eq!(
        e.tilesets[0].source,
        Some(PathBuf::from("assets/tilesheet.tsx"))
    );
    assert_eq!(
        e.tilesets[0].image.as_ref().unwrap().source,
        PathBuf::from("assets/tilesheet.png")
    );
}

#[test]
fn test_just_tileset() {
    let r = Map::parse_file("assets/tiled_base64_external.tmx").unwrap();
    let path = "assets/tilesheet.tsx";
    let t = Tileset::parse_with_path(File::open(path).unwrap(), 1, path).unwrap();
    assert_eq!(r.tilesets[0], t);
}

#[test]
fn test_infinite_tileset() {
    let r = Map::parse_file("assets/tiled_base64_zlib_infinite.tmx").unwrap();

    if let LayerData::Infinite(chunks) = &r.layers[0].tiles {
        assert_eq!(chunks.len(), 4);

        assert_eq!(chunks[&(0, 0)].width, 32);
        assert_eq!(chunks[&(0, 0)].height, 32);
        assert_eq!(chunks[&(-32, 0)].width, 32);
        assert_eq!(chunks[&(0, 32)].height, 32);
        assert_eq!(chunks[&(-32, 32)].height, 32);
    } else {
        assert!(false, "It is wrongly recognised as a finite map");
    }
}

#[test]
fn test_image_layers() {
    let r = Map::parse_file("assets/tiled_image_layers.tmx").unwrap();
    assert_eq!(r.image_layers.len(), 2);
    {
        let first = &r.image_layers[0];
        assert_eq!(first.name, "Image Layer 1");
        assert!(
            first.image.is_none(),
            "{}'s image should be None",
            first.name
        );
    }
    {
        let second = &r.image_layers[1];
        assert_eq!(second.name, "Image Layer 2");
        let image = second
            .image
            .as_ref()
            .expect(&format!("{}'s image shouldn't be None", second.name));
        assert_eq!(image.source, PathBuf::from("assets/tilesheet.png"));
        assert_eq!(image.width, 448);
        assert_eq!(image.height, 192);
    }
}

#[test]
fn test_tile_property() {
    let r = Map::parse_file("assets/tiled_base64.tmx").unwrap();
    let prop_value: String = if let Some(&PropertyValue::StringValue(ref v)) =
        r.tilesets[0].tiles[0].properties.get("a tile property")
    {
        v.clone()
    } else {
        String::new()
    };
    assert_eq!("123", prop_value);
}

#[test]
fn test_layer_property() {
    let r = Map::parse_file(&Path::new("assets/tiled_base64.tmx")).unwrap();
    let prop_value: String =
        if let Some(&PropertyValue::StringValue(ref v)) = r.layers[0].properties.get("prop3") {
            v.clone()
        } else {
            String::new()
        };
    assert_eq!("Line 1\r\nLine 2\r\nLine 3,\r\n  etc\r\n   ", prop_value);
}

#[test]
fn test_object_group_property() {
    let r = Map::parse_file("assets/tiled_object_groups.tmx").unwrap();
    let prop_value: bool = if let Some(&PropertyValue::BoolValue(ref v)) = r.object_groups[0]
        .properties
        .get("an object group property")
    {
        *v
    } else {
        false
    };
    assert!(prop_value);
}
#[test]
fn test_tileset_property() {
    let r = Map::parse_file("assets/tiled_base64.tmx").unwrap();
    let prop_value: String = if let Some(&PropertyValue::StringValue(ref v)) =
        r.tilesets[0].properties.get("tileset property")
    {
        v.clone()
    } else {
        String::new()
    };
    assert_eq!("tsp", prop_value);
}

#[test]
fn test_flipped_gid() {
    let r = Map::parse_file("assets/tiled_flipped.tmx").unwrap();

    if let LayerData::Finite(tiles) = &r.layers[0].tiles {
        let t1 = tiles[0][0];
        let t2 = tiles[0][1];
        let t3 = tiles[1][0];
        let t4 = tiles[1][1];
        assert_eq!(t1.gid, t2.gid);
        assert_eq!(t2.gid, t3.gid);
        assert_eq!(t3.gid, t4.gid);
        assert!(t1.flip_d);
        assert!(t1.flip_h);
        assert!(t1.flip_v);
        assert!(!t2.flip_d);
        assert!(!t2.flip_h);
        assert!(t2.flip_v);
        assert!(!t3.flip_d);
        assert!(t3.flip_h);
        assert!(!t3.flip_v);
        assert!(t4.flip_d);
        assert!(!t4.flip_h);
        assert!(!t4.flip_v);
    } else {
        assert!(false, "It is wrongly recognised as an infinite map");
    }
}

#[test]
fn test_ldk_export() {
    let r = Map::parse_file("assets/ldk_tiled_export.tmx").unwrap();
    if let LayerData::Finite(tiles) = &r.layers[0].tiles {
        assert_eq!(tiles.len(), 8);
        assert_eq!(tiles[0].len(), 8);
        assert_eq!(tiles[0][0].gid, 0);
        assert_eq!(tiles[1][0].gid, 1);
    } else {
        assert!(false, "It is wrongly recognised as an infinite map");
    }
}

#[test]
fn test_object_property() {
    let r = parse_map_without_source(&Path::new("assets/tiled_object_property.tmx")).unwrap();
    let prop_value = if let Some(PropertyValue::ObjectValue(v)) = r.object_groups[0].objects[0]
        .properties
        .get("object property")
    {
        *v
    } else {
        0
    };
    assert_eq!(3, prop_value);
}
