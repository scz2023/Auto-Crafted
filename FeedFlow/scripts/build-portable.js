import { execSync } from 'child_process';
import { existsSync, mkdirSync, copyFileSync, readdirSync, statSync, writeFileSync, createWriteStream, cpSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import archiver from 'archiver';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const rootDir = join(__dirname, '..');
const tauriDir = join(rootDir, 'src-tauri');
const releaseDir = join(tauriDir, 'target', 'release');
const portableDir = join(rootDir, 'dist-portable');
const productName = 'FeedFlow';
const version = '1.0.0';

console.log('🚀 开始构建便携版...\n');

// 1. 清理旧的便携版目录
if (existsSync(portableDir)) {
  console.log('📁 清理旧的便携版目录...');
  execSync(`rmdir /s /q "${portableDir}"`, { shell: true, stdio: 'inherit' });
}

// 2. 构建前端
console.log('🔨 构建前端...');
try {
  execSync('npm run build', { 
    cwd: rootDir, 
    stdio: 'inherit',
    shell: true 
  });
} catch (error) {
  console.error('❌ 前端构建失败:', error.message);
  process.exit(1);
}

// 3. 构建 Tauri 应用（Release 模式，不打包成 MSI，只生成可执行文件）
console.log('🔨 构建 Tauri 应用（Release 模式）...');
try {
  execSync('cargo build --release', { 
    cwd: tauriDir, 
    stdio: 'inherit',
    shell: true 
  });
} catch (error) {
  console.log('⚠️  Tauri 构建过程有错误，但检查 release 版本是否已生成...');
  // 即使构建失败，release 版本可能已经生成，继续检查
}

// 4. 检查 release 目录
if (!existsSync(releaseDir)) {
  console.error('❌ Release 目录不存在:', releaseDir);
  process.exit(1);
}

// 5. 创建便携版目录
console.log('📦 创建便携版目录...');
mkdirSync(portableDir, { recursive: true });

// 6. 复制可执行文件
const exeName = `${productName.toLowerCase()}.exe`;
const exePath = join(releaseDir, exeName);
if (existsSync(exePath)) {
  console.log(`📋 复制可执行文件: ${exeName}`);
  copyFileSync(exePath, join(portableDir, exeName));
} else {
  console.error(`❌ 可执行文件不存在: ${exePath}`);
  process.exit(1);
}

// 7. 复制 DLL 文件和其他依赖
console.log('📋 复制依赖文件...');
const filesToCopy = readdirSync(releaseDir).filter(file => {
  const filePath = join(releaseDir, file);
  const stat = statSync(filePath);
  if (stat.isFile()) {
    const ext = file.split('.').pop()?.toLowerCase();
    return ext === 'dll' || ext === 'pdb' || file === 'WebView2Loader.dll';
  }
  return false;
});

filesToCopy.forEach(file => {
  const srcPath = join(releaseDir, file);
  const destPath = join(portableDir, file);
  copyFileSync(srcPath, destPath);
  console.log(`  ✓ ${file}`);
});

// 8. 复制服务器端文件（.output/server 目录）
const outputServerDir = join(rootDir, '.output', 'server');
const portableOutputDir = join(portableDir, '.output');
const portableServerDir = join(portableOutputDir, 'server');
if (existsSync(outputServerDir)) {
  console.log('📋 复制服务器端文件...');
  console.log(`  源目录: ${outputServerDir}`);
  console.log(`  目标目录: ${portableServerDir}`);
  
  // 确保 .output 目录存在
  if (!existsSync(portableOutputDir)) {
    mkdirSync(portableOutputDir, { recursive: true });
  }
  
  cpSync(outputServerDir, portableServerDir, { recursive: true });
  console.log('  ✓ 服务器端文件已复制');
  
  // 验证复制是否成功
  const serverIndex = join(portableServerDir, 'index.mjs');
  if (existsSync(serverIndex)) {
    console.log('  ✓ 服务器 index.mjs 已验证');
  } else {
    console.error('  ❌ 错误: 复制后未找到服务器 index.mjs!');
    process.exit(1);
  }
} else {
  console.error('❌ 错误: 服务器端目录不存在:', outputServerDir);
  console.error('请先运行 "npm run build" 生成服务器文件。');
  process.exit(1);
}

// 9. 检查并复制资源文件（如果有）
const resourcesDir = join(releaseDir, '_up_');
if (existsSync(resourcesDir)) {
  console.log('📋 复制资源文件...');
  const resourcesFiles = readdirSync(resourcesDir);
  resourcesFiles.forEach(file => {
    const srcPath = join(resourcesDir, file);
    const destPath = join(portableDir, file);
    if (statSync(srcPath).isFile()) {
      copyFileSync(srcPath, destPath);
      console.log(`  ✓ ${file}`);
    }
  });
}

// 10. 创建 README 文件
const readmeContent = `# ${productName} 便携版

版本: ${version}

## 使用说明

1. 解压此文件到任意目录
2. 运行 ${exeName} 即可使用
3. 无需安装，可直接运行

## 注意事项

- 首次运行可能需要一些时间加载
- 数据文件会保存在应用数据目录
- 可以随时删除整个文件夹，不会在系统中留下痕迹

## 系统要求

- Windows 10/11
- WebView2 Runtime（通常已预装，如未安装会自动下载）
- Node.js（服务器端 API 需要）
`;

const readmePath = join(portableDir, 'README.txt');
writeFileSync(readmePath, readmeContent, 'utf8');
console.log('📝 创建 README.txt');

// 11. 打包成 ZIP
console.log('\n📦 打包成 ZIP 文件...');
const zipFileName = `${productName}-${version}-portable.zip`;
const zipPath = join(rootDir, zipFileName);

const output = createWriteStream(zipPath);
const archive = archiver('zip', {
  zlib: { level: 9 }
});

output.on('close', () => {
  const sizeMB = (archive.pointer() / 1024 / 1024).toFixed(2);
  console.log(`✅ 便携版打包完成!`);
  console.log(`📦 文件: ${zipFileName}`);
  console.log(`📊 大小: ${sizeMB} MB`);
  console.log(`📁 位置: ${zipPath}`);
  console.log(`\n✨ 便携版已准备就绪！`);
});

archive.on('error', (err) => {
  console.error('❌ 打包失败:', err);
  process.exit(1);
});

archive.pipe(output);
archive.directory(portableDir, false);
archive.finalize();

