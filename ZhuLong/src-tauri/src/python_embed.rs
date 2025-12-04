use pyo3::prelude::*;
use pyo3::types::PyList;
use std::ffi::CString;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// 去掉 Windows 长路径前缀 \\?\（Python 可能无法正确处理）
/// 已废弃：使用 Docker 方式，不再需要
#[allow(dead_code)]
fn remove_verbatim_prefix(path: &PathBuf) -> String {
    let path_str = path.to_string_lossy().to_string();
    if path_str.starts_with(r"\\?\") {
        path_str[4..].to_string()
    } else {
        path_str
    }
}

/// 查找 Strix 路径（与 strix.rs 中的逻辑相同）
/// 已废弃：使用 Docker 方式，不再需要
#[allow(dead_code)]
fn find_strix_path(app: &AppHandle) -> Result<PathBuf, String> {
    // 尝试从资源目录获取（打包后）
    if let Ok(resource_dir) = app.path().resource_dir() {
        let strix_main = resource_dir
            .join("strix-0.4.0")
            .join("strix")
            .join("interface")
            .join("main.py");
        
        if strix_main.exists() {
            return Ok(strix_main);
        }
    }
    
    // 尝试从当前目录获取（开发环境）
    if let Ok(current_dir) = std::env::current_dir() {
        // 优先：如果当前目录是 src-tauri，直接查找 strix-0.4.0
        if current_dir.ends_with("src-tauri") {
            let strix_main = current_dir
                .join("strix-0.4.0")
                .join("strix")
                .join("interface")
                .join("main.py");
            
            if strix_main.exists() {
                return Ok(strix_main);
            }
        }
        
        // 检查当前目录（可能是项目根目录）
        let strix_main = current_dir
            .join("strix-0.4.0")
            .join("strix")
            .join("interface")
            .join("main.py");
        
        if strix_main.exists() {
            return Ok(strix_main);
        }
        
        // 如果当前目录是 src-tauri，尝试父目录（向后兼容）
        if current_dir.ends_with("src-tauri") {
            if let Some(parent) = current_dir.parent() {
                let strix_main = parent
                    .join("strix-0.4.0")
                    .join("strix")
                    .join("interface")
                    .join("main.py");
                
                if strix_main.exists() {
                    return Ok(strix_main);
                }
            }
        }
        
        // 尝试向上查找项目根目录（查找包含 strix-0.4.0 的目录）
        let mut search_dir = current_dir.clone();
        for _ in 0..5 {
            // 检查当前层级
            let strix_main = search_dir
                .join("strix-0.4.0")
                .join("strix")
                .join("interface")
                .join("main.py");
            
            if strix_main.exists() {
                return Ok(strix_main);
            }
            
            // 检查 src-tauri 子目录
            let strix_main = search_dir
                .join("src-tauri")
                .join("strix-0.4.0")
                .join("strix")
                .join("interface")
                .join("main.py");
            
            if strix_main.exists() {
                return Ok(strix_main);
            }
            
            if let Some(parent) = search_dir.parent() {
                search_dir = parent.to_path_buf();
            } else {
                break;
            }
        }
    }
    
    Err("Strix 主程序未找到，请确保 strix-0.4.0 目录存在于 src-tauri 或项目根目录".to_string())
}

/// 初始化嵌入的 Python 解释器
/// 已废弃：使用 Docker 方式，不再需要
#[allow(dead_code)]
pub fn init_python() -> PyResult<()> {
    Python::with_gil(|py| {
        // 确保 Python 已初始化
        let code = CString::new("import sys").unwrap();
        py.run(code.as_c_str(), None, None)?;
        Ok(())
    })
}

