# Architecture

This document describes the structure and architectural decisions of the Scylla frontend, a React application built on **Clean Architecture** principles with **TanStack Query** for async data management.

---

## 1. Overview

### 1.1 Core Principles

- **Separation of concerns**: Each layer has a single, clear responsibility
- **Framework independence**: Business logic does not depend on React or any UI library
- **Testability**: Layers are decoupled via interfaces
- **Dependency inversion**: Outer layers depend on inner layers through abstractions

### 1.2 Error Handling — `ScyllaResult<T>`

All async operations return `ScyllaResult<T>`, a Result type encapsulating success or failure:

```typescript
// Wrapping an async call
const result = await ScyllaResult.tryAsync(
  async () => await api.call(),
  'Error message'
);

// Pattern matching
result.fold({
  onSuccess: (data) => handleSuccess(data),
  onError: (error) => handleError(error)
});

// Or throw on error
const data = result.unwrap();
```

`ScyllaError` extends `Error` with gRPC error code extraction and network error detection.

---

## 2. Module Structure

The application is organized into **independent modules** in `src/modules/`:

```
src/modules/
├── core/           → Infrastructure, DI wiring, routing, auth guard
├── features/       → Feature modules (business functionality)
│   ├── agents/
│   ├── apps/
│   ├── jobs/
│   ├── login/
│   ├── marketplace/
│   ├── organization/
│   ├── permission/   (exposed in DI as `authz`)
│   ├── pipeline/
│   ├── project/
│   ├── secret/
│   └── user/
├── layout/         → App shell (sidebar, topbar, breadcrumbs, context selector)
└── shared/         → Reusable components, hooks, stores, utilities
```

### 2.1 Core Module

Global infrastructure and app-level concerns:

| Folder | Content |
|--------|---------|
| `di/` | `CoreModule` (gRPC transport), `Dependencies` (aggregates all feature modules) |
| `infrastructure/grpc/` | `CoreGrpcTransport` — shared gRPC-Web transport |
| `presentation/ui/router/` | `CoreRouter` (route definitions), `AuthGuard`, `ContextCleanerWrapper` |
| `presentation/providers/` | `DependenciesProvider` (React context for DI) |
| `presentation/structs/` | `RouteHandle`, `ScyllaForm` shape types |

### 2.2 Feature Modules

Each feature follows an identical layered structure (see §3).

| Module | DI key | Description |
|--------|--------|-------------|
| `login` | `login` | Authentication (login flow) |
| `organization` | `organization` | Organization CRUD |
| `project` | `project` | Project CRUD |
| `pipeline` | `pipeline` | Pipeline dashboard, creation/editing, charts |
| `jobs` | `jobs` | Job list per pipeline (+ logs, tail) |
| `user` | `user` | User admin (CRUD), user settings |
| `permission` | `authz` | Roles, grants, effective permissions, authz vocabulary |
| `secret` | `secret` | Project-scoped secrets (metadata; value is write-only) |
| `apps` | `apps` | Machine principals / API credentials |
| `agents` | `agents` | Agents (workers that pick up jobs) |
| `marketplace` | `marketplace` | Component marketplace |

> The DI key is how the module is reached in hooks (`useDependencies().<key>`). It usually matches the folder name — the exception is `permission`, exposed as `authz`.

### 2.3 Layout Module

App shell rendered inside authenticated routes:

- `Layout.tsx` — Sidebar + TopBar + animated outlet
- `AppSidebar.tsx` — Navigation with context selector
- `ScyllaBreadcrumbs.tsx` — Dynamic breadcrumbs from route handles
- `context-selector/` — Organization/Project selector components

### 2.4 Shared Module

Reusable across all features — **no business logic**.

