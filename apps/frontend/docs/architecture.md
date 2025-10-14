# Architecture

This document describes the structure and architectural choices of our React frontend, based on **Clean Architecture** and using **TanStack Query**.

## 1. Architectural Choices

### 1.1 Clean Architecture
For the frontend of Scylla, we use the **Clean Architecture** concept.  
It allows each module to be split into several layers:

- **Presentation**: Contains all code related to the UI (components, pages, hooks).
- **Domain**: Contains the business logic and use cases.
- **Repository**: Acts as a bridge between the data layer and the domain layer, mapping raw data from the data layer into business types.
- **Data**: Contains stores and gateways to fetch data from local, remote, or other sources.

## 2. Folder structure
This is the modular folder structure of the application, following a **Clean Architecture** approach.
Each module is in the `src/modules/` folder.

### `presentation/`
- Contains everything related to **UI** and **local state management**.
- `ui/` → React components.
- `hooks/` → Hooks specific to the module’s presentation layer.
- `stores/` → Zustand store managing UI state (e.g., forms, modals).

### `domain/`
- Pure **business logic**, independent of UI and data layers.
- `useCases/` → Classes orchestrating business operations.
- `repository/` → Abstract interfaces for data access (API, local storage, etc.).

### `repository/`
- Concrete implementation of the repositories defined in `domain/repository`.
- `stores/` → Interfaces for stores used in this layer.

### `data/remote/`
- Handles **API calls** and remote data sources.
- Can be extended with `data/local/` if local data storage is needed (e.g., AsyncStorage, IndexedDB).


## 3. Recommended call flow
This section describes the **recommended data and action flow** in the app, following a modular and clean architecture approach.

### 3.1 Step-by-Step Flow

1. **UI Component → Hook (TanStack Query)**
    - React components **do not call the domain or repository directly**.
    - They use a **hook** (integrated with TanStack Query) to fetch or mutate data.
    - The hook handles **caching, loading state, and re-fetching** automatically.

2. **Hook → Domain Use Case**
    - The hook calls a **use case** in the domain layer.
    - The use case contains the **business logic** for the operation (e.g., `CreateUser`, `FetchEvents`).
    - This ensures **all rules and validations** are applied before reaching the data layer.

3. **Use Case → Repository Interface**
    - The use case interacts with a **repository interface**, which abstracts the data access method.
    - This keeps the **domain layer independent** from the actual data source (API, database, local storage).

4. **Repository → Data Layer (Remote / Local)**
    - The repository implementation calls the **data layer**, which performs the actual **API calls, database queries, or local storage operations**.
    - The data is returned back up through the repository, use case, hook, and finally to the UI component.

### 3.2 Visual Flow

React Component
│
▼
Hook (TanStack Query)
│
▼
Domain Use Case
│
▼
Repository Interface
│
▼
Data Layer

### 3.3 Key Principles

- **UI components never communicate directly with the data layer.**
- **All business logic resides in the domain layer.**
- **Repositories act as a bridge** between the domain and data layers.
- **Hooks encapsulate data fetching and state management**, keeping UI components clean and simple.