/// 使用嵌入的 Python 解释器运行 Strix
/// 已废弃：使用 Docker 方式，不再需要
#[allow(dead_code)]
pub async fn run_strix_with_embedded_python(
    strix_path: PathBuf,
    strix_dir: PathBuf,
    workspace_dir: PathBuf,
    args: Vec<String>,
    env_vars: std::collections::HashMap<String, String>,
) -> Result<(String, String, i32), String> {
    // 构建完整的执行命令用于错误信息
    let args_clone = args.clone();
    let full_command = format!(
        "python {} {}",
        remove_verbatim_prefix(&strix_path),
        args_clone.join(" ")
    );
    
    // 验证路径
    if !strix_path.exists() {
        return Err(format!("Strix 主程序不存在: {}", strix_path.display()));
    }
    
    if !strix_dir.exists() {
        return Err(format!("Strix 目录不存在: {}", strix_dir.display()));
    }
    
    // 确保工作目录存在
    std::fs::create_dir_all(&workspace_dir)
        .map_err(|e| format!("创建工作目录失败 (路径: {}): {}", workspace_dir.display(), e))?;
    
    // 验证工作目录是否可访问
    std::fs::metadata(&workspace_dir)
        .map_err(|e| format!("无法访问工作目录 (路径: {}): {}", workspace_dir.display(), e))?;
    
    // 在 Tokio 运行时中运行 Python 代码
    let result = tokio::task::spawn_blocking(move || {
        Python::with_gil(|py| -> PyResult<(String, String, i32)> {
            // 设置环境变量
            let sys = py.import("sys")?;
            let os = py.import("os")?;
            
            // 设置环境变量
            let env_dict = os.getattr("environ")?;
            for (key, value) in env_vars {
                env_dict.call_method1("__setitem__", (key, value))?;
            }
            
            // 将 Strix 根目录添加到 Python 路径（strix_dir 是 strix 目录的父目录）
            let path = sys.getattr("path")?;
            let path_list = path.downcast::<PyList>()?;
            // strix_dir 应该是 strix-0.4.0 目录，需要添加 strix 子目录到路径
            let strix_package_dir = strix_dir.join("strix");
            let strix_package_str = strix_package_dir.to_string_lossy().replace('\\', "/");
            if !path_list.contains(&strix_package_str)? {
                path_list.insert(0, strix_package_str.clone())?;
            }
            
            // 转换路径为字符串（使用正斜杠，Python 兼容）
            // 使用绝对路径避免相对路径问题，但要去掉 Windows 的长路径前缀 \\?\
            let workspace_dir_abs = workspace_dir.canonicalize()
                .unwrap_or(workspace_dir.clone());
            let strix_path_abs = strix_path.canonicalize()
                .unwrap_or(strix_path.clone());
            let strix_dir_abs = strix_dir.canonicalize()
                .unwrap_or(strix_dir.clone());
            
            let workspace_dir_str = remove_verbatim_prefix(&workspace_dir_abs).replace('\\', "/");
            let strix_path_str = remove_verbatim_prefix(&strix_path_abs).replace('\\', "/");
            let strix_root = remove_verbatim_prefix(&strix_dir_abs).replace('\\', "/");
            
            // 构建命令行参数字符串（转义单引号）
            let args_python = args.iter()
                .map(|arg| {
                    // 转义单引号和反斜杠
                    let escaped = arg.replace('\\', r"\\").replace('\'', r"\'");
                    format!("r'{}'", escaped)
                })
                .collect::<Vec<_>>()
                .join(", ");
            
            let code = format!(
                r#"
import sys
import os
import asyncio
import traceback

try:
    print("[DEBUG] ========== Strix 执行开始 ==========")
    
    # 规范化路径（Windows 需要反斜杠）
    workspace_raw = r"{}"
    workspace = os.path.normpath(workspace_raw)
    print(f"[DEBUG] 工作目录（原始）: {{workspace_raw}}")
    print(f"[DEBUG] 工作目录（规范化）: {{workspace}}")
    
    # 设置工作目录（确保目录存在）
    if not os.path.exists(workspace):
        os.makedirs(workspace, exist_ok=True)
    if not os.path.isdir(workspace):
        raise OSError(f"工作目录不是有效的目录: {{workspace}}")
    
    # 使用绝对路径并规范化
    workspace_abs = os.path.abspath(workspace)
    print(f"[DEBUG] 工作目录（绝对路径）: {{workspace_abs}}")
    
    try:
        os.chdir(workspace_abs)
        print(f"[DEBUG] 成功切换到工作目录: {{os.getcwd()}}")
    except OSError as e:
        raise OSError(f"无法切换到工作目录 {{workspace_abs}}: {{e}}")

    # 直接执行文件路径
    strix_main_raw = r"{}"
    strix_main = os.path.normpath(strix_main_raw)
    strix_main_abs = os.path.abspath(strix_main)
    print(f"[DEBUG] Strix 主程序（原始）: {{strix_main_raw}}")
    print(f"[DEBUG] Strix 主程序（规范化）: {{strix_main}}")
    print(f"[DEBUG] Strix 主程序（绝对路径）: {{strix_main_abs}}")
    
    if not os.path.exists(strix_main_abs):
        raise FileNotFoundError(f"Strix 主程序未找到: {{strix_main_abs}}")
    if not os.path.isfile(strix_main_abs):
        raise OSError(f"Strix 主程序路径不是文件: {{strix_main_abs}}")

    # 将 Strix 目录添加到路径（确保可以导入 strix 模块）
    strix_root_raw = r"{}"
    strix_root = os.path.normpath(strix_root_raw)
    strix_package = os.path.join(strix_root, "strix")
    
    # 使用绝对路径
    strix_root_abs = os.path.abspath(strix_root)
    strix_package_abs = os.path.abspath(strix_package) if os.path.exists(strix_package) else None
    
    print(f"[DEBUG] Strix 根目录: {{strix_root_abs}}")
    if strix_package_abs:
        print(f"[DEBUG] Strix 包目录: {{strix_package_abs}}")
    
    if strix_package_abs and os.path.exists(strix_package_abs) and strix_package_abs not in sys.path:
        sys.path.insert(0, strix_package_abs)
        print(f"[DEBUG] 已添加 Strix 包路径到 sys.path: {{strix_package_abs}}")
    if strix_root_abs not in sys.path:
        sys.path.insert(0, strix_root_abs)
        print(f"[DEBUG] 已添加 Strix 根路径到 sys.path: {{strix_root_abs}}")

    # 设置命令行参数
    # sys.argv[0] 应该是脚本文件名，后续是真正的命令行参数
    script_name = os.path.basename(strix_main_abs)
    cmd_args = [{}]
    sys.argv = [script_name] + cmd_args
    print(f"[DEBUG] 脚本名称: {{script_name}}")
    print(f"[DEBUG] 完整命令行参数: {{sys.argv}}")
    cmd_str = ' '.join(cmd_args)
    print(f"[DEBUG] 等效命令: python {{strix_main_abs}} {{cmd_str}}")
    
    print(f"[DEBUG] 开始执行 Strix 主程序...")
    
    # 直接执行文件（这样会触发 if __name__ == "__main__" 的逻辑）
    with open(strix_main_abs, 'r', encoding='utf-8') as f:
        file_content = f.read()
        print(f"[DEBUG] 已读取文件内容，长度: {{len(file_content)}} 字符")
        code_obj = compile(file_content, strix_main_abs, 'exec')
        print(f"[DEBUG] 已编译代码对象")
        exec(code_obj, {{'__name__': '__main__', '__file__': strix_main_abs}})
        print(f"[DEBUG] 代码执行完成")
        
except Exception as e:
    error_type = type(e).__name__
    error_msg = str(e)
    error_trace = traceback.format_exc()
    print(f"[ERROR] {{error_type}}: {{error_msg}}", file=sys.stderr)
    print(error_trace, file=sys.stderr)
    raise
"#,
                workspace_dir_str,
                strix_path_str,
                strix_root,
                args_python
            );
            
            // 捕获 stdout 和 stderr
            let io = py.import("io")?;
            let stdout_capture = io.call_method1("StringIO", ())?;
            let stderr_capture = io.call_method1("StringIO", ())?;
            
            // 重定向输出（使用引用，避免移动）
            sys.setattr("stdout", &stdout_capture)?;
            sys.setattr("stderr", &stderr_capture)?;
            
            // 执行代码
            let code_cstr = CString::new(code).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("代码字符串包含空字符: {}", e)
            ))?;
            let result = py.run(code_cstr.as_c_str(), None, None);
            
            // 获取输出（无论成功或失败）
            let stdout_str = stdout_capture.call_method0("getvalue")?
                .extract::<String>()?;
            let stderr_str = stderr_capture.call_method0("getvalue")?
                .extract::<String>()?;
            
            match result {
                Ok(_) => {
                    Ok((stdout_str, stderr_str, 0))
                }
                Err(e) => {
                    // 获取详细的错误信息（Python 代码已经在 stderr 中输出了完整的 traceback）
                    let error_msg = format!("{}", e);
                    // 如果 stderr 为空，使用错误对象的信息
                    let full_error = if stderr_str.trim().is_empty() {
                        format!("Python 执行错误: {}", error_msg)
                    } else {
                        format!("{}\nPython 错误: {}", stderr_str, error_msg)
                    };
                    Ok((stdout_str, full_error, 1))
                }
            }
        })
        .map_err(move |e| {
            format!(
                "Python 执行错误: {}\n\n完整执行命令:\n  {}\n\n路径信息:\n  - Strix 路径: {}\n  - Strix 目录: {}\n  - 工作目录: {}\n\n命令行参数:\n  {}",
                e,
                full_command,
                remove_verbatim_prefix(&strix_path),
                remove_verbatim_prefix(&strix_dir),
                remove_verbatim_prefix(&workspace_dir),
                args_clone.join(" ")
            )
        })
    })
    .await
    .map_err(|e| format!("任务执行错误: {}", e))?;
    
    result
}