| Folder | Content |
|--------|---------|
| `domain/structs/` | `PaginationInfo`, `PaginationParams` |
| `presentation/ui/` | `FeatureHeader`, `FormDialog`, `ScyllaForm`, `DataTable`, `Pagination`, `ErrorState`, `ConfirmOperationAlertDialog`, `ListCard` |
| `presentation/ui/shadcn/` | shadcn/ui primitives |
| `presentation/hooks/` | `useSelection`, `usePagination`, `usePipelineJobs`, `useScyllaNavigate` |
| `presentation/stores/` | `useContextStore` (org/project context), `useSelectionStore` (generic selection) |
| `presentation/structs/` | `ScyllaForm` shapes (`FormItem`, `FormChange`, `FormItemType`) |
| `utils/` | `ScyllaResult`, `dateUtils`, `jobStatusMapper` |

---

## 3. Feature Layer Architecture

Every feature module follows the same 4-layer structure:

```
feature/
├── di/                          → Dependency wiring
│   └── feature.module.ts
├── domain/                      → Pure business logic
│   ├── usecases/
│   ├── repository/              → Repository interfaces
│   ├── entities/                → Domain entities ({Name}Entity, identity objects)
│   └── structs/                 → Plain data shapes: value objects, enums, DTOs, wrappers (no identity)
├── infrastructure/              → Technical implementations
│   ├── repository/
│   │   ├── feature.repository.ts        → Repository implementation
│   │   ├── data-sources/
│   │   │   └── feature-remote.data-source.ts   → Data source interface
│   │   └── mappers/
│   │       └── grpc-feature.mapper.ts          → Proto → Domain mapping
│   └── data/
│       └── remote/
│           └── feature-remote.data-source.impl.ts  → gRPC calls
├── locales/                     → i18n translations (en/, fr/)
└── presentation/                → UI layer
    ├── hooks/                   → React Query hooks
    ├── stores/                  → Zustand stores (UI state only)
    └── ui/                      → React components (pages, dialogs, tables)
```

### 3.1 Domain Layer

**Pure business logic, zero external dependencies.**

- **Use Cases**: Single-responsibility classes that call repository methods
- **Repository Interfaces**: Abstract contracts — no knowledge of gRPC or HTTP
- **Entities** (`domain/entities/*.entity.ts`): identity-bearing business objects, independent from proto-generated types
- **Structs** (`domain/structs/*.struct.ts`): plain data shapes with no identity — value objects, enums, DTOs, and list/result wrappers

```typescript
export class GetUsersUseCase {
  constructor(private readonly _repository: UserRepository) {}
  execute = () => this._repository.getAll();
}
```

#### Entities vs. Structs

The domain layer separates two kinds of types. Both are pure and proto-independent, but they answer different questions:

| | **Entity** (`entities/*.entity.ts`) | **Struct** (`structs/*.struct.ts`) |
|---|---|---|
| Question | "What *thing* does this feature own?" | "What *plain shapes* describe or move data?" |
| Identity | Yes — has an `id` (or a stable key) | No — interchangeable by value |
| Examples | `SecretEntity`, `RoleEntity`, `UserEntity`, `PipelineEntity` | `Permission`, `PermissionScope` (enums), `ProjectList`, `CreatedApp`, `PipelineMetadata` |
| Naming | `{Name}Entity` | plain PascalCase (no suffix) |
| One file per | aggregate / entity | cohesive group of related shapes |

An entity file is the home for everything that revolves around that entity, not just the read shape:

```typescript
// secret.entity.ts — read shape (metadata only; the value is write-only)
export interface SecretEntity {
  id: string;
  projectId: string;
  name: string;
  description: string;
  createdAt: string;
  updatedAt: string;
}

// ...plus the input shapes that belong to it
export interface CreateSecretInput {
  projectId: string;
  name: string;
  value: string; // write-only, sent once, never returned
  description: string;
}
```

Entities may also carry **pure domain behavior** (validation, transformations) — still no framework or transport imports:

```typescript
// role.entity.ts
export const updateRole = (role: RoleEntity, changes: Partial<RoleEntity>): RoleEntity => {
  if (changes.name !== undefined && changes.name.trim() === '') {
    throw new Error('Role name cannot be empty');
  }
  return { ...role, ...changes, id: role.id };
};
```

