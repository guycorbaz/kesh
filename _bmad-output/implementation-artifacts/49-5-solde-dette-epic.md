# Story 49-5 : Solde de la dette de l'Epic 49 — une charge de ligne de facture typée par son type (#1464), un avoir dont le taux et le compte ne se collent plus (#1466), un test de recette qui supporte deux copies simultanées (#1499)

Status: drafted — fiche écrite le 2026-11-24 ; validation P1 (Sonnet, 0/1/2/9) et P2 (Opus, 0/0/1/8) **remédiées** le 2026-11-24 ; **passe ciblée P3 à lancer** (prompt `49-5-solde-dette-epic-validation-prompt-p3-ciblee.md`, Sonnet) ; passe à `ready-for-dev` **après** la validation seulement.

> ⚠️ **Plafond provisoire — à confirmer par le Project Lead.** Le kickoff de l'Epic 49 (2026-11-18) n'a pas fixé de plafond
> d'issues pour sa story de solde (`CLAUDE.md` § *Priorités des défauts*). Cette fiche prend **trois issues**, le plafond que le
> Project Lead a retenu pour l'Epic 53, sur consigne de l'orchestrateur du 2026-11-24 (C-e49s-1). Si le Project Lead fixe un autre
> plafond, la fiche se réduit (les trois lots sont indépendants) ou s'étend depuis la section *Issues écartées*.

*Fiche rédigée le 2026-11-24 (Opus 5.5, `bmad-create-story`), worktree `/home/gcorbaz/devel/kesh-49-solde`, branche
`story/49-5-solde-dette-epic`, partie de `origin/epic/49-types-de-lignes-facture` à **`59fb28712`**. **Tout le code cité est lu à
`59fb28712`.** Release visée : **0.20.0**, vendredi 2026-11-27 (coupure le jeudi 2026-11-26 au soir) ; **la story doit être close
avant le vendredi 2026-11-27 au matin**. **Aucune migration, aucun changement du PDF, aucun changement de l'API.** Journaux :
`/home/gcorbaz/devel/kesh-gate-logs/e49solde/` (issues relues : `issues-candidates.json`, `issues-recherche.json` ; inventaires
`inv-1464.txt`, `inv-1466.txt`, `inv-1499.txt`, régénérables par les commandes qu'ils portent en tête). Clé courte des bases :
`e49s` (`kesh_e49s`, `kesh_ci_e49s`, `kesh_e2e_e49s`). Registre des choix : `e0200-choix-autonomes-e49solde.md`.*

## Story

En tant que comptable qui émet des factures et des avoirs, et en tant que développeur qui fait tourner plusieurs gates à la fois,
je veux que la charge d'une ligne de facture ne puisse pas, même en type, partir chiffrée sans montant ; que la fiche d'un avoir
sépare le taux de TVA du compte de produit ; et que le test de la recette de montée supporte deux copies lancées en même temps,
afin qu'un futur appelant ne puisse pas fabriquer une ligne que seul le serveur refuserait, que je ne lise pas « 8.103000 » là où
il y a « 8.10 » et « 3000 », et qu'un gate ne rougisse pas parce qu'un autre tourne à côté.

## Périmètre : la dette des stories de l'Epic 49, plafonnée à trois issues (provisoire)

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
| **#1499** | `technical-debt`, P3 | constatée pendant la 49-1d (code de la 33-10) | **oui** (lot C) |
| **#1466** | `bug`, P4 | revue de code P1 de la 49-1c, constat A-4 (préexistant) | **oui** (lot B) — la seule P4 `bug` retenable (les autres P4 `bug` nées de l'épique — #1347, #1411, #1495 — sont écartées pour leur raison propre, § *Issues écartées*), sur une pièce qu'il émet |

**Lecture de « dette des stories de l'epic »** : les **constats des revues et du développement** des stories de l'épique, non le
seul code qu'elles ont écrit. #1464 vise du code de la 49-1c ; #1466 (`git blame` : `credit-notes/[id]/+page.svelte:198`,
2026-06-27 ; `:199`, 2026-08-04) et #1499 (`recette-montee.test.sh:385-395`, `c239fc1096`, 2026-11-07, Story 33-10) visent du code
**antérieur** à l'épique, constaté par ses revues (49-1c A-4) ou pendant son développement (49-1d). La lecture « code écrit par
l'épique » ne retiendrait que #1464 ; elle est écartée parce que le `CLAUDE.md` ne la demande pas et qu'elle laisserait sans
story des constats nés de l'épique.

Aucune issue du PDF n'est retenue : #1363 et #1400 appartiennent à la 49-4b (Epic 52) ; la seule candidate de dette, #1399, change le rendu de toute facture non figée (§ *Issues écartées*).

**Modules** (grain du `CLAUDE.md` § *La passe ciblée* : un crate ou un domaine du frontend dont le **code de production** change) :
**deux au plus** — `frontend` domaine `invoices` (lot A : `src/lib/features/invoices/` ; si D-A3 exigeait de toucher
`InvoiceForm.svelte`, il vit à `src/lib/components/invoices/`, même domaine, autre dossier) et la route
`src/routes/(app)/credit-notes/[id]/` (lot B). Le `CLAUDE.md` définit le domaine par `frontend/src/lib/features/<domaine>` et ne
tranche pas le cas d'une route : lue comme domaine `credit-notes`, on compte deux modules ; lue comme hors domaine, un seul. Le
décompte tient sous les deux lectures. Le lot C ne touche que de l'outillage de test (`scripts/tests/`) et
`docs/testing.md` : pas de code de production. Aucun crate Rust ne change. Règle de splitting (> 5 modules) : non atteinte.

