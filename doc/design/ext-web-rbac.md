# `ext/web` 可配置 RBAC（角色 ⊥ 权限）

| | |
|---|---|
| 验收 | qdqc：注册登录、评论、desk 内建角色/赋权；`tests/ext/web-rbac-*-smoke.mq.md` |
| 状态 | **Accepted · 已落地核心**（HTTP register + `/_rbac/*` + gate permissions；样站 qdqc 已接线） |

---

## 0. 一句话

权限是原子能力（`resource:action`）；角色是权限包；用户通过 `user_roles` 获得角色。门禁与业务都问 `can(user, permission)`。管理员可在后台新建角色、勾选权限、赋给用户——**禁止**只靠写死的 `role=admin|user` 字符串作为唯一模型（兼容期仍可读旧 `roles=` gate）。

---

## 1. 表结构（SQLite / Postgres 同构）

| 表 | 列 |
|----|-----|
| `web_permissions` | `code` PK, `resource`, `action`, `description` |
| `web_roles` | `id` PK, `name` UNIQUE, `is_system` INT, `tenant_id` TEXT NULL |
| `web_role_permissions` | `role_id`, `permission_code` PK(role_id, permission_code) |
| `web_user_roles` | `user_id`, `role_id`, `tenant_id` TEXT NULL, PK(user_id, role_id, tenant_id) |
| `web_users` | `id` PK, `username` UNIQUE, `password_hash`, `created_at` |

命名带 `web_` 前缀，避免与业务表冲突。站点也可映射自定义表名（后期）；首版固定。

---

## 2. 权限码约定

形如 `comments:create`、`desk:access`、`roles:manage`。内置种子见 `DefaultCatalog`。`*` 或角色名 `superadmin` / 兼容 `admin`：登录时展开为目录全集。

---

## 3. 作者 API

```markdown
**`应用` = > `应用`.权限 确保结构=真 目录=`权限目录`**
**`应用` = > `应用`.门禁 路径="/desk" 权限="desk:access" 匹配="prefix" 拒绝="redirect"**
**`应用` = > `应用`.鉴权 用户表=`管理员` 注册=真 注册路径="/register" …**
```

| 方法 | ABI | 说明 |
|------|-----|------|
| `app.rbac` / `权限` | `web_app_rbac` | ensure schema + optional seed catalog table |
| `app.gate` | `web_app_gate` | 新增 `permissions`/`权限`（CSV）；与 `roles` 可并存：有 permissions 时按权限判定 |
| `rbac.can` | `web_rbac_can` | `{ok, allowed}` |
| `rbac.assign_role` | `web_rbac_assign_role` | 用户←角色 |
| `rbac.set_role_permissions` | `web_rbac_set_role_permissions` | 角色←权限列表 |

防提权：`set_role_permissions` / `assign_role` 在 HTTP 管理面检查操作者 `roles:manage` 且不能授予自身没有的 permission。

---

## 4. 运行时

1. `listen`：若 `auth.rbac=true` 或调用过 `app.rbac`，对 `db` URL `EnsureSchema` + 种子。  
2. 登录成功：解析角色 → DB `user_roles` ∪ GFM 角色展开 → session `permissions`（CSV）+ 兼容 `role`。  
3. `withRBAC`：gate 含 `permissions` 则 `PermissionAllowed`；否则旧 `RoleAllowed`。  
4. 注册：`POST /register` 写入 `web_users`，默认赋 `member` 角色。

---

## 5. qdqc 映射

| 产品 | 权限码 |
|------|--------|
| 发评论 | `comments:create` |
| 删任意评论 | `comments:delete` |
| 进 desk | `desk:access` |
| 管角色 | `roles:manage` |

首个 GFM `admin` 用户登录后视为 `superadmin`（全权限）；DB 迁移后绑定 `superadmin` 角色。

---

## 6. 非目标（本切片）

对象级 ACL 全图、ABAC；租户列先可空，多租户波次再启用。
