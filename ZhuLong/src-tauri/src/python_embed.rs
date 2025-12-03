use pyo3::prelude::*;
use pyo3::types::PyList;
use std::ffi::CString;
use std::path::PathBuf;

/// 初始化嵌入的 Python 解释器
pub fn init_python() -> PyResult<()> {
    Python::with_gil(|py| {
        // 确保 Python 已初始化
        let code = CString::new("import sys").unwrap();
        py.run(code.as_c_str(), None, None)?;
        Ok(())
    })
}

/// 使用嵌入的 Python 解释器运行 Strix
pub async fn run_strix_with_embedded_python(
    strix_path: PathBuf,
    strix_dir: PathBuf,
    args: Vec<String>,
    env_vars: std::collections::HashMap<String, String>,
) -> Result<(String, String, i32), String> {
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
            
            // 将 Strix 目录添加到 Python 路径
            let path = sys.getattr("path")?;
            let path_list = path.downcast::<PyList>()?;
            let strix_dir_str = strix_dir.to_string_lossy().to_string();
            path_list.insert(0, strix_dir_str)?;
            
            // 构建命令行参数 - 直接在 Python 代码中设置
            // 注意：我们将在下面的 Python 代码中设置 sys.argv
            
            // 执行 Python 代码
            // 直接执行 Strix 的 main.py 文件，模拟命令行执行
            let workspace_dir = strix_dir.parent()
                .unwrap_or(&strix_dir)
                .to_string_lossy()
                .replace('\\', "/");
            let strix_path_str = strix_path.to_string_lossy().replace('\\', "/");
            let strix_root = strix_dir.to_string_lossy().replace('\\', "/");
            
            // 构建命令行参数字符串（转义单引号）
            let args_python = args.iter()
                .map(|arg| format!("r'{}'", arg.replace('\\', r"\\").replace('\'', r"\'")))
                .collect::<Vec<_>>()
                .join(", ");
            
            let code = format!(
                r#"
import sys
import os
import asyncio

# 设置工作目录
os.chdir(r"{}")

# 将 Strix 目录添加到路径（确保可以导入 strix 模块）
strix_root = r"{}"
if strix_root not in sys.path:
    sys.path.insert(0, strix_root)

# 设置命令行参数
sys.argv = [{}]

# 直接执行文件（这样会触发 if __name__ == "__main__" 的逻辑）
with open(r"{}", 'r', encoding='utf-8') as f:
    code = compile(f.read(), r"{}", 'exec')
    exec(code, {{'__name__': '__main__', '__file__': r"{}"}})
"#,
                workspace_dir,
                strix_root,
                args_python,
                strix_path_str,
                strix_path_str,
                strix_path_str
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
                    // 获取错误信息
                    let error_msg = format!("{}", e);
                    Ok((stdout_str, format!("{}\n{}", stderr_str, error_msg), 1))
                }
            }
        })
        .map_err(|e| format!("Python 执行错误: {}", e))
    })
    .await
    .map_err(|e| format!("任务执行错误: {}", e))?;
    
    result
}

