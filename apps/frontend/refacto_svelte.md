# Migration React → Svelte 5

Migration de la couche `presentation/` de Scylla Frontend, de React 18 vers **Svelte 5 + Vite**.
Objectif final : plus une ligne de React, et une surface de dépendances divisée par deux.

> Ce document est le contrat de la migration. Il complète `CLAUDE.md`, il ne le remplace pas :
> les règles d'architecture (4 couches, barrels, DI, `ScyllaModule`, i18n, tests) restent
> intégralement en vigueur pendant et après.

---

## 0. La décision, et ses raisons

**Décidé : on migre. Svelte 5 + Vite. Pas SvelteKit.**

### Pourquoi

1. **La surface de dépendances.** 41 dépendances runtime déclarées, **861 paquets npm transitifs**.
   Scylla est une plateforme CI/CD : la supply chain npm est le modèle de menace de sa propre
   catégorie. Le backend Rust a un `deny.toml` ; le frontend n'a rien d'équivalent parce que le
   nombre est ingérable. Cible après migration : **~400 paquets**. C'est le gain principal, il est
   durable, et il compose avec le temps.

2. **La cohérence du projet.** Un binaire, un port, pas de broker, pas de hostname en dur, l'UI
   embarquée par `rust-embed`. Scylla paie à la compilation, pas à l'exécution — c'est l'identité
   du projet, côté Rust comme côté déploiement. Svelte compile et disparaît ; React expédie 77 kB
   de framework au navigateur à chaque chargement. Svelte est le choix cohérent.

3. **La fenêtre est ouverte maintenant, et elle se referme.** La Clean Architecture fait que
   **60 % du code ne bouge pas** : `domain/`, `infrastructure/`, mappers, repositories, `ScyllaResult`,
   proto. C'est une situation rare — la plupart des migrations de framework échouent parce que la
   logique métier est enchevêtrée dans les composants. Ici elle ne l'est pas. Chaque mois qui passe,
   la couche présentation grossit et la facture monte.

### Ce que la migration n'apportera **pas**

**Le virtual DOM n'est pas le problème de Scylla.** Pour un dashboard CI affichant des tableaux de
quelques centaines de lignes pilotés par TanStack Query, le coût du vDOM est imperceptible. Ce qui
détermine la fluidité perçue, c'est la latence du streaming de logs, le rafraîchissement des jobs
et le débit gRPC-Web — du backend et du réseau, rien que la migration ne touche.

Attendre un gain de performance ressentie de cette migration, c'est se préparer une déception à
l'arrivée. Le gain est ailleurs : dépendances, cohérence, maintenabilité, et un bundle plus léger
au boot (mesuré en §2).

### Pourquoi pas SvelteKit

`crates/scylla-core/build.rs` embarque `apps/frontend/dist` dans le binaire via `rust-embed`,
servi en `fallback_service`. **Il n'y a jamais de Node à l'exécution.**

SvelteKit devrait donc tourner en `adapter-static` + fallback SPA, mode dans lequel SSR, routes
serveur, form actions, `+page.server.ts` et `hooks.server` sont tous inutilisables — c'est-à-dire
tout ce pour quoi SvelteKit existe. Il ne resterait que le routing par fichiers, qui entre en
collision frontale avec `ScyllaModule.routes` : le contrat sur lequel repose `compose-module-routes.ts`,
la déclaration unique de `permission` alimentant le guard *et* la sidebar, et
`module-permissions.test.ts` qui vérifie tout ça en lisant `registry.ts`.

Adopter SvelteKit reviendrait à ajouter une dépendance-framework pour zéro bénéfice, en démontant
au passage le meilleur mécanisme du codebase. **Svelte + Vite**, avec le routeur dérivé des modules.

### Le rythme : migration opportuniste

**650 fichiers `presentation/` uniques ont été touchés ces 6 derniers mois, pour 445 fichiers
existants.** La couche présentation est intégralement réécrite ~1,5 fois par semestre.

