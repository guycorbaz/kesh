# Story 15.5c : Le rapprochement se lit — libellés des refus par lot et manuel du rapprochement

Status: ready-for-dev

<!-- Troisième sous-story de la 15-5, créée le 2026-10-08 à la passe de validation P2 de la 15-5b
     (finding F-3, choix C15 de `epic-15-choix-autonomes.md`). Elle reprend de la 15-5b le premier volet
     de son ancien AC14 (libellés de `failed[]`, choix C8) et la réécriture du manuel du rapprochement
     (choix C11), étendue aux passages faux relevés par le finding F-2 de la même passe, qui recoupent
     l'issue #481. Choix applicables : C8, C11, C15, C16 (forme du détail `rejected`), C32 (écran réel
     vérifié au code, `PERIOD_LOCKED` sans détail). Validation P1 faite (Change Log). -->

**Issues** : **ferme #481** (manuel du rapprochement : *Modifier*, lot « atomique », manuel et
éclatement « par facture »), **#492** (codes bruts des refus par lot, P3) et **#519** (manuel des règles
d'affectation). La PR porte `closes #481 closes #492 closes #519` (mots-clés **dans la PR**, le dépôt
merge en squash). Les trois points de #481 sont couverts (AC5, AC6, AC7) : rien n'en reste ouvert.

**Dépend de la 15-5b** (qui émet `ACCOUNT_NOT_POSTABLE` dans `failed[]`, avec
`details.rejected[{accountId, accountNumber}]`, et qui fixe le comportement des règles que le manuel
décrit : plus proposées, réactivation refusée). Ne pas commencer avant son merge.

## Story

En tant que **comptable qui rapproche ses relevés bancaires dans Kesh**,
je veux que **les refus d'une acceptation par lot me disent en clair pourquoi chaque proposition a
échoué, et que le manuel décrive le rapprochement tel que l'écran le fait**,
afin de **corriger ce qui bloque sans connaître les codes internes**, et de ne pas chercher un bouton,
une option ou un comportement qui n'existent pas.

### Pourquoi une story à part

Ces deux sujets étaient dans la 15-5b, dite « rollout » ; la validation P2 a relevé (F-3) qu'ils n'en
avaient rien de mécanique : un module neuf de libellés dans quatre locales, et la réécriture de toute une
partie du manuel. Ils ne touchent aucun code serveur et dépendent de ce que la 15-5b livre.

## Acceptance Criteria

### Les refus par lot, lisibles (#492)

1. **AC1 — Un libellé traduit par `errorCode`** pour **tous** les codes que `failed[]` peut porter.
   Nouveau module `frontend/src/lib/features/reconciliation/failed-proposal-label.ts`, sur le patron de
   `failedItemLabel` (`frontend/src/lib/features/payment-batches/payment-batch-helpers.ts:53-80`) :
   `switch` sur le code, **clés écrites en toutes lettres** (jamais construites par gabarit — elles
   doivent être vues par `i18n-keys.test.ts`), repli **avec le code brut** pour un code inconnu. Les
   codes, relevés sur `1920381e` par
   `grep -ohE 'error_code: "[A-Z_]+"' crates/kesh-api/src/routes/reconciliation.rs | sort -u` (25), plus
   `ACCOUNT_NOT_POSTABLE` (15-5b) : `ACCOUNT_NOT_FOUND`, `ACCOUNT_NOT_POSTABLE`,
   `BANK_ACCOUNT_NOT_CONFIGURED`, `BANK_ACCOUNT_NOT_FOUND`, `BANK_TRANSACTION_NOT_FOUND`,
   `DATABASE_ERROR`, `FISCAL_YEAR_INVALID`, `INTERNAL_ERROR`, `INVOICE_NOT_FOUND`,
   `INVOICE_SALE_ENTRY_MALFORMED`, `PERIOD_LOCKED`, `PROJECT_ARCHIVED`, `PROJECT_NOT_FOUND`,
   `RECONCILIATION_ALREADY_RECONCILED`, `RECONCILIATION_CURRENCY_MISMATCH`,
   `RECONCILIATION_FISCAL_YEAR_CLOSED`, `RECONCILIATION_INVOICE_NOT_ELIGIBLE`,
   `RECONCILIATION_OVERPAYMENT`, `RECONCILIATION_RULE_MISMATCH`, `RECONCILIATION_RULE_NO_LONGER_MATCHES`,
   `RECONCILIATION_RULE_NOT_FOUND`, `RECONCILIATION_SCORE_TOO_LOW`, `RECONCILIATION_SPLIT_IMBALANCE`,
   `RECONCILIATION_TRANSACTION_NOT_PENDING`, `ROUNDING_ACCOUNT_NOT_CONFIGURED`, `VALIDATION_ERROR`
   (**26**). Le dev **refait** la commande sur `HEAD` (après le merge de la 15-5b) et ajoute tout code
   apparu entre-temps. Les libellés, dans les **quatre** locales, sont **relevés** sur les messages
   existants quand il y en a (p. ex. `ROUNDING_ACCOUNT_NOT_CONFIGURED` reprend
   `error-rounding-account-not-configured`, qui dit où agir) — pas inventés.
