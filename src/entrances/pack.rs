use anyhow::{Result,anyhow};
use crate::compression::{compress,pack_tar};
use crate::signature::{sign,verify};
use std::path::Path;
use crate::types::Signature;
use std::fs::{create_dir_all,remove_dir_all, File,write};

pub fn pack(source_dir:String,into_file:String,file_stem:String,packager:String,need_sign:bool)->Result<()>{
    // 打包检查
    let manifest=vec!["package.toml","workflows"];
    for file_name in manifest{
        let p=Path::new(&source_dir).join(file_name);
        if !p.exists() {
            return Err(anyhow!("Error:Missing '{}' in '{}', can't pack to nep",&file_name,&source_dir));
        }
    }

    // 创建临时目录
    let temp_dir_path=Path::new("./temp").join(&file_stem);
    if temp_dir_path.exists(){
        remove_dir_all(&temp_dir_path);
    }
    create_dir_all(&temp_dir_path);

    // 生成内包
    let inner_path_str=temp_dir_path.join(&(file_stem.clone()+".tar.zst")).to_string_lossy().to_string();
    compress(source_dir, inner_path_str.clone())?;

    // 对内包进行签名
    let signature=if need_sign {
        let signature=sign(inner_path_str.clone())?;
        Some(signature)
    }else{
        None
    };
    let sign_file_path=temp_dir_path.join("signature.toml");
    let signature_struct=Signature{
        packager,
        signature
    };
    let text=toml::to_string_pretty(&signature_struct)?;
    write(sign_file_path,&text)?;

    // 生成外包
    pack_tar(temp_dir_path.to_string_lossy().to_string(), into_file)?;

    Ok(())
}

#[test]
fn test_pack(){
    let res=pack(
        "./examples/VSCode".to_string(),
        "./examples/VSCode_1.0.0.0_Cno.nep".to_string(),
        "VSCode_1.0.0.0_Cno".to_string(),
        "test@edgeless.top".to_string(),
        true);
        println!("{:?}",res);
}