Conséquence directe : après les phases 0 et 1, **il n'y a pas de chantier séparé qui concurrence
les features**. On migre chaque module au moment où on l'ouvre déjà pour y travailler. La migration
voyage avec le travail normal et se termine en 2-3 trimestres sans bloquer une seule release.

Les phases 2 à 5 ci-dessous décrivent donc un **ordre de priorité**, pas un planning bloquant.

---

## 1. Périmètre

### Ne bouge pas (≈ 60 % du code)

`domain/` et `infrastructure/` de chaque feature, `platform/grpc`, `shared/domain`,
`shared/infrastructure`, `shared/utils`, mappers, `ScyllaResult`, repositories, proto généré.
Ce code n'a aucune dépendance framework — c'est le dividende de la Clean Architecture.

`platform/di`, `platform/authz`, `platform/context` gardent leur **logique** ; seule leur
enveloppe React (context, provider, hook) est remplacée — Phase 0.

### Bouge (mesuré)

| Module | fichiers presentation | LOC | tests | dépendances lourdes |
|---|---|---|---|---|
| `shared` | 76 | 5 961 | 31 | shadcn/Radix, react-table, framer-motion, sonner, codemirror |
| `pipeline` | 37 | 2 755 | 13 | **reactflow**, codemirror, react-table |
| `roles` | 26 | 2 463 | 19 | CheckboxTree / matrice de permissions |
| `jobs` | 24 | 1 671 | 11 | codemirror (logs), react-table |
| `triggers` | 22 | 1 720 | 12 | react-table |
| `membership` | 14 | 1 491 | 12 | — |
| `agents` | 12 | 1 455 | 5 | — |
| `layout` | 14 | 1 051 | 2 | framer-motion, react-router |
| `apps` | 8 | 850 | 4 | — |
| `dashboard` | 4 | 768 | 1 | **recharts** |
| `project` | 14 | 617 | 4 | — |
| `user` | 12 | 560 | 3 | react-table |
| `secret` | 11 | 507 | 4 | react-table |
| `organization` | 11 | 475 | 4 | framer-motion |
| `core` | 6 | 335 | 5 | react-router (shell) |
| `platform/authz` | 6 | 226 | 4 | — |
| `marketplace` | 7 | 144 | 1 | — |
| `login` | 4 | 134 | 2 | — |
| **Total** | **~308** | **~23 200** | **~137** | |

---

## 2. Le bundle : mesures réelles

Build du `main` actuel, gzip :

| Chunk | gzip | Après migration |
|---|---|---|
| `vendor-codemirror` | **137 kB** | ≈ inchangé — CodeMirror 6 est agnostique |
| `vendor-charts` (recharts + d3) | **119 kB** | **−119 kB**, et supprimable *sans* Svelte |
| `vendor-react` | 77 kB | −65 kB (runtime Svelte ≈ 12 kB) |
| `vendor-ui` (Radix + lucide + sonner) | 69 kB | −20 kB (bits-ui ≈ Radix en poids) |
| `index` (code applicatif) | 53 kB | −15 kB environ |
| `vendor-motion` (framer-motion) | 41 kB | **−41 kB**, gain pur |
| `vendor-flow` (reactflow) | 29 kB | ≈ inchangé |
| `vendor-query` | 24 kB | −4 kB (`query-core` partagé) |
| `vendor-grpc`, `vendor-i18n`, locales, CSS | 58 kB | inchangé |

**Avant Lot A : 294,8 kB gzip au chargement initial, 660 kB au total.**
**Après Lot A (mesuré) : 253,6 kB initial, 619 kB total.**

Deux enseignements à garder en tête :

- **Les deux plus gros postes ne sont pas React.** CodeMirror survit à la migration ; recharts peut
  être supprimé cette semaine sans toucher au framework. D'où le **Lot A** de la Phase 0.
