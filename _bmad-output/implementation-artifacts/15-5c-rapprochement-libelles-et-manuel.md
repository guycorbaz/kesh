# Story 15.5c : Le rapprochement se lit — libellés des refus par lot et manuel du rapprochement

Status: done

<!-- Troisième sous-story de la 15-5, créée le 2026-10-08 à la passe de validation P2 de la 15-5b
     (finding F-3, choix C15 de `epic-15-choix-autonomes.md`). Elle reprend de la 15-5b le premier volet
     de son ancien AC14 (libellés de `failed[]`, choix C8) et la réécriture du manuel du rapprochement
     (choix C11), étendue aux passages faux relevés par le finding F-2 de la même passe, qui recoupent
     l'issue #481. Choix applicables : C8, C11, C15, C16 (forme du détail `rejected`), C32 (écran réel
     vérifié au code, `PERIOD_LOCKED` sans détail ; corrigé par la P2 sur la candidate affichée et le
     sort d'une transaction rejetée), C37 (réutilisation des clés `error-*`), C38 (échecs partiels
     toujours visibles, ligne désignée par sa date et son montant). Validations P1 et P2 faites
     (Change Log). -->

**Issues** : **ferme #481** (manuel du rapprochement : *Modifier*, lot « atomique », manuel et
éclatement « par facture »), **#492** (codes bruts des refus par lot, P3) et **#519** (manuel des règles
d'affectation). La PR porte `closes #481 closes #492 closes #519` (mots-clés **dans la PR**, le dépôt
merge en squash). Les trois points de #481 sont couverts (AC5, AC6, AC7) : rien n'en reste ouvert.
Le manuel **cite sans les corriger** trois défauts voisins, ouverts par l'orchestrateur : **#526**
(une transaction rejetée n'est plus atteignable depuis l'interface), **#527** (une facture de score
faible masque une règle de score maximal) et **#529** (un paiement client reçu plus de 30 jours après
la date de facture n'est jamais proposé) — `refs #526 refs #527 refs #529` dans la PR.

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
   existants quand il y en a — pas inventés —, selon la règle suivante (choix C37, findings R-7 et
   F-13) :
   - quand une clé `error-*` existante convient **mot pour mot** et **sans variable**, le module la
     **lit directement** (l'espace `error-` est global, `frontend/scripts/lint-i18n-ownership.js`,
     `GLOBAL_NAMESPACES`) au lieu d'en dupliquer la traduction : c'est le cas de
     `ROUNDING_ACCOUNT_NOT_CONFIGURED` → `error-rounding-account-not-configured`
     (`crates/kesh-i18n/locales/fr-CH/messages.ftl:275`, « Ce paiement solde la facture au centime, mais
     aucun compte de différences d'arrondi utilisable n'est désigné : choisissez-en un dans Paramètres →
     Facturation. ») ;
   - quand le message serveur porte une **variable** que le client n'a pas (p. ex.
     `error-fiscal-year-closed`, `{ $date }`, `messages.ftl:235`), ou ne convient pas tel quel : clé
     neuve `reconciliation-failed-*`, texte **sans** la variable (ou avec une valeur lue de `details`,
     ce que seule l'AC2 fait) ;
   - chaque site, clé réutilisée ou neuve, compte dans la borne `sitesTotal` (AC4) ; le Dev Agent
     Record dit, code par code, quelle clé est lue.
   **Contrôle des formes non littérales** (finding R-8, règle *Inventorier les sites NON RÉSOLUS*) : la
   commande ne voit que les littéraux ; le dev exécute aussi
   `grep -nE 'error_code:' crates/kesh-api/src/routes/reconciliation.rs | grep -vE 'error_code: "[A-Z_]+"'`
   — qui ne doit rendre que la définition du champ dans la structure `FailedProposal` — et relit la
   conversion `DbError → FailedProposal` de ce fichier ; tout code posé par une constante ou par
   `DbError::error_code()` (forme de la 15-5a et de la 15-5b pour `ACCOUNT_NOT_POSTABLE`) est ajouté à la
   liste.
2. **AC2 — `ACCOUNT_NOT_POSTABLE` nomme les comptes, sans se fier au type.** `FailedProposal.details`
   est typé `unknown | null` (`frontend/src/lib/features/reconciliation/reconciliation.types.ts:94`) :
   le module lit `details.rejected[].accountNumber` derrière une **garde de type** (objet, tableau,
   chaînes) ; un `details` absent ou d'une autre forme rend le libellé **sans** numéros, jamais une
   exception ni « undefined ». La forme `rejected[{accountId, accountNumber}]` est celle que la 15-5a a
   posée (choix C16).
3. **AC3 — La ligne de refus affiche le libellé, désigne la transaction, et reste visible** (choix
   C38).
   - `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte:362` (aujourd'hui
     `TX #{f.bankTransactionId} — {f.errorCode}`) affiche **la date, le montant et la contrepartie** de
     la transaction, puis le libellé. L'identifiant n'est affiché nulle part dans le tableau (seulement
     en `data-tx-id`) : `TX #<id>` ne désigne rien que l'utilisateur voie (finding F-6). Ces trois
     valeurs sont relevées dans `proposals` **avant** le `load()` qui suit le lot (la transaction
     acceptée ou rejetée n'y sera plus) ; une transaction introuvable dans ce relevé garde `TX #<id>`
     en repli. Le code brut reste lisible dans un `title` ou entre parenthèses, pour le support.
   - le bloc *Échecs partiels* et le compteur de succès sortent de la branche `{:else}` de
     `proposals.length === 0` (`:192-199`, `:354-367`) : ils s'affichent **même quand la liste est
     vide** après le lot (finding F-5) — c'est le cas de tous les refus de *Rejeter*
     (`BANK_TRANSACTION_NOT_FOUND`, `RECONCILIATION_ALREADY_RECONCILED`, `reconciliation.rs:2789-2820`),
     qui portent sur une transaction sortie de la liste.
   - `ReconciliationProposals.test.ts` gagne **trois** tests : une réponse d'**acceptation** portant un
     `failed[]` → le libellé traduit et la date/le montant de la transaction sont affichés (et non le
     code seul) ; une réponse de **rejet** (`onReject`, `:166-181`, qui alimente le même `failed`)
     portant un `failed[]` → idem (finding P1 F-5) ; un lot dont le **second `getProposals` rend `[]`**
     → les *Échecs partiels* restent affichés (un `getProposals` mocké à valeur constante ne le verrait
     pas). Aujourd'hui aucun test n'exerce l'affichage de `failed[]`.
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
   ajoutés, en **recomptant la ventilation** — les champs réels sont `sitesTotal`, `sitesNonResolus`,
   `relais`, `sitesGabarit`, `litterauxMin` et `clesDepuisTsMin` (`:456-461`, finding F-12) — depuis le
   relevé du test, et l'écrire au Dev Agent Record. Ne pas la remplacer par une borne inférieure. La
   15-5d relève aussi cette borne : celle des deux qui merge en second la relève sur l'état rebasé.

### Le manuel du rapprochement dit ce que fait l'écran (#481, #519)

`docs/manual/fr/user-manual.tex`, section *Réconciliation bancaire* (`:1491-1586`). Lignes relevées sur
`1920381e`, revérifiées en validation P1 sur `92770300` et **corrigées en P2 sur `007c4eb1`** (finding
R-4) ; le dev les refait (T0). Le dev décrit **ce
qu'il voit** dans le code et à l'écran, pas ce que disent ces critères.

**L'écran réel**, relevé au code en validation P1 (`frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`,
choix C32) — c'est lui que les AC5 à AC7 décrivent :
- *Mensuel* → *Réconciliation* liste les transactions **en attente** du compte bancaire choisi, non
  rejetées (`find_pending_transactions_for_account`, `crates/kesh-db/src/repositories/reconciliation.rs:163-185`),
  **les 100 plus récentes au plus** (`getProposals`, `limit = 100`,
  `frontend/src/lib/features/reconciliation/reconciliation.api.ts:16-22` ; tri `booking_date DESC, id
  DESC`, `:166` du dépôt ; `hasMore` est typé mais aucun composant ne le lit — finding F-11), colonnes
  *Date*, *Montant*, *Contrepartie*, *Candidate*, *Score* (`:226-238`) ;
- la colonne *Candidate* montre la **première** candidate de la réponse, `candidates[0]` (`:137`,
  `:265`) — **pas** « la meilleure » (findings R-1/F-2 de la P2) : le serveur range d'abord les
  **factures**, par score décroissant, et ajoute la **règle** après elles sans retrier
  (`crates/kesh-api/src/routes/reconciliation.rs:586-626`, `:631-664`). À l'écran, la candidate est donc
  la **facture de meilleur score** s'il y en a une (de score non nul) ; une **règle** n'apparaît que
  s'il n'y a **aucune** facture candidate — même quand la règle, de score 1,0, l'emporterait sur une
  facture faible (#527, non corrigée ici). Une facture s'affiche avec son numéro, son reste dû, et
  « reste dû sur » le total quand ils diffèrent (`:283-292`) ; une règle, avec un badge *Règle*,
  « libellé → compte » (`:265-282`) ; sans candidate, « Aucune correspondance » ;
- une **case à cocher** par ligne, présente seulement s'il y a une candidate (`:243-252`) ;
- au-dessus de la liste, deux boutons **de lot** qui agissent sur les lignes cochées : **« Accepter
  (N) »** (`:201-209`, N = nombre de lignes cochées) et **« Rejeter »** (`:210-218`) ; il n'y a **pas**
  de bouton *Accepter* / *Rejeter* par ligne, ni *Modifier*, ni *Accepter sélectionnées* ;
- par ligne, deux boutons : **« Affecter manuellement »** (`:302-312`) et **« Éclater »** (`:313-323`),
  présents sur toutes les lignes, avec ou sans candidate ;
- après un lot, un message « N opération(s) réussie(s) » et, s'il y a des refus, une liste *Échecs
  partiels* (`:354-367`) — **aujourd'hui dans la branche `{:else}`** de `proposals.length === 0`
  (`:192-199`) : si le lot vide la liste, ni le compteur ni les refus ne s'affichent (finding F-5 ;
  corrigé par l'AC3).

5. **AC5 — *Acceptation des propositions* décrit l'écran réel** (#481, point 1 ; `:1509-1528`). Les
   sous-sections *Acceptation des propositions* et *Acceptation par lot* (`:1530-1532`) décrivent **une
   seule** interaction et sont **fusionnées** en une sous-section (p. ex. *Accepter ou rejeter les
   propositions*) qui dit, sur l'écran relevé ci-dessus :
   - ce que la page affiche — **pas** de liste de « factures impayées candidates » (`:1517-1520`, puce
     `:1519`) : une ligne par transaction en attente (les 100 plus récentes au plus), avec **une**
     candidate, la **première** que propose Kesh — la facture la mieux notée s'il y en a une, une règle
     seulement s'il n'y a aucune facture candidate ; la phrase `:1511` (« La meilleure candidate est
     affichée ») est **réécrite** en ce sens. Le manuel ne promet pas que la règle l'emporte sur une
     facture faible (#527) ;
   - qu'on **coche** les transactions puis qu'on clique **« Accepter (N) »** ou **« Rejeter »** ;
     **« Accepter »** accepte, pour chaque ligne cochée, **la candidate affichée** (`candidates[0]`,
     `:128-153`) : une facture → l'écriture de règlement (Débit banque / Crédit débiteurs, et l'écart
     d'arrondi en troisième ligne s'il y a lieu, phrase actuelle conservée) ; une règle → l'écriture
     entre la banque et le compte de la règle. **« Rejeter »** (puce `:1526`) marque les transactions
     cochées comme revues (`auto_match_rejected_at`, `crates/kesh-api/src/routes/reconciliation.rs:2600-2612`)
     sans changer leur statut, qui reste `pending`. Ce qui est vrai, et que le manuel écrit (findings
     R-2/F-1 de la P2, qui corrigent le relevé de la P1) :
     - la transaction **quitte la liste** de *Réconciliation*, qui n'affiche que les transactions non
       rejetées (`find_pending_transactions_for_account`, `auto_match_rejected_at IS NULL`) ;
     - elle **reste visible** dans le **détail de son import** (*Import bancaire* → l'import),
       `frontend/src/routes/(app)/bank-import/[id]/+page.svelte:110-117`, qui liste **toutes** les
       transactions de l'import (`bank_transactions::list_by_import`,
       `crates/kesh-db/src/repositories/bank_transactions.rs:49-58`) — avec le statut brut `pending`,
       sans marque de rejet et **sans action** ;
     - **aucun geste de l'interface** ne la rapproche ensuite : ses boutons vivaient dans la liste qui
       l'exclut ; l'**annulation d'un rapprochement ne s'y applique pas** — elle ne porte que sur une
       transaction **rapprochée** (`crates/kesh-db/src/repositories/reconciliation_cancel.rs:285`,
       `:329-333`, `status = 'reconciled'`), or une transaction rejetée ne l'est pas ;
     - la phrase actuelle « La transaction reste à rapprocher manuellement » est donc **fausse** ; le
       manuel ne mentionne **aucun** chemin de retour, et le dev ne cite pas l'annulation (#526) ;
   - plus d'item *Modifier* (`:1527`) : pour imputer autrement qu'à la candidate affichée, on utilise
     **« Affecter manuellement »** ou **« Éclater »** (AC7) ;
   - la puce *Accepter* (`:1525`) ne cite plus le **code brut** `ROUNDING\_ACCOUNT\_NOT\_CONFIGURED` :
     elle dit que, sans compte de différences d'arrondi, la ligne apparaît dans les *Échecs partiels*
     avec le **libellé** de l'AC1 (la clé `error-rounding-account-not-configured`, réutilisée — C37) et
     qu'on le désigne dans *Paramètres* → *Facturation* (finding P1 R-2/F-2).
6. **AC6 — Le lot : succès partiel, refus en clair** (#481, point 2 ; `:1532`, intégré à la
   sous-section fusionnée de l'AC5). Le texte actuel dit l'opération « atomique : soit toutes … soit
   aucune » et renvoie à « la doc CLAUDE.md projet » et à « failed[] » — **faux et hors de propos pour
   un utilisateur**. Le réécrire : les transactions acceptées le sont, les refusées sont listées sous
   *Échecs partiels* avec leur motif en clair (AC1–AC3) ; un refus n'empêche pas les autres ; il en va
   de même pour *Rejeter*.
7. **AC7 — *Réconciliation manuelle* et *Éclatement* : un compte de contrepartie, une seule écriture**
   (#481, point 3 ; `:1534-1545` et `:1546-1559`, légendes des deux `\keshscreenshot` comprises).
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
   - l'**ordre** des règles : elles sont évaluées par **priorité croissante** — le **plus petit**
     nombre l'emporte —, puis par ancienneté, et **seule la première** qui correspond est proposée
     (`ORDER BY priority ASC, id ASC`, `crates/kesh-db/src/repositories/reconciliation_rules.rs:83` ;
     `crates/kesh-reconciliation/src/rules.rs:76-95`) ; le **type de correspondance** ne se choisit qu'à
     la **création** et ne se modifie pas ensuite (`RuleFormModal.svelte:138`, `{#if !isEdit}`)
     (finding F-8) ;
   - désactiver / réactiver ; **archiver une règle la désactive** (`UPDATE … active = FALSE`,
     `crates/kesh-db/src/repositories/reconciliation_rules.rs`, `soft_delete_by_id_for_company`) — la
     liste l'affiche « Archivée » (`RulesList.svelte:146-148`) : le manuel le dit, sans laisser croire à
     deux états distincts s'ils n'en sont qu'un ;
   - le cas de l'AC9 de la 15-5b : une règle dont le compte est devenu non imputable n'est plus
     proposée, se modifie, mais ne se réactive pas sans changer de compte ; **l'écran des règles
     l'affiche toujours « Active »** (sa colonne d'état ne connaît que *Active* / *Archivée*,
     `RulesList.svelte:146-148`) — le manuel le dit, pour qu'un utilisateur ne cherche pas pourquoi une
     règle « Active » ne propose rien (finding P1 R-4) ;
   - **l'exemple et les conditions fictifs** (`:1565` « automatiser la création d'écritures »,
     l'exemple du loyer `:1567`, conditions et action `:1574-1575`) disparaissent : « crée
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
       **30 jours de part et d'autre de la date de la facture** — **pas** de son échéance
       (`WINDOW_DAYS`, `:54-56` ; `i.date BETWEEN DATE_SUB(?, INTERVAL ? DAY) AND DATE_ADD(…)`,
       `crates/kesh-db/src/repositories/reconciliation.rs:135`, finding F-3) : une facture à 30 jours
       payée avec retard, ou après un rappel, **n'est pas proposée** (#529) ; et seulement pour une
       transaction **créditrice en CHF** (`:503-512`) ;
     - « référence QR Bill structurée » est **faux** : le score de référence compare la référence de la
       transaction au **numéro de la facture** (`crates/kesh-reconciliation/src/matching.rs:13-16`) ;
     - « libellé trop vague » : le critère réel est le **nom de la contrepartie** comparé au nom du
       client (10 %, `matching.rs:17-19`) ; et une candidate de score nul n'est pas proposée ;
     - si la FAQ parle des règles, elle dit ce qui vaut **à l'écran** : une règle n'est **affichée**
       que s'il n'y a **aucune** facture candidate (findings R-1/F-2 ; #527). Le seuil de 0,5
       (`INVOICE_OVERRIDE_THRESHOLD`, `reconciliation.rs:563`, `:588`) ne vaut que pour la réponse de
       l'API ; il n'a pas sa place au manuel utilisateur ;
     - **la phrase finale** (`:2105`, « Procédez à un rapprochement manuel, ou créez une règle
       d'affectation pour les transactions récurrentes »), qu'aucune passe n'avait inventoriée, est
       **réécrite** (finding F-3) : pour un **paiement de client** non proposé, la facture se règle
       **depuis sa fiche** (*Enregistrer un règlement*, § `sec:reglement-client`) — **ni** par
       *Affecter manuellement* ou *Éclater*, qui ne proposent que les classes 5, 6 et 7 (le compte
       débiteurs y est inatteignable, la facture resterait ouverte), **ni** par une **règle
       d'affectation**, qui ne vise qu'un compte de charge ou de produit (`RuleFormModal.svelte:43-50`) et
       porterait le paiement en produit : **produit compté deux fois**, facture toujours due — un faux
       rattachement muet. Le manuel le dit en ces termes et **ne conseille pas** de règle pour un
       paiement de client. Ce que devient alors la transaction bancaire (elle reste dans la liste, sans
       lien avec le règlement saisi) est dit tel quel, sans promettre de lien (#529) ; la règle
       d'affectation reste conseillée pour les **charges** récurrentes (loyer, abonnements).
   - Le PDF est régénéré (`latexmk -xelatex` dans `docs/manual/fr/`), commité, et **contrôlé aplati en
     normalisant les ligatures** (finding F-7) :
     `pdftotext -nopgbrk docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g'`
     — le corps du PDF rend « ff » par la ligature `ﬀ` (59 occurrences sur `007c4eb1`, « La meilleure
     candidate est aﬀichée ») : sans normalisation, un contrôle de **présence** de « Affecter
     manuellement », « différences d'arrondi » ou « affiche » rend un faux négatif. Les phrases retirées
     sont absentes, les nouvelles présentes.
   - `docs/manual/fr/admin-manual.tex` : **pas sans objet** (findings R-6/F-9) — `:82` (liste des
     fonctions, « Réconciliation automatique et manuelle … avec règles d'affectation ») et `:2033` (un
     paiement « règlement manuel ou rapprochement bancaire » refusé sans compte d'arrondi, « avec un
     message qui renvoie à *Paramètres* → *Facturation* ») mentionnent le rapprochement. Attendu au Dev
     Agent Record : `:82` reste vrai ; `:2033` **devient exact par l'AC3** — aujourd'hui, au lot,
     l'écran affiche le code brut (`ReconciliationProposals.svelte:362`).
   - `CHANGELOG.md`, section `## [0.13.0] — Non publié` (créée par la 15-5a ; **la créer en tête si
     absente** — motif exact exigé par `scripts/prepare-release.sh:189`), rubrique **Corrigé** : les refus d'une
     acceptation par lot s'affichent en clair (#492) ; le manuel du rapprochement décrivait un bouton,
     une opération atomique, un rapprochement par facture et un écran de règles qui n'existent pas
     (#481, #519) — même rubrique, le CHANGELOG n'ayant pas de rubrique *Documentation*.
   - **Autres passages faux relevés en P1** (finding R-3), à réécrire avec la section : l'introduction
     (`:1493`, « rapprocher … avec les écritures comptables ou les factures ») — Kesh ne rapproche pas
     une transaction d'une écriture existante : il **crée** l'écriture, en réglant une facture ou en
     imputant un compte (manuellement, par éclatement ou par règle).
   - **Autres passages faux relevés en P2**, hors de la section, à réécrire sur le même fond :
     - § rappels, `:1126` : « le rapprochement bancaire propose la facture pour un paiement de ce
       montant, même après des règlements partiels » — faux dès que le paiement tombe plus de 30 jours
       après la **date** de la facture, donc presque toujours après un rappel (finding F-3, #529) : dire
       que la facture est proposée si le paiement arrive dans les 30 jours de sa date, et sinon qu'on
       enregistre le règlement depuis la fiche de la facture ;
     - glossaire, `:2226` (*Réconciliation bancaire* : « Rapprochement … avec les écritures comptables
       (factures, paiements) ») — même idée fausse que `:1493` (finding F-4) ;
     - *Bonnes pratiques*, `:2188` (« Liez systématiquement les transactions bancaires aux factures via
       la référence QR Bill structurée ») — le score de référence compare au **numéro de facture**
       (`matching.rs:13-16`), comme la FAQ le dira (finding R-5).

## Tasks / Subtasks

- [x] **T0 — Refaire les relevés** sur `HEAD`, après le merge de la 15-5b : la liste des codes de
      l'AC1 (commande citée **et** contrôle des formes non littérales) et les numéros de ligne du
      manuel (AC5–AC9). Tout code ou passage neuf est classé au Change Log.
- [x] **T1 — Le module de libellés** (AC1, AC2) : `failed-proposal-label.ts` et
      `failed-proposal-label.test.ts` — chaque code de la liste, **écrite en dur dans le test**, rend un
      libellé distinct du repli ; la clé lue par code (réutilisée `error-*` ou neuve, C37) est écrite
      au Dev Agent Record ; `ACCOUNT_NOT_POSTABLE` nomme les numéros de `details.rejected` ; un
      `details` absent, `null`, ou d'une autre forme → libellé sans numéros ; un code inconnu → repli
      avec le code.
- [x] **T2 — Les clés** (AC4) : quatre `messages.ftl` ; `lint-i18n-ownership` ; borne `sitesTotal`
      relevée délibérément, ventilation recomptée ; `cargo test -p kesh-i18n` (les tests du chargeur
      lisent les `.ftl`).
- [x] **T3 — L'affichage** (AC3) : `ReconciliationProposals.svelte:362` (date, montant, contrepartie,
      relevés avant le `load()`) ; *Échecs partiels* et compteur hors de la branche vide ; trois tests
      de `ReconciliationProposals.test.ts`.
- [x] **T4 — Le manuel** (AC5–AC9) : l'introduction (`:1493`), `:1511`, la sous-section fusionnée
      *Accepter ou rejeter les propositions* (ex-*Acceptation des propositions* et *Acceptation par
      lot*), le rapprochement manuel, l'éclatement, les règles et la FAQ, **légendes des captures
      comprises** ; **relire au code** le filtre des deux sélecteurs (`ManualMatchModal.svelte:65-69`,
      `TransactionSplitModal.svelte:71-73`, `AccountAutocomplete.svelte:200-207`), le sort d'une
      transaction rejetée (AC5, détail d'import compris), l'ordre des candidates et la fenêtre de 30
      jours avant d'écrire ; la FAQ **et sa phrase finale** `:2105`, `:1126`, `:2188`, `:2226` ; PDF
      régénéré, commité, contrôlé aplati **ligatures normalisées** ; `admin-manual.tex` `:82` et `:2033`
      contrôlés.
- [x] **T5 — Propagation** (règle *Propagation post-patch*) :
      `grep -rnE "auto-validate|atomique|CLAUDE\.md|Modifier.*facture|N écritures|par facture|brouillon|Rapprocher manuellement|sélectionnées|candidates|meilleure candidate|1 CHF près|QR Bill structurée|ROUNDING.ACCOUNT|propose la facture|rapprochement manuel|écritures comptables|annul.*rapproch|texttt\{[A-Z]+.?_[A-Z]" docs/manual/fr/*.tex`
      et le même motif sur le PDF aplati, ligatures normalisées (où le `\_` devient `_`) ; chaque occurrence qui décrit le
      rapprochement est réécrite ou justifiée au Dev Agent Record — en particulier tout **code brut**
      d'erreur cité au manuel utilisateur (finding P1 R-2). `CHANGELOG.md` (AC9).
- [x] **T6 — Gates** : gate frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`,
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
- Le sort d'une transaction **rejetée** (AC5) : la story **écrit** ce qui est vrai — elle quitte la
  liste, reste visible sans action dans le détail de son import, et aucun geste ne la rapproche — ; elle
  ne crée pas de chemin de retour (**#526**).
- L'**ordre des candidates** — une facture faible avant une règle de score maximal (**#527**) : la
  story décrit `candidates[0]` tel quel, sans corriger le serveur.
- Le **paiement client tardif** — fenêtre de 30 jours centrée sur la date de facture, aucun moyen de
  rapprocher la transaction de la facture à l'écran (**#529**) : la story écrit la vérité et renvoie au
  règlement depuis la fiche facture.

### Décisions consignées (registre `epic-15-choix-autonomes.md`)

- **C8** — libellés traduits pour **tous** les codes de `failed[]` ; ferme #492.
- **C11** — réécriture du § *Règles d'affectation automatique* ; ferme #519.
- **C15** — la création de cette story ; ferme aussi #481.
- **C16** — la forme `details.rejected[{accountId, accountNumber}]` qu'AC2 lit.
- **C32** — l'écran réel du rapprochement et le filtre réel des sélecteurs, vérifiés au code en P1 ;
  `PERIOD_LOCKED` affiché sans son détail. **Corrigé en P2** sur deux points (la candidate affichée est
  `candidates[0]`, non « la meilleure » ; la transaction rejetée reste visible dans le détail
  d'import, et l'annulation ne s'y applique pas).
- **C37** — une clé `error-*` existante, mot pour mot et sans variable, est lue directement ; sinon clé
  neuve `reconciliation-failed-*`, sans variable.
- **C38** — les *Échecs partiels* restent visibles quand la liste se vide ; la ligne de refus désigne
  la transaction par sa date, son montant et sa contrepartie.

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

- Issues : #481, #492, #519 ; #526, #527, #529 (cités, non corrigés) ; #427, #429 (15-5b, 15-5d).
- Fiches : `15-5-gardes-postabilite-serveur.md` (mère, `split`), `15-5a-refus-non-imputable.md`,
  `15-5b-gardes-surfaces-neuves.md`.
- `CLAUDE.md` : § *Le prompt d'une passe doit NOMMER le manuel*, § *Pattern batch*.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`), agent de développement de l'Epic 15, en autonomie (worktree
`kesh-15-5c`).

### Debug Log References

- Gate backend : `scripts/test-fast.sh` sur base `kesh` remise à zéro (DROP/CREATE + migrations +
  seed, sans redémarrer le conteneur) — fmt, clippy, nextest **2784 exécutés, 2784 verts, 4 ignorés**.
- Gate frontend : `npm run check` (0 erreur, 27 avertissements préexistants), `lint-i18n-ownership`
  vert, `npm run test:unit` **108 fichiers, 1020/1020** (dont **41 tests neufs** de `f289414e` à
  `b48a1231` : 38 dans `failed-proposal-label.test.ts`, 3 dans `ReconciliationProposals.test.ts`,
  9 → 12, recomptés par Vitest), `npm run build` vert ; `cargo test -p kesh-i18n`
  **31/31**.
- E2E complet au commit de code `b48a1231`, base `kesh_e2e` recréée, montage complet (SMTP, inbox,
  documents ; `smtpConfigured: true` vérifié) : **240 verts, 7 rouges, 19 ignorés**. Les 7 rouges sont
  les sept KF-029 de `docs/testing.md` § « Les échecs attendus » (`mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) ; aucun hors liste ;
  les dix specs `reconciliation*` sont vertes.
- Mutations frontend (restaurées, fichier retouché) : libellé remplacé par `f.errorCode` → 3 rouges ;
  `describeTx` toujours `TX #<id>` → 2 rouges ; bloc *Échecs partiels* remis sous condition
  `proposals.length > 0` → 2 rouges.
- **Remédiation de la revue de code P1 (`1f481654`) — gates rejoués à ce dernier commit de code** :
  bases `kesh` et `kesh_e2e` recréées (DROP/CREATE + migrations + seed de `kesh`, conteneur non
  redémarré) ; `scripts/test-fast.sh` — fmt, clippy, nextest **2804 exécutés, 2804 verts, 4 ignorés**
  (aucun fichier Rust ni `Cargo.toml` modifié depuis `f289414e` : l'écart avec les 2784 déclarés au
  développement ne vient pas de la story et n'est pas expliqué ici) ; frontend `check` (0 erreur, 27
  avertissements, aucun dans un fichier de la story), `lint-i18n-ownership` vert, `test:unit`
  **108 fichiers, 1029/1029** (+9 depuis `b48a1231` : 6 dans `failed-proposal-label.test.ts`, 3 dans
  `ReconciliationProposals.test.ts`, recomptés par Vitest), `build` vert. E2E complet (port 3000,
  `kesh_e2e`, montage complet, `smtpConfigured: true`) : **239 verts, 8 rouges, 19 ignorés** — les sept
  KF-029 et `sidebar-navigation.spec.ts:75` (KF-046 #424, listée dans `docs/testing.md`, **verte rejouée
  seule**) ; aucun rouge hors liste. Mutations (restaurées, fichier retouché) : retirer
  `clearBatchReport()` d'`onManualSuccess`/`onSplitSuccess` et rendre le repli vide → 3 rouges.
- **Intégration sur `main` après la 15-8a (`52a9b19b`) — gates rejoués sur l'état rebasé, au commit
  `38739f65`** (dernier commit de code ; C-15-5c-4, C-15-5c-5). Bases `kesh` et `kesh_e2e` recréées
  (DROP/CREATE + 75 migrations + seed de `kesh`, conteneur non redémarré). `scripts/test-fast.sh` —
  fmt, clippy, nextest **2809 exécutés, 2809 verts, 4 ignorés** (+5 sur 2804 : les tests de la 15-8a,
  la 15-5c n'ajoutant aucun test Rust). Frontend : `check` 0 erreur (27 avertissements),
  `lint-i18n-ownership` vert, `test:unit` **111 fichiers, 1086/1086**, `build` vert. Bornes recomptées
  sur l'état rebasé et vérifiées par les tests : `sitesTotal` **1904** (1876 de la 15-8a + 28 de la
  15-5c), `CANDIDATES_ATTENDUES` **48** (`ecartee` 7, `conforme` 41). PDF utilisateur régénéré
  (`make user`, 79 pages, aucune référence indéfinie ; contrôlé aplati : le paragraphe « Une facture
  proposée peut être refusée à l'acceptation » de la 15-5c et « tant que son exercice est ouvert » de
  la 15-8a présents). E2E complet (port 3000, `kesh_e2e`, montage complet, `smtpConfigured: true`,
  lancé à 16:17 UTC) : **244 verts, 7 rouges, 19 ignorés** — les sept KF-029 de `docs/testing.md`
  § « Les échecs attendus », aucun rouge hors liste ; specs `reconciliation*` et `journal-entr*`
  vertes.

### Completion Notes List

- **T0 — relevés sur `HEAD` (`f289414e`)** : la commande de l'AC1 rend **25** littéraux, identiques à la
  liste de la fiche ; le contrôle des formes non littérales rend `:163` (le champ de `FailedProposal`)
  et `:239` (`DbError::AccountsNotPostable(list).error_code()`, soit `ACCOUNT_NOT_POSTABLE`) ; la
  conversion `project_error_to_failed_proposal` ne pose que des littéraux déjà relevés
  (`PROJECT_ARCHIVED`, `PROJECT_NOT_FOUND`, `PERIOD_LOCKED`, `DATABASE_ERROR`). **26 codes, aucun neuf.**
  Manuel : la section *Réconciliation bancaire* commençait à `:1495` (fiche : `:1491`), soit un
  décalage de +4 sur toute la plage ; FAQ `:2099`, bonnes pratiques `:2192`, glossaire `:2230`, rappels
  `:1130` ; aucun passage neuf hors de ceux de la fiche, sinon `:1861` (§ projets analytiques,
  « rapprochement manuel » → « affectation manuelle », C-15-5c-2).
- **T1 — clé lue, code par code** (C37, C-15-5c-1) :
  `ACCOUNT_NOT_FOUND` → `reconciliation-failed-account-not-found` ;
  `ACCOUNT_NOT_POSTABLE` → `reconciliation-failed-account-not-postable` (avec `{ $numbers }` lus de
  `details.rejected`) ou `reconciliation-failed-account-not-postable-generic` (sans numéros) ;
  `BANK_ACCOUNT_NOT_CONFIGURED` → `reconciliation-failed-bank-account-not-configured` ;
  `BANK_ACCOUNT_NOT_FOUND` → `reconciliation-failed-bank-account-not-found` ;
  `BANK_TRANSACTION_NOT_FOUND` → `reconciliation-failed-bank-transaction-not-found` ;
  `DATABASE_ERROR` → `reconciliation-failed-database-error` ;
  `FISCAL_YEAR_INVALID` → `error-fiscal-year-invalid` (réutilisée) ;
  `INTERNAL_ERROR` → `error-internal` (réutilisée) ;
  `INVOICE_NOT_FOUND` → `reconciliation-failed-invoice-not-found` ;
  `INVOICE_SALE_ENTRY_MALFORMED` → `reconciliation-failed-invoice-sale-entry-malformed` ;
  `PERIOD_LOCKED` → `reconciliation-failed-period-locked` ;
  `PROJECT_ARCHIVED` → `reconciliation-failed-project-archived` ;
  `PROJECT_NOT_FOUND` → `reconciliation-failed-project-not-found` ;
  `RECONCILIATION_ALREADY_RECONCILED` → `reconciliation-errors-already-reconciled` (réutilisée) ;
  `RECONCILIATION_CURRENCY_MISMATCH` → `reconciliation-failed-currency-mismatch` ;
  `RECONCILIATION_FISCAL_YEAR_CLOSED` → `error-fiscal-year-invalid` (réutilisée : même constat que
  `FISCAL_YEAR_INVALID`, aucun exercice ouvert ne couvre la date) ;
  `RECONCILIATION_INVOICE_NOT_ELIGIBLE` → `reconciliation-errors-invoice-not-eligible` (réutilisée) ;
  `RECONCILIATION_OVERPAYMENT` → `reconciliation-failed-overpayment` ;
  `RECONCILIATION_RULE_MISMATCH` → `reconciliation-failed-rule-mismatch` ;
  `RECONCILIATION_RULE_NO_LONGER_MATCHES` → `reconciliation-failed-rule-no-longer-matches` ;
  `RECONCILIATION_RULE_NOT_FOUND` → `reconciliation-failed-rule-not-found` ;
  `RECONCILIATION_SCORE_TOO_LOW` → `reconciliation-failed-score-too-low` ;
  `RECONCILIATION_SPLIT_IMBALANCE` → `reconciliation-split-error-imbalance` (réutilisée) ;
  `RECONCILIATION_TRANSACTION_NOT_PENDING` → `reconciliation-failed-transaction-not-pending` ;
  `ROUNDING_ACCOUNT_NOT_CONFIGURED` → `error-rounding-account-not-configured` (réutilisée) ;
  `VALIDATION_ERROR` → `error-validation` (réutilisée) ;
  code inconnu → `reconciliation-failed-unknown` (« Refus non reconnu ({ $code }) »).
  Soit **8 codes sur 26** sur une clé existante (7 clés distinctes) et **20 clés neuves**.
  **`PERIOD_LOCKED` n'utilise pas `details`** (`lockedThrough`, `attempted`) : limite assumée (C32),
  le libellé renvoie au verrou de période ; seul `ACCOUNT_NOT_POSTABLE` lit son détail (AC2).
- **T2** : 20 clés `reconciliation-failed-*` dans chacune des quatre locales (recompté :
  `grep -c '^reconciliation-failed-'` = 20 × 4), plates (aucun sélecteur) ; terminologie alignée sur
  `error-account-not-postable` de chaque locale (*bebuchbar*, *registrabile*, *grouping*).
  **Borne `sitesTotal` : 1868 → 1895 (+27)**, relevée par le test et recoupée par `grep -o "i18nMsg("`
  aux deux bornes : `failed-proposal-label.ts` 0 → 27 (25 `case` porteurs d'un appel — deux codes
  partagent le leur —, la variante sans numéros, le repli), `ReconciliationProposals.svelte` 18 → 18 ;
  `sitesNonResolus` 31, `relais` 6, `sitesGabarit` 10 inchangés (assertions exactes vertes),
  `litterauxMin` et `clesDepuisTsMin` inchangés. **Garde voisine** :
  `i18n-libelle-en-dur.test.ts` (`CANDIDATES_ATTENDUES`) a rougi aussi — `failedProposalLabel` porte le
  suffixe `Label` — : **46 → 47**, `conforme` **40 → 41**, déclaration nommée dans le commentaire.
- **T3** : ligne de refus « date · montant devise · contrepartie — libellé », relevée par
  `snapshotSelected()` avant l'appel au serveur (donc avant le `load()`), repli `TX #<id>` ; code brut en
  `title` et `data-error-code` ; compteur (en tête de section) et *Échecs partiels* (en pied) sortis de la
  chaîne `{#if loading} … {:else}` ; le compteur est remis à zéro au début de chaque lot. Trois tests
  ajoutés à `ReconciliationProposals.test.ts` (acceptation, rejet, liste vidée — second `getProposals`
  explicitement distinct).
- **T4** : section réécrite (C-15-5c-2). `admin-manual.tex` contrôlé : `:82` (« Réconciliation
  automatique et manuelle … avec règles d'affectation ») reste vrai ; `:2033` (« refusé … avec un message
  qui renvoie à *Paramètres* → *Facturation* ») **devient exact par l'AC3** — le lot affiche désormais
  `error-rounding-account-not-configured` au lieu du code brut. Admin non modifié, non régénéré. PDF
  utilisateur régénéré (`make user`, 77 pages, sans référence indéfinie), contrôlé aplati ligatures
  normalisées : les 16 phrases retirées sont absentes (« La meilleure candidate est affichée »,
  « Accepter sélectionnées », « Rapprocher manuellement », « atomique », « CLAUDE.md », « failed[] »,
  « auto-validate », « 1 CHF près », « QR Bill structurée », « ROUNDING_ACCOUNT », « propose la facture
  pour un paiement », « reste à rapprocher manuellement », « en N écritures », …), les nouvelles
  présentes (« Affecter manuellement », « Accepter (N) », « Échecs partiels », « 30 jours de la date de
  la facture », « une seule écriture », « classes 5, 6 et 7 », « sans aucune action », « produit serait
  compté deux fois », …). Brochure non régénérée, inchangée.
  **Fait relevé au code, absent de la fiche** : une règle dont le compte n'est plus imputable ou est
  archivé est **sautée**, et la règle suivante qui correspond est proposée à sa place
  (`first_matching_rule` sur l'ensemble des comptes imputables, `reconciliation.rs:586-591`) — le manuel
  l'écrit.
- **T5 — propagation** : le motif de la tâche, sur `docs/manual/fr/*.tex`, ne rend plus que des
  occurrences hors rapprochement ou justes : `admin-manual.tex` — variables d'environnement
  `KESH_*`/`MARIADB_*` en `\texttt` (pas des codes d'erreur), `user-manual.tex` — `brouillon` des
  factures et des écritures, *Annuler un rapprochement* (juste : transaction rapprochée), `écritures
  comptables` du journal et de la TVA. Aucun code d'erreur brut ne reste au manuel utilisateur.
- **T6** : cf. *Debug Log References*.
- **Revue de code P1** : `RECONCILIATION_INVOICE_NOT_ELIGIBLE` lit désormais `details.reason` pour la
  seule raison `payment_date_before_invoice_date` (clé neuve `reconciliation-failed-payment-before-invoice`,
  4 locales + repli ; C-15-5c-3). **Codes à plusieurs causes sous un libellé unique, assumé (C32, C37)** :
  `PERIOD_LOCKED` (détail non lu), `RECONCILIATION_INVOICE_NOT_ELIGIBLE` (six raisons, dont cinq sur le
  libellé générique) et `VALIDATION_ERROR` (six raisons sur sept sites, toutes « Erreur de validation ») ;
  le code brut reste en `title`. Écrit au commentaire de `failed-proposal-label.ts`. **Bornes** :
  `sitesTotal` 1895 → **1896** (`grep -c "i18nMsg("` sur `failed-proposal-label.ts` : 27 → 28 ; le
  `.svelte` reste à 18) ; `CANDIDATES_ATTENDUES` **inchangé à 47** (`failureReason` et
  `clearBatchReport` ne portent pas de suffixe `Label`/`Text`/`Display`). Clés `reconciliation-failed-*` :
  **21 par locale** (recompté `grep -c`).

### File List

- `frontend/src/lib/features/reconciliation/failed-proposal-label.ts` (neuf)
- `frontend/src/lib/features/reconciliation/failed-proposal-label.test.ts` (neuf)
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.test.ts`
- `frontend/src/lib/features/reconciliation/ModalSuccessStub.test.svelte` (neuf, doublure de test — revue P1)
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md` (C-15-5c-1 à C-15-5c-5)
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/15-5c-rapprochement-libelles-et-manuel.md`

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
- 2026-10-08 — **Passe de validation P2** (prompt versionné `15-5c-validate-prompt-p2.md` ; deux
  lentilles **Opus** en contexte frais : **R** chasseur de régressions de la remédiation P1
  (`007c4eb1`), **F** adversaire de périmètre complet ; rotation D6 : P1 Sonnet ×2 → P2 Opus ×2).
  **0 CRITICAL, 0 HIGH.** Bruts : R 2 MEDIUM + 6 LOW, F 3 MEDIUM + **10** LOW (son bilan en annonce 9 ;
  il en numérote 10, F-4 à F-13 — recompté). Doublons inter-lentilles : R-1 = F-2, R-2 = F-1, R-3 =
  F-10, R-6 = F-9 → **3 MEDIUM et 14 LOW distincts**. Trend : P1 3 MEDIUM / 7 LOW → **P2 3 MEDIUM /
  14 LOW**.

  | finding | sév. | objet | sort | origine (amendement D5) |
  |---|---|---|---|---|
  | R-1 = F-2 | MEDIUM | le relevé (C32) et l'AC9 disent la candidate affichée « la meilleure », une règle proposée sous 0,5 — l'écran affiche `candidates[0]` : la facture faible masque la règle | relevé, AC5, AC9 : `candidates[0]`, règle affichée seulement sans facture candidate ; `:1511` à réécrire ; **#527** citée, non corrigée | **né de la remédiation P1** (relevé de C32) |
  | R-2 = F-1 | MEDIUM | AC5 : « aucun écran n'affiche une transaction rejetée » et « l'annulation remet le marquage à zéro » — faux à l'écran | AC5 : visible dans le détail d'import (`bank-import/[id]/+page.svelte:110-117`, statut `pending`, sans action) ; aucun geste ne la rapproche ; l'annulation ne s'y applique pas ; **#526** citée | **né de la remédiation P1** (parenthèse ajoutée par `007c4eb1`) |
  | F-3 | MEDIUM | FAQ : la phrase finale `:2105` conseille une règle pour un paiement client (produit compté deux fois) ; fenêtre de 30 jours centrée sur la date de facture ; `:1126` (rappels) promet la proposition | AC9 : phrase finale réécrite (règlement depuis la fiche facture, jamais de règle pour un paiement client), fenêtre écrite, `:1126` réécrit ; **#529** citée ; T5 étendu | **d'origine** (passages antérieurs à la story, jamais inventoriés) |
  | R-3 = F-10 | LOW | #526 existe, la fiche dit « signalé pour une issue » | *Issues*, *Hors périmètre*, *References* : #526, #527, #529 | — |
  | R-4 | LOW | numéros de ligne du manuel décalés | AC5, AC7, AC8 corrigés (`:1517-1520`, `:1525`, `:1526`, `:1534-1545`, `:1565`, `:1567`, `:1574-1575`) | — |
  | R-5 | LOW | `:2188` (*Bonnes pratiques*, « référence QR Bill structurée ») hors des plages | AC9, autres passages faux | — |
  | R-6 = F-9 | LOW | `admin-manual.tex` « sans objet » inexact : `:82`, `:2033` | AC9 : `:82` reste vrai, `:2033` devient exact par l'AC3 | — |
  | R-7 | LOW | réutiliser la clé `error-rounding-account-not-configured` ou copier le texte ? | AC1 : réutilisée (C37) | — |
  | R-8 | LOW | la commande de relevé ne voit que les littéraux | AC1, T0 : contrôle des formes non littérales | — |
  | F-4 | LOW | glossaire `:2226` | AC9 ; T5 (`écritures comptables`) | — |
  | F-5 | LOW | *Échecs partiels* masqués quand la liste se vide | AC3 : hors de la branche vide ; test avec un second `getProposals` vide (C38) | — |
  | F-6 | LOW | `TX #<id>` ne désigne rien de visible | AC3 : date, montant, contrepartie relevés avant le `load()` (C38) | — |
  | F-7 | LOW | ligatures du PDF aplati (`ﬀ`) : faux négatifs de présence | AC9 : `pdftotext -nopgbrk` + normalisation `ﬀ/ﬁ/ﬂ` ; même recette à la 15-5b et à la 15-5d ; recette du `CLAUDE.md` signalée à l'orchestrateur | — |
  | F-8 | LOW | AC8 : priorité (le plus petit l'emporte, seule la première règle) ; type non modifiable | AC8 | — |
  | F-11 | LOW | la liste est bornée à 100 transactions | relevé, AC5 | — |
  | F-12 | LOW | noms des champs de la ventilation `sitesTotal` | AC4 : champs réels | — |
  | F-13 | LOW | libellés « relevés » à variable ; doublons de traduction | AC1 : règle C37 | — |

  **Signal de la règle de découpage** (sévérité MEDIUM → MEDIUM, P1 → P2) : **constaté**. Selon
  l'amendement D5 : deux MEDIUM (R-1/F-2, R-2/F-1) sont **nés de la remédiation P1** — non du recyclage
  d'un finding de P1, mais d'un relevé d'écran écrit en P1 et faux sur deux points ; un (F-3) est
  **d'origine**. La story reste à **trois modules** (frontend/reconciliation, kesh-i18n, manuel), sans
  code serveur : les défauts sont des **faits mal relevés**, que la remédiation a vérifiés au code ligne
  par ligne, pas une dispersion de périmètre. **Pas de découpage proposé** ; signal déclaré au Project
  Lead par l'orchestrateur, à qui revient l'arbitrage. **Décisions de l'orchestrateur** : citer #526,
  #527, #529 sans corriger ; trancher la clé d'arrondi (C37). Ajoutée pendant la remédiation : C38.
  **Propagation post-patch** : `meilleure`, `reconciliation_cancel`, `annul`, `disparaît`, `aucun écran`,
  `0,5`, `1515-1518`, `1524-1525`, `1572-1573`, `1534-1546`, `QR Bill structurée`, `écritures
  comptables`, `propose la facture`, `sans objet`, `litteraux`, `pdftotext` grepés sur les fiches 15-5,
  15-5a (lecture seule), 15-5b, 15-5c, 15-5d et le registre — les occurrences restantes sont
  historiques (Change Log) ou nient la formule. Décompte inchangé : **9 AC, 7 tâches T0–T6**
  (recompté). **Une passe P3 suit** (des MEDIUM en P2) ; elle peut être ciblée sur ce commit.
- 2026-10-08 — **Passe de validation P3, ciblée** (prompt versionné `15-5c-validate-prompt-p3-ciblee.md` ;
  une lentille **Haiku**, contexte frais : chasseur de régressions braqué sur le seul commit de la
  remédiation P2, `67c31c95` ; passe ciblée de fin de boucle, décision D6). Rapport :
  `target/gate-logs/15-5c-p3-ciblee.md`. **0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW.**
  - **« 0 » vérifié par l'orchestrateur** (règle « un 0 finding se vérifie comme un finding ») sur
    l'affirmation centrale de la remédiation : l'écran lit bien `candidates[0]`
    (`frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte:137`,
    `const c = p.candidates[0];`, et `:265` pour l'affichage) — conforme au relevé de l'AC5 et de l'AC9.
  - Axes déclarés exercés : affirmations sur le code (`candidates[0]`, fenêtre de 30 jours sur la date
    de facture, classes 5/6/7, clé d'arrondi, formes non littérales), références d'issues et de choix,
    cohérence interne, décomptes. Non exercés, déclarés : numéros de ligne du manuel au-delà d'un
    sondage (refaits en T0), borne exacte des codes de l'AC1 (dépend du merge de la 15-5b), exécution
    des tests.
  - La remédiation P2 ne touche aucune ligne de code de production (fiche seule) : **boucle de
    validation close.**

  **Trend complet de la fiche** :

  | passe | modèle(s) | périmètre | bilan |
  |---|---|---|---|
  | P1 | Sonnet ×2 | fiche entière | 0 C / 0 H / 3 MEDIUM / 7 LOW |
  | P2 | Opus ×2 | fiche entière | 0 C / 0 H / 3 MEDIUM / 14 LOW |
  | P3 ciblée | Haiku ×1 | commit `67c31c95` | 0 C / 0 H / 0 MEDIUM / 0 LOW |

  Décompte inchangé : **9 AC, 7 tâches T0–T6** (recompté). Fiche prête pour le développement, après le
  merge de la 15-5b.
- 2026-10-08 — **Développement** (`bmad-dev-story`, Opus 5.5). Module de libellés des 26 codes de
  `failed[]` (8 sur clé existante, 20 clés neuves × 4 locales), affichage des refus par transaction et
  hors de la branche vide, manuel du rapprochement réécrit et PDF régénéré, CHANGELOG. Bornes relevées :
  `sitesTotal` 1868 → 1895, `CANDIDATES_ATTENDUES` 46 → 47 (garde non prévue par la fiche). Gates :
  backend 2784/2784 (4 ignorés), frontend 1020/1020, `kesh-i18n` 31/31, E2E 240 verts / 7 rouges KF-029
  / 19 ignorés, au commit de code `b48a1231`. Choix C-15-5c-1, C-15-5c-2. Statut : `review`.
- 2026-10-08 — **Revue de code P1** (prompt versionné `15-5c-review-prompt-p1.md` ; trois lentilles
  **Sonnet** en contexte frais : **B** Blind Hunter, **E** Edge Case Hunter, **A** Acceptance Auditor ;
  rapports `target/gate-logs/15-5c-review-p1-{B,E,A}.md`). **0 CRITICAL, 0 HIGH, 1 MEDIUM, 10 LOW**
  bruts (B 3 LOW, E 1 MEDIUM + 4 LOW, A 4 LOW) ; doublons B1 = E3, B3 = E4, E2 ≈ A-4 → **1 MEDIUM,
  7 LOW distincts**. Remédiation `1f481654` :

  | finding | sév. | objet | sort |
  |---|---|---|---|
  | E1 | MEDIUM | une facture datée jusqu'à 30 jours après le paiement est proposée, puis refusée à l'acceptation (paiement antérieur de plus d'un jour) | **moteur inchangé — alignement : #548** (P3, ouverte par l'orchestrateur) ; manuel : paragraphe « Une facture proposée peut être refusée à l'acceptation » (date de valeur, même borne au règlement depuis la fiche, `invoice_settlements_write.rs:96`) et FAQ ; libellé dédié lu dans `details.reason` (C-15-5c-3) |
  | B1 = E3 | LOW | bilan du lot périmé après affectation manuelle ou éclatement | `clearBatchReport()` au début de tout nouveau bilan ; tests par doublure de modale (mutation : 2 rouges) |
  | A-3 | LOW | repli `TX #<id>` non testé | test (mutation : 1 rouge) |
  | A-1 | LOW | « charges et produits » : le sélecteur filtre sur les classes 5, 6, 7 | manuel : « classes 5, 6 et 7, et eux seuls » ; FAQ « une charge diverse » |
  | A-2 | LOW | « score de 1 » alors que l'écran affiche « 100 % » | manuel corrigé |
  | B2 | LOW | commentaire de test inexact (test 1) | reformulé |
  | B3 = E4 | LOW | italien : impératif au lieu de l'infinitif | « scegliere » (deux clés) |
  | E2 + A-4 | LOW | codes à plusieurs causes sous un libellé unique | écrit au commentaire du module et au Dev Agent Record (C32, C37) |
  | E5 | LOW | date et montant bruts, comme le tableau | sans action (la lentille le dit) |

  PDF utilisateur régénéré (`make user`, 77 pages, sans référence indéfinie), contrôlé aplati
  (ligatures et apostrophes normalisées) : phrases neuves présentes, « score de 1. », « charges et
  produits », « produit divers » absents ; brochure et manuel admin inchangés. **Propagation** : symptômes
  grepés sur `docs/`, `website/`, `README.md` (aucun résidu) ; raisons de `details.reason` relevées au
  code (`race_during_update` ajoutée au décompte). Gates complets au commit de code `1f481654` : cf.
  *Debug Log References* (backend 2804/2804, frontend 1029/1029, E2E 239 / 8 rouges attendus / 19).
  La remédiation touche du code de production (composant et module de libellés) : **une passe ciblée
  sur `1f481654` reste à lancer** selon la règle de clôture. Statut : `done` (gate vert).
- **2026-10-08 — Revue de code P2 ciblée (Haiku, une lentille, prompt `15-5c-review-prompt-p2-ciblee.md`) sur
  `1f481654` : 0 finding.** Rapport : `target/gate-logs/15-5c-review-p2-ciblee.md`. Le rapport ne listant pas ses axes
  non exercés, l'orchestrateur a repris lui-même les deux axes porteurs : la raison `payment_date_before_invoice_date`
  n'est posée qu'à `crates/kesh-api/src/routes/reconciliation.rs:1337` et lue à
  `frontend/src/lib/features/reconciliation/failed-proposal-label.ts:154` ; `clearBatchReport()` est appelée aux quatre
  débuts de bilan (`ReconciliationProposals.svelte:142`, `:154`, `:171`, `:216`). **Boucle de revue CLOSE** (P1 Sonnet
  ×3 : 1 MEDIUM, 10 LOW → P2 ciblée Haiku : 0).
- 2026-10-08 — **Intégration sur `main` après le merge de la 15-8a** (`52a9b19b`) : branche reconstruite
  (planification rejouée d'un bloc par `cherry-pick -m 2`, puis les sept commits de la 15-5c ; C-15-5c-4),
  registre et `sprint-status.yaml` fusionnés par union, CHANGELOG/`.tex`/`.ftl` sans conflit, PDF
  régénéré, compteurs i18n recomptés (`sitesTotal` 1904, `CANDIDATES_ATTENDUES` 48 ; C-15-5c-5). Gates
  rejoués au commit `38739f65` : backend 2809/2809, frontend 1086/1086, E2E 244 verts / 7 KF-029.
