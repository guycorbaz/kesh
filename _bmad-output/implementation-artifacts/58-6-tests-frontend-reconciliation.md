# Story 58-6 : Tests frontend de la réconciliation — dix trous de couverture et une garde morte (#1078, #1079, #1080, #1082, #1083, #1084, #1088, #1090, #1091, #1093)

Status: drafted

*Fiche rédigée le 2026-11-28 (Sonnet 5.5, en autonomie), workflow `bmad-create-story`. Worktree `/home/gcorbaz/devel/kesh-58-6`, branche
`story/58-6-tests-frontend-reconciliation` partie de `epic/65-nettoyage-suite` / `main` à `287deea33` (v0.20.1). Fichier d'epic :
`_bmad-output/planning-artifacts/epic-65-nettoyage-suite.md` (§ *Story 58-6*). Epic 65 (nettoyage), release visée : vendredi 2026-12-04 (coupure jeudi 2026-12-03 au soir).
Port E2E **3656**, bases `kesh_586` et `kesh_e2e_586`. **Aucune migration, aucun changement d'API, aucun texte visible, aucune règle métier changée** : la story ne touche que des
tests, et un seul morceau de code de production, la garde morte de #1088 (retrait). Pour écrire la fiche, une seule mutation a été jouée (retrait de l'effet de
`ManualMatchModal.svelte`, restauré, `git status` vide) ; aucun gate complet n'a tourné.*

## Story

