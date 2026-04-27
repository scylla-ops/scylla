# Architecture

Ce document décrit la structure et les choix architecturaux de notre frontend React, basé sur la **Clean Architecture** et utilisant **TanStack Query** pour la gestion des données asynchrones.

---
## 1. Vue d'ensemble de l'architecture

### 1.1 Principes fondamentaux

L'application suit les principes de la **Clean Architecture** adaptée pour une application React moderne :

- **Séparation des responsabilités** : Chaque couche a une responsabilité claire et limitée
- **Indépendance des frameworks** : La logique métier ne dépend pas de React ou d'autres librairies UI
- **Testabilité** : Les couches sont découplées, facilitant les tests unitaires
- **Inversion de dépendance** : Les couches externes dépendent des couches internes via des interfaces

### 1.2 Gestion des erreurs et résultats

L'application utilise un système de gestion des résultats personnalisé via `ScyllaResult<T>` et `ScyllaError` :

- **ScyllaResult** : Encapsule le résultat d'une opération (succès ou erreur)
- **ScyllaError** : Classe d'erreur personnalisée avec support du code d'erreur et du logging
- Les méthodes `tryAsync` et `try` permettent d'encapsuler automatiquement les opérations dans un Result
- Le pattern `fold` permet de gérer proprement les deux cas (succès/erreur)

```typescript
const result = await ScyllaResult.tryAsync(
  async () => await api.call(),
  'Error message'
);

result.fold({
  onSuccess: (data) => handleSuccess(data),
  onError: (error) => handleError(error)
});
```

---

## 2. Structure modulaire

L'application est organisée en **modules indépendants** dans `src/modules/`, chacun suivant une architecture en couches.

### 2.1 Types de modules

#### **Modules Core** (`core/`)
Infrastructure partagée et configuration globale :
- `di/` : Injection de dépendances (CoreModule, Dependencies)
- `infrastructure/` : Services d'infrastructure (GrpcTransport)
- `presentation/` : Composants React de base (App, CoreRouter, AuthGuard)

#### **Modules Features** (`features/`)
Fonctionnalités métier isolées :
- `login/` : Authentification
- `organization/` : Gestion des organisations
- `project/` : Gestion des projets
- `pipeline-dashboard/` : Tableau de bord des pipelines
- `pipeline-creation/` : Création/édition de pipelines
- `marketplace/` : Marketplace de composants
- `user_settings/` : Paramètres utilisateur

#### **Module Layout** (`layout/`)
Structure visuelle de l'application :
- Sidebar, TopBar, Navigation
- Context Selector (sélection Organisation/Project)

#### **Module Shared** (`shared/`)
Composants et utilitaires réutilisables :
- `presentation/ui/shadcn/` : Composants UI (shadcn)
- `presentation/stores/` : Stores Zustand partagés (useContext)
- `utils/` : Utilitaires (ScyllaResult)

---

## 3. Architecture des couches (par module feature)

Chaque module feature suit la même structure en couches :

### 3.1 Couche **Presentation**

**Responsabilité** : Interface utilisateur et état UI local

**Contenu** :
- `ui/` : Composants React (pages, formulaires, dialogs)
- `hooks/` : Hooks React pour la gestion des données (avec TanStack Query)
- `stores/` : Stores Zustand pour l'état UI local (formulaires, modales, sélections)
- `locales/` : Traductions i18n (optionnel)

**Exemples** :
- `use-login.ts` : Hook pour l'authentification avec mutation TanStack Query
- `LoginForm.tsx` : Formulaire de connexion
- `usePipelineDashboardStore.ts` : Store Zustand pour la sélection de pipelines

**Règles** :
- Les composants n'appellent **jamais** directement les repositories ou data sources
- Les hooks encapsulent les appels aux use cases via TanStack Query
- Les stores Zustand gèrent uniquement l'état UI éphémère

### 3.2 Couche **Domain**

**Responsabilité** : Logique métier pure, indépendante de toute technologie

**Contenu** :
- `usecases/` : Classes de cas d'usage (orchestration de la logique métier)
- `repository/` : Interfaces des repositories (contrats abstraits)
- `models/` : Modèles métier (optionnel)

**Exemples** :
- `login.use-case.ts` : Cas d'usage pour la connexion
- `GetPipelinesUseCase.ts` : Cas d'usage pour récupérer les pipelines
- `login.repository.ts` : Interface du repository de login

