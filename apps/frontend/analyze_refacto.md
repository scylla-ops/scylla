# Analyse critique du refacto d'architecture — Frontend Scylla

Revue indépendante du travail décrit dans `refacto.md`, sur l'état actuel de la
branche `refactor/frontend/clean-archi`.

**Méthode.** Lecture du code (471 modules, ~27 400 LOC hors généré/locales),
exécution des quatre barrières, inspection du bundle produit (`dist/assets/`),
et recoupement systématique entre ce que `refacto.md` / `CLAUDE.md` affirment et
ce que le code fait réellement. Chaque constat ci-dessous est vérifié, pas
supposé ; les affirmations que je n'ai pas pu vérifier sont marquées comme
telles.

> **Mise à jour du 2026-09-07 — ce qui a été traité depuis la rédaction.**
> L'**étape 8 est faite** (§ 3.6) : 14 `index.ts`, 0 import profond cross-module,
> 6 nouvelles règles `dependency-cruiser` en `error`, chacune vérifiée par une sonde.
> Le **§ 3.5 est résolu** : plus aucune feature n'atteint l'accesseur DI d'une autre.
> Le **§ 4.3 est partiellement traité** : `projects` et `pipelines` ont des factories de clés
> partagées, les trois caches concurrents sur la liste des projets sont fusionnés.
> Une conclusion du plan initial est **infirmée** : `scripts/check-module-cycles.mjs` ne devient
> pas redondant une fois les barrels en place — preuve empirique en § 3.6bis. Il est conservé.
> Restent ouverts : §§ 3.1, 3.2, 3.3, 3.4, 3.7, 3.8, 3.9, 4.1, 4.2, 4.4-4.7.

**État des barrières au moment de l'analyse** — les quatre passent :

```
pnpm typecheck        ✔
pnpm lint             ✔  (0 warning)
pnpm depcruise        ✔  no dependency violations found (471 modules, 1998 dependencies)
pnpm depcruise:cycles ✔  no module cycles (17 modules cruised)
```

---

## 0. Verdict

**Le refacto était utile, et il l'était nettement.** Trois choses seulement le
justifient déjà à elles seules, et aucune n'est cosmétique :

1. **Casser la composante fortement connexe.** 16 modules dans une seule SCC,
   c'est une application monolithique déguisée : impossible de lire un module
   sans lire les quinze autres, impossible de code-splitter, impossible de faire
   contribuer quelqu'un sur un coin isolé. Pour un projet open-source qui grossit
   et attend des contributions externes, c'est le blocage numéro un. Il est levé.
2. **Rendre l'architecture exécutable.** `CLAUDE.md` décrivait une Clean
   Architecture que rien n'appliquait. Le trio
   `.dependency-cruiser.cjs` + `check-module-cycles.mjs` + CI transforme un
   document d'intention en contrat vérifié à chaque PR. C'est la seule chose qui
   empêche l'entropie de revenir sur un dépôt ouvert.
3. **La suppression des 64 use cases pass-through.** C'est la décision la plus
   courageuse du lot, et elle est correcte : elle va contre le culte du cargo
   « Clean Architecture = toujours quatre couches » et applique le vrai critère
   (une couche existe si elle fait quelque chose).

**Mais** — et c'est l'objet de cette analyse — le refacto s'est arrêté à
mi-parcours sur trois axes, et la partie *architecture applicative* (par
opposition à *architecture des dépendances*) a été peu touchée :

- **La justification qui a tué les 64 use cases n'a pas été appliquée à la couche
  d'en dessous**, où le même pass-through existe encore, à l'identique
  (§ 3.1). C'est le défaut structurel n°1.
- **Le code splitting est annulé pour toute la couche infrastructure** : les 12
  clients gRPC sont dans le chunk d'entrée, mesuré (§ 3.2).
- **L'étape 8 (API publiques) n'est pas faite**, donc la garantie centrale du
  refacto — « on n'atteint un module que par son API publique » — n'est ni vraie
  ni garantie : il reste **27 imports profonds cross-feature** (§ 3.6).

Et deux problèmes que le refacto a *déplacés* plutôt que résolus : le state
serveur dans Zustand (§ 3.4) et la session éparpillée en localStorage brut
(§ 3.3).

Score global : **7/10 en tant que refacto de dépendances**, **4/10 en tant que
refacto d'architecture applicative**. Le reste du document détaille.

---

## 1. Ce que le refacto a réellement gagné (vérifié)

| Gain | Vérification |
|---|---|
| Graphe acyclique | `depcruise:cycles` → 0 cycle sur 17 modules. Confirmé. |
| Frontières machine-vérifiées | 8 règles, 0 violation, en gate CI (`frontend.yml` : `depcruise` et `depcruise:cycles` sans `\|\| true`). Confirmé. |
| CI frontend | Existait pas ; existe maintenant, et se déclenche aussi sur `crates/scylla-protocol/proto/**`. Le raisonnement est juste : les clients TS ne sont pas commités, un changement proto peut casser le front sans toucher le front. Bien vu. |
| `platform/` sous les features | Réel et respecté : `platform-knows-no-feature` est vert, et `platform/` n'importe effectivement aucune feature. C'est la bonne décision structurante — elle rend l'acyclicité vraie *par construction*, pas par discipline. |
| Suppression des 64 use cases | Vérifié : il reste exactement 1 `*.use-case.ts` (`update-role.use-case.ts`). Le critère d'admission écrit dans `CLAUDE.md` est bon et défendable. |
| Domain purifié du proto | `domain-is-pure` vert. `idValue` n'apparaît plus que dans `infrastructure/` — sauf un cas, § 4. |
| Routes dérivées des modules | `Core.router.tsx` fait 83 lignes de shell. Ajouter une page = toucher un fichier. Réel. |
| Code splitting des pages | 39 chunks de route. Réel et efficace côté UI. |
| `msg` au lieu de JSX pour les breadcrumbs | Bonne décision, non évidente : c'est ce qui garde les `*.module.ts` en `.ts`. |
| Pas de packages pnpm par feature | **Décision correcte**, et pour la bonne raison. À 20 kLOC, un déployable, un consommateur, le coût du build graph n'est pas payé par le bénéfice. Le raisonnement « les cycles devaient sauter de toute façon, et une fois sautés les packages n'ajoutent que `index.ts` + lint » est exact. |

Deux points de qualité que je souligne parce qu'ils sont rares :

- **`platform/di` prend le registry en prop** (`Dependencies.provider.tsx`) au
  lieu de l'importer. C'est précisément ce qui permet au provider de vivre sous
  les features, et ça rend une feature testable en isolation. C'est la bonne
  forme.
- **Le `viaOnly: { dependencyTypesNot: ['dynamic-import'] }`** sur `no-circular`
  (`.dependency-cruiser.cjs:28`) : comprendre qu'un cycle traversant un import
  dynamique *est* une frontière de chunk et pas un cycle, c'est le genre de
  détail qui distingue une config copiée d'une config comprise.

