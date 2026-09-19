# Migration React → Svelte 5

Migration de la couche `presentation/` de Scylla Frontend, de React 18 vers **Svelte 5 + Vite**.  
Objectif final : plus une ligne de React, et une surface de dépendances divisée par deux.

> Ce document est le contrat de la migration. Il complète `CLAUDE.md`, il ne le remplace pas :  
> les règles d'architecture (4 couches, barrels, DI, `ScyllaModule`, i18n, tests) restent  
> intégralement en vigueur pendant et après.

---

## État d'avancement

| Phase | État | Bundle initial | Total | Deps |
|---|---|---|---|---|
| — départ | | 294,8 kB | 660 kB | 41 |
| **0 · Lot A** — nettoyage deps | ✅ **fait** | **253,6 kB** | **619 kB** | **39** |
| **0 · Lot B** — dé-React-ification | ✅ **fait** | 253,5 kB | 619 kB | 43 |
| **1** — `shared/` + design system | ✅ **fait** | 253,5 kB | 619 kB | 47 |
| **2** — 6 features pilotes | ✅ **fait** | **292,3 kB** | **862 kB** | 47 |
| **3** — apps, agents, membership, jobs, triggers | ⬜ | | | |
| **4** — roles + dashboard (`recharts` sort ici) | ⬜ | | | |
| **5** — pipeline (`reactflow` sort ici) | ⬜ | | | |
| **6** — shell + suppression de React | ⬜ | | | ~20 |

Cible finale : ~170 kB initial, ~380 kB total, ~400 paquets transitifs.

**La Phase 2 fait monter le chargement initial, et c'est attendu** : 253,6 → 292,3 kB gzip.  
Le détail, mesuré : le runtime Svelte (`vendor-svelte`, 20,3 kB) devient nécessaire dès l'entrée —  
`sveltePage` est dans le graphe eager du registre —, `@tanstack/svelte-query` rejoint  
`vendor-query` (24 → 35,3 kB), et le code applicatif prend 1,7 kB. Les deux moitiés du design  
system coexistent, donc le total passe de 619 à 862 kB. Tout cela repart en Phase 6 avec  
`vendor-react` (77,1 kB) et la moitié React de `vendor-ui` et `vendor-query`.

**`vendor-ui-svelte` (35,9 kB), lui, n'est *pas* dans l'entrée, et il a fallu le faire exprès.**  
La shell importe le barrel d'`organization` pour ses queries ; un composant `.svelte` réexporté  
par ce barrel n'est pas éliminable par Rollup, et tirait bits-ui au premier paint pour deux  
dialogues que personne n'a ouverts. D'où `loadAddOrganizationDialog()` /  
`loadEditOrganizationDialog()` — des fonctions, donc tree-shakeables — et `LazySvelteIsland` en  
face. **Règle générale : un barrel que la shell importe n'exporte pas de composant Svelte, il  
exporte un loader.**

Le Lot B ajoute 4 dépendances (`svelte`, `@tanstack/svelte-query`, `@tanstack/query-core`,  
`@sveltejs/vite-plugin-svelte` & co) **sans toucher au bundle de production** : aucune UI Svelte  
n'est encore livrée, donc rien de tout ça n'entre dans un chunk. Le compte redescend à partir de  
la Phase 2, quand les paires React sortent.

Même remarque pour les 4 dépendances qu'ajoute la Phase 1 (`bits-ui`, `@lucide/svelte`,  
`@tanstack/svelte-table`, `@tanstack/table-core`) : aucun code de production ne les importe encore  
— le `shared` Svelte n'a pas de consommateur avant la Phase 2 — donc le build est **identique au  
bit près**, vérifié en comparant les hashes de chunks et non supposé. `vendor-ui-svelte` n'existe  
même pas encore dans `dist/`.

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

## 4. Décisions techniques & Architecture Svelte 5

Avec Svelte 5, les *custom hooks* React disparaissent. Nous adoptons une approche **MVVM / Presenter** stricte en utilisant les **Runes** natives (`$state`, `$derived`) et le pattern **Query Options**.

### 4.1 Architecture Svelte 5 : L'Art de la Présentation

#### Matrice de décision : Où placer la logique UI ?

Ne créez pas des classes abstraites pour tout. Soyez pragmatiques en suivant cette matrice :

| Besoin UI / Complexité | Solution Technique | Emplacement |
|---|---|---|
| **État UI simple** (toggle, modale, formulaire local à 2 champs, tabs) | Runes natives (`$state`) | Directement dans le `<script>` du `.svelte` |
| **Logique page / Orchestration** (TanStack + Filtres + Pagination) | ViewModel (Classe) | `features/.../presentation/*.state.svelte.ts` |
| **Logique UI complexe sans réseau** (Wizard, sélection matricielle) | ViewModel purement UI | `features/.../presentation/*.state.svelte.ts` |
| **Algorithmes purement mathématiques** (Positionnement graphe A*) | Fonction pure TS | `features/.../presentation/*.calculator.ts` (ou `.helpers.ts`) |
| **Mesures DOM / Événements natifs** (Scroll infini, Canvas, D&D) | Svelte Action | `features/.../presentation/*.actions.ts` |

