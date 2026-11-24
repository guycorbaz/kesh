# Story 49-5 : Solde de la dette de l'Epic 49 — une charge de ligne de facture typée par son type (#1464), un avoir dont le taux et le compte ne se collent plus (#1466), un test de recette qui supporte deux copies simultanées (#1499)

Status: drafted — fiche écrite le 2026-11-24 ; **validation P1 à lancer** (prompt `49-5-solde-dette-epic-validation-prompt-p1.md`, Sonnet) ; passe à `ready-for-dev` **après** la validation seulement.

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
| **#1466** | `bug`, P4 | revue de code P1 de la 49-1c, constat A-4 (préexistant) | **oui** (lot B) — la seule P4 visible de l'utilisateur, sur une pièce qu'il émet |

Aucune issue du PDF n'est retenue : la seule candidate, #1399, change le rendu de toute facture non figée (§ *Issues écartées*).

**Modules** (grain du `CLAUDE.md` § *La passe ciblée* : un crate ou un domaine du frontend dont le **code de production** change) :
**deux** — `frontend` domaine `invoices` (lot A : `src/lib/features/invoices/`) et `frontend` domaine `credit-notes` (lot B : la
route `src/routes/(app)/credit-notes/[id]/`). Le lot C ne touche que de l'outillage de test (`scripts/tests/`) et
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

**Inventaire par le symptôme** (« une cellule alignée à droite immédiatement suivie d'une cellule alignée à gauche, sans marge
horizontale », `inv-1466.txt`, commande `git grep -nE '<t[hd] class="py-[0-9]( text-right)?"' -- 'frontend/src/routes/(app)'`) :

| Site | Paire droite → gauche sans marge | Classement |
|---|---|---|
| `credit-notes/[id]/+page.svelte:172-174`, `:198-199` | TVA % → Compte de produit | **corrigé** (D-B1) |
| `audit-log/+page.svelte:364-365`, `:387-388` | N° → Détails | **hors périmètre** (domaine `audit-log`, un troisième module) : **issue à ouvrir par l'orchestrateur** (P4, cosmétique), constat au code seul, non capturé |
| `supplier-invoices/[id]/+page.svelte:1029-1056` | aucune : après la description, toutes les colonnes sont à droite | hors classe |
| `credit-notes/+page.svelte:51-68`, `payment-batches/+page.svelte:255-272`, `payment-batches/[id]/+page.svelte:129-147`, `supplier-invoices/+page.svelte:331-365` | aucune : la seule colonne à droite est la dernière | hors classe |
| `settings/dunning/+page.svelte:260-272`, `settings/vat-rates/+page.svelte:219-241`, `journal-entries/+page.svelte:645-662`, `journal-entries/[id]/+page.svelte:684-687` | aucune paire droite → gauche (dernière colonne à droite, ou `td` vide) | hors classe |
| `bank-accounts/+page.svelte:976`, `settings/api-keys/+page.svelte:389` | `th` vide | hors classe |

Deux colonnes **alignées à gauche** et sans marge peuvent aussi se toucher quand la table manque de largeur ; ce n'est pas le
constat de l'issue, et la capture ne le montre pas : **angle mort assumé**, hors classe.

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
   resté à l'écoute (`:308-310`). Une copie voit donc le port de l'autre — refus ou constat inattendu, d'où l'interruption décrite par
   l'issue. Le mécanisme exact de l'interruption « sans résumé » (`set -euo pipefail`, `:11`) n'est pas établi par lecture :
   **à reproduire à l'étape 0** (T5-rouge).
3. **Les autres ressources partagées** (`inv-1499.txt`) : la racine jetable est unique (`mktemp -d /tmp/recette-montee-test.XXXXXX`,
   `:19`) ; `ecouter` (`:472-481`) laisse le système choisir son port (`bind(("127.0.0.1",0))`) ; aucun autre chemin fixe sous
   `/tmp` n'est écrit (la ligne `:574` est une assertion sur un chemin **dans le conteneur factice**). Les ports `23310`/`23410`
   n'apparaissent nulle part ailleurs dans `scripts/tests` ni `crates/*/tests/*`. Ensemble clos : **les ports seuls**.
4. **L'enveloppe** — `crates/kesh-db/tests/recette_montee_script.rs` lance le test sous `timeout -k 10 600` et exige le résumé
   « aucune assertion en échec ». Le test prend **337 à 389 s** (`docs/testing.md:643`, mesures du 2026-11-24) : une copie qui
   **attendrait** la fin de l'autre dépasserait les 600 s (D-C1, option écartée).
5. **La documentation** — `docs/testing.md:643` porte : « ⚠️ Ses scénarios choisissent leurs ports au moment du test : deux copies
   lancées en même temps se gênent, et l'une s'interrompt avant son résumé — une campagne de mutations les joue une à une. » Seul
   site qui décrit le symptôme (`git grep -n -iE 'deux copies|en même temps se g|mêmes ports' -- docs scripts CLAUDE.md`).