2. **AC2 — `ACCOUNT_NOT_POSTABLE` nomme les comptes, sans se fier au type.** `FailedProposal.details`
   est typé `unknown | null` (`frontend/src/lib/features/reconciliation/reconciliation.types.ts:94`) :
   le module lit `details.rejected[].accountNumber` derrière une **garde de type** (objet, tableau,
   chaînes) ; un `details` absent ou d'une autre forme rend le libellé **sans** numéros, jamais une
   exception ni « undefined ». La forme `rejected[{accountId, accountNumber}]` est celle que la 15-5a a
   posée (choix C16).
3. **AC3 — La ligne de refus affiche le libellé.**
   `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte:362` (aujourd'hui
   `TX #{f.bankTransactionId} — {f.errorCode}`) affiche `TX #<id> — <libellé>` ; le code brut reste
   lisible dans un `title` ou entre parenthèses, pour le support. `ReconciliationProposals.test.ts`
   gagne **deux** tests : une réponse d'**acceptation** portant un `failed[]` → le libellé traduit est
   affiché (et non le code seul) ; une réponse de **rejet** (`onReject`, `:166-181`, qui alimente le
   même `failed`) portant un `failed[]` → idem (finding P1 F-5) — aujourd'hui aucun test n'exerce
   l'affichage de `failed[]`.
   **`PERIOD_LOCKED` : le libellé n'utilise pas `details`** (`lockedThrough`, `attempted`, posés par
   `crates/kesh-api/src/routes/reconciliation.rs:~200-215`) — limite **assumée** (choix C32) : le libellé
   dit que la période est verrouillée et renvoie à la clôture de période ; seul `ACCOUNT_NOT_POSTABLE`
   lit son détail (AC2). Le Dev Agent Record le redit.
4. **AC4 — Les clés existent dans les quatre locales et le compteur des sites i18n est relevé
   délibérément.** Clés neuves, préfixe `reconciliation-failed-`, dans les quatre
   `crates/kesh-i18n/locales/*/messages.ftl` ; **clés plates** (pas de sélecteur Fluent : le frontend
   lit le dictionnaire pré-résolu, qui figerait un sélecteur sur `*[other]` — cf.
   `crates/kesh-i18n/src/loader.rs`, `SELECTEURS_RESOLUS_COTE_SERVEUR`). `npm run lint-i18n-ownership`
   vert. `frontend/src/lib/shared/i18n-keys.test.ts` porte une **borne exacte** (`sitesTotal: 1868`,
   `:456`) : elle **rougira**, et c'est voulu — la relever à la main, du nombre de sites réellement
   ajoutés, en **recomptant la ventilation** (`litteraux`, `gabarits`, `nonResolus`) depuis le relevé
   du test, et l'écrire au Dev Agent Record. Ne pas la remplacer par une borne inférieure.

### Le manuel du rapprochement dit ce que fait l'écran (#481, #519)