- **Après Lot A** : 253,6 kB initial / 619 kB total — `recharts` et `@uiw/react-codemirror`
  ayant été déplacés en phases 4 et 1 pour ne pas écrire deux fois le même composant.
  **Après migration complète** : ~170 kB initial / ~380 kB total.

Le bundle est un bénéfice réel mais secondaire. Le bénéfice principal reste les 461 paquets npm
en moins.

---

## 3. Stratégie : React hôte, Svelte en îlots, bascule du shell en dernier

**Le problème.** Un composant Svelte monté dans un arbre React ne voit **aucun contexte React**.
Or aujourd'hui tout passe par du contexte : `QueryClientProvider`, `DependenciesProvider`,
`I18nProvider`, `ThemeProvider`, le router. Sans préparation, chaque îlot Svelte doit se faire
re-câbler ces cinq choses à la main — on l'écrirait 14 fois.

**La solution.** Avant de migrer la moindre UI, on **dé-React-ifie la plomberie** : QueryClient,
registre DI, i18n, thème et stores deviennent des **singletons de module**, importables de partout.
React et Svelte lisent alors la *même* instance, sans pont. Après ça, un composant Svelte n'a besoin
que d'un `import`, et le wrapper d'îlot se réduit à « monte ce composant dans cette div ».

Ce n'est pas un détour : c'est un nettoyage qui a de la valeur même si la migration s'arrêtait là.

**Pourquoi pas l'inverse (Svelte hôte, îlots React).** 24 fichiers de features utilisent
`useNavigate` / `useParams` / `useLocation`. Une page React montée sous un routeur Svelte n'a plus
de `RouterProvider` : il faudrait simuler react-router et le synchroniser avec l'historique. On
garde donc react-router jusqu'au bout, et on le remplace **une seule fois**, à la fin, quand plus
aucune page React ne l'utilise.

**Sens de migration dans un module : feuilles → racine.** Composants présentationnels, puis
conteneurs, puis la page, puis on retire l'îlot. Un composant Svelte peut contenir du Svelte ; il ne
peut pas contenir du React. On ne migre jamais un parent avant ses enfants.

---

## 4. Décisions techniques

### 4.1 Le routeur — **décidé : maison (~300 LOC)**

SvelteKit est écarté (§0). Les micro-routeurs de l'écosystème (`svelte-spa-router`, `svelte-routing`)
ne gèrent pas correctement layouts imbriqués, `lazy` et métadonnées de route — or `RouteGuard` et
les breadcrumbs en dépendent.

La surface react-router réellement utilisée est étroite : `useNavigate` (45), `useParams` (35),
`Outlet` (16), `useLocation` (14), `Navigate` (8), `useMatches` (5). Un matcher de chemins + un
`<Outlet>` imbriqué + un loader `lazy` + la propagation de `handle`, c'est ~300 lignes testables,
qui vivent dans `@platform/routing` aux côtés de `compose-module-routes.ts`. Et la Phase 0 interdit
aux features d'importer react-router directement : au moment de la bascule, il n'y a **qu'un seul
endroit** à changer.

### 4.2 Correspondance des dépendances