Les trois lots sont **indépendants** : aucun ne lit le code d'un autre ; chacun se retire de la fiche sans toucher aux deux autres.

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
Les constats I-2 à I-6 sont faits **au code seul**, sans capture.

**Angle mort déclaré du balayage** (écrit aussi en tête du script) : classes calculées (`class={…}`, `class:` directive, `clsx`),
styles en ligne, tableaux faits de `<div>`, cellules séparées par un bloc `{#if}`/`{#each}` dont la branche ne commence pas par une
cellule, marges posées par une feuille de style globale ou un composant parent (aucune ne pose de marge aux cellules aujourd'hui :
`grep -nE "\b(td|th)\b" frontend/src/app.css` vide, relevé de la validation P1). Deux colonnes **alignées à gauche** et sans
marge peuvent aussi se toucher quand la table manque de largeur : ce n'est pas le constat de l'issue, la capture ne le montre pas —
**angle mort assumé**, hors classe.

### Lot C — #1499 : deux copies du test de recette prennent les mêmes ports

1. **Le choix des ports** — `scripts/tests/recette-montee.test.sh:385-395` :
   ```bash
   # Deux ports libres, trouvés au moment du test, sous la plage éphémère du système : les
   # connexions à la base des autres tests y prendraient sinon le port entre deux contrôles.
   ports_a_l_ecoute() { ss -ltn | awk '{print $4}' | sed 's/.*://'; }
   port_libre() {
     local p=$1 pris
     pris=$(ports_a_l_ecoute)
     while /usr/bin/grep -qx "$p" <<<"$pris"; do p=$((p + 2)); done
     echo "$p"
   }
   PA=$(port_libre 23310)
   PB=$(port_libre 23410)
   ```
   « Libre » veut dire « personne n'écoute à l'instant » : **rien ne réserve** le port. Deux copies lancées ensemble lisent `ss`
   avant que l'une ou l'autre n'écoute, et prennent **les mêmes** `PA=23310`, `PB=23410`. Constat de l'issue **confirmé** au code.
2. **Ce qui fait alors collision** : le test écoute réellement sur `PORT_BASE + 1` pendant 30 s (`:864-873`, crochet de la phase 2 :
   `s.bind((\"127.0.0.1\", int(sys.argv[1]))); s.listen(1); time.sleep(30)" "$((PORT_BASE + 1))"`), et le script sous test refuse
   un port déjà pris (`scripts/recette-montee.sh:153-157` : `refus "port $p déjà pris"`) et relève à la fin un port de la recette
   resté à l'écoute (`:308-310`). Une copie voit donc le port de l'autre ; la collision se joue dans la fenêtre de 30 s de l'écoute
   du crochet. **Le mécanisme de l'interruption « sans résumé » est établi** par la fiche 49-1d (`49-1d-types-de-lignes-recette-montee.md:944`,
   « onze journaux tronqués, `mut/juge-tirage-parallele.log` », et `:1496-1502`, qui rattache ce défaut (ii) à #1499) : le script
   sous test refuse **avant** d'écrire « dossier », puis la ligne `D=$(/usr/bin/grep -oE 'dossier .*$' "$FX/sortie" | tail -1 | sed …)`
   rend 1 sous `pipefail`, et `set -e` (`:11`) arrête le test avant son résumé. Quatre sites ont cette forme : `:589`, `:755`,
   `:858`, `:875` (`grep -nF "grep -oE 'dossier .*\$'"`).
   **Périmètre** : cette story supprime la **collision de ports** (la cause constatée). La **fragilité** des quatre sites
   `D=$(grep …)` sous `set -e`/`pipefail` — toute autre cause de refus arrête encore le test sans dire pourquoi — **reste hors
   périmètre** (plafond de trois issues) : elle va à une **issue distincte, I-1**, que l'orchestrateur ouvrira avant le
   développement ; **#1499 ne couvre que la collision de ports**. La scission se fait **avant** le développement, à l'ouverture
   d'I-1 : le nouveau corps de #1499 (texte prêt, § *Issues à ouvrir par l'orchestrateur*) renvoie la fragilité à I-1.
3. **Les autres ressources partagées** (`inv-1499.txt`) : la racine jetable est unique (`mktemp -d /tmp/recette-montee-test.XXXXXX`,
   `:19`) ; `ecouter` (`:473-485`) laisse le système choisir son port (`bind(("127.0.0.1",0))`) ; aucun autre chemin fixe sous
   `/tmp` n'est écrit (la ligne `:574` est une assertion sur un chemin **dans le conteneur factice**). Les ports `23310`/`23410`
   n'apparaissent nulle part ailleurs dans `scripts/tests` ni `crates/*/tests/*`. Ensemble clos : **les ports seuls**.
4. **L'enveloppe** — `crates/kesh-db/tests/recette_montee_script.rs` lance le test sous `timeout -k 10 600` et exige le résumé
   « aucune assertion en échec ». Le test prend **337 à 389 s** (`docs/testing.md:643`, mesures du 2026-11-24) : une copie qui
   **attendrait** la fin de l'autre dépasserait les 600 s (D-C1, option écartée).