**Règles** :
- **Aucune dépendance** vers les couches externes (UI, API, gRPC)
- Les use cases dépendent uniquement d'**interfaces** de repositories
- Retourne toujours des `Promise<ScyllaResult<T>>`

### 3.3 Couche **Infrastructure**

**Responsabilité** : Implémentation concrète des abstractions du domain

**Contenu** :
- `repository/` : Implémentation des repositories
  - `data-sources/` : Interfaces des sources de données (RemoteDataSource, LocalDataSource)
  - Repository impl qui coordonne les data sources
- `data/` : Implémentation des data sources
  - `remote/` : Appels API/gRPC (RemoteDataSourceImpl)
  - `local/` : Stockage local si nécessaire (optionnel)

**Exemples** :
- `default-login.repository.ts` : Implémentation du LoginRepository
- `login-remote.data-source.ts` : Interface de la source de données remote
- `grpc-login-remote.data-source.ts` : Implémentation avec appels gRPC

**Règles** :
- Les repositories implémentent les interfaces du domain
- Les data sources effectuent les appels techniques (gRPC, REST, localStorage)
- Utilisation de `ScyllaResult.tryAsync` pour encapsuler les appels API

### 3.4 Couche **DI** (Dependency Injection)

**Responsabilité** : Wiring des dépendances et instanciation des objets

**Contenu** :
- `{Feature}Module.ts` : Factory pour créer et configurer les dépendances

**Exemple** :
```typescript
// login.module.ts
const loginRemoteDataSource = new GrpcLoginRemoteDataSource(
  CoreModule.data.grpcTransport
);
const loginRepository = new DefaultLoginRepository(loginRemoteDataSource);
const loginUseCase = new LoginUseCase(loginRepository);

export const LoginModule = {
  domain: { loginUseCase }
};
```

**Règles** :
- Chaque module expose son API via la propriété `domain`
- Les dépendances sont injectées manuellement (pas de container IoC)
- Le `CoreModule` est importé pour obtenir les services d'infrastructure

---

## 4. Flux de données

### 4.1 Flux classique (lecture de données)

```
┌─────────────────┐
│  UI Component   │
│  (ProjectList)  │
└────────┬────────┘
         │
         │ appel hook
         ▼
┌─────────────────┐
│   React Hook    │
│ (useProjects)   │  ← TanStack Query (cache, loading, refetch)
└────────┬────────┘
         │
         │ appel use case
         ▼
┌─────────────────┐
│   Use Case      │
│ (GetProjectsUseCase)   │  ← Logique métier pure
└────────┬────────┘
         │
         │ appel repository
         ▼
┌─────────────────┐
│   Repository    │
│   Interface     │  ← Abstraction
└────────┬────────┘
         │
         │ implémentation
         ▼
┌─────────────────┐
│   Repository    │
│     Impl        │  ← Orchestration des data sources
└────────┬────────┘
         │
         │ appel data source
         ▼
┌─────────────────┐
│  RemoteData     │
│   SourceImpl    │  ← Appels gRPC/API
└────────┬────────┘
         │
         │ retour ScyllaResult<T>
         ▼
```

### 4.2 Exemple concret (Login)

1. **UserModel clique sur "Login"** → `LoginForm.tsx`
2. **Formulaire appelle** → `useLogin()` hook
3. **Hook déclenche mutation** → `deps.login.loginUseCase.execute()`
4. **Use case appelle** → `loginRepository.login()`
5. **Repository appelle** → `loginRemoteDataSource.login()`
6. **Data source effectue** → Appel gRPC via `AuthServiceClient`
7. **Retour** → `ScyllaResult<void>`
8. **Hook unwrap** → `result.unwrap()` (throw si erreur)
9. **TanStack Query gère** → Success/error callbacks
10. **UI met à jour** → Navigation vers /user-settings

### 4.3 Gestion de l'état global (Context)

L'application utilise un store Zustand partagé pour le contexte global :

```typescript
// shared/presentation/stores/use-context.store.ts
useContextStore = {
  organization: { id, name },
  project: { id, name },
  setOrganization(id, name),
  setProject(id, name)
}
```

**Utilisé par** :
- **Layout** : ContextSelector pour afficher et changer le contexte
- **Features** : Organization et Project lists pour mettre à jour le contexte
- **Hooks** : Récupération du contexte pour filtrer les données

---

## 5. Injection de dépendances

### 5.1 Structure

