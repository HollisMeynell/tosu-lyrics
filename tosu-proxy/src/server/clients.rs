//! 在线展示端与定向事件（B-08）。
//!
//! - `GET  /api/clients`              在线展示端列表
//! - `POST /api/clients/{id}/blink`   让**指定**展示端闪烁
//! - `POST /api/clients/blink`        让全部展示端闪烁
//!
//! 只有**展示端**（`ClientType::Client`）会出现在列表里：管理 / setter 连接
//! 没有画面，让它们参与"在线客户端"既没意义也会误导用户。
//!
//! 定向事件只走 WS 单播 —— 这就是 WS 在本项目里的定位：
//! **展示事件传输**，而不是管理 RPC。管理一律走 HTTP。

use crate::config::{
    CONFIG_ENDPOINT_CLIENTS, CONFIG_ENDPOINT_CLIENTS_BLINK, CONFIG_ENDPOINT_CLIENTS_SETTINGS,
};
use crate::model::setting::LyricSettingsPatch;
use crate::service::{current_settings, key_value_of, keys_of};
use crate::model::websocket::WebSocketMessage;
use crate::model::websocket::setting::SettingPayload;
use crate::server::ALL_SESSIONS;
use crate::server::response::{CODE_INVALID_PARAM, CODE_NOT_FOUND, render_error};
use salvo::http::StatusCode;
use salvo::websocket::Message;
use salvo::prelude::*;
use serde_json::json;

/// 让一个展示端闪烁：复用展示事件通道（与 `setClear` 同一类消息），
/// 不引入新的管理消息类型。
fn blink_message() -> Message {
    let payload = SettingPayload::new("setBlink".to_string());
    Into::<WebSocketMessage>::into(payload).into()
}

#[handler]
async fn list_clients(res: &mut Response) {
    let clients = ALL_SESSIONS.display_clients().await;
    let items: Vec<_> = clients
        .into_iter()
        .map(|(key, snapshot)| {
            json!({
                "id": key,
                "identity": snapshot.identity,
                "connectedAt": snapshot.connected_at,
                "userAgent": snapshot.user_agent,
            })
        })
        .collect();
    res.render(Json(json!({ "total": items.len(), "items": items })));
}

#[handler]
async fn blink_client(req: &mut Request, res: &mut Response) {
    let Some(target) = req.param::<String>("id") else {
        render_error(
            res,
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "缺少目标 id",
        );
        return;
    };

    // 目标可以是会话 key，也可以是客户端自报的稳定身份
    let targets = ALL_SESSIONS.resolve_display_targets(&target).await;
    if targets.is_empty() {
        render_error(
            res,
            StatusCode::NOT_FOUND,
            CODE_NOT_FOUND,
            format!("展示端 {target} 不在线"),
        );
        return;
    }

    let message = blink_message();
    for key in &targets {
        ALL_SESSIONS.send_to_one_client(key, message.clone()).await;
    }
    res.render(Json(json!({ "ok": true, "blinked": targets.len() })));
}

/// `POST /api/clients/{id}/settings`
///
/// **只推给目标展示端，不落库。**
///
/// 这是"单独客户端调整"的实现：选中某个端之后改样式，只有它变，
/// 其余端与全局设置都不受影响。它**不是**每客户端持久配置系统 ——
/// 后端不保存任何 per-client 状态，重连后回到全局设置。
#[handler]
async fn apply_client_settings(req: &mut Request, res: &mut Response) {
    let Some(target) = req.param::<String>("id") else {
        render_error(res, StatusCode::BAD_REQUEST, CODE_INVALID_PARAM, "缺少目标 id");
        return;
    };

    let payload = match req.payload().await {
        Ok(p) => p,
        Err(err) => {
            render_error(res, StatusCode::BAD_REQUEST, CODE_INVALID_PARAM, format!("读取请求体失败: {err}"));
            return;
        }
    };
    let text = match std::str::from_utf8(payload) {
        Ok(t) => t,
        Err(err) => {
            render_error(res, StatusCode::BAD_REQUEST, CODE_INVALID_PARAM, format!("请求体不是合法 UTF-8: {err}"));
            return;
        }
    };
    let patch: LyricSettingsPatch = match crate::util::to_json(text) {
        Ok(p) => p,
        Err(err) => {
            render_error(res, StatusCode::BAD_REQUEST, CODE_INVALID_PARAM, format!("请求体格式错误: {err}"));
            return;
        }
    };

    let targets = ALL_SESSIONS.resolve_display_targets(&target).await;
    if targets.is_empty() {
        render_error(res, StatusCode::NOT_FOUND, CODE_NOT_FOUND, format!("展示端 {target} 不在线"));
        return;
    }

    // 以**当前全局设置**为基准算出目标值，但不保存
    let base = current_settings().await;
    let next = match base.with_patch(patch) {
        Ok(next) => next,
        Err(err) => {
            crate::server::response::render_service_error(res, err);
            return;
        }
    };

    // 只推送真正变化的项，且推的是**完整的该项取值**，展示端直接套用即可
    let keys = keys_of(&base, &next);
    let mut sent = 0;
    for key in &keys {
        let mut payload = SettingPayload::new(key.ws_key().to_string());
        payload.value = Some(key_value_of(&next, *key));
        let message: Message = Into::<WebSocketMessage>::into(payload).into();
        for id in &targets {
            ALL_SESSIONS.send_to_one_client(id, message.clone()).await;
            sent += 1;
        }
    }

    res.render(Json(json!({
        "ok": true,
        "clients": targets.len(),
        "keys": keys.iter().map(|k| k.ws_key()).collect::<Vec<_>>(),
        "sent": sent,
    })));
}

#[handler]
async fn blink_all(res: &mut Response) {
    let clients = ALL_SESSIONS.display_clients().await;
    let message = blink_message();
    for (key, _) in &clients {
        ALL_SESSIONS.send_to_one_client(key, message.clone()).await;
    }
    res.render(Json(json!({ "ok": true, "blinked": clients.len() })));
}

pub fn get_clients_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_CLIENTS)
        .get(list_clients)
        .push(Router::with_path(CONFIG_ENDPOINT_CLIENTS_BLINK).post(blink_all))
        .push(
            Router::with_path("{id}")
                .push(Router::with_path(CONFIG_ENDPOINT_CLIENTS_BLINK).post(blink_client))
                .push(
                    Router::with_path(CONFIG_ENDPOINT_CLIENTS_SETTINGS)
                        .post(apply_client_settings),
                ),
        )
}
