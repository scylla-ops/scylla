# Naming Conventions

This document defines the naming rules applied across the Scylla frontend codebase.

---

## 1. Files & Folders

### 1.1 General Rule

All files and folders use **kebab-case**:

```
use-create-user.ts
user-remote.data-source.ts
pagination.model.ts
```

### 1.2 File Suffixes

Files are suffixed by their role to make intent clear at a glance:

| Layer | Suffix | Example |
|-------|--------|---------|
| **Page component** | `.page.tsx` | `UserAdmin.page.tsx` |
| **Use case** | `.use-case.ts` | `create-user.use-case.ts` |
| **Repository interface** | `.repository.ts` | `user.repository.ts` |
| **Repository implementation** | `.repository.ts` (in `infrastructure/`) | `user.repository.ts` |
| **Data source interface** | `.data-source.ts` | `user-remote.data-source.ts` |
| **Data source implementation** | `.data-source.impl.ts` | `user-remote.data-source.impl.ts` |
| **Mapper** | `.mapper.ts` | `grpc-user.mapper.ts` |
| **Domain entity** | `.entity.ts` | `secret.entity.ts`, `role.entity.ts`, `grant.entity.ts` |
| **Domain model (value object)** | `.model.ts` | `permission.model.ts`, `pagination.model.ts` |
| **Presentation model** | `.model.ts` | `scylla-form.model.ts`, `route-handle.model.ts` |
| **Hook** | `use-{name}.ts` | `use-create-user.ts`, `use-selection.ts` |
| **Zustand store** | `use-{name}.store.ts` | `use-context.store.ts`, `use-selection.store.ts` |
| **DI module** | `.module.ts` | `user.module.ts`, `pipeline.module.ts` |
| **Guard / Wrapper** | `.guard.tsx` / `.wrapper.tsx` | `Auth.guard.tsx`, `ContextCleaner.wrapper.tsx` |
| **Router** | `.router.tsx` | `Core.router.tsx` |

### 1.3 React Component Files

Component files use **PascalCase**:

```
FeatureHeader.tsx
FormDialog.tsx
ScyllaForm.tsx
DataTable.tsx
```

### 1.4 Folder Names

Always **kebab-case**, matching the module or concept name:

```
features/pipeline/
presentation/hooks/
infrastructure/data-sources/
```

---

## 2. TypeScript Naming

### 2.1 Interfaces & Types

**PascalCase**, no `I` prefix:

```typescript
interface UserRepository { ... }
interface PaginationInfo { ... }
type FormItem = FormItemBase & (FormInput | FormSelect);
type PipelineTableProps = { ... };
```

### 2.2 Domain Entities & Value Objects

The domain layer distinguishes **entities** (identity-bearing business objects, in `domain/entities/*.entity.ts`) from **value-object models** (no identity — enums, scopes, shared primitives — in `domain/models/*.model.ts`). See `architecture.md` §3.1 for the rationale.

| Kind | File | Type name | Example |
|------|------|-----------|---------|
| Entity | `{name}.entity.ts` | `{Name}Entity` | `secret.entity.ts` → `SecretEntity`; `role.entity.ts` → `RoleEntity` |
| Entity input/creation shape | (same entity file) | `Create{Name}Input` / `{Name}CreationData` | `CreateSecretInput`, `RoleCreationData` |
| Entity behavior (pure fn) | (same entity file) | `camelCase` verb | `updateRole(role, changes)` |
| Value object / enum | `{name}.model.ts` | plain PascalCase | `Permission`, `PermissionScope`, `PrincipalKind` |

```typescript
// secret.entity.ts
export interface SecretEntity { id: string; projectId: string; name: string; /* ... */ }
export interface CreateSecretInput { projectId: string; name: string; value: string; /* ... */ }

// permission.model.ts
export enum PermissionScope { UNSPECIFIED = 0, SYSTEM = 1, ORGANIZATION = 2, PROJECT = 3 }
```

The `Entity` suffix is the identity signal — use it for the thing the feature *owns*; keep enums and value objects suffix-free in `models/`.

> Older modules (`jobs`, `pipeline`, `user`) keep their entities as plain `{Name}` in `models/*.model.ts` (e.g. `Job`, `Pipeline`, `User`). New code should use the `entities/` + `{Name}Entity` convention.

### 2.3 Classes

**PascalCase**, suffixed by role:

| Role | Pattern | Example |
|------|---------|---------|
| Use case | `{Verb}{Entity}UseCase` | `GetUsersUseCase`, `CreateUserUseCase`, `DeleteUserUseCase` |
| Repository impl | `{Entity}RepositoryImpl` | `UserRepositoryImpl` |
| Data source impl | `{Entity}RemoteDataSourceImpl` | `UserRemoteDataSourceImpl` |
| Mapper | `Grpc{Entity}Mapper` | `GrpcUserMapper` |
| Error | `ScyllaError` | — |
| Result | `ScyllaResult<T>` | — |