En tant que mainteneur de Kesh, je veux que dix tests de la réconciliation, que la revue de code avait relevés comme verts sans rien prouver (assertion qui passe aussi sur l'ancien
code, état déjà vrai avant ce qu'on attend, clic synthétique, nombre qui accepte « 10 », bornes non testées, comportement non fixé), prouvent ce qu'ils annoncent, et qu'une garde
que plus aucun chemin n'atteint soit retirée, afin qu'une mutation du code de production fasse rougir un test, et non qu'on le découvre sur les livres.

## Contrôle de couverture (règle du 2026-11-26 : on vérifie qu'une issue n'est pas déjà couverte avant de la retenir)

Chaque issue a été relue (`gh issue view`, lecture seule) et le test visé cherché au code de `287deea33`. `git log --all --grep` sur chaque numéro ne rend aucun commit. Les dix issues
sont ouvertes ; **aucune n'est déjà corrigée, aucune ne révèle de défaut de production** : les dix restent. Les numéros de ligne de la colonne « au code » remplacent ceux des issues (qui ont bougé).

| # | Au code (`287deea33`) | Verdict |
|---|---|---|
| #1078 | `reconciliation-manual-entry.spec.ts:239` : `expect(corps.counterpartyAccountId).toBe(capital!.id)` ; le test (l. 173-247) rougit par `toHaveValue(libelle)` (l. 229) avant d'y arriver. Avant la 35-7, le brouillon `zzz` restait affiché mais `2000` partait déjà : l'assertion est verte sur l'ancien code. | **gardée** (l. 239, non 236-237) |
| #1079 | `ManualMatchModal.test.ts:340` `expect(champ().value).toBe('')` précède `:341` `submit.disabled` et `:343` (`manualMatchTransaction` non appelé). **Vérifié par mutation** (effet `keepOfferedCounterparty` du composant neutralisé) : l. 340 **passe**, l. 341 rougit seule. L'assertion du texte est donc muette, mais le test **rougit** déjà ; le défaut est l'ordre et l'absence d'explication. | **gardée**, plus étroite que l'issue : réordonner et dire pourquoi |
| #1080 | `TransactionSplitModal.test.ts:553` (« deux comptes restés proposés ne sont pas touchés ») et `ManualMatchModal.test.ts:348` (« un compte resté proposé n’est pas touché… »). **Vérifié par mutation** : avec l'effet neutralisé, le test Manual de la l. 348 **reste vert** (16 verts sur 17, seul le test de la l. 340 rougit). | **gardée** |
| #1082 | `reconciliation-manual-entry.spec.ts:220` `await option.dispatchEvent('mousedown');` (bloc 35-7). La l. 133 (test de la 34-3, hors issue) fait de même : jumeau corrigé dans la story (D3, C-58-6-k, AC13). | **gardée** |
| #1083 | `TransactionSplitModal.svelte:110-116` (effet par ligne) ; `TransactionSplitModal.test.ts:518` (ligne 1) et `:537` (ligne 2) : deux lignes seulement ; `split-add-line` n'est cliqué qu'aux l. 218, 408 et 695, hors du bloc 35-7. `MAX_SPLITS` = 50. | **gardée** |
| #1084 | `AccountAutocomplete.svelte:340-370` (`handleKeydown`) n'a pas de branche `Tab` ; `commitDraft` (l. ~465-500) jette le brouillon d'un champ strict. `grep -n Tab AccountAutocomplete.test.ts` : vide. Aucun test ne fixe « numéro complet puis Tab ». | **gardée** (l. 340-370 et 465-500, non 255-273 et 297-300) |
| #1088 | `ReconciliationProposals.svelte:520` `if (!current) return;`. **Inatteignable** : `proposals` n'est écrit qu'à la l. 204 (dans `load()`), qui commence par `pendingGesture = null` (l. 182) et passe `loading` à vrai, ce qui démonte la confirmation (`{:else}` de `{#if loading}`, l. 636) ; la sélection est vidée avec elle. Le test d'invariant existe : `ReconciliationProposals.test.ts:1899` (AC4, M4), mais pour quatre gestes sur cinq (voir D2). | **gardée** ; tranchée : **retrait** (D2) |
| #1090 | `reconciliation-already-booked-reject.spec.ts:170-177` `cliquerHors` : `expect(async () => { page.mouse.click(5, 5); expect(confirmation).toHaveCount(0, { timeout: 500 }) }).toPass({ timeout: 5000 })`. Appelée l. 235 et 335. | **gardée** |
| #1091 | `reconciliation-already-booked-reject.spec.ts:242` `toContainText('1')`. | **gardée** (l. 242) ; **site jumeau** `reconciliation-reject-unmatched.spec.ts:173`, corrigé dans la story sans issue (C-58-6-k, AC14) |
| #1093 | `ReconciliationProposals.test.ts:1790` (une signalée parmi deux cochées, `introRejetDes(2)`) et `:1811` (une cochée et signalée, `INTRO_REJET_UNE`). Ni trois cochées dont une signalée, ni toutes signalées. | **gardée** |

## Ce qui se lit au code (`287deea33`)

- **Montage des tests 35-7.** `CounterpartyModalHost.test.svelte` passe les props une à une et ne change que `bankLedgerAccountId` (bouton `host-resolve-bank`) : un `rerender` rejouerait la remise à zéro du dialogue. Les
  deux modales ont le même effet : `ManualMatchModal.svelte:94-97` (un champ) et `TransactionSplitModal.svelte:110-116` (une boucle sur les lignes), tous deux fondés sur
  `keepOfferedCounterparty` (`reconciliation-counterparties.ts:54-63` : `null` si le compte est celui de la banque, l'identifiant sinon, y compris un identifiant absent de `accounts`).
- **Message d'*Éclater*.** `TransactionSplitModal.svelte:232-241` rend « Ligne N : compte requis » pour la **première** ligne sans compte seulement : avec les lignes 2 et 3 vidées, c'est « Ligne 2 ».
- **Le champ vide ne discrimine pas.** Quand le compte de la banque sort des options, le champ s'affiche vide même si la valeur garde l'identifiant ; ce qui discrimine est
  le bouton désactivé et l'absence d'appel de `manualMatchTransaction` / `splitTransaction`.
- **`confirmGesture`** (`ReconciliationProposals.svelte:509-523`) : gère `accept` et `reject` (sélection relue), puis relit la ligne des trois autres gestes dans `proposals` ; seul le `if (!current) return;` de la l. 520 est à l'écart.
  Son doc-comment (l. 494-508) parle de « gardes défensives ».
- **Mutation de contrôle (jouée pour écrire cette fiche, restaurée)** : `if (kept !== counterpartyId) counterpartyId = kept;` remplacé par `void kept;` dans `ManualMatchModal.svelte` → `ManualMatchModal.test.ts` : **1 rouge sur 17**,
  « le compte de la banque choisi, puis identifié… » à la l. 341 (`submit.disabled`), alors que la l. 340 (`champ().value === ''`) passe ; journal `kesh-gate-logs/e65/58-6/mutation-1079-manual.log`.

## Décisions

- **D1 — #1078 : commenter et attacher au libellé, pas inventer un cas.** L'ancien code envoyait déjà le même compte : aucun cas ne fait envoyer à l'ancien code « un autre compte que celui affiché » sans changer le produit. La fiche retient les deux gestes de
  l'issue qui sont honnêtes : (a) l'assertion de la l. 239 se relie au **libellé affiché** (`libelle` relevé à la l. 223 : on en tire le numéro, on retrouve l'identifiant dans `accounts`, on compare au corps du `POST`) — elle dit « le compte affiché est
  le compte envoyé » au lieu d'un identifiant posé d'avance ; (b) un commentaire écrit que l'assertion ne rougit **pas** sur le code d'avant la 35-7 (c'est `toHaveValue` qui porte ce rouge) et qu'elle garde le défaut inverse (un champ
  qui montre un compte et en envoie un autre). Raison : une story de nettoyage ne bâtit pas de produit pour rougir un test ; elle écrit ce que le test prouve.
- **D2 — #1088 : retirer la garde et la relecture qui l'accompagne, après avoir étendu l'invariant au cinquième geste.** *Exercer* la garde exigerait un état que ni l'interface ni `load()` ne produisent (la confirmation est
  fermée avant que `proposals` change : l. 182, 204, 636) : un test qui le fabriquerait (en forçant `pendingGesture`) prouverait un état irréalisable. Le *retrait* demande de ne laisser aucune ligne `find` sans garde (TypeScript la refuse : `current` est
  `ReconciliationProposal | undefined`) : `confirmGesture` devient `const gesture = pendingGesture; pendingGesture = null; if (!gesture) return; runGesture(gesture);` — le `if (!gesture)` reste (typage, et double clic sur *Continuer*) ; les trois gestes `manual`/`split`/`group` reçoivent
  `gesture.proposal`, **le même objet** que `proposals.find(...)` aurait rendu (la ligne vient de `proposals`, qui n'a pas été réécrit depuis). Le doc-comment de la fonction se réécrit (plus de « gardes défensives »).
  **Le retrait repose sur l'invariant « une relecture referme la confirmation »** ; or le test qui le tient (`ReconciliationProposals.test.ts:1899`) ne couvre que `accept`, `reject`, `manual`, `split`, **pas `group`** (déclaré par 43-3). La story l'étend à `group-settlement-button` avant de retirer : sans cela, le retrait
  s'appuierait sur un geste que rien n'éprouve. Si la relecture ne referme pas la confirmation d'un geste `group` (le test rougit), **la garde est conservée** et la story se ferme sur ce constat : un vrai défaut de production, à sortir en issue.
- **D3 — #1082 : le bloc 35-7, et les sites jumeaux du module (C-58-6-k).** `reconciliation-manual-entry.spec.ts:133` (test de la 34-3), `reconciliation-split.spec.ts:152` et `reconciliation-group.spec.ts:304`, `:394`, `:752` emploient aussi `dispatchEvent('mousedown')` : même défaut, même module, même genre (inventaire : `grep -rnF "dispatchEvent('mousedown')" frontend/tests/e2e`, sept sites, dont `credit-card-statement.spec.ts:121`, hors module). Décision de l'orchestrateur du 2026-11-28 : pas d'issue ; le défaut se corrige dans la story (§ *Priorités des défauts*, « un LOW se corrige dans la story »), en tâche **sans issue** rattachée à #1082 (AC13).
- **D4 — #1084 : on fixe le comportement, on ne le change pas.** Tab ne choisit rien (D2 de la 35-7) : le test l'écrit tel quel. Faire choisir l'unique correspondance exacte serait une règle d'interface nouvelle : décision du Project Lead, hors de cette story.
- **D5 — #1090 : un état observable puis un seul clic ; repli à 100 ms mesuré.** Attendre que les animations de la confirmation soient terminées (`confirmation.evaluate(el => Promise.all(el.getAnimations().map(a => a.finished)))`) puis **un seul** `page.mouse.click(5, 5)` et `toHaveCount(0)` ;
  si le rejeu discriminant (AC8) montre que cela ne suffit pas (les écouteurs de bits-ui se posent après `afterSleep(1)` et le clic est traité après un `debounce` de 10 ms : C-e354-2), repli : `waitForTimeout(100)` documenté, jamais une boucle de clics. Le commentaire de `cliquerHors` se réécrit. L'attente des animations n'est qu'un **indice** : elle mesure la fin des animations de 100 ms du dialogue (`dialog-content.svelte`, `duration-100`), non la pose des écouteurs de bits-ui ; la corrélation est probable, non établie, et c'est le `--repeat-each=10` de l'AC8 qui la confirme.
- **D6 — #1080 pour *Écriture manuelle* : une doublure qui observe l'effet.** *Écriture manuelle* n'a qu'un champ : on ne peut pas y vider « une autre ligne ». L'effet se prouve en enveloppant `keepOfferedCounterparty` d'un espion (`vi.mock('./reconciliation-counterparties', async (importOriginal) => …` avec
  `vi.fn(original)`) et en exigeant qu'il ait été appelé avec le compte conservé et l'identifiant de la banque (410) **après** `host-resolve-bank`. Pour *Éclater*, la forme de l'issue (une autre ligne sur la banque) : trois lignes, deux sur `6900`, une sur `1020`. `vi.mock` est hissé et vaut pour **tout le fichier** `ManualMatchModal.test.ts` : le choix est un mock de fichier avec l'original étalé (`{ ...original, keepOfferedCounterparty: vi.fn(original.keepOfferedCounterparty) }`), sans danger pour les autres tests (`clearAllMocks` garde l'implémentation, `restoreAllMocks` ne touche pas un `vi.fn` sous Vitest 4).
- **D7 — #1091 : le motif des autres specs.** `toHaveText(/^\s*1\s+\D/)`, déjà employé par `reconciliation-priority.spec.ts:247` et `reconciliation-deleted-import.spec.ts:177` : le nombre exact, puis un mot.

## Critères d'acceptation

1. **#1078** — `reconciliation-manual-entry.spec.ts:239` compare le corps du `POST` à l'identifiant du compte dont le **libellé affiché** porte le numéro ; un commentaire dit que cette assertion ne rougit pas sur le code d'avant la 35-7. Aucune autre assertion du test ne change.
2. **#1079** — `ManualMatchModal.test.ts`, test de la l. 340 **et** `TransactionSplitModal.test.ts`, tests des l. 518 et 537 (même ordre : texte vide d'abord, bouton ensuite ; même raisonnement) : l'ordre devient valeur nulle d'abord (`submit.disabled`, appel de l'API non fait), puis texte et message ; un commentaire dit que `champ().value === ''` seul est vrai sans l'effet.
3. **#1080** — *Éclater* : le test « deux comptes restés proposés » (l. 553) a trois lignes (`6900`, `6900`, `1020`) : lignes 1 et 2 intactes, ligne 3 vidée, « Ligne 3 : compte requis », bouton désactivé. *Écriture manuelle* : le test de la l. 348 exige l'appel de `keepOfferedCounterparty` avec `(401, <comptes>, 410)` après `host-resolve-bank`, en plus du champ et du bouton inchangés.
4. **#1082** — `reconciliation-manual-entry.spec.ts:220` : `await option.click();`. Si le montage (liste qui se referme au `blur` avant le `click`) l'empêche, la raison s'écrit en commentaire et la tâche **s'arrête** ; ne jamais retomber en silence sur `dispatchEvent`.
5. **#1083** — un test à trois lignes d'*Éclater* : lignes 2 et 3 sur le compte de la banque, ligne 1 sur `6900` → ligne 1 intacte, lignes 2 et 3 vides, « Ligne 2 : compte requis » (et non « Ligne 3 »), `splitTransaction` non appelé.
6. **#1084** — un test d'`AccountAutocomplete` : valeur `1` (`1000 Caisse`), un second compte `6900`, saisie `6900` (liste ouverte, option désignée), `Tab` (keydown) puis `blur` : `onSelect` **non appelé**, le champ réaffiche `1000 — Caisse`.
7. **#1088** — `confirmGesture` sans `find` ni garde `!current` (D2) ; doc-comment réécrit ; `ReconciliationProposals.test.ts:1899` couvre aussi `group-settlement-button` ; `npm run check` vert.
8. **#1090** — `cliquerHors` n'a plus de boucle de clics (D5), **un seul** clic ; rejoué `--repeat-each=10` sur les deux tests qui l'appellent, sans rouge.
9. **#1091** — l. 242 : `toHaveText(/^\s*1\s+\D/)` (D7).
10. **#1093** — deux tests de plus : trois cochées dont une signalée → `introRejetDes(3)` et un seul élément nommé ; toutes les cochées signalées (deux, ou trois) → autant d'éléments que de lignes, `introRejetDes(n)`.
11. **Preuve du rouge** — chaque test renforcé ou ajouté rougit sur sa mutation nommée (§ *Preuve du rouge*), jouée puis restaurée avec `touch`, journaux dans `kesh-gate-logs/e65/58-6/`.
12. **Aucun effet hors périmètre** — `git diff` : tests, `ReconciliationProposals.svelte` (garde #1088 seulement), le fichier de la story, `sprint-status.yaml` et les artefacts de `_bmad-output/implementation-artifacts/` (prompts de validation et de revue) ; aucun `.ftl`, aucun `.rs`, aucune migration. **Recette visuelle : sans objet, aucun contrôle visible touché.**
13. **Jumeau de #1082 (C-58-6-k, sans issue)** — `reconciliation-manual-entry.spec.ts:133` (test de la 34-3) : `await option.click();` ; même réserve qu'à l'AC4 (si le montage l'empêche, la raison s'écrit et la tâche s'arrête).
14. **Jumeau de #1091 (C-58-6-k, sans issue)** — `reconciliation-reject-unmatched.spec.ts:173` : `toHaveText(/^\s*1\s+\D/)` à la place de `toContainText('1')`.
15. **Jumeau de #1082 (C-58-6-k, sans issue)** — `reconciliation-split.spec.ts:152` : `await option.first().click();` ; même réserve qu'à l'AC4.
16. **Jumeau de #1082 (C-58-6-k, sans issue)** — `reconciliation-group.spec.ts:304` : `.first().click()` à la place de `.first().dispatchEvent('mousedown')` ; même réserve.
17. **Jumeau de #1082 (C-58-6-k, sans issue)** — `reconciliation-group.spec.ts:394` : idem.
18. **Jumeau de #1082 (C-58-6-k, sans issue)** — `reconciliation-group.spec.ts:752` : idem.

## Preuve du rouge

À appliquer par **Edit** (jamais `sed -i`), faire tourner le test nommé, noter le rouge, **restaurer le fichier puis le `touch`** (un binaire ou un cache de test garde la mutation : `memory/mutation-restauree-binaire-perime`), recontrôler `git status`.

| AC | Test | Mutation (fichier) | Rouge attendu |
|---|---|---|---|
| 2 | `ManualMatchModal.test.ts` (l. 340) | `ManualMatchModal.svelte:96` : `void kept;` à la place de `if (kept !== counterpartyId) counterpartyId = kept;` | rouge sur `submit.disabled` **en premier** (déjà observé le 2026-11-28 : 1 rouge sur 17) ; le texte seul passe |
| 3 | « un compte resté proposé » (Manual) | `ManualMatchModal.svelte:95` : `const kept = counterpartyId;` à la place de l'appel à `keepOfferedCounterparty` (ou retrait du bloc `$effect` l. 93-97) : l'effet **ne tourne plus** | **rouge** sur l'espion non appelé avec `(401, <comptes>, 410)` (aujourd'hui vert). Sous `void kept;` (mutation de l'AC2), ce test reste **vert**, et c'est voulu : l'espion prouve que l'effet tourne, non qu'il écrit |
| 3 | « deux comptes restés proposés » (Éclater, trois lignes) | `TransactionSplitModal.svelte:110-116` : boucle supprimée | rouge sur « ligne 3 vidée » (aujourd'hui vert) |
| 5 | Éclater, lignes 2 et 3 | même boucle, `for (let i = 0; i < 1; i++)` (seule la ligne 1 est examinée) | rouge : lignes 2 et 3 gardent la banque |
| 5 | idem | boucle `for` qui s'arrête à la première ligne retirée (`break` après la première écriture) | rouge : ligne 3 non vidée |
| 6 | `AccountAutocomplete` « numéro complet puis Tab » | `AccountAutocomplete.svelte` `handleKeydown` : ajouter `else if (e.key === 'Tab') { const d = open ? filtered[highlightIndex] : undefined; if (d) handleSelect(d); }` | rouge : `onSelect` appelé avec 6 |
| 10 | trois cochées, une signalée | `ReconciliationProposals.svelte` `guard()` : `confirmSelectedCount = Math.min(selected.size, 2)` (au lieu de `selected.size`) | rouge (2 ≠ 3) ; les cas existants 1→1 (l. 1811) et 2→2 (l. 1790) restent verts : seul le nouveau test détecte |
| 10 | toutes signalées | `guard()` : `confirmTransactions = flagged.slice(0, 1)` | rouge : un seul élément nommé |
| 9 | E2E de rejet (l. 242) | `ReconciliationProposals.svelte:615` : `lastSuccessCount = r.rejected.length + 9;` (affiche « 10 opération(s) réussie(s). ») | rouge avec le motif ; **vert** avec l'ancien `toContainText('1')` (c'est la démonstration de l'issue) |
| 8 | E2E de rejet, `cliquerHors` | `AlreadyBookedConfirmDialog.svelte` : ignorer la première fermeture (`let ignore = true; onOpenChange={(v) => { if (!v) { if (ignore) { ignore = false; return; } onCancel(); } }}`) | **démonstration valable pour le premier test seulement (l. 235)** : rouge avec **un seul** clic ; vert avec l'ancienne boucle ; elle ne vaut pas pour le second (l. 335), que la mutation fait rougir dès l'Échap (`page.keyboard.press('Escape')`, l. 328), avant `cliquerHors` (démonstration de l'issue) |
| 7 | `ReconciliationProposals.test.ts:1899`, cas `group` | `load()` : supprimer `pendingGesture = null;` (l. 182) | rouge pour les **cinq** gestes (aujourd'hui quatre) : c'est le filet sous le retrait de la garde |
| 1 | E2E 35-7, l. 239 | `ManualMatchModal.svelte` : envoyer `filteredAccounts[0].id` au lieu de `counterpartyId` | rouge sur la comparaison libellé/identifiant ; **ne rougit pas** sur le code d'avant la 35-7 (écrit au commentaire, AC1) |
| 14 | `reconciliation-reject-unmatched` (l. 173) | même mutation que l'AC9 (`lastSuccessCount = r.rejected.length + 9`, chemin *Rejeter*) | rouge avec le motif ; vert avec l'ancien `toContainText('1')` |
| 13, 15 à 18 | E2E `reconciliation-manual-entry.spec.ts:133`, `reconciliation-split.spec.ts:152`, `reconciliation-group.spec.ts:304`, `:394`, `:752` (`option.click()`) | même mutation que l'AC4 (choix déplacé sur `onclick`), jouée dans `AccountAutocomplete.svelte`, composant commun du champ | rouge avec un vrai clic, mêmes réserves qu'à l'AC4 |
| 4 | E2E 35-7, `option.click()` | `AccountAutocomplete.svelte` : le choix de l'option déplacé de `onmousedown` (l. 617-619) vers `onclick`, sans `preventDefault` | rouge avec un vrai clic : le `blur` part, `open = false` démonte la liste, le `click` se perd, le champ ne prend pas le compte. **Écart déclaré** : sous cette mutation `dispatchEvent('mousedown')` rougit aussi (plus de gestionnaire) ; la preuve est donc que le test à vrai clic **peut** rougir, non qu'il détecte ce que l'ancien ne détectait pas |

*Écart déclaré (F3 de la validation P1)* : un vrai clic ne provoque aucun `blur` ici (l'option choisit sur `mousedown` et annule le geste par défaut, `AccountAutocomplete.svelte:166, 617`) ; la séquence `mousedown` → `blur` → `click` de #1082 n'existe donc pas telle quelle, et aucune mutation de `handleBlur` ne passe avant le choix. La mutation retenue est « choix déplacé sur `onclick` » ; elle ne départage pas l'ancien du nouveau test (voir la ligne de l'AC4) mais prouve que le test à vrai clic rougit : aucune exception à l'AC11.

## Tâches

- [ ] **T0 — Base et état** (AC: 11, 12)
  - [ ] `git status` vide, branche `story/58-6-tests-frontend-reconciliation` ; bases `kesh_586` / `kesh_e2e_586` ; `docs/testing.md` § *Prérequis Playwright local* à la lettre (port 3656).
  - [ ] Relire chaque ligne citée ici au code (les numéros de ligne ont pu bouger) ; ne rien écrire que le code ne montre.
- [ ] **T1 — Vitest d'*Écriture manuelle* et d'*Éclater*** (AC: 2, 3, 5)
  - [ ] Réordonner et commenter les tests des l. 340 (Manual) et 518 (Éclater) ; espion `keepOfferedCounterparty` ; trois lignes dans le test « deux comptes restés proposés » ; test des lignes 2 et 3.
  - [ ] Jouer les mutations des AC 2, 3, 5 ; restaurer, `touch`.
- [ ] **T2 — `AccountAutocomplete`** (AC: 6)
  - [ ] Test « numéro complet puis Tab » ; mutation ; restaurer.
- [ ] **T3 — `ReconciliationProposals.test.ts`** (AC: 7, 10)
  - [ ] Deux tests de bornes (AC10) ; `group-settlement-button` dans le `it.each` de l. 1893 (AC7).
  - [ ] Mutation `pendingGesture = null` retirée de `load()` : les cinq gestes rougissent ; restaurer.
- [ ] **T4 — Garde de #1088** (AC: 7)
  - [ ] **Seulement si T3 a montré que les cinq gestes rougissent sans `pendingGesture = null`.** Réécrire `confirmGesture` et son doc-comment (D2) ; `grep -rn "confirmGesture\|défensives" frontend docs _bmad-output/implementation-artifacts` pour propager le symptôme ; `npm run check`.
- [ ] **T5 — E2E** (AC: 1, 4, 8, 9)
  - [ ] `reconciliation-manual-entry.spec.ts` : AC1, AC4 ; `reconciliation-already-booked-reject.spec.ts` : AC8, AC9. Mutations des AC 1, 4, 8, 9 (le front se reconstruit pour chaque mutation : `npm run build`).
  - [ ] Jumeaux sans issue (C-58-6-k) : AC13 (`reconciliation-manual-entry.spec.ts:133`), AC15 à AC18 (`reconciliation-split.spec.ts:152`, `reconciliation-group.spec.ts:304`, `:394`, `:752`) et AC14 (`reconciliation-reject-unmatched.spec.ts:173`, ajouter cette spec aux E2E ciblés).
  - [ ] `--repeat-each=10` sur les deux tests de la spec de rejet ; `--repeat-each=3` sur le test 35-7 de la spec d'écriture manuelle.
- [ ] **T6 — Gates et clôture** (AC: 11, 12)
  - [ ] Gates ci-dessous ; Dev Agent Record : mutations jouées (log cité), `git diff --stat`, chiffres recomptés.

## Gates

- **Frontend** (voie frontend, `flock …/.gate-front.lock scripts/mem-guard.sh`) : Vitest ciblé par fichier à chaque tâche — `ManualMatchModal.test.ts`, `TransactionSplitModal.test.ts`, `ReconciliationProposals.test.ts`, `AccountAutocomplete.test.ts` ; en fin de story : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit` (total recompté), `npm run build`.
- **E2E ciblés** (voie base, backend dans la prise ; port 3656, bases `kesh_e2e_586`) : `reconciliation-already-booked-reject.spec.ts`, `reconciliation-manual-entry.spec.ts`, `reconciliation-reject-unmatched.spec.ts` (AC14), `reconciliation-split.spec.ts` (AC15) et `reconciliation-group.spec.ts` (AC16 à AC18) ; noms vérifiés dans `frontend/tests/e2e/`.
- **E2E complet à la clôture** (dernier commit de code) : sous `LANG=fr_CH.UTF-8`, « vert » = les sept échecs de la KF-029 seuls ; ou sur idefix (`gate-distant.sh <branche> tout`, branche poussée).
- **Rust** : aucun fichier Rust ne bouge ; pas de gate Rust, pas de `gate-ci.sh`. Dire au Dev Agent Record « gate Rust non lancé : aucun fichier Rust modifié ».
- **Le story file n'affirme que ce qui a tourné.**

## Note de version

*Pour l'assemblage du CHANGELOG (jamais écrit dans la story).* Section « Interne » : « Tests de la réconciliation renforcés (dix assertions qui passaient aussi sur l'ancien code) ; une garde inatteignable retirée de la confirmation « déjà comptabilisée ». Aucun changement visible. »

## Choix consignés

- **C-58-6-k — les deux défauts jumeaux se corrigent dans la story, sans issue** (décision de l'orchestrateur, 2026-11-28 ; règle du `CLAUDE.md` : un défaut LOW du même module et du même genre se corrige dans la story). Six tâches sans issue (extension du 2026-11-28 après la validation P1, F4 : tous les sites du même module `reconciliation-*`) : `reconciliation-reject-unmatched.spec.ts:173` (`toContainText('1')`, même correctif que #1091 : AC14) `reconciliation-manual-entry.spec.ts:133`, `reconciliation-split.spec.ts:152` et `reconciliation-group.spec.ts:304`, `:394`, `:752` (`dispatchEvent('mousedown')`, même correctif que #1082 : AC13, AC15 à AC18). **Sites hors module, en issue** (textes dans `kesh-gate-logs/e65/58-6/issues-a-ouvrir.md`, à ouvrir par l'orchestrateur après vérification `--state all`) : `credit-card-statement.spec.ts:121` (même `dispatchEvent`) ; `reminders.spec.ts:178`, `dunning-roundtrip.spec.ts:89` et `reminder-batch-refus-pdf.spec.ts:57` (même `toContainText('1')` sur `reminder-selected-count`, une seule issue) — **angles morts assumés de la story**, non touchés. Les commits les rattachent à `refs #1091` et `refs #1082` ; aucune issue n'est ouverte. Elles passent par la même boucle de revue que les dix autres.

## Questions (pour l'orchestrateur ; aucune ne bloque)

1. **D2 (#1088)** : retrait avec extension de l'invariant au geste `group`. Si l'orchestrateur préfère **conserver** la garde, la story ne touche plus au code de production et #1088 se réduit à un commentaire ; cette alternative est écartée ici parce que le TypeScript exige une garde sur `current` tant que le `find` reste.
2. **D5 (#1090)** : le repli à 100 ms est une attente fixe ; elle n'est employée que si l'attente des animations ne suffit pas au rejeu.

## Dev Notes

- **Édition par Edit seulement.** Mutations : même règle ; chaque mutation est restaurée par `cp` depuis une copie ou par `git checkout -- <fichier>` **dans le worktree**, puis `touch` du fichier restauré, puis `git status --porcelain` pour s'assurer que rien ne reste muté.
- **`CounterpartyModalHost.test.svelte`** : ne pas passer par `rerender` (il rejoue la remise à zéro du dialogue). Pour ajouter une ligne à *Éclater* dans l'hôte, cliquer `split-add-line`.
- **Montage de l'espion (D6)** : `vi.mock` est hissé et vaut pour tout le fichier (il ne se borne pas à un bloc) ; le chemin est le même module que celui qu'importe le composant (`./reconciliation-counterparties`), l'original se récupère par `importOriginal` et s'étale (`{ ...original, keepOfferedCounterparty: vi.fn(original.keepOfferedCounterparty) }`) pour que les autres tests du fichier gardent le comportement réel.
- **`cliquerHors`** : `page.mouse.click` n'attend pas la stabilité de l'élément, contrairement à `locator.click` ; d'où l'attente explicite des animations (D5).
- **Fichiers communs avec la 58-5** : `ReconciliationProposals.svelte` (zones disjointes de la l. 509-523), `ReconciliationProposals.test.ts`, `AccountAutocomplete.svelte` et `AccountAutocomplete.test.ts` ; la 58-5 le déclare de son côté. Un conflit se résout en gardant les deux côtés.
- **Manuels** : aucun texte visible ne change : pas de `docs/manual/` à toucher. Le prompt de validation nommera le manuel quand même (règle du dépôt) pour que la passe le constate.
- **`docs/testing.md`** § *Les échecs attendus* : le rouge d'un E2E se juge fichier par fichier, jamais au nombre.

### Project Structure Notes

- `frontend/src/lib/features/reconciliation/` (`ManualMatchModal.test.ts`, `TransactionSplitModal.test.ts`, `ReconciliationProposals.test.ts`, `ReconciliationProposals.svelte`), `frontend/src/lib/features/journal-entries/AccountAutocomplete.test.ts`,
  `frontend/tests/e2e/reconciliation-manual-entry.spec.ts`, `frontend/tests/e2e/reconciliation-already-booked-reject.spec.ts`, `frontend/tests/e2e/reconciliation-reject-unmatched.spec.ts`. Aucun nouveau fichier de production.

### References

- Issues #1078, #1079, #1080, #1082, #1083, #1084 (Story 35-7, #1000), #1088, #1090, #1091, #1093 (Story 35-4, #1023).
- `_bmad-output/planning-artifacts/epic-65-nettoyage-suite.md` § *Story 58-6* ; `CLAUDE.md` § *Review Iteration Rule*, § *E2E (Playwright)*, § *Priorités des défauts* ; `docs/testing.md` ; `docs/agents-orchestration.md`.

## Dev Agent Record

### Agent Model Used

*(à remplir au développement)*

### Debug Log References

- `kesh-gate-logs/e65/58-6/mutation-1079-manual.log` : mutation de contrôle de la rédaction (1 rouge sur 17).

### Completion Notes List

*(à remplir)*

### File List

*(à remplir)*

## Change Log

- 2026-11-28 — Fiche rédigée (Sonnet 5.5), statut `drafted` ; validation à lancer par l'orchestrateur (alternance de modèles, première passe jamais à Haiku ; le prompt nomme le manuel et exige les axes exercés / non exercés).
- 2026-11-28 — Validation P1 (Opus) : 0/0/4/6, remédiée (F1 mutation AC3 Manual ; F2 mutation AC10 ; F3 mutation du vrai clic ; F4 sites jumeaux du module en AC15 à AC18, site hors module en issue ; F5 AC2 sans condition ; F6 AC8 et D5 ; F7 numéros de ligne ; F8 `vi.mock` de fichier ; F9 angle mort des rappels, une issue ; F10 AC12 et recette visuelle sans objet). Rapport : `kesh-gate-logs/e65/58-6/58-6-validation-p1.md`.