5. **La documentation** — `docs/testing.md:643` porte : « ⚠️ Ses scénarios choisissent leurs ports au moment du test : deux copies
   lancées en même temps se gênent, et l'une s'interrompt avant son résumé — une campagne de mutations les joue une à une. » Seul
   site qui décrit le symptôme (`git grep -n -iE 'deux copies|en même temps se g|mêmes ports' -- docs scripts CLAUDE.md`).
6. **Où la collision se produit réellement** : **pas** entre deux gates — `test-fast.sh`, `gate-ci.sh` et `cargo nextest` sont
   sérialisés par le verrou de voie `.gate.lock` (`docs/agents-orchestration.md` § 5.1, voie « Base et Rust »). Elle se produit dans
   une **campagne de mutations** qui lance plusieurs copies (fiche 49-1d `:944` : onze journaux tronqués) et dans tout lancement
   **hors verrou** (`bash scripts/tests/recette-montee.test.sh` direct, à côté d'un gate). La story reste justifiée (P3 ouverte,
   campagnes de mutations à venir), sur ce motif-là.

### Manuels

Aucune phrase des manuels ne décrit la marge des colonnes de la fiche d'un avoir, le type de la charge d'une ligne, ni le test de
recette (`docs/manual/*/` : à vérifier en validation, `.tex` et PDF aplatis, axe A8). `docs/testing.md:643` change (lot C).
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

### Lot C — #1499