### 2.4 Enums

**PascalCase** for names, **UPPER_SNAKE_CASE** or **PascalCase** for values:

```typescript
enum FormItemType {
  Input = 'input',
  Select = 'select',
}

enum Act {
  CREATE = 0,
  READ = 1,
}
```

### 2.5 Constants

**UPPER_SNAKE_CASE** for true constants, **camelCase** for derived/computed values:

```typescript
const DEFAULT_PAGE_SIZE = 10;
const MAX_JOBS_PER_PIPELINE = 10;
const EMPTY_ARRAY: string[] = [];

const JOBS_QUERY_KEY = (pipelineId: string) => ['jobs', 'pipeline', pipelineId] as const;
```

---

## 3. React Naming

### 3.1 Components

**PascalCase**, named by what they render:

```typescript
export const FeatureHeader = ({ ... }: FeatureHeaderProps) => { ... };
export const PipelineTable = ({ ... }: PipelineTableProps) => { ... };
export function FormDialog({ ... }: FormDialogProps) { ... }
```

### 3.2 Props

**PascalCase** type name = `{ComponentName}Props`:

```typescript
interface FeatureHeaderProps {
  count: number;
  label: string;
  onNew?: () => void;
}
```

### 3.3 Hooks

**camelCase** starting with `use`:

| Type | Pattern | Example |
|------|---------|---------|
| Data fetching | `use{Entity}s` / `use{Entity}` | `useUsers()`, `useUser(id)` |
| Mutation | `use{Verb}{Entity}` | `useCreateUser()`, `useDeleteUser()` |
| Shared behavior | `use{Concept}` | `useSelection(key)`, `usePagination()` |
| Navigation | `useScyllaNavigate` | `useScyllaNavigate()` |
| Form state | `useFormState` | `useFormState(items)` |

### 3.4 Stores (Zustand)

Hook-style naming with `Store` suffix for the raw store, plain `use{Name}` for the consumer hook:

```typescript
// Store definition
export const useSelectionStore = create<SelectionState>(...);

// Consumer hook (wraps the store)
export const useSelection = (key: string) => { ... };
```

---

## 4. DI Module Naming

Each feature exposes a module object with a `domain` property:

```typescript
export const UserModule = {
  domain: {
    getUsers: getUsersUseCase,
    createUser: createUserUseCase,
    deleteUser: deleteUserUseCase,
  },
};
```

The `Dependencies` object maps features to their domain API:

```typescript
export const dependencies = {
  user: UserModule.domain,
  pipeline: PipelineModule.domain,
  // ...
};
```

Access in hooks: `useDependencies().user.createUser.execute(...)`.

---

## 5. Query Keys

Use **factory functions** to ensure consistency between queries and invalidations:

```typescript
// Definition (in the hook file)
export const JOBS_QUERY_KEY = (pipelineId: string) =>
  ['jobs', 'pipeline', pipelineId, MAX_JOBS_PER_PIPELINE] as const;

// Usage in query
queryKey: [...JOBS_QUERY_KEY(pipelineId)],

// Usage in invalidation
queryClient.invalidateQueries({ queryKey: JOBS_QUERY_KEY(pipelineId), exact: true });
```

---

## 6. i18n

- Translation files live in `locales/{lang}/messages.po` (per feature or global)
- Use `<Trans>` for JSX, `` t`...` `` for strings
- Labels in `FormItem[]` accept `ReactNode` to support `<Trans>` components

---

## 7. Summary Table

| Concept | Casing | Example |
|---------|--------|---------|
| File (non-component) | kebab-case | `use-create-user.ts` |
| File (component) | PascalCase | `FeatureHeader.tsx` |
| Folder | kebab-case | `data-sources/` |
| Interface / Type | PascalCase | `UserRepository` |
| Domain entity | PascalCase + `Entity` (`*.entity.ts`) | `SecretEntity`, `RoleEntity` |
| Value object / enum | PascalCase (`*.model.ts`) | `Permission`, `PermissionScope` |
| Class | PascalCase + suffix | `GetUsersUseCase` |
| Hook | camelCase `use*` | `useCreateUser` |
| Store | camelCase `use*Store` | `useSelectionStore` |
| Constant | UPPER_SNAKE_CASE | `DEFAULT_PAGE_SIZE` |
| Enum | PascalCase | `FormItemType` |
| Component | PascalCase | `FeatureHeader` |
| Props type | PascalCase + `Props` | `FeatureHeaderProps` |
| DI module | PascalCase + `Module` | `UserModule` |
| Query key factory | UPPER_SNAKE_CASE | `JOBS_QUERY_KEY` |

