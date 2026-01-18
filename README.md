# PDB - PC 窗口控制器

一个类似 ADB 接口风格的 Rust PC 窗口控制库。支持点击、滑动、截图、文本输入、按键事件等操作。

## 安装

```bash
cargo build --release
```

## 快速开始

### 命令行使用

```bash
# 列出所有窗口
pdb-client --local devices

# 点击（坐标相对于窗口客户区）
pdb-client --local click <hwnd> <x> <y>

# 滑动
pdb-client --local swipe <hwnd> <x1> <y1> <x2> <y2> [时长ms]

# 截图
pdb-client --local screenshot <hwnd> output.png

# 输入文本
pdb-client --local text <hwnd> "ciallo"

# 发送按键
pdb-client --local key <hwnd> enter

# 追踪鼠标位置
pdb-client --local coord <hwnd>
```

### 库使用

```rust
use pdb::{WindowController, Device};

fn main() -> pdb::Result<()> {
    let controller = WindowController::new();
    
    // 列出窗口
    for window in controller.list_windows()? {
        println!("{}: {}", window.hwnd, window.title);
    }
    
    // 通过标题查找窗口
    let info = controller.find_window("记事本")?;
    let device = Device::new(info);
    
    // 执行操作
    device.click(100, 200)?;
    device.swipe(100, 100, 300, 300, 500)?;
    device.input_text("你好")?;
    device.key_event(pdb::KeyCode::Enter)?;
    
    let screenshot = device.screenshot()?;
    screenshot.save("截图.png")?;
    
    Ok(())
}
```

### 远程控制

```bash
# 启动服务端
pdb-server

# 客户端命令（默认连接 127.0.0.1:5037）
pdb-client devices
pdb-client click <hwnd> <x> <y>

# 连接远程服务器
pdb-client devices 192.168.1.100:5037
```

## 命令参考

| 命令 | 说明 |
|------|------|
| `devices` / `list` | 列出所有可见窗口 |
| `click <hwnd> <x> <y>` | 点击指定位置 |
| `swipe <hwnd> <x1> <y1> <x2> <y2> [ms]` | 滑动（默认 500ms）|
| `text <hwnd> <文本>` | 输入文本 |
| `key <hwnd> <按键>` | 发送按键 |
| `screenshot <hwnd> <路径>` | 截图保存到文件 |
| `coord <hwnd>` | 追踪鼠标位置（仅本地）|
| `ping` | 检查服务器状态（仅远程）|

## 按键代码

`enter`, `escape`, `backspace`, `tab`, `space`, `up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown`, `insert`, `delete`, `a-z`, `0-9`, `f1-f12`

## 注意事项

- **HWND 格式**: 支持十六进制（`0x12345`）或十进制
- **坐标**: 相对于窗口客户区（非屏幕坐标）
- **最小化窗口**: 自动恢复执行操作后重新最小化
- **截图**: 使用 Windows Graphics Capture API，支持硬件加速窗口

## 系统要求

- Windows 10 1803+（Windows Graphics Capture 支持）
- Rust 1.70+

## 许可证

MIT
