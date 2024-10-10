use anyhow::Result;

use super::mixed_fs::MixedFS;

#[deprecated(note = "Use VerifiableMixed instead")]
pub trait Verifiable {
    #![allow(clippy::ptr_arg)]
    fn verify_self(&self, located: &String) -> Result<()>;
}
pub trait VerifiableMixed {
    #![allow(clippy::ptr_arg)]
    fn verify_self(&self, mixed_fs: &MixedFS) -> Result<()>;
}

/// 校验字符串的枚举值是否有效
/// 入参：(当前步骤名称),字段名称,值,枚举值1|枚举值2|...
#[macro_export]
macro_rules! verify_enum {
    ($step:expr,$field:expr,$val:expr,$($enum:pat_param)|+) => {
        if matches!($val.as_str(),$($enum)|+){
            Ok(())
        }else{
            Err(anyhow::anyhow!("Error({}):Illegal enumeration value at field '{}' : expected to be one of [{}], got '{}'",
            $step,
            $field,
            stringify!($($enum),+),
            $val,
        ))
        }
    };
    ($field:expr,$val:expr,$($enum:pat_param)|+) => {
        if matches!($val.as_str(),$($enum)|+){
            Ok(())
        }else{
            Err(anyhow::anyhow!("Error:Illegal enumeration value at field '{}' : expected to be one of [{}], got '{}'",
            $field,
            stringify!($($enum),+),
            $val,
        ))
        }
    };
}