`docs/manual/fr/user-manual.tex`, section *Réconciliation bancaire* (`:1491-1586`). Lignes relevées sur
`1920381e`, revérifiées en validation P1 sur `92770300` ; le dev les refait (T0). Le dev décrit **ce
qu'il voit** dans le code et à l'écran, pas ce que disent ces critères.

**L'écran réel**, relevé au code en validation P1 (`frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`,
choix C32) — c'est lui que les AC5 à AC7 décrivent :
- *Mensuel* → *Réconciliation* liste les transactions **en attente** du compte bancaire choisi, non
  rejetées (`find_pending_transactions_for_account`, `crates/kesh-db/src/repositories/reconciliation.rs:163-185`),
  colonnes *Date*, *Montant*, *Contrepartie*, *Candidate*, *Score* (`:226-238`) ;
- la colonne *Candidate* montre la **meilleure** candidate seulement : soit une **facture** (numéro, reste
  dû, et « reste dû sur » le total quand ils diffèrent, `:283-292`), soit une **règle** d'affectation,
  marquée d'un badge *Règle*, « libellé → compte » (`:265-282`) ; sans candidate, « Aucune
  correspondance » ;
- une **case à cocher** par ligne, présente seulement s'il y a une candidate (`:243-252`) ;
- au-dessus de la liste, deux boutons **de lot** qui agissent sur les lignes cochées : **« Accepter
  (N) »** (`:201-209`, N = nombre de lignes cochées) et **« Rejeter »** (`:210-218`) ; il n'y a **pas**
  de bouton *Accepter* / *Rejeter* par ligne, ni *Modifier*, ni *Accepter sélectionnées* ;
- par ligne, deux boutons : **« Affecter manuellement »** (`:302-312`) et **« Éclater »** (`:313-323`),
  présents sur toutes les lignes, avec ou sans candidate ;
- après un lot, un message « N opération(s) réussie(s) » et, s'il y a des refus, une liste *Échecs
  partiels* (`:352-366`).

5. **AC5 — *Acceptation des propositions* décrit l'écran réel** (#481, point 1 ; `:1513-1528`). Les
   sous-sections *Acceptation des propositions* et *Acceptation par lot* (`:1530-1532`) décrivent **une
   seule** interaction et sont **fusionnées** en une sous-section (p. ex. *Accepter ou rejeter les
   propositions*) qui dit, sur l'écran relevé ci-dessus :
   - ce que la page affiche — **pas** de liste de « factures impayées candidates » (`:1515-1518`) :
     une ligne par transaction en attente, avec sa meilleure candidate, facture **ou règle** ;
   - qu'on **coche** les transactions puis qu'on clique **« Accepter (N) »** ou **« Rejeter »** ;
     **« Accepter »** accepte, pour chaque ligne cochée, **la candidate affichée** (`candidates[0]`,
     `:128-153`) : une facture → l'écriture de règlement (Débit banque / Crédit débiteurs, et l'écart
     d'arrondi en troisième ligne s'il y a lieu, phrase actuelle conservée) ; une règle → l'écriture
     entre la banque et le compte de la règle. **« Rejeter »** marque les transactions cochées comme
     revues (`auto_match_rejected_at`, `crates/kesh-api/src/routes/reconciliation.rs:2600-2612`) : elles
     **quittent la liste**, qui n'affiche que les transactions non rejetées. ⚠️ Fait vérifié en P1 :
     aucun écran du frontend n'affiche une transaction rejetée (aucune lecture de
     `auto_match_rejected_at` dans `frontend/src`) — la phrase actuelle « La transaction reste à
     rapprocher manuellement » est donc **fausse** à l'écran ; le dev le confirme au navigateur et écrit
     ce qui est vrai (la transaction disparaît de la liste ; l'annulation d'un rapprochement remet à
     zéro ce marquage, `reconciliation_cancel.rs:331`) — **sans** promettre un chemin qui n'existe pas ;
   - plus d'item *Modifier* (`:1527`) : pour imputer autrement qu'à la candidate affichée, on utilise
     **« Affecter manuellement »** ou **« Éclater »** (AC7) ;
   - la puce *Accepter* (`:1524-1525`) ne cite plus le **code brut** `ROUNDING\_ACCOUNT\_NOT\_CONFIGURED` :
     elle dit que, sans compte de différences d'arrondi, la ligne apparaît dans les *Échecs partiels*
     avec le **libellé** de l'AC1 (relevé sur `error-rounding-account-not-configured`) et qu'on le
     désigne dans *Paramètres* → *Facturation* (finding P1 R-2/F-2).