| Aujourd'hui | Demain | Note |
|---|---|---|
| `react`, `react-dom` | `svelte` | |
| `react-router-dom` (45 fichiers) | `@platform/routing` maison | −1 dép |
| `@tanstack/react-query` (49) | `@tanstack/svelte-query` | **même `query-core`, même `QueryClient`, cache partagé** |
| `zustand` (7) | runes (`$state` en module) | −1 dép |
| `framer-motion` (5) | `transition:` / `animate:` / `crossfade` natifs | −1 dép, −41 kB |
| `next-themes` (6) | ~25 lignes maison | −1 dép |
| `sonner` (24 fichiers, **1 seul point d'entrée** : `shared/presentation/utils/toast.ts`) | `svelte-sonner` | échange trivial |
| `lucide-react` (89) | `@lucide/svelte` | mapping 1:1, mécanique |
| `@radix-ui` + `radix-ui` (38 fichiers, 33 primitives shadcn) | `shadcn-svelte` (sur `bits-ui`) | **Tailwind et classes identiques → tout le style survit tel quel** |
| `@tanstack/react-table` (12) | `@tanstack/svelte-table` | `ColumnDef` vient de `table-core` : les défs survivent, seuls les `cell:` JSX sont réécrits |
| `@uiw/react-codemirror` (8) | CodeMirror 6 direct via une `action` Svelte | **−1 dép** (le wrapper React disparaît, pas CodeMirror) |
| `reactflow` (6) | `@xyflow/svelte` | port officiel, API proche |
| `recharts` (2) | SVG maison | **−119 kB** ; un seul graphe concerné |

Cible : **41 → ~20 dépendances runtime** (39 après Lot A), **861 → ~400 paquets transitifs**.
`vendor-react`, `vendor-motion` et `vendor-charts` disparaissent des `VENDOR_CHUNKS`.

### 4.3 Points durs

1. **`reactflow` → `@xyflow/svelte`** (`pipeline`). API proche mais pas identique (nodes/edges en
   stores, `$props` pour les custom nodes). `blueprint-converter.ts` est du code pur et survit avec
   ses tests — c'est le filet de sécurité. Les 4 composants de canvas sont une réécriture. Morceau
   le plus long de la migration.
2. **`dependency-cruiser` ne parse pas `.svelte`.** Les six règles `error` qui protègent les barrels
   et le sens des couches deviendraient **aveugles** sur tout le code neuf. Traité en Phase 0, §4.5.
3. **Lingui n'extrait pas depuis `.svelte`.** Voir §4.4.
4. **`recharts`** n'a aucun portage Svelte. Décision prise : SVG maison (un seul graphe), ce qui
   sort aussi `d3-*` et `victory-vendor`. Repli si ça dérape : `LayerChart`.

### 4.4 i18n — la règle à ne pas rater

`@lingui/core` est déjà agnostique ; ce sont `@lingui/react` (`<Trans>`, `useLingui`) et le plugin
SWC qui ne le sont pas. `lingui extract` ne sait pas lire un `.svelte`, et ni `pnpm i18n:collisions`
ni les seuils de couverture ne le verraient passer : **les traductions disparaîtraient en silence**.

**Règle pour tout composant Svelte** : les messages sont déclarés avec la macro `msg` dans un
fichier `.ts` voisin, que Lingui extrait normalement. Le `.svelte` ne fait que les référencer.

```ts
// Secret.messages.ts — extrait par `lingui extract` comme aujourd'hui
import { msg } from '@lingui/core/macro';
export const secretMessages = {
  title: msg`Secrets`,
  empty: msg`No secret yet`,
};
```

```svelte
<!-- Secret.page.svelte -->
<script lang="ts">
  import { t } from '@shared/presentation/utils/i18n-svelte.ts'; // réactif au changement de locale
  import { secretMessages } from './Secret.messages.ts';
</script>
<h1>{t(secretMessages.title)}</h1>
```

Contrainte annexe : **après tout déplacement de composant entre modules**,
`node scripts/restore-translations.mjs`. La règle existante s'applique telle quelle, et la migration
déplace beaucoup de fichiers. À lancer en fin de chaque phase, avec `--dry-run` pour vérifier.

### 4.5 Les garde-fous doivent survivre

Aucune phase n'est terminée si un gate est désactivé « le temps de la migration ».

- **`depcruise`** : `enhancedResolveOptions.extensions` accepte `.svelte`, avec pré-traitement
  `svelte2tsx`. Si ça résiste : **plan B obligatoire** — `eslint-plugin-svelte` + une règle
  `no-restricted-imports` qui rejoue les six règles de barrel (`feature-api-only`,
  `platform-api-only`, `module-declaration-is-private`, `domain-accessor-is-private`, …).
  La protection ne baisse pas d'un cran.
- **`module-permissions.test.ts`** et **`feature-permissions.test.ts`** : ils lisent
  `core/di/registry.ts` et le source de `presentation/ui/`. Ils continuent de fonctionner à
  condition que `ScyllaModule.routes[].lazy` garde sa forme et que leurs globs incluent `.svelte`.
  **Vérifié en Phase 0, pas après.**
- **Couverture** : `coverage.include` passe à `src/modules/**/*.{ts,tsx,svelte}`. Les seuils sont un
  cliquet : ils ne baissent jamais, même temporairement. Un module migré rend ses tests, sinon il
  n'est pas migré.
- **`i18n:collisions`** : zéro à chaque phase.

---

## 5. Les phases

Chaque phase se termine par `pnpm typecheck && pnpm test && pnpm lint && pnpm depcruise &&
pnpm depcruise:cycles && pnpm i18n:collisions` **verts**, plus `pnpm build` et une passe manuelle
sur les écrans touchés.

Les phases 0 et 1 sont séquentielles et bloquantes. **Les phases 2 à 5 sont un ordre de priorité**,
pas un planning : on migre un module quand on l'ouvre pour autre chose (§0, migration opportuniste).

---

### Phase 0 — Alléger, puis dé-React-ifier

Deux lots indépendants. **Le Lot A est livrable seul et se justifie même si la migration
s'arrêtait là.**

#### Lot A — Nettoyage des dépendances ✅ **fait**

Règle appliquée pour choisir ce qui entre dans ce lot : **on n'écrit pas ici du code React qui
serait réécrit en Svelte trois phases plus loin.** Deux des quatre candidats initiaux sont donc
partis ailleurs.

| Dépendance | Sort | Gain |
|---|---|---|
| `framer-motion` (5 fichiers) | ✅ utilitaires `tw-animate-css` + un `@keyframes` | **−40,3 kB gzip** |
| `next-themes` (5 fichiers) | ✅ store agnostique maison (~60 lignes) | −1 kB, mais c'est du Lot B livré d'avance |
| `recharts` (2 fichiers) | → **Phase 4** | le chart SVG serait écrit deux fois |
| `@uiw/react-codemirror` (8 fichiers) | → **Phase 1** | le montage devient une `action` Svelte |

**Résultat mesuré : 294,8 → 253,6 kB initial, 660 → 619 kB total, −2 dépendances.**

Deux notes sur ce qui a été livré :

- **Les animations de sortie sont perdues** et ne reviendront pas : le CSS ne peut pas animer un
  nœud que React a déjà démonté. La navigation n'a plus son fade-out de 200 ms, ce qui la rend
  perçue comme plus rapide. Svelte, lui, sait faire des transitions de sortie (`out:`) — c'est
  récupérable en Phase 1 si le rendu manque.
- `prefers-reduced-motion` couvre désormais `.animate-in` et `[class*='animate-[']`, donc aussi
  `smooth-pulse` qui ne l'était pas. framer-motion ne le respectait pas non plus ici.

Le thème a été livré directement sous la forme visée par le Lot B — **store agnostique +
binding framework** — parce que c'est précisément ce que le point 5 ci-dessous demandait :

```
shared/presentation/stores/theme.store.ts   getTheme / setTheme / subscribeToTheme  (zéro React)
shared/presentation/hooks/use-theme.ts      useSyncExternalStore  (supprimé en Phase 6)
```

`subscribeToTheme` renvoie déjà la forme qu'attend le contrat de store Svelte. C'est le patron
que les stores 2 à 4 du Lot B suivent. Note de nommage : ce n'est ni un hook ni un store Zustand,
donc ni `use-*.ts` ni `use-*.store.ts` — `theme.store.ts` est une entrée nouvelle dans le tableau
des conventions, à reporter dans `CLAUDE.md` en Phase 6.

#### Lot B — Dé-React-ification de la plomberie

*Aucune UI migrée. C'est ce qui rend toutes les phases suivantes mécaniques.*

**Outillage** — `svelte` 5, `@sveltejs/vite-plugin-svelte`, `svelte-check`, `eslint-plugin-svelte`,
`@testing-library/svelte`. Plugin Svelte à côté du plugin React dans `vite.config.ts` (les deux
coexistent), `coverage.include` étendu à `.svelte`, alias `@/` inchangés, mêmes règles ESLint de
fond (`no-floating-promises`, `consistent-type-imports`).

**Le cœur :**

1. `QueryClient` sort de `App.tsx` vers `platform/query/client.ts` — singleton unique avec ses
   `QueryCache`/`MutationCache` et leurs handlers d'erreur globaux. React le reçoit via
   `QueryClientProvider`, Svelte via le contexte de `@tanstack/svelte-query`. **Même instance,
   même cache, mêmes handlers** : un module migré et un module non migré partagent leurs données.
2. `platform/di` : `dependencies` devient un singleton lisible directement (`getModuleDomain(id)`),
   `useModuleDomain` n'est plus qu'un wrapper React. **Point à concevoir avec soin** : l'injection
   doit rester substituable en test *sans* contexte React (un `setRegistry()` scopé, testé).
3. `platform/authz` : `can(permission, scope)` devient une fonction pure sur l'état du store ;
   `useCan` l'enveloppe. Le store des permissions reste la source de vérité unique — les tests
   continuent de le piloter via `usePermissionsStore.setState(...)`.
4. Stores Zustand (`use-context.store.ts`, `use-selection.store.ts`) : `createStore` vanilla +
   adaptateur Svelte (`subscribe` → readable, ~10 lignes). Zustand est supprimé en Phase 6.
5. i18n : `i18n-svelte.ts` (helper `t()` réactif) + la convention `*.messages.ts` de §4.4.
6. **Interdiction faite aux features d'importer `react-router` directement** : tout passe par
   `useScyllaNavigate` / `@platform/routing`, via `no-restricted-imports`. C'est ce qui rendra la
   Phase 6 petite.

**Le pont** — un seul fichier : `<SvelteIsland component={X} props={…} />`. Après les points 1-5,
il n'a **rien** d'autre à ponter.

**Garde-fous** — §4.5 traité intégralement, ici et pas plus tard.

**Critère de sortie** : un composant Svelte jetable, monté dans une page React, qui lit une query
TanStack existante, une traduction, une permission et le store de contexte — et les 6 gates verts.
Ce composant est ensuite supprimé.

---

### Phase 1 — `shared/` : le design system Svelte + le harnais de test

*76 fichiers, ~6 000 LOC. La plus grosse phase, la plus mécanique, et elle débloque tout le reste.*
**Bloquante : rien d'autre ne peut avancer avant.**

- Port `shadcn/ui` → **`shadcn-svelte`** pour les 33 primitives. Les classes Tailwind sont
  identiques : transposition de syntaxe, pas redesign. Radix → `bits-ui`.
- `DataTable` sur `@tanstack/svelte-table` ; `Pagination`, `FeatureHeader`, `IconButton`,
  `TruncatedText`, `CopyableText`, `ListCard`, `StatusBar`, `ErrorState`, dialogs.
- `ScyllaForm` / `FormDialog` / `useFormState` → version runes. **Le typage générique sur les ids
  d'items doit survivre** (`FormItem<'a'|'b'>` → `FormValues` typé) : c'est ce qui empêche de
  chercher les valeurs par id, et ça se perd facilement dans un port.
- `toast.ts` → `svelte-sonner` (un fichier).
- `AnimatedOutlet`, `ScyllaLoadingScreen` → transitions natives.
- CodeMirror : `use-code-mirror-theme.ts` + `code-mirror-theme.ts` → une `action` Svelte sur
  `EditorView`. **`@uiw/react-codemirror` sort ici** (déplacé du Lot A) : son rôle est le montage,
  qui est exactement ce qu'une `action` remplace. Les deux usages actuels prennent son `basicSetup`
  par défaut — autocomplétion, lint, recherche, historique — dont un visualiseur de logs en lecture
  seule n'a rien à faire : reconfigurer explicitement les extensions est un gain à chiffrer sur
  les 137 kB du chunk.
- Harnais de test : `src/test/render.svelte.ts` jumeau de `render.tsx` (`renderWithProviders`,
  `createTestQueryClient`, injection du registre DI). `setup.ts` est **partagé** — on ne re-stub
  jamais `ResizeObserver` & co. par fichier.

**Coût assumé** : `shared/` existe en double pendant les phases 2 à 5. Le `shared` React est
**gelé** — correctifs uniquement, aucune évolution. Toute nouveauté va dans la version Svelte.

**À partir d'ici, toute nouvelle feature s'écrit en Svelte.** La checklist « Adding a feature » de
`CLAUDE.md` est mise à jour dans cette phase.

---

### Phase 2 — Features pilotes : `login`, `marketplace`, `secret`, `user`, `project`, `organization`

*~2 400 LOC, 59 fichiers, aucune dépendance exotique.*

Six features indépendantes, six PRs, recette identique. C'est la phase qui **valide la recette à
l'échelle** — si quelque chose cloche dans le plan, ça se voit ici et pas au milieu de `pipeline`.

`login` en premier (134 LOC) : le plus petit chemin complet page + formulaire + mutation. Il sert
d'étalon — c'est lui qui dit si la recette de §6 tient.

---

### Phase 3 — Le gros bloc : `apps`, `agents`, `membership`, `jobs`, `triggers`

*~7 200 LOC, 80 fichiers. Même recette, en volume.*

- `jobs` embarque le **premier CodeMirror** (affichage de logs en lecture seule, via
  `use-streamed-log-view.ts`) : le cas le plus simple des deux, à faire ici pour dé-risquer la
  Phase 5. Le streaming est le point à tester sérieusement — c'est un système externe, donc une
  `action` Svelte, pas de la réactivité.
- `triggers`, `jobs` : tables sur `svelte-table` (les `ColumnDef` survivent, les `cell:` deviennent
  des snippets).

À l'issue de cette phase, **10 features sur 14 sont en Svelte**.

---

### Phase 4 — `roles` + `dashboard`

- **`roles`** (2 463 LOC, 19 tests) : la matrice de permissions et `CheckboxTree` sont la logique UI
  la plus dense du projet. **Porter les tests d'abord** — ils sont le cahier des charges.
- **`dashboard`** : **`recharts` sort ici** (déplacé du Lot A), remplacé par du SVG maison écrit
  directement en Svelte — **−119 kB gzip**, et sortent avec lui `d3-*` et `victory-vendor`.
  Faire porter la géométrie (échelles, courbes monotones, ticks) par un `.ts` pur testé en
  `@vitest-environment node` : c'est la partie qui survivrait à un changement de framework.
  Repli si ça dérape : `LayerChart`.

---

### Phase 5 — `pipeline`

*2 755 LOC, 37 fichiers. Le morceau le plus risqué, isolé volontairement.*

- `reactflow` → `@xyflow/svelte` : `BlueprintCanvas`, `PipelineStepNode`, `StartNode`,
  `DeletableEdge`, `use-blueprint-state.ts`. `blueprint-converter.ts` ne bouge pas et garde ses
  tests — c'est le filet.
- `PipelineEditor` / `StepNodeFormDialog` : 2ᵉ CodeMirror (édition), en réutilisant l'action écrite
  en Phase 1 et éprouvée en Phase 3.

**Critère de sortie** : `reactflow` sort du `package.json`.

---

### Phase 6 — Bascule du shell et suppression de React

*`layout` (1 051) + `core` (335) + `platform/authz` presentation (226). ~1 600 LOC, mais c'est la
phase qui rend le reste définitif.*

1. **Routeur maison** (§4.1) : `compose-module-routes.ts` réécrit contre lui, `RouteGuard`,
   `route-handle.struct.ts`, chargement `lazy`, `useMatches` pour les breadcrumbs. Les features ne
   l'appellent que via `@platform/routing` (règle posée en Phase 0), la surface est concentrée.
2. `layout` : `Layout`, `AppSidebar`, `NavMain`, `ScyllaBreadcrumbs`, `context-selector`.
3. `core` : `App`, `Core.router`, `Auth.guard`, les trois wrappers (`OrganizationSync`,
   `OrganizationRedirect`, `ContextCleaner`), `main.tsx`.
4. `platform/authz` : `Can`, `RequirePermission` en Svelte (`useCan` disparaît, `can()` reste).
5. **Suppression** : `react`, `react-dom`, `react-router-dom`, `@tanstack/react-query`,
   `@tanstack/react-table`, `@radix-ui/*`, `radix-ui`, `lucide-react`, `sonner`, `zustand`,
   `@lingui/react`, `@vitejs/plugin-react-swc`, `@lingui/swc-plugin`, `@testing-library/react`,
   `@types/react*`, `eslint-plugin-react-hooks`, `eslint-plugin-react-refresh`. Le `shared` React
   est supprimé, le `<SvelteIsland>` aussi.
6. `VENDOR_CHUNKS` nettoyé.
7. **`CLAUDE.md` réécrit** : « React — Best Practices » → « Svelte — Best Practices », stack,
   conventions de nommage (`*.page.svelte`, `*.svelte.ts` pour les stores runes), checklist
   « Adding a feature ». Les 18 `AGENTS.md` sont déjà à jour, phase par phase.

**Critère de sortie** : `grep -r "react" package.json` ne renvoie rien, `pnpm ls` sous ~400 paquets,
et les 6 gates verts.

---

## 6. La recette, pour un module

Identique de la Phase 2 à la Phase 5. C'est le cœur réutilisable de ce document.

1. Lire le `AGENTS.md` du module. Relever l'API publique exacte (`index.ts`), les routes, les
   permissions, ce que d'autres modules consomment.
2. **Porter les tests d'abord** vers `@testing-library/svelte` — ils décrivent le comportement
   attendu et deviennent le filet. Requêtes toujours par rôle et nom accessible ; le faux repository
   passe toujours par le DI.
3. Feuilles → racine : composants présentationnels, puis conteneurs, puis la page.
4. Hooks `use-*` → `*.svelte.ts` (runes + `createQuery`/`createMutation`). **Les query keys ne
   changent pas** — c'est ce qui permet au cache d'être partagé avec les modules encore en React.
5. Messages i18n extraits dans `*.messages.ts` (§4.4).
6. `*.module.ts` : seul le `lazy:` change. `permission`, `breadcrumb`, `nav`, `id`, `domain` sont
   **inchangés** — donc `module-permissions.test.ts` continue de garantir le gating sans qu'on y
   touche.
7. Retirer le `<SvelteIsland>` du parent quand tout le sous-arbre est passé.
8. `index.ts`, `AGENTS.md`, `README.md` mis à jour dans la même PR.
9. `node scripts/restore-translations.mjs --dry-run` si des fichiers ont changé de module.
10. Les 6 gates + `pnpm build` + passe manuelle sur les écrans du module.

---

## 7. Règles pendant la migration

- **Le `shared` React est gelé** dès la fin de Phase 1 : correctifs uniquement. Toute divergence
  entre les deux versions est une dette payée deux fois.
- **Un module est soit React soit Svelte**, jamais à moitié à la fin d'une PR. Le `<SvelteIsland>`
  vit à l'intérieur d'une PR, pas entre deux.
- **Aucun gate désactivé**, même temporairement. Les seuils de couverture sont un cliquet.
- **Aucun deep import** ne devient tolérable parce que « c'est la migration ». Les barrels tiennent.
- **Nouveaux modules : Svelte**, à partir de la Phase 1.
- **`main` reste déployable à tout instant.** Chaque phase est une suite de PRs mergeables, et le
  binaire Rust embarque un `dist/` fonctionnel à chaque commit.
