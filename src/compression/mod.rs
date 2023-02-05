mod tar;
mod zstd;

use self::tar::{pack_tar,release_tar};
use self::zstd::{compress_zstd,decompress_zstd};
use anyhow::{Result, anyhow};
use std::fs::{remove_file};
use std::path::Path;

fn get_temp_tar(zstd_file:String)->String{
    let p=Path::new(&zstd_file);
    let stem=p.file_stem().unwrap().to_string_lossy().to_string();
    return p.with_file_name(&stem).to_string_lossy().to_string();
}

#[test]
fn test_get_temp_tar(){
    let res=get_temp_tar(r"D:\Desktop\Projects\EdgelessPE\ept\examples\VSCode\Pack.tar.zst".to_string());
    assert_eq!(res,r"D:\Desktop\Projects\EdgelessPE\ept\examples\VSCode\Pack.tar".to_string());
}

pub fn compress(source_dir:String,into_file:String)->Result<()>{
    let temp_tar=get_temp_tar(into_file.clone());
    let tar_res=pack_tar(source_dir.clone(), temp_tar.clone());
    if tar_res.is_err() {
        return Err(anyhow!("Error:Can't archive '{}' into '{}' : {}",&source_dir,&temp_tar,tar_res.unwrap_err()));
    }

    let zst_res=compress_zstd(temp_tar.clone(), into_file.clone());
    if zst_res.is_err() {
        return Err(anyhow!("Error:Can't compress '{}' into '{}' : {}",&temp_tar,&into_file,zst_res.unwrap_err()));
    }

    let rm_res=remove_file(&temp_tar);
    if rm_res.is_err() {
        return Err(anyhow!("Warning:Can't remove temp tar '{}' : {}",&temp_tar,rm_res.unwrap_err()));
    }

    Ok(())
}

pub fn decompress(source_file:String,into_dir:String)->Result<()>{
    let temp_tar=get_temp_tar(source_file.clone());
    let zst_res=decompress_zstd(source_file.clone(), temp_tar.clone());
    if zst_res.is_err() {
        return Err(anyhow!("Error:Can't decompress '{}' into '{}' : {}",&source_file,&temp_tar,zst_res.unwrap_err()));
    }

    let tar_res=release_tar(temp_tar.clone(), into_dir.clone());
    if tar_res.is_err() {
        return Err(anyhow!("Error:Can't release '{}' into '{}' : {}",&temp_tar,&into_dir,tar_res.unwrap_err()));
    }

    let rm_res=remove_file(&temp_tar);
    if rm_res.is_err() {
        return Err(anyhow!("Warning:Can't remove temp tar '{}' : {}",&temp_tar,rm_res.unwrap_err()));
    }

    Ok(())
}

#[test]
fn test_compress(){
    let res=compress(
        r"D:\Desktop\Projects\EdgelessPE\ept\examples\VSCode".to_string(),
       r"./examples/VSCode_1.0.0.0_Cno.tar.zst".to_string(),
       );
   println!("{:?}",res);
}

#[test]
fn test_decompress(){
    let res=decompress(
       r"./VSCode_1.0.0.0_Cno.tar.zst".to_string(),
        r"./VSCode_1.0.0.0_Cno".to_string(),
       );
   println!("{:?}",res);
}