6. **AC6 — Le lot : succès partiel, refus en clair** (#481, point 2 ; `:1532`, intégré à la
   sous-section fusionnée de l'AC5). Le texte actuel dit l'opération « atomique : soit toutes … soit
   aucune » et renvoie à « la doc CLAUDE.md projet » et à « failed[] » — **faux et hors de propos pour
   un utilisateur**. Le réécrire : les transactions acceptées le sont, les refusées sont listées sous
   *Échecs partiels* avec leur motif en clair (AC1–AC3) ; un refus n'empêche pas les autres ; il en va
   de même pour *Rejeter*.
7. **AC7 — *Réconciliation manuelle* et *Éclatement* : un compte de contrepartie, une seule écriture**
   (#481, point 3 ; `:1534-1546` et `:1548-1559`, légendes des deux `\keshscreenshot` comprises).
   - Rapprochement manuel : le bouton s'appelle **« Affecter manuellement »** (`:1541` dit *Rapprocher
     manuellement* — à corriger) ; il passe une écriture entre le compte de la banque et **un compte de
     contrepartie** que l'utilisateur choisit (`ManualMatchModal.svelte:157-160`) — il ne porte ni
     « facture impayée » ni « écriture existante », et ne crée pas d'écriture « à la volée » au sens du
     texte actuel. **Ce que propose le sélecteur, vérifié au code en P1** (les findings R-1 et F-1 se
     contredisaient ; les deux avaient raison en partie) : la modale réduit la liste aux comptes dont
     le numéro commence par **5, 6 ou 7** (`ManualMatchModal.svelte:65-69`), puis
     `AccountAutocomplete` ne garde que les comptes **actifs et imputables**
     (`frontend/src/lib/features/journal-entries/AccountAutocomplete.svelte:200-207`, sans compte
     exempté : la modale ne passe pas `postableExemptAccountId`). Le manuel dit donc : un compte des
     classes 5, 6 ou 7, actif et imputable — **sans** promettre davantage ; un compte non imputable
     envoyé autrement (intégration) est refusé (15-5b).
   - Éclatement : l'utilisateur répartit le montant en lignes, **un compte par ligne**
     (`TransactionSplitModal.svelte:265-268`), choisi dans **le même sélecteur** (classes 5, 6, 7 par
     préfixe, `:71-73`, puis actifs et imputables) ; le tout produit **une seule** écriture à
     plusieurs lignes (`reconciliation.rs:3619`), pas « N écritures » ; le cas d'usage « virement
     groupé de trois factures » est remplacé par un cas que l'écran sert réellement (p. ex. un
     prélèvement à ventiler entre plusieurs charges), ou réécrit sans promettre de régler des factures.
   - **Légendes des captures** (`:1536`, `:1548`) : « sélection explicite de la facture correspondant à
     une transaction » et « éclatement … en plusieurs écritures (paiement partiel, ventilation) » sont
     fausses → légendes réécrites sur ce que montre l'écran (choix d'un compte de contrepartie ;
     ventilation sur plusieurs comptes, une écriture).
