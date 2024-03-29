use std::{fs, path::Path};

fn main() -> std::io::Result<()> {
    let protos_path = Path::new("protos");
    let im_path = protos_path.join("im.basic");
    let zt_path = protos_path.join("zt.live.interactive");
    println!("cargo:rerun-if-changed={}", im_path.to_str().unwrap());
    println!("cargo:rerun-if-changed={}", zt_path.to_str().unwrap());
    let proto_im = fs::read_dir(&im_path)?;
    let mut protos: Vec<_> = proto_im.map(|r| r.unwrap().path()).collect();
    let proto_zt = fs::read_dir(&zt_path)?;
    protos.append(&mut proto_zt.map(|r| r.unwrap().path()).collect());
    let mut config = prost_build::Config::new();
    config.type_attribute(
        ".",
        "#[cfg_attr(feature = \"_serde\", derive(::serde::Deserialize, ::serde::Serialize))]",
    );
    config.compile_protos(&protos, &[im_path, zt_path])?;

    Ok(())
}
