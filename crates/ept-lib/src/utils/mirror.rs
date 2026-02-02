use anyhow::{anyhow, Result};
use fs_extra::file::read_to_string;
use semver::VersionReq;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;
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
use crate::types::mirror::QuickMaps;
use crate::types::mirror::SearchResult;
use crate::types::mixed_fs::MixedFS;
use crate::types::permissions::PermissionKey;
use crate::{
    p2s,
    types::{
        context::VerifiableCtx,
        mirror::{MirrorHello, MirrorPkgSoftware, Service, ServiceKeys},
        verifiable::Verifiable,
    },
    utils::get_path_mirror,
};

use super::cfg::get_flags_score;
use super::constants::MIRROR_FILE_HELLO;
use super::constants::MIRROR_FILE_QUICK_MAP;
use super::download::fill_url_template;
use super::fs::ensure_dir_exist;
use super::fs::try_recycle;
use super::path::find_scope_with_name;
use super::permissions::filter_permissions;

// 读取 meta
pub fn read_local_mirror_hello(
    cfg: &crate::types::context::RuntimeContext,
    name: &str,
) -> Result<(MirrorHello, PathBuf)> {
    let dir_path = get_path_mirror(cfg)?.join(name);
    let p = dir_path.join(MIRROR_FILE_HELLO);
    if !p.exists() {
        return Err(anyhow!("Error:Mirror '{name}' hasn't been added"));
    }
    let text = read_to_string(&p)?;
    let hello: MirrorHello = from_str(&text)
        .map_err(|e| anyhow!("Error:Invalid hello content at '{fp}' : {e}", fp = p2s!(p)))?;
    let ctx = VerifiableCtx {
        mixed_fs: &MixedFS::new(""),
        runtime_ctx: cfg,
    };
    hello.verify_self(&ctx)?;
    Ok((hello, dir_path))
}

// 读取 pkg-software
// pub fn read_local_mirror_pkg_software(name: &str) -> Result<MirrorPkgSoftware> {
//     let p = get_path_mirror()?.join(name).join(MIRROR_FILE_PKG_SOFTWARE);
//     if !p.exists() {
//         return Err(anyhow!("Error:Mirror '{name}' hasn't been added"));
//     }
//     let text = read_to_string(&p)?;
//     let pkg_software: MirrorPkgSoftware = from_str(&text).map_err(|e| {
//         anyhow!(
//             "Error:Invalid pkg software content at '{fp}' : {e}",
//             fp = p2s!(p)
//         )
//     })?;
//     Ok(pkg_software)
// }

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

struct SchemaFields {
    schema: Schema,
    name: Field,
    scope: Field,
    version: Field,
    description: Field,
    tags: Field,
    bin: Field,
    alias: Field,
}

fn get_schema() -> Result<SchemaFields> {
    let mut schema_builder = Schema::builder();
    let opt = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("jieba")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();
    let name = schema_builder.add_text_field("name", opt.clone());
    let scope = schema_builder.add_text_field("scope", STORED);
    let version = schema_builder.add_text_field("version", STORED);
    let description = schema_builder.add_text_field("description", STORED);
    let tags = schema_builder.add_text_field("tags", TEXT | STORED);
    let bin = schema_builder.add_text_field("bin", TEXT | STORED);
    let alias = schema_builder.add_text_field("alias", TEXT | STORED);

    Ok(SchemaFields {
        schema: schema_builder.build(),
        name,
        scope,
        version,
        description,
        tags,
        bin,
        alias,
    })
}

fn register_tokenizer(index: &mut Index) {
    let tokenizer = tantivy_jieba::JiebaTokenizer::new();
    let analyzer = TextAnalyzer::builder(tokenizer)
        .filter(RemoveLongFilter::limit(40))
        .filter(LowerCaser)
        .filter(Stemmer::default())
        .build();
    index.tokenizers().register("jieba", analyzer);
}

