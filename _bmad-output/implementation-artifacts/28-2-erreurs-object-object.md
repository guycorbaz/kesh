# Story 28.2 : Une erreur d'API ne s'affiche plus « [object Object] » — un utilitaire unique rend le message

## Status

**review** — développement fait le 2026-10-13 (`bmad-dev-story`, Opus 5.5, dernier commit de code `4ff3a11e`) ;
revue de code P1 à lancer : `28-2-review-prompt-p1.md` (Sonnet, lentilles B, E, A). Validation close le 2026-10-13
(P3 ciblée : 0 MEDIUM, 2 LOW appliqués). Fiche créée le 2026-10-13 en autonomie (`bmad-create-story`, Opus 5.5), sur `6f971d76`
(rétrospective de l'Epic 27, #641), branche `story/28-2`, worktree `/home/gcorbaz/devel/kesh-28-2`. Port **3035**,
bases `kesh_282` / `kesh_e2e_282`. Choix consignés au registre de l'epic `epic-28-choix-autonomes.md`
(§ Story 28-2, versé par union à la validation P2), **C-28-2-1 à C-28-2-29**. 11 AC (AC1 à AC11).
Validation P1 faite (Sonnet, lentilles R et F : 7 MEDIUM, 11 LOW, remédiés) ; validation P2 faite (Opus, lentilles
R et F : 3 MEDIUM rapportés, 2 distincts — F1 = R2 et R1 — ; 18 LOW rapportés, 13 distincts ; remédiés) ; passe ciblée P3 à lancer :
`28-2-validate-prompt-p3-ciblee.md` (Haiku, une lentille, diff de la remédiation P2). PR : **`closes #520`**,
**`refs #644`** (classe D).

## Story

**As a** comptable qui utilise Kesh,
**I want** que toute erreur renvoyée par le serveur s'affiche par son message — traduit dans la langue de
l'installation dès que le catalogue est chargé (§ Angles morts, n° 5) —, et qu'une panne imprévue s'affiche par un
message générique traduit,
**so that** je ne lise plus jamais « [object Object] » à la place de la raison du refus, et que je sache quoi
corriger.

Issue : [#520](https://github.com/guycorbaz/kesh/issues/520) (bug, **P2**), relevée par la remédiation de la passe P1
de validation de la Story 15-5 (Epic 15). Epic 28 (cible **v0.13.3**, kickoff du 2026-10-13 ; rétrospective de
l'Epic 27, action B1 : #640, #520, #526, #482, #548). La rétrospective de l'Epic 27 note (§ 9) : *« #520 est un
inventaire : la règle "inventorier les sites non résolus" s'applique »*.

### Le défaut, au sol

`ApiError` est une **interface** (`frontend/src/lib/shared/types/api.ts:9-14`), pas une sous-classe d'`Error` : le
client (`api-client.ts`) lève des **objets simples** `{ code, message, details?, status }`. Le motif
`e instanceof Error ? e.message : String(e)` rend donc `String({…})` = **« [object Object] »** pour toute erreur de
l'API. La Story 15-5b a créé l'utilitaire `errorMessageOf` (`api-client.ts:30-41`) et l'a posé sur les sept sites du
module rapprochement (C-15-5b-1) ; elle a laissé, hors module, `settings/+page.svelte:236/:260` — suivi par #520
(`15-5b-gardes-surfaces-neuves.md:979`).

Trois constats, au sol, à `6f971d76` :

1. **L'issue est en partie périmée.** `reports/+page.svelte:210` (cité par l'issue) est aujourd'hui gardé par
   `isApiError` (`formatError`, `:196-216`) : il ne rend plus « [object Object] » pour une `ApiError`, mais garde
   le repli `err instanceof Error ? err.message : String(err)` (`:215`). Les sites du rapprochement cités par l'issue
   sont traités (15-5b), sauf `ManualMatchModal.svelte:153`, `TransactionSplitModal.svelte:216` et
   `CancelReconciliationDialog.svelte:60/:97`, qui gardent un repli `String`.
2. **Quatre sites rendent encore « [object Object] » pour une `ApiError`** (classe A ci-dessous), dont deux que
   l'issue ne cite pas (`BankProfileList.svelte:19`, `bank-import/profiles/[id]/+page.svelte:25`).
3. **`errorMessageOf` lui-même n'est ni traduit ni testé** : il rend `err.message` d'une `Error` (texte technique,
   souvent anglais : « Failed to fetch », « No candidate for txId=5 »), `String(err)` sinon (« [object Object] »
   pour un objet qui n'est pas une `ApiError`) ; `grep -rn "errorMessageOf" frontend/src --include=*.test.ts` ne rend
   **rien**. Et les messages que le **client** fabrique (`NETWORK_ERROR`, `TIMEOUT`, `UNKNOWN_ERROR`,
   `api-client.ts:226-251` et `:288-296`) sont écrits **en français en dur** : un germanophone lit « Impossible de
   contacter le serveur » — sur chacune des **177** lectures de `e.message`/`err.message`/`error.message` du
   frontend (`grep -rnE "\b(e|err|error)\.message\b" frontend/src --include=*.svelte --include=*.ts | grep -v "\.test\." | wc -l`),
   pas seulement sur les sites de cette story.

## Critères d'acceptation

### AC1 — Un utilitaire unique, hors du client HTTP

`errorMessageOf(err: unknown): string` vit dans un module neuf **`frontend/src/lib/shared/utils/error-message.ts`**
(C-28-2-2), documenté (`///`-équivalent JSDoc, le motif de l'issue cité). Il est **retiré** de `api-client.ts`, qui
ne le réexporte pas ; ses quatre importeurs l'importent du module neuf. Règles, dans cet ordre :

| entrée | rendu |
|---|---|
| `ApiError` (`isApiError`) dont `message` est une chaîne non vide après `trim()` | `err.message`, tel quel (le serveur l'a déjà traduit, `crates/kesh-api/src/errors.rs:31-60`) |
| `ApiError` dont `message` est vide ou blanc, **ou n'est pas une chaîne** (`undefined`, nombre, objet : `isApiError`, `api-client.ts:19-28`, ne vérifie que `code` et `status` — C-28-2-24) | `i18nMsg('error-unexpected', 'Erreur inattendue.')` |
| toute autre valeur — `Error`, `TypeError`, `DOMException`, chaîne, nombre, `null`, `undefined`, objet quelconque | `i18nMsg('error-unexpected', 'Erreur inattendue.')`, **et** `console.error('[errorMessageOf]', err)` pour le diagnostic (C-28-2-3) |

⛔ Le rendu n'est **jamais** `'[object Object]'`, jamais la chaîne vide, jamais `'undefined'`/`'null'`.
⚠️ Le repli `'Erreur inattendue.'` est **exactement** celui des **20** sites existants de `error-unexpected` (point
final compris ; `grep -rn "'error-unexpected'" frontend/src … | grep -v "'Erreur inattendue.'"` ne rend rien) : `i18n-un-repli-par-cle.test.ts` exige un repli unique par clé.

`isApiError` est **extrait** dans `frontend/src/lib/shared/utils/api-error.ts` (sans autre import qu'un `import type`)
et **réexporté** par `api-client.ts` (`export { isApiError } from './api-error'`) — ses **53** fichiers
importeurs existants (**164** appels hors tests — 166 occurrences de `isApiError(`, dont la déclaration
`api-client.ts:19` et le doc-comment `notify.ts:89` ; recompté en validation P3, la P2 écrivait « 166 appels », la
rédaction antérieure « ~110 » ; commande : `git grep -nE "isApiError\(" -- frontend/src | grep -vE "\.test\.ts:" |
grep -vF "export function isApiError(" | grep -vE "^\S+:[0-9]+:\s*(\*|//)" | wc -l`) ne changent pas ; `error-message.ts` l'importe de `api-error.ts`, pour ne pas dépendre des `vi.mock` du client
(C-28-2-2).

### AC2 — Les messages que le client fabrique sont traduits à la source

Dans `api-client.ts`, les trois messages synthétisés par le client passent par `i18nMsg`, **le repli étant le texte
français actuel, à l'identique** (aucun changement en français, ni avant ni après le chargement du catalogue) :

| code | site actuel | clé | repli |
|---|---|---|---|
| `NETWORK_ERROR` | `toFetchApiError`, `:294` | `error-network` | `Impossible de contacter le serveur. Vérifiez votre connexion.` |
| `TIMEOUT` | `toFetchApiError`, `:293` | `error-timeout` | `Le serveur ne répond pas. Réessayez ultérieurement.` |
| `UNKNOWN_ERROR` | `parseErrorResponse`, `:228`, `:242` (repli seul), `:250` | `error-server-status` | `Erreur serveur ({ $status })`, `{ status: res.status }` |

Les trois occurrences d'`Erreur serveur (${res.status})` sont factorisées en **une** fonction locale
(`serverStatusMessage(status)`), soit **un** site `i18nMsg` (DRY ; C-28-2-4). Au site `:242`, seul le **repli** est
traduit : un `message` présent dans le corps de la réponse est toujours repris tel quel.
⚠️ Le repli de `error-server-status` porte la variable sous forme **Fluent** `{ $status }`, jamais en gabarit JS
`` `${status}` `` : `i18n-entrees-a-variables.test.ts` (maillons 1 et 3) le contrôle.
⚠️ `api-client.ts` importe alors `i18nMsg` de `i18n.svelte.ts`, qui importe déjà `apiClient` : **cycle d'import**.
Il est bénin (aucun des deux modules n'utilise l'autre au niveau supérieur, seulement à l'appel) — C-28-2-4 ; l'AC10
le prouve par un test qui charge le catalogue **puis** provoque un `NETWORK_ERROR` réel du client.
Hors périmètre, écrits au § Angles morts : `SETUP_REQUIRED` (`:407`, `:504`) et `UNAUTHENTICATED` (`:427`, `:521`),
qui précèdent une redirection (C-28-2-5).

### AC3 — Les quatre sites qui rendent « [object Object] » (classe A) passent par l'utilitaire

| # | site (à `6f971d76`) | avant | après |
|---|---|---|---|
| A1 | `routes/(app)/settings/+page.svelte:237` (`submitLock`) | `e instanceof Error ? e.message : String(e)` | `errorMessageOf(e)` |
| A2 | `routes/(app)/settings/+page.svelte:261` (`submitUnlock`) | idem | `errorMessageOf(e)` |
| A3 | `lib/features/bank-import/BankProfileList.svelte:19` | `` `Erreur chargement profils : ${String(err)}` `` | `errorMessageOf(err)` |
| A4 | `routes/(app)/bank-import/profiles/[id]/+page.svelte:25` | `` `Profil introuvable : ${String(err)}` `` | `errorMessageOf(err)` |

Pour A3 et A4, le préfixe français en dur disparaît (C-28-2-6) : le message du serveur (« … introuvable », traduit)
ou du client (réseau, traduit par l'AC2) se suffit, et « Profil introuvable » était **faux** pour une panne réseau.

**Même fichier qu'A3, un rejet sans aucun message** (C-28-2-20) : `BankProfileList.svelte:25-31` (`handleDelete`)
appelle `await deleteBankProfile(id)` **sans `try/catch`** — un refus de suppression devient un rejet non géré,
rien ne s'affiche. Il reçoit un `try { await deleteBankProfile(id); await load(); } catch (err) {
notifyError(errorMessageOf(err)); }` : une **notification**, et non `errorMessage`, qui remplace toute la liste
(`{:else if errorMessage}`, `:41`) et ferait disparaître les profils pour un refus ponctuel. Ce site n'est pas un
« [object Object] » (il n'a pas de `catch`, aucun détecteur de la fiche ne le voit) : il est traité parce que la
story touche ce fichier et que l'écran d'A3 serait laissé avec un second chemin muet.

### AC4 — Les seize replis `String` / `instanceof Error` (classe B) passent par l'utilitaire

Chaque site ci-dessous traite déjà l'`ApiError`, mais rend `String(x)` (ou `x.message` d'une `Error`) pour tout le
reste. Remplacement **mécanique** d'une expression par un appel pour B1-B10 et B15 ; **réécriture d'un
`if/else`** pour B11-B14 (deux branches fondues en une) ; **suppression** pour B16. Les branches spécifiques
existantes (codes, drapeaux) sont **préservées**.

| # | site | après |
|---|---|---|
| B1 | `routes/(app)/bank-accounts/+page.svelte:76-78` (`baResult.reason`, `Promise.allSettled`) | `loadError = errorMessageOf(baResult.reason)` |
| B2-B4 | `routes/(app)/bank-accounts/+page.svelte:239`, `:265`, `:281` | `errorMessageOf(err)` |
| B5 | `lib/features/bank-import/DeleteBankImportAction.svelte:62` (dernière branche) | `errorMsg = errorMessageOf(err)` |
| B6-B8 | `routes/(app)/settings/api-keys/+page.svelte:63`, `:159`, `:198` | `errorMessageOf(err)` |
| B9 | `lib/features/reconciliation/CancelReconciliationDialog.svelte:60` | `errorMessageOf(err)` |
| B10 | `lib/features/reconciliation/CancelReconciliationDialog.svelte:97` | `isApiError(err) ? reconciliationCancelErrorMessage(err) : errorMessageOf(err)` |
| B11-B12 | `lib/features/bank-accounts/BankAccountJournalLinkForm.svelte:94-99`, `:123-128` | le `if/else` devient `errorMessage = errorMessageOf(err)` |
| B13 | `lib/features/reconciliation/ManualMatchModal.svelte:150-154` | `else if (isApiError(e)) …; else …` devient `else errorMsg = errorMessageOf(e)` ; la branche `BANK_ACCOUNT_NOT_CONFIGURED` (drapeau `bankNotConfigured`) **reste** |
| B14 | `lib/features/reconciliation/TransactionSplitModal.svelte:213-217` | idem B13 |
| B15 | `routes/(app)/reports/+page.svelte:215` (`formatError`) | `return errorMessageOf(err)` ; la branche `REPORT_PERIOD_OUT_OF_FISCAL_YEAR` **reste** |
| B16 | `lib/shared/utils/api-client.ts:38-41` (`errorMessageOf` lui-même) | supprimé (AC1) |

Les quatre importeurs actuels d'`errorMessageOf` changent d'import, sans autre changement :
`routes/(app)/reconciliation/rules/+page.svelte:5`, `lib/features/reconciliation/ReconciliationProposals.svelte:22`,
`lib/features/reconciliation/rules/RulesList.svelte:7`, `lib/features/reconciliation/rules/RuleFormModal.svelte:5`.
Un import d'`isApiError` qui devient inutilisé est **retiré** (`npm run check` / ESLint le signaleraient) — au
moins celui de `BankAccountJournalLinkForm.svelte:11`, dont les deux seuls usages (`:95`, `:124`) sont B11 et B12.

### AC5 — L'ensemble des sites non résolus est clos, et chaque reste est nommé

À la fin de la story, l'inventaire du § Dev Notes *Inventaire des sites NON RÉSOLUS* est **recompté** depuis la
source (mêmes commandes), et le Dev Agent Record rend : classe A = 0, classe B = 0 ; classe C (journalisation)
= **3, légitimes par critère**, inchangées — sous l'exemption `console` de l'AC6, que ces trois sites exercent au
parcours réel en plus de ses auto-tests (C-28-2-25, qui retire C-28-2-19) ; classe D (affichage réservé aux `ApiError`, rien pour le reste) = **angle mort assumé**, comptée, suivie par
l'issue **#644** (bug, P3, ouverte par l'orchestrateur — C-28-2-22 ; la PR la cite en `refs #644`, aucune autre
issue de suivi n'est à demander) ; classe E (repli fixe pour une valeur qui n'est pas une `ApiError`) conforme, non
refactorée.

**Classe C, hors périmètre** (C-28-2-25 — *révisé en validation P2* : la remédiation P1 les faisait passer à la
valeur brute, C-28-2-19, retiré) : `hooks.client.ts:27`, `lib/app/stores/auth.svelte.ts:301`,
`lib/shared/utils/api-client.ts:162` journalisent `error instanceof Error ? error.message : String(error)`, mais
**aucune `ApiError` ne les atteint** : `auth.svelte.ts:301` et `api-client.ts:162` entourent un `fetch` **direct**
de `/api/v1/auth/me` (`auth.svelte.ts:229`, `api-client.ts:131` ; le commentaire `auth.svelte.ts:305` le dit) suivi
de `res.json()`, et ne reçoivent qu'un `TypeError`, une `DOMException` ou un `SyntaxError` — des `Error`, dont le
`message` est déjà journalisé ; `hooks.client.ts:27` n'attrape que ce qui sort de `hydrate()`, dont tout le corps
réseau est dans un `try/catch/finally` (`auth.svelte.ts:228-315`) — au plus une `DOMException` de
`localStorage.removeItem` (`:222-226`), une `Error` aussi. Rien n'y devient « [object Object] » : ces trois lignes
**ne changent pas**, et `hooks.client.ts` comme `auth.svelte.ts` restent **hors** des fichiers touchés.

### AC6 — Une garde de non-régression lit les sources

Test Vitest neuf **`frontend/src/lib/shared/erreurs-en-texte.test.ts`** (modèle : `i18n-libelle-en-dur.test.ts`,
même préambule « ce qu'elle voit / ce qu'elle ne peut pas voir », C-28-2-7). Il parcourt `src/` (`.svelte`, `.ts`,
hors `.test.` — `dansLePerimetreDeFichier` d'`i18n-harvest.js`), masque les commentaires (`masquerCommentaires`
d'`i18n-literal-reader.js`), recueille les **liaisons d'erreur** de chaque fichier — paramètre de `catch (x)` /
`catch (x: unknown)`, paramètre d'un rappel `.catch((x) => …)` / `.catch(x => …)` / `.catch(function (x) …)`, y
compris **typé** (`.catch((x: unknown) => …)`) ou **`async`** (`.catch(async (x) => …)`, `.catch(async x => …)`,
`.catch(async function (x) …)`) — ces deux dernières formes comptent 0 occurrence à `6f971d76`
(`grep -rnE "\.catch\(\s*(async\s*)?\(\s*\w+\s*:"` et `grep -rnE "\.catch\(\s*async"` vides) et sont recueillies
pour qu'une liaison future n'échappe pas au recueil (C-28-2-29) — et **échoue en nommant `fichier:ligne`** sur les
règles G1-G7 ci-dessous.

**Portée d'une liaison** (C-28-2-16) — les règles G1, G4-G7 ne sont cherchées **que dans la portée** de la liaison
qu'elles nomment :

| forme | portée |
|---|---|
| `catch (x) {` / `catch (x: T) {` | le bloc dont l'accolade ouvrante suit la parenthèse **fermante** de la liaison (et non un `{` de type inline), borné par `corpsDeFonction` (`i18n-literal-reader.js:408`), qui saute chaînes, gabarits et regex |
| `.catch((x) => {` / `.catch(function (x) {` / `.catch(x => {` | le bloc du rappel, même calcul |
| `.catch((x) => <expression>)` / `.catch(x => <expression>)` (corps-expression) | de `=>` jusqu'à la parenthèse qui **ferme** `.catch(`, par équilibrage des parenthèses avec le même saut des chaînes, gabarits et regex. ⚠️ Les outils de ce saut (`ouvreUnLitteral`, `:141`, `estDebutDeRegex`, `:183`, `finDeRegex`, `:211`) sont **privés** au module — seuls `readLiteral` et `corpsDeFonction` sont exportés : la boucle de `corpsDeFonction` est donc **généralisée** en une fonction exportée `blocEquilibre(source, ouvrante, ouvre, ferme)` (même module), dont `corpsDeFonction` devient l'appel `blocEquilibre(source, ouvrante, '{', '}')`, signature et comportement inchangés ; la garde l'appelle avec `'('`, `')'`. Pas de copie de la boucle dans la garde (DRY). Tests dans `i18n-literal-reader.test.ts` : ceux de `corpsDeFonction` (`:310-331`) restent verts **sans modification** ; `blocEquilibre` en parenthèses reçoit les mêmes cas (chaîne, regex, non équilibré → `null`) |

⛔ **Une portée incalculable fait ÉCHOUER la garde**, en nommant `fichier:ligne` de la liaison : `corpsDeFonction`
rend `null` sur des accolades non équilibrées, et le précédent de la 23-1a (le commentaire de
`i18n-literal-reader.js:413-418`) est exactement celui d'un `null` qui faisait jeter un site **en silence**. Une
liaison dont la portée n'a pas été inspectée n'est pas une liaison sans défaut. La règle vaut sur les **trois**
chemins qui calculent une portée (C-28-2-29) : le bloc d'un `catch` ou d'un rappel (accolades), le corps-expression
d'un `.catch(` (parenthèses), et la liste d'arguments d'un appel `console.*` de l'exemption ci-dessous
(parenthèses) — un `null` y est un **échec** nommé, jamais lu « dans l'appel » (ce qui exempterait en silence) ni
« hors de l'appel ».

G2 et G3 ne dépendent pas d'une liaison : elles sont cherchées dans **tout le fichier** (commentaires masqués,
exemption `console` appliquée).

| règle | forme interdite | précédent |
|---|---|---|
| G1 | `String(<liaison>)` | A1-A4, B2-B14 (B15 et B16 n'en relèvent pas : leur `err` est un **paramètre de fonction** — `formatError(err: unknown)`, `errorMessageOf(err: unknown)` —, non une liaison ; seule G3 les voit, cf. angle mort n° 4) |
| G2 | `String(<expr>.reason)` (résultat de `Promise.allSettled`) | B1 |
| G3 | motif `\binstanceof\s+Error\b` (bornes de mot : `instanceof ErrorEvent` n'en relève pas), **tout usage** — ternaire, `if (x instanceof Error)`, **classification comprise** (`err instanceof Error && err.name === 'AbortError'`, qui n'affiche rien, rougit aussi : C-28-2-27) —, dans tout le fichier (C-28-2-18), exemption `console` appliquée. Raison : afficher le `message` d'une `Error` est ce que C-28-2-3 écarte, et la forme peut vivre **hors** d'une portée de liaison — un `.reason` d'`allSettled` (la forme de B1, aujourd'hui écrite avec `String` seul) ou une fonction intermédiaire qui l'emploie (angle mort n° 4) —, ce que seule une recherche à l'échelle du fichier attrape ; une règle qui distinguerait l'affichage de la classification ne serait plus décidable par lecture. Le **message d'échec** de la garde oriente vers `err.name`, `instanceof DOMException` ou `instanceof TypeError` pour classer. À `6f971d76`, 11 lignes (`grep -rnE "\binstanceof\s+Error\b"`, hors tests), dont 2 commentaires : des 9 lignes de code, **6 disparaissent** avec A et B (A1, A2, B13-B16) et **3 restent, exemptées** (classe C, appels `console`) | A1, A2, B13-B16 |
| G4 | `${<liaison>}` ou `${String(<liaison>)}` dans un gabarit | A3, A4 |
| G5 | `<liaison> + ''`, `'' + <liaison>` (et guillemets doubles / gabarit vide) | — (forme connue, absente) |
| G6 | `<liaison>.toString()` | — |
| G7 | `JSON.stringify(<liaison>` | — |

**Exemption par critère, jamais par entrée** : une occurrence située **dans la liste d'arguments d'un appel
`console.(log|info|warn|error|debug)(`** (portée calculée par équilibrage des parenthèses — le même
`blocEquilibre` —, appel multi-ligne compris) est permise — journaliser n'est pas afficher. À `6f971d76`, trois sites en relèvent
(`hooks.client.ts:23-28`, `lib/app/stores/auth.svelte.ts:301`, `lib/shared/utils/api-client.ts:159-163`) et y
**restent** à la fin de la story (classe C, C-28-2-25) : l'exemption est exercée au parcours réel par ces trois sites
— dont deux multi-lignes —, et par ses auto-tests. Aucune allowlist de fichiers ; `error-message.ts` n'en a pas besoin (il ne convertit
rien en texte).

**Anti-muet** (le mode d'échec du test muet, CLAUDE.md § E2E) — des planchers **par forme** (C-28-2-17), parce
qu'un plancher global de 160 laissait passer un recueil qui perdrait **toutes** les liaisons d'une forme (les 5
`.catch` liés : 168 − 5 = 163 ; les 4 `catch (e: unknown)` : 164) :

| compte | mesuré à `6f971d76` (commentaires masqués) | attendu en fin de story | plancher |
|---|---|---|---|
| fichiers parcourus | 317 | 319 (+ `api-error.ts`, `error-message.ts`) | ≥ **310** |
| `catch (x)` lié, non typé | 159 | 160 (+ `handleDelete`, C-28-2-20) | ≥ **150** |
| `catch (x: <type>)` lié, typé | 4 (`RulesList.svelte:64`, `:87`, `reconciliation/rules/+page.svelte:32`, `RuleFormModal.svelte:112`) | 4 | ≥ **4** |
| rappel `.catch` lié | 5 (liste au § Inventaire) | 5 | ≥ **5** |
| total des liaisons | 168 | 169 | **aucun** — informatif, affiché par la garde (C-28-2-28) |

Les deux petites formes portent un plancher **égal à leur mesure** : sur 4 ou 5 occurrences, une marge vaut
absence de contrôle. Si la story ou une suivante retire légitimement une de ces liaisons, le plancher rougit et se
baisse **avec la raison écrite** au-dessus de lui — « une borne exacte qui rougit fait son travail ». Le total ne
porte **pas** de plancher : la somme des trois planchers de forme (150 + 4 + 5 = 159) le borne déjà, et un plancher
total à 160 n'ajoutait qu'une unité de contrôle (*révisé en validation P2*, C-28-2-28 ; la remédiation P1 l'avait
posé). La story **n'ôte aucune** liaison. Le Dev Agent Record recompte les cinq valeurs au dernier commit et les
confronte à la colonne « attendu en fin de story ».

**Auto-tests du détecteur** (sources fictives passées à la fonction de détection, non au parcours) :

- chaque règle G1-G7 a au moins une **fictive positive** (le détecteur la signale) ; G3 en forme ternaire **et** en
  forme `if (e instanceof Error) msg = e.message; else msg = repli;` ;
- l'exemption `console` a une fictive négative **mono-ligne et multi-ligne**, et une fictive positive : la même
  conversion **hors** de l'appel `console`, dans un fichier qui journalise par ailleurs (M16) ;
- **chaque forme de liaison** est recueillie et sa portée inspectée : `catch (caught)` (nom autre que `e`/`err`),
  `catch (e: unknown)`, `.catch((err) => …${String(err)}…)` (corps-expression), `.catch(err => String(err))`,
  `.catch(function (x) { … String(x) … })`, `.catch((err: unknown) => String(err))` (typé) et
  `.catch(async (err) => { … String(err) … })` (`async`, C-28-2-29) ;
- **G3, bornes de mot** : `instanceof ErrorEvent` n'est **pas** signalé ; `err instanceof Error && err.name ===
  'AbortError'` (classification) **l'est** (C-28-2-27) ;
- **portée et littéraux** : à l'intérieur d'un `catch`, une chaîne qui contient une accolade seule
  (`'{'`, `"}"`), un gabarit imbriqué (`` `a ${`b ${n} }`} {` ``) et une regex (`/[{]/`), suivis d'un `String(err)` —
  qui doit **encore** être signalé (la portée n'a pas été coupée trop tôt) ; et un `String(err)` placé **après**
  la fermeture du `catch`, sur une autre variable `err` hors portée, qui **ne l'est pas** ;
- **portée incalculable**, sur les trois chemins (C-28-2-29) : un `catch (err) {` dont l'accolade ne se ferme jamais
  (M24), un `.catch((err) => String(err)` dont la parenthèse ne se ferme jamais, et un `console.error(` non fermé
  suivi d'un `String(err)` font chacun échouer la détection en nommant la ligne (M27) ;
- un `String(x)` sur une variable qui **n'est pas** une liaison d'erreur n'est pas signalé ; un commentaire
  contenant `String(err)` n'est pas signalé.

### AC7 — Quatre locales

Trois clés neuves dans les quatre catalogues `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`,
à côté d'`error-unexpected` (fr-CH `:1733`, autres `:1625`), sous un commentaire `# Story 28-2 — erreurs fabriquées
par le client (#520)`. Textes au § *Textes proposés* (alignés sur `auth-recovery-error-network`, déjà traduite).
Préfixe `error-` : espace de noms global, permis partout par `scripts/lint-i18n-ownership.js:16`.
`loader.rs::parity_between_locales`, `i18n-keys.test.ts`, `i18n-un-repli-par-cle.test.ts`,
`i18n-entrees-a-variables.test.ts` verts. Et trois gardes que la première rédaction ne nommait pas :

- **`frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts`** — dès que les trois clés figurent au catalogue
  `fr-CH`, `i18nMsg` fait `_messages[key] || fallback` : le texte **fr-CH** et le **repli** du site doivent être
  identiques **au caractère près** (point final, espace de `({ $status })`, apostrophe). C'est elle qui tient
  l'exigence « aucun changement en français » de l'AC2 ; sa liste `TOLEREES` ne reçoit **aucune** entrée.
- **`frontend/src/lib/shared/i18n-harvest.test.ts`** — preuves du moissonneur (`moissonner`, périmètre et
  détection des conflits de repli, 23-1b) **sur fixtures** : il ne lit pas l'arbre (aucun `readFileSync` ni import
  de `node:`), et `moissonner` ne retient que les clés **absentes** du catalogue ; il ne voit donc **aucun** des
  quatre sites neufs et n'a pas à changer (*révisé en validation P2* : la remédiation P1 écrivait qu'ils « entrent
  dans son périmètre »). Le repli unique d'`error-unexpected` sur les vingt et un sites est tenu par
  `i18n-un-repli-par-cle.test.ts`, déjà nommé.
- **`frontend/src/lib/shared/i18n-dette-connue.ts`** — liste **vide** depuis la 23-6 ; elle le reste : une clé
  manquante se répare dans les quatre catalogues, jamais en l'y inscrivant.

### AC8 — Tests unitaires de l'utilitaire et du client

- `frontend/src/lib/shared/utils/error-message.test.ts` (neuf) : une ligne par cas du tableau de l'AC1 — `ApiError`
  → son message ; `ApiError` message `''` et `'  '` → générique ; `new Error('boom')`, `new TypeError('x')`,
  `'texte'`, `42`, `null`, `undefined`, `{}`, `{ message: 'm' }` (objet sans `code`/`status`) → générique ; pour
  chacun, `not.toBe('[object Object]')` et `not.toBe('')` ; `console.error` appelé avec la valeur d'origine pour les
  non-`ApiError`, **pas** appelé pour une `ApiError` ; **traduction** : catalogue chargé avec
  `error-unexpected = Unerwarteter Fehler.` → rendu allemand.
- `frontend/src/lib/shared/utils/api-client.test.ts` (complété) : `fetch` rejeté (`TypeError`) → `ApiError`
  `NETWORK_ERROR` dont le `message` est le texte du catalogue chargé (allemand) ; `AbortError` → `TIMEOUT` idem ;
  réponse 502 non JSON, JSON sans `error`, JSON `{ error: { code: 'X' } }` sans message → `error-server-status`
  interpolé (`502`) ; JSON `{ error: { message: 'm' } }` sans code → `message` `'m'` (repris, non traduit). Sans
  catalogue : les replis français **à l'identique** de l'avant-story (non-régression FR).
  ⚠️ Le chargement du catalogue passe par `loadI18nMessages()` avec le `fetch` simulé de
  `/api/v1/i18n/messages` (ou tout moyen qui n'ajoute pas d'export de production pour les tests).
  ⚠️ **`_messages` est un état de module** (`i18n.svelte.ts:11`) qui persiste d'un test à l'autre du même fichier :
  chaque cas « catalogue allemand » est suivi d'un rechargement d'un catalogue vide `{}` (`afterEach`), faute de quoi
  les cas « sans catalogue » dépendent de l'ordre d'exécution. Même règle dans `error-message.test.ts`.
  Deux cas de plus : **(a)** `loadI18nMessages()` lui-même en échec (`fetch` rejeté sur `/api/v1/i18n/messages`) →
  la promesse **se résout** (son `catch {}` silencieux, `i18n.svelte.ts:48-50`, avale l'`ApiError` que le client a
  fabriquée — en appelant `i18nMsg` sur un catalogue vide, sans récursion), et un `NETWORK_ERROR` provoqué ensuite
  porte le **repli français** (c'est le cas du démarrage) ; **(b)** dans `error-message.test.ts`, une `ApiError` dont `message`
  vaut `undefined` ou `42` → le générique (C-28-2-24).

### AC9 — Tests des écrans de la classe A, et un test par fichier de la classe B

- **A1/A2** : `routes/(app)/settings/settings-page.test.ts` (complété) — `lockBooks` puis `releaseBooksLock`
  rejetés par une `ApiError` `{ code: 'VALIDATION_ERROR', message: 'La date doit suivre le verrou actuel.', status: 400 }`
  → `[data-testid="books-lock-error"]` contient ce message et **pas** `[object Object]` ; rejet par
  `new Error('boom')` → le générique, pas `boom`. ⚠️ Pour A2, le bouton de déverrouillage
  (`books-lock-release-toggle`) n'apparaît que si la société simulée porte un verrou : la réponse simulée de la
  société doit être **enrichie** de `booksLockedThrough` (le test existant ne le simule pas), et le **motif** doit
  être saisi avant la soumission : `submitUnlock` sort avant tout appel si `unlockMotif.trim()` est vide
  (`settings/+page.svelte:248-251`), et le test passerait sans exercer A2.
- **A3** : `lib/features/bank-import/BankProfileList.test.ts` (neuf) — `listBankProfiles` rejeté par une
  `ApiError` → son message, sans préfixe, sans `[object Object]`. **Et `handleDelete`** (C-28-2-20) :
  `deleteBankProfile` rejeté par une `ApiError` → `notifyError` (simulé, `vi.mock('$lib/shared/utils/notify', …)`)
  appelé avec son message, la liste des profils **toujours affichée**. ⚠️ `handleDelete` commence par
  `if (!confirm(…)) return;` (`BankProfileList.svelte:26`) et jsdom n'implémente pas `window.confirm` : le test le
  simule, `vi.stubGlobal('confirm', () => true)` (patron `RulesList.test.ts:91`), faute de quoi `deleteBankProfile`
  n'est jamais appelé. C'est l'assertion « `notifyError` appelé » qui fait rougir **M23** de façon nominative ; le
  rejet non géré qu'elle laisse fait en outre sortir Vitest en échec (erreur hors test attribuée au fichier, sans
  que le test lui-même soit rouge — *révisé en validation P2* : la remédiation P1 écrivait « Vitest fait échouer le
  fichier »).
- **A4** : `routes/(app)/bank-import/profiles/[id]/profile-page.test.ts` (neuf ; le préfixe `+` est réservé par
  SvelteKit) — `getBankProfile` rejeté par une `ApiError` 404 → son message.
- **B** : un test au moins **par fichier** touché de la classe B, sur un rejet **non-`ApiError`**
  (`new Error('boom')`) → le générique traduit, ni `boom`, ni `Error: boom`, ni `[object Object]` ; dans les
  fichiers de test existants quand il y en a (`bank-accounts-page.test.ts`, `DeleteBankImportAction.test.ts`,
  `CancelReconciliationDialog.test.ts`, `BankAccountJournalLinkForm.test.ts`, `ManualMatchModal.test.ts`,
  `TransactionSplitModal.test.ts`) ; `BankAccountJournalLinkForm.test.ts` ne contient aujourd'hui **aucun** cas
  de rejet (`grep -nE "Rejected|new Error"` vide) : le cas s'écrit de zéro, montage compris. **Le test vise le site
  de la mutation** quand le fichier en porte plusieurs : dans `bank-accounts-page.test.ts`, le rejet de
  `listBankAccounts` au chargement (`Promise.allSettled`, B1 — M13) ; dans `CancelReconciliationDialog.test.ts`, le
  rejet de `cancelReconciliation` à la confirmation (B10, `:97` — M14), et non celui du chargement (B9, `:60`).
- **`ManualMatchModal` et `TransactionSplitModal` — tests de SOUMISSION, à écrire de zéro** (C-28-2-26). Aucun test
  existant de ces deux fichiers ne soumet : `ManualMatchModal.test.ts:196-227` simule un rejet 412 mais ne choisit
  pas de contrepartie (son commentaire `:213-221` le dit, « partiellement smoke »), et `handleConfirm` sort d'emblée
  sans contrepartie (`ManualMatchModal.svelte:126`) ; `TransactionSplitModal.test.ts` n'attend que le bouton et son
  état `disabled` (`:103`, `:150`). Les cas neufs **pilotent le vrai `AccountAutocomplete`** (pas de simulation du
  composant) : `fireEvent.focus` sur son champ (`input[aria-autocomplete="list"]`, à distinguer du champ de
  description), puis `fireEvent.mouseDown` sur l'`option` voulue (`AccountAutocomplete.svelte:384-388` sélectionne
  au `mousedown`), la liste `accounts` passée en prop portant au moins un compte de classe 5, 6 ou 7, actif et
  postable (seuls proposés par `AccountAutocomplete`, `AccountAutocomplete.test.ts:57-79` ; filtre de classe
  `ManualMatchModal.svelte:84-89`, `TransactionSplitModal.svelte:71-73`) ; pour la ventilation, **deux lignes**,
  chacune avec son compte et un montant, dont la somme égale la valeur absolue du montant de la transaction
  (`balanceExact`, `:108-113`), puis clic sur `split-submit` / `manual-match-submit`. Deux cas par modale :
  **(i)** rejet `new Error('boom')` → `manual-match-error` / `split-error` affiche le générique ; **(ii)** rejet
  `{ code: 'BANK_ACCOUNT_NOT_CONFIGURED', message: 'm', status: 412 }` → `manual-match-bank-not-configured` /
  `split-bank-not-configured` présent — c'est ce cas qui fait rougir **M15** (B13) et **M26** (B14), la branche
  du drapeau étant préservée par l'AC4.
  **Sans exception** (C-28-2-21, qui révise C-28-2-11) : `settings/api-keys` et `reports` n'ont pas de test de
  page (`ls` : `+page.svelte` seul, et `+page.ts` pour `reports` ; `lib/features/reports/reports.api.test.ts` teste
  l'API, pas l'écran) — deux tests **neufs**, `routes/(app)/settings/api-keys/api-keys-page.test.ts` (un rejet
  non-`ApiError` du chargement de la liste, B6) et `routes/(app)/reports/reports-page.test.ts` (un rejet
  non-`ApiError` d'un rapport, rendu par `formatError`, B15 ; la branche `REPORT_PERIOD_OUT_OF_FISCAL_YEAR` vérifiée
  préservée). ⚠️ Pour `reports-page.test.ts`, le rejet passe par **`generate()`** — un rapport, p. ex.
  `getBalanceSheet` (`:309`) ou `getGeneralLedger` (`:251`), dont le `catch` appelle `formatError` sans condition
  (`:254`, `:269`, `:291`, `:335`) — et **non** par un export (`:403`, `:437`, `:473`, `:505`), qui n'appelle
  `formatError` que pour une `ApiError` porteuse d'un `code` et afficherait sinon un message générique d'export :
  M21 ne rougirait pas. `+page.svelte` reçoit `data` (`fiscalYears`, `companyName`) en prop
  (`reports/+page.svelte:78`) ; `+page.ts` n'a pas à être monté.
- Les tests existants « pas « [object Object] » » du rapprochement (`rules-page.test.ts:35`,
  `RuleFormModal.test.ts:106`, `RulesList.test.ts:76`, `ReconciliationProposals.test.ts:295-340`) restent verts
  **sans modification** de leurs assertions.

### AC10 — E2E : l'erreur traverse réellement la frontière HTTP

`frontend/tests/e2e/error-messages.spec.ts` (neuf), par **interception** `page.route` — aucune écriture dans la base
partagée (C-28-2-8) :

1. *Paramètres* → `POST **/api/v1/companies/current/books-lock` intercepté, réponse `400`
   `{"error":{"code":"VALIDATION_ERROR","message":"Message de refus de test (28-2)."}}` → saisir une date, cliquer
   `books-lock-submit` → `books-lock-error` a **exactement** ce texte ; la page ne contient nulle part
   `[object Object]`.
2. *Profils bancaires* → `GET **/api/v1/bank-profiles*` interrompu (`route.abort('failed')`), délais de rejeu
   raccourcis par `window.__KESH_RETRY_DELAYS = [10, 10, 10, 10]` (patron `db-resilience.spec.ts:73`) → la liste
   affiche le texte de `error-network` **du catalogue servi** (lu par l'API `/api/v1/i18n/messages` dans le test,
   pas recopié en dur), sans `[object Object]`.

⚠️ **Ce que le scénario 2 n'établit pas** (C-28-2-23) : l'instance E2E tourne en `fr-CH`, où le texte servi égale
le repli — l'assertion ne distingue pas un message traduit d'un littéral français conservé (M5 ne la fait pas
rougir). Il prouve la traversée réelle (absence de « [object Object] », M11) ; **la traduction n'est prouvée que par
les tests unitaires** au catalogue allemand (AC8, M5-M7). Le Dev Agent Record ne l'écrit pas autrement.

Recette visuelle (action B2 de la rétrospective de l'Epic 27) : le scénario 1 prend une capture de la section
`settings-books-lock` en thème **clair** et **sombre** (`page.emulateMedia({ colorScheme })`), **avant** le
correctif (T1, spec rouge) et **après** ; les quatre fichiers vont dans
`/home/gcorbaz/devel/kesh-gate-logs/28-2-captures/`, **hors dépôt**, et leurs chemins sont cités au Dev Agent
Record et au prompt de revue P1 (C-28-2-14).

### AC11 — CHANGELOG, manuel, README

- `CHANGELOG.md` : section **`## [0.13.3] — Non publié`** créée au-dessus de `## [0.13.2]` si elle n'existe pas
  (union avec les branches 28-1 et 28-3 à la fusion, C-28-2-12), rubrique `### Corrigé` : *« Une erreur renvoyée par
  le serveur ne s'affiche plus « [object Object] » — verrouillage des livres, profils bancaires — : le message du
  refus est affiché. Une panne imprévue affiche « Erreur inattendue. », et les messages de perte de connexion
  suivent la langue de l'installation une fois la page chargée — ou rechargée — avec une session ouverte ; la page
  de connexion, et l'interface jusqu'au premier rechargement qui suit une connexion, restent en français
  ([#520](https://github.com/guycorbaz/kesh/issues/520)). »* (C-28-2-23 : la phrase de la première rédaction,
  « s'affichent dans la langue de l'installation », promettait plus que le code ne tient — § Angles morts, n° 5 ;
  *révisé en validation P2* : « une fois l'interface chargée avec une session ouverte » se lisait, pour qui vient
  de se connecter, comme son cas, alors que `goto('/')` ne recharge pas le catalogue.)
- **Manuel** (C-28-2-15) : **un** passage cite un de ces messages — `docs/manual/fr/user-manual.tex:1871-1873`
  (encadré de l'import CSV : fichier modifié sur le disque après l'aperçu, « l'envoi échoue avec un message d'erreur
  de connexion (« Impossible de contacter le serveur… ») qui se répète à chaque clic »). La citation est coupée sur
  deux lignes du `.tex` : c'est pourquoi le `grep` mono-ligne de la première rédaction rendait zéro, et le recompte
  se fait **sur texte aplati**. Ce message est le `NETWORK_ERROR` du client (`api-client.ts:294`), que l'AC2 rend
  traduisible.
  - **En français**, le passage **reste exact** : le repli et le texte `fr-CH` d'`error-network` sont le texte
    actuel au caractère près (AC2, garde `i18n-repli-divergent-actif.test.ts`), et la citation tronquée
    (« Impossible de contacter le serveur… ») en est le début.
  - **Dans les autres langues**, une installation `de-CH`, `it-CH` ou `en-CH` dont le catalogue est chargé affichera
    le texte traduit (« Server nicht erreichbar. … », etc.) ; or les manuels `de/`, `en/`, `it/` ne contiennent
    qu'un `README.md`, sans `.tex` : aucun manuel ne cite le message dans ces langues, et leur traduction future
    citera le texte de la clé `error-network`.
  - **Le manuel ne change donc pas**, ni son PDF ; aucune mention « dans la langue de l'installation » n'est
    ajoutée (le manuel français s'adresse à une installation française, où rien ne change).
  - Contrôle à refaire au Dev Agent Record, sur `.tex` **et** PDF :
    ```sh
    for f in docs/manual/fr/*.tex; do echo "== $f"; tr '\n' ' ' < "$f" | tr -s ' ' \
      | grep -oiE "object Object|Impossible de contacter|Erreur serveur|ne répond pas|Erreur inattendue" | sort | uniq -c; done
    for f in docs/manual/fr/*.pdf; do echo "== $f"; pdftotext "$f" - | tr '\n' ' ' | tr -s ' ' \
      | grep -oiE "object Object|Impossible de contacter|Erreur serveur|ne répond pas|Erreur inattendue" | sort | uniq -c; done
    ```
    Sortie à `6f971d76` : `user-manual` (.tex et .pdf) **1** « Impossible de contacter » ; `admin-manual` (.tex et
    .pdf) **1** « erreur serveur » — `admin-manual.tex:1472`, « le téléchargement répond une erreur serveur » (PDF
    figé altéré) : une description de la réponse HTTP, non une citation du texte `Erreur serveur ({ $status })`,
    **sans rapport** et inchangée ; `marketing-brochure` 0. Tout autre résultat est à confronter au code.
- **README / site** : rien (correctif sans fonctionnalité nouvelle).

## Tasks / Subtasks

- [x] **T0 — Montage** (consignes 6, 8, 11, 13) : bases `kesh_282` / `kesh_e2e_282` (migrations du worktree, seed),
  `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-28-2/target`, `node_modules` par `cp -al` si absent, port 3035.
- [x] **T1 — Tests d'abord, rouges** (AC8, AC9, AC10 scénario 1 + captures « avant ») : écrire
  `error-message.test.ts`, les cas de `api-client.test.ts`, les tests A1-A4 ; constater le rouge et le noter.
- [x] **T2 — `api-error.ts` et `error-message.ts`** (AC1) ; retrait d'`errorMessageOf` d'`api-client.ts`, réexport
  d'`isApiError` ; quatre importeurs (AC4).
- [x] **T3 — Traduction à la source** (AC2) : `serverStatusMessage`, `toFetchApiError` ; trois clés × 4 locales (AC7).
- [x] **T4 — Classe A** (AC3) et `handleDelete` de `BankProfileList.svelte` (C-28-2-20), puis **classe B** (AC4),
  fichier par fichier, un commit par lot cohérent. La classe C ne change pas (AC5, C-28-2-25).
- [x] **T5 — Garde** `erreurs-en-texte.test.ts` (AC6) avec ses auto-tests, et `blocEquilibre` dans
  `i18n-literal-reader.js` avec ses tests (C-28-2-16) ; elle doit **rougir** sur la base de
  `6f971d76` (le vérifier avant T4 en la posant sur une copie, ou en rejouant M7) puis verdir après T4.
- [x] **T6 — Tests de la classe B** (AC9, dont les deux tests de page neufs `api-keys-page.test.ts` et
  `reports-page.test.ts`) et E2E scénario 2 (AC10) ; captures « après ».
- [x] **T7 — Recomptes** : inventaire (AC5) par les commandes du § Inventaire ; `sitesTotal` et compagnie
  d'`i18n-keys.test.ts` (§ Pièges) ; les cinq planchers de l'AC6.
- [x] **T8 — CHANGELOG, manuel** (AC11 : contrôle sur texte aplati, `.tex` et PDF).
- [x] **T9 — Mutations** M1-M27 (§ Mutations), restauration `git checkout -- <f>` + `touch` (mémoire *Mutation
  restaurée, binaire périmé*).
- [x] **T10 — Gates complets au dernier commit de code** (consigne 3, D7) : `npm run check`,
  `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build` ; `scripts/test-fast.sh` (le catalogue FTL est
  lu par `kesh-i18n`, `loader.rs::parity_between_locales`) ; E2E complet (8 échecs attendus à juger fichier par
  fichier, `docs/testing.md` § *Les échecs attendus*). Tous sous `flock …/.gate.lock` (consigne 13).

## Dev Notes

### Inventaire des sites NON RÉSOLUS (ensemble clos), au sol à `6f971d76`

**Définition** : un site *non résolu* est un endroit où une **valeur attrapée** — liaison de `catch`, paramètre de
rappel `.catch`, `reason` d'un `Promise.allSettled` — dont le type n'est pas prouvé `ApiError` est **convertie en
texte affiché** : `String(x)`, `` `${x}` ``, `x + ''`, `x.toString()`, `JSON.stringify(x)`, `x instanceof Error ?
x.message : …`. (`x.message` sur une liaison `unknown` ne compile pas : `frontend/tsconfig.json:12` `"strict": true`
active `useUnknownInCatchVariables` ; `npm run check` est donc la garde de cette forme-là.)

**Univers** (mesuré par un script d'inventaire de l'auteur, non versionné, qui équilibre les accolades ; ses
comptes sont reproductibles à la main par les `grep` ci-dessous) : 317 fichiers `.svelte`/`.ts` hors tests sous
`frontend/src`, **256** instructions `catch` hors commentaires (**163** liées — dont 4 typées `catch (e: unknown)`,
159 non typées —, 93 sans liaison — `catch {` ne peut rien convertir), **20** rappels `.catch(` dont 5 liés
(`mode.svelte.ts:57`, `audit-log/+page.svelte:133`, `bank-import/profiles/[id]/+page.svelte:24`,
`vat-rates.store.svelte.ts:48`, `InvoiceForm.svelte:351`), soit **168** liaisons, et deux lectures de `.reason`
d'`allSettled` (`journal-entries/[id]/+page.svelte:88`, `bank-accounts/+page.svelte:76-78`). *(Recompté en
validation P1 : la première rédaction écrivait 257 / 164 / « 169 mesurés ». Le `grep`
`(^|[^.A-Za-z_])catch\s*(\(|\{)` hors tests rend 258 lignes et 165 liées, dont **deux commentaires** —
`settings/fiscal-years/+page.svelte:320` « try/catch (miroir… » et `lib/shared/utils/notify.ts:88`, exemple de
doc-comment —, que la garde masque. Liste des cinq `.catch` liés remise derrière « dont 5 liés » en validation P2.)*

**Commandes** (depuis `frontend/src`, `F="--include=*.svelte --include=*.ts"`) et sorties :

```sh
grep -rnE "String\([^)]*(\b(e|err|error|ex|reason|cause)\b|Result\.reason)[^)]*\)" . $F | grep -v "\.test\."   # 26 lignes
grep -rnE "instanceof Error" . $F | grep -v "\.test\."                                                          # 11 lignes, toutes dans la précédente
grep -rnE '\$\{(e|err|error|ex|reason)\}' . $F | grep -v "\.test\."                                             # 0
grep -rnE "\b(e|err|error)\s*\+\s*(''|\"\")" . $F | grep -v "\.test\."                                          # 0
grep -rnE "\b(e|err|error)\.toString\(\)" . $F | grep -v "\.test\."                                             # 0
grep -rnE "JSON\.stringify\((e|err|error)\b" . $F | grep -v "\.test\."                                          # 1 : audit-log/+page.svelte:390, `e` = entrée du journal ({#each}), pas une erreur
grep -rnE "\.catch\(" . $F | grep -v "\.test\."                                                                  # 20
grep -rnE "\{:catch" . --include=*.svelte                                                                        # 0
```

Les 26 lignes de la première commande se ventilent en : **3 commentaires** (`api-client.ts:34`, `:36`,
`ManualMatchModal.svelte:144`), **3 journalisations** (classe C), **20 sites** — 4 de classe A et 16 de classe B :

| classe | définition | sites | sort |
|---|---|---|---|
| **A** | `ApiError` rendue « [object Object] » (aucune garde `isApiError` avant la conversion) | A1 `settings/+page.svelte:237`, A2 `:261`, A3 `BankProfileList.svelte:19`, A4 `bank-import/profiles/[id]/+page.svelte:25` | **résolus** (AC3) |
| **B** | `ApiError` traitée, repli `String`/`instanceof Error` pour le reste (« TypeError: … », « [object Object] » pour un objet) | B1-B16 (AC4) : `bank-accounts/+page.svelte:78`, `:239`, `:265`, `:281` ; `DeleteBankImportAction.svelte:62` ; `settings/api-keys/+page.svelte:63`, `:159`, `:198` ; `CancelReconciliationDialog.svelte:60`, `:97` ; `BankAccountJournalLinkForm.svelte:98`, `:127` ; `ManualMatchModal.svelte:153` ; `TransactionSplitModal.svelte:216` ; `reports/+page.svelte:215` ; `api-client.ts:40` | **résolus** (AC4) |
| **C** | conversion dans un appel `console.*` — journal, non affiché | `hooks.client.ts:27`, `lib/app/stores/auth.svelte.ts:301`, `lib/shared/utils/api-client.ts:162` | **légitimes par critère** (exemption de l'AC6), inchangées : 3 à la fin. Aucune `ApiError` ne les atteint — `fetch` direct (AC5, C-28-2-25) |
| **D** | affichage réservé aux `ApiError` (`if (isApiError(err)) x = err.message;` sans autre branche) : une valeur d'une autre nature n'affiche **rien** — le « message vide » | ≈ **23** sites, détecteur ci-dessous | **angle mort assumé**, suivi par **#644** (C-28-2-10, C-28-2-22) |
| **E** | `isApiError(err) ? err.message : <repli fixe>` — le repli étant soit un `i18nMsg` générique, soit un **littéral français** (`journal-entries/[id]/+page.svelte:89`, « Erreur de chargement de l'écriture » ; `ProductPicker.svelte:55`, « Erreur de chargement des produits » ; les préfixes d'`InvoiceForm`) | nombreux (`invoices/[id]`, `dunning`, `projects`, `audit-log`, …) | **conformes au sujet** : jamais « [object Object] », jamais un texte technique ; non refactorés (registre de la demande) ; les replis en littéral français sont une dette i18n, § Angles morts n° 3 |

**Détecteur de la classe D** (heuristique, à recompter, pas une garde) : `catch` lié dont le corps (commentaires
retirés) contient `isApiError(` et ne contient **aucun** de `else`, `?`, `errorMessageOf`, `String(`, `i18nMsg`,
`msg(`, `||`, `throw`. À `6f971d76` il rend **25** lignes, dont **2 faux positifs** vérifiés à la main
(`lib/shared/utils/notify.ts:88`, exemple de doc-comment ; `invoices/[id]/+page.svelte:884`, qui affiche toujours
`pdfErrorMessage(err)`) : **23** sites réels, dans `credit-notes` (2), `invoices` (liste 1, fiche 3, édition 1,
échéancier 1, relances 5), `payment-batches` (5), `supplier-invoices` (liste 2, fiche 2, import 1). ⚠️ Une forme qui
contient un `?` pour une autre raison lui échappe : le nombre est un **plancher**, pas l'ensemble.
Pourquoi la classe D n'est pas traitée ici : la cause n'est pas #520 (aucun « [object Object] », aucun texte faux :
un silence), le cas ne survient que sur une valeur qui n'est pas une `ApiError` (le client convertit toute panne
réseau et tout refus HTTP en `ApiError` ; restent un corps de **succès** non JSON — `res.json()` qui lève — et une
erreur de programmation), et la traiter ajouterait **quatre** modules métier — `credit-notes`, `invoices`,
`payment-batches`, `supplier-invoices` — (règle de découpage préventif). *(Corrigé en
validation P2 : la rédaction antérieure écrivait « sept » modules, pour les quatre qu'elle nommait.)*
La validation P1 (F4, R2, R3) a confirmé que la correction **n'est pas mécanique** : chaque site demande un choix
(notification, message en ligne, ou silence), et certains silences sont **voulus** — `invoices/[id]/+page.svelte:186`
(`afterReconciliationCancelled`) et `:483` (`reloadInvoiceAndSettlements`, « échecs tolérés ») suivent une action
**réussie** et relisent la facture ; `errorMsg` y est un état de page (`{:else if errorMsg}`, `:1111`, avant
`{:else if invoice}`, `:1113`), si bien qu'un `errorMsg = errorMessageOf(err)` y masquerait la fiche facture
derrière « Erreur inattendue. » pour une opération qui a abouti.
→ Suivi : issue **#644** (bug, **P3**), ouverte par l'orchestrateur le 2026-10-13 (C-28-2-22). La PR la cite en
**`refs #644`** ; aucune autre issue de suivi n'est à demander.

### Angles morts assumés (écrits pour qu'on ne les croie pas couverts)

1. **Classe D** (ci-dessus) — suivie par **#644** (`refs #644`).
2. **`SETUP_REQUIRED` / `UNAUTHENTICATED`** (`api-client.ts:404-430`, `:501-524`) : messages français en dur, non
   traduits par l'AC2 ; ils précèdent une redirection (`/onboarding`, `/login`) et ne sont lus par aucun écran
   connu. Un écran qui afficherait `err.message` à cet instant montrerait « Session expirée » en français (C-28-2-5).
3. **Littéraux français en dur voisins, hors du sujet** : `'ID invalide'` (`bank-import/profiles/[id]/+page.svelte:15`),
   le message réseau recopié en dur de `routes/login/+page.svelte:54`, `ResetPasswordForm.svelte:77`,
   `ForgotPasswordForm.svelte:54` (ces deux-là en repli d'`i18nMsg`, conformes), et les préfixes français de
   `InvoiceForm.svelte:354-355`, `:379-380`, `:542-543` (`` `… (${err.message})` ``), et les replis de classe E en
   littéral (`journal-entries/[id]/+page.svelte:89`, `ProductPicker.svelte:55`, le suffixe « — Configurez les
   comptes par défaut… » d'`invoices/[id]/+page.svelte:411`). Dette i18n préexistante,
   invisible des gardes i18n (le préambule d'`i18n-libelle-en-dur.test.ts` le dit) ; **signalée**, non traitée
   (C-28-2-13). L'orchestrateur vérifie si une issue la couvre.
4. **La garde AC6 ne voit pas** : une conversion par une **fonction intermédiaire** (`const f = (x: unknown) =>
   String(x)` appelée avec une erreur) — elle ne connaît que les liaisons d'erreur, pas le flot. Deux sites que la
   story traite sont de cette forme : `formatError(err: unknown)` (`reports/+page.svelte:196`, B15) et
   `errorMessageOf(err: unknown)` lui-même (B16, puis `error-message.ts`) ; à la base, seule G3 les voit (par leur
   `instanceof Error`), et à la fin un `return String(err)` réintroduit seul dans `formatError` ne serait vu
   d'**aucune** règle — seuls `reports-page.test.ts` et `error-message.test.ts` le tiennent. De même, une erreur
   **copiée dans une variable** (`const err = entryResult.reason;`, `journal-entries/[id]/+page.svelte:88` ; `const
   x = err`) puis convertie n'est vue ni par G1 (pas une liaison) ni par G2 (pas `String(<expr>.reason)`) ; une conversion
   **dans le balisage** Svelte (`{String(err)}`) d'une variable d'état qui contiendrait une erreur brute ; un
   `catch` dont la liaison est **déstructurée** (`catch ({ message })`) — 0 à `6f971d76`
   (`grep -rnE "catch \(\{" frontend/src` ne rend rien) ; un rejet **sans** `catch` du tout (le cas de
   `handleDelete`, corrigé ici, C-28-2-20 ; un autre ailleurs resterait invisible). Ces formes sont écrites dans son
   préambule.
5. **La traduction des messages du client ne vaut qu'une fois le catalogue chargé** (C-28-2-23). Le catalogue est
   servi par `/api/v1/i18n/messages`, **derrière l'authentification**, et `routes/+layout.svelte:51-53` ne le charge
   qu'à l'`onMount` racine, **si** une session est ouverte (le commentaire `:36-50` l'écrit : « fallback FR figé reste
   affiché sur `/login` … Limitation à corriger v0.2 via endpoint i18n public »). Hors `onboarding` (`:18`, `:27`),
   aucun autre site ne l'appelle. D'où trois cas où un `NETWORK_ERROR` / `TIMEOUT` / `Erreur serveur` reste **en
   français** sur une installation `de-CH`, `it-CH` ou `en-CH` : la **page de connexion** (qui recopie d'ailleurs le
   message réseau en dur, n° 3) ; un **rechargement complet** alors que le serveur est injoignable (le chargement du
   catalogue échoue lui-même) ; et la navigation qui suit une **connexion** par `goto('/')`
   (`routes/login/+page.svelte:50`), l'`onMount` racine ne se rejouant pas — jusqu'au prochain rechargement complet.
   Ce dernier cas n'est pas propre à cette story : **tout** `i18nMsg` de l'interface est alors en français.
   Limitation préexistante, hors périmètre ; l'orchestrateur vérifie si une issue la couvre.

### État actuel des fichiers modifiés — ce qui doit être préservé

- `api-client.ts` (584 lignes) : le mutex de refresh, le rejeu exponentiel (`DEGRADED_RETRY_DELAYS_MS`,
  `window.__KESH_RETRY_DELAYS`), `apiHealth.setDegraded/clearDegraded`, la forme `ApiError` (`code`, `status` —
  `0` pour réseau/délai) sont **inchangés** ; seuls changent les trois textes (AC2), l'extraction d'`isApiError`
  (réexport) et le retrait d'`errorMessageOf`. Les `code` ne changent pas : `ForgotPasswordForm.svelte:43-56` et
  `routes/login/+page.svelte:51-54` aiguillent sur `err.code === 'NETWORK_ERROR'`.
- `i18n.svelte.ts` : inchangé. `i18nMsg` interpole `{ $x }` côté client (`:19-27`) ; avant chargement du catalogue,
  le repli s'affiche — d'où l'exigence « repli = texte actuel ».
- `settings/+page.svelte` : `lockError` sert aussi au contrôle « motif obligatoire » (`:249`, `msg(…)`) ; ne pas y
  toucher. Le test existant `settings-page.test.ts` simule `$lib/features/onboarding/onboarding.svelte` (`i18nMsg`
  → repli) et `settings.api` (`lockBooks`, `releaseBooksLock` déjà des `vi.fn()`).
- `ManualMatchModal.svelte` / `TransactionSplitModal.svelte` : le drapeau `bankNotConfigured` (branche
  `BANK_ACCOUNT_NOT_CONFIGURED`) pilote un lien vers la liaison au plan comptable — à préserver ; le commentaire
  P-H1 de `ManualMatchModal.svelte:142-146` peut renvoyer à `error-message.ts`.
- `reports/+page.svelte` `formatError` : la branche `REPORT_PERIOD_OUT_OF_FISCAL_YEAR` (interpolation des bornes)
  reste ; seule la dernière ligne change. Ses huit appelants (`:253` à `:504`) ne changent pas.
- `BankProfileList.svelte` : outre A3, `handleDelete` (`:25-31`) reçoit son `try/catch` (C-28-2-20) ; le
  `confirm(...)` d'entrée et le rechargement après succès sont inchangés ; `notifyError` s'importe de
  `$lib/shared/utils/notify`.
- `hooks.client.ts`, `lib/app/stores/auth.svelte.ts` : **non touchés** (C-28-2-25, qui retire C-28-2-19 — la
  remédiation P1 y changeait l'argument de journalisation) ; leurs trois journalisations de classe C restent telles
  quelles (AC5), de même que la ligne `api-client.ts:159-163`.
- `ReconciliationProposals.svelte:206`/`:224` lèvent des `Error` techniques (« No candidate for txId=… ») attrapées
  à `:230` : elles s'afficheront désormais « Erreur inattendue. » (journalisées) — voulu (C-28-2-3).

### Pièges connus du dépôt, applicables ici

- **`sitesTotal` d'`i18n-keys.test.ts:601`** (2075 à `6f971d76`) compte les sites `i18nMsg` de tout `src/` : l'AC1
  en ajoute **1** (`error-message.ts`, `error-unexpected`), l'AC2 **3** (`error-network`, `error-timeout`,
  `error-server-status`) → **2079 attendu**, à **recompter** par la collecte du test (une borne exacte qui rougit
  fait son travail) ; écrire le delta ventilé dans le commentaire au-dessus de la borne, **sans réécrire les
  entrées passées** (consigne 15). `sitesNonResolus`, `relais`, `sitesGabarit` : à recompter, a priori inchangés.
- **`vi.mock('$lib/shared/utils/api-client', …)`** : **18** fichiers de test simulent le client (`grep -rlF
  "vi.mock('\$lib/shared/utils/api-client'" frontend/src` → 17, plus 1 en guillemets doubles), presque tous **sans**
  `isApiError` — raison de
  l'extraction en `api-error.ts` (C-28-2-2). Un composant testé sous un tel mock qui importe `errorMessageOf`
  obtient le vrai `isApiError`.
- **Repli unique par clé** (`i18n-un-repli-par-cle.test.ts`) : `error-unexpected` → `'Erreur inattendue.'` (avec
  point) ; ne pas confondre avec `common-error` → `'Erreur inattendue'` (sans point).
- **Mutation restaurée, binaire périmé** : après `git checkout -- <f>`, `touch <f>`.
- **Édition par l'outil Edit uniquement** (consigne 14) : aucun `sed -i`/`perl -pi` sur les seize sites, même
  « mécaniques ».
- **Base de gate remise à zéro avant le gate complet** — sur `kesh_282`, jamais par `docker restart` (consigne 2).

### Textes proposés (DE / IT / EN), alignés sur `auth-recovery-error-network`

| clé | fr-CH | de-CH | it-CH | en-CH |
|---|---|---|---|---|
| `error-network` | Impossible de contacter le serveur. Vérifiez votre connexion. | Server nicht erreichbar. Überprüfen Sie Ihre Verbindung. | Impossibile contattare il server. Verifica la connessione. | Unable to reach the server. Check your connection. |
| `error-timeout` | Le serveur ne répond pas. Réessayez ultérieurement. | Der Server antwortet nicht. Versuchen Sie es später erneut. | Il server non risponde. Riprova più tardi. | The server is not responding. Try again later. |
| `error-server-status` | Erreur serveur ({ $status }) | Serverfehler ({ $status }) | Errore del server ({ $status }) | Server error ({ $status }) |

### Découpage préventif — dérogation

La story touche **six** modules de premier niveau (`shared/utils`, `settings`, `bank-import`, `bank-accounts`,
`reconciliation`, `reports`) : le critère « plus de cinq modules » est franchi d'un. **Dérogation** (C-28-2-9) : la
seule logique neuve vit dans `shared/utils` (AC1, AC2, testés seuls) ; les vingt sites sont un remplacement
**mécanique** d'une expression par un appel, relu fichier par fichier — c'est la forme « story-zéro + rollout » que
la règle recommande, tenue dans une seule fiche parce que le rollout est d'une ligne par site et que le scinder
laisserait une garde AC6 rouge sur la sous-story pilote. Risque accepté : une revue qui survolerait les seize
sites B. Parade : le prompt de revue P1 exige la revue site par site contre le tableau de l'AC4. **Signal déclaré au
Project Lead** par le rapport de l'agent.
*(Validation P1.)* La remédiation ajoutait deux fichiers d'**infrastructure** de démarrage, une ligne de journalisation
chacun (`hooks.client.ts`, `lib/app/stores/auth.svelte.ts` — C-28-2-19) : en les comptant comme un septième module
(`app`), le critère était franchi de deux ; aucun module **métier** ne s'ajoute (`handleDelete` est dans
`bank-import`, déjà compté), et la classe D, qui en aurait ajouté au moins quatre, reste hors périmètre (#644). La
dérogation est maintenue, et le signal redéclaré. Les réécritures d'`if/else` de B11-B14 (AC4) ne sont pas une
substitution d'une ligne : la parade « revue site par site » les vise aussi.
*(Validation P2.)* C-28-2-19 est **retiré** (C-28-2-25 : sa prémisse était fausse, aucune `ApiError` n'atteint la
classe C) : `hooks.client.ts` et `auth.svelte.ts` sortent du périmètre, et la story revient à **six** modules du
frontend. S'y ajoute le crate **`kesh-i18n`**, que les décomptes antérieurs omettaient : les trois clés × 4 locales
(AC7) sont des **données** (catalogues FTL), sans une ligne de Rust, mais le crate est un module au sens de la
règle (C-28-2-9, révisé). Six modules et un crate de données : le critère est franchi d'un (ou de deux en comptant le
crate) ; la dérogation est maintenue sur le même motif — logique neuve concentrée dans `shared/utils`, sites
relus un par un —, et le signal **redéclaré au Project Lead**.

### Mutations (à jouer en T9, chacune doit faire rougir au moins un test nommé)

| # | mutation | doit rougir |
|---|---|---|
| M1 | `errorMessageOf` : branche `ApiError` → `String(err)` | `error-message.test.ts` |
| M2 | `errorMessageOf` : branche « autre » → `err instanceof Error ? err.message : String(err)` | `error-message.test.ts` (cas `Error`, `{}`) + garde G3 (`err` est un paramètre de fonction, non une liaison : G1 ne le voit pas) |
| M3 | `errorMessageOf` : message vide d'`ApiError` rendu tel quel | `error-message.test.ts` |
| M4 | `errorMessageOf` : `console.error` retiré | `error-message.test.ts` |
| M5 | `toFetchApiError` : `NETWORK_ERROR` → littéral français | `api-client.test.ts` (catalogue allemand) |
| M6 | `toFetchApiError` : `TIMEOUT` → littéral | `api-client.test.ts` |
| M7 | `serverStatusMessage` → littéral `` `Erreur serveur (${status})` `` | `api-client.test.ts` |
| M8 | site `:242` : le `message` du corps ignoré au profit du repli | `api-client.test.ts` (corps sans code) |
| M9 | A1 rétabli (`e instanceof Error ? e.message : String(e)`) | `settings-page.test.ts`, garde G1/G3, E2E scénario 1 |
| M10 | A2 rétabli | `settings-page.test.ts`, garde |
| M11 | A3 rétabli | `BankProfileList.test.ts`, garde G4, E2E scénario 2 |
| M12 | A4 rétabli | `profile-page.test.ts`, garde G4 |
| M13 | B1 rétabli (`String(baResult.reason)`) | garde G2, `bank-accounts-page.test.ts` (cas du rejet de `listBankAccounts` au chargement, AC9) |
| M14 | B10 rétabli (`… : String(err)`) | garde G1, `CancelReconciliationDialog.test.ts` (cas du rejet de `cancelReconciliation`, `:97`, AC9 — non celui du chargement) |
| M15 | B13 : branche `BANK_ACCOUNT_NOT_CONFIGURED` supprimée | `ManualMatchModal.test.ts`, cas **neuf** de soumission (ii) — rejet 412 → `manual-match-bank-not-configured` présent (AC9, C-28-2-26). Le test existant `:196-227` ne soumet pas et ne la ferait pas rougir |
| M16 | garde : exemption `console` étendue à tout le fichier | auto-test G (fictive positive hors `console` dans un fichier qui journalise) |
| M17 | garde : liaisons de rappel `.catch` non recueillies | auto-tests `.catch` (AC6) + plancher « `.catch` lié ≥ 5 » + M12 |
| M18 | garde : parcours pointé sur un dossier vide | plancher « fichiers ≥ 310 » |
| M19 | `error-network` retiré de `de-CH` | `loader.rs::parity_between_locales` / `i18n-keys.test.ts` |
| M20 | repli de `error-server-status` en gabarit JS, ou site passant `{ code: … }` | `i18n-entrees-a-variables.test.ts` |
| M21 | B15 rétabli (`err instanceof Error ? err.message : String(err)` dans `formatError`) | `reports-page.test.ts` (rejet par `generate()`, AC9), garde G3 (paramètre de fonction : G1 ne le voit pas) |
| M22 | B6 rétabli (`isApiError(err) ? err.message : String(err)`) | `api-keys-page.test.ts`, garde G1 |
| M23 | `handleDelete` : `try/catch` retiré | `BankProfileList.test.ts` — l'assertion « `notifyError` appelé » (`confirm` simulé, AC9) ; le rejet non géré fait en outre sortir Vitest en échec |
| M24 | garde : une liaison dont la portée vaut `null` est sautée au lieu d'échouer | auto-test « portée incalculable » |
| M25 | garde : liaisons typées `catch (e: unknown)` non recueillies | auto-test `catch (e: unknown)` + plancher « typé ≥ 4 » |
| M26 | B14 : branche `BANK_ACCOUNT_NOT_CONFIGURED` supprimée | `TransactionSplitModal.test.ts`, cas **neuf** de soumission (ii) — rejet 412 → `split-bank-not-configured` présent (AC9, C-28-2-26) |
| M27 | garde : une portée `null` du corps-expression `.catch(` ou de l'appel `console.*` lue comme « dans l'appel » ou sautée | auto-tests « portée incalculable » des deux chemins en parenthèses (AC6, C-28-2-29) |

### Fichiers communs avec les branches 28-1 et 28-3 (non mergées)

`CHANGELOG.md` (section 0.13.3), `sprint-status.yaml` (section `epic-28`), `i18n-keys.test.ts` (`sitesTotal`), les
catalogues FTL : union à la fusion, `sitesTotal` **recompté** après rebase, jamais additionné de confiance.

### Project Structure Notes

- Modules neufs dans `frontend/src/lib/shared/utils/` (emplacement canonique des utilitaires transverses, à côté de
  `pdf-error.ts`, `notify.ts`) ; garde dans `frontend/src/lib/shared/` (à côté des gardes i18n).
- Aucun changement backend hors catalogues FTL ; aucune migration ; aucune route ; version Cargo inchangée
  (`0.13.2`, la release 0.13.3 la portera).

### References

- Issue #520 ; `15-5b-gardes-surfaces-neuves.md:713` (AC14, C-15-5b-1) et `:976-979` (suivi #520).
- `frontend/src/lib/shared/types/api.ts:9-14` ; `frontend/src/lib/shared/utils/api-client.ts:18-41`, `:221-252`,
  `:288-296` ; `frontend/src/lib/shared/utils/i18n.svelte.ts:19-27` ; `frontend/src/lib/shared/utils/pdf-error.ts`
  (patron d'utilitaire d'erreur traduit).
- `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts` (modèle de garde) ; `i18n-literal-reader.js`
  (`masquerCommentaires`) ; `i18n-harvest.js:31` (`dansLePerimetreDeFichier`).
- `CLAUDE.md` § *Inventorier les sites NON RÉSOLUS*, § *Règle de splitting préventif*, § *Test Locally First* (E2E).
- `epic-27-retro-2026-10-13.md` § 7 (B2 recette visuelle, B3 édition) et § 9 (#520).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`bmad-dev-story`, autonomie), le 2026-10-13, sur `4b31fcac` (validation close) rebasé sur `5aad8b35`.

### Debug Log References

Journaux hors dépôt, `/home/gcorbaz/devel/kesh-gate-logs/` :

- T1 « avant » : `28-2-t1-avant-{build-front,build-back,backend,e2e}.log` ;
- mutations : `28-2-mutations.log` (trois campagnes), sorties par mutation dans `28-2-mut/` ;
- gate backend : `28-2-gate-testfast.log` (+ `28-2-gate-rebuild-db.log`) ; frontend au premier passage :
  `28-2-gate-front-{check,lint-i18n-ownership,test:unit,build}.log` ;
- frontend et E2E au dernier commit de code : `28-2-final-front-{check,lint-i18n-ownership,test:unit}.log`,
  `28-2-final-{build-front,build-back,backend,e2e}.log` ;
- captures de la recette visuelle : `28-2-captures/books-lock-{avant,apres}-{light,dark}.png`.

### Completion Notes List

- Fiche créée par analyse exhaustive du contexte (bmad-create-story) — guide de développement complet.

**Ce qui a été livré, AC par AC.**

- **AC1** — `utils/api-error.ts` (garde-type, `import type` seul) et `utils/error-message.ts` (`errorMessageOf`,
  tableau de l'AC1 dans l'ordre, `console.error('[errorMessageOf]', err)` pour toute valeur qui n'est pas une
  `ApiError`). `api-client.ts` réexporte `isApiError` et ne porte plus `errorMessageOf` ; les quatre importeurs
  changent d'import.
- **AC2** — `serverStatusMessage(status)` (un site pour les trois « Erreur serveur »), `toFetchApiError` par
  `i18nMsg` ; replis = texte français d'avant, au caractère près ; un `message` du corps toujours repris.
- **AC3 / AC4** — les 4 sites A et les 16 sites B, tableau par tableau ; `handleDelete` de `BankProfileList`
  enveloppé (`notifyError`) ; branches préservées ; imports `isApiError` retirés là où ils devenaient inutiles
  (`bank-accounts/+page.svelte`, `BankAccountJournalLinkForm.svelte`).
- **AC5** — inventaire recompté au `HEAD` par les commandes du § Inventaire : la commande `String(…)` rend **5**
  lignes (3 journalisations de classe C + 2 commentaires : `ManualMatchModal.svelte:145`, doc-comment
  `error-message.ts:11`) ; `instanceof Error`, les **mêmes 5** ; gabarits, `+ ''`, `.toString()` : **0**. Classe
  A = 0, B = 0, C = 3 inchangées (`hooks.client.ts`, `auth.svelte.ts` non touchés). Classe D : non recomptée par le
  détecteur heuristique (non versionné) ; aucun fichier des quatre modules qui la portent (`credit-notes`,
  `invoices`, `payment-batches`, `supplier-invoices`) n'est touché — suivie par #644. Classe E : non touchée.
- **AC6** — garde `erreurs-en-texte.test.ts` ; `blocEquilibre` dans `i18n-literal-reader.js`, `corpsDeFonction`
  devenu son appel en accolades (ses quatre tests verts sans modification, cinq tests neufs en parenthèses).
  **Vérifiée rouge sur les sources de `5aad8b35`** (les fichiers A et B ramenés à la base, garde jouée, puis
  restaurés) : les 20 sites nommés `fichier:ligne`, les 3 journalisations exemptées. Recueil au `HEAD`, mesuré par
  la garde : **319** fichiers, **160** `catch (x)`, **4** `catch (x: T)`, **5** rappels `.catch`, **169** liaisons —
  exactement la colonne « attendu en fin de story ».
- **AC7** — trois clés × quatre catalogues, sous `# Story 28-2 — erreurs fabriquées par le client (#520)` ;
  `i18n-repli-divergent-actif`, `i18n-un-repli-par-cle`, `i18n-entrees-a-variables`, `i18n-harvest`,
  `i18n-keys` verts ; `parity_between_locales` dans le gate backend.
- **AC8 / AC9** — tests listés au File List. Les tests de soumission des deux modales pilotent le vrai
  `AccountAutocomplete` et constatent l'appel API.
- **AC10** — `error-messages.spec.ts` : rouge avant le correctif (scénario 1 : `Received "[object Object]"` ;
  scénario 2 : la clé `error-network` n'était pas servie), vert après. ⚠️ Comme la fiche l'écrit : le scénario 2
  prouve la traversée, **pas la traduction** (fr-CH, texte servi = repli) ; la traduction n'est prouvée que par
  les tests unitaires au catalogue allemand.
- **AC11** — `CHANGELOG.md` : `## [0.13.3] — Non publié`, `### Corrigé`, la phrase de l'AC11. Manuel : contrôle
  refait sur `.tex` et PDF aplatis — sortie **identique** à celle écrite pour `6f971d76` (`user-manual` 1
  « Impossible de contacter », `admin-manual` 1 « erreur serveur », brochure 0) ; « Erreur chargement profils »,
  « Profil introuvable », « ID invalide » : 0 dans les trois `.tex`, les trois PDF, le README, `website/`. Le manuel
  ne change pas.

**Mutations (T9)** — 28 jouées (M1-M27, M27 en deux variantes a/b) **plus M20b**, une par une, motif vérifié
unique, restauration `git checkout --` puis `touch`, sous le verrou ; journal `28-2-mutations.log`. **27 / 27
mutations de la fiche tuées ou déclarées** : 26 tuées par un test nommé (rouge pour la bonne raison, sorties
relues dans `28-2-mut/`) ; **M20** (repli d'`error-server-status` en gabarit JS) **survit** à cinq tests et est
**déclarée équivalente** (C-28-2-35 : rendu identique dans tous les chemins ; la garde nommée par la fiche écarte
par construction les replis non littéraux) ; **M20b** (site qui passe `{ code: … }`), la seconde forme écrite par
la fiche, est **tuée**. Incident de campagne : M19 a d'abord été **abandonnée** par le pilote, motif trouvé deux
fois (`auth-recovery-error-network` porte le même texte) ; ancrée en début de ligne et rejouée : tuée par
`cargo test -p kesh-i18n` (`i18n-keys.test.ts` reste vert — il ne compare pas les locales).

**Gates (T10)** — bases `kesh_282` / `kesh_e2e_282` reconstruites avant chaque gate complet.

| gate | commit | résultat |
|---|---|---|
| `scripts/test-fast.sh` (fmt + clippy + nextest, `DATABASE_URL` → `kesh_282`) | `5aff1a89` | **3514 exécutés, 3514 passés, 6 ignorés** |
| `npm run check` | `5aff1a89` | **rouge** : 7 erreurs TS dans le test neuf de `TransactionSplitModal` → corrigé par `4ff3a11e` |
| `npm run check` | `4ff3a11e` | 0 erreur, 29 avertissements (5 fichiers, aucun sur une ligne de la story) |
| `npm run lint-i18n-ownership` | `4ff3a11e` | PASS |
| `npm run test:unit` | `4ff3a11e` | **140 fichiers, 1483 tests passés** |
| `npm run build` | `4ff3a11e` | vert (dans le script E2E) |
| E2E complet (port 3035, `smtpConfigured:true`, 0 « Permission denied ») | `4ff3a11e` | **267 passés, 9 échoués, 19 ignorés** |

`4ff3a11e` est le **dernier commit de code** ; son diff avec `5aff1a89` ne touche qu'un fichier de test frontend
(`TransactionSplitModal.test.ts`, typage) : le gate backend n'a **pas** été rejoué sur `4ff3a11e`, et c'est
déclaré ainsi. Les **9 échecs E2E**, jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus* :
les 7 de la **KF-029** (#97 — `mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`, `:77`,
`:150`) et les 2 de la **KF-045** (#421 — `invoices:415`, `:439` ; run lancé vers 10:50 UTC, donc avant midi
UTC). **Aucun huitième de pollution** sur ce run ; aucun échec hors liste, rien à rejouer seul. Les deux scénarios
de `error-messages.spec.ts` sont **verts**.

**Décomptes, avec leur périmètre** (`4b31fcac` → `4ff3a11e`, hors `_bmad-output`) : 41 fichiers (`git diff --name-status` : 9 neufs —
2 modules, 6 tests Vitest dont la garde, 1 spec E2E —, 32 modifiés dont 4 catalogues et le CHANGELOG) ; **76**
`it(`/`test(` ajoutés (74 Vitest + 2 E2E) et **2** `it.each` ; 0 test retiré. `sitesTotal` 2075 → **2079**, mesuré
par la collecte d'`i18n-keys.test.ts` ; `sitesNonResolus`, `relais`, `sitesGabarit` inchangés. Clés : 3 × 4
(`grep -c` : 3 par catalogue). `isApiError(` : 164 appels hors tests, inchangés par la story (AC1).

**Recette visuelle (B2)** — captures « avant » (`[object Object]` sous le titre du verrou) et « après » (le message
du serveur) en clair et en sombre. **Deux constats, hors périmètre, à remonter** : (1) `text-destructive` n'a
**aucune couleur définie** (`app.css` ne porte que `--color-error` ; la classe est employée dans 40 fichiers
`.svelte`) — le message d'erreur du verrou s'affiche en **noir**, avant comme après ; (2) l'application **n'a pas de
thème sombre** (aucune règle `prefers-color-scheme` ni `.dark` dans `app.css`) : les captures « sombre » sont
identiques aux claires. L'orchestrateur vérifie si une issue couvre le (1).

**Écarts à la fiche** :

1. **M20** déclarée équivalente, **M20b** ajoutée (C-28-2-35) — la fiche attribuait M20 à une garde qui ne voit pas
   les replis en gabarit.
2. **G4** voit aussi `${String(<liaison>)}`, d'où un double constat G1 + G4 sur cette forme (C-28-2-32).
3. **`formatError`** garde `return err.message` dans sa branche `ApiError` : une `ApiError` au message vide y rend
   `''` et rien ne s'affiche (C-28-2-33) — la fiche dit « seule la dernière ligne change ».
4. **Captures** pilotées par `KESH_E2E_CAPTURES_DIR` / `_TAG` et prises **avant** l'assertion du texte
   (C-28-2-30).
5. **Le test de `BankProfileList.svelte` couvre `handleDelete` par une `ApiError`** ; la fiche ne demandait pas de
   cas `Error` pour lui (il passe par le même `errorMessageOf`, testé seul).
6. **Incident d'environnement** : le scratchpad de session est **partagé** avec l'agent de la 28-3, qui a écrasé
   le script `e2e.sh` de cet agent à 12:59, pendant le run final (Playwright fini à 13:01 ; la sortie est celle du
   script de la 28-2, le backend du port 3035 a été arrêté). Aucun effet constaté sur les résultats ; les scripts
   de gate des agents gagneraient à porter la clé de la story dans leur nom.

**Choix neufs** : C-28-2-30 à C-28-2-35 (registre `epic-28-choix-autonomes.md`, § Story 28-2).

### File List

Neufs :

- `frontend/src/lib/shared/utils/api-error.ts`
- `frontend/src/lib/shared/utils/error-message.ts`
- `frontend/src/lib/shared/utils/error-message.test.ts`
- `frontend/src/lib/shared/erreurs-en-texte.test.ts`
- `frontend/src/lib/features/bank-import/BankProfileList.test.ts`
- `frontend/src/routes/(app)/bank-import/profiles/[id]/profile-page.test.ts`
- `frontend/src/routes/(app)/settings/api-keys/api-keys-page.test.ts`
- `frontend/src/routes/(app)/reports/reports-page.test.ts`
- `frontend/tests/e2e/error-messages.spec.ts`

Modifiés :

- `CHANGELOG.md`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `frontend/src/lib/shared/utils/api-client.ts`, `api-client.test.ts`
- `frontend/src/lib/shared/i18n-literal-reader.js`, `i18n-literal-reader.test.ts`, `i18n-keys.test.ts`
- `frontend/src/routes/(app)/settings/+page.svelte`, `settings-page.test.ts`
- `frontend/src/routes/(app)/settings/api-keys/+page.svelte`
- `frontend/src/routes/(app)/bank-import/profiles/[id]/+page.svelte`
- `frontend/src/routes/(app)/bank-accounts/+page.svelte`, `bank-accounts-page.test.ts`
- `frontend/src/routes/(app)/reports/+page.svelte`
- `frontend/src/routes/(app)/reconciliation/rules/+page.svelte`
- `frontend/src/lib/features/bank-import/BankProfileList.svelte`
- `frontend/src/lib/features/bank-import/DeleteBankImportAction.svelte`, `DeleteBankImportAction.test.ts`
- `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.svelte`, `BankAccountJournalLinkForm.test.ts`
- `frontend/src/lib/features/reconciliation/CancelReconciliationDialog.svelte`, `CancelReconciliationDialog.test.ts`
- `frontend/src/lib/features/reconciliation/ManualMatchModal.svelte`, `ManualMatchModal.test.ts`
- `frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte`, `TransactionSplitModal.test.ts`
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`
- `frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte`, `RulesList.svelte`
- `_bmad-output/implementation-artifacts/28-2-erreurs-object-object.md`, `epic-28-choix-autonomes.md`,
  `sprint-status.yaml`, `28-2-review-prompt-p1.md` (neuf)

## Change Log

- 2026-10-13 — Fiche créée (bmad-create-story, Opus 5.5, autonomie), sur `6f971d76`. Inventaire : 20 sites non
  résolus (A 4, B 16), C 3 légitimes, D ≈ 23 assumés, E conformes. Choix C-28-2-1 à C-28-2-14. Statut ready-for-dev.
- 2026-10-13 — **Validation P1** (Sonnet 5.5, lentilles R et F, rapports hors dépôt
  `kesh-gate-logs/28-2-validate-p1-{R,F}.md`) : **0 CRITICAL, 0 HIGH, 7 MEDIUM, 11 LOW** (F : 4 MEDIUM, 4 LOW ;
  R : 3 MEDIUM, 7 LOW). Convergences : F1 = R1 (l'AC11 niait à tort qu'un manuel cite ces messages :
  `user-manual.tex:1871-1873`, citation coupée sur deux lignes) ; F5 = R8 ; F6 ≈ R5. Remédiation (Opus 5.5) :
  - **F1/R1** — AC11 réécrit : le passage est cité, il reste exact en français, ce qu'il devient dans les autres
    langues est écrit, le manuel ne change pas ; contrôle sur texte aplati, `.tex` et PDF (C-28-2-15).
  - **F2** — AC6 : portée de liaison définie pour `catch`, `.catch` à bloc et `.catch` à corps-expression ; une portée
    incalculable fait échouer la garde ; auto-tests avec accolades dans une chaîne, un gabarit, une regex (C-28-2-16).
  - **F3** — AC6 : planchers par forme (fichiers ≥ 310, non typé ≥ 150, typé ≥ 4, `.catch` lié ≥ 5, total ≥ 160) et
    auto-tests de chaque forme de liaison (C-28-2-17).
  - **F4, R2, R3** (classe D) — décision de l'orchestrateur : **hors périmètre**, issue **#644** ouverte par lui ; la
    fiche la cite en angle mort (`refs #644`), avec les deux silences voulus d'`invoices/[id]` ; aucune autre issue
    de suivi n'est demandée (C-28-2-22). AC5 et § Inventaire mis à jour en conséquence.
  - **LOW retenus** : F5/R8 (classe C : la valeur brute est journalisée, C-28-2-19) ; F6/R5 (portée réelle de la
    traduction : angle mort n° 5, CHANGELOG exact, ce que l'E2E scénario 2 n'établit pas — C-28-2-23) ; F7 (état de
    module du catalogue, échec de `loadI18nMessages`, `ApiError` à `message` non textuel — C-28-2-24) ; F8 (G3 étendue
    à toute forme, tout le fichier — C-28-2-18) ; R4 (trois gardes i18n nommées à l'AC7) ; R6 (recompte : 256 `catch`
    hors commentaires, 163 liés dont 4 typés, 168 liaisons) ; R7 (deux tests de page neufs, `reports` et
    `api-keys`, C-28-2-21 ; formulation « mécanique » de l'AC4 corrigée ; A2 et `BankAccountJournalLinkForm.test.ts`
    précisés) ; R9 (classe E redéfinie, replis en littéral ajoutés à l'angle mort n° 3) ; R10 (`handleDelete` de
    `BankProfileList.svelte` corrigé dans la story, C-28-2-20).
  - Mutations M21-M25 ajoutées ; dérogation au découpage réexaminée (deux fichiers d'infrastructure, aucun module
    métier de plus). Choix C-28-2-15 à C-28-2-24. Validation P2 : `28-2-validate-prompt-p2.md` (Opus, R et F).
- 2026-10-13 — **Validation P2** (Opus 5.5, lentilles R et F, rapports hors dépôt
  `kesh-gate-logs/28-2-validate-p2-{R,F}.md`) : **0 CRITICAL, 0 HIGH, 3 MEDIUM, 18 LOW** rapportés (F : 1 MEDIUM,
  10 LOW ; R : 2 MEDIUM, 8 LOW). Convergences : F1 = R2 ; F2 = R3 ; F6 = R10 ; F11 = R6 ; F7 et F9 sont repris par
  R9 — soit **2 MEDIUM et 13 LOW distincts**. Trend : P1 7 MEDIUM → P2 2 MEDIUM distincts. Branche rebasée sur
  `origin/main` (kickoff de l'Epic 28, `5aad8b35`) ; `sprint-status.yaml` réuni par union ; registre de branche
  versé par union dans `epic-28-choix-autonomes.md` (§ Story 28-2), fichier de branche supprimé. Remédiation
  (Opus 5.5) :
  - **F1 = R2** (MEDIUM, origine : remédiation P1) — **C-28-2-19 retiré** (C-28-2-25) : les trois `catch` de classe C
    n'enveloppent que des `fetch` directs et ne reçoivent que des `Error` ; classe C = 3, légitimes, inchangées,
    sous l'exemption `console` ; `hooks.client.ts` et `auth.svelte.ts` hors périmètre. AC5, tableau de la classe C,
    G3 (6 lignes disparaissent, 3 restent exemptées), exemption `console`, T4, § État actuel, dérogation et
    C-28-2-9 (note de révision) corrigés.
  - **R1** (MEDIUM, origine : rédaction d'origine) — AC9 : tests de **soumission** de `ManualMatchModal` et
    `TransactionSplitModal`, à écrire de zéro, avec le vrai `AccountAutocomplete` (focus, `mouseDown` sur
    l'option, comptes de classe 5 à 7 actifs et postables ; deux lignes équilibrées pour la ventilation) ; un cas
    412 par modale tue M15 (B13) et **M26** (B14, neuve) (C-28-2-26).
  - **LOW** : F2 = R3 (G1 « B2-B14 », M2 et M21 « garde G3 », `formatError` à l'angle mort n° 4) ; F3 (G3 : tout
    usage, classification comprise, motif `\binstanceof\s+Error\b`, C-28-2-27) ; F4 et F5 (formes typée et
    `async`, portée `null` en échec sur les trois chemins, M27, C-28-2-29) ; F6 = R10 (plancher total retiré,
    valeurs de fin de story écrites, C-28-2-28) ; F7, F9 et R9 (`confirm` simulé, M23 par l'assertion
    `notifyError`, `reports-page` par `generate()` et `data` en prop, motif saisi pour A2, sites de M13 et M14
    nommés) ; F8 (C-28-2-9 révisé, crate `kesh-i18n` compté) ; F10 (alias d'une erreur à l'angle mort n° 4) ;
    F11 = R6 (CHANGELOG : chargée ou rechargée, interface en français jusqu'au premier rechargement qui suit une
    connexion) ; R4 (`i18n-harvest.test.ts` travaille sur fixtures, ne voit aucun site neuf) ; R5 (liste des cinq
    `.catch` liés remise à sa place) ; R7 (53 fichiers importeurs, 166 appels, au lieu de « ~110 ») ; R8 (classe D :
    quatre modules métier, non sept).
  - **Signal de découpage (D5) déclaré au Project Lead** : la sévérité MEDIUM se maintient de P1 à P2, et F1 = R2
    est **né de la remédiation P1** (C-28-2-19, repris de F5/R8) — la forme « recyclage » que D5 vise. Constat : sa
    correction est un **retrait** (deux fichiers et un module sortent du périmètre), non un correctif de plus ; R1
    est un défaut d'origine, distinct ; aucune dispersion neuve. L'agent ne découpe pas ; la décision revient au
    Project Lead.
  - Mutations M26, M27 ajoutées (M1-M27) ; choix C-28-2-25 à C-28-2-29. Passe ciblée P3 :
    `28-2-validate-prompt-p3-ciblee.md` (Haiku, une lentille, diff aplati de cette remédiation).
- 2026-10-13 — **Validation P3, passe ciblée** (Haiku 5.5, une lentille, diff de `eb9c68c8` ; rapport hors dépôt
  `kesh-gate-logs/28-2-validate-p3-ciblee.md`) : **0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW**, tous deux nés de la
  remédiation P2, tous deux vérifiés au sol avant patch. Remédiation :
  - **P3-1** — AC9, `reports-page.test.ts` : `:473` retiré de la liste des `catch` de `generate()` qui appellent
    `formatError` sans condition ; il est dans `exportReport` (`:388-484`), gardé par `isApiError(e) && e.code`, et
    rejoint la liste des exports (`:403`, `:437`, `:473`, `:505`).
  - **P3-2** — AC1 : « 166 appels » → **164 appels** (166 occurrences de `isApiError(` hors tests, moins la
    déclaration `api-client.ts:19` et le doc-comment `notify.ts:89`), commande jointe, recomptée à 164. Les entrées
    passées du registre (C-28-2-2, C-28-2-24) et de ce Change Log (P2, R7) ne sont **pas réécrites** : le registre
    reçoit une annotation datée (consigne 15).
  - **Clôture de la validation.** Trend : P1 (Sonnet) **7 MEDIUM** → P2 (Opus) **2 MEDIUM** distincts → P3 ciblée
    (Haiku) **0 MEDIUM**. La remédiation P3 ne touche aucune ligne de code (fiche et registre seuls) : la boucle est
    close. **Signal D5** déclaré au Project Lead en P2 (MEDIUM maintenu de P1 à P2, F1 = R2 né de la remédiation P1) :
    traité par un **retrait de périmètre** (C-28-2-19 retiré, classe C inchangée), non par un découpage. Statut
    `in-progress`.
- 2026-10-13 — **Développement** (`bmad-dev-story`, Opus 5.5, autonomie) : T0-T10 faits ; commits `e4016aeb`
  (utilitaire, traduction à la source, catalogues), `f744b4dc` (classes A et B), `86705aa9` (garde,
  `blocEquilibre`), `bb45f4a0` (tests de la classe B, modales, E2E), `5aff1a89` (CHANGELOG), `4ff3a11e` (typage
  d'un test — dernier commit de code). Mutations : 26 tuées, M20 déclarée équivalente, M20b ajoutée et tuée.
  Gates : backend 3514/3514 (6 ignorés) à `5aff1a89` ; frontend complet et E2E complet (267 passés, 9 échecs
  attendus KF-029 ×7 et KF-045 ×2) à `4ff3a11e`. Choix C-28-2-30 à 35. Écarts et constats de recette au Dev Agent
  Record. Statut `review` ; prompt de revue P1 : `28-2-review-prompt-p1.md`.
