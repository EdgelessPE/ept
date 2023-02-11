use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Author {
    pub name: String,
    pub email: Option<String>,
}

impl PartialEq for Author {
    fn eq(&self,other:&Self)->bool{
        // 如果两个都有邮箱则判断为一致
        if self.email.is_some() && other.email.is_some(){
            return true;
        }
        self.name==other.name
    }
    fn ne(&self,other:&Self)->bool{
        !self.eq(other)
    }
}