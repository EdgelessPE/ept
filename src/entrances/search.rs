use anyhow::{anyhow, Result};

use crate::{
    types::mirror::SearchResult,
    utils::{fs::read_sub_dir, get_path_mirror, mirror::search_index_for_mirror},
};

pub fn search(text: &String) -> Result<Vec<SearchResult>> {
    // 扫描出所有的镜像源目录
    let root = get_path_mirror()?;
    let mirror_dirs = read_sub_dir(&root)?;
    if mirror_dirs.len() == 0 {
        return Err(anyhow!("Error:No mirror added yet"));
    }

    // 添加扫描结果
    let mut arr = Vec::new();
    for mirror_name in mirror_dirs {
        let search_res = search_index_for_mirror(text, root.join(&mirror_name).join("index"))?;
        let mut mapped: Vec<SearchResult> = search_res
            .iter()
            .map(|raw| {
                let mut node = raw.to_owned();
                node.from_mirror = Some(mirror_name.clone());
                node
            })
            .collect();
        arr.append(&mut mapped);
    }

    if arr.len() == 0 {
        Err(anyhow!("Error:No result found with keyword '{text}'"))
    } else {
        Ok(arr)
    }
}

// #[test]
// fn test_search() {
//     let res = search(&"vscode".to_string()).unwrap();
//     println!("{res:#?}");
// }
