# paste

[English](./README.md)

一个基于 `Tauri 2 + Vue 3 + TypeScript + Rust` 构建的、`Windows 优先` 的本地剪贴板管理器。

## 项目简介

`paste` 是一个参考 Paste 使用方式的桌面剪贴板工具，目前聚焦两个入口：

- 管理页面：用于浏览、搜索、置顶、删除和控制剪贴板历史
- 底部快速列表：用于高频查看和快速回填最近的剪贴板内容

应用常驻托盘运行，数据完全本地存储，当前优先面向 Windows 使用场景。

## 截图

### 管理页面

![管理页面](./screenshot/dashboard.png)

### 底部快速列表

![底部快速列表](./screenshot/pointer.png)

## 当前已实现功能

- 托盘常驻运行
- 两个全局快捷键
- `Ctrl+Shift+V` 打开管理页面
- `Alt+V` 打开底部快速列表
- 监听系统剪贴板中的 `文本` 和 `图片`
- 使用 `SQLite` 做本地持久化
- 图片原图保存到应用数据目录
- 去重规则
- 文本按规范化内容去重
- 图片按内容哈希去重
- 支持置顶 / 取消置顶
- 支持删除单条记录
- 支持清空全部历史
- 支持暂停剪贴板监听
- 管理页面支持搜索历史记录
- 底部快速列表支持双击复制
- 托盘菜单支持打开管理页、打开快速列表、暂停/恢复监听、清空历史、退出

## 当前行为

- 应用启动后默认隐藏并驻留托盘
- 关闭管理页面或快速列表时只会隐藏窗口，不会退出进程
- 置顶项始终排在顶部
- 非置顶历史最多保留 `500` 条
- 图片内容以文件形式存储，元数据进入 SQLite
- 底部快速列表固定出现在屏幕底部中央附近，并会根据可见条目数量调整宽度

## 技术栈

- 前端：`Vue 3`、`TypeScript`、`Vite`
- 桌面容器：`Tauri 2`
- 后端：`Rust`
- 数据存储：基于 `sqlx` 的 `SQLite`

## 项目结构

```text
src/                  Vue 前端
src/App.vue           管理页面
src/PickerApp.vue     底部快速列表
src/lib/commands.ts   前端调用 Tauri command 的封装
src-tauri/src/        Rust 后端模块
src-tauri/src/lib.rs  Tauri 应用入口与 commands
src-tauri/src/storage.rs
src-tauri/src/clipboard_monitor.rs
src-tauri/src/hotkey.rs
src-tauri/src/tray.rs
src-tauri/src/windowing.rs
```

## 当前暴露的 Tauri Commands

前端当前会使用以下 commands：

- `get_history`
- `get_app_state`
- `copy_item_to_clipboard`
- `toggle_item_pin`
- `delete_item`
- `clear_history`
- `set_monitoring_paused`
- `hide_window`
- `sync_picker_layout`

## 开发方式

### 环境要求

- `Node.js`
- `pnpm`
- `Rust`
- Windows 下 Tauri v2 所需构建环境

### 安装依赖

```bash
pnpm install
```

### 开发运行

```bash
pnpm tauri dev
```

### 构建前端

```bash
pnpm build
```

### 构建桌面应用

```bash
pnpm tauri build
```

## 本地存储

应用数据存放在系统应用数据目录中，包括：

- SQLite 数据库：`paste.sqlite`
- 图片缓存目录：`images/`

当前版本不包含云同步，也没有账号系统。

## 当前范围说明

已经实现：

- 文本与图片的本地剪贴板历史
- 管理页面
- 底部快速列表
- 托盘集成
- 管理页面的键盘优先操作

暂未实现：

- 云同步
- 富文本 / HTML 历史
- 文件列表剪贴板支持
- 应用黑名单规则
- 自定义快捷键设置界面
- 自动粘贴工作流

## License

当前仓库还没有声明开源许可证。
