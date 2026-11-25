# Story 49-5 : Solde de la dette de l'Epic 49 — une charge de ligne de facture typée par son type (#1464), un avoir dont le taux et le compte ne se collent plus (#1466)

Status: done — close le 2026-11-25 : gates verts (backend 6270/6270, frontend 4851, E2E sept KF-029 seuls), revue de code P1 (Opus 0/0/0/3) → ciblée Haiku (0/0/1/2) → ciblée Sonnet (0/0/0/2) ; validation : P1 (Sonnet, 0/1/2/9), P2 (Opus, 0/0/1/8), P3 ciblée (Sonnet, 0/0/1/4), P4 ciblée (Haiku, 0/0/0/3) ; **lot C (#1499) retiré** le 2026-11-24 sur décision de l'orchestrateur (signal de recyclage, Change Log), conception conservée dans `49-5-lot-c-reporte-1499.md` ; issues ouvertes : #1547 à #1553.

> ⚠️ **Plafond provisoire — à confirmer par le Project Lead.** Le kickoff de l'Epic 49 (2026-11-18) n'a pas fixé de plafond
> d'issues pour sa story de solde (`CLAUDE.md` § *Priorités des défauts*). Cette fiche prend **deux issues** sur le plafond de trois que le
> Project Lead a retenu pour l'Epic 53, sur consigne de l'orchestrateur du 2026-11-24 (C-e49s-1). Si le Project Lead fixe un autre
> plafond, la fiche se réduit (les lots sont indépendants) ou s'étend depuis la section *Issues écartées*. **Plafond utilisé : deux
> issues sur trois** depuis le retrait du lot C (#1499).

*Fiche rédigée le 2026-11-24 (Opus 5.5, `bmad-create-story`), worktree `/home/gcorbaz/devel/kesh-49-solde`, branche
`story/49-5-solde-dette-epic`, partie de `origin/epic/49-types-de-lignes-facture` à **`59fb28712`**. **Tout le code cité est lu à
`59fb28712`.** Release visée : **0.20.0**, vendredi 2026-11-27 (coupure le jeudi 2026-11-26 au soir) ; **la story doit être close
avant le vendredi 2026-11-27 au matin**. **Aucune migration, aucun changement du PDF, aucun changement de l'API.** Journaux :
`/home/gcorbaz/devel/kesh-gate-logs/e49solde/` (issues relues : `issues-candidates.json`, `issues-recherche.json` ; inventaires
`inv-1464.txt`, `inv-1466.txt` (et `inv-1499.txt`, du lot C reporté), régénérables par les commandes qu'ils portent en tête). Clé courte des bases :
`e49s` (`kesh_e49s`, `kesh_ci_e49s`, `kesh_e2e_e49s`). Registre des choix : `e0200-choix-autonomes-e49solde.md`.*

## Story

En tant que comptable qui émet des factures et des avoirs,
je veux que la charge d'une ligne de facture ne puisse pas, même en type, partir chiffrée sans montant, et que la fiche d'un avoir
sépare le taux de TVA du compte de produit,
afin qu'un futur appelant ne puisse pas fabriquer une ligne que seul le serveur refuserait, et que je ne lise pas « 8.103000 » là
où il y a « 8.10 » et « 3000 ».

## Périmètre : la dette des stories de l'Epic 49, plafonnée à trois issues (provisoire), deux utilisées

Story de solde (`CLAUDE.md` § *Priorités des défauts* : « seulement la dette des stories de l'epic, plafonnée par un nombre
d'issues fixé au kickoff »). Sixième story de l'épique (49-1a, 49-1b, 49-1c, 49-1d, 49-4a, 49-5) : la borne de six stories, solde
comprise, est tenue (`CLAUDE.md` § *Numérotation des versions*). Candidates : les issues ouvertes nées des stories de l'épique
(fiches 49-1a à 49-4a, `sprint-status.yaml`), relues le 2026-11-24 par deux requêtes GraphQL en lecture seule
(`kesh-gate-logs/e49solde/issues-candidates.json`, `issues-recherche.json`). Ordre de choix (consigne de l'orchestrateur) :
**ce qui touche le PDF des factures** (Guy émet des factures réelles), **puis les P3, puis le reste** ; issues petites, sûres,
sans migration, **au plus deux modules** ; aucune retenue sans constat confirmé au code.

| Issue | Labels | Origine | Retenue |
|---|---|---|---|
| **#1464** | `technical-debt`, P3 | revue de code P1 de la 49-1c, constat B-1 | **oui** (lot A) |
| #1499 | `technical-debt`, P3 | constatée pendant la 49-1d (code de la 33-10) | **retirée** le 2026-11-24 (ex-lot C, décision de l'orchestrateur : signal de recyclage) ; reste ouverte, conception dans `49-5-lot-c-reporte-1499.md` |
| **#1466** | `bug`, P4 | revue de code P1 de la 49-1c, constat A-4 (préexistant) | **oui** (lot B) — la seule P4 `bug` retenable (les autres P4 `bug` nées de l'épique — #1347, #1411, #1495 — sont écartées pour leur raison propre, § *Issues écartées*), sur une pièce qu'il émet |

**Lecture de « dette des stories de l'epic »** : les **constats des revues et du développement** des stories de l'épique, non le
seul code qu'elles ont écrit. #1464 vise du code de la 49-1c ; #1466 (`git blame` : `credit-notes/[id]/+page.svelte:198`,
2026-06-27 ; `:199`, 2026-08-04) vise du code **antérieur** à l'épique, constaté par ses revues (49-1c A-4). La lecture « code écrit par
l'épique » ne retiendrait que #1464 ; elle est écartée parce que le `CLAUDE.md` ne la demande pas et qu'elle laisserait sans
story des constats nés de l'épique.

Aucune issue du PDF n'est retenue : #1363 et #1400 appartiennent à la 49-4b (Epic 52) ; la seule candidate de dette, #1399, change le rendu de toute facture non figée (§ *Issues écartées*).

**Modules** (grain du `CLAUDE.md` § *La passe ciblée* : un crate ou un domaine du frontend dont le **code de production** change) :
**deux au plus** — `frontend` domaine `invoices` (lot A : `src/lib/features/invoices/` ; si D-A3 exigeait de toucher
`InvoiceForm.svelte`, il vit à `src/lib/components/invoices/`, même domaine, autre dossier) et la route
`src/routes/(app)/credit-notes/[id]/` (lot B). Le `CLAUDE.md` définit le domaine par `frontend/src/lib/features/<domaine>` et ne
tranche pas le cas d'une route : lue comme domaine `credit-notes`, on compte deux modules ; lue comme hors domaine, un seul. Le
décompte tient sous les deux lectures. Aucun crate Rust ne change. Règle de splitting (> 5 modules) : non atteinte.

Les deux lots sont **indépendants** : aucun ne lit le code de l'autre ; chacun se retire de la fiche sans toucher à l'autre.

## Ce qui se lit au code (`59fb28712`)

### Lot A — #1464 : `CreateInvoiceLineRequest` admet une ligne chiffrée sans montant

1. **Le type** — `frontend/src/lib/features/invoices/invoices.types.ts:361-387` :
   ```ts
   export interface CreateInvoiceLineRequest {
   	lineType: InvoiceLineType;            // :366
   	description: string;                   // :367
   	quantity?: string;                     // :373
   	unitPrice?: string;                    // :374
   	vatRate?: string;                      // :375
   	revenueAccountId?: number | null;      // :386
   }
   ```
   Le doc-comment `:368-372` dit déjà la règle que le type ne tient pas : « Quantité, prix unitaire et taux : exigés d'une ligne
   `priced`, **absents** d'une ligne de texte (`comment`, `heading`), dont la charge est `{ lineType, description }` seulement ».
   `{ lineType: 'priced', description: 'x' }` se compile : seul le serveur le refuserait (400). Constat de l'issue **confirmé**.
2. **`InvoiceLineType`** — `invoices.types.ts:23` : `export type InvoiceLineType = 'priced' | 'comment' | 'heading';`.
3. **Le seul constructeur** — `frontend/src/lib/features/invoices/invoice-lines.ts:84-96`, `ligneAEnvoyer` :
   ```ts
   export function ligneAEnvoyer(l: LigneFormulaire): CreateInvoiceLineRequest {
   	if (!estChiffree(l)) {
   		return { lineType: l.lineType, description: l.description };   // :86
   	}
   	return { lineType: 'priced', description: l.description, quantity: l.quantity,
   		unitPrice: l.unitPrice, vatRate: l.vatRate, revenueAccountId: l.revenueAccountId };
   }
   ```
   `LigneFormulaire.lineType` est `InvoiceLineType` (`invoice-lines.ts:32`), non nul.
4. **Les prédicats** — `frontend/src/lib/features/invoices/invoice-line-type.ts:18-20` (`estChiffree(l): boolean`) et `:31-33`
   (`estLigneDeTexte(t): boolean`, « le contraire exact » de `estChiffree`) ; « seule source de la règle pour le total, le
   formulaire et les deux fiches » (`:2-3`, Story 49-1c D6/D7). **Ni l'un ni l'autre n'est un prédicat de type** : dans
   `ligneAEnvoyer`, `!estChiffree(l)` ne rétrécit pas `l.lineType`, qui reste `InvoiceLineType` à la ligne `:86`. Avec une union
   discriminée, cette ligne ne se compilerait plus sans geste (D-A2).
5. **Les sites d'appel** (`inv-1464.txt`, ensemble clos) : `InvoiceForm.svelte:744` (`const payloadLines: CreateInvoiceLineRequest[]`),
   `:747` et `:756` (`payloadLines.push(ligneAEnvoyer(…))`), `:759` (`CreateInvoiceRequest`), `:768` (`UpdateInvoiceRequest`) ;
   `invoices.api.ts:55`, `:61` (signatures). **Aucun autre constructeur** : `git grep` de `CreateInvoiceLineRequest`,
   `CreateInvoiceRequest`, `UpdateInvoiceRequest` dans `frontend/src` et `frontend/tests` ne rend que ces sites ; les E2E postent
   du JSON non typé. Les champs `quantity?`, `unitPrice?`, `vatRate?` ne sont optionnels **que** dans ce type
   (`git grep -nE "(quantity|unitPrice|vatRate)\?:" -- frontend/src/lib/features` → `invoices.types.ts:373-375` seuls).
6. **Les prédicats dans les gabarits** : `estLigneDeTexte(...)` est appelé dans `InvoiceForm.svelte:1009`,
   `invoices/[id]/+page.svelte:1391`, `credit-notes/[id]/+page.svelte:182`, et `invoice-lines.ts:53`. Un prédicat de type y
   rétrécirait `line.lineType` dans chaque branche ; aucune de ces branches n'affecte `lineType` (relu), donc aucun effet attendu
   à `npm run check` — **à constater à l'étape 0**.
7. **La couverture typée** : `frontend/tsconfig.json` étend `.svelte-kit/tsconfig.json`, dont `include` porte `../src/**/*.ts` :
   `npm run check` (svelte-check) vérifie donc les fichiers `*.test.ts` de `src/`. Aucun `@ts-expect-error` n'existe encore dans
   `frontend/src` (`git grep -n '@ts-expect-error' -- frontend/src` vide) : la story en pose le premier usage, pour un test de type.

### Lot B — #1466 : sur la fiche d'un avoir, « TVA % » et « Compte de produit » se touchent

1. **Le tableau** — `frontend/src/routes/(app)/credit-notes/[id]/+page.svelte:166-216`. En-têtes `:169-177` : `py-2`, `py-2
   text-right` (Qté, Prix unitaire, **TVA %** `:172`), `py-2` (**Compte de produit** `:174-176`), `py-2 text-right` (Total HT
   `:177`). Ligne chiffrée `:191-203` : `:198` `<td class="py-2 text-right">{line.vatRate}</td>` puis `:199`
   `<td class="py-2" data-testid="credit-note-line-revenue-account">`. **Aucune marge horizontale** sur aucune cellule : une cellule
   alignée à droite suivie d'une cellule alignée à gauche n'a aucun blanc entre leurs contenus. Constat de l'issue **confirmé** au
   code et à la capture `kesh-gate-logs/e491c/captures/06-fiche-avoir.png` : l'en-tête se lit « TVA %Compte de produit », la cellule
   « **8.103000 — Ventes CI** » — le taux `8.10` et le numéro `3000` du compte se lisent comme **un seul nombre**. Plus qu'un défaut
   cosmétique : une lecture fausse du taux est possible (la sévérité P4 reste celle de l'issue).
2. **Le symétrique côté facture** — `frontend/src/routes/(app)/invoices/[id]/+page.svelte:1374-1410` : chaque `th` et `td` du tableau
   des lignes porte `pr-2` (`<th class="py-2 pr-2">TVA %</th>` `:1377`, `<td class="py-2 pr-2">{l.vatRate}%</td>` `:1406`) — treize
   occurrences de `pr-2` entre `:1368` et `:1415`. Le défaut n'existe pas sur la fiche de facture.
3. **Pied et commentaire** : `:186` (commentaire, `colspan={COLONNES_LIGNES}`), `:209` (`colspan="5"`, libellé « Total HT ») et
   `:210` (total). `COLONNES_LIGNES = 6` (`:26`).
4. **Le test existant** — `credit-note-display-page.test.ts:85-120` (49-1c, T9) asserte la structure : `colspan` 6, une cellule
   pour un commentaire, six pour une ligne chiffrée ; il ne lit aucune classe de marge.

**Inventaire par le symptôme** (« une cellule `th`/`td` alignée à droite immédiatement suivie, dans la même rangée, d'une cellule
qui ne l'est pas, aucune des deux ne portant de marge horizontale ni de `colspan` »). *Remédiation de V1-3 (validation P1)* : le
premier inventaire (`git grep` de `class="py-N"` sous `routes/(app)`) omettait toute autre classe et tout fichier hors des routes ;
il est remplacé par un balayage de **tous** les `.svelte` de `frontend/src`, quelles que soient les classes :

```
$ python3 -I /home/gcorbaz/devel/kesh-gate-logs/e49solde/outils/paires-droite-gauche.py frontend/src   (lecture seule ; sortie : inv-1466.txt)
routes/(app)/credit-notes/[id]/+page.svelte:172 -> :174         routes/(app)/credit-notes/[id]/+page.svelte:198 -> :199
routes/(app)/journal-entries/+page.svelte:635 -> :638           routes/(app)/audit-log/+page.svelte:364 -> :365
routes/(app)/audit-log/+page.svelte:387 -> :388                 routes/(app)/bank-import/[id]/+page.svelte:170 -> :171
routes/(app)/bank-import/[id]/+page.svelte:183 -> :184          lib/features/reconciliation/GroupSettlementModal.svelte:797 -> :798
lib/features/reconciliation/ReconciliationProposals.svelte:708 -> :709   lib/features/reconciliation/ReconciliationProposals.svelte:756 -> :757
lib/features/journal-entries/JournalEntryForm.svelte:864 -> :873          lib/features/bank-import/BankImportUpload.svelte:867 -> :868
lib/features/bank-import/BankImportUpload.svelte:875 -> :876              lib/features/bank-import/ManualStatementForm.svelte:424 -> :425
lib/components/invoices/InvoiceForm.svelte:1155 -> :1158
```

Ensemble clos des quinze paires rendues :

| Site | Paire droite → gauche sans marge | Classement |
|---|---|---|
| `routes/(app)/credit-notes/[id]/+page.svelte:172-174`, `:198-199` | TVA % → Compte de produit | **corrigé** (D-B1) |
| `routes/(app)/audit-log/+page.svelte:364-365`, `:387-388` | N° → Détails | **hors périmètre** (domaine `audit-log`) : issue P4 **I-2** |
| `routes/(app)/bank-import/[id]/+page.svelte:170-171`, `:183-184` | Montant → Référence (`{tx.amount} {tx.currency}` → `{tx.reference ?? '—'}`) | **hors périmètre** (domaine `bank-import`) : issue P4 **I-3** |
| `lib/features/bank-import/BankImportUpload.svelte:867-868`, `:875-876` | Montant → Détails | **hors périmètre** (domaine `bank-import`) : issue P4 **I-4** |
| `lib/features/reconciliation/ReconciliationProposals.svelte:708-709`, `:756-757` | Montant → Contrepartie | **hors périmètre** (domaine `reconciliation`) : issue P4 **I-5** |
| `lib/features/journal-entries/JournalEntryForm.svelte:864-873` | en-tête Crédit → Compte analytique (`JournalEntryForm.svelte:874`, `journal-entry-form-col-project`), colonne affichée seulement avec `showProjectColumn` ; les cellules du corps portent `pr-2`, `:884-951`) | **hors périmètre** (domaine `journal-entries`) : issue P4 **I-6**, en-tête seul, constat au code à confirmer à l'écran |
| `routes/(app)/journal-entries/+page.svelte:635-638`, `lib/features/reconciliation/GroupSettlementModal.svelte:797-798`, `lib/features/bank-import/ManualStatementForm.svelte:424-425`, `lib/components/invoices/InvoiceForm.svelte:1155-1158` | la seconde cellule est vide, ou ne porte qu'un texte `sr-only` | hors classe : rien à coller |

Chaque site hors périmètre va à **une issue P4 par site** (« une issue par problème »), ouverte par l'orchestrateur
(§ *Issues à ouvrir par l'orchestrateur*) : les corriger ici ajouterait quatre domaines et dépasserait le plafond de trois issues.
Les constats I-2 à I-7 sont faits **au code seul**, sans capture.

**Angle mort déclaré du balayage** (écrit aussi en tête du script) : classes calculées (`class={…}`, `class:` directive, `clsx`),
styles en ligne, tableaux faits de `<div>`, cellules séparées par un bloc `{#if}`/`{#each}` dont la branche ne commence pas par une
cellule, marges posées par une feuille de style globale ou un composant parent (aucune ne pose de marge aux cellules aujourd'hui :
`grep -nE "\b(td|th)\b" frontend/src/app.css` vide, relevé de la validation P1). Deux colonnes **alignées à gauche** et sans
marge peuvent aussi se toucher quand la table manque de largeur : ce n'est pas le constat de l'issue, la capture ne le montre pas —
**angle mort assumé**, hors classe.

### Manuels

Aucune phrase des manuels ne décrit la marge des colonnes de la fiche d'un avoir ni le type de la charge d'une ligne
(`docs/manual/*/`, `.tex` et PDF aplatis, vérifié par les validations P1 et P2, axe A8). Aucun document du dépôt ne change.
`docs/api-external.md` ne change pas : l'API ne change pas (le lot A ne touche que le typage du client web).

## Décisions

### Lot A — #1464

- **D-A1 — Union discriminée** dans `invoices.types.ts` :
  ```ts
  /** Les types d'une ligne de texte (sans montant). */
  export type InvoiceTextLineType = Exclude<InvoiceLineType, 'priced'>;
  export interface CreatePricedInvoiceLineRequest {
  	lineType: 'priced'; description: string;
  	quantity: string; unitPrice: string; vatRate: string;
  	revenueAccountId?: number | null;   // doc-comment de :376-385 conservé tel quel
  }
  export interface CreateTextInvoiceLineRequest { lineType: InvoiceTextLineType; description: string; }
  export type CreateInvoiceLineRequest = CreatePricedInvoiceLineRequest | CreateTextInvoiceLineRequest;
  ```
  Le nom `CreateInvoiceLineRequest` est gardé (ses quatre sites d'appel ne changent pas). `revenueAccountId` reste optionnel :
  son absence a un sens documenté (compte par défaut, `:377-385`). `lineType` reste **obligatoire** dans les deux variantes : l'API
  accepte son absence (contrat de la 0.19.0, doc-comment `:362-365`), mais l'écran l'envoie toujours ; le type décrit ce que
  **l'écran** envoie, non tout ce que l'API accepte (doc-comment à garder). *Écartée* : rendre `lineType` optionnel dans la variante
  chiffrée — le type perdrait sa discrimination et l'écran n'en a pas l'usage.
- **D-A2 — `estLigneDeTexte` devient un prédicat de type** (`invoice-line-type.ts:31`) :
  `export function estLigneDeTexte(t: InvoiceLineType | null | undefined): t is InvoiceTextLineType` ; corps inchangé
  (`return !estChiffree({ lineType: t });`). `ligneAEnvoyer` teste `estLigneDeTexte(l.lineType)` au lieu de `!estChiffree(l)` —
  même règle par définition (`:27-30`), et la branche de texte rend `{ lineType: l.lineType, description: l.description }` sans
  conversion. Le doc-comment de `estLigneDeTexte` dit la limite : une valeur hors de l'union (un type qu'un serveur plus récent
  ajouterait) est classée texte **et typée** `InvoiceTextLineType` — c'est déjà son comportement à l'exécution (elle part
  `{ lineType, description }`), le type ne fait que le dire. *Écartées* : un `as` dans `ligneAEnvoyer` (il ferait taire exactement
  le contrôle que la story pose) ; un `switch (l.lineType)` local (seconde source de la règle, contre D6 de la 49-1c et la règle
  DRY) ; faire de `estChiffree` un prédicat sur l'objet (le rétrécissement d'une propriété par un prédicat sur l'objet entier ne
  porte pas sur la branche fausse).
- **D-A3 — Aucun changement à l'exécution** : la charge envoyée est identique octet pour octet (les tests de
  `invoice-lines.test.ts:124-170` restent inchangés et verts) ; aucun changement de `InvoiceForm.svelte` n'est attendu. Si
  `npm run check` en exige un (rétrécissement dans un gabarit, § lot A, point 6), il est **minimal**, typé, sans `as`, et déclaré au
  Dev Agent Record.

### Lot B — #1466

- **D-B1 — La convention de la fiche de facture** : `pr-2` sur **chaque** `th` et `td` du tableau des lignes de la fiche d'un avoir
  (en-têtes `:169-177`, ligne chiffrée `:192-202`, cellule de commentaire `:186`, pied `:209-210`), comme
  l'en-tête et le corps de `invoices/[id]/+page.svelte:1374-1410`. **Le pied diffère de la facture** : celui de la facture
  (`:1426` et suivants, `<td colspan="5" class="py-1 text-right">`) n'a **pas** `pr-2`. Sur l'avoir, le pied le prend aussi, pour que
  « Total HT » reste aligné sur sa colonne (la dernière cellule du corps porte `pr-2`) : ne pas copier le pied de la facture ; T3
  l'asserte. *Écartées* : `pr-2` sur la seule cellule du taux (la marge naîtrait d'une exception, et la
  prochaine paire droite → gauche aurait le même défaut) ; `pl-*` sur la cellule du compte (même raison) ; ajouter `%` au taux
  comme sur la facture (`{l.vatRate}%`) — c'est un changement de contenu, hors du constat ; et sans marge, « 8.10%3000 » resterait
  collé.
- **D-B2 — Rien d'autre ne change** : ni textes, ni clés i18n, ni `data-testid`, ni structure (`colspan`, nombre de cellules).

## Critères d'acceptation

### Lot A — #1464

- **AC-A1** — `invoices.types.ts` exporte `InvoiceTextLineType`, `CreatePricedInvoiceLineRequest`, `CreateTextInvoiceLineRequest`
  et `CreateInvoiceLineRequest` = leur union (D-A1) ; `quantity`, `unitPrice`, `vatRate` sont **obligatoires** dans la variante
  `priced` et **absents** de la variante de texte. Constat : lecture du fichier, et `git grep -nE "(quantity|unitPrice|vatRate)\?:"
  -- frontend/src/lib/features` **vide**.
- **AC-A2** — Un test de type (T1, `invoice-lines.test.ts` ou un fichier voisin `invoice-line-request.types.test.ts`) porte, chacune
  sous `// @ts-expect-error` : (a) une ligne `priced` sans `quantity` ; (b) sans `unitPrice` ; (c) sans `vatRate` ; (d) une ligne
  `comment` avec `quantity` ; (e) une ligne `heading` avec `vatRate` ; et **sans** directive : (f) une ligne `priced` complète ;
  (g) `{ lineType: 'comment', description }`. **Chaque cas tient sur une ligne** (un littéral d'une ligne affecté à une constante
  typée `CreateInvoiceLineRequest`), la directive sur la ligne qui précède : une propriété manquante rougit à la déclaration, une
  propriété en trop à la ligne de la propriété ; sur une ligne, les deux coïncident. Les cas sont **enveloppés dans un `it(…)`**
  avec un `expect` (vitest exécute aussi le fichier, et un fichier `*.test.ts` sans test échoue : « No test suite found »). Le test
  n'importe que `CreateInvoiceLineRequest` (nom déjà existant), pour que T1 se compile sur le code de `59fb28712`.
  `npm run check` : **0 erreur** ; `npm run test:unit` : vert.
- **AC-A3** — `estLigneDeTexte` est un prédicat de type (D-A2) ; `ligneAEnvoyer` n'a **aucun** `as` (`git grep -n ' as ' --
  frontend/src/lib/features/invoices/invoice-lines.ts` sans nouvelle occurrence) ; les tests `invoice-lines.test.ts:124-170`
  passent **inchangés** (`git diff` vide sur ces lignes).
- **AC-A4** — Aucune charge envoyée ne change : `npm run test:unit` vert, dont `InvoiceForm.test.ts` sans modification de ses
  assertions de charge.

### Lot B — #1466

- **AC-B1** — Chaque `th` et `td` du tableau des lignes de `credit-notes/[id]/+page.svelte` porte `pr-2` (D-B1) ; T3 l'asserte sur
  l'en-tête, une ligne chiffrée, une ligne de commentaire et le pied, rendus par `render(Page)`.
- **AC-B2** — La structure est inchangée : le test T9 de la 49-1c (`credit-note-display-page.test.ts:97-120`) passe **inchangé**.
- **AC-B3** — **Recette visuelle** (thème clair, base de démonstration, largeur 1280 px) : la fiche d'un avoir portant au moins
  une ligne chiffrée à 8.10 % sur le compte 3000 et un commentaire, **avant** (sur `59fb28712`) et **après** ; l'après montre
  « TVA % » séparé de « Compte de produit » et « 8.10 » séparé de « 3000 — … ». Captures dans
  `kesh-gate-logs/e49solde/captures/`, jointes au Dev Agent Record et au prompt de la revue de code.

### Transverses

- **AC-T1** — Aucun fichier de `crates/`, de `crates/kesh-db/migrations/`, des catalogues i18n, des manuels (`docs/manual/`) ni du
  PDF ne change (`git diff --stat origin/epic/49-types-de-lignes-facture...HEAD` le montre) ; aucune facture, aucun avoir, aucune
  écriture, aucun PDF ne change.
- **AC-T2** — Gates verts au dernier commit de code (§ *Gates*) ; E2E complet : seulement les échecs attendus (`docs/testing.md`
  § *Les échecs attendus*, sept de la KF-029), jugés fichier par fichier.

## Tests à écrire, et la preuve du rouge

| Test | Lot | Où | Rouge prouvé par |
|---|---|---|---|
| **T1** — test de type, cas (a) à (g) d'AC-A2 | A | `frontend/src/lib/features/invoices/` | **écrit avant le code** : `npm run check` sur le type de `59fb28712` rend « Unused '@ts-expect-error' directive » pour (a) à (e) — journal `t1-rouge.log` ; puis vert après D-A1 |
| **M1** — mutation : `quantity?: string` dans la variante `priced` | A | `invoices.types.ts` | `npm run check` rouge sur (a) ; journal `m1.log` ; mutation défaite (et `touch` du fichier) |
| **M2** — mutation : `estLigneDeTexte` rendu à `: boolean` | A | `invoice-line-type.ts` | `npm run check` rouge sur `invoice-lines.ts` (branche de texte) — prouve que le rétrécissement est porté par le prédicat ; journal `m2.log` |
| **T2** — tests existants de `ligneAEnvoyer` | A | `invoice-lines.test.ts:124-170` | filet : inchangés et verts (pas de rouge à prouver) |
| **T3** — `pr-2` sur chaque cellule (AC-B1) | B | `credit-note-display-page.test.ts` | **écrit avant le code** : rouge sur `59fb28712` (aucune cellule ne porte `pr-2`) — journal `t3-rouge.log` |
| **M3** — mutation : retirer `pr-2` de la seule cellule du taux (`:198`) | B | `+page.svelte` | T3 rouge ; journal `m3.log` |
| **T4** — captures avant / après (AC-B3) | B | `kesh-gate-logs/e49solde/captures/` | constat visuel ; un test vérifie qu'une classe existe, non qu'on voit l'écart (`CLAUDE.md` § *Recette visuelle*) |

## Tâches

- [x] **Étape 0** — Bases `kesh_e49s`, `kesh_ci_e49s`, `kesh_e2e_e49s` (consignes `docs/agents-orchestration.md` § 4) ; relire les
      `chemin:ligne` de la fiche à HEAD ; lot A, point 6 (prédicat dans les gabarits). Commit et push de la branche `story/…`.
- [x] **Lot A** — T1 (rouge, journal) ; D-A1 ; D-A2 ; T1 vert ; M1, M2 ; doc-comments (`invoices.types.ts`, `invoice-line-type.ts`).
      Commit `refs #1464`.
- [x] **Lot B** — captures **avant** (T4) ; T3 (rouge, journal) ; D-B1 ; T3 vert ; M3 ; captures **après**. Commit `refs #1466`.
- [x] **Propagation** — `git grep` des symptômes corrigés (`quantity?:`, `py-2 text-right">{line.vatRate}`) sur tout
      le dépôt ; tout site neuf traité ou écrit.
- [x] **Gates** (§ *Gates*), Dev Agent Record, Change Log ; statut `review`. Chiffres réels : frontend `check` 0 erreur, 282 fichiers / 4851 tests, lint-i18n et build verts (à `2c773dec7`) ; backend `test-fast --ci` 6270/6270, 17 ignorés (à `2c773dec7`, + un commentaire seul à `17126a445`) ; E2E complet à `17126a445` (496 passés, 18 ignorés, 7 échecs = les sept KF-029, catalogues chargés depuis `/home/gcorbaz/devel/kesh-49-solde/crates/kesh-i18n/locales`) ; le premier E2E, invalide (backend lancé hors du worktree), est consigné au Dev Agent Record.

## Gates

- Frontend (`CLAUDE.md` § *Frontend*) : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`, sous
  `scripts/mem-guard.sh`.
- Backend (aucun crate ne change, mais la CI lance son job `backend` quel que soit le fichier : même gate en local, `CLAUDE.md` § *Test Locally First*) : `scripts/test-fast.sh` puis
  `scripts/gate-ci.sh kesh_ci_e49s`, sous le verrou `flock <kesh-gate-logs>/.gate.lock`, base remise à zéro selon
  `docs/agents-orchestration.md` (bases propres, jamais de redémarrage du conteneur partagé).
- E2E complet au dernier commit de code (`docs/testing.md` § *Prérequis Playwright local*, à la lettre) ; seulement les échecs
  attendus.
- Le Dev Agent Record n'affirme que ce qui a tourné, chiffres recomptés depuis les journaux.

## Note de version

- Avoirs : sur la fiche d'un avoir, le taux de TVA et le compte de produit ne se touchent plus — « 8.10 » et « 3000 — Ventes » se
  lisaient « 8.103000 — Ventes » (refs #1466).
- Interne : la charge d'une ligne de facture envoyée par l'écran est typée par son type — une ligne chiffrée ne peut plus, même en
  type, partir sans quantité, prix ou taux (refs #1464).

## Issues écartées

Relues le 2026-11-24 (`issues-candidates.json`, `issues-recherche.json`). Une ligne chacune ; « au code » quand la prémisse a été
lue, sinon la raison suffit à écarter.

| Issue | Labels | Raison |
|---|---|---|
| #1399 | `bug`, P4 — **PDF** | Confirmée au code (`crates/kesh-qrbill/src/pdf.rs:585` `let col_tot = left + 160.0;` = 180 mm, montants posés **à gauche** en `Mm(col_tot)` — total de ligne `:697`, récapitulatif `:715`, total TTC `:735` —, filets jusqu'à `PAGE_W - 20.0` = 190 mm `:624`, `:706`). **Écartée** : le correctif change le PDF de toute facture non encore figée, contre la contrainte 3 du fichier d'épique (« une facture émise avant la 0.20.0 doit se réimprimer à l'identique » ; seule dérogation admise : la 49-4b) ; il ferait rougir la recette de montée de la release (49-1d : comparaison des PDF des pièces non figées) ; même remède que #1423 (montants alignés à gauche), rangée « Plus tard ». Décision du Project Lead. |
| #1465 | `bug`, P3 | Confirmée au code (`InvoiceForm.svelte:1009-1024`, libellé « Commentaire — ligne N » pour un `heading` ; `crates/kesh-api/src/routes/invoices.rs:626` « le type de ligne {line_type} n'est pas encore disponible »). **Écartée** : son remède est un choix d'écran que la 49-3 (Epic 52) tranche en ouvrant `heading` ; inatteignable par l'API ; demanderait des clés i18n dans quatre langues et une recette visuelle d'un état que la 49-3 défera. **Proposition** : la rattacher à la 49-3. |
| #1503 | `technical-debt`, P3 | Le script visé, `kesh-gate-logs/rel0200/perte-assumee.sh`, est **hors dépôt** : aucune story ne le livre. À l'orchestrateur. |
| #1506 | `technical-debt`, P3 | Dette documentée (MEDIUM de la revue P3 de la 49-1d), propriétaire l'orchestrateur, remédiation planifiée au milestone v0.21.0 ; garde `TABLES` ↔ schéma et scénarios par lecture : pas petite. |
| #1444, #1445, #1446, #1447 | `technical-debt`, P3 | Rangées « Plus tard » par le Project Lead ; chacune est un chantier de la recette de montée, pas une petite correction. |
| #1448 | `documentation`, P3 | Rangée « Plus tard » par le Project Lead (section « Passer à la 0.19.0 » du manuel d'administration). |
| #1411 | `bug`, P4 | Rangée « Plus tard » ; constat non reproduit, à reproduire d'abord (corps de l'issue). |
| #1347 | `bug`, P4 | Pagination de `kesh_facture` (MCP) : un choix de conception, pas une petite correction. |
| #1494 | `technical-debt`, P4 | Déplacer le contrôle de capacité dans la transaction de validation : touche le chemin de validation (`kesh-api`), ni petit ni sûr à trois jours de la release. |
| #1495 | `bug`, P4 | Choix déclaré en D3 de la 49-4a ; disparaît avec la 49-4b (#1400). |
| #1497 | `technical-debt`, P4 | Attend la 49-3 (lignes `Heading` à comparer au rendu). |
| #1498 | `technical-debt`, P4 | Attend la 49-4b (Epic 52). |
| #1496 | `technical-debt`, P4 | Test E2E seul ; P4, sous le plafond. Candidate pour une prochaine solde. |
| #1402, #1403 | `technical-debt`, P4 | Tests de `configuration_transmise` seuls ; P4, sous le plafond. Candidates pour une prochaine solde. |
| #1500 | `technical-debt`, P4 | Installer `shellcheck` sur la station et en CI : hors du périmètre d'une story (installation interdite aux agents). |
| #1502 | `technical-debt`, P4 | Le juge des mutations vit **hors dépôt** (`kesh-gate-logs/e491d/mut/`). |
| #1504 | `technical-debt`, P4 | Déjà corrigée, **hors dépôt** : `/home/gcorbaz/devel/kesh-gate-logs/rel0200/verifs-0200.sh:15` (`succes=$(SQL "SELECT success FROM _sqlx_migrations WHERE version = 20261118000002")`) ; le commit `d2416c596` que cite le commentaire de l'issue ne touche pas ce fichier (`git show --stat` : `scripts/recette-montee.sh` et son test seulement). |
| #1499 | `technical-debt`, P3 | **Retirée** le 2026-11-24 (ex-lot C) : signal de recyclage de la règle de splitting à la passe ciblée P3 (V3-1 né du correctif de V2-2, né de celui de V1-1/V1-7, sévérité égale) ; décision de l'orchestrateur, à déclarer au Project Lead. Reste ouverte, à traiter dans une story à elle ; conception dans `49-5-lot-c-reporte-1499.md`. |
| #1363 | `bug`, P3 — **PDF** | Désignation tronquée à 44 caractères : portée par la 49-4b (Epic 52), milestone v0.22.0 ; pas de la dette des revues de l'épique. |
| #1330, #1374, #1364, #1400, #1401, #1423 | — | Hors classe : CR, issues portées par les stories elles-mêmes, ou rangées à d'autres milestones (v0.20.0, v0.21.0, v0.22.0, « Plus tard ») ; pas de la dette des revues de l'épique. |


## Issues à ouvrir par l'orchestrateur

Sept issues, **une par problème**, **ouvertes le 2026-11-24** (I-1 = #1547, I-2 à I-7 = #1548 à #1553 ; texte ci-dessous = texte prévu), à ouvrir **avant le développement** (I-1, avec le nouveau corps de #1499) ou à la clôture (I-2 à
I-7), au rythme des limites GitHub (cinq secondes au moins entre deux créations). Texte prêt :

**I-1 = #1547** — `technical-debt`, P3 — *recette-montee.test.sh : un refus du script sous test arrête le test sous `set -e` avant son
résumé (`D=$(grep -oE 'dossier .*$' …)`)*
> Dans `scripts/tests/recette-montee.test.sh`, quatre sites lisent le dossier de la recette par
> `D=$(/usr/bin/grep -oE 'dossier .*$' "$FX/sortie" | tail -1 | sed 's/^dossier //')` (`:589`, `:755`, `:858`, `:875` à `59fb28712`).
> Quand `scripts/recette-montee.sh` refuse avant d'écrire « dossier », `grep` rend 1, `pipefail` le propage et `set -e` (`:11`)
> arrête le test **sans résumé** : l'enveloppe Rust (`crates/kesh-db/tests/recette_montee_script.rs`) ne dit alors que « le test
> shell n'a pas rendu son résumé », sans la cause. Établi par la Story 49-1d (fiche `:944`, onze journaux tronqués d'une campagne de
> mutations). La collision de ports qui provoquait ce refus est suivie par #1499 ; la fragilité demeure pour toute autre cause de
> refus, et ne dépend pas de la correction de #1499. **Remède** : lire le dossier sans arrêt du test (`… || true`, ou lecture sans tube) et faire échouer
> l'assertion qui suit, avec la sortie du script. Origine : validation P1 de la fiche 49-5 (V1-2), refs #1499.

**I-2 = #1548** — `bug`, `triage`, P4 — *Journal d'audit : « N° » et « Détails » collés (marge droite absente)*
> `frontend/src/routes/(app)/audit-log/+page.svelte:364-365` (en-têtes) et `:387-388` (cellules) : la cellule du numéro d'entité,
> alignée à droite (`class="py-2 text-right"`), est suivie de la cellule « Détails », alignée à gauche, sans marge horizontale ;
> leurs contenus se touchent. Même défaut que #1466 (fiche d'un avoir), corrigé là par `pr-2` sur chaque cellule (convention de la
> fiche de facture). Constat au code seul (balayage de la validation P1 de la Story 49-5), sans capture. Cosmétique.

**I-3 = #1549** — `bug`, `triage`, P4 — *Détail d'un import bancaire : « Montant » et « Référence » collés*
> `frontend/src/routes/(app)/bank-import/[id]/+page.svelte:170-171` (en-têtes `class="text-right"` → `class="text-left"`) et
> `:183-184` (`{tx.amount} {tx.currency}` → `{tx.reference ?? '—'}`) : aucune marge horizontale entre la colonne du montant,
> alignée à droite, et celle de la référence. Même défaut que #1466. Constat au code seul (Story 49-5), sans capture. Cosmétique.

**I-4 = #1550** — `bug`, `triage`, P4 — *Import de relevé (aperçu) : « Montant » et « Détails » collés*
> `frontend/src/lib/features/bank-import/BankImportUpload.svelte:867-868` (en-têtes) et `:875-876` (cellules) : la colonne du
> montant, alignée à droite, est suivie de « Détails », alignée à gauche, sans marge horizontale. Même défaut que #1466. Constat au
> code seul (Story 49-5), sans capture. Cosmétique.

**I-5 = #1551** — `bug`, `triage`, P4 — *Propositions de rapprochement : « Montant » et « Contrepartie » collés*
> `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte:708-709` (en-têtes) et `:756-757` (cellules,
> `{p.transaction.amount} {p.transaction.currency}` → `{p.transaction.counterpartyName ?? ''}`) : aucune marge horizontale entre
> le montant, aligné à droite, et la contrepartie. Même défaut que #1466. Constat au code seul (Story 49-5), sans capture.
> Cosmétique.

**I-6 = #1552** — `bug`, `triage`, P4 — *Saisie d'une écriture : en-têtes « Crédit » et « Compte analytique » collés quand la colonne
analytique est affichée*
> `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte:864-873` : l'en-tête « Crédit » (`text-right`, sans marge) est
> suivi de l'en-tête « Compte analytique » (`:874`, clé `journal-entry-form-col-project` ; `text-left`, sans marge) quand
> `showProjectColumn` est vrai ; les cellules du corps portent `pr-2`
> (`:884-951`), pas les en-têtes. Constat au code seul (Story 49-5), à confirmer à l'écran avant correction. Cosmétique.

**I-7 = #1553** — `bug`, `triage`, P4 — *Fiche d'une facture : les montants du pied sont décalés de 8 px vers la droite par rapport aux totaux
de ligne*
> `frontend/src/routes/(app)/invoices/[id]/+page.svelte` : le total de chaque ligne porte `pr-2` (`:1410`,
> `<td class="py-2 pr-2 text-right font-mono">`), mais pas les montants du pied (`:1429` sous-total HT, `:1438` TVA par taux,
> `:1448` arrondi, `:1457` et `:1466`, `class="py-1 text-right font-mono"` / `class="py-3 text-right …"`) : les montants du pied
> ne s'alignent pas sur la colonne des totaux de ligne. Relevé par la validation P2 de la fiche 49-5 (la fiche d'un avoir, corrigée
> par #1466, met `pr-2` aussi sur son pied). Constat au code seul, sans capture. Cosmétique.

**Nouveau corps de #1499 — posé** (avec son nouveau titre, le 2026-11-25 à l'ouverture d'I-1 = #1547 ; le titre devient
*« recette-montee.test.sh : deux exécutions simultanées prennent les mêmes ports »*) :
> Deux exécutions simultanées de `scripts/tests/recette-montee.test.sh` prennent les mêmes ports : `port_libre`
> (`:388-395` à `59fb28712`) constate qu'un port n'est pas à l'écoute mais ne le **réserve** pas, si bien que deux copies lancées
> ensemble choisissent `PA=23310` et `PB=23410` ; l'une voit alors le port de l'autre (écoute du crochet sur `PB + 1`, `:864-873`)
> et le script sous test refuse (`scripts/recette-montee.sh:153-157`). Code de la Story 33-10, constaté pendant la Story 49-1d.
> **Reportée de la Story 49-5** : le remède (réservation de chaque port par un verrou `flock` pris dans le shell principal) et ses
> tests sont conçus dans `_bmad-output/implementation-artifacts/49-5-lot-c-reporte-1499.md` (branche `story/49-5-solde-dette-epic` jusqu'à sa fusion, puis branche de l'Epic 49), avec les
> constats encore ouverts ; une story à elle reprendra de là.
> **Hors de cette issue** : l'arrêt du test sous `set -e` avant son résumé quand le script sous test refuse (les quatre
> `D=$(grep -oE 'dossier .*$' …)`) est suivi par I-1 (#1547), qu'il vienne de cette collision ou d'une autre cause.
> Origine : story 49-1d (#1242) ; scission décidée à la validation de la fiche 49-5 ; retrait de la 49-5 le 2026-11-24 (signal de
> recyclage de la règle de splitting).

## Dev Notes

- **Factures réelles** : aucun des deux lots ne touche le serveur, le PDF, les données ni l'API (AC-T1). Le lot B ne change que
  des classes CSS d'une page de consultation.
- **Ordre** : les lots sont indépendants.
- **Lot A** : le test de type vit dans un `*.test.ts` sous `src/` (couvert par `npm run check`, § lot A, point 7) ; vitest
  l'exécute aussi : les cas sont dans un `it(…)` avec un `expect` (AC-A2), chacun sur une ligne, affecté à une constante typée.
- **Mutations** : par l'outil `Edit`, défaites par `git checkout -- <fichier>` puis `touch` (mémoire « mutation restaurée, binaire
  périmé »).
- **Estimation** : lot A 1 h, lot B 1 h (captures comprises), gates et E2E 1 h 30, soit 3 h 30 ; revue
  de code une à deux passes. Close le mercredi 2026-11-25 au soir si la validation finit le 2026-11-25 au matin.

## Dev Agent Record

### Agent Model Used

Sonnet 5.5 (`bmad-dev-story`), worktree `/home/gcorbaz/devel/kesh-49-solde`. Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e49solde/`.

### Debug Log References

- T1 rouge : `t1-rouge.log` (cinq « Unused '@ts-expect-error' directive », lignes 13, 15, 17, 19, 21 = cas (a) à (e)).
- Vert après D-A1/D-A2 : `a-check1.log` (`npm run check` : 0 erreur).
- M1 (`quantity?: string` dans la variante `priced`) : `m1.log` — 1 erreur, cas (a) (`:13`) ; restaurée par `git checkout --` puis `touch`.
- M2 (`estLigneDeTexte` rendu à `: boolean`) : `m2.log` — 1 erreur, `invoice-lines.ts:86` (`lineType: InvoiceLineType` non assignable à `CreateTextInvoiceLineRequest`) ; restaurée puis `touch`.
- T3 rouge : `t3-rouge.log` (`credit-note-display-page.test.ts`, 1 échec sur 5 : six `th` sans `pr-2`) ; vert : `t3-vert.log` (5/5).
- M3 (retirer `pr-2` de la seule cellule du taux) : `m3.log` — 1 échec, `<td class="py-2 text-right">8.10</td>` ; restaurée puis `touch`.
- Captures (recette visuelle, AC-B3), thème clair, 1280 px, base `kesh_e49s`, avoir portant deux lignes chiffrées à 8.10 % sur 3000 et un commentaire : `captures/avoir-avant.png`, `avoir-avant-tableau.png` (build de `59fb28712` + lot A, page inchangée : « TVA %Compte de produit », « 8.103000 — Ventes CI »), `avoir-apres.png`, `avoir-apres-tableau.png` (« TVA % » séparé, « 8.10 » séparé de « 3000 — Ventes CI » ; total et commentaire alignés comme avant). Outil : `outils/capture.cjs`.
- Gate frontend au dernier commit de code (`2c773dec7`) : `front-check.log` (0 erreur, 29 avertissements préexistants), `front-lint-i18n-ownership.log` (PASS), `front-test-unit.log` (282 fichiers, 4851 tests, tous passés), `front-build.log` (ok).
- Gate backend (`scripts/test-fast.sh --ci`, base `kesh_ci_e49s`, au code de `2c773dec7` ; le seul commit de code suivant, `17126a445`, ne change qu'un commentaire de `invoices.types.ts`) : **vert**, 6270 tests passés sur 6270, 17 ignorés, 730 s (`gate-backend.log`).
- **Premier E2E complet : INVALIDE, sans valeur** (`invalide-e2e.log`, `invalide-e2e-backend.log`). `e2e-interne.sh` lançait `kesh-api` sans `cd` dans le worktree : le backend chargea les catalogues par le chemin relatif `crates/kesh-i18n/locales` depuis le dépôt principal, sans les clés neuves de l'Epic 49. 8 échecs : les sept KF-029 plus `invoice-validation-pdf.spec.ts:53` (clé brute `error-invoice-lines-do-not-fit-pdf`). Défaut du montage de l'agent, non du code.
- **Second E2E complet** (base `kesh_e2e_e49s`, `LANG=fr_CH.UTF-8`, tête `17126a445`, `cd` dans le worktree et `KESH_LOCALES_DIR` absolu, ligne « locales chargées » relevée dans `e2e-locales.txt`) : journaux `e2e.log`, `e2e-backend.log` ; **verdict : 496 passés, 18 ignorés, 7 échecs**, exactement les sept KF-029 (#97), jugés fichier par fichier : `mode-expert.spec.ts:26` et `:41`, `onboarding-path-b.spec.ts:65` et `:92`, `onboarding.spec.ts:57`, `:77` et `:150` ; aucun autre échec ; catalogues chargés depuis `/home/gcorbaz/devel/kesh-49-solde/crates/kesh-i18n/locales` (`e2e-locales.txt`) ; `e2e.log`, durée 22,5 min.


### Completion Notes List

- Lot A (#1464) : union discriminée `CreatePricedInvoiceLineRequest | CreateTextInvoiceLineRequest` ; `estLigneDeTexte` prédicat de type ; `ligneAEnvoyer` teste `estLigneDeTexte(l.lineType)`, sans `as`. **D-A3 : aucun changement de gabarit ni de `InvoiceForm.svelte`** (`npm run check` : 0 erreur, relevé au point 6 de la fiche). `invoice-lines.test.ts` non modifié.
- Lot B (#1466) : `pr-2` sur chaque `th` et `td` du tableau (en-tête, ligne chiffrée, commentaire, pied), 15 cellules ; rien d'autre ne change (D-B2).
- Propagation : `git grep -nE "(quantity|unitPrice|vatRate)\?:" -- frontend/src/lib/features` vide ; `py-2 text-right">{line.vatRate}` ne subsiste que dans la fiche (citations) ; `supplier-invoices/[id]/+page.svelte:1054` (`{line.vatRate}%`) n'est pas dans l'inventaire des quinze paires.
- AC-T1 : `git diff --stat origin/epic/49-types-de-lignes-facture...HEAD` ne montre, hors `_bmad-output/`, que six fichiers sous `frontend/src`.

### File List

- `frontend/src/lib/features/invoices/invoices.types.ts`
- `frontend/src/lib/features/invoices/invoice-line-type.ts`
- `frontend/src/lib/features/invoices/invoice-lines.ts`
- `frontend/src/lib/features/invoices/invoice-line-request.types.test.ts` (neuf)
- `frontend/src/routes/(app)/credit-notes/[id]/+page.svelte`
- `frontend/src/routes/(app)/credit-notes/[id]/credit-note-display-page.test.ts`
- `_bmad-output/implementation-artifacts/49-5-solde-dette-epic-review-prompt-p1.md` (neuf), fiche, `sprint-status.yaml`

## Change Log

| Date | Entrée |
|---|---|
| 2026-11-24 | **Fiche écrite** (Opus 5.5, `bmad-create-story`), base `59fb28712`. Plafond **provisoire** de trois issues, à confirmer par le Project Lead (C-e49s-1). Retenues : #1464 (P3), #1499 (P3), #1466 (P4) ; #1399 (PDF) et #1465 (P3) écartées au code (C-e49s-2, C-e49s-3). Deux modules de production (`invoices`, `credit-notes`), aucune migration. Statut `drafted` ; validation P1 (Sonnet) à lancer. |
| 2026-11-24 | **Validation P1 remédiée** (Opus 5.5). **Passe P1** : Sonnet 5.5, complète, contexte frais (`kesh-gate-logs/e49solde/validation-p1.md`) : **0/1/2/9** (CRITICAL/HIGH/MEDIUM/LOW), onze axes déclarés, tous exercés (A6 par raisonnement : aucune exécution). Prémisses relues au code avant remède. **V1-1 (HIGH)** : `PA=$(port_libre …)` (`recette-montee.test.sh:394-395`) tient le verrou dans un sous-shell — D-C1 réécrit : appel direct, résultat dans `PORT_RESERVE`, sites `:394`, `:395` nommés, `flock -n` gardé par `if` sous `set -e` ; T5 (i) rougit si le verrou est perdu au retour, mutation M5 neuve. **V1-2 (MEDIUM)** : « mécanisme non établi » retiré, cité à la 49-1d (`:944`, `:1496-1502`) ; les quatre `D=$(grep -oE …)` (`:589`, `:755`, `:858`, `:875`) restent hors périmètre (plafond de trois issues) → issue I-1, et #1499 ne couvre que la collision de ports (corps précisé à sa fermeture). **V1-3 (MEDIUM)** : inventaire du lot B refait par un balayage de tous les `.svelte` (`kesh-gate-logs/e49solde/outils/paires-droite-gauche.py`, lecture seule) : quinze paires, ensemble clos classé ; hors module → une issue P4 chacune (I-2 audit-log, I-3 bank-import/[id], I-4 BankImportUpload, I-5 ReconciliationProposals, et I-6 JournalEntryForm, que le balayage a ajouté) ; angle mort du balayage écrit. **LOW** : V1-4 (chemin de `InvoiceForm.svelte`, lecture du grain « module » sous les deux sens, `ecouter` `:473-485`) ; V1-5 (exposition réelle : campagnes de mutations et lancements hors verrou, non les gates, sérialisés par `.gate.lock`) ; V1-6 (AC-C3 : une seule invocation sous `.gate.lock` et `mem-guard`, preuve du recouvrement par horodatages) ; V1-7 (`PB + 1` contrôlé à `ss`, verrous dans un dossier par utilisateur, fichiers non retirés, plage distincte pour T5) ; V1-8 (cas de type sur une ligne, dans un `it`) ; V1-9 (le pied de la facture n'a pas `pr-2` : écrit dans D-B1) ; V1-10 (`docs/ci.md:101` : `flock` et la valeur périmée « 99 à 104 s » corrigés dans le lot C) ; V1-11 (« seule P4 `bug` retenable », lecture de « dette des stories », #1504 cité à `verifs-0200.sh:15`). Section *Issues à ouvrir par l'orchestrateur* ajoutée (I-1 à I-6, texte prêt). Remédiation textuelle seule (fiche) ; validation P2 (Opus, complète) à lancer, prompt `49-5-solde-dette-epic-validation-prompt-p2.md`. |
| 2026-11-24 | **Validation P2 remédiée** (Opus 5.5). **Passe P2** : Opus 5.5, complète, contexte frais (`kesh-gate-logs/e49solde/validation-p2.md`) : **0/0/1/8**, treize axes déclarés, tous exercés (A6 par raisonnement et sondes sans écriture). Prémisses relues au code avant remède (`docs/agents-orchestration.md:197`, `recette-montee.sh:86`, `:96`, `JournalEntryForm.svelte:874`, `invoices/[id]/+page.svelte:1410`, `:1429`, `:1438`, `:1448`, `:1457`, `:1466`). **V2-1 (MEDIUM, né de la remédiation de V1-1/V1-7 ; sévérité inférieure au HIGH de la P1 : signal de splitting non atteint)** : T5 se joue dans des dossiers de verrous propres à la copie (`KESH_RECETTE_TEST_VERROUS`, lue par `port_libre`, défaut le dossier par utilisateur), attentes relatives au port rendu ; cas (iv) neuf et mutation M6 (variable ignorée) ; D-C1 point 5 dit pourquoi T6 et une campagne à plusieurs copies restent verts ; T6 exige T5 vert dans les deux journaux. **LOW** : V2-2 (tenants de fond `flock -n`, prêts signalés par fichier, sonde de vérification, `kill`/`wait`, `trap` étendu, descripteurs fermés) ; V2-3 (`p` relâché quand `p + 1` est refusé ; garantie du dossier par utilisateur reformulée, cas du dossier squatté) ; V2-4 (T6 sous `/tmp/station-heavy.lock` puis `.gate.lock`, § 5.1) ; V2-5 (relance citée `:86-97`, `:96`) ; V2-6 (#1363 ajoutée aux écartées) ; V2-7 (I-6 « Compte analytique » ; `triage` sur I-2 à I-6) ; V2-8 (nouveau corps de #1499 prêt, posé à l'ouverture d'I-1, non à la fermeture) ; V2-9 (I-7, pied de la fiche de facture sans `pr-2`). Remédiation textuelle seule (fiche, lot C et sections d'issues) : passe ciblée P3 (Sonnet) sur ce seul commit, prompt `49-5-solde-dette-epic-validation-prompt-p3-ciblee.md`. |
| 2026-11-24 | **Passe ciblée P3 ; lot C retiré** (Opus 5.5). **Passe P3** : Sonnet 5.5, ciblée sur `7f435dbad`, contexte frais (`kesh-gate-logs/e49solde/validation-p3-ciblee.md`) : **0/0/1/4**, six axes déclarés, tous exercés. Le MEDIUM V3-1 naît du correctif de V2-2, lui-même né du correctif de V1-1/V1-7, **à sévérité égale (MEDIUM)** : **signal de recyclage** de la règle de splitting préventif (`CLAUDE.md`), tout entier dans le lot C. **Décision de l'orchestrateur** (à déclarer au Project Lead au résumé de 9 h) : **le lot C sort de la 49-5** ; la story garde les lots A (#1464) et B (#1466) ; plafond utilisé : deux issues sur trois. #1499 reste ouverte ; D-C1 à D-C4, AC-C1 à AC-C4, T5, T6, M4 à M6, la tâche et la note du lot C, et les constats V3-1 à V3-4 (non remédiés : remèdes écrits) sont **déplacés** dans `49-5-lot-c-reporte-1499.md`. I-1 reste à ouvrir (indépendante du lot C) ; le nouveau corps de #1499 se pose quand même, avec la mention du report. **V3-5** corrigé : « I-2 à I-7 » ; décompte des mutations recalculé sans le lot C — trois mutations (M1, M2, M3) et quatre tests (T1 à T4, dont T2 filet et T4 recette visuelle), recomptés au tableau des tests. Modules : inchangés (deux au plus, frontend seul) ; charge : 3 h 30 au lieu de 6 h. Passe ciblée P4 (Haiku) sur ce seul commit, prompt `49-5-solde-dette-epic-validation-prompt-p4-ciblee.md`. |
| 2026-11-24 | **Validation terminée** (Opus 5.5). **Passe ciblée P4** : Haiku, sur `0675644b7`, contexte frais (`kesh-gate-logs/e49solde/validation-p4-ciblee.md`) : **0/0/0/3**. Tendance des quatre passes (CRITICAL/HIGH/MEDIUM/LOW) : P1 Sonnet 0/1/2/9 → P2 Opus 0/0/1/8 → P3 ciblée Sonnet 0/0/1/4 (lot C retiré) → P4 ciblée Haiku 0/0/0/3. **V4-1** : estimation (2 h 30, dont 4 × 6 min de temps machine) et note « lancer tôt » du lot C ajoutées au report. **V4-2** : en-tête « deux issues sur le plafond de trois ». **V4-3** : le corps prévu de #1499 nomme la branche `story/49-5-solde-dette-epic` jusqu'à sa fusion. Remédiations textuelles seules ; prémisses de la fiche rejouées au code à l'étape 0 (voir Dev Agent Record). Statut `ready-for-dev` puis `in-progress`. |
| 2026-11-24 | **Développement terminé** (Sonnet 5.5, `bmad-dev-story`). Lots A (#1464) et B (#1466) livrés, rouges et mutations M1 à M3 prouvés (journaux au Dev Agent Record). **Revue de code P1** (Opus, `kesh-gate-logs/e49solde/review-p1.md`) : **0/0/0/3** ; **R1-1** corrigé (doc-comment de `CreateTextInvoiceLineRequest` : `heading` est dans le type, le serveur le refuse encore, `invoices.rs:626`), commentaire seul, **dernier commit de code** ; R5-1 : orchestrateur. **Premier E2E invalide** : `e2e-interne.sh` lançait `kesh-api` sans `cd` dans le worktree, le backend chargea les catalogues du dépôt principal (`crates/kesh-i18n/locales` relatif), sans les clés neuves de l'Epic 49 : 8 échecs (les sept KF-029 plus `invoice-validation-pdf.spec.ts:53`, clé brute `error-invoice-lines-do-not-fit-pdf`) ; consigné sans valeur, rejoué après correction (voir Dev Agent Record). |
| 2026-11-25 | **Clôture du développement ; statut `review`** (Sonnet 5.5). Tendance des passes (CRITICAL/HIGH/MEDIUM/LOW) : **validation** P1 Sonnet 0/1/2/9 → P2 Opus 0/0/1/8 → P3 ciblée Sonnet 0/0/1/4 (signal de recyclage, lot C sorti) → P4 ciblée Haiku 0/0/0/3 ; **revue de code** P1 Opus 0/0/0/3 → ciblée Haiku (`17126a445`) 0/0/1/2 → ciblée à venir sur ce commit de clôture. Remédiations : R1-1 (doc-comment, `heading` refusé par le serveur), F1 (case « Gates » cochée avec les chiffres réels), F2 (Status et tâche cohérents), F3 (« l'écran n'en crée pas »). Gates : backend 6270/6270 (17 ignorés) ; frontend 282 fichiers / 4851 tests ; E2E à `17126a445` vert (sept KF-029 seuls) ; premier E2E invalide, cause consignée. Issues ouvertes : I-1 = #1547, I-2 à I-7 = #1548 à #1553 ; #1499 a reçu son nouveau titre et son corps. Notes de version prêtes (section *Note de version*). |
| 2026-11-25 | **Clôture ; statut `done`** (orchestrateur). Passe ciblée Sonnet de `8cf1b5b53` : 0/0/0/2 (`kesh-gate-logs/e49solde/review-ciblee-8cf1b5b53.md`) ; G1 (ligne Status) et G2 (textes d'attente périmés, ici et dans `sprint-status.yaml`) corrigés dans ce commit, documents seuls ; boucle de revue close ; statut `done`. |