---

## 2. Ce qui a été surévalué dans `refacto.md`

Rien de malhonnête, mais trois affirmations que le code ne soutient pas :

### 2.1 « Il n'y a plus de deuxième liste à maintenir en sync »

Faux. Il en reste **deux** :

- `lingui.config.js` — 18 entrées de catalogue écrites à la main. Ajouter une
  feature demande toujours d'y penser, et l'oubli est silencieux (pas d'erreur,
  juste des chaînes non traduites).
- `vite.config.ts` / `VENDOR_CHUNKS` — liste manuelle de packages.

`lingui.config.js` est dérivable : les chemins suivent tous
`src/modules/features/<x>/locales/{locale}/messages`. Un `glob` sur
`src/modules/*/locales` + `src/modules/features/*/locales` supprimerait la liste.
C'est ~10 lignes.

### 2.2 « Le bundle initial est passé de 2310 kB à 1093 kB »

Vrai, mais la ventilation cache le problème du § 3.2 : le chunk `index` de
374 kB contient **la totalité de la couche infrastructure de toutes les
features**, y compris les 12 clients gRPC. Le gain est venu presque entièrement
du vendor splitting et de la locale non-eager, pas de la lazification du code
métier. Le code métier propre à chaque feature est encore majoritairement eager.

### 2.3 « Étape 3 : corrige une violation de CLAUDE.md — du state serveur était stocké dans Zustand »

Le state serveur est toujours dans Zustand, dans un autre fichier. Voir § 3.4.

---

## 3. Les défauts structurels restants, par ordre d'importance

### 3.1 🔴 La couche `data-source` ↔ `repository` est exactement le pass-through que vous venez de supprimer

**C'est le point le plus important de cette analyse.**

Le critère écrit dans `CLAUDE.md` est :

> A class whose `execute(args)` just forwards to `repository.method(args)` is a
> second name for the same operation — delete it and call the repository.

Appliquons-le à `features/secret`. L'interface domain :

```ts
// domain/repository/secret.repository.ts
export interface SecretRepository {
  listByProjectId(projectId: string): Promise<ScyllaResult<SecretEntity[]>>;
  create(input: CreateSecretInput): Promise<ScyllaResult<SecretEntity>>;
  deleteById(secretId: string): Promise<ScyllaResult<void>>;
}
```

L'interface data source, dans `infrastructure/repository/data-sources/secret-remote.data-source.ts` :

```ts
export interface SecretRemoteDataSource {
  listByProjectId(projectId: string): Promise<ScyllaResult<SecretEntity[]>>;
  create(input: CreateSecretInput): Promise<ScyllaResult<SecretEntity>>;
  deleteById(secretId: string): Promise<ScyllaResult<void>>;
}
```

**Copie octet pour octet, y compris le type de retour domain.** Et
l'implémentation :

```ts
export class DefaultSecretRepository implements SecretRepository {
  constructor(private readonly remoteDataSource: SecretRemoteDataSource) {}
  public listByProjectId(id: string) { return this.remoteDataSource.listByProjectId(id); }
  public create(input: CreateSecretInput) { return this.remoteDataSource.create(input); }
  public deleteById(id: string)          { return this.remoteDataSource.deleteById(id); }
}
```

C'est *littéralement* le pattern qui a valu la suppression de 64 fichiers, un
niveau plus bas. Et le raisonnement de défense usuel (« le repository fait le
mapping ») ne tient pas ici : le data source renvoie déjà `SecretEntity`, donc
**c'est le data source qui mappe**, ce qui contredit directement `CLAUDE.md` :

> Infrastructure — Data sources (interface + `*.impl`) per transport; repository
> impl **coordinates data sources and maps infra types → domain types via mappers**.

État réel des 12 repositories :

| Population | Features | Le repository fait |
|---|---|---|
| **Pass-through pur** | `secret`, `agents`, `apps`, `marketplace`, `login` | rien — le data source renvoie déjà du domain |
| **Une ligne de mapping** | `user`, `jobs`, `project`, `organization`, `pipeline`, `triggers` | `(await ds.x()).map(Mapper.toDomain)` |
| **Fait vraiment quelque chose** | `roles` (136 lignes) | oui |

Autrement dit : sur 12, **5 sont à supprimer selon votre propre critère**, et 6
sont une ligne de `.map()` qui appartient au data source. Il y avait ici une
occasion de faire un vrai gain (−17 fichiers, −2 interfaces par feature) et le
refacto l'a manquée parce qu'il s'est arrêté à la couche use case.

**Ce qu'il fallait faire, et ce qu'il faut faire :**

L'inversion de dépendance dont vous avez besoin est *une seule* : `presentation
→ SecretRepository (domain) ← implémentation (infrastructure)`. La deuxième
interface n'ajoute rien, parce qu'il n'y a **qu'un seul transport** (gRPC-Web) et
qu'aucun test n'utilise le point de substitution. Cible :

```
domain/repository/secret.repository.ts        SecretRepository        (le contrat)
infrastructure/repository/grpc-secret.repository.ts
                                              GrpcSecretRepository    (impl gRPC : appel + mapping)
