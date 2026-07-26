# Auth SSO

CPOAuth 作为唯一身份源，基于非对称加密验证下游应用，签发 JWT。

## 两组密钥

| 密钥对 | 用途 |
|--------|------|
| Client Sign/Verify | 签名/验证下游 App 凭证 |
| JWT Sign/Verify | 签发/验证用户 JWT |

## 流程

```
App 用私钥对请求签名 → /flow/auth?credential=...&signature=...&callback_url=...
                    → 验签通过 → 302 到 CPOAuth
                    → 用户在 CPOAuth 登录
                    → CPOAuth 回调 /flow/callback
                    → 取 CPOAuth userinfo → 签发 JWT（sub + username + 所有字段）
                    → /flow/complete 展示，可点击返回 App
App 收到 token → 可调 POST /api/oauth/refresh 续期（最多 12 次，每次 7 天）
```

## 快速开始

```bash
.\scripts\setup-env.ps1        # 生成两组密钥 + .env
# 编辑 .env 填 CPOAUTH_* 字段
cargo run --bin keygen -- app -c my-app --callback-url https://my.app/cb
cargo run -p auth              # http://127.0.0.1:3000
```

## 环境变量

| 变量 | 说明 |
|------|------|
| `AUTH_CLIENT_VERIFY_KEY` | 验证 App 凭证的公钥 |
| `AUTH_JWT_SIGN_KEY` | 签发 JWT 的私钥 |
| `AUTH_JWT_VERIFY_KEY` | 验证 JWT 的公钥 |
| `CPOAUTH_CLIENT_ID` | 本服务在 CPOAuth 的 client_id |
| `CPOAUTH_CLIENT_SECRET` | 本服务在 CPOAuth 的 client_secret |
| `CPOAUTH_REDIRECT_URI` | CPOAuth 回调（本服务 /flow/callback） |

## 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/flow/auth` | App 触发认证 (?credential=&signature=&callback_url=&state=) |
| `GET` | `/flow/callback` | CPOAuth 回调 |
| `GET` | `/flow/complete` | 认证完成页 |
| `POST` | `/api/oauth/refresh` | 刷新 JWT（{"token": "..."}） |

## keygen

```bash
cargo run --bin keygen -- server    # 生成两组服务器密钥（.env 用 3 个，sign key 给 keygen 用）
cargo run --bin keygen -- app -c my-app --callback-url https://my.app/cb  # 签发 App 凭证（需要 AUTH_CLIENT_SIGN_KEY 环境变量）
```