8. **AC8 — *Règles d'affectation automatique* réécrite sur le comportement réel** (#519, choix C11 ;
   `:1561-1586`) :
   - accès *Administration* → *Règles d'affectation* (`/reconciliation/rules`,
     `frontend/src/routes/(app)/+layout.svelte:123`, groupe `administration`) — **le chemin actuel du
     manuel est juste et reste** ;
   - une règle = libellé, **type de correspondance** (`counterparty_contains`, `counterparty_exact`,
     `reference_contains`, `iban_exact`, `crates/kesh-db/src/entities/reconciliation_rule.rs`) et valeur,
     **compte de contrepartie** (charge ou produit, **imputable**), priorité, projet par défaut ;
   - elle produit une **proposition** dans l'écran de rapprochement, que l'utilisateur accepte — aucune
     écriture n'est créée à l'import, il n'existe ni brouillon ni option *auto-validate rules*, ni
     « acceptation par lot d'écritures en brouillon » (sous-section `:1584-1586` supprimée) ;
   - désactiver / réactiver ; **archiver une règle la désactive** (`UPDATE … active = FALSE`,
     `crates/kesh-db/src/repositories/reconciliation_rules.rs`, `soft_delete_by_id_for_company`) — la
     liste l'affiche « Archivée » (`RulesList.svelte:146-148`) : le manuel le dit, sans laisser croire à
     deux états distincts s'ils n'en sont qu'un ;
   - le cas de l'AC9 de la 15-5b : une règle dont le compte est devenu non imputable n'est plus
     proposée, se modifie, mais ne se réactive pas sans changer de compte ; **l'écran des règles
     l'affiche toujours « Active »** (sa colonne d'état ne connaît que *Active* / *Archivée*,
     `RulesList.svelte:146-148`) — le manuel le dit, pour qu'un utilisateur ne cherche pas pourquoi une
     règle « Active » ne propose rien (finding P1 R-4) ;
   - **l'exemple et les conditions fictifs** (`:1565`, `:1572-1573`) disparaissent : « crée
     automatiquement l'écriture Débit 6000 / Crédit 1020 », « montant entre 2'400 et 2'600 », « compte
     tiers IBAN = … », et l'**action** « créer écriture Débit compte X / Crédit compte Y » n'existent
     pas. Une règle a **une** condition, d'un des quatre types réels (`counterparty_contains`,
     `counterparty_exact`, `reference_contains`, `iban_exact`, `reconciliation_rule.rs:52-55`), avec les
     libellés que montre le formulaire (`RuleFormModal.svelte:141-176`), et **un** compte de
     contrepartie ; l'exemple du loyer est réécrit sur ces termes (p. ex. contrepartie contient
     « Régie » → compte de loyer) ; la légende de la capture (`:1563`) aussi.
   Le dev décrit **ce qu'il voit** dans `RuleFormModal.svelte`, `RulesList.svelte` et `get_proposals`.