infrastructure/repository/mappers/grpc-secret.mapper.ts
```

Le point de substitution en test reste `SecretRepository`, injecté par le
module — inchangé. On supprime juste le nom intermédiaire.

> **Nuance honnête** : la couche data source *aurait* du sens si vous prévoyez un
> cache local (IndexedDB) ou un second transport. Si c'est au programme, gardez-la
> pour les features concernées — mais alors gardez-la *seulement* pour celles-là,
> et faites le repository coordonner réellement (`local` d'abord, `remote`
> ensuite). Aujourd'hui aucune feature n'a deux data sources, donc c'est de
> l'abstraction spéculative, ce que `CLAUDE.md` interdit explicitement
> (« No speculative abstraction. Factor at the 2nd or 3rd real usage »).

### 3.2 🔴 Le registry eager annule le code splitting de toute la couche infrastructure

**Mesuré, pas supposé.** Le chunk d'entrée `dist/assets/index-DbFp5XkS.js`
(374 kB) contient les 12 clients gRPC :

```
scylla.auth.v1.AuthService          scylla.pipeline.v1.PipelineService
scylla.organization.v1.Organization scylla.project.v1.ProjectService
scylla.job.v1.JobService            scylla.user.v1.UserService
scylla.app.v1.AppService            scylla.agent.v1.AgentAdminService
scylla.secret.v1.SecretService      scylla.authz.v1.GrantService
scylla.authz.v1.RoleService         scylla.trigger.v1.TriggerService
```

**Cause exacte.** Chaque `*.module.ts` instancie au niveau module :

```ts
// secret.module.ts:9-12
const secretRemoteDataSource: SecretRemoteDataSource = new GrpcSecretRemoteDataSource(grpcTransport);
const secretRepository = new DefaultSecretRepository(secretRemoteDataSource);
```

`core/di/registry.ts` importe les 14 `*.module.ts` en statique → les 14
constructions s'exécutent au démarrage → toute la chaîne
`data-source → client gRPC → types de messages proto` est tirée dans le chunk
d'entrée. Les pages sont lazy, mais leur infrastructure ne l'est pas.

C'est ironique, parce que le commentaire de `scylla-module.struct.ts:65-70`
explique justement qu'on ne met pas le module dans `index.ts` *pour éviter de
tirer les pages dans le chunk d'entrée* — mais on tire l'infra à la place.

**Correction, en une ligne de contrat :** rendre `domain` paresseux.

```ts
// platform/routing/scylla-module.struct.ts
readonly domain: TDomain | (() => TDomain);
```

```ts
// secret.module.ts
domain: () => {
  const ds = new GrpcSecretRemoteDataSource(grpcTransport);
  return { secretRepository: new DefaultSecretRepository(ds) };
},
```

Le hic : un thunk synchrone dans `use-module-domain.ts` ne suffit pas à faire
sortir le code du graphe statique — il faut que le module `import()` son infra.
Deux options, par ordre de préférence :

1. **`domain: () => import('./secret.domain.ts')`** — le registry mémoïse la
   promesse ; `useSecretDomain()` la lit via `use()` (React 18 : via un
   `Suspense` ou en la résolvant dans le `lazy` de la route). Plus intrusif, mais
   c'est le seul qui déplace vraiment les octets.
2. **Plus simple et probablement suffisant** : instancier le repository dans le
   `lazy` de la route au lieu du corps du module — le `lazy` est déjà un
   `import()`, donc l'infra suivrait la page dans son chunk. Ça demande de
   séparer `<feature>.wiring.ts` du `<feature>.module.ts`.

**Gain attendu** : je n'ai pas mesuré la variante corrigée, donc je ne chiffre
pas. L'ordre de grandeur est celui des 12 clients + leurs types de messages
sur ~1 Mo de proto généré ; c'est mesurable en une heure et ça vaut le coup de
le faire avant de décider.

**Bénéfice secondaire, plus important que les octets** : aujourd'hui le fait que
`registry.ts` connaisse les 14 modules *à l'exécution* fait de la DI un couplage
au démarrage. Avec un `domain` paresseux, un module non visité ne s'initialise
jamais — et ça vous ouvre la porte à des modules réellement optionnels (plugins),
ce qui est la trajectoire naturelle d'un projet open-source.

### 3.3 🟠 `platform/session` manquant — 12 accès `localStorage` bruts, dont un dans un data source

Le refacto a créé `platform/{authz, context, di, grpc, routing}` mais a oublié la
capacité transverse la plus évidente : **la session**. Résultat, le token et
l'identité utilisateur sont lus/écrits en dur, partout :

```
core/presentation/ui/App.tsx:32,40,58                  removeItem('token') × 3 + window.location.href
core/presentation/ui/router/Auth.guard.tsx:4           getItem('token')   ← pendant le render
platform/grpc/scylla-grpc-transport.ts:56              getItem('token')
layout/presentation/ui/NavUser.tsx:31,106              getItem('userId'), removeItem('token')
features/login/.../grpc-login-remote.data-source.ts:19 setItem('token'), setItem('userId')
features/organization/.../grpc-organization-remote.data-source.ts:32   getItem('userId')   ⚠️
features/roles/.../use-refresh-my-permissions.ts:16    getItem('userId')
features/roles/.../use-permission-sync.ts:20           getItem('userId')
features/user/.../UserSettings.page.tsx:30             getItem('userId')
features/user/.../UserAdmin.page.tsx:32                getItem('userId')
features/membership/.../OrganizationMembers.page.tsx:120  getItem('userId')
features/membership/.../ProjectMembers.page.tsx:166       getItem('userId')
```

Trois problèmes distincts :

1. **Un data source construit une requête à partir du `localStorage`**
   (`grpc-organization-remote.data-source.ts:32`). Le paramètre `userId` d'un
   appel n'est pas une donnée de transport ; il devrait descendre depuis
   l'appelant. C'est la même classe d'erreur que le proto qui remontait dans le
   domain (étape 7) — juste dans l'autre sens, et elle n'a pas été traitée.
2. **`AuthGuard` lit `localStorage` pendant le render** et n'est donc pas
   réactif : une déconnexion dans un autre onglet ne redirige pas. Le code
   contourne ça par `window.location.href = '/login'` (rechargement complet) dans
   `App.tsx` — ce qui « marche » mais impose un full reload à chaque 401.
3. **La chaîne de clés est stringly-typed** (`'token'`, `'userId'`) et répétée
   12 fois. Un renommage est un `grep`, pas un refactor sûr.

**Correction** : `platform/session/` — `use-session.store.ts` (Zustand persisté
sur `localStorage`, exactement comme `use-context.store.ts` le fait déjà :
`token`, `userId`), plus `useSession()` / `getToken()` pour l'intercepteur. Les
12 sites deviennent un import. `AuthGuard` devient réactif, la déconnexion
n'exige plus de reload, et `login` écrit dans le store au lieu d'écrire dans le
`localStorage` depuis un data source.

Coût estimé : une demi-journée. C'est probablement le meilleur rapport
gain/effort de toute cette liste.

### 3.4 🟠 `platform/authz` garde du state serveur dans Zustand — la violation a déménagé, pas disparu

`CLAUDE.md` est explicite :

> Zustand is for UI state only (modals, form values) — **server state lives in
> TanStack Query, never Zustand**.

Or `platform/authz/presentation/stores/use-permissions.store.ts` stocke
`EffectivePermissionsEntity`, c'est-à-dire une réponse backend. `refacto.md`
étape 3 présente l'extraction de cette slice comme *corrigeant* la violation ;
elle l'a seulement déplacée dans un store dédié.

Le coût est visible, et il est plus concret que le principe. Parce que le store
n'a pas de cache, il a fallu réinventer TanStack Query à la main :

```ts
// features/roles/presentation/hooks/use-permission-sync.ts
const lastSyncedKey = useRef<string | null>(null);
useEffect(() => {
  const userId = localStorage.getItem('userId') ?? '';
  const key = `${userId}/${orgId ?? ''}/${projectId ?? ''}`;
  if (lastSyncedKey.current === key) return;   // ← une clé de cache
  lastSyncedKey.current = key;                  // ← écrite à la main
  void refresh();
}, [refresh, orgId, projectId]);
```

Un `useRef` comme clé de cache, un `useEffect` pour déclencher un fetch, et une
contrainte de couplage documentée en commentaire (« Mount it **once** (in
`Layout`) »). Chacun de ces trois points est listé dans la section « useEffect is
a last resort » de `CLAUDE.md` comme un anti-pattern (« Server data → ❌
`useEffect` + gRPC call, ✅ a TanStack Query hook »).

Un `useQuery({ queryKey: ['authz','me',userId,orgId,projectId] })` remplace le
store, le `useRef`, le `useEffect` et la contrainte de mount — et donne
l'invalidation gratuite (aujourd'hui, changer un rôle ne rafraîchit pas les
permissions effectives sans repasser par `usePermissionSync`).

**Le blocage réel**, et il est légitime : `platform/authz` ne peut pas importer
le repository de `features/roles`, donc `useCan` ne peut pas faire l'appel
lui-même. `refacto.md` piège n°3 conclut « `platform/authz` n'a besoin d'aucune
infrastructure » — mais c'est vrai *parce qu'on a choisi le store*, ce n'est pas
une propriété du problème. Trois sorties, par ordre de préférence :

1. **Donner à `platform/authz` son propre `authzRepository`** (une méthode :
   `getMyEffectivePermissions`). C'est une capacité de plateforme qui a
   légitimement un backend — comme `platform/grpc` a un transport. `useCan`
   devient un `useQuery`, `features/roles` garde l'administration des rôles.
   C'est la version propre.
2. **Laisser `features/roles` écrire dans le cache Query** (`setQueryData` sur
   la clé `['authz','me',…]`) et `platform/authz` le lire avec un `useQuery`
   `enabled: false`. Zéro déplacement de code, mais le producteur et le
   consommateur communiquent par une clé de chaîne — fragile.
3. Garder le store, mais assumer la décision et **retirer la phrase « never
   Zustand » de `CLAUDE.md`** ou la nuancer, parce qu'aujourd'hui la règle et le
   code se contredisent, et sur un projet open-source c'est la règle qui perd.

### 3.5 🟠 `dashboard` et `roles` court-circuitent les hooks d'une autre feature via son accesseur DI

```ts
// features/dashboard/presentation/hooks/use-org-overview.ts:2-3
import { usePipelineDomain } from '@/modules/features/pipeline/presentation/hooks/use-pipeline-domain.ts';
import { useProjectDomain }  from '@/modules/features/project/presentation/hooks/use-project-domain.ts';
```

`useXDomain()` est la surface *privée* d'une feature : c'est le point où elle fixe
le type de sa propre injection. Qu'une autre feature l'appelle pour taper
directement sur `projectRepository` signifie que `dashboard` construit ses
propres requêtes sur les données de `project` et `pipeline`, avec ses propres
clés de cache (`['dashboard','projects',orgId]`) — donc **deux caches sur la même
ressource**, qui ne s'invalident pas l'un l'autre. Créer un projet invalide
`['projects']` et laisse `['dashboard','projects',…]` périmé (mitigé par
`staleTime: 30_000`, mais c'est un pansement).

Même chose dans `roles` (`useProjectDomain`), et 27 imports profonds
cross-feature au total (§ 3.6).

**Correction** : une feature consomme les *hooks* d'une autre, jamais son
accesseur DI. Concrètement `dashboard` devrait appeler `useProjects()` /
`usePipelinesMetadata()`, et si ces hooks ne conviennent pas (pagination
différente, fan-out), c'est à `project` et `pipeline` d'exposer la variante dans
leur API publique. La règle est facile à rendre exécutable :

```js
{
  name: 'domain-accessor-is-private',
  severity: 'error',
  from: { path: '^src/modules/features/([^/]+)/' },
  to:   { path: '^src/modules/features/(?!$1)[^/]+/presentation/hooks/use-[^/]+-domain\\.ts$' },
}
```

### 3.6 🟠 L'étape 8 n'est pas faite — la garantie centrale du refacto n'est pas garantie

`refacto.md` § 7 le dit lui-même. Chiffré aujourd'hui :

- **1 feature sur 14** a un `index.ts` (`membership`).
- **27 imports profonds cross-feature** subsistent, tous dans les internes d'un
  autre module (`domain/entities`, `presentation/hooks`, `presentation/ui`).

Répartition :

| Depuis | Vers (profond) |
|---|---|
| `membership` | `roles` ×6, `organization` ×2, `project` ×1, `user` ×1 |
| `dashboard` | `project` ×3, `agents` ×2, `pipeline` ×2 |
| `pipeline` | `jobs` ×4, `secret` ×1, `agents` ×1 |
| `roles` | `user` ×2, `organization` ×2, `project` ×2 |
| `organization` | `user` ×4 (dont une **page** : `UserSettings.page.tsx`) |
| `jobs` | `agents` ×1 |

Le graphe est acyclique, donc rien ne casse aujourd'hui. Mais la seule chose qui
empêche un contributeur de recréer un cycle demain est `check-module-cycles.mjs`
— un script maison qui, comme `refacto.md` le note honnêtement, existe *parce
que* les imports ne passent pas par des barrels. Le contrat est vérifié a
posteriori (« ce chemin crée-t-il un cycle ? ») au lieu d'être impossible à
violer (« ce symbole est-il exporté ? »).

Deuxième conséquence, plus insidieuse : **la surface publique de chaque feature
est aujourd'hui égale à tous ses fichiers**. Sur un projet open-source, ça veut
dire que déplacer n'importe quel fichier interne est un breaking change potentiel
pour une PR en cours.

Note au passage : le seul `index.ts` existant, `features/membership/index.ts`,
**exporte ses deux pages**. C'est exactement le piège que `CLAUDE.md` documente
pour le registry — un barrel qui réexporte de l'UI tire les pages dans le chunk
de quiconque l'importe. Il ne mord pas aujourd'hui (personne n'importe ce
barrel), mais il donne le mauvais modèle pour les 13 autres. Un `index.ts` de
feature devrait exporter des types, des hooks et éventuellement des composants
*composables* — jamais une page, qui n'a qu'un consommateur : son propre `lazy`.

### 3.6bis ✅ `check-module-cycles.mjs` n'est **pas** devenu redondant — mesuré

`refacto.md` § 5 prévoyait de supprimer le script une fois l'étape 8 faite, au motif qu'« un cycle
de dossier *devient* un cycle de fichiers ». **C'est faux**, et la sonde le montre. Cas testé :

- `features/secret/…/__a.ts` (exporté par le barrel de secret) importe `@/modules/features/user` ;
- `features/user/…/__b.ts` (**non** exporté par le barrel de user) importe `@/modules/features/secret`.

Il y a bien un cycle **de modules** `secret ⇄ user`, mais aucun cycle **de fichiers** : le barrel de
`user` ne mène jamais à `__b.ts`. Résultat :

```
depcruise (no-circular)   ✔ no dependency violations found     ← rate le cycle
depcruise:cycles          ✗ features/secret -> features/user
                            features/user   -> features/secret  ← le trouve
