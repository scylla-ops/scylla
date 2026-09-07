# Plan de tests — frontend Scylla

Stratégie de test pour `apps/frontend`, écrite le **2026-08-25** sur la branche
`feat/frontend/roles`.

Objectif posé : **app moderne, construite vite, scalable, de façon très
pragmatique, en sautant les étapes inutiles.** Ce dossier assume cet objectif —
il dit autant ce qu'il faut *ne pas* faire que ce qu'il faut faire.

Aucun code n'a été modifié.

---

## Le verdict en cinq lignes

Il y a **0 test et 0 test runner** dans le frontend (243 tests côté Rust).
Il n'y a **aucune CI** qui exécute quoi que ce soit : le seul workflow GitHub
publie la doc mdBook. C'est pour ça que `pnpm typecheck` est cassé depuis
plusieurs commits sans que personne ne le voie.

**Le meilleur investissement n'est donc pas un test : c'est une CI de 20 lignes.**
Elle attrape aujourd'hui plus de bugs que les 50 premiers tests unitaires réunis.
Ensuite seulement, des tests — mais peu, et bien choisis.

---

## La stratégie en une image

```
        ~5 tests E2E Playwright          ← les parcours qui rapportent de l'argent
       ────────────────────────────         (login, créer+lancer une pipeline)
                  │
     ~15 tests d'intégration RTL         ← les écrans à état complexe
    ──────────────────────────────────      (gating, formulaire de rôle)
                  │
     ~40 tests unitaires Vitest          ← la logique pure, sans React
  ───────────────────────────────────       (canAccess, ScyllaResult, mappers)
                  │
   ══════════════════════════════════    ← LA BASE : tsc + eslint en CI
        typecheck + lint (CI)               gratuit, déjà écrit, non branché
```

**~60 tests au total, pas 600.** Sur 435 fichiers source, viser une couverture
large serait de la dette déguisée en qualité. On teste ce qui casse en silence,
pas ce qui casse bruyamment.

---

## Deux bonnes nouvelles trouvées dans le code

L'architecture rend le testing **beaucoup** moins cher que dans une app React
typique. Deux points d'injection existent déjà, gratuitement :

1. **`DependenciesContext`** (`core/presentation/contexts/dependencies.context.ts`) —
   tous les use cases passent par un contexte React. Tester un composant = lui
   fournir un objet `Dependencies` factice. **Aucune librairie de mock
   nécessaire**, pas de `vi.mock` sur des chemins de fichiers.

2. **`TestTransport`** de `@protobuf-ts/runtime-rpc` — **déjà une dépendance
   directe** du projet. Elle permet de tester la chaîne complète
   *data source → mapper → repository* contre un faux transport gRPC.
   **Pas besoin de MSW**, pas de serveur de test, pas de mock manuel.

Ces deux seams sont la raison pour laquelle ce plan tient en ~60 tests.

---

## Table des fichiers

| Fichier | Contenu |
|---|---|
| [01-etat-des-lieux.md](01-etat-des-lieux.md) | Ce qui existe, ce qui manque, ce qui est testable et à quel coût |
| [02-strategie.md](02-strategie.md) | La pyramide adaptée à *cette* app — et **la liste de ce qu'il ne faut pas tester** |
| [03-outillage-et-setup.md](03-outillage-et-setup.md) | Vitest + RTL + Playwright : quoi installer, config copiable, pièges jsdom/Radix |
| [04-quoi-tester.md](04-quoi-tester.md) | Les cibles précises, par couche, avec les fichiers réels du repo |
| [05-exemples-copiables.md](05-exemples-copiables.md) | 6 tests modèles à copier-coller (un par catégorie) |
| [06-e2e.md](06-e2e.md) | Playwright contre le stack docker-compose, auth en 1 fois, 5 parcours |
| [07-ci.md](07-ci.md) | Le workflow GitHub Actions manquant |
| [08-plan-action.md](08-plan-action.md) | Découpage en 4 demi-journées |

---

## Ce que ce plan refuse explicitement

| Refusé | Pourquoi |
|---|---|
| **MSW** | Le transport est gRPC-Web **binaire**. `TestTransport` est plus simple, plus rapide, typé. |
| **Storybook** | Coût de maintenance élevé, valeur faible sur une app interne sans design system propre (shadcn fait déjà foi). |
| **Tests de snapshot** | Ils cassent à chaque refacto de classe Tailwind et ne détectent aucun bug réel. |
| **Seuil de couverture en CI** | Transforme l'écriture de tests en jeu de chiffres. On mesure, on ne bloque pas. |
| **Tester les 31 composants `shadcn/`** | Code vendored, testé en amont. |
| **Tester les 65 use cases** | La plupart sont des passe-plats d'une ligne vers le repository. Voir [04](04-quoi-tester.md). |
| **Tester les `*.module.ts` (DI)** | Ce sont des `new X(y)`. Si le DI est faux, le typecheck le dit. |
| **E2E sur chaque écran** | Lent, instable, cher. 5 parcours critiques suffisent. |
| **Atteindre 80 % de couverture** | Objectif inventé. La bonne question : « ce bug serait-il passé ? » |