// 为包构建索引
pub fn build_index_for_mirror(
    cfg: &crate::types::context::RuntimeContext,
    content: MirrorPkgSoftware,
    dir: PathBuf,
) -> Result<()> {
    let schema_fields = get_schema()?;
    if dir.exists() {
        try_recycle(&dir)?;
    }
    ensure_dir_exist(&dir)?;
    let mut index = Index::create_in_dir(&dir, schema_fields.schema.clone())?;
    register_tokenizer(&mut index);
    let mut index_writer = index.writer(50_000_000)?;
    // 快查索引
    let mut full_map = HashMap::new();
    let mut scope_map = HashMap::new();
    for (scope_str, node) in content.tree.iter() {
        for item in node {
            // 写快查索引
            let name_str = item.name.clone();
            full_map.insert(
                (scope_str.to_lowercase(), name_str.to_lowercase()),
                item.to_owned(),
            );
            scope_map
                .entry(name_str.to_lowercase())
                .or_insert_with(|| (Vec::new(), name_str.clone()))
                .0
                .push(scope_str.clone());

            // 构建搜索索引
            // 筛选出最高版本号
            let releases = item.releases.to_owned();
            if releases.is_empty() {
                continue;
            }
            let release = filter_release(cfg, releases, None, false)?;
            let meta_res = if let Some(meta) = release.meta {
                // 收集二进制文件
                let mut bin_stems: Vec<String> = Vec::new();
                for perm in filter_permissions(PermissionKey::path_entrances, meta.permissions) {
                    for target in perm.targets {
                        let p = Path::new(&target);
                        bin_stems.push(p2s!(p.file_stem().unwrap()));
                    }
                }
                let software = meta.package.software.unwrap();
                (
                    // 描述
                    meta.package.package.description,
                    // 标签
                    software.tags.unwrap_or_default().join(" "),
                    // 二进制
                    bin_stems.join(" "),
                    // 别名
                    software.alias.unwrap_or_default(),
                )
            } else {
                (
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                )
            };
            index_writer.add_document(doc!(
              schema_fields.name => item.name.as_str(),
              schema_fields.scope => scope_str.as_str(),
              schema_fields.version => release.version.to_string().as_str(),
              schema_fields.description => meta_res.0.as_str(),
              schema_fields.tags => meta_res.1.as_str(),
              schema_fields.bin => meta_res.2.as_str(),
              schema_fields.alias => meta_res.3.as_str(),
            ))?;

            // 为别名添加 scope_map
            if !meta_res.3.is_empty() {
                log!(
                    "Debug:Adding alias '{}' for '{scope_str}/{name_str}'",
                    meta_res.3
                );
                scope_map
                    .entry(meta_res.3.to_lowercase())
                    .or_insert_with(|| (Vec::new(), name_str.clone()))
                    .0
                    .push(scope_str.clone());
            }
        }
    }

    // 写索引
    let serialized_quick_map = postcard::to_stdvec(&QuickMaps {
        scope_map,
        full_map,
        url_template: content.url_template,
    })?;
    let quick_path = dir.join(MIRROR_FILE_QUICK_MAP);
    std::fs::write(&quick_path, serialized_quick_map).map_err(|e| {
        anyhow!(
            "Error:Failed to write quick map to {}:{e}",
            p2s!(quick_path)
        )
    })?;
    index_writer.commit()?;

    Ok(())
}

// 从索引中搜索内容
pub fn search_index_for_mirror(
    text: &str,
    dir: PathBuf,
    is_regex: bool,
) -> Result<Vec<SearchResult>> {
    let schema_fields = get_schema()?;

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
        let query = RegexQuery::from_pattern(text, schema_fields.name)
            .map_err(|e| anyhow!("Error:Invalid regex : {e}"))?;
        searcher.search(&query, &TopDocs::with_limit(10))?
    } else {
        let query_parser = QueryParser::for_index(
            &index,
            vec![
                schema_fields.name,
                schema_fields.tags,
                schema_fields.bin,
                schema_fields.alias,
            ],
        );
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
            name: read_field(schema_fields.name)?,
            scope: read_field(schema_fields.scope)?,
            version: read_field(schema_fields.version)?,
            description: read_field(schema_fields.description)?,
            from_mirror: None,
        })
    }

    Ok(arr)
}