- **D-C1 — Réserver le port, non seulement le constater, dans le shell principal** (*remédiation de V1-1, HIGH, et V1-7*).
  1. **`port_libre` ne s'appelle plus en substitution de commande.** `PA=$(port_libre 23310)` s'exécute dans un **sous-shell** : un
     descripteur ouvert là se ferme à sa sortie et le verrou tombe avant que `PA` ne serve. La fonction s'appelle **directement**
     et rend son résultat dans une **variable globale** : `port_libre 23310; PA=$PORT_RESERVE` puis `port_libre 23410 2;
     PB=$PORT_RESERVE`. **Sites d'appel réécrits** : `scripts/tests/recette-montee.test.sh:394` (`PA=$(port_libre 23310)`) et `:395`
     (`PB=$(port_libre 23410)`) — les deux seuls (`git grep -n 'port_libre' -- scripts` : `:388`, `:394`, `:395`). Les descripteurs
     sont gardés dans un tableau global (`VERROUS_PORTS+=("$fd")`), jamais fermés avant la fin du processus.
  2. **Le verrou** : pour chaque candidat `p` (et `p + 1` quand la fonction reçoit une largeur de 2, cas de `PB`), ouvrir
     `exec {fd}>"$DOSSIER_VERROUS/port-$p.lock"` (où `DOSSIER_VERROUS=${KESH_RECETTE_TEST_VERROUS:-/tmp/kesh-recette-montee-test-$(id -u)}`,
     relu à chaque appel, point 4) puis prendre `flock -n "$fd"`. **Sous `set -euo pipefail` (`:11`), `flock -n` se
     garde toujours par `if`** : `if flock -n "$fd"; then …; else exec {fd}>&-; p=$((p + 2)); continue; fi` — un `flock -n` nu en
     échec interromprait le test sans résumé, le symptôme même de l'issue. **Largeur 2** : si `p` est pris mais que `p + 1` est
     refusé (verrou tenu ou port à l'écoute), le verrou de `p` **se relâche** (`exec {fd}>&-` sur le descripteur de `p`) avant de
     passer au candidat suivant : aucun verrou orphelin ne reste tenu. L'ouverture du fichier se garde de même
     (`if ! exec {fd}>…; then …`, ou un test d'écriture préalable) et, en échec, arrête le test **avec un message** (`echo … >&2;
     exit 2`), jamais en silence.
  3. **`ss` contrôle `p` et `p + 1`** : un processus étranger à l'écoute sur `PB + 1` ferait refuser le script sous test
     (`recette-montee.sh:153-157`) ; le candidat est sauté si l'un des deux écoute.
  4. **Dossier des verrous** : `port_libre` lit la variable **`KESH_RECETTE_TEST_VERROUS`** ; son défaut est le dossier par
     utilisateur `/tmp/kesh-recette-montee-test-$(id -u)` (créé par `mkdir -p`, mode 700). C'est ce défaut que partagent les copies
     pour réserver `PA` et `PB` ; T5 le remplace par des dossiers propres à la copie (AC-C2). **Garantie, dite exactement** : un
     fichier de verrou créé par un autre compte dans un dossier partagé ne peut plus faire échouer l'ouverture, puisque chaque compte
     a son dossier ; **sauf dossier squatté** — un autre compte qui aurait créé `/tmp/kesh-recette-montee-test-<uid>` avant le premier
     lancement (`mkdir -p` réussit alors sans en changer le propriétaire) : l'ouverture échoue et le test s'arrête **avec un
     message** (point 2), cas théorique sur la station. Les fichiers de verrou **ne se
     retirent pas** (retirer un fichier dont le verrou vient d'être relâché rouvrirait la course entre deux copies) ; c'est écrit en
     commentaire au-dessus de `port_libre`.
  5. Deux copies simultanées prennent ainsi des ports distincts, sans attendre ni refuser. **T6 et une campagne de mutations à
     plusieurs copies restent verts** : `PA` et `PB` se réservent dans le dossier partagé (c'est ce que T6 éprouve), et T5 ne touche
     **jamais** ce dossier ni les plages 23310/23410 — il travaille dans des dossiers sous `$WORK` (unique par copie,
     `mktemp -d`, `:19`), sur des ports que rien n'écoute, et ses attentes sont relatives au port rendu : deux copies jouent donc T5
     en même temps sans se voir. Vérifié par la validation P1 : `PA`
     (pas de 2 depuis 23310) et `PB + 1` (impair, depuis 23411) ne se chevauchent pas avant 50 sauts.
  *Écartées* : un verrou global `flock -n` qui refuse la seconde copie (il fait rougir un lancement parce qu'un autre tourne — le
  défaut même) ; un verrou global bloquant (la seconde copie attendrait 337 à 389 s et dépasserait l'enveloppe de 600 s, § lot C,
  point 4) ; des ports choisis par le système (`bind` sur 0) puis relâchés (course entre la libération et l'usage, et `PB + 1` doit
  suivre `PB`) ; garder `$(…)` et ouvrir le verrou après coup dans le shell principal (une fenêtre de course entre le choix et le
  verrou).
- **D-C2 — Ce que la story vérifie avant d'écrire** (étape 0) : relu par la validation P1 sans objection — `exec 3>&1`/`3>&-`
  (`recette-montee.sh:317`, `:409`, `:423`) ne touchent que le descripteur 3, et la relance `env -i … bash` (`:86-97`, `exec` à `:96`) ne ferme pas
  les descripteurs ≥ 10. L'agent de développement le relit à HEAD. Les processus de fond du test héritent du descripteur : un
  verrou peut survivre quelques secondes à la sortie (écoutes de 30 et 60 s) ; la copie suivante saute alors au port d'après —
  **sans effet sur le verdict**, à écrire en commentaire au-dessus de `port_libre`. Si la relecture contredit D-C1, l'agent
  **s'arrête et le signale** au lieu d'improviser une autre mécanique.
- **D-C3 — `docs/ci.md:101`** (*remédiation de V1-10*) : ajouter `flock` (`util-linux`, présent sur `ubuntu-latest` et à la station)
  à la liste des outils requis (`python3`, `ss`, `jq`, `realpath`, `md5sum`, `timeout`), **et corriger dans le même patch** la
  valeur périmée de la même ligne — « `recette-montee.test.sh` : 99 à 104 s sous nextest » et « ajoutent une à deux minutes à la
  suite » — sur les mesures de `docs/testing.md:643` (337 à 389 s ; la phrase dit de quelle date et quel lancement). Greper les
  **valeurs** (`99 à 104`, `une à deux minutes`) sur tout le dépôt et traiter chaque site.
- **D-C4 — La phrase de `docs/testing.md:643`** se réécrit : deux copies simultanées prennent des ports distincts (verrou par port,
  dans le shell principal) ; une campagne de mutations peut donc lancer plusieurs copies **sans collision de ports** ; un refus du
  script sous test arrête encore le test avant son résumé (I-1, ouverte à part). Ne rien affirmer des gates, qui étaient déjà
  sérialisés par `.gate.lock` (§ lot C, point 6).

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

### Lot C — #1499

- **AC-C1** — `port_libre` est appelé directement aux deux sites (`:394`, `:395` d'origine) et rend `PORT_RESERVE` ; il réserve le
  port par un verrou exclusif non bloquant pris **dans le shell principal**, gardé par `if` ; `PB + 1` est réservé et contrôlé à `ss`
  avec `PB` ; dossier des verrous par utilisateur (D-C1). Constat : `git grep -n '\$(port_libre' -- scripts` **vide**, et lecture.
- **AC-C2** — **T5** (dans le test shell lui-même, section « ports ») se joue **entièrement dans des dossiers de verrous propres à la
  copie** — `KESH_RECETTE_TEST_VERROUS="$WORK/verrous-t5-<cas>" port_libre …` (affectation préfixée, qui ne vaut que pour l'appel)
  — sur des ports **distincts par cas**, hors de 23310/23410, et **ses attentes sont relatives au port rendu** (jamais une valeur
  absolue que l'état de la station pourrait déplacer) :
  - (i) **verrou tenu au retour** — après `port_libre 24310` (dossier `verrous-t5-i`), un processus de fond tente
    `flock -n <fichier du port rendu> true` — qui **rouvre** le fichier, jamais sur un descripteur hérité, dont la description
    partagée prendrait le verrou sans conflit — et **échoue** : le verrou est encore tenu par le shell principal (c'est ce cas qui
    rougit si le verrou est pris dans un sous-shell, M5) ;
  - (ii) **port tenu sauté** — un tenant de fond prend le verrou du port `q` (premier port libre à `ss` à partir de 24330, dossier
    `verrous-t5-ii`) ; `port_libre 24330` dans ce dossier rend un port **différent de `q` et supérieur** (M4) ;
  - (iii) **témoin positif** — dans un dossier `verrous-t5-iii` vide, `flock -n` sur un port jamais réservé (24390) réussit depuis
    un processus de fond ;
  - (iv) **la variable est lue** — un tenant de fond tient `r` (premier port libre à `ss` à partir de 24350) dans `verrous-t5-iv-autre` ;
    `KESH_RECETTE_TEST_VERROUS="$WORK/verrous-t5-iv-autre" port_libre 24350` rend un port ≠ `r`, et
    `KESH_RECETTE_TEST_VERROUS="$WORK/verrous-t5-iv" port_libre 24350` rend `r` (M6).
  **Tenants de fond synchronisés** (V2-2) : chacun se lance par `flock -n <fichier> bash -c 'touch "<prêt>"; sleep 60'`
  (toujours `-n`, jamais bloquant), sorties vers `/dev/null` et descripteurs du test fermés (`3>&-`, et ceux de `VERROUS_PORTS`),
  pour ne pas retenir la lecture de l'enveloppe Rust ; le test attend le fichier `<prêt>` par une boucle **bornée** (5 s, puis
  `fail` avec message), et **vérifie** par une sonde `flock -n … true` qui échoue que le verrou est bien tenu, avant d'appeler
  `port_libre`. Chaque tenant est tué et attendu (`kill`, `wait`) en fin de bloc, et un **`trap … EXIT`** étendu (`kill` des PID
  notés dans `T5_PIDS`, puis `rm -rf "$WORK"`, `:20`) garantit le nettoyage même si le test s'arrête. Témoins positif et négatif dans
  le même bloc.
- **AC-C3** — **T6** : **une seule invocation**, prise comme la voie « Base et Rust » (`docs/agents-orchestration.md` § 5.1,
  `:197` : recettes de montée et mutations), lance les deux copies en même temps :
  `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh bash -c 'bash scripts/tests/recette-montee.test.sh >
  …/t6-copie1.log 2>&1 & bash scripts/tests/recette-montee.test.sh > …/t6-copie2.log 2>&1 & wait'` (deux copies dans **une**
  place de `mem-guard`, qui n'en offre que deux sur la station : deux invocations séparées pourraient s'attendre et ne pas se
  recouvrir). Chaque copie **imprime au démarrage** `PA`, `PB` et un horodatage, et à la fin un horodatage ; le journal
  `kesh-gate-logs/e49solde/t6-deux-copies.log` réunit les deux sorties et **prouve le recouvrement** (intervalles qui se
  chevauchent) ; chaque copie rend « ✅ aucune assertion en échec » et le code 0, **T5 compris** (ses lignes ✓ figurent dans les deux journaux) ;
  les `PA`/`PB` sont distincts. Durée de chaque
  copie notée (deux copies concurrentes allongent les 337 à 389 s mesurés ; lancé directement, le test n'a pas l'enveloppe de
  600 s).
- **AC-C4** — `docs/testing.md:643` et `docs/ci.md:101` sont à jour (D-C3, D-C4) ; `git grep -n -iE 'deux copies lancées en même
  temps se gênent|99 à 104 s|une à deux minutes à la suite'` vide (ou chaque site restant justifié au Dev Agent Record).

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
| **T5** — réservation (AC-C2, cas i à iv) | C | `scripts/tests/recette-montee.test.sh` | **rouge par mutations** M4, M5 et M6 |
| **M4** — mutation : `port_libre` sans verrou (retour au code de `59fb28712`) | C | idem | T5 (ii) rouge (le port tenu n'est pas sauté) ; journal `m4.log` |
| **M6** — mutation : `port_libre` ignore `KESH_RECETTE_TEST_VERROUS` (dossier par utilisateur en dur) | C | idem | T5 (iv) rouge (le premier appel rend `r`, le port tenu ailleurs) ; journal `m6.log`. Preuve que T5 joue dans ses propres dossiers, donc que deux copies ne s'y voient pas (V2-1) |
| **M5** — mutation : verrou pris **dans un sous-shell** — `port_libre` remis en `PORT=$( … )` autour de l'ouverture et du `flock` | C | idem | T5 (i) rouge (le verrou est relâché au retour : le `flock -n` de fond réussit) ; journal `m5.log`. C'est la preuve que V1-1 est fermé, non portée par T6 seul (probabiliste) |
| **T6** — deux copies simultanées (AC-C3) | C | une invocation sous `.gate.lock` et `mem-guard` | **rouge avant le code** : les deux copies sur `59fb28712`, même invocation — au moins une sans résumé ou en échec (collision dans la fenêtre de 30 s du crochet), journal `t6-rouge.log` ; si deux tirages ne reproduisent pas, le dire au Dev Agent Record (le rouge reste porté par M4 et M5) |

## Tâches

- [ ] **Étape 0** — Bases `kesh_e49s`, `kesh_ci_e49s`, `kesh_e2e_e49s` (consignes `docs/agents-orchestration.md` § 4) ; relire les
      `chemin:ligne` de la fiche à HEAD ; lot A, point 6 (prédicat dans les gabarits) ; D-C2 (descripteurs dans
      `recette-montee.sh`). Commit et push de la branche `story/…`.
- [ ] **Lot A** — T1 (rouge, journal) ; D-A1 ; D-A2 ; T1 vert ; M1, M2 ; doc-comments (`invoices.types.ts`, `invoice-line-type.ts`).
      Commit `refs #1464`.
- [ ] **Lot B** — captures **avant** (T4) ; T3 (rouge, journal) ; D-B1 ; T3 vert ; M3 ; captures **après**. Commit `refs #1466`.
- [ ] **Lot C** — T6-rouge (deux copies sur le code d'origine, une invocation) ; D-C1 (sites `:394`, `:395` réécrits) ; T5 ; M4, M5, M6 ;
      T6 vert ; D-C3 (dont la valeur périmée de `docs/ci.md:101`), D-C4. Commit `refs #1499`.
- [ ] **Propagation** — `git grep` des symptômes corrigés (`quantity?:`, « deux copies », `py-2 text-right">{line.vatRate}`) sur tout
      le dépôt ; tout site neuf traité ou écrit.
- [ ] **Gates** (§ *Gates*), Dev Agent Record, Change Log ; statut `review`.

## Gates

- Frontend (`CLAUDE.md` § *Frontend*) : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`, sous
  `scripts/mem-guard.sh`.
- Backend (le lot C est lancé par `crates/kesh-db/tests/recette_montee_script.rs`) : `scripts/test-fast.sh` puis
  `scripts/gate-ci.sh kesh_ci_e49s`, sous le verrou `flock <kesh-gate-logs>/.gate.lock`, base remise à zéro selon
  `docs/agents-orchestration.md` (bases propres, jamais de redémarrage du conteneur partagé).
- E2E complet au dernier commit de code (`docs/testing.md` § *Prérequis Playwright local*, à la lettre) ; seulement les échecs
  attendus.
- Le Dev Agent Record n'affirme que ce qui a tourné, chiffres recomptés depuis les journaux.

## Note de version

- Avoirs : sur la fiche d'un avoir, le taux de TVA et le compte de produit ne se touchent plus — « 8.10 » et « 3000 — Ventes » se
  lisaient « 8.103000 — Ventes » (refs #1466).
- Interne : la charge d'une ligne de facture envoyée par l'écran est typée par son type — une ligne chiffrée ne peut plus, même en
  type, partir sans quantité, prix ou taux (refs #1464) ; le test de la recette de montée supporte deux exécutions simultanées
  (refs #1499).

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
| #1363 | `bug`, P3 — **PDF** | Désignation tronquée à 44 caractères : portée par la 49-4b (Epic 52), milestone v0.22.0 ; pas de la dette des revues de l'épique. |
| #1330, #1374, #1364, #1400, #1401, #1423 | — | Hors classe : CR, issues portées par les stories elles-mêmes, ou rangées à d'autres milestones (v0.20.0, v0.21.0, v0.22.0, « Plus tard ») ; pas de la dette des revues de l'épique. |


## Issues à ouvrir par l'orchestrateur

Sept issues, **une par problème**, à ouvrir **avant le développement** (I-1, avec le nouveau corps de #1499) ou à la clôture (I-2 à
I-7), au rythme des limites GitHub (cinq secondes au moins entre deux créations). Texte prêt :

**I-1** — `technical-debt`, P3 — *recette-montee.test.sh : un refus du script sous test arrête le test sous `set -e` avant son
résumé (`D=$(grep -oE 'dossier .*$' …)`)*
> Dans `scripts/tests/recette-montee.test.sh`, quatre sites lisent le dossier de la recette par
> `D=$(/usr/bin/grep -oE 'dossier .*$' "$FX/sortie" | tail -1 | sed 's/^dossier //')` (`:589`, `:755`, `:858`, `:875` à `59fb28712`).
> Quand `scripts/recette-montee.sh` refuse avant d'écrire « dossier », `grep` rend 1, `pipefail` le propage et `set -e` (`:11`)
> arrête le test **sans résumé** : l'enveloppe Rust (`crates/kesh-db/tests/recette_montee_script.rs`) ne dit alors que « le test
> shell n'a pas rendu son résumé », sans la cause. Établi par la Story 49-1d (fiche `:944`, onze journaux tronqués d'une campagne de
> mutations). La collision de ports qui provoquait ce refus est traitée par #1499 (Story 49-5) ; la fragilité demeure pour toute
> autre cause de refus. **Remède** : lire le dossier sans arrêt du test (`… || true`, ou lecture sans tube) et faire échouer
> l'assertion qui suit, avec la sortie du script. Origine : validation P1 de la fiche 49-5 (V1-2), refs #1499.

**I-2** — `bug`, `triage`, P4 — *Journal d'audit : « N° » et « Détails » collés (marge droite absente)*
> `frontend/src/routes/(app)/audit-log/+page.svelte:364-365` (en-têtes) et `:387-388` (cellules) : la cellule du numéro d'entité,
> alignée à droite (`class="py-2 text-right"`), est suivie de la cellule « Détails », alignée à gauche, sans marge horizontale ;
> leurs contenus se touchent. Même défaut que #1466 (fiche d'un avoir), corrigé là par `pr-2` sur chaque cellule (convention de la
> fiche de facture). Constat au code seul (balayage de la validation P1 de la Story 49-5), sans capture. Cosmétique.

**I-3** — `bug`, `triage`, P4 — *Détail d'un import bancaire : « Montant » et « Référence » collés*
> `frontend/src/routes/(app)/bank-import/[id]/+page.svelte:170-171` (en-têtes `class="text-right"` → `class="text-left"`) et
> `:183-184` (`{tx.amount} {tx.currency}` → `{tx.reference ?? '—'}`) : aucune marge horizontale entre la colonne du montant,
> alignée à droite, et celle de la référence. Même défaut que #1466. Constat au code seul (Story 49-5), sans capture. Cosmétique.

**I-4** — `bug`, `triage`, P4 — *Import de relevé (aperçu) : « Montant » et « Détails » collés*
> `frontend/src/lib/features/bank-import/BankImportUpload.svelte:867-868` (en-têtes) et `:875-876` (cellules) : la colonne du
> montant, alignée à droite, est suivie de « Détails », alignée à gauche, sans marge horizontale. Même défaut que #1466. Constat au
> code seul (Story 49-5), sans capture. Cosmétique.

**I-5** — `bug`, `triage`, P4 — *Propositions de rapprochement : « Montant » et « Contrepartie » collés*
> `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte:708-709` (en-têtes) et `:756-757` (cellules,
> `{p.transaction.amount} {p.transaction.currency}` → `{p.transaction.counterpartyName ?? ''}`) : aucune marge horizontale entre
> le montant, aligné à droite, et la contrepartie. Même défaut que #1466. Constat au code seul (Story 49-5), sans capture.
> Cosmétique.

**I-6** — `bug`, `triage`, P4 — *Saisie d'une écriture : en-têtes « Crédit » et « Compte analytique » collés quand la colonne
analytique est affichée*
> `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte:864-873` : l'en-tête « Crédit » (`text-right`, sans marge) est
> suivi de l'en-tête « Compte analytique » (`:874`, clé `journal-entry-form-col-project` ; `text-left`, sans marge) quand
> `showProjectColumn` est vrai ; les cellules du corps portent `pr-2`
> (`:884-951`), pas les en-têtes. Constat au code seul (Story 49-5), à confirmer à l'écran avant correction. Cosmétique.

**I-7** — `bug`, `triage`, P4 — *Fiche d'une facture : les montants du pied sont décalés de 8 px vers la droite par rapport aux totaux
de ligne*
> `frontend/src/routes/(app)/invoices/[id]/+page.svelte` : le total de chaque ligne porte `pr-2` (`:1410`,
> `<td class="py-2 pr-2 text-right font-mono">`), mais pas les montants du pied (`:1429` sous-total HT, `:1438` TVA par taux,
> `:1448` arrondi, `:1457` et `:1466`, `class="py-1 text-right font-mono"` / `class="py-3 text-right …"`) : les montants du pied
> ne s'alignent pas sur la colonne des totaux de ligne. Relevé par la validation P2 de la fiche 49-5 (la fiche d'un avoir, corrigée
> par #1466, met `pr-2` aussi sur son pied). Constat au code seul, sans capture. Cosmétique.

**Nouveau corps de #1499** (à poser par l'orchestrateur **à l'ouverture d'I-1**, avant le développement ; le titre devient
*« recette-montee.test.sh : deux exécutions simultanées prennent les mêmes ports »*) :
> Deux exécutions simultanées de `scripts/tests/recette-montee.test.sh` prennent les mêmes ports : `port_libre`
> (`:388-395` à `59fb28712`) constate qu'un port n'est pas à l'écoute mais ne le **réserve** pas, si bien que deux copies lancées
> ensemble choisissent `PA=23310` et `PB=23410` ; l'une voit alors le port de l'autre (écoute du crochet sur `PB + 1`, `:864-873`)
> et le script sous test refuse (`scripts/recette-montee.sh:153-157`). Code de la Story 33-10, constaté pendant la Story 49-1d.
> **Remède** (Story 49-5) : réservation de chaque port par un verrou `flock` pris dans le shell principal.
> **Hors de cette issue** : l'arrêt du test sous `set -e` avant son résumé quand le script sous test refuse (les quatre
> `D=$(grep -oE 'dossier .*$' …)`) est suivi par I-1 (#N à reporter), qu'il vienne de cette collision ou d'une autre cause.
> Origine : story 49-1d (#1242) ; scission décidée à la validation de la fiche 49-5.

## Dev Notes

- **Factures réelles** : aucun des trois lots ne touche le serveur, le PDF, les données ni l'API (AC-T1). Le lot B ne change que
  des classes CSS d'une page de consultation.
- **Ordre** : les lots sont indépendants ; le lot C coûte le plus en temps machine (deux copies de 6 min, deux fois) : le lancer tôt.
- **Lot A** : le test de type vit dans un `*.test.ts` sous `src/` (couvert par `npm run check`, § lot A, point 7) ; vitest
  l'exécute aussi : les cas sont dans un `it(…)` avec un `expect` (AC-A2), chacun sur une ligne, affecté à une constante typée.
- **Lot C** : `exec {fd}>"$f"` alloue un descripteur ≥ 10 ; il n'existe que dans le shell qui l'a ouvert — d'où l'appel direct
  (D-C1, point 1). Garder les descripteurs dans un tableau global pour que rien ne les ferme avant la fin. Ne pas toucher
  `scripts/recette-montee.sh` (le script de la release) : seul son test change. Les quatre `D=$(grep …)` (`:589`, `:755`, `:858`,
  `:875`) **ne se touchent pas** ici (I-1).
- **Mutations** : par l'outil `Edit`, défaites par `git checkout -- <fichier>` puis `touch` (mémoire « mutation restaurée, binaire
  périmé »).
- **Estimation** : lot A 1 h, lot B 1 h (captures comprises), lot C 2 h 30 (dont au moins 4 × 6 min de temps machine : T6-rouge, T6 vert, M4, M5, plus la durée allongée de deux copies concurrentes), gates et E2E 1 h 30 ; revue
  de code une à deux passes. Close le mercredi 2026-11-25 au soir si la validation finit le 2026-11-25 au matin.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Entrée |
|---|---|
| 2026-11-24 | **Fiche écrite** (Opus 5.5, `bmad-create-story`), base `59fb28712`. Plafond **provisoire** de trois issues, à confirmer par le Project Lead (C-e49s-1). Retenues : #1464 (P3), #1499 (P3), #1466 (P4) ; #1399 (PDF) et #1465 (P3) écartées au code (C-e49s-2, C-e49s-3). Deux modules de production (`invoices`, `credit-notes`), aucune migration. Statut `drafted` ; validation P1 (Sonnet) à lancer. |
| 2026-11-24 | **Validation P1 remédiée** (Opus 5.5). **Passe P1** : Sonnet 5.5, complète, contexte frais (`kesh-gate-logs/e49solde/validation-p1.md`) : **0/1/2/9** (CRITICAL/HIGH/MEDIUM/LOW), onze axes déclarés, tous exercés (A6 par raisonnement : aucune exécution). Prémisses relues au code avant remède. **V1-1 (HIGH)** : `PA=$(port_libre …)` (`recette-montee.test.sh:394-395`) tient le verrou dans un sous-shell — D-C1 réécrit : appel direct, résultat dans `PORT_RESERVE`, sites `:394`, `:395` nommés, `flock -n` gardé par `if` sous `set -e` ; T5 (i) rougit si le verrou est perdu au retour, mutation M5 neuve. **V1-2 (MEDIUM)** : « mécanisme non établi » retiré, cité à la 49-1d (`:944`, `:1496-1502`) ; les quatre `D=$(grep -oE …)` (`:589`, `:755`, `:858`, `:875`) restent hors périmètre (plafond de trois issues) → issue I-1, et #1499 ne couvre que la collision de ports (corps précisé à sa fermeture). **V1-3 (MEDIUM)** : inventaire du lot B refait par un balayage de tous les `.svelte` (`kesh-gate-logs/e49solde/outils/paires-droite-gauche.py`, lecture seule) : quinze paires, ensemble clos classé ; hors module → une issue P4 chacune (I-2 audit-log, I-3 bank-import/[id], I-4 BankImportUpload, I-5 ReconciliationProposals, et I-6 JournalEntryForm, que le balayage a ajouté) ; angle mort du balayage écrit. **LOW** : V1-4 (chemin de `InvoiceForm.svelte`, lecture du grain « module » sous les deux sens, `ecouter` `:473-485`) ; V1-5 (exposition réelle : campagnes de mutations et lancements hors verrou, non les gates, sérialisés par `.gate.lock`) ; V1-6 (AC-C3 : une seule invocation sous `.gate.lock` et `mem-guard`, preuve du recouvrement par horodatages) ; V1-7 (`PB + 1` contrôlé à `ss`, verrous dans un dossier par utilisateur, fichiers non retirés, plage distincte pour T5) ; V1-8 (cas de type sur une ligne, dans un `it`) ; V1-9 (le pied de la facture n'a pas `pr-2` : écrit dans D-B1) ; V1-10 (`docs/ci.md:101` : `flock` et la valeur périmée « 99 à 104 s » corrigés dans le lot C) ; V1-11 (« seule P4 `bug` retenable », lecture de « dette des stories », #1504 cité à `verifs-0200.sh:15`). Section *Issues à ouvrir par l'orchestrateur* ajoutée (I-1 à I-6, texte prêt). Remédiation textuelle seule (fiche) ; validation P2 (Opus, complète) à lancer, prompt `49-5-solde-dette-epic-validation-prompt-p2.md`. |
| 2026-11-24 | **Validation P2 remédiée** (Opus 5.5). **Passe P2** : Opus 5.5, complète, contexte frais (`kesh-gate-logs/e49solde/validation-p2.md`) : **0/0/1/8**, treize axes déclarés, tous exercés (A6 par raisonnement et sondes sans écriture). Prémisses relues au code avant remède (`docs/agents-orchestration.md:197`, `recette-montee.sh:86`, `:96`, `JournalEntryForm.svelte:874`, `invoices/[id]/+page.svelte:1410`, `:1429`, `:1438`, `:1448`, `:1457`, `:1466`). **V2-1 (MEDIUM, né de la remédiation de V1-1/V1-7 ; sévérité inférieure au HIGH de la P1 : signal de splitting non atteint)** : T5 se joue dans des dossiers de verrous propres à la copie (`KESH_RECETTE_TEST_VERROUS`, lue par `port_libre`, défaut le dossier par utilisateur), attentes relatives au port rendu ; cas (iv) neuf et mutation M6 (variable ignorée) ; D-C1 point 5 dit pourquoi T6 et une campagne à plusieurs copies restent verts ; T6 exige T5 vert dans les deux journaux. **LOW** : V2-2 (tenants de fond `flock -n`, prêts signalés par fichier, sonde de vérification, `kill`/`wait`, `trap` étendu, descripteurs fermés) ; V2-3 (`p` relâché quand `p + 1` est refusé ; garantie du dossier par utilisateur reformulée, cas du dossier squatté) ; V2-4 (T6 sous `/tmp/station-heavy.lock` puis `.gate.lock`, § 5.1) ; V2-5 (relance citée `:86-97`, `:96`) ; V2-6 (#1363 ajoutée aux écartées) ; V2-7 (I-6 « Compte analytique » ; `triage` sur I-2 à I-6) ; V2-8 (nouveau corps de #1499 prêt, posé à l'ouverture d'I-1, non à la fermeture) ; V2-9 (I-7, pied de la fiche de facture sans `pr-2`). Remédiation textuelle seule (fiche, lot C et sections d'issues) : passe ciblée P3 (Sonnet) sur ce seul commit, prompt `49-5-solde-dette-epic-validation-prompt-p3-ciblee.md`. |