6. **Pourquoi c'est réel ici** : la station fait tourner deux voies de gate en parallèle (`CLAUDE.md` § *Plafonds mémoire* : « deux
   exécutions à la fois sur toute la station ») ; deux `test-fast.sh` ou `gate-ci.sh` de deux worktrees lancent chacun ce test.

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
  `invoices/[id]/+page.svelte:1374-1410`. *Écartées* : `pr-2` sur la seule cellule du taux (la marge naîtrait d'une exception, et la
  prochaine paire droite → gauche aurait le même défaut) ; `pl-*` sur la cellule du compte (même raison) ; ajouter `%` au taux
  comme sur la facture (`{l.vatRate}%`) — c'est un changement de contenu, hors du constat ; et sans marge, « 8.10%3000 » resterait
  collé.
- **D-B2 — Rien d'autre ne change** : ni textes, ni clés i18n, ni `data-testid`, ni structure (`colspan`, nombre de cellules).

### Lot C — #1499

- **D-C1 — Réserver le port, non seulement le constater** : `port_libre` ne rend un port qu'après avoir pris un **verrou exclusif
  non bloquant** sur un fichier qui le nomme (`flock -n` sur `/tmp/kesh-recette-montee-test.port-<p>.lock`, descripteur ouvert par
  `exec {fd}>…` et gardé jusqu'à la fin du processus) ; un port dont le verrou est tenu, ou qui est à l'écoute (`ss`, comme
  aujourd'hui), est sauté. `PB` réserve **aussi** `PB + 1` (le port de la seconde base, phase 5 et crochet `:870`). Deux copies
  simultanées prennent ainsi des ports distincts, sans attendre ni refuser. *Écartées* : un verrou global `flock -n` qui refuse la
  seconde copie (il fait rougir un gate parce qu'un autre tourne — le défaut même) ; un verrou global bloquant (la seconde copie
  attendrait 337 à 389 s et dépasserait l'enveloppe de 600 s, § lot C, point 4) ; des ports choisis par le système (`bind` sur 0)
  puis relâchés (course entre la libération et l'usage, et `PB + 1` doit suivre `PB`).
- **D-C2 — Ce que la story vérifie avant d'écrire** (étape 0) : que `scripts/recette-montee.sh`, lancé par le test dans un
  environnement fermé, ne ferme ni n'inspecte les descripteurs hérités au point de rendre le verrou inopérant ou de refuser son
  lancement (lire `recette-montee.sh:55-110`, la relance sous environnement fermé, et `:317` `exec 3>&1`). Les processus de fond
  du test héritent du descripteur : un verrou peut survivre quelques secondes à la sortie (écoutes de 30 et 60 s) ; la copie
  suivante saute alors au port d'après — **sans effet sur le verdict**, à écrire en commentaire au-dessus de `port_libre`. Si la
  lecture contredit D-C1, l'agent de développement **s'arrête et le signale** au lieu d'improviser une autre mécanique.
- **D-C3 — `flock`** est de `util-linux`, présent sur `ubuntu-latest` et à la station ; l'ajouter à la liste des outils requis
  (`docs/ci.md:101` nomme ceux des tests des scripts de la Story 33-10 : `python3`, `ss`, `jq`, `realpath`, `md5sum`, `timeout`).
- **D-C4 — La phrase de `docs/testing.md:643`** se réécrit : deux copies simultanées prennent des ports distincts (verrou par port),
  et la campagne de mutations n'a plus à les jouer une à une **pour les ports** (elle peut garder cet ordre pour d'autres raisons,
  qu'elle ne dit pas : ne rien affirmer de plus).

## Critères d'acceptation

### Lot A — #1464

- **AC-A1** — `invoices.types.ts` exporte `InvoiceTextLineType`, `CreatePricedInvoiceLineRequest`, `CreateTextInvoiceLineRequest`
  et `CreateInvoiceLineRequest` = leur union (D-A1) ; `quantity`, `unitPrice`, `vatRate` sont **obligatoires** dans la variante
  `priced` et **absents** de la variante de texte. Constat : lecture du fichier, et `git grep -nE "(quantity|unitPrice|vatRate)\?:"
  -- frontend/src/lib/features` **vide**.
- **AC-A2** — Un test de type (T1, `invoice-lines.test.ts` ou un fichier voisin `invoice-line-request.types.test.ts`) porte, chacune
  sous `// @ts-expect-error` : (a) une ligne `priced` sans `quantity` ; (b) sans `unitPrice` ; (c) sans `vatRate` ; (d) une ligne
  `comment` avec `quantity` ; (e) une ligne `heading` avec `vatRate` ; et **sans** directive : (f) une ligne `priced` complète ;
  (g) `{ lineType: 'comment', description }`. `npm run check` : **0 erreur**.
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

- **AC-C1** — `port_libre` réserve le port par un verrou exclusif non bloquant (D-C1) ; `PB + 1` est réservé avec `PB`.
- **AC-C2** — T5 (dans le test shell lui-même, section « ports ») : un processus de fond tient le verrou du premier port candidat ;
  `port_libre` rend le suivant ; le verrou relâché, il rend de nouveau le premier. Témoin positif et négatif dans le même bloc.
- **AC-C3** — T6 : **deux copies** de `bash scripts/tests/recette-montee.test.sh` lancées ensemble (sous `scripts/mem-guard.sh`)
  rendent **chacune** « ✅ aucune assertion en échec » et le code 0 ; journal `kesh-gate-logs/e49solde/t6-deux-copies.log` (sorties
  des deux, avec `PA`/`PB` de chacune, distincts).
- **AC-C4** — `docs/testing.md:643` et `docs/ci.md:101` (outil `flock`) sont à jour (D-C3, D-C4) ; `git grep -n -iE 'deux copies
  lancées en même temps se gênent'` vide.

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
| **T5** — réservation (AC-C2) | C | `scripts/tests/recette-montee.test.sh` | **rouge par mutation** M4 |
| **M4** — mutation : `port_libre` sans verrou (retour au code de `59fb28712`) | C | idem | T5 rouge (le port tenu n'est pas sauté) ; journal `m4.log` |
| **T6** — deux copies simultanées (AC-C3) | C | lancement manuel | **rouge avant le code** : les deux copies sur `59fb28712` — au moins une sans résumé ou en échec, journal `t6-rouge.log` ; si deux tirages ne reproduisent pas, le dire au Dev Agent Record (le rouge reste porté par M4) |

## Tâches

- [ ] **Étape 0** — Bases `kesh_e49s`, `kesh_ci_e49s`, `kesh_e2e_e49s` (consignes `docs/agents-orchestration.md` § 4) ; relire les
      `chemin:ligne` de la fiche à HEAD ; lot A, point 6 (prédicat dans les gabarits) ; D-C2 (descripteurs dans
      `recette-montee.sh`). Commit et push de la branche `story/…`.
- [ ] **Lot A** — T1 (rouge, journal) ; D-A1 ; D-A2 ; T1 vert ; M1, M2 ; doc-comments (`invoices.types.ts`, `invoice-line-type.ts`).
      Commit `refs #1464`.
- [ ] **Lot B** — captures **avant** (T4) ; T3 (rouge, journal) ; D-B1 ; T3 vert ; M3 ; captures **après**. Commit `refs #1466`.
- [ ] **Lot C** — T6-rouge (deux copies sur le code d'origine) ; D-C1 ; T5 ; M4 ; T6 vert ; D-C3, D-C4. Commit `refs #1499`.
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
| #1504 | `technical-debt`, P4 | Déjà corrigée dans la 49-1d (commit `d2416c596`, commentaire de l'issue) ; se ferme avec la PR de l'épique. |
| #1330, #1374, #1364, #1400, #1401, #1423 | — | Hors classe : CR, issues portées par les stories elles-mêmes, ou rangées à d'autres milestones (v0.20.0, v0.21.0, v0.22.0, « Plus tard ») ; pas de la dette des revues de l'épique. |

**Issue à demander à l'orchestrateur** : la même paire droite → gauche sans marge sur le journal d'audit
(`frontend/src/routes/(app)/audit-log/+page.svelte:364-365`, `:387-388`, N° → Détails), constatée au code seul (P4, cosmétique).

## Dev Notes

- **Factures réelles** : aucun des trois lots ne touche le serveur, le PDF, les données ni l'API (AC-T1). Le lot B ne change que
  des classes CSS d'une page de consultation.
- **Ordre** : les lots sont indépendants ; le lot C coûte le plus en temps machine (deux copies de 6 min, deux fois) : le lancer tôt.
- **Lot A** : le test de type vit dans un `*.test.ts` sous `src/` (couvert par `npm run check`, § lot A, point 7) ; vitest
  l'exécute aussi : chaque cas est une affectation à une constante typée, sans effet à l'exécution (`void x;` si le linter le
  demande).
- **Lot C** : `exec {fd}>"$f"` alloue un descripteur ≥ 10 ; garder les descripteurs dans un tableau global pour que rien ne les
  ferme avant la fin. Ne pas toucher `scripts/recette-montee.sh` (le script de la release) : seul son test change.
- **Mutations** : par l'outil `Edit`, défaites par `git checkout -- <fichier>` puis `touch` (mémoire « mutation restaurée, binaire
  périmé »).
- **Estimation** : lot A 1 h, lot B 1 h (captures comprises), lot C 2 h (dont 30 min de temps machine), gates et E2E 1 h 30 ; revue
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
