# wx-dl-ui

微信视频号下载器可视化桌面端，基于 Tauri 2 + Vue 3 封装 [ltaoo/wx_channels_download](https://github.com/ltaoo/wx_channels_download) 的命令行程序。

## 特性

- 可视化启动 / 停止 `wx_video_download` 核心服务，实时查看运行日志
- **核心程序外置**：不随软件打包，用户在设置中选择可执行文件路径即可，方便随时替换新版本
- 图形化配置下载目录、并发数、端口、系统代理等常用选项，保存后自动在核心程序所在目录生成 `config.yaml`
- 服务状态实时检测，一键打开管理页（`http://127.0.0.1:2022`）

## 使用说明

1. 到 [ltaoo/wx_channels_download Releases](https://github.com/ltaoo/wx_channels_download/releases) 下载对应平台的 `wx_video_download` 可执行文件，放在任意位置
2. 打开本软件，在「设置」页选择该可执行文件路径
3. 回到「控制台」点击「启动服务」，首次使用会自动安装证书（需要管理员权限）
4. 打开微信 PC 端进入视频号页面，即可使用页面内注入的下载按钮

> 注意：系统代理模式需要管理员权限；退出时请通过本软件的「停止服务」按钮停止，以便正确清理系统代理设置。

## 开发

```bash
npm install        # 本机 npm 配置了 dev=true 时请使用 npm install --include=dev
npm run tauri dev
```

## 打包

推送 `v*` 标签或手动触发 workflow_dispatch，GitHub Actions 会自动构建三端产物并发布 Release：

- macOS Apple Silicon（aarch64-apple-darwin）
- macOS Intel（x86_64-apple-darwin）
- Windows（x86_64）

```bash
git tag v0.1.0
git push origin v0.1.0
```