// 读取快查索引
pub fn read_quick_maps(
    cfg: &crate::types::context::RuntimeContext,
    mirror_name: &str,
) -> Result<QuickMaps> {
    let quick_path = get_path_mirror(cfg)?
        .join(mirror_name)
        .join("index")
        .join(MIRROR_FILE_QUICK_MAP);
    if !quick_path.exists() {
        return Err(anyhow!("Error:Missing quick map at '{}'", p2s!(quick_path)));
    }
    let bin_data = std::fs::read(&quick_path).map_err(|e| {
        anyhow!(
            "Error:Failed to read quick map at '{}' : {e}",
            p2s!(quick_path)
        )
    })?;
    let quick_map: QuickMaps = postcard::from_bytes(&bin_data).map_err(|e| {
        anyhow!(
            "Error:Invalid quick map bin at '{}' : {e}",
            p2s!(quick_path)
        )
    })?;

    Ok(quick_map)
}

// 匹配 release
// 如果没有提供 semver matcher 则返回最大版本
pub fn filter_release(
    cfg: &crate::types::context::RuntimeContext,
    releases: Vec<MirrorPkgSoftwareRelease>,
    semver_matcher: Option<VersionReq>,
    enable_flags_score: bool,
) -> Result<MirrorPkgSoftwareRelease> {
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
                        get_flags_score(cfg, &flags)
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
                "Error:The latest release ('{}') is blocked due to configured preference policy or system architecture, try change your preference in config",
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
    cfg: &crate::types::context::RuntimeContext,
    matcher: PackageMatcher,
) -> Result<(String, MirrorPkgSoftwareRelease, String)> {
    // 查找 scope 并使用 scope 更新纠正大小写
    let (scope, package_name) = find_scope_with_name(cfg, &matcher.name, matcher.scope.as_deref())?;
    // 拿到 info online
    let (info, url_template, mirror_name) =
        info_online(cfg, &scope, &package_name, matcher.mirror)?;
    // 匹配版本
    let matched_release = filter_release(cfg, info.releases, matcher.version_req, true)?;
    // 填充模板获取 url
    let url = fill_url_template(
        &url_template,
        &scope,
        &info.name,
        &matched_release.file_name,
    )?;
    Ok((url, matched_release, mirror_name))
}

#[test]
fn test_filter_release() {
    use crate::types::extended_semver::ExSemVer;
    use crate::utils::test::_default_test_cfg;
    let cfg = _default_test_cfg();
    // 直接筛选最高版本
    let arr = vec![
        MirrorPkgSoftwareRelease {
            file_name: "VSCode_1.85.1.0_Cno.nep".to_string(),
            version: ExSemVer::parse("1.85.1.0").unwrap(),
            size: 94245376,
            timestamp: 1704554724,
            integrity: None,
            meta: None,
        },
        MirrorPkgSoftwareRelease {
            file_name: "VSCode_1.86.1.0_Cno.nep".to_string(),
            version: ExSemVer::parse("1.86.1.0").unwrap(),
            size: 94245376,
            timestamp: 1704554724,
            integrity: None,
            meta: None,
        },
        MirrorPkgSoftwareRelease {
            file_name: "VSCode_1.85.2.0_Cno.nep".to_string(),
            version: ExSemVer::parse("1.85.2.0").unwrap(),
            size: 94245376,
            timestamp: 1704554724,
            integrity: None,
            meta: None,
        },
    ];
    let res = filter_release(&cfg, arr, None, false).unwrap();
    assert_eq!(res.version.to_string(), "1.86.1.0".to_string());

    // 使用 matcher
    let arr = vec![
        MirrorPkgSoftwareRelease {
            file_name: "Chrome_120.0.6099.200_Cno.nep".to_string(),
            version: ExSemVer::parse("120.0.6099.200").unwrap(),
            size: 133763072,
            timestamp: 1704554608,
            integrity: None,
            meta: None,
        },
        MirrorPkgSoftwareRelease {
            file_name: "Chrome_121.0.6099.200_Cno.nep".to_string(),
            version: ExSemVer::parse("121.0.6099.200").unwrap(),
            size: 133763072,
            timestamp: 1704554608,
            integrity: None,
            meta: None,
        },
        MirrorPkgSoftwareRelease {
            file_name: "Chrome_122.0.6099.200_Cno.nep".to_string(),
            version: ExSemVer::parse("122.0.6099.200").unwrap(),
            size: 133763072,
            timestamp: 1704554608,
            integrity: None,
            meta: None,
        },
    ];
    let res = filter_release(&cfg, arr, Some(VersionReq::parse("121").unwrap()), false).unwrap();
    assert_eq!(res.version.to_string(), "121.0.6099.200".to_string());
}

