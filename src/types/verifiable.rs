use anyhow::Result;

pub trait Verifiable {
    fn verify_self(&self, located: &String) -> Result<()>;
}

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
}