```
CoreModule (infrastructure partagée)
    └── grpcTransport: GrpcTransport

LoginModule, OrganizationModule, ProjectModule, etc.
    ├── remoteDataSource (utilise CoreModule.grpcTransport)
    ├── repository (utilise remoteDataSource)
    └── useCases (utilisent repository)

Dependencies (agrégation)
    └── Expose tous les modules via leur API domain
```

### 5.2 Utilisation dans les composants

```typescript
// Hook utilisant les dépendances
const deps = useDependencies(); // Via Context
const result = await deps.login.loginUseCase.execute(login, password);
```

### 5.3 Configuration

- `DependenciesProvider` wrap l'application dans `App.tsx`
- `DependenciesContext` expose l'objet `dependencies`
- Chaque module est instancié une seule fois au démarrage

---

## 6. Routing et Layout

### 6.1 CoreRouter

Configuration centralisée dans `Core.router.tsx` :
- Routes publiques (`/login`)
- Routes protégées (wrappées par `AuthGuard`)
- Routes avec layout (wrappées par `Layout`)
- Configuration via `handle` (topbar, tabs)

### 6.2 Layout

Composant `Layout.tsx` :
- Sidebar (avec ContextSelector)
- TopBar dynamique (via route handle)
- Outlet pour le contenu de la page
- Gestion des tabs (optionnel par route)

### 6.3 Context Selector

Système générique pour sélectionner Organisation/Project :
- **CurrentContextDisplay** : Affichage du contexte actuel
- **ContextItem** : Item de liste dans le dropdown
- **ContextSelector** : Composant générique qui combine tout
- **OrganizationList / ProjectList** : Listes spécifiques aux features

---

## 7. Bonnes pratiques

### 7.1 Règles générales

✅ **À FAIRE** :
- Utiliser `ScyllaResult` pour toutes les opérations asynchrones
- Les use cases retournent `Promise<ScyllaResult<T>>`
- Les hooks unwrap les résultats avec `.unwrap()`
- TanStack Query gère le cache et les erreurs
- Les stores Zustand pour l'état UI local uniquement
- Les interfaces dans le domain, implémentations dans infrastructure

❌ **À ÉVITER** :
- Appeler directement les repositories depuis les composants
- Mettre de la logique métier dans les hooks ou composants
- Utiliser les stores Zustand pour de la donnée backend
- Coupler le domain à React, gRPC, ou toute techno externe

### 7.2 Conventions de nommage

- **Use Cases** : Verbe à l'infinitif (`GetProjectsUseCase`, `CreatePipeline`)
- **Repositories** : `{Feature}Repository` (interface) et `{Feature}RepositoryImpl`
- **Data Sources** : `{Feature}RemoteDataSource` / `{Feature}RemoteDataSourceImpl`
- **Hooks** : `use{Action}` (`useLogin`, `useProjects`, `useCreateProject`)
- **Stores** : `use{Feature}Store` (`usePipelineDashboardStore`, `useScriptStore`)
- **Modules** : `{Feature}Module` (`LoginModule`, `ProjectModule`)

### 7.3 Organisation des fichiers

```
feature/
├── di/
│   └── FeatureModule.ts
├── domain/
│   ├── usecases/
│   │   └── GetData.ts
│   ├── repository/
│   │   └── FeatureRepository.ts
│   └── models/ (optionnel)
├── infrastructure/
│   ├── repository/
│   │   ├── FeatureRepositoryImpl.ts
│   │   └── data-sources/
│   │       └── FeatureRemoteDataSource.ts
│   └── data/
│       └── remote/
│           └── FeatureRemoteDataSourceImpl.ts
└── presentation/
    ├── hooks/
    │   └── useFeature.ts
    ├── stores/
    │   └── useFeatureStore.ts
    └── ui/
        └── FeaturePage.tsx
```

---

## 8. Technologies utilisées

- **React 18** : UI library
- **TypeScript** : Type safety
- **TanStack Query** : Gestion des données asynchrones, cache
- **Zustand** : State management local
- **React Router** : Routing
- **Lingui** : Internationalisation (i18n)
- **gRPC-Web** : Communication backend
- **shadcn/ui** : Composants UI
- **Vite** : Build tool

---

## 9. Points d'attention


### 9.3 Gestion du contexte

Le contexte Organisation/Project est géré de manière centralisée dans `shared/` :
- Store global dans `use-context.store.ts`
- Les features l'utilisent mais ne le possèdent pas
---

