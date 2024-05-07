use anyhow::{anyhow, Result};
use fs_extra::file::read_to_string;
use semver::VersionReq;
use std::path::PathBuf;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::ReloadPolicy;

use toml::from_str;

use crate::entrances::info_online;
use crate::types::matcher::PackageMatcher;
use crate::types::mirror::MirrorPkgSoftwareRelease;
use crate::types::mirror::SearchResult;
use crate::{
    p2s,
    types::{
        mirror::{MirrorHello, MirrorPkgSoftware, Service, ServiceKeys},
        verifiable::Verifiable,
    },
    utils::get_path_mirror,
};

use super::download::fill_url_template;
use super::fs::ensure_dir_exist;
use super::fs::try_recycle;
use super::path::find_scope_with_name;

// 读取 meta
pub fn read_local_mirror_hello(name: &String) -> Result<(MirrorHello, PathBuf)> {
    let dir_path = get_path_mirror()?.join(name);
    let p = dir_path.join("hello.toml");
    if !p.exists() {
        return Err(anyhow!("Error:Mirror '{name}' hasn't been added"));
    }
    let text = read_to_string(&p)?;
    let hello: MirrorHello = from_str(&text)
        .map_err(|e| anyhow!("Error:Invalid hello content at '{fp}' : {e}", fp = p2s!(p)))?;
    hello.verify_self(&"".to_string())?;
    Ok((hello, dir_path))
}

// 读取 pkg-software
pub fn read_local_mirror_pkg_software(name: &String) -> Result<MirrorPkgSoftware> {
    let p = get_path_mirror()?.join(name).join("pkg-software.toml");
    if !p.exists() {
        return Err(anyhow!("Error:Mirror '{name}' hasn't been added"));
    }
    let text = read_to_string(&p)?;
    let pkg_software: MirrorPkgSoftware = from_str(&text).map_err(|e| {
        anyhow!(
            "Error:Invalid pkg software content at '{fp}' : {e}",
            fp = p2s!(p)
        )
    })?;
    Ok(pkg_software)
}

// 从 meta 中筛选出服务，返回的第一个参数是拼接了 root_url 后的路径
pub fn filter_service_from_meta(hello: MirrorHello, key: ServiceKeys) -> Result<(String, Service)> {
    let res = hello.service.iter().find(|s| s.key == key);
    if let Some(r) = res {
        Ok((format!("{r}{p}", r = hello.root_url, p = r.path), r.clone()))
    } else {
        Err(anyhow!(
            "Error:Failed to find service '{key:?}' in current mirror meta"
        ))
    }
}

fn get_schema() -> Result<(Schema, Field, Field, Field)> {
    let mut schema_builder = Schema::builder();
    let name = schema_builder.add_text_field("name", TEXT | STORED);
    let scope = schema_builder.add_text_field("scope", TEXT | STORED);
    let version = schema_builder.add_text_field("version", TEXT | STORED);
    Ok((schema_builder.build(), name, scope, version))
}

// 为包构建索引
pub fn build_index_for_mirror(content: MirrorPkgSoftware, dir: PathBuf) -> Result<()> {
    let (schema, name, scope, version) = get_schema()?;
    if dir.exists() {
        try_recycle(&dir)?;
    }
    ensure_dir_exist(&dir)?;
    let index = Index::create_in_dir(&dir, schema.clone())?;
    let mut index_writer = index.writer(50_000_000)?;
    for (scope_str, node) in content.tree.iter() {
        for item in node {
            // 筛选出最高版本号
            let releases = item.releases.to_owned();
            let latest = filter_release(releases, None)?.version.to_string();
            index_writer.add_document(doc!(
              name => item.name.as_str(),
              scope => scope_str.as_str(),
              version => latest.as_str(),
            ))?;
        }
    }
    index_writer.commit()?;

    Ok(())
}

// 从索引中搜索内容
pub fn search_index_for_mirror(text: &String, dir: PathBuf) -> Result<Vec<SearchResult>> {
    let (_schema, name, scope, version) = get_schema()?;

    let index = Index::open_in_dir(dir)?;
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommitWithDelay)
        .try_into()?;
    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![name]);
    let query = query_parser.parse_query(text)?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    let mut arr = Vec::new();
    for (_score, doc_address) in top_docs {
        let res: TantivyDocument = searcher.doc(doc_address)?;
        let read_field = |field: Field| {
            let str = res.get_first(field);
            if let Some(s) = str {
                Ok(s.as_str().unwrap_or("").to_string())
            } else {
                Err(anyhow!("Error:Failed to restore data from index '{res:?}'"))
            }
        };
        arr.push(SearchResult {
            name: read_field(name)?,
            scope: read_field(scope)?,
            version: read_field(version)?,
            from_mirror: None,
        })
    }

    Ok(arr)
}

