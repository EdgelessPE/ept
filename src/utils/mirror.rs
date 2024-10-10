use anyhow::{anyhow, Result};
use fs_extra::file::read_to_string;
use semver::VersionReq;
use std::cmp::Ordering;
use std::path::PathBuf;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::query::RegexQuery;
use tantivy::schema::*;
use tantivy::tokenizer::*;
use tantivy::Index;
use tantivy::ReloadPolicy;
use toml::from_str;

use crate::entrances::info_online;
use crate::types::matcher::PackageMatcher;
use crate::types::mirror::MirrorPkgSoftwareRelease;
use crate::types::mirror::SearchResult;
use crate::types::mixed_fs::MixedFS;
use crate::{
    p2s,
    types::{
        mirror::{MirrorHello, MirrorPkgSoftware, Service, ServiceKeys},
        verifiable::Verifiable,
    },
    utils::get_path_mirror,
};

use super::cfg::get_config;
use super::cfg::get_flags_score;
use super::constants::MIRROR_FILE_HELLO;
use super::constants::MIRROR_FILE_PKG_SOFTWARE;
use super::download::fill_url_template;
use super::fs::ensure_dir_exist;
use super::fs::try_recycle;
use super::path::find_scope_with_name;

// 读取 meta
pub fn read_local_mirror_hello(name: &String) -> Result<(MirrorHello, PathBuf)> {
    let dir_path = get_path_mirror()?.join(name);
    let p = dir_path.join(MIRROR_FILE_HELLO);
    if !p.exists() {
        return Err(anyhow!("Error:Mirror '{name}' hasn't been added"));
    }
    let text = read_to_string(&p)?;
    let hello: MirrorHello = from_str(&text)
        .map_err(|e| anyhow!("Error:Invalid hello content at '{fp}' : {e}", fp = p2s!(p)))?;
    hello.verify_self(&MixedFS::new(""))?;
    Ok((hello, dir_path))
}

