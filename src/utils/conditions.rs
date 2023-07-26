use anyhow::{anyhow, Result};
use evalexpr::*;
use regex::Regex;
use std::sync::{Arc, Mutex};

use crate::{
    executor::{
        condition_eval, get_eval_function_names, get_eval_function_permission,
        verify_eval_function_arg,
    },
    types::permissions::Permission,
};

lazy_static! {
    static ref RESOURCE_REGEX: Regex = Regex::new(r"^[^/]+/[^/]+$").unwrap();
}

pub fn ensure_arg(val: &Value) -> std::result::Result<String, error::EvalexprError> {
    if let Value::String(str) = val {
        Ok(str.to_string())
    } else {
        Err(error::EvalexprError::ExpectedString {
            actual: val.clone(),
        })
    }
}

/// 使用虚拟的函数定义捕获函数运行信息，返回（函数名，参数，所属表达式）
fn capture_function_info(conditions: &Vec<String>) -> Result<Vec<(String, String, String)>> {
    // 获取已注册的 eval 函数名称
    let info_arr = get_eval_function_names();

    // 迭代所有条件语句
    let res = Arc::new(Mutex::new(Vec::new()));
    for cond in conditions {
        // 初始化上下文
        let mut context = HashMapContext::new();

        // 迭代函数信息，创建收集闭包
        for name in info_arr.clone() {
            let r = res.clone();
            let c = cond.clone();
            context
                .set_function(
                    name.to_string(),
                    Function::new(move |val| {
                        let arg = ensure_arg(val)?;
                        let mut r = r.lock().unwrap();
                        r.push((name.to_string(), arg, c.to_owned()));

                        Ok(Value::Boolean(true))
                    }),
                )
                .unwrap();
        }

        // 执行
        eval_boolean_with_context(&cond, &context)
            .map_err(|e| anyhow!("Error:Failed to execute expression '{cond}' : {e}"))?;
    }

    let res = res.lock().unwrap();
    Ok(res.clone())
}

pub fn get_permissions_from_conditions(conditions: Vec<String>) -> Result<Vec<Permission>> {
    // 捕获函数执行信息
    let func_info = capture_function_info(&conditions)?;

    // 匹配生成权限信息
    let mut permissions = Vec::new();
    for (name, arg, _) in func_info {
        permissions.push(get_eval_function_permission(name, arg)?);
    }

    Ok(permissions)
}

pub fn verify_conditions(conditions: Vec<String>, located: &String) -> Result<()> {
    // 捕获函数执行信息
    let func_info = capture_function_info(&conditions)?;

    // 匹配函数入参进行校验
    for (name, arg, _) in func_info {
        verify_eval_function_arg(name, arg)?;
    }

    // 对条件进行 eval 校验
    for cond in conditions {
        condition_eval(&cond, 0, located)
            .map_err(|e| anyhow!("Error:Failed to validate condition : {e}"))?;
    }

    Ok(())
}

#[test]
fn test_condition() {
    let located = "examples/VSCode".to_string();

    let conditions: Vec<String> = vec![
        "\"${ExitCode}\"==114",
        "\"${SystemDrive}\"==\"C:\"",
        "\"${DefaultLocation}\"==\"./unknown/VSCode\"",
        "Exist(\"src/main.rs\") && IsDirectory(\"src\")",
        "Exist(\"${AppData}\") && IsDirectory(\"${SystemDrive}/Windows\")",
        "Exist(\"./src/main.ts\")",
        "(IsInstalled(\"Foo/Bar\") && IsAlive(\"陈睿's mother.exe\")) || IsAlive(\"aunt.exe\")",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect();

    // verify_conditions
    verify_conditions(conditions.clone(), &located).unwrap();

    // capture_function_info
    let res = capture_function_info(&conditions.clone()).unwrap();
    let answer: Vec<(String, String, String)> = vec![
        (
            "Exist",
            "src/main.rs",
            "Exist(\"src/main.rs\") && IsDirectory(\"src\")",
        ),
        (
            "IsDirectory",
            "src",
            "Exist(\"src/main.rs\") && IsDirectory(\"src\")",
        ),
        (
            "Exist",
            "${AppData}",
            "Exist(\"${AppData}\") && IsDirectory(\"${SystemDrive}/Windows\")",
        ),
        (
            "IsDirectory",
            "${SystemDrive}/Windows",
            "Exist(\"${AppData}\") && IsDirectory(\"${SystemDrive}/Windows\")",
        ),
        ("Exist", "./src/main.ts", "Exist(\"./src/main.ts\")"),
        (
            "IsInstalled",
            "Foo/Bar",
            "(IsInstalled(\"Foo/Bar\") && IsAlive(\"陈睿's mother.exe\")) || IsAlive(\"aunt.exe\")",
        ),
        (
            "IsAlive",
            "陈睿's mother.exe",
            "(IsInstalled(\"Foo/Bar\") && IsAlive(\"陈睿's mother.exe\")) || IsAlive(\"aunt.exe\")",
        ),
        (
            "IsAlive",
            "aunt.exe",
            "(IsInstalled(\"Foo/Bar\") && IsAlive(\"陈睿's mother.exe\")) || IsAlive(\"aunt.exe\")",
        ),
    ]
    .into_iter()
    .map(|(func, arg, source)| (func.to_string(), arg.to_string(), source.to_string()))
    .collect();
    assert_eq!(res, answer);

    // get_permissions_from_conditions
    use crate::types::permissions::PermissionLevel;
    let res = get_permissions_from_conditions(conditions.clone()).unwrap();
    let answer = vec![
        Permission {
            key: "fs_read".to_string(),
            level: PermissionLevel::Normal,
            targets: vec!["src/main.rs".to_string()],
        },
        Permission {
            key: "fs_read".to_string(),
            level: PermissionLevel::Normal,
            targets: vec!["src".to_string()],
        },
        Permission {
            key: "fs_read".to_string(),
            level: PermissionLevel::Sensitive,
            targets: vec!["${AppData}".to_string()],
        },
        Permission {
            key: "fs_read".to_string(),
            level: PermissionLevel::Sensitive,
            targets: vec!["${SystemDrive}/Windows".to_string()],
        },
        Permission {
            key: "fs_read".to_string(),
            level: PermissionLevel::Normal,
            targets: vec!["./src/main.ts".to_string()],
        },
        Permission {
            key: "nep_installed".to_string(),
            level: PermissionLevel::Normal,
            targets: vec!["Foo/Bar".to_string()],
        },
        Permission {
            key: "process_query".to_string(),
            level: PermissionLevel::Normal,
            targets: vec!["陈睿's mother.exe".to_string()],
        },
        Permission {
            key: "process_query".to_string(),
            level: PermissionLevel::Normal,
            targets: vec!["aunt.exe".to_string()],
        },
    ];
    assert_eq!(res, answer);

    // println!("{res:#?}");
}
