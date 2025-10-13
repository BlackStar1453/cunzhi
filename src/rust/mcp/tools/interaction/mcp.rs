use anyhow::Result;
use rmcp::{Error as McpError, model::*};

use crate::mcp::{ZhiRequest, PopupRequest};
use crate::mcp::handlers::{create_tauri_popup, parse_mcp_response};
use crate::mcp::utils::{generate_request_id, popup_error};

/// 智能代码审查交互工具
///
/// 支持预定义选项、自由文本输入和图片上传
#[derive(Clone)]
pub struct InteractionTool;

impl InteractionTool {
    pub async fn zhi(
        request: ZhiRequest,
    ) -> Result<CallToolResult, McpError> {
        use crate::log_debug;

        // 尝试获取会话 ID（工作目录）
        // 优先级：working_directory 参数 > CUNZHI_SESSION_ID > PWD > current_dir
        let session_id = request.working_directory
            .clone()
            .or_else(|| std::env::var("CUNZHI_SESSION_ID").ok())
            .or_else(|| std::env::var("PWD").ok())
            .or_else(|| {
                std::env::current_dir()
                    .ok()
                    .and_then(|path| path.to_str().map(|s| s.to_string()))
            });

        log_debug!("检测到的 session_id: {:?}", session_id);

        let popup_request = PopupRequest {
            id: generate_request_id(),
            message: request.message,
            predefined_options: if request.predefined_options.is_empty() {
                None
            } else {
                Some(request.predefined_options)
            },
            bot_name: None, // 使用默认 bot 或根据 session_id 映射
            session_id,     // 传递会话 ID
            is_markdown: request.is_markdown,
        };

        match create_tauri_popup(&popup_request) {
            Ok(response) => {
                // 解析响应内容，支持文本和图片
                let content = parse_mcp_response(&response)?;
                Ok(CallToolResult::success(content))
            }
            Err(e) => {
                Err(popup_error(e.to_string()).into())
            }
        }
    }
}