/// 测试 LLM 连接（使用系统 Python，因为需要安装的依赖）
/// 已废弃：使用 Docker 方式，不再需要
#[allow(dead_code)]
pub async fn test_llm_connection(
    app: AppHandle,
    llm_provider: String,
    llm_api_key: String,
    llm_api_base: Option<String>,
) -> Result<String, String> {
    use tokio::process::Command;
    
    // 获取 Strix 路径
    let strix_path = find_strix_path(&app)?;
    let strix_dir = strix_path.parent()
        .ok_or("无法获取 Strix 目录")?
        .parent()
        .ok_or("无法获取 Strix 根目录")?
        .to_path_buf();
    
    // 查找系统 Python
    let python_cmd = if Command::new("python")
        .arg("--version")
        .output()
        .await
        .is_ok()
    {
        "python"
    } else if Command::new("python3")
        .arg("--version")
        .output()
        .await
        .is_ok()
    {
        "python3"
    } else {
        return Err("未找到系统 Python。请确保已安装 Python 3.12+，并且 Strix 的依赖已安装（运行: cd strix-0.4.0 && poetry install）".to_string());
    };
    
    // 构建测试脚本
    let strix_dir_str = remove_verbatim_prefix(&strix_dir).replace('\\', "/");
    let strix_package_str = remove_verbatim_prefix(&strix_dir.join("strix")).replace('\\', "/");
    
    let api_base_str = llm_api_base.as_ref()
        .map(|s| format!("r'{}'", s.replace('\\', r"\\").replace('\'', r"\'")))
        .unwrap_or_else(|| "None".to_string());
    
    let test_script = format!(
        r#"
import sys
import os
import traceback

# 添加 Strix 路径
strix_root = r"{}"
strix_package = r"{}"
if strix_package not in sys.path:
    sys.path.insert(0, strix_package)
if strix_root not in sys.path:
    sys.path.insert(0, strix_root)

try:
    import litellm
    from strix.interface.utils import validate_llm_response
    
    # 设置环境变量
    os.environ["STRIX_LLM"] = r"{}"
    os.environ["LLM_API_KEY"] = r"{}"
    if {}:
        os.environ["LLM_API_BASE"] = {}
    
    # 获取配置
    model_name = os.getenv("STRIX_LLM", "openai/gpt-5")
    api_key = os.getenv("LLM_API_KEY")
    api_base = (
        os.getenv("LLM_API_BASE")
        or os.getenv("OPENAI_API_BASE")
        or os.getenv("LITELLM_BASE_URL")
        or os.getenv("OLLAMA_API_BASE")
    )
    
    if not api_key:
        raise ValueError("LLM_API_KEY 未设置")
    
    # 配置 litellm
    litellm.api_key = api_key
    if api_base:
        litellm.api_base = api_base
    
    # 测试消息
    test_messages = [
        {{"role": "system", "content": "You are a helpful assistant."}},
        {{"role": "user", "content": "Reply with just 'OK'."}},
    ]
    
    llm_timeout = int(os.getenv("LLM_TIMEOUT", "60"))
    
    print(f"[TEST] 正在测试 LLM 连接...")
    print(f"[TEST] 模型: {{model_name}}")
    print(f"[TEST] API Base: {{api_base if api_base else '默认'}}")
    
    # 执行测试请求
    response = litellm.completion(
        model=model_name,
        messages=test_messages,
        timeout=llm_timeout,
    )
    
    # 验证响应
    validate_llm_response(response)
    
    # 获取响应内容
    response_content = response.choices[0].message.content
    print(f"[TEST] ✅ LLM 连接成功！")
    print(f"[TEST] 响应: {{response_content}}")
    
    result_msg = f"✅ LLM 连接成功！\\n\\n模型: {{model_name}}\\nAPI Base: {{api_base if api_base else '默认'}}\\n响应: {{response_content}}"
    print(result_msg)
    
except ImportError as e:
    error_msg = f"导入模块失败: {{e}}\\n\\n请确保已安装 Strix 的依赖：\\n  cd strix-0.4.0\\n  poetry install\\n\\n或者使用 pip 安装：\\n  pip install litellm[proxy]"
    print(f"[TEST] ❌ {{error_msg}}", file=sys.stderr)
    sys.exit(1)
except Exception as e:
    error_type = type(e).__name__
    error_msg = str(e)
    error_trace = traceback.format_exc()
    print(f"[TEST] ❌ LLM 连接失败: {{error_type}}: {{error_msg}}", file=sys.stderr)
    print(error_trace, file=sys.stderr)
    sys.exit(1)
"#,
        strix_dir_str,
        strix_package_str,
        llm_provider.replace('\\', r"\\").replace('\'', r"\'"),
        llm_api_key.replace('\\', r"\\").replace('\'', r"\'"),
        if llm_api_base.is_some() { "True" } else { "False" },
        api_base_str
    );
    
    // 执行 Python 脚本
    let output = Command::new(python_cmd)
        .arg("-c")
        .arg(&test_script)
        .output()
        .await
        .map_err(|e| format!("执行 Python 失败: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if output.status.success() {
        if stdout.trim().is_empty() {
            Ok("✅ LLM 连接测试成功，但未收到输出".to_string())
        } else {
            Ok(stdout.trim().to_string())
        }
    } else {
        let error_msg = if stderr.trim().is_empty() {
            format!("LLM 连接测试失败: {}", stdout.trim())
        } else {
            format!("{}\n{}", stderr.trim(), stdout.trim())
        };
        Err(error_msg)
    }
}

