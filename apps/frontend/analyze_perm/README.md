# Analyse — gestion des permissions (frontend Scylla)

Analyse du module `src/modules/features/permission/` et de tous ses consommateurs,
faite le **2026-08-25** sur la branche `feat/frontend/roles` (HEAD `1b256839`).

Aucun code n'a été modifié. Les références sont au format `chemin:ligne`.

Les contrats backend ont été vérifiés directement dans le monorepo
(`crates/scylla-proto/proto/scylla/authz/v1/*.proto` et
`crates/scylla-auth/src/authz/`), donc les écarts
front/back signalés ici sont factuels, pas supposés.

---

## Résumé exécutif

L'architecture d'autorisation est **bien pensée** : séparation domaine/infra/présentation
respectée, `canAccess` pur et testable, primitives UI cohérentes
(`Can` / `RequirePermission` / `PermissionButton`), gestion explicite des arms
`unknown` dans les mappers, permissions jamais persistées dans `localStorage`.
Le socle est solide.

Mais **la chaîne est cassée à trois endroits qui, cumulés, rendent le système
inutilisable pour tout utilisateur qui n'est pas system-admin** :

1. Le seul appel qui alimente `can()` utilise le mauvais RPC — un RPC réservé aux
   system-admins. Pour tout le monde d'autre : `{ scopes: [] }`, donc application
   entièrement verrouillée.
2. Le catalogue de permissions de l'éditeur de rôles est écrit à la main et ne
   couvre que **20 permissions sélectionnables sur 59**. La majorité des permissions
   sur lesquelles l'app fait ses gates ne sont donc **pas attribuables** via l'UI.
3. `pnpm typecheck` ne passe pas (5 erreurs), dont 3 dans le périmètre permissions.

---

## Table des fichiers

| Fichier | Contenu |
|---|---|
| [01-bugs-bloquants.md](01-bugs-bloquants.md) | P0 — ce qui casse le système aujourd'hui |
| [02-catalogue-permissions.md](02-catalogue-permissions.md) | Le catalogue codé en dur vs. le vocabulaire backend, matrice de couverture |
| [03-semantique-autorisation.md](03-semantique-autorisation.md) | `canAccess` : faux positifs / faux négatifs, fraîcheur, scopes |
| [04-couverture-ui.md](04-couverture-ui.md) | Où le gating est appliqué, où il manque, feature par feature |
| [05-fonctionnalites-manquantes.md](05-fonctionnalites-manquantes.md) | Code mort, RPC non câblés, écrans absents |
| [06-conventions-architecture.md](06-conventions-architecture.md) | Écarts vs. CLAUDE.md (nommage, structs/entities, query keys) |
| [07-i18n.md](07-i18n.md) | Le module entier échappe à Lingui |
| [08-qualite-bugs-mineurs.md](08-qualite-bugs-mineurs.md) | Bugs UI, états de chargement, robustesse |
| [09-plan-action.md](09-plan-action.md) | Ordre de traitement recommandé |

---

## Tableau de synthèse

| # | Constat | Gravité | Fichier détaillé |
|---|---|---|---|
| 1 | `useRefreshMyPermissions` appelle `getEffectivePermissions` (system-admin) au lieu de `getMyPermissions` | 🔴 Bloquant | [01](01-bugs-bloquants.md) |
| 2 | `use-my-permissions.ts` ne compile pas (`useDependencies().authz` n'existe pas) | 🔴 Bloquant | [01](01-bugs-bloquants.md) |
| 3 | `pnpm typecheck` KO — 5 erreurs, enum domaine désynchronisé du proto | 🔴 Bloquant | [01](01-bugs-bloquants.md) |
| 4 | 39 permissions sur 59 non attribuables depuis l'éditeur de rôles | 🔴 Bloquant fonctionnel | [02](02-catalogue-permissions.md) |
| 5 | `usePermissionVocabulary` (source de vérité backend) implémenté mais jamais branché | 🟠 Majeur | [02](02-catalogue-permissions.md) |
| 6 | `fullControl` à un scope étroit confère les permissions système côté client | 🟠 Majeur | [03](03-semantique-autorisation.md) |
| 7 | Aucun gating sur secrets / projets / users / agents / triggers / apps | 🟠 Majeur | [04](04-couverture-ui.md) |
| 8 | Module `permission` absent de `lingui.config.js` → 0 chaîne traduite | 🟠 Majeur | [07](07-i18n.md) |
| 9 | Pas d'écran d'accès par org/projet, pas de grant vers une App, `RevokeAllAccess` non câblé | 🟡 Moyen | [05](05-fonctionnalites-manquantes.md) |
| 10 | `RoleFormDialog` : `updateRole.reset()` après une création → dialog gelé | 🟡 Moyen | [08](08-qualite-bugs-mineurs.md) |
| 11 | Révocation d'un grant non gardée par permission, sans confirmation | 🟡 Moyen | [08](08-qualite-bugs-mineurs.md) |
| 12 | `Roles.page` ignore `isLoading` / `isError` | 🟡 Moyen | [08](08-qualite-bugs-mineurs.md) |
| 13 | Écarts de conventions vs. CLAUDE.md (structs/entities, query keys) | 🔵 Mineur | [06](06-conventions-architecture.md) |
