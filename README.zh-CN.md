# CodexMeter 中文说明

[![CI](https://github.com/SundeqiX1/CodexMeter/actions/workflows/ci.yml/badge.svg)](https://github.com/SundeqiX1/CodexMeter/actions/workflows/ci.yml)

CodexMeter 是面向 macOS 与 Windows 的轻量 Codex 额度状态工具。它采用 Tauri 2、Rust 和 React，主要驻留在菜单栏或系统托盘，并提供可选悬浮 Widget。

> This is an independent community project and is not affiliated with or endorsed by OpenAI.

> **版本状态：** `0.1.0` 是预览版。额度解析、断线重试、前端构建、Rust 测试以及 macOS Apple Silicon 原生打包已经验证；Windows 与 macOS Intel 安装包由 GitHub Actions 生成，完成对应真机安装冒烟测试后再标记为稳定版。Codex App Server 目前仍是上游实验性接口，未来 Codex 版本可能调整协议。

安装包已集中放在 [GitHub Releases 页面](https://github.com/SundeqiX1/CodexMeter/releases)，不再需要到 Actions 深层目录查找。当前提供的是未签名测试包，会明确标记为 Preview，并非稳定正式版；请按处理器架构选择对应文件，并阅读下方系统安全提示。

## 普通用户需要安装什么

- macOS 13+，或 Windows 10/11 x64；
- 本机已有登录状态、且能提供 App Server 的 Codex 可执行程序；
- Windows 需要 WebView2，Windows 10/11 通常已经自带。

下载安装包的普通用户**不需要**安装 Node.js、Rust、Visual Studio 或 Microsoft C++ Build Tools；这些仅供从源码构建的开发者使用。

macOS 会优先查找 ChatGPT/Codex 桌面应用内部自带的 `codex`，所以只安装桌面应用也可能直接使用。Windows 预览版目前需要在 `PATH` 中找到 `codex.exe`、`codex.cmd` 或 `codex.bat`，也可以在设置里手动选择完整路径。仅安装 Windows 桌面应用是否足够，取决于它是否提供可被选择、且支持 App Server 的 Codex 可执行程序。

## 使用方式

启动后，macOS 菜单栏会显示 5 小时与周额度；Windows 用户可以左键单击托盘图标打开面板。面板提供手动刷新、悬浮 Widget、开机启动和设置入口。关闭面板或 Widget 不会退出后台托盘程序。

悬浮 Widget 默认显示 `5h 76%   W 43%`。点击一次展开重置时间，再点击一次立即收起；点击悬浮窗不会打开主面板或设置。它始终置顶、不会获取键盘焦点，并会记住最后位置。开启“隐藏服务器未返回的额度窗口”后，菜单栏、主面板和悬浮窗会使用相同的隐藏规则；只显示一个额度窗口时，悬浮窗宽度会自动缩小。

设置中的语言可选择“跟随系统”、English 或“简体中文”。保存后，主面板、悬浮窗、状态文字、日期和系统托盘菜单会一起切换。

### macOS 安装与运行

1. Apple Silicon Mac 使用 `arm64` DMG，Intel Mac 使用 `x64` DMG。打开 DMG 后把 CodexMeter 拖入“应用程序”；开发包也可以直接运行 `CodexMeter.app`。
2. 首次打开未签名的预览包如果被系统拦截，请按住 Control 单击应用，选择“打开”，再确认“打开”。正式公开版应进行 Developer ID 签名和公证。
3. 点击菜单栏中的 `5h … · W …` 即可打开或隐藏额度面板。它是菜单栏应用，不占用 Dock。
4. 在主面板点 **Floating Widget**，或从菜单栏右键菜单选择 **Floating Widget**；也可以进入 Settings，勾选 **Show floating widget** 后保存。点击悬浮窗一次展开重置详情，再点击一次收起；设置只从主面板或菜单栏菜单进入。首次显示在屏幕中央，拖动后会记住位置。

额度面板右上角的 `×` 只隐藏面板；悬浮窗的 `×` 只隐藏悬浮窗；二者都不会退出后台程序。需要完全退出时，请选择 **Quit CodexMeter**。

### Windows 10/11 x64 安装与运行

1. 运行 `CodexMeter-Windows-Setup-<版本>-x64.exe`，或解压 portable ZIP 后运行 `CodexMeter.exe`。
2. 确保原生 Windows `codex` 命令已经登录并位于 `PATH`。如果安装位置不在 `PATH`，请在 Settings 填写 `codex.exe`、`codex.cmd` 或 `codex.bat` 的完整路径。
3. CodexMeter 会驻留在通知区域；Windows 可能把它收进“隐藏的图标”。左键托盘图标打开或隐藏面板；按住面板顶部的短横把手或空白区域即可拖动。右键托盘图标可选择 **Show Widget**、**Refresh**、**Launch at Startup**、**Settings** 和 **Quit CodexMeter**。

Codex 官方安装和登录方法见[官方 Codex CLI 文档](https://developers.openai.com/codex/cli)，本项目使用的本地 JSONL 协议及 `account/rateLimits/read` 见[官方 Codex App Server 文档](https://developers.openai.com/codex/app-server)。当前 Mac 可以直接产出 macOS 包；Windows 安装包需要在 Windows 上构建，或在项目推送到 GitHub 后由 Release 工作流构建。

### 开发模式运行

macOS 或 Windows 安装 Node.js 20+、Rust stable 和对应平台依赖后执行：

```bash
cd apps/desktop-tauri
npm ci
npm run tauri dev
```

Windows 还需要 Microsoft C++ Build Tools 与 Edge WebView2，详见 [Tauri 官方前置依赖](https://v2.tauri.app/start/prerequisites/)。请不要只运行 `npm run dev`：那只会启动网页前端，不包含 Rust 后端、系统托盘、App Server 连接和原生悬浮窗。

## 额度解析规则

应用通过 `codex app-server --stdio` 调用 `account/rateLimits/read`。解析顺序为：

1. 优先读取 `rateLimitsByLimitId.codex`。
2. 不存在时兼容旧版 `rateLimits`。
3. 遍历 `primary` 与 `secondary`，根据 `windowDurationMins` 匹配窗口。
4. `300` 分钟显示为 5 Hour，`10080` 分钟显示为 Weekly。
5. 未返回的窗口显示 `--` 或按用户设置隐藏，不进行估算。

显示百分比为服务端 `usedPercent` 的补数，并限制在 0–100 范围内。

### Plus 与 Pro 兼容性

解析逻辑不依赖套餐名称，因此 Plus、Pro 5x、Pro 20x、Business 以及未来套餐使用同一套规则。只要 `rateLimitsByLimitId.codex` 或旧版 `rateLimits` 返回了窗口，就可以显示。

如果 App Server 返回 `planType`，主面板标题旁会显示 **Plus**、**Pro 5×** 或 **Pro 20×** 等订阅徽标。套餐只按服务端明确字段展示并保留在内存中，不会根据额度百分比推测；未来出现未知值时会显示整理后的原始名称，而不会误判成现有套餐。

如果 Pro 账户当前只返回 Weekly：

- 默认显示 `5h -- · W 43%`；
- 开启“隐藏服务器未返回的额度窗口”后，只显示 `W 43%`；
- 不会根据 5x、20x 或 Weekly 数据反推一个假的 5 小时额度。

OpenAI 当前官方套餐页说明 Pro 提供 Plus 的 5 倍或 20 倍 Codex 用量，也说明可能存在 Weekly 限额；但每个账户实际返回哪些窗口仍由服务端决定，参见 [OpenAI Codex 套餐与用量说明](https://learn.chatgpt.com/docs/pricing)。

## 断线行为

CodexMeter 启动后立即读取一次，随后按设置每 30 或 60 秒刷新。App Server 退出、查询超时或临时不可用时，应用不会闪退；最近一次有效快照继续留在内存并标为 `Disconnected · Stale`，后台刷新周期会继续尝试恢复。也可以手动点击 Refresh 或 Reconnect。

## 本地配置

配置文件只允许保存：

- 悬浮 Widget 的位置与显示状态；
- 界面语言设置；
- 菜单栏紧凑模式与缺失窗口隐藏选项；
- 30/60 秒刷新间隔；
- 自定义 Codex 可执行文件路径。

系统开机启动由 Tauri 官方 autostart 插件交给操作系统管理。额度、Token、账户标识、App Server 原始响应和聊天内容不会写入配置文件。

## 开发与发布

```bash
cd apps/desktop-tauri
npm ci
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --locked --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

如果只想得到容易找到的安装包，请在 `apps/desktop-tauri` 中运行：

```bash
# macOS
env APPLE_SIGNING_IDENTITY=- npm run package:mac

# Windows PowerShell
npm run package:windows
```

完成后直接到项目根目录的 [`artifacts`](artifacts) 文件夹获取 App、DMG 或 EXE，以及 `SHA256SUMS.txt`，不需要再进入很深的 Tauri `target` 目录。

GitHub Actions 在 macOS 和 Windows 上运行检查。带 `v*` 的 tag 会触发发布构建：Windows x64 生成安装 EXE 与便携 ZIP；macOS 分别生成 arm64、x64 的 DMG 与 ZIP；所有平台同时生成 SHA-256 校验文件。

正式公开发布前，请配置 Apple Developer ID、公证凭据与 Windows 代码签名。没有签名凭据时，CI 生成的包只适合作为开发预览。

## 开源与归属

本项目是 [codex-quota-tool](https://github.com/changzhengithub/codex-quota-tool) 的 MIT 衍生项目，保留原项目许可证与 copyright。实现设计也参考了 [codex-quota-overlay](https://github.com/cpys/codex-quota-overlay) 的发布验证经验。

参见 [隐私政策](PRIVACY.md)、[安全政策](SECURITY.md) 与 [贡献指南](CONTRIBUTING.md)。
