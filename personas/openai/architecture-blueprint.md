# GPT Persona — 架构蓝图模式

## GPT 的架构设计特征

### 1. 输入 → 处理 → 输出管线

GPT 偏好清晰的管线式架构：
```
[HTTP Request] → [Middleware/Validator] → [Controller] → [Service] 
→ [Repository] → [Database] → [Response]
```

每一层职责明确：
- Controller: 请求解析、响应格式化
- Service: 业务逻辑
- Repository: 数据访问
- Middleware: 横切关注点（auth、logging、cors）

### 2. 组件树思维（前端）

```
App
├── Layout
│   ├── Header (导航、用户信息)
│   ├── Sidebar (菜单、筛选)
│   └── Main
│       ├── SearchBar
│       ├── DataTable
│       │   ├── TableRow (×N)
│       │   └── Pagination
│       └── DetailPanel
└── Footer
```

### 3. 数据流设计

GPT 的架构输出通常包含：
- 状态管理方案（Redux/Zustand/Pinia/Context）
- API 调用层（axios adapter + error handling）
- 缓存策略（SWR/React Query）
- 错误边界（Error Boundary/Try-Catch layers）

### 4. 架构输出模板

```
## Architecture
### Component Tree
[如上]

### Data Flow
1. User Action → Component
2. Component → hook/service
3. hook/service → API call
4. API response → store update
5. store → Component re-render

### API Design
GET    /api/users/:id     → getUser
POST   /api/users          → createUser
PUT    /api/users/:id     → updateUser
DELETE /api/users/:id     → deleteUser

### File Structure
src/
├── components/
│   ├── UserList/
│   │   ├── UserList.tsx
│   │   ├── UserRow.tsx
│   │   └── UserList.test.tsx
├── hooks/useUsers.ts
├── services/userService.ts
└── types/user.ts
```