#### L'Architecture d'une Page Complexe (Le Pattern State)

Quand une vue nécessite d'orchestrer des données distantes (TanStack) avec des filtres complexes, le code est réparti en 4 strates :

##### 1. `*.queries.ts` (Query Options Factory — TS Pur)
Sépare la déclaration de la requête de son exécution. Indépendant de Svelte.
```typescript
import { queryOptions } from '@tanstack/svelte-query';
import type { UserRepository } from '../domain/user.repository.ts';

export const userQueries = {
  list: (repo: UserRepository, filters: () => { search: string }) =>
    queryOptions({
      queryKey: ['users', 'list', filters()],
      queryFn: () => repo.getUsers(filters()),
    }),
};

```

### *.mutations.ts (same principle as queries.ts but for mutations, if we multiple state or component use the same mutation)
exemple : 
```typescript
export const userMutations = {
    create: (repo: UserRepository, queryClient: QueryClient) =>
        mutationOptions({
            mutationFn: (newUserData: CreateUserDTO) => repo.createUser(newUserData),
            onSuccess: () => {
                // La grosse force ici : on invalide la liste des users automatiquement partout !
                queryClient.invalidateQueries({ queryKey: ['users', 'list'] });
            },
        }),
};

usage :

private mutation = createMutation(() =>
    userMutations.create(this.repo, this.queryClient)
);
```

##### 2. `*.state.svelte.ts` (Le ViewModel / State)
Nommé selon la *vue* qu'il contrôle (ex: `user-list.state.svelte.ts`, pas un global `user.state.svelte.ts`). Il rassemble l'état, la query et l'injection de contexte. Il doit être strictement divisé en 4 sections :

```typescript
import { setContext, getContext } from 'svelte';
import { createQuery } from '@tanstack/svelte-query';
import { userQueries } from './user.queries.ts';
import type { UserRepository } from '../domain/user.repository.ts';

const CONTEXT_KEY = Symbol('UserListState');

export class UserListState {
  // ---------------------------------------------------------------------------
  // 1. ÉTAT UI LOCAL (Inputs utilisateur)
  // ---------------------------------------------------------------------------
  searchQuery = $state('');
  selectedUserId = $state<string | null>(null);

  // ---------------------------------------------------------------------------
  // 2. DATA FETCHING (TanStack Query)
  // Lit les propriétés réactives de l'état UI via des getters arrow functions
  // ---------------------------------------------------------------------------
  private query = createQuery(() => 
    userQueries.list(this.repo, () => ({ search: this.searchQuery }))
  );

  constructor(private repo: UserRepository) {}

  // ---------------------------------------------------------------------------
  // 3. OUTPUTS & CALCULS MÉMOÏSÉS ($derived)
  // ---------------------------------------------------------------------------
  get users() { return this.query.data ?? []; }
  get isLoading() { return this.query.isLoading; }

  // Recalculé uniquement si 'users' ou 'selectedUserId' change
  selectedUser = $derived.by(() => {
    if (!this.selectedUserId) return null;
    return this.users.find(u => u.id === this.selectedUserId) ?? null;
  });

  // ---------------------------------------------------------------------------
  // 4. CONTEXT API (Pour éviter le prop drilling)
  // ---------------------------------------------------------------------------
  static set(repo: UserRepository) {
    return setContext(CONTEXT_KEY, new UserListState(repo));
  }
  static get(): UserListState {
    return getContext<UserListState>(CONTEXT_KEY);
  }
}
```

##### 3. `*.page.svelte` (La Vue Racine)
Instancie le state et le distribue (implicitement via le Context ou explicitement).
```svelte
<script lang="ts">
  import { getModuleDomain } from '@platform/di';
  import { UserListState } from './states/user-list.state.svelte.ts';
  import UserTable from './components/UserTable.svelte';

  const repo = getModuleDomain('user').repository;
  
  // Instancie le ViewModel ET l'injecte dans le contexte Svelte
  const state = UserListState.set(repo); 
</script>

<div class="page-container">
  <input bind:value={state.searchQuery} placeholder="Rechercher..." />
  
  <!-- N'a pas besoin de props, il consommera le Context en interne -->
  <UserTable/>
</div>
```

##### 4. `*.actions.ts` (Svelte Actions)
Les actions (`use:action`) remplacent les `useEffect` React liés aux références (`useRef`). Elles sont la **seule interface autorisée** pour toucher le DOM directement.  
**Règle :** Les actions doivent être nommées par des verbes (ex: `autoScroll`, `trapFocus`, `renderCodeMirror`).

```typescript
import type { Action } from 'svelte/action';

export const autoScroll: Action<HTMLElement, boolean enabled: { }> = (node, options) => {
  let isEnabled = options.enabled;

  const observer = new ResizeObserver(() => {
    if (isEnabled) node.scrollTop = node.scrollHeight;
  });

  observer.observe(node);

  return {
    update(newOptions) {
      isEnabled = newOptions.enabled;
    },
    destroy() {
      observer.disconnect();
    }
  };
};
```

### 4.2 Le routeur — **décidé : maison (~300 LOC)**