**Structs** are shared across entities and use cases — e.g. `permission.struct.ts` exports the `Permission`, `PermissionScope`, and `PrincipalKind` enums that `RoleEntity`, `GrantEntity`, and `EffectivePermissionsEntity` all reference; `project.struct.ts` exports the `ProjectList` wrapper around `ProjectEntity`. A struct file may import an entity (e.g. a `CreatedApp` result wrapping an `AppEntity`), but never the reverse direction of identity ownership.

> There are **no `*.model.ts` files** — every domain type is either an entity (`*.entity.ts` / `{Name}Entity`) or a struct (`*.struct.ts` / plain name). The `Entity` suffix is the identity signal and also disambiguates from the proto-generated type of the same bare name (aliased in mappers, e.g. `User as ProtoUser`). Presentation-layer view models follow the same rule under `presentation/structs/`.

Mappers convert proto → entity (one `Grpc{Entity}Mapper` per entity, e.g. `GrpcSecretMapper.toDomain` returns a `SecretEntity`).

### 3.2 Infrastructure Layer

**Concrete implementations of domain abstractions.**

- **Data Sources**: Interface + implementation for each transport (gRPC, localStorage, etc.)
- **Repository Impl**: Coordinates data sources, maps infrastructure types to domain types
- **Mappers**: Transform proto-generated types ↔ domain entities/structs

```typescript
export class UserRepositoryImpl implements UserRepository {
  constructor(private readonly _remoteDataSource: UserRemoteDataSource) {}

  public async getAll(): Promise<ScyllaResult<UserList>> {
    return (await this._remoteDataSource.getAll()).map(list => GrpcUserMapper.toDomainList(list));
  }
}
```

### 3.3 Presentation Layer

**React components, hooks, and local UI state.**

- **Hooks**: Wrap use cases with TanStack Query (`useQuery` / `useMutation`)
- **Stores**: Zustand stores for ephemeral UI state (form state, modal visibility)
- **UI**: Pages, dialogs, tables, columns

```typescript
export const useCreateUser = () => {
  const { createUser } = useDependencies().user;
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ username, password }) =>
      (await createUser.execute(username, password)).unwrap(),
    onSuccess: () => {
      toast.success('User created');
      return queryClient.invalidateQueries({ queryKey: ['users'] });
    },
  });
};
```

### 3.4 DI Layer

**Manual dependency injection, no IoC container.**

```typescript
const dataSource = new UserRemoteDataSourceImpl(CoreModule.data.grpcTransport);
const repository = new UserRepositoryImpl(dataSource);

export const UserModule = {
  domain: {
    getUsers: new GetUsersUseCase(repository),
    createUser: new CreateUserUseCase(repository),
    deleteUser: new DeleteUserUseCase(repository),
  },
};
```

All modules are aggregated in `Dependencies.ts` and provided to the React tree via `DependenciesProvider`.

---

## 4. Data Flow

### 4.1 Read Flow

```
Component → useQuery hook → UseCase.execute() → Repository (interface)
    → RepositoryImpl → RemoteDataSourceImpl → gRPC client
    → ScyllaResult<T> ← mapper ← proto response
```

### 4.2 Write Flow

```
Component → useMutation hook → UseCase.execute() → Repository
    → RemoteDataSourceImpl → gRPC client
    → onSuccess: invalidateQueries() → automatic refetch
```

### 4.3 Batch Data Loading

For N+1 query avoidance (e.g., loading jobs for all pipelines on a dashboard):

```typescript
// usePipelinesJobs.ts — uses useQueries for parallel fetching
const queries = useQueries({
  queries: pipelineIds.map(id => ({
    queryKey: JOBS_QUERY_KEY(id),
    queryFn: () => getPipelineJobs.execute(id, { page: 1, pageSize: 10 }),
  })),
});
// Returns Map<pipelineId, JobResponse[]>
```

---

## 5. Shared Patterns

### 5.1 Selection — `useSelection(key)`

Generic selection system backed by a single Zustand store (`useSelectionStore`), keyed by feature name:

```typescript
const { selectedIds, select, clearSelection } = useSelection('pipelines');
```

Used by `DataTable` (row click) and `FeatureHeader` (clear/delete actions). No per-feature store needed.