#[test]
fn test_filter_release_with_flags() {
    use crate::utils::flags::{set_flag, Flag};
    use crate::utils::test::_default_test_cfg;
    set_flag(Flag::Debug, true);
    use crate::types::cfg::PreferenceEnum;
    use crate::types::extended_semver::ExSemVer;
    use std::str::FromStr;

    let releases = vec![
        MirrorPkgSoftwareRelease {
            file_name: "Firefox_127.0.0.1_Cno.I.nep".to_string(),
            version: ExSemVer::from_str("127.0.0.1").unwrap(),
            size: 94245376,
            timestamp: 1704554724,
            integrity: None,
            meta: None,
        },
        MirrorPkgSoftwareRelease {
            file_name: "Firefox_127.0.0.1_Cno.IE.nep".to_string(),
            version: ExSemVer::from_str("127.0.0.1").unwrap(),
            size: 94245376,
            timestamp: 1704554724,
            integrity: None,
            meta: None,
        },
        MirrorPkgSoftwareRelease {
            file_name: "Firefox_127.0.0.1_Cno.P.nep".to_string(),
            version: ExSemVer::from_str("127.0.0.1").unwrap(),
            size: 94245376,
            timestamp: 1704554724,
            integrity: None,
            meta: None,
        },
        MirrorPkgSoftwareRelease {
            file_name: "Firefox_127.0.0.1_Cno.PE.nep".to_string(),
            version: ExSemVer::from_str("127.0.0.1").unwrap(),
            size: 94245376,
            timestamp: 1704554724,
            integrity: None,
            meta: None,
        },
    ];

    let modifier = |i: PreferenceEnum, p: PreferenceEnum, e: PreferenceEnum| {
        let mut cfg = _default_test_cfg();
        cfg.cfg.preference.installer = i;
        cfg.cfg.preference.portable = p;
        cfg.cfg.preference.expandable = e;
        cfg
    };

    // 默认优先级配置，会匹配到 PE 版本
    let cfg = modifier(
        PreferenceEnum::LowPriority,
        PreferenceEnum::HighPriority,
        PreferenceEnum::HighPriority,
    );
    assert_eq!(
        filter_release(&cfg, releases.clone(), None, true)
            .unwrap()
            .file_name,
        "Firefox_127.0.0.1_Cno.PE.nep".to_string()
    );

    // 便携且不要可拓展模式，匹配到 P 版本
    let cfg = modifier(
        PreferenceEnum::Forbidden,
        PreferenceEnum::HighPriority,
        PreferenceEnum::Forbidden,
    );
    assert_eq!(
        filter_release(&cfg, releases.clone(), None, true)
            .unwrap()
            .file_name,
        "Firefox_127.0.0.1_Cno.P.nep".to_string()
    );

    // 全部禁用，会报错
    let cfg = modifier(
        PreferenceEnum::Forbidden,
        PreferenceEnum::Forbidden,
        PreferenceEnum::Forbidden,
    );
    assert!(filter_release(&cfg, releases.clone(), None, true).is_err());
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
//     let r = search_index_for_mirror("Code", p.clone()).unwrap();
//     println!("{r:#?}");
// }