9. **AC9 — FAQ, PDF, CHANGELOG.**
   - FAQ *Une transaction bancaire ne propose aucun match automatique* (`:2095-2105`) : ajouter le cas
     d'une **règle écartée** — son compte est archivé ou non imputable, elle n'est plus proposée.
     **Les trois puces existantes sont confrontées au code en validation P1** (finding F-4) et
     réécrites sur ce qui suit :
     - « à 1 CHF près » est **faux** : une facture n'est candidate que si son **reste dû** égale le
       montant de la transaction **à 5 centimes près** (`AMOUNT_TOLERANCE_HUNDREDTHS = 5`,
       `crates/kesh-api/src/routes/reconciliation.rs:58-61`, appliqué `:500-521`), dans une fenêtre de
       **30 jours** (`WINDOW_DAYS`, `:54-56`) — ce que dit déjà l'algorithme (`:1501`) ; et seulement
       pour une transaction **créditrice en CHF** (`:503-512`) ;
     - « référence QR Bill structurée » est **faux** : le score de référence compare la référence de la
       transaction au **numéro de la facture** (`crates/kesh-reconciliation/src/matching.rs:13-16`) ;
     - « libellé trop vague » : le critère réel est le **nom de la contrepartie** comparé au nom du
       client (10 %, `matching.rs:17-19`) ; et une candidate de score nul n'est pas proposée ;
     - une **règle** n'est proposée que si aucune facture n'atteint un score de 0,5
       (`INVOICE_OVERRIDE_THRESHOLD`, `reconciliation.rs:563`, `:588`) — à dire si la FAQ parle des
       règles.
   - Le PDF est régénéré (`latexmk -xelatex` dans `docs/manual/fr/`), commité, et **contrôlé aplati**
     (`pdftotext … - | tr '\n' ' ' | tr -s ' '`) : les phrases retirées sont absentes, les nouvelles
     présentes.
   - `docs/manual/fr/admin-manual.tex` : contrôle **sans objet** attendu (aucune description du
     rapprochement) — le refaire et l'écrire au Dev Agent Record.
   - `CHANGELOG.md`, section `## [0.13.0] — Non publié` (créée par la 15-5a ; **la créer en tête si
     absente** — motif exact exigé par `scripts/prepare-release.sh:189`), rubrique **Corrigé** : les refus d'une
     acceptation par lot s'affichent en clair (#492) ; le manuel du rapprochement décrivait un bouton,
     une opération atomique, un rapprochement par facture et un écran de règles qui n'existent pas
     (#481, #519) — même rubrique, le CHANGELOG n'ayant pas de rubrique *Documentation*.
   - **Autres passages faux relevés en P1** (finding R-3), à réécrire avec la section : l'introduction
     (`:1493`, « rapprocher … avec les écritures comptables ou les factures ») — Kesh ne rapproche pas
     une transaction d'une écriture existante : il **crée** l'écriture, en réglant une facture ou en
     imputant un compte (manuellement, par éclatement ou par règle).

## Tasks / Subtasks

- [ ] **T0 — Refaire les relevés** sur `HEAD`, après le merge de la 15-5b : la liste des codes de
      l'AC1 (commande citée) et les numéros de ligne du manuel (AC5–AC9). Tout code ou passage neuf est
      classé au Change Log.
- [ ] **T1 — Le module de libellés** (AC1, AC2) : `failed-proposal-label.ts` et
      `failed-proposal-label.test.ts` — chaque code de la liste, **écrite en dur dans le test**, rend un
      libellé distinct du repli ; `ACCOUNT_NOT_POSTABLE` nomme les numéros de `details.rejected` ; un
      `details` absent, `null`, ou d'une autre forme → libellé sans numéros ; un code inconnu → repli
      avec le code.
- [ ] **T2 — Les clés** (AC4) : quatre `messages.ftl` ; `lint-i18n-ownership` ; borne `sitesTotal`
      relevée délibérément, ventilation recomptée ; `cargo test -p kesh-i18n` (les tests du chargeur
      lisent les `.ftl`).
- [ ] **T3 — L'affichage** (AC3) : `ReconciliationProposals.svelte:362` ; test de
      `ReconciliationProposals.test.ts`.
- [ ] **T4 — Le manuel** (AC5–AC9) : l'introduction (`:1493`), la sous-section fusionnée
      *Accepter ou rejeter les propositions* (ex-*Acceptation des propositions* et *Acceptation par
      lot*), le rapprochement manuel, l'éclatement, les règles et la FAQ, **légendes des captures
      comprises** ; **relire au code** le filtre des deux sélecteurs (`ManualMatchModal.svelte:65-69`,
      `TransactionSplitModal.svelte:71-73`, `AccountAutocomplete.svelte:200-207`) et le sort d'une
      transaction rejetée (AC5) avant d'écrire ; PDF régénéré, commité, contrôlé aplati ;
      `admin-manual.tex` contrôlé.
- [ ] **T5 — Propagation** (règle *Propagation post-patch*) :
      `grep -rnE "auto-validate|atomique|CLAUDE\.md|Modifier.*facture|N écritures|par facture|brouillon|Rapprocher manuellement|sélectionnées|candidates|1 CHF près|QR Bill structurée|ROUNDING.ACCOUNT|texttt\{[A-Z]+.?_[A-Z]" docs/manual/fr/*.tex`
      et le même motif sur le PDF aplati (où le `\_` devient `_`) ; chaque occurrence qui décrit le
      rapprochement est réécrite ou justifiée au Dev Agent Record — en particulier tout **code brut**
      d'erreur cité au manuel utilisateur (finding P1 R-2). `CHANGELOG.md` (AC9).
- [ ] **T6 — Gates** : gate frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`,
      `build`) ; `cargo test -p kesh-i18n` ; gate backend complet avant le push (règle *Test Locally
      First*) ; **E2E Playwright complet au dernier commit de code** (décision D7), jugé fichier par
      fichier contre `docs/testing.md` § « Les échecs attendus ».

*(Décompte : 9 AC, 7 tâches T0–T6.)*

## Dev Notes

### Ce qui doit être préservé

- Le pattern batch (§ *Pattern batch — FailedProposal* du `CLAUDE.md`) : cette story n'en change rien
  côté serveur ; elle ne fait qu'afficher `failed[]`.
- La règle du manuel : décrire l'écran réel ; une fonction absente n'est pas « à venir » dans le texte
  utilisateur.
- Un code d'erreur **brut** (`ROUNDING_ACCOUNT_NOT_CONFIGURED`, …) n'a pas sa place au manuel
  utilisateur : on cite le libellé affiché et l'endroit où agir.

### Hors périmètre, et écrit

- Le **détail** de `PERIOD_LOCKED` (`lockedThrough`, `attempted`) n'est pas lu par le libellé (C32) :
  le libellé reste générique. Les autres codes non plus, sauf `ACCOUNT_NOT_POSTABLE` (AC2).
- Le sort d'une transaction **rejetée**, qu'aucun écran ne montre plus (AC5) : la story **écrit** ce
  fait au manuel ; elle ne crée pas d'écran pour la retrouver (signalé à l'orchestrateur pour une
  issue).

### Décisions consignées (registre `epic-15-choix-autonomes.md`)

- **C8** — libellés traduits pour **tous** les codes de `failed[]` ; ferme #492.
- **C11** — réécriture du § *Règles d'affectation automatique* ; ferme #519.
- **C15** — la création de cette story ; ferme aussi #481.
- **C16** — la forme `details.rejected[{accountId, accountNumber}]` qu'AC2 lit.
- **C32** — l'écran réel du rapprochement et le filtre réel des sélecteurs, vérifiés au code en P1 ;
  `PERIOD_LOCKED` affiché sans son détail.

### Fichiers touchés (prévision)

`frontend/src/lib/features/reconciliation/{failed-proposal-label.ts,failed-proposal-label.test.ts,ReconciliationProposals.svelte,ReconciliationProposals.test.ts}`,
`frontend/src/lib/shared/i18n-keys.test.ts` (borne), `crates/kesh-i18n/locales/*/messages.ftl`,
`docs/manual/fr/user-manual.{tex,pdf}`, `CHANGELOG.md`. Aucun code serveur, **aucune migration**.
Modules : `frontend/reconciliation`, `kesh-i18n` (locales), manuel — sous le seuil de la règle de
splitting.

### Tests — ce qui rendrait un test vert sans rien prouver

- Un test de libellés qui itère sur la table de libellés elle-même : vert par construction. Lister les
  codes **en dur**.
- Un test qui vérifie seulement qu'un libellé « existe » : le repli avec le code brut existe toujours.
  Asserter qu'il **diffère** du repli.
- Un contrôle du manuel au seul `.tex` : `pdftotext` coupe les lignes ; aplatir avant de greper.

### References

- Issues : #481, #492, #519 ; #427, #429 (15-5b).
- Fiches : `15-5-gardes-postabilite-serveur.md` (mère, `split`), `15-5a-refus-non-imputable.md`,
  `15-5b-gardes-surfaces-neuves.md`.
- `CLAUDE.md` : § *Le prompt d'une passe doit NOMMER le manuel*, § *Pattern batch*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-08 — **Créée à la passe de validation P2 de la 15-5b** (choix C15), sur les findings F-3
  (MEDIUM : la 15-5b n'était pas un rollout mécanique) et F-2 (MEDIUM : manuel faux sur le
  rapprochement manuel, l'éclatement, le bouton *Modifier* et la FAQ) de la lentille F, et sur les
  findings de la lentille R qui portaient sur ces objets : R-2 (le chemin *Administration → Règles
  d'affectation* du manuel est **juste** — la remédiation P1 l'avait déclaré faux), R-6 c (aucun test
  de l'affichage de `failed[]`), R-6 d (`details` typé `unknown`, garde de type), R-6 e = F-9 (archiver
  = désactiver), et F-7 (borne `sitesTotal`). Reprend de la 15-5b l'ancien AC14 (premier volet, choix
  C8) et la réécriture des passages *Acceptation par lot* et *Règles d'affectation* de l'ancien AC17
  (choix C11) ; y ajoute #481, dont les trois points sont couverts. Validation : à lancer.
- 2026-10-08 — **Passe de validation P1** (prompt versionné `15-5c-validate-prompt-p1.md` ; deux
  lentilles **Sonnet** en contexte frais : **R** chasseur de régressions (la fiche née en P2 de la
  15-5b, `92770300`), **F** adversaire de périmètre complet). **0 CRITICAL, 0 HIGH.** Bruts : R 2 MEDIUM
  + 3 LOW, F 2 MEDIUM + 4 LOW. Doublon inter-lentilles : F-2 = R-2 → **3 MEDIUM et 7 LOW distincts**.
  Trend : première passe de cette fiche.

  | finding | sév. | objet | sort |
  |---|---|---|---|
  | R-1 | MEDIUM | AC7 : « compte proposé actif et imputable » — contredit par F, qui le tenait d'`AccountAutocomplete` | **vérifié au code par l'orchestrateur** : les deux filtres s'appliquent (préfixe 5/6/7 dans les modales, puis `active && postable` dans `AccountAutocomplete`) ; AC7 et T4 écrivent les deux (C32) |
  | R-2 = F-2 | MEDIUM | `user-manual.tex:1524-1525` cite le code brut `ROUNDING_ACCOUNT_NOT_CONFIGURED` | AC5 : libellé affiché et lieu où agir ; T5 grep des codes bruts du manuel |
  | F-1 | MEDIUM | boutons fictifs : *Accepter* / *Rejeter* par ligne, *Accepter sélectionnées*, *Rapprocher manuellement* ; « factures impayées candidates » ; candidat règle ignoré | relevé de l'écran réel en tête des AC du manuel ; AC5 fusionne *Acceptation des propositions* et *par lot* ; AC7 « Affecter manuellement » ; T5 grep (C32) |
  | R-3 | LOW | passages faux hors des plages : `:1493`, exemple et conditions du loyer `:1565-1573`, légendes des captures | AC7 (légendes), AC8 (exemple, types réels), AC9 (`:1493`) |
  | R-4 | LOW | l'écran des règles affiche toujours « Active » une règle active sur compte non imputable | AC8 |
  | R-5 | LOW | `sitesTotal` : remarque sans correction | sans objet |
  | F-3 | LOW | CHANGELOG : section `[0.13.0]` peut-être absente | AC9 : « la créer en tête si absente » |
  | F-4 | LOW | FAQ « à 1 CHF près » contre « 5 centimes » de l'algorithme | **confronté au code** : 5 centimes, 30 jours, transaction créditrice en CHF, référence comparée au numéro de facture, règle seulement sous un score de 0,5 ; AC9 écrit le vrai |
  | F-5 | LOW | `failed[]` aussi alimenté par *Rejeter* | AC3 : second test, côté rejet |
  | F-6 | LOW | `PERIOD_LOCKED` porte un `details` que le libellé ne lit pas | assumé et écrit (AC3, Dev Notes, C32) |

  **Trouvé pendant la remédiation** (vérification de la puce *Rejeter*) : une transaction rejetée
  quitte la liste (`auto_match_rejected_at IS NULL`, `kesh-db/src/repositories/reconciliation.rs:163-185`)
  et **aucun écran** ne la montre plus — « la transaction reste à rapprocher manuellement » est faux ;
  AC5 et *Hors périmètre* l'écrivent ; signalé à l'orchestrateur pour une issue.
  **Signal de la règle de découpage** : sans objet (première passe ; trois modules). **Décision** :
  C32. **Propagation post-patch** : `Rapprocher manuellement`, `sélectionnées`, `candidates`,
  `1 CHF`, `ROUNDING`, `actif et imputable`, `Modifier` grepés sur les fiches 15-5, 15-5a, 15-5b, 15-5c
  et le registre. Décompte inchangé : **9 AC, 7 tâches T0–T6** (recompté). **Une passe P2 suit** (des
  MEDIUM en P1).