SvelteKit est écarté (§0). Les micro-routeurs de l'écosystème (`svelte-spa-router`, `svelte-routing`)  
ne gèrent pas correctement layouts imbriqués, `lazy` et métadonnées de route — or `RouteGuard` et  
les breadcrumbs en dépendent.

La surface react-router réellement utilisée est étroite : `useNavigate` (45), `useParams` (35),  
`Outlet` (16), `useLocation` (14), `Navigate` (8), `useMatches` (5). Un matcher de chemins + un  
`<Outlet>` imbriqué + un loader `lazy` + la propagation de `handle`, c'est ~300 lignes testables,  
qui vivent dans `@platform/routing` aux côtés de `compose-module-routes.ts`. Et la Phase 0 interdit  
aux features d'importer react-router directement : au moment de la bascule, il n'y a **qu'un seul  
endroit** à changer.

### 4.3 Correspondance des dépendances

| Aujourd'hui | Demain | Note |
|---|---|---|
| `react`, `react-dom` | `svelte` | |
| `react-router-dom` (45 fichiers) | `@platform/routing` maison | −1 dép ; depuis la Phase 2, la navigation passe par `setAppNavigator` / `navigateTo` et **`Core.router.tsx` est le seul fichier à changer** |
| `@tanstack/react-query` (49) | `@tanstack/svelte-query` | **même `query-core`, même `QueryClient`, cache partagé**. ⚠ Phase 2 : `createQuery`/`createMutation` s'importent de `@platform/query`, qui y lie le client — un îlot n'a pas le contexte Svelte que les originaux lisent |
| `zustand` (7) | runes (`$state` en module / classes `*.state.svelte.ts`) | −1 dép |
| `framer-motion` (5) | `transition:` / `animate:` / `crossfade` natifs | −1 dép, −41 kB |
| `next-themes` (6) | ~25 lignes maison | −1 dép |
| `sonner` (24 fichiers, **1 seul point d'entrée** : `shared/presentation/utils/toast.ts`) | `svelte-sonner` | échange trivial |
| `lucide-react` (89) | `@lucide/svelte` | mapping 1:1, mécanique |
| `@radix-ui` + `radix-ui` (38 fichiers, 33 primitives shadcn) | `shadcn-svelte` (sur `bits-ui`) | **Tailwind et classes identiques → tout le style survit tel quel** |
| `@tanstack/react-table` (12) | `@tanstack/svelte-table` **v9** | ⚠️ corrigé en Phase 1 : pas de v8 pour Svelte 5. La *forme* des défs survit, la signature générique non — voir « Ce que la fin de la Phase 1 a appris » |
| `@uiw/react-codemirror` (8) | CodeMirror 6 direct via une `action` Svelte | **−1 dép** (le wrapper React disparaît, pas CodeMirror) |
| `reactflow` (6) | `@xyflow/svelte` | port officiel, API proche |
| `recharts` (2) | SVG maison | **−119 kB** ; un seul graphe concerné |

Cible : **41 → ~20 dépendances runtime** (39 après Lot A, 47 au pic de la Phase 1 — les deux
moitiés du design system coexistent, et le compte ne redescend qu'à partir de la Phase 2 quand les
paires React sortent), **861 → ~400 paquets transitifs**.  
`vendor-react`, `vendor-motion` et `vendor-charts` disparaissent des `VENDOR_CHUNKS`.

### 4.4 Points durs

1. **`reactflow` → `@xyflow/svelte`** (`pipeline`). API proche mais pas identique (nodes/edges en  
   stores, `$props` pour les custom nodes). `blueprint-converter.ts` est du code pur et survit avec  
   ses tests — c'est le filet de sécurité. Les 4 composants de canvas sont une réécriture. Morceau  
   le plus long de la migration.
2. **`dependency-cruiser` ne parse pas `.svelte`.** Les six règles `error` qui protègent les barrels  
   et le sens des couches deviendraient **aveugles** sur tout le code neuf. Traité en Phase 0, §4.6.
3. **Lingui n'extrait pas depuis `.svelte`.** Voir §4.5.
4. **`recharts`** n'a aucun portage Svelte. Décision prise : SVG maison (un seul graphe), ce qui  
   sort aussi `d3-*` et `victory-vendor`. Repli si ça dérape : `LayerChart`.

### 4.5 i18n — la règle à ne pas rater

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

### 4.6 Les garde-fous doivent survivre

Aucune phase n'est terminée si un gate est désactivé « le temps de la migration ».

- **`depcruise`** : ✅ fonctionne tel quel sur `.svelte`, sans configuration ni pré-traitement.  
  Vérifié par violation délibérée, pas supposé. Le plan B ESLint n'existe plus.
- **`module-permissions.test.ts`** et **`feature-permissions.test.ts`** : ils lisent  
  `core/di/registry.ts` et le source de `presentation/ui/`. Ils continuent de fonctionner à  
  condition que `ScyllaModule.routes[].lazy` garde sa forme et que leurs globs incluent `.svelte`.  
  **Vérifié en Phase 0, pas après.**
- **Couverture** : ✅ `coverage.include` est passé à `src/modules/**/*.{ts,tsx,svelte}`. Les seuils  
  restent un cliquet : ils ne baissent jamais, même temporairement. Un module migré rend ses tests,  
  sinon il n'est pas migré.
- **`svelte-check`** : `.svelte` est invisible pour `tsc -b`. `pnpm typecheck` enchaîne donc  
  `tsc -b && svelte-check` — le gate garde son nom et rien ne passe entre les mailles.
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

#### Lot B — Dé-React-ification de la plomberie ✅ **fait**

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
5. i18n : `i18n-svelte.ts` (helper `t()` réactif) + la convention `*.messages.ts` de §4.5.
6. **Interdiction faite aux features d'importer `react-router` directement** : tout passe par  
   `useScyllaNavigate` / `@platform/routing`, via `no-restricted-imports`. C'est ce qui rendra la  
   Phase 6 petite.

**Le pont** — un seul fichier : `<SvelteIsland component={X} props={…} />`. Après les points 1-5,  
il n'a **rien** d'autre à ponter.

**Garde-fous** — §4.6 traité intégralement, ici et pas plus tard.

**Critère de sortie** : un composant Svelte jetable, monté dans une page React, qui lit une query  
TanStack existante, une traduction, une permission et le store de contexte — et les 6 gates verts.  
Ce composant est ensuite supprimé.

---

### Phase 1 — `shared/` : le design system Svelte + le harnais de test

*76 fichiers, ~6 000 LOC annoncés. **Mesuré : beaucoup moins.** La plus mécanique, et elle  
débloque le reste.* **Bloquante : rien d'autre ne peut avancer avant.**

#### Ce que la Phase 2 a réellement besoin (mesuré, pas estimé)

Un script a résolu les imports transitifs des six features pilotes jusqu'aux primitives. Résultat :

- **10 primitives shadcn sur 31** : `alert-dialog`, `avatar`, `button`, `card`, `checkbox`,  
  `dialog`, `input`, `skeleton`, `table`, `tooltip`.
- **21 différées**, sans consommateur Svelte avant longtemps : `sidebar` (Phase 6), `chart`  
  (Phase 4), `select`, `tabs`, `sheet`, `dropdown-menu`, `breadcrumb`, `collapsible`,  
  `scroll-area`, `progress`, `switch`, `radio-group`, `toggle`, `toggle-group`, `separator`,  
  `badge`, `label`, `field`, `pagination`, `code-snippet`, `sonner`.

On porte à la demande : une primitive arrive la phase où son premier consommateur Svelte arrive.  
Porter les 31 d'un coup, c'est 21 composants sans usage — exactement ce que la règle de  
minimalisme interdit.

**Deux simplifications tombées de cette mesure :**

- **`toast.ts` n'a rien à porter.** Le fichier est `export { toast } from 'sonner'`, et le  
  `toast()` de sonner est un singleton de module agnostique qui pousse dans un store global. Un  
  composant Svelte l'importe tel quel. `svelte-sonner` ne devient nécessaire qu'avec le  
  `<Toaster>`, donc en **Phase 6**.
- **L'action CodeMirror part en Phase 3**, où vit son premier consommateur (`jobs`). L'écrire ici  
  serait du code sans usage.

#### Découpage et avancement

| Tranche | État |
|---|---|
| Harnais de test (`src/test/render.svelte.ts`) | ✅ |
| Primitives sans dépendance headless : `button`, `card` (×7), `input`, `skeleton` | ✅ |
| Primitives sur `bits-ui` : `tooltip`, `dialog`, `alert-dialog`, `checkbox`, `avatar` | ✅ |
| `table` + `DataTable` sur `@tanstack/svelte-table` | ✅ |
| Wrappers génériques : `IconButton`, `TruncatedText`, `CopyableText`, `ContextItem`, `ErrorState`, `ConfirmOperationAlertDialog`, `FeatureHeader` | ✅ |
| Formulaires : `ScyllaForm`, `FormDialog`, `useFormState` (le typage générique doit survivre) | ✅ |
| Hooks : `use-selection`, `use-pagination`, `use-feature-selection` | ✅ |
| Dette d'animations (tableau ci-dessous) | ✅ pour `shared/` — les 2 autres sites vivent dans `layout/` et `pipeline/`, donc Phases 6 et 5 |

Primitives ajoutées en cours de route, chacune parce que son premier consommateur Svelte est  
arrivé : `table` (les 5 parties que compose `DataTable`), puis `label`, `field` et `select` que  
`ScyllaForm` exige. Restent différées, sans consommateur : `sheet`, `dropdown-menu`, `breadcrumb`,  
`collapsible`, `scroll-area`, `progress`, `switch`, `radio-group`, `toggle`, `toggle-group`,  
`separator`, `badge`, `pagination`, `code-snippet`, `tabs`, `sidebar` (Phase 6), `chart` (Phase 4),  
`sonner` (Phase 6).

**Non porté, et délibérément** : `Pagination` / `PaginationSlot` et sa primitive `pagination`,  
`ListCard`, `StatusBar`, `CheckboxTree`. Le texte de la Phase 1 les citait, mais le tableau des  
tranches — qui est la checklist — ne les demande pas, et la mesure des six features pilotes ne les  
fait pas remonter. `CheckboxTree` en particulier appartient à `roles` (Phase 4), dont il est la  
logique UI la plus dense. Les écrire ici, ce serait exactement les composants sans usage que la  
règle de minimalisme interdit.

#### Une règle apprise en portant `button`

**`tsc` ne voit d'un `.svelte` que son export par défaut.** Les exports d'un `<script module>`  
lui sont invisibles, donc tout ce qu'un `.ts` doit importer — une config `cva`, un type de  
variante — vit dans un `.ts` à côté (`button-variants.ts`), jamais dans le composant. Le gate  
`typecheck` attrape la faute, mais autant ne pas la faire.

#### Cinq règles apprises en portant Radix → `bits-ui`

Aucune n'est attrapée par le compilateur ; toutes cassent à l'exécution. Elles valent pour les
21 primitives restantes.

1. **`asChild` devient le snippet `child`**, qui passe par `...rest` sans code dédié :
   `<TooltipTrigger>{#snippet child({ props })}<Button {...props}/>{/snippet}`. Les props du
   trigger atterrissent **sur** l'élément de l'appelant, donc son `data-slot` gagne sur celui du
   `Button` — c'est `data-variant` qui identifie encore ce dernier. `IconButton` en dépend.
2. **Un nom de variable CSS change** : `--radix-*-content-transform-origin` devient
   `--bits-floating-transform-origin`. Rien n'échoue bruyamment si on l'oublie, l'animation part
   juste du mauvais coin.
3. **Les parties sans style s'aliasent depuis `bits-ui` dans l'`index.ts`**, elles ne se wrappent
   pas — `shadcn/dialog.tsx` fait déjà exactement ça avec Radix. Un fichier dont tout le corps est
   `<Primitive {...rest} />` est un fichier à maintenir pour rien.
4. **`AlertDialogAction` / `AlertDialogCancel` restent de simples `Button`**, pas ceux de bits-ui
   qui ferment la boîte au clic : toutes les confirmations d'ici gardent la boîte ouverte et
   désactivée pendant la mutation (`ConfirmOperationAlertDialog`, `isLoading`). En prime, bits-ui
   a un vrai `role="alertdialog"` qui ignore le clic extérieur — ce que la version Radix, bâtie
   sur un dialog ordinaire, n'a jamais été.
5. **Tester un dialog** : bits-ui verrouille `<body>` en `pointer-events: none`, et le verrou
   survivait au démontage — il faisait échouer le test *suivant* du fichier avec une erreur qui
   désignait un innocent. `setup.ts` le nettoie désormais avant chaque test. Un clic « à
   l'extérieur » reste hors de portée de `userEvent` : `fireEvent.pointerDown(document.body)`,
   c'est ce que la couche de dismiss écoute.

Et une règle pour `VENDOR_CHUNKS` : **`vendor-ui-svelte` ne liste que des paquets exclusivement
Svelte.** `@floating-ui` et `tabbable` sont partagés avec Radix ; revendiquer le scope a déplacé
8,7 kB gzip de positionnement *React* dans un chunk au nom de Svelte, préchargé depuis l'entrée.
Les paquets partagés restent non assignés jusqu'à la Phase 6. Vérifié en comparant les hashes de
build : à ce stade le bundle de production est **inchangé au bit près**, aucune UI Svelte n'étant
encore livrée.

#### Ce que la fin de la Phase 1 a appris

Quatre choses qu'aucun des deux documents n'anticipait, et qui coûteront cher aux Phases 2 à 5 si
on les redécouvre module par module.

1. **`@tanstack/svelte-table` est en v9, pas en v8 — les `ColumnDef` ne « survivent » donc pas
   tels quels.** §4.3 le promettait ; c'est faux. La v8 de l'adaptateur a un peer `svelte ^4` et
   est inutilisable ici. En v9 les *features* portent des types, donc
   `ColumnDef<TData, TValue>` devient `ColumnDef<TFeatures, TData, TValue>`, `useReactTable`
   devient `createTable({ features, get data() {…} })`, et `flexRender(def, ctx)` devient le
   composant `<FlexRender {cell} />`. La **forme** des définitions survit — `accessorKey`,
   `header`, `size`, `minSize`, `meta` — et c'est ça le vrai dividende : `DataTableColumn<TData>`
   masque le jeu de features, et une feature migrée ne réécrit que ses `cell:`.
   Conséquence : `@tanstack/table-core` v9 est épinglé en dépendance directe, comme `query-core`,
   parce qu'une augmentation de module (`ColumnMeta.align`) doit viser le paquet qui *déclare*
   l'interface, pas celui qui la ré-exporte.
2. **Le nom d'un placeholder fait partie du msgid.** Porter
   `plural(selectedCount, …)` en `plural(count, …)` a créé un second message, vide, pendant que le
   premier gardait sa traduction. Rien n'aurait échoué : `extract` écrit l'entrée, `i18n:collisions`
   reste à zéro, la couverture ne bouge pas, et l'arme plurielle rend en anglais. **Un message porté
   garde les noms de variables de l'original** — et c'est le `*.fr.test.ts` qui le prouve, pas un
   gate.
3. **bits-ui laisse tomber des rôles ARIA que Radix posait.** Le contenu de tooltip n'avait pas
   `role="tooltip"` ; le trigger de select avait `aria-haspopup`, `aria-expanded` et
   `aria-activedescendant` mais pas `role="combobox"`. Rien ne casse bruyamment — le texte passe
   quand même par `aria-describedby`, le bouton s'ouvre quand même. **Vérifier le rôle à chaque
   primitive portée**, et le remettre dans *notre* wrapper.
4. **`tsc -b` et `svelte-check` ne voient pas le même code, et c'est le plus faible qui dicte la
   syntaxe.** Pour `tsc`, tout import `.svelte` est le `LegacyComponentType` ambiant de Svelte :
   sans paramètres de type, donc une expression d'instanciation (`DataTable<Datum>`) échoue en
   TS2635 alors que `svelte-check` l'accepte. La sortie est un `*.fixture.svelte` qui instancie le
   générique *dans* un fichier `.svelte`, là où `tsc` ne regarde pas et où `svelte-check` vérifie
   pour de vrai. Le test rend la fixture. Même remède pour les trois composants génériques —
   `DataTable`, `ScyllaForm`, `FormDialog`.

Trois ajouts au harnais partagé, faits une fois pour toutes dans `setup.ts` / `render.svelte.ts` :
le nettoyage de `pointer-events` sur `<body>`, un stub de `Element.prototype.animate` (jsdom n'a
aucune Web Animations API, dont dépend chaque `transition:`), et `findFloating(role, name?)` —
floating-ui n'ayant rien à mesurer sous jsdom, il laisse ses wrappers en `visibility: hidden`, ce
qui masque le nœud à `getByRole` **et** vide son nom accessible.

- Port `shadcn/ui` → **`shadcn-svelte`** pour les 33 primitives. Les classes Tailwind sont  
  identiques : transposition de syntaxe, pas redesign. Radix → `bits-ui`.
- `DataTable` sur `@tanstack/svelte-table` ; `Pagination`, `FeatureHeader`, `IconButton`,  
  `TruncatedText`, `CopyableText`, `ListCard`, `StatusBar`, `ErrorState`, dialogs.
- `ScyllaForm` / `FormDialog` / `useFormState` → version runes. **Le typage générique sur les ids  
  d'items doit survivre** (`FormItem<'a'|'b'>` → `FormValues` typé) : c'est ce qui empêche de  
  chercher les valeurs par id, et ça se perd facilement dans un port.
- `toast.ts` → `svelte-sonner` (un fichier).
- `AnimatedOutlet`, `ScyllaLoadingScreen` → transitions natives.

  **Dette à rembourser ici, explicitement.** Le Lot A a perdu les animations de *sortie* : le CSS  
  ne peut pas animer un nœud que React a déjà démonté, donc la navigation n'a plus son fade-out et  
  le libellé de statut de l'éditeur de pipeline n'a plus sa sortie vers le haut. Svelte n'a pas  
  cette limite — `out:`, `transition:` et surtout `{#key}` + `crossfade` gardent le nœud sortant  
  vivant le temps de l'animer. **Le rendu doit revenir au niveau d'avant le Lot A au minimum**, et  
  la cible est mieux que ça :

  | Endroit | Avant (framer) | Après Lot A | État | Livré |
    |---|---|---|---|---|
  | Changement de route | fade + scale in/out, 200 ms | entrée seule | ✅ | `PageTransition.svelte` — `{#key}` + `in:pageIn` / `out:pageOut` |
  | Logo de chargement | fade + rotation | identique | ✅ | `ScyllaLoadingScreen.svelte` — et il gagne la sortie que React ne pouvait pas faire |
  | Écran « première orga » | fade + scale + y, 800 ms | entrée seule | ⬜ | vit dans `layout/Layout.tsx` → **Phase 6** |
  | Statut éditeur pipeline | crossfade vertical 150 ms | entrée seule | ⬜ | vit dans `pipeline/PipelineEditorHeader.tsx` → **Phase 5** |

  Les deux sites restants ne sont pas un oubli : ce sont des composants React de modules non
  migrés, et les convertir voudrait dire migrer `layout/` et `pipeline/` en avance. Les
  transitions dont ils ont besoin existent déjà dans `ui-svelte/motion/` ; il ne restera qu'à les
  appliquer.

  **Pas de `crossfade`, finalement.** Le `crossfade` de Svelte apparie des éléments *envoyés* et
  *reçus* entre deux blocs — c'est l'outil du « cet élément se déplace d'ici à là », pas celui de
  « cette page remplace celle-là ». Un `{#key}` avec `in:` et `out:` donne le chevauchement
  recherché, sans appariement à déclarer. Le wrapper est `relative` et les deux panneaux
  `absolute inset-0` : sinon la page sortante occupe la mise en page et pousse l'entrante vers le
  bas pendant 140 ms.

  Le `mode='wait'` de framer retenait la nouvelle page pendant 200 ms le temps que l'ancienne
  sorte. Ici les deux se chevauchent : **entrée 200 ms sans aucun délai, sortie 140 ms**, donc
  rien n'est jamais retenu et le contenu est lisible bien avant la fin — sur une courbe `cubicOut`
  l'opacité passe 0,8 vers 100 ms. L'échelle part de 0,99, pas de 0,95 : à l'échelle d'une page,
  un zoom visible donne l'impression que l'interface se reconstruit.

  `prefers-reduced-motion` doit continuer d'être respecté — et le Lot A ne suffit pas ici : la
  règle CSS de `index.css` neutralise les animations CSS, mais **une transition Svelte écrit des
  styles inline depuis JavaScript et cette media query ne la voit jamais**. D'où
  `motion/reduced-motion.ts` : chaque transition interroge la préférence et ramène sa durée à zéro.
  Même préférence, honorée deux fois, parce qu'il y a deux mécanismes.
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

### Phase 2 — Features pilotes : `login`, `marketplace`, `secret`, `user`, `project`, `organization` ✅ **fait**

*~2 400 LOC, 59 fichiers, aucune dépendance exotique.*

Six features indépendantes, recette identique. C'est la phase qui **valide la recette à  
l'échelle** — si quelque chose cloche dans le plan, ça se voit ici et pas au milieu de `pipeline`.

`login` en premier (134 LOC) : le plus petit chemin complet page + formulaire + mutation. Il a  
servi d'étalon, et la recette de §6 a tenu — au prix de quatre pièces que le plan n'avait pas vues.

#### Ce qu'il a fallu construire avant de migrer quoi que ce soit

1. **`sveltePage()`** (`@platform/routing`) : `lazy` doit rendre un composant que react-router sait  
   monter, donc un îlot. `ScyllaModule` ne change que d'une ligne, et `module-permissions.test.ts`  
   continue de vérifier le gating sans qu'on y touche. **Les paramètres de route arrivent en props**  
   — c'est la seule chose qu'une page ne peut pas lire dans un singleton : ils vivent dans l'état du  
   routeur React, et les relire depuis `window.location` reviendrait à réécrire le matching.
2. **Un navigateur agnostique** (`setAppNavigator` / `navigateTo`, dans `@platform/context`).  
   Le Lot B annonçait l'interdiction d'importer react-router dans les features ; elle n'avait jamais  
   été posée, et `useScyllaNavigate` était un hook. Il est devenu la **liaison React** d'un objet  
   `scyllaNavigate` sans framework : une seule implémentation des URLs, et `Core.router.tsx` est  
   désormais la seule ligne du projet qui nomme react-router pour naviguer.
3. **`createQuery` / `createMutation` re-exportés par `@platform/query`.** Ceux de  
   `@tanstack/svelte-query` lisent leur client dans le contexte Svelte — qu'un îlot monté dans  
   l'arbre React n'a pas. Le client est lié une fois ; `no-restricted-imports` interdit l'import  
   direct, parce que les deux sont indiscernables au point d'appel et que l'oubli ne casse qu'à  
   l'exécution.
4. **`can()` est devenu réactif** (`toRune` sur les deux stores). Il lisait `getState()` : correct  
   dans un handler, faux dans un `$derived`. Une page rendue avant que `usePermissionSync` ne  
   réponde serait restée refusée pour toujours. Hors contexte réactif, rien ne change.

#### Ce que la phase a appris

1. **Les hooks de lecture deviennent des `*.queries.ts`, et c'est ce qui sauve les consommateurs.**  
   Le vrai obstacle n'était pas de porter une page, c'était que `roles`, `membership`, `dashboard`,  
   `core` et `layout` — tous encore en React — consomment `useUsers`, `useProjects`,  
   `useOrganizations`, `useSecrets`. Un objet `queryOptions` n'a pas de framework : react-query le  
   prend tel quel. Les 14 sites d'appel changent d'une ligne, **partagent la même entrée de cache**,  
   et la règle 4 de `feature-permissions.test.ts` s'applique encore — elle lit maintenant les  
   fabriques en plus des hooks, sinon elle se serait vidée en silence au fil des migrations.
2. **Un composant React passé en prop est le seul pont qu'on ne peut pas construire.**  
   `OrganizationList` recevait son wrapper de ligne (`DropdownMenuItem`) de la sidebar. Ni îlot ni  
   snippet ne franchissent ça : le roving focus de Radix passe par un contexte React. Le rendu a  
   donc **déménagé dans `layout/`**, où vivent ses primitives et où il mourra en Phase 6 ; la  
   version Svelte d'`organization` sert le panneau des réglages. C'est le seul endroit des six où  
   le code existe en double, et c'est délibéré. **À vérifier avant d'ouvrir un module : qui lui  
   passe un composant ?**
3. **bits-ui volait le focus du premier champ d'un formulaire en dialogue.** Radix focalisait le  
   premier élément tabulable du contenu, bits-ui focalise le contenu lui-même : le curseur  
   n'arrivait plus dans le champ. En test, la conséquence était une valeur tronquée — `"DATA"` pour  
   `"DATABASE_URL"`, à une longueur différente à chaque exécution, sans rien qui échoue bruyamment.  
   Corrigé dans `dialog-content.svelte` (`onOpenAutoFocus` honore `autofocus`), plus  
   `focusSettled()` dans le harnais pour attendre que ça se pose.
4. **Un test qui moque un barrel migré moque désormais une fabrique**, pas un hook : `stubQuery()`  
   (`src/test/queries.ts`) rend des options avec `initialData`, ce qui garde ces tests synchrones.  
   Et `runQueryFn` / `runMutationFn` exécutent une fabrique sans composant : ce que  
   `use-marketplace.test.tsx` demandait à un arbre React, une pile de providers et un `waitFor` est  
   devenu un appel de fonction, en environnement `node`.

#### Deux dettes assumées, nommées

- **`marketplace` et `secret` portent des composants que personne ne monte** (`MarketItemList` &
  co., `SecretHealthOverview`, `SecretPagination`). Ils ont été portés tels quels plutôt que
  supprimés — ce n'est pas au chantier de migration de trancher —, et c'est écrit dans leur
  `AGENTS.md`. Le seul dommage réel : les trois nombres de `SecretPagination` perdent leur gras,
  parce que `<Trans>` pouvait les envelopper dans des éléments et qu'une chaîne `t()` ne peut pas.
- **La liste du sélecteur d'organisation existe en double** jusqu'à la Phase 6 (point 2 ci-dessus).

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

1. **Routeur maison** (§4.2) : `compose-module-routes.ts` réécrit contre lui, `RouteGuard`,  
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
   conventions de nommage (`*.page.svelte`, `*.state.svelte.ts` pour les ViewModels runes), checklist  
   « Adding a feature ». Les 18 `AGENTS.md` sont déjà à jour, phase par phase.

**Critère de sortie** : `grep -r "react" package.json` ne renvoie rien, `pnpm ls` sous ~400 paquets,  
et les 6 gates verts.

---

## 6. La recette, pour un module (Workflow de migration)

Identique de la Phase 2 à la Phase 5. C'est le cœur réutilisable de ce document.

1. Lire le `AGENTS.md` du module actuel pour en comprendre le comportement fonctionnel. Relever l'API publique exacte (`index.ts`), les routes, les permissions, ce que d'autres modules consomment.
2. **Porter les tests d'abord** vers `@testing-library/svelte` (ou en pur TS pour les classes `.state.svelte.ts`). Ils décrivent le comportement attendu et deviennent le filet. Requêtes toujours par rôle et nom accessible ; le faux repository passe toujours par le DI.
3. Supprimer les custom hooks React du module.
4. Structurer la couche Présentation selon les patterns Svelte 5 :
    - Garder `*.repository.ts` intact (Couche Infrastructure).
    - Créer `*.queries.ts` (Pure TS).
    - Décider de la granularité UI selon la **Matrice de décision** (Section 4.1).
    - Si complexe : créer `*.state.svelte.ts`.
    - Si manipulation DOM : créer `*.actions.ts`.
    - Si calcul lourd purement algorithmique : créer `*.calculator.ts`.
5. Réécrire les composants de bas en haut (feuilles → racine : composants présentationnels, puis conteneurs, puis `*.page.svelte`).
6. Messages i18n extraits dans `*.messages.ts` (§4.5).
7. `*.module.ts` : seul le `lazy:` change — `Component: sveltePage((await import('./…​.page.svelte')).default)`. `permission`, `breadcrumb`, `nav`, `id`, `domain` sont **inchangés**, donc `module-permissions.test.ts` continue de garantir le gating sans qu'on y touche. Les paramètres de route arrivent en props de la page (`let { projectId }: { projectId?: string } = $props()`).
7 bis. **Les consommateurs d'abord.** Avant de supprimer un hook exporté par le barrel, chercher qui l'importe (`grep "from '@/modules/features/<id>'"`). Un hook de lecture devient une fabrique `*.queries.ts` que react-query sait exécuter telle quelle : le consommateur React change d'une ligne et garde la même entrée de cache. Un **composant** passé en prop par la shell, lui, n'a pas de pont — voir Phase 2, point 2.
8. Retirer le `<SvelteIsland>` du parent quand tout le sous-arbre est passé.
9. `index.ts`, `AGENTS.md`, `README.md` mis à jour dans la même PR.
10. Lancer `node scripts/restore-translations.mjs --dry-run` si des fichiers ont changé de module.
11. Validation : Les 6 gates verts (`pnpm typecheck && pnpm test && pnpm lint && pnpm depcruise && pnpm depcruise:cycles && pnpm i18n:collisions`) + `pnpm build` + QA manuelle sur les écrans du module.

---

## 7. Garde-fous et Règles pendant la migration

- **Le `shared` React est gelé** dès la fin de Phase 1 : correctifs uniquement. Toute divergence entre les deux versions est une dette payée deux fois.
- **Un module est soit 100 % React soit 100 % Svelte** : aucun mélange interne n'est toléré à la fin d'une PR. Le `<SvelteIsland>` vit à l'intérieur d'une PR, pas entre deux.
- **Aucun gate désactivé**, même temporairement. Couverture, lint, typage et tests d'architecture doivent rester au vert continu.
- **Interdiction de mettre des calculs lourds dans les `.svelte`** : utilisez la séparation en `*.calculator.ts` ou dans un ViewModel via `$derived.by()`.