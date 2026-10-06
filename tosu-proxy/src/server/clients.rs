use crate::config::{
    CONFIG_ENDPOINT_CLIENTS, CONFIG_ENDPOINT_CLIENTS_BLINK, CONFIG_ENDPOINT_CLIENTS_SETTINGS,
};
use crate::model::http::clients::{
    ApplySettingsResponse, BlinkResponse, ClientListResponse, DisplayClientDto,
};
use crate::model::shared::setting::LyricSettingsPatch;
use crate::model::websocket::WebSocketMessage;
use crate::model::websocket::setting::SettingPayload;
use crate::server::ALL_SESSIONS;
use crate::server::response::{CODE_INVALID_PARAM, CODE_NOT_FOUND, render_error};
use crate::service::{current_settings, key_value_of, keys_of};
use salvo::http::StatusCode;
use salvo::prelude::*;
use salvo::websocket::Message;

fn blink_message() -> Message {
    let payload = SettingPayload::new("setBlink".to_string());
    Into::<WebSocketMessage>::into(payload).into()
}

#[handler]
async fn list_clients(res: &mut Response) {
    let clients = ALL_SESSIONS.display_clients().await;
    let items: Vec<_> = clients
        .into_iter()
        .map(|(key, snapshot)| DisplayClientDto {
            id: key,
            identity: snapshot.identity,
            connected_at: snapshot.connected_at,
            user_agent: snapshot.user_agent,
        })
        .collect();
    res.render(Json(ClientListResponse {
        total: items.len(),
        items,
    }));
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
    res.render(Json(BlinkResponse {
        ok: true,
        blinked: targets.len(),
    }));
}

/// 只推给目标展示端，不落库。重连后回到全局设置。
#[handler]
async fn apply_client_settings(req: &mut Request, res: &mut Response) {
    let Some(target) = req.param::<String>("id") else {
        render_error(
            res,
            StatusCode::BAD_REQUEST,
            CODE_INVALID_PARAM,
            "缺少目标 id",
        );
        return;
    };

    let payload = match req.payload().await {
        Ok(p) => p,
        Err(err) => {
            render_error(
                res,
                StatusCode::BAD_REQUEST,
                CODE_INVALID_PARAM,
                format!("读取请求体失败: {err}"),
            );
            return;
        }
    };
    let text = match std::str::from_utf8(payload) {
        Ok(t) => t,
        Err(err) => {
            render_error(
                res,
                StatusCode::BAD_REQUEST,
                CODE_INVALID_PARAM,
                format!("请求体不是合法 UTF-8: {err}"),
            );
            return;
        }
    };
    let patch: LyricSettingsPatch = match crate::util::to_json(text) {
        Ok(p) => p,
        Err(err) => {
            render_error(
                res,
                StatusCode::BAD_REQUEST,
                CODE_INVALID_PARAM,
                format!("请求体格式错误: {err}"),
            );
            return;
        }
    };

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

    let base = current_settings().await;
    let next = match base.with_patch(patch) {
        Ok(next) => next,
        Err(err) => {
            crate::server::response::render_service_error(res, err);
            return;
        }
    };

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

    res.render(Json(ApplySettingsResponse {
        ok: true,
        clients: targets.len(),
        keys: keys.iter().map(|k| k.ws_key().to_string()).collect(),
        sent,
    }));
}

#[handler]
async fn blink_all(res: &mut Response) {
    let clients = ALL_SESSIONS.display_clients().await;
    let message = blink_message();
    for (key, _) in &clients {
        ALL_SESSIONS.send_to_one_client(key, message.clone()).await;
    }
    res.render(Json(BlinkResponse {
        ok: true,
        blinked: clients.len(),
    }));
}

pub fn get_clients_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_CLIENTS)
        .get(list_clients)
        .push(Router::with_path(CONFIG_ENDPOINT_CLIENTS_BLINK).post(blink_all))
        .push(
            Router::with_path("{id}")
                .push(Router::with_path(CONFIG_ENDPOINT_CLIENTS_BLINK).post(blink_client))
                .push(
                    Router::with_path(CONFIG_ENDPOINT_CLIENTS_SETTINGS).post(apply_client_settings),
                ),
        )
}