// 如果没有提供 semver matcher 则返回最大版本
pub fn filter_release(
    releases: Vec<MirrorPkgSoftwareRelease>,
    semver_matcher: Option<VersionReq>,
) -> Result<MirrorPkgSoftwareRelease> {
    // 筛选 matcher
    let mut req_str = "".to_string();
    let mut arr = if let Some(matcher) = semver_matcher {
        req_str = matcher.to_string();
        let res_arr: Vec<MirrorPkgSoftwareRelease> = releases
            .iter()
            .filter(|node| matcher.matches(&node.version.semver_instance))
            .cloned()
            .collect();
        res_arr
    } else {
        releases.clone()
    };
    arr.sort_by(|a, b| b.version.cmp(&a.version));
    if let Some(f) = arr.first() {
        Ok(f.to_owned())
    } else {
        let versions: Vec<String> = releases
            .iter()
            .map(|node| node.version.to_string())
            .collect();
        Err(anyhow!(
            "Error:No releases matched with req '{req_str}', available versions : '{v}'",
            v = versions.join(", ")
        ))
    }
}

// 通过匹配 VersionReq 解析出包的 url
pub fn get_url_with_version_req(
    matcher: PackageMatcher,
) -> Result<(String, MirrorPkgSoftwareRelease)> {
    // 查找 scope 并使用 scope 更新纠正大小写
    let (scope, package_name) = find_scope_with_name(&matcher.name, matcher.scope)?;
    // 拿到 info online
    let (info, url_template) = info_online(&scope, &package_name, matcher.mirror)?;
    // 匹配版本
    let matched_release = filter_release(info.releases, matcher.version_req)?;
    // 填充模板获取 url
    let url = fill_url_template(
        &url_template,
        &scope,
        &info.name,
        &matched_release.file_name,
    )?;
    Ok((url, matched_release))
}

#[test]
fn test_filter_release() {
    use crate::types::extended_semver::ExSemVer;
    let arr = vec![
        MirrorPkgSoftwareRelease {
            file_name: "VSCode_1.85.1.0_Cno.nep".to_string(),
            version: ExSemVer::parse(&"1.85.1.0".to_string()).unwrap(),
            size: 94245376,
            timestamp: 1704554724,
            integrity: None,
        },
        MirrorPkgSoftwareRelease {
            file_name: "VSCode_1.86.1.0_Cno.nep".to_string(),
            version: ExSemVer::parse(&"1.86.1.0".to_string()).unwrap(),
            size: 94245376,
            timestamp: 1704554724,
            integrity: None,
        },
        MirrorPkgSoftwareRelease {
            file_name: "VSCode_1.85.2.0_Cno.nep".to_string(),
            version: ExSemVer::parse(&"1.85.2.0".to_string()).unwrap(),
            size: 94245376,
            timestamp: 1704554724,
            integrity: None,
        },
    ];
    let res = filter_release(arr, None).unwrap();
    assert_eq!(res.version.to_string(), "1.86.1.0".to_string());

    let arr = vec![
        MirrorPkgSoftwareRelease {
            file_name: "Chrome_120.0.6099.200_Cno.nep".to_string(),
            version: ExSemVer::parse(&"120.0.6099.200".to_string()).unwrap(),
            size: 133763072,
            timestamp: 1704554608,
            integrity: None,
        },
        MirrorPkgSoftwareRelease {
            file_name: "Chrome_121.0.6099.200_Cno.nep".to_string(),
            version: ExSemVer::parse(&"121.0.6099.200".to_string()).unwrap(),
            size: 133763072,
            timestamp: 1704554608,
            integrity: None,
        },
        MirrorPkgSoftwareRelease {
            file_name: "Chrome_122.0.6099.200_Cno.nep".to_string(),
            version: ExSemVer::parse(&"122.0.6099.200".to_string()).unwrap(),
            size: 133763072,
            timestamp: 1704554608,
            integrity: None,
        },
    ];
    let res = filter_release(arr, Some(VersionReq::parse("121").unwrap())).unwrap();
    assert_eq!(res.version.to_string(), "121.0.6099.200".to_string());
}

// #[test]
// fn test_build_index_for_mirror() {
//     build_index_for_mirror(
//         MirrorPkgSoftware::_demo(),
//         get_path_mirror().unwrap().join("official").join("index"),
//     )
//     .unwrap();
// }

// #[test]
// fn test_search_index_for_mirror() {
//     let p = get_path_mirror().unwrap().join("official").join("index");
//     let r = search_index_for_mirror(&"Code".to_string(), p.clone()).unwrap();
//     println!("{r:#?}");
// }