```

Le script est donc **conservé** et `CLAUDE.md` documente désormais pourquoi les deux commandes ne
font pas double emploi. Le supprimer aurait été une régression silencieuse.

### 3.7 🟡 Le contrat `ScyllaModule` a quatre trous

Le contrat est la meilleure idée du refacto. Ces trous valent d'être bouchés
maintenant, tant que 14 modules seulement l'implémentent.

**a) `mount` est obligatoire sur les routes enfants, où il ne veut rien dire.**

```ts
// organization.module.ts
{ mount: 'organization', path: 'users',
  children: [ { mount: 'organization', path: ':userId', … } ] }
//             ^^^^^^^^^^^^^^^^^^^^^^ ignoré par toRouteObject, obligatoire par le type
```

`routesFor` ne filtre que le niveau racine et `toRouteObject` jette le `mount`
des enfants. Le type devrait le refléter :

```ts
interface ModuleRouteNode { path?; index?; permission?; breadcrumb?; lazy?; children?: ModuleRouteNode[] }
interface ModuleRoute extends ModuleRouteNode { mount: RouteMount }
```

**b) `mergeSharedParents` ne fusionne qu'au premier niveau.** `routesFor`
l'applique à la liste racine ; les `children` sont concaténés sans être
re-fusionnés. Si deux modules déclaraient un jour `users/:userId/a` et
`users/:userId/b`, on obtiendrait deux `:userId` frères et react-router prendrait
le premier — silencieusement. Le correctif est une ligne :

```ts
existing.children = mergeSharedParents([...(existing.children ?? []), ...(route.children ?? [])]);
```

**c) La fusion des `handle` peut effacer une permission.** Le commentaire
(`compose-module-routes.ts:61-62`) promet :

> Whichever contributor declared a handle keeps it; **defined values win** so the
> merge does not depend on registry order.

Le code ne fait pas ça :

```ts
existing.handle = { ...(route.handle ?? {}), ...(existing.handle ?? {}) };
```

Comme `toRouteObject` écrit toujours **les deux clés** (`handle: { permission, breadcrumb }`,
même quand l'une vaut `undefined`), un `existing.handle.permission === undefined`
**écrase** un `route.handle.permission === X`. Ce n'est pas « defined values
win », c'est « existing wins », et ça dépend bel et bien de l'ordre du registry —
lequel, dit `registry.ts:20-21`, est modifiable pour régler l'ordre de la
sidebar.

Scénario : deux modules contribuent au même chemin, le premier déclare seulement
un breadcrumb, le second déclare `permission`. **La permission disparaît, la
route devient ouverte, aucun test ne le voit.** Ça ne se produit pas aujourd'hui
(vérifié : `organization` contribue `users` sans handle du tout), mais c'est une
mine amorcée dans le composant de routage qui a été entièrement réécrit et qui
n'a **aucun test**.

Correctif :

```ts
const mergeHandle = (a = {}, b = {}) => ({
  permission: a.permission ?? b.permission,
  breadcrumb: a.breadcrumb ?? b.breadcrumb,
});
```

**d) Pas de slot « bootstrap ».** `Layout.tsx` appelle `usePermissionSync()`
importé de `features/roles`, et importe aussi `useOrganizations`,
`useCreateOrganization`, `OrganizationList`, `AddOrganizationDialog`. Le shell
connaît donc en dur trois features (`organization`, `user`, `roles`). C'est
autorisé (le shell peut tout importer) mais ça contredit la promesse du contrat :
un module ne peut pas déclarer « j'ai besoin qu'on monte ce hook au démarrage ».
Un champ `bootstrap?: () => void` sur `ScyllaModule`, appelé par `Layout` en
boucle, supprimerait le cas `roles` proprement. Les deux autres (`organization`
dans la sidebar, `user` dans `NavUser`) sont des cas de composition légitimes,
mais mériteraient de passer par les `index.ts` du § 3.6.

Même remarque pour `core/presentation/ui/router/{OrganizationRedirect,OrganizationSync,ContextCleaner}.wrapper.tsx`,
qui importent `useOrganizations` et `useProjects` en profondeur : les 4 valeurs
de `RouteMount` sont un enum figé dans `platform/`, donc ajouter un scope
(p.ex. `/settings/*`) demande de toucher `platform` **et** `core`. Acceptable à
14 modules, à surveiller.

### 3.8 🟡 La DI est stringly-typed et repose sur un cast

```ts
// platform/di/use-module-domain.ts:18-24
const domain = registry[moduleId];
if (domain == null) throw new Error(`No module registered under id "${moduleId}"`);
return domain as TDomain;
```

Le raisonnement pour ne pas typer le registry est **correct** (un type global
`Dependencies` recouplerait toutes les features), et le cast est confiné à un
site par feature. Mais deux conséquences non traitées :

- **L'id est une chaîne libre.** `useModuleDomain<typeof SecretModule.domain>('secret')` :
  si l'id du module change, le cast reste valide au compile-time et le hook
  *throw à l'exécution*, sur la page concernée seulement. `ScyllaModule.id`
  devrait être un littéral (`id: 'secret'` avec `readonly id: string` → il l'est
  déjà grâce à `satisfies`) et l'accesseur devrait le lire du module plutôt que
  de le réécrire : `useModuleDomain<typeof SecretModule.domain>(SECRET_MODULE_ID)`
  où `SECRET_MODULE_ID` est exporté à côté du module (une constante, pas le
  module entier, pour ne pas tirer l'infra).
- **Rien ne vérifie qu'un module enregistré est unique.** `Object.fromEntries`
  (`registry.ts:46`) écrase silencieusement un doublon d'id. Trois lignes de garde
  au moment de la construction règlent ça.

### 3.9 🟡 Zéro test, sur du code qui vient d'être entièrement réécrit

`refacto.md` le liste, mais je le remonte parce que c'est la conséquence la plus
lourde de tout ce qui précède : `compose-module-routes.ts` contient un bug latent
(§ 3.7c) que dix lignes de test auraient attrapé, et `canAccess`
(`effective-permissions.entity.ts`) implémente la sémantique
SYSTEM ⊃ ORGANIZATION ⊃ PROJECT + la règle `IMPLIED_BY` — c'est-à-dire de la
logique d'autorisation, pure et sans dépendance, donc *triviale* à tester, et
non testée.

Priorité de test, dans l'ordre du rapport risque/effort :

1. `canAccess` — pure, 0 dépendance, sémantique de sécurité. ~30 assertions.
2. `compose-module-routes` — `routesFor` par mount, `mergeSharedParents`
   (y compris le cas c ci-dessus), la récursion.
3. `RouteGuard` — le `.at(-1)`.
4. `ScyllaResult` — `map`/`fold`/`unwrap`/`tryAsync`.

Vitest + testing-library, aucun besoin de MSW pour ces quatre. C'est un
après-midi et ça couvre exactement ce que le refacto a réécrit à neuf.

---

## 4. Incohérences résiduelles et petits défauts

Aucun n'est structurel, mais sur un projet open-source, chacun est une question
que se posera un contributeur.

### 4.1 La documentation de référence est périmée

`CLAUDE.md:3` dirige les contributeurs vers `docs/architecture.md` et
`docs/naming-conventions.md` comme « full references ». **Les deux décrivent
l'architecture d'avant** :

| Fichier | Contenu périmé |
|---|---|
| `docs/architecture.md:55,88,94` | module `permission` « exposed in DI as `authz` », `useDependencies().<key>` |
| `docs/architecture.md:130` | `domain/usecases/` (renommé `use-cases/` à l'étape 2) |
| `docs/architecture.md:161,271-273,287,295` | `GetUsersUseCase`, `Component → useQuery → UseCase.execute() → Repository` |
| `docs/naming-conventions.md:26,107,201-203,258` | idem, plus `{Entity}RepositoryImpl` |

Pour un dépôt ouvert, c'est le pire type de dette : la doc d'accueil contredit
les règles que la CI applique. Soit on les réécrit, soit on les supprime et
`CLAUDE.md` devient la référence unique. Vu que `CLAUDE.md` fait déjà 300 lignes
et couvre tout, **je recommande la suppression** — une seule source de vérité,
c'est aussi le principe du reste du refacto.

Même chose, en plus petit, pour les commentaires devenus faux dans le code :

| Fichier | Dit | Réalité |
|---|---|---|
| `.dependency-cruiser.cjs:4-6` | « CI runs this in reporting mode until the module graph is cycle-free, then it becomes a gate » | C'est **déjà** un gate |
| `platform/di/use-module-domain.ts:5` | « Reads one module's **use cases** » | des repositories (étape 12) |
| `platform/di/dependencies.context.ts:5,11` | « that module's `domain` (its **use-case instances**) », « see each feature's `di/use-*-domain.ts` » | repositories ; `di/` n'existe plus dans les features (étape 11) |
| `platform/routing/scylla-module.struct.ts:69` | « Declared in `di/<feature>.module.ts` » | à la racine du module (étape 11) |
| `platform/di/Dependencies.provider.tsx:5` | « see `core/di/dependencies.ts` » | `core/di/registry.ts` |
| `core/di/registry.ts:45` | « Module id -> its **use cases** » | repositories |

Le refacto a supprimé les use cases mais pas les six commentaires qui les
nomment.

### 4.2 Convention de nommage : 4 emplacements pour la même chose

`CLAUDE.md` dit : data source impl = `*.data-source.impl.ts` dans
`infrastructure/data/remote/`. Réalité sur 12 features :

| Forme | Features | Conforme ? |
|---|---|---|
| `infrastructure/data/remote/user-remote.data-source.impl.ts` | `user` | ✅ (1/12) |
| `infrastructure/data/remote/grpc-<x>-remote.data-source.ts` | `jobs`, `login`, `pipeline`, `triggers` | ❌ |
| `infrastructure/data/grpc-<x>-remote.data-source.ts` | `organization`, `project`, `roles`, `secret` | ❌ |
| `infrastructure/data/<x>-remote.data-source.ts` | `agents`, `apps` | ❌ |

Autres écarts vérifiés :

- **5 fichiers en camelCase** alors que la règle est kebab-case :
  `project/presentation/hooks/useProjects.ts`, `useCreateProject.ts`,
  `organization/.../useOrganizations.ts`, `useCreateOrganization.ts`,
  `secret/presentation/utils/createSecretItems.ts`.
- **`ProjectPage.tsx`** — seul page component sur 18 à ne pas suivre
  `*.page.tsx`.
- **`default-organization.repository.ts`** — seul repository en `export default`.
- **`roles/domain/repository/permission.repository.ts` →
  `PermissionRepository`**, et `infrastructure/repository/default-permission.repository.ts` :
  reliquat du renommage `permission` → `roles`. Le module s'appelle `roles`, le
  repository s'appelle `permission`, alors que `Permission` désigne autre chose
  et vit dans `platform/authz`. C'est précisément la collision que le renommage
  visait à supprimer (`refacto.md` § 5).
- **`shared/domain/types/paginated-list.type.ts`** — suffixe `.type.ts` absent de
  la table de nommage, à côté de `shared/domain/structs/pagination.struct.ts`.
  Deux conventions pour deux fichiers voisins.
- **`features/pipeline/domain/.gitkeep`** — reliquat, le dossier n'est plus vide.
- **`secret/infrastructure/data/grpc-credential-remote.data-source.ts`** — nommé
  `credential` dans la feature `secret`.
- **`UserRemoteDataSourceImpl`** — seule classe en `*Impl`, les 11 autres sont
  `Grpc*`.

Rien de grave individuellement. Cumulé, ça veut dire qu'un contributeur qui
copie le module voisin a **une chance sur quatre** de tomber sur la forme
canonique.

### 4.3 Les clés de query : trois styles coexistent, malgré une règle explicite

`CLAUDE.md` : « Always use **factory functions** ». Réalité :

| Style | Occurrences | Exemples |
|---|---|---|
| Factory ✅ | 8 | `JOBS_QUERY_KEY(pipelineId)`, `TRIGGERS_QUERY_KEY(id)` |
| Constante-racine épandue | 5 | `[WORKERS_QUERY_KEY, organizationId]`, `[APPS_QUERY_KEY, …]`, `[SECRETS_QUERY_KEY, projectId]` |
| Littéral inline | ~15 | `['organizations','mine']`, `['users']`, `['user', userId]`, `['projects']`, `['pipelines', projectId, paginationParams]`, `['marketplace']` |

Conséquences concrètes :

- **`JOBS_QUERY_KEY` a été déplacé dans `jobs` à l'étape 6 pour casser le cycle,
  mais deux appelants réécrivent la clé à la main** :
  `triggers/.../use-fire-trigger-now.ts:22` et `jobs/.../use-delete-jobs.ts:12`
  font `['jobs','pipeline', pipelineId]`. Ça marche par préfixe, mais c'est
  exactement la dérive que la factory existe pour empêcher — et la factory
  contient un 4ᵉ segment (`MAX_JOBS_PER_PIPELINE`), donc les deux formes ne sont
  déjà plus identiques.
- **Deux `SECRETS_QUERY_KEY` distincts** dans deux fichiers
  (`secret/.../use-secrets.ts:9` = `'secrets'`,
  `apps/.../use-apps.ts:97` = `'app-secrets'`). Même nom, sens différent.
- **Noms périmés** : `WORKERS_QUERY_KEY = 'agents'`
  (`agents/.../use-agents.ts:7`) et `ROLES_QUERY_KEY = 'permission-roles'`
  (`roles/.../use-roles.ts:6`) — reliquats de renommages.
- **Deux caches pour la même ressource** : `['projects']` (feature `project`) et
  `['dashboard','projects',orgId]` (feature `dashboard`, § 3.5), qui ne
  s'invalident pas mutuellement.

Une convention unique — un fichier `<feature>.query-keys.ts` par feature,
uniquement des factories, réexporté par l'`index.ts` du § 3.6 — règle les
quatre points d'un coup, et rend l'invalidation cross-feature légale au lieu
d'être une chaîne devinée.

### 4.4 Duplication non détectée : le graphe d'outcomes existe deux fois

`features/dashboard/presentation/ui/AgentOutcomesChart.tsx` (278 lignes) et
`features/agents/presentation/ui/components/OutcomesChart.tsx` (229 lignes) sont
le même graphe : mêmes plages `7d/14d/30d`, même `RANGE_DAYS`, même
`CHART_CONFIG` (`completed`/`failed`/`cancelled`, mêmes couleurs), même
`AreaChart` recharts. Seule différence réelle : l'un récupère ses données
(`useStats`), l'autre les reçoit en props.

C'est le cas d'école de `CLAUDE.md` : « A component used by ≥ 2 features moves up
to `shared/presentation/ui/` ». La version props-only est la bonne ; le dashboard
devrait la nourrir avec son propre hook. ~250 lignes en moins et un seul endroit
où corriger un bug d'affichage.

### 4.5 Clés de sélection dupliquées en chaînes magiques

`useSelection(key)` / `useFeatureSelection(key, …)` sont bien conçus, mais la clé
est écrite à la main dans **deux fichiers différents** par feature — le header et
la table :

```
PipelineDashboardHeader.tsx:23  useFeatureSelection('pipelines', …)
PipelineTable.tsx:26            useSelection('pipelines')
TriggersHeader.tsx:24 / TriggersTable.tsx:20        'triggers'
UserAdmin.page.tsx:20 / UserTable.tsx:12            'users'
SecretHeader.tsx:22   / SecretList.tsx:19           'secrets'
JobsHeader.tsx:32     / JobsTable.tsx:22            'jobs'
ProjectHeader.tsx:18  / ProjectCard.tsx:22          'projects'
```

Une faute de frappe casse la sélection **silencieusement** (deux slices
distinctes dans le store, la case cochée n'apparaît pas dans le header). Une
constante exportée par feature, ou un type union `SelectionKey`, supprime la
classe entière.

### 4.6 Le bug d'authentification streaming est toujours là

Signalé par `refacto.md` § 7 comme « hors périmètre ». Confirmé présent
(`platform/grpc/scylla-grpc-transport.ts:52-64`) :

```ts
const authInterceptor: RpcInterceptor = {
  interceptUnary(next, method, input, options: RpcOptions) { /* pose Authorization */ },
  // ← pas d'interceptServerStreaming
};
```

Alors que `errorDecodeInterceptor`, juste au-dessus dans le même fichier,
implémente **les deux**. `tailJobLogs` part donc sans en-tête `Authorization`.
Le correctif est de factoriser la pose du header et de le brancher sur les deux
méthodes — ~8 lignes. Ça mérite sa propre PR et un test manuel sur les logs
temps réel.

### 4.7 `shared/` connaît la topologie des features par un glob

`shared/presentation/utils/i18n.ts:20` :

```ts
const catalogs = import.meta.glob<{ messages: Messages }>('../../../**/locales/*/messages.ts');
```

Résolu depuis `src/modules/shared/presentation/utils/`, ça balaye
`src/modules/**` — donc `shared/` scanne `features/`. `depcruise` ne le voit pas
(un glob n'est pas un import statique), mais c'est bien `shared-is-generic` qui
est enfreint en esprit. Le fichier appartient à `platform/i18n/` : c'est
exactement le profil d'une capacité transverse (comme `platform/grpc`), et ça
rendrait aussi la génération de `lingui.config.js` du § 2.1 naturelle.

---

## 5. Aurait-on pu faire mieux ? Les alternatives, honnêtement

### 5.1 Les packages pnpm — la décision est bonne, et pour la bonne raison

Je confirme le raisonnement de `refacto.md` § 3.1. J'ajoute deux choses :

- Le vrai argument, pas assez mis en avant, est que **`index.ts` + une règle
  `eslint-plugin-boundaries` donnent 100 % de la garantie recherchée pour 0 % du
  coût de build graph**. Un `package.json` par feature n'ajoute rien qu'une règle
  de lint ne fasse — sauf la publication npm, qui n'est pas au programme.
- Le déclencheur à surveiller est bien celui listé (2ᵉ app front), mais j'en
  ajouterais un : **si le temps de `pnpm build` en CI dépasse ~5 min**, le cache
  incrémental de Turborepo commence à payer. Aujourd'hui, non.

### 5.2 Le vrai découpage manquant est vertical, pas horizontal

Le refacto a beaucoup travaillé l'axe **horizontal** (4 couches, direction des
dépendances) et presque pas l'axe **vertical** (qu'est-ce qu'un module,
au juste ?). Deux symptômes :

- **`dashboard` et `membership` ont `domain: {}`.** Ce sont des modules de *vue
  composite* : ils n'ont pas de données propres, ils agrègent celles de 3-4
  autres features. Ils ne sont pas au même niveau conceptuel que `secret` ou
  `jobs`, et c'est ce qui produit les 27 imports profonds du § 3.6 (à eux deux,
  ils en concentrent 16).

  Une lecture plus juste : **il existe une couche entre `features/` et `app/`**,
  celle des pages qui composent plusieurs domaines. Vous pouvez soit l'assumer
  (`views/` ou `features/_composite/`, autorisé à importer les `index.ts` de
  plusieurs features, importé par personne), soit les remonter dans le shell.
  Aujourd'hui elle est déguisée en feature ordinaire, ce qui rend la règle
  « une feature n'importe pas les internes d'une autre » impossible à appliquer
  uniformément — et c'est probablement pourquoi l'étape 8 est restée en plan.

- **`membership` n'a ni repository ni infrastructure** (`domain/` contient un
  seul `*.struct.ts`). Le découpage du god-module `permission` était le bon
  geste, mais `membership` est sorti comme une feature *sans domaine* qui tape
  sur `roles` ×6, `organization` ×2, `project` ×1, `user` ×1. Si « qui appartient
  à quoi » est vraiment un domaine à part entière — et je pense que oui —, il
  devrait avoir son repository (`listOrganizationMembers`, `listProjectMembers`)
  et son mapper, plutôt que de recomposer côté présentation à partir des
  repositories de quatre voisins.

### 5.3 Ce qu'il ne faut PAS faire

Pour équilibrer : trois choses que j'ai vu proposer dans ce genre de revue et qui
seraient des régressions ici.

- **Ne pas réintroduire les use cases.** La décision de l'étape 12 est correcte
  et bien argumentée. Le critère écrit dans `CLAUDE.md` (« only when it does
  something a repository method cannot ») est le bon. Appliquez-le *aussi* à la
  couche data source (§ 3.1), n'en revenez pas.
- **Ne pas typer globalement le `DomainRegistry`.** Le cast de
  `use-module-domain.ts` est laid mais correct : un type global recouplerait
  toutes les features, ce que l'étape 4 a explicitement corrigé. Améliorez l'id
  (§ 3.8), pas le registry.
- **Ne pas ajouter d'`index.ts` qui exportent des pages.** Voir § 3.6 : le seul
  barrel existant fait cette erreur, ne la propagez pas aux 13 autres.

---

## 6. Plan d'action priorisé

Classé par **(valeur pour un projet open-source qui grossit) ÷ (effort)**.
Les cinq premiers items sont ceux que je ferais avant d'ouvrir les
contributions externes.

| # | Action | § | Effort | Pourquoi maintenant |
|---|---|---|---|---|
| 1 | Réécrire ou **supprimer** `docs/architecture.md` + `docs/naming-conventions.md`, et corriger les 6 commentaires périmés | 4.1 | 1 h | La doc d'accueil contredit la CI. Sur un dépôt ouvert, c'est la règle qui perd. |
| 2 | Corriger le bug de fusion des `handle` + rendre `mergeSharedParents` récursif | 3.7b-c | 1 h | Une permission de route peut disparaître silencieusement, dans du code sans test. |
| 3 | Tests Vitest sur `canAccess`, `compose-module-routes`, `RouteGuard`, `ScyllaResult` | 3.9 | 1 j | Couvre exactement ce que le refacto a réécrit à neuf. Rend le reste de cette liste sûr. |
| 4 | Créer `platform/session/` et y ramener les 12 accès `localStorage` | 3.3 | 0,5 j | Meilleur rapport gain/effort de la liste. Débloque aussi la déconnexion sans reload. |
| 5 | Fixer l'intercepteur d'auth streaming | 4.6 | 15 min | Bug fonctionnel réel sur `tailJobLogs`. PR indépendante. |
| 6 | **Étape 8** : `index.ts` par feature + `eslint-plugin-boundaries` en `error` | 3.6 | 2-3 j | C'est ce qui rend le refacto irréversible. Sans ça, tout le reste est une photo. |
| 7 | Supprimer la couche data source pass-through (5 features), fusionner le `.map()` dans le repository (6 autres) | 3.1 | 1-2 j | Applique votre propre critère. −17 fichiers, −11 interfaces. |
| 8 | Rendre `domain` paresseux → sortir les 12 clients gRPC du chunk d'entrée | 3.2 | 1 j | Mesurer d'abord le gain réel ; ouvre la voie aux modules optionnels. |
| 9 | `useCan` sur TanStack Query, ou assumer et amender `CLAUDE.md` | 3.4 | 0,5 j | Supprime `usePermissionSync`, son `useRef`-cache et sa contrainte de mount. |
| 10 | Une convention unique de query keys + une factory par feature | 4.3 | 0,5 j | Supprime les 2 caches concurrents et les clés réécrites à la main. |
| 11 | Interdire `use-*-domain.ts` cross-feature (règle depcruise), corriger `dashboard` et `roles` | 3.5 | 0,5 j | Trois lignes de config pour fermer un trou définitivement. |
| 12 | Fusionner les deux `OutcomesChart` dans `shared/` | 4.4 | 2 h | −250 lignes. |
| 13 | Normaliser le nommage (5 camelCase, `ProjectPage`, `permission.repository` → `role.repository`, emplacement des data sources, `.gitkeep`) | 4.2 | 0,5 j | À faire **avec** l'item 7, qui touche déjà ces fichiers. |
| 14 | Générer `lingui.config.js` par glob ; déplacer `i18n.ts` dans `platform/i18n/` | 2.1, 4.7 | 2 h | Supprime la dernière liste manuelle. |
| 15 | `mount` hors du type des routes enfants ; slot `bootstrap?` sur `ScyllaModule` ; garde anti-doublon d'id | 3.7a, 3.7d, 3.8 | 0,5 j | Tant qu'il n'y a que 14 implémenteurs du contrat. |
| 16 | Décider du statut de `dashboard`/`membership` (couche composite assumée, ou domaine propre pour `membership`) | 5.2 | discussion | Prérequis conceptuel de l'item 6 : sans ça, l'étape 8 rebutera sur ces deux modules. |

**Ordre suggéré** : 1 → 5 → 2 → 3 → 4 → 16 (décision) → 6 → 7+13 → 11 → 10 → 9 → 8 → 12 → 14 → 15.

L'item 16 est placé avant le 6 exprès : décider ce qu'est `dashboard` *avant*
d'écrire les `index.ts`, sinon vous écrirez des barrels qui exportent des
internes juste pour satisfaire `dashboard`, et l'étape 8 perdra son sens.

---

## 7. Réponse directe à la question posée

> **Ce refacto était-il vraiment utile ?**

Oui, sans ambiguïté. Passer d'une SCC de 16 modules à un graphe acyclique
machine-vérifié n'est pas une amélioration cosmétique : c'est la différence entre
un projet où un contributeur externe peut travailler sur un coin isolé, et un
projet où il doit tout comprendre. Ajouté à la première CI frontend et à la
suppression de 64 indirections vides, c'est le travail de fondation qu'il fallait
faire, et il a été fait avec un raisonnement solide — les huit « pièges »
documentés dans `refacto.md` montrent que les décisions ont été prises en
comprenant le problème, pas en appliquant un patron.

> **Y a-t-il des choses qui auraient pu être mieux faites ?**

Trois, par ordre d'importance :

1. **Le critère qui a tué les use cases n'a pas été appliqué une couche plus
   bas.** La paire `repository ↔ data source` est aujourd'hui le même
   pass-through, à l'identique, dans 5 features sur 12 — et la doc décrit un
   partage des rôles (« le repository mappe ») que le code ne respecte pas. Le
   refacto a eu la bonne intuition et s'est arrêté un cran trop tôt.
2. **Deux violations ont été déplacées plutôt que corrigées** : le state serveur
   dans Zustand (§ 3.4, présenté comme corrigé alors qu'il a changé de fichier)
   et la session en `localStorage` brut (§ 3.3, jamais adressée alors que c'était
   la capacité `platform/` la plus évidente à créer).
3. **L'étape 8 manquante enlève sa valeur de garantie au reste.** Le graphe est
   acyclique *aujourd'hui*, mais 27 imports profonds cross-feature signifient que
   la surface publique de chaque module est encore « tous ses fichiers ». Le
   refacto est une photo, pas encore un contrat.

> **Une séparation plus modulaire aurait-elle été préférable ?**

Pas au sens « packages pnpm » — cette décision-là est correcte et bien
argumentée. Mais oui au sens **vertical** : la vraie question non traitée est
qu'est-ce qu'un module. `dashboard` et `membership` sont des vues composites sans
domaine propre, déguisées en features, et à elles deux elles concentrent 16 des
27 imports profonds. Tant qu'elles restent au même niveau que `jobs` ou `secret`,
la règle « une feature ne touche pas les internes d'une autre » restera
inapplicable — et c'est très probablement la raison de fond pour laquelle
l'étape 8 n'a pas pu être terminée.