// 读取 pkg-software
pub fn read_local_mirror_pkg_software(name: &String) -> Result<MirrorPkgSoftware> {
    let p = get_path_mirror()?.join(name).join(MIRROR_FILE_PKG_SOFTWARE);
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
pub fn filter_service_from_meta(
    hello: &MirrorHello,
    key: ServiceKeys,
) -> Result<(String, Service)> {
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
    let opt = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("jieba")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();
    let name = schema_builder.add_text_field("name", opt.clone());
    let scope = schema_builder.add_text_field("scope", opt.clone());
    let version = schema_builder.add_text_field("version", opt);
    Ok((schema_builder.build(), name, scope, version))
}

fn register_tokenizer(index: &mut Index) {
    let tokenizer = tantivy_jieba::JiebaTokenizer {};
    let analyzer = TextAnalyzer::builder(tokenizer)
        .filter(RemoveLongFilter::limit(40))
        .filter(LowerCaser)
        .filter(Stemmer::default())
        .build();
    index.tokenizers().register("jieba", analyzer);
}

// 为包构建索引
pub fn build_index_for_mirror(content: MirrorPkgSoftware, dir: PathBuf) -> Result<()> {
    let (schema, name, scope, version) = get_schema()?;
    if dir.exists() {
        try_recycle(&dir)?;
    }
    ensure_dir_exist(&dir)?;
    let mut index = Index::create_in_dir(&dir, schema.clone())?;
    register_tokenizer(&mut index);
    let mut index_writer = index.writer(50_000_000)?;
    for (scope_str, node) in content.tree.iter() {
        for item in node {
            // 筛选出最高版本号
            let releases = item.releases.to_owned();
            let latest = filter_release(releases, None, false)?.version.to_string();
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
pub fn search_index_for_mirror(
    text: &str,
    dir: PathBuf,
    is_regex: bool,
) -> Result<Vec<SearchResult>> {
    let (_schema, name, scope, version) = get_schema()?;

    let mut index = Index::open_in_dir(dir)?;
    register_tokenizer(&mut index);
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommitWithDelay)
        .try_into()?;
    let searcher = reader.searcher();
    log!(
        "Debug:Searching index for '{text}' ({})",
        if is_regex { "regex" } else { "text" }
    );
    let top_docs = if is_regex {
        let query = RegexQuery::from_pattern(text, name)
            .map_err(|e| anyhow!("Error:Invalid regex : {e}"))?;
        searcher.search(&query, &TopDocs::with_limit(10))?
    } else {
        let query_parser = QueryParser::for_index(&index, vec![name]);
        let query = query_parser.parse_query(text)?;
        searcher.search(&query, &TopDocs::with_limit(10))?
    };

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
    enable_flags_score: bool,
) -> Result<MirrorPkgSoftwareRelease> {
    let cfg = get_config();
    // 筛选 matcher
    let matcher_str = semver_matcher
        .clone()
        .map_or_else(|| "None".to_string(), |m| m.to_string());
    let mut req_str = "".to_string();
    let arr = if let Some(matcher) = semver_matcher {
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
    // 计算各个 release 的 flags 得分
    let mut arr_with_score: Vec<(MirrorPkgSoftwareRelease, i32)> = arr
        .into_iter()
        .map(|node| {
            let score = if enable_flags_score {
                node.get_flags()
                    .map(|flags| {
                        get_flags_score(&flags, &cfg)
                            .map_err(|e| {
                                anyhow!(
                                    "Error:Failed to calculate flags score for '{}' : {e}",
                                    node.file_name
                                )
                            })
                            .unwrap()
                    })
                    .unwrap_or_default()
            } else {
                0
            };
            (node, score)
        })
        .collect();
    arr_with_score.sort_by(|(a, a_score), (b, b_score)| {
        // 优先按照版本号排序
        let ver_cmp_res = b.version.cmp(&a.version);
        // 版本号一致时使用 flags 的分数排序
        if ver_cmp_res == Ordering::Equal {
            b_score.cmp(a_score)
        } else {
            ver_cmp_res
        }
    });
    if let Some((f, score)) = arr_with_score.first() {
        log!(
            "Debug:Matched version '{}' ('{}', score:{score}) with matcher '{matcher_str}'",
            f.version.to_string(),
            f.file_name
        );
        if *score >= 0 {
            Ok(f.to_owned())
        } else {
            Err(anyhow!(
                "Error:The latest release ('{}') is blocked due to configured preference policy, try change your preference in config",
                f.file_name
            ))
        }
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
    let matched_release = filter_release(info.releases, matcher.version_req, true)?;
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
    // 直接筛选最高版本
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
    let res = filter_release(arr, None, false).unwrap();
    assert_eq!(res.version.to_string(), "1.86.1.0".to_string());

    // 使用 matcher
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
    let res = filter_release(arr, Some(VersionReq::parse("121").unwrap()), false).unwrap();
    assert_eq!(res.version.to_string(), "121.0.6099.200".to_string());
}

#[test]
fn test_filter_release_with_flags() {
    use crate::utils::flags::{set_flag, Flag};
    set_flag(Flag::Debug, true);
    use crate::types::cfg::PreferenceEnum;
    use crate::types::extended_semver::ExSemVer;
    use crate::utils::cfg::set_config;
    use std::str::FromStr;
    let cfg_bak = get_config();

    let releases = vec![
        MirrorPkgSoftwareRelease {
            file_name: "Firefox_127.0.0.1_Cno.I.nep".to_string(),
            version: ExSemVer::from_str("127.0.0.1").unwrap(),
            size: 94245376,
            timestamp: 1704554724,
            integrity: None,
        },
        MirrorPkgSoftwareRelease {
            file_name: "Firefox_127.0.0.1_Cno.IE.nep".to_string(),
            version: ExSemVer::from_str("127.0.0.1").unwrap(),
            size: 94245376,
            timestamp: 1704554724,
            integrity: None,
        },
        MirrorPkgSoftwareRelease {
            file_name: "Firefox_127.0.0.1_Cno.P.nep".to_string(),
            version: ExSemVer::from_str("127.0.0.1").unwrap(),
            size: 94245376,
            timestamp: 1704554724,
            integrity: None,
        },
        MirrorPkgSoftwareRelease {
            file_name: "Firefox_127.0.0.1_Cno.PE.nep".to_string(),
            version: ExSemVer::from_str("127.0.0.1").unwrap(),
            size: 94245376,
            timestamp: 1704554724,
            integrity: None,
        },
    ];

    let modifier = |i: PreferenceEnum, p: PreferenceEnum, e: PreferenceEnum| {
        let mut cfg = cfg_bak.clone();
        cfg.preference.installer = i;
        cfg.preference.portable = p;
        cfg.preference.expandable = e;
        set_config(cfg).unwrap();
    };

    // 默认优先级配置，会匹配到 PE 版本
    modifier(
        PreferenceEnum::LowPriority,
        PreferenceEnum::HighPriority,
        PreferenceEnum::HighPriority,
    );
    assert_eq!(
        filter_release(releases.clone(), None, true)
            .unwrap()
            .file_name,
        "Firefox_127.0.0.1_Cno.PE.nep".to_string()
    );

    // 便携且不要可拓展模式，匹配到 P 版本
    modifier(
        PreferenceEnum::Forbidden,
        PreferenceEnum::HighPriority,
        PreferenceEnum::Forbidden,
    );
    assert_eq!(
        filter_release(releases.clone(), None, true)
            .unwrap()
            .file_name,
        "Firefox_127.0.0.1_Cno.P.nep".to_string()
    );

    // 全部禁用，会报错
    modifier(
        PreferenceEnum::Forbidden,
        PreferenceEnum::Forbidden,
        PreferenceEnum::Forbidden,
    );
    assert!(filter_release(releases.clone(), None, true).is_err());

    // 恢复原有配置
    set_config(cfg_bak).unwrap();
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