### 5.2 Feature Header — `FeatureHeader`

Standardized header component for list pages:

```tsx
<FeatureHeader
  count={totalCount}
  label='Pipeline'
  selectedCount={selectedIds.length}
  onClearSelection={clearSelection}
  onDeleteSelection={handleDelete}
  onNew={() => setOpenDialog(true)}
  newLabel={<Trans>New pipeline</Trans>}
/>
```

Provides: title with count, clear selection button, delete button with confirmation dialog, create button.

### 5.3 Form System — `ScyllaForm` + `FormDialog`

Declarative form rendering from `FormItem[]` definitions:

```typescript
const items: FormItem[] = [
  { id: 'name', label: t`Name`, type: FormItemType.Input, inputType: 'text' },
  { id: 'org', label: t`Org`, type: FormItemType.Select, options: [...] },
];
```

- **`ScyllaForm`**: Standalone form with customizable footer (render prop)
- **`FormDialog`**: Wraps `ScyllaForm` inside a `Dialog` with Cancel/Submit buttons
- **`useFormState`**: Hook managing form values, change handler, reset, validation

### 5.4 Pagination — `usePagination`

Generic pagination hook with optimistic page updates:

```typescript
const { paginationParams, paginationInfo, setPage, updatePaginationInfo } = usePagination();
```

Maintains local page/pageSize state, merges with server-returned `totalCount`/`totalPages` for immediate UI responsiveness.

---

## 6. Routing

Centralized in `Core.router.tsx` using React Router v7. Authenticated routes are nested under an **organization slug** segment; `OrganizationRedirectWrapper` sends `/` to the active org and `OrganizationSyncWrapper` keeps the context store in sync with `:organizationSlug`.

| Route | Page | Auth |
|-------|------|------|
| `/login` | Login | Public |
| `/` | Redirect to active org slug (`OrganizationRedirectWrapper`) | Protected |
| `/:organizationSlug/projects` | Project list | Protected |
| `/:organizationSlug/projects/:projectId` | Pipeline dashboard | Protected |
| `/:organizationSlug/projects/:projectId/secrets` | Secrets | Protected |
| `/:organizationSlug/projects/:projectId/create` | Pipeline creation | Protected |
| `/:organizationSlug/projects/:projectId/edit/:pipelineId` | Pipeline editing | Protected |
| `/:organizationSlug/projects/:projectId/pipelines/:pipelineId/jobs` | Jobs list | Protected |
| `/:organizationSlug/marketplace` | Marketplace | Protected |
| `/:organizationSlug/agents` | Agents list | Protected |
| `/:organizationSlug/agents/:agentId` | Agent details | Protected |
| `/:organizationSlug/users-admin` | User admin | Protected |
| `/:organizationSlug/users` | User admin | Protected |
| `/:organizationSlug/users/:userId` | User settings | Protected |
| `/:organizationSlug/users/me` | User settings | Protected |
| `*` | Redirect to `/login` | — |

All protected routes are wrapped by `AuthGuard` and `Layout`. The `:projectId` subtree is additionally wrapped by `ContextCleanerWrapper` (clears stale project/pipeline context). Breadcrumbs come from each route's `handle.breadcrumb`.

---

## 7. Global State

| Store | Scope | Purpose |
|-------|-------|---------|
| `useContextStore` | App-wide | Current organization & project selection |
| `useSelectionStore` | App-wide | Generic row selection keyed by feature |

All other state is managed by TanStack Query (server state) or local `useState` (component state).

---

## 8. Tech Stack

| Technology | Purpose |
|------------|---------|
| React 18 | UI rendering |
| TypeScript 5.8 | Type safety |
| TanStack Query 5 | Server state, caching, mutations |
| Zustand 5 | Client state management |
| React Router 7 | Routing |
| Lingui 5 | Internationalization (en, fr) |
| gRPC-Web (protobuf-ts) | Backend communication |
| shadcn/ui + Radix | UI component primitives |
| Tailwind CSS 4 | Styling |
| Vite 7 | Build tool |
| Framer Motion | Page transitions |
