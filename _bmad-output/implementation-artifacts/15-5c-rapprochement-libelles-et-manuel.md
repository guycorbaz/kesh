# Story 15.5c : Le rapprochement se lit — libellés des refus par lot et manuel du rapprochement

Status: ready-for-dev

<!-- Troisième sous-story de la 15-5, créée le 2026-10-08 à la passe de validation P2 de la 15-5b
     (finding F-3, choix C15 de `epic-15-choix-autonomes.md`). Elle reprend de la 15-5b le premier volet
     de son ancien AC14 (libellés de `failed[]`, choix C8) et la réécriture du manuel du rapprochement
     (choix C11), étendue aux passages faux relevés par le finding F-2 de la même passe, qui recoupent
     l'issue #481. Choix applicables : C8, C11, C15, C16 (forme du détail `rejected`). Validation : à
     lancer. -->

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
   gagne un test : une réponse d'acceptation portant un `failed[]` → le libellé traduit est affiché (et
   non le code seul) — aujourd'hui aucun test n'exerce l'affichage de `failed[]`.
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

`docs/manual/fr/user-manual.tex`, section *Réconciliation bancaire*. Lignes relevées sur `1920381e` ;
le dev les refait (T0). Le dev décrit **ce qu'il voit** dans le code et à l'écran, pas ce que disent
ces critères.

5. **AC5 — *Acceptation des propositions* : plus de bouton *Modifier*** (#481, point 1 ; `:1528`).
   L'écran n'offre qu'*Accepter* (la meilleure candidate) et *Rejeter* ; `ReconciliationProposals.svelte`
   ne soumet jamais que `candidates[0]`. L'item *Modifier* est retiré ; s'il faut dire comment choisir
   une autre facture, renvoyer au rapprochement manuel (AC7) tel qu'il est.
6. **AC6 — *Acceptation par lot* : succès partiel, refus en clair** (#481, point 2 ; `:1532`). Le texte
   actuel dit l'opération « atomique : soit toutes … soit aucune » et renvoie à « la doc CLAUDE.md
   projet » — **faux et hors de propos pour un utilisateur**. Le réécrire : les propositions acceptées
   le sont, les refusées sont listées avec leur motif en clair (AC1–AC3) ; une proposition refusée
   n'empêche pas les autres.
7. **AC7 — *Réconciliation manuelle* et *Éclatement* : un compte de contrepartie, une seule écriture**
   (#481, point 3 ; `:1534-1546` et `:1548-1559`, légendes des deux `\keshscreenshot` comprises).
   - Rapprochement manuel : il passe une écriture entre le compte de la banque et **un compte de
     contrepartie** que l'utilisateur choisit (`ManualMatchModal.svelte:157-160`) — il ne porte ni
     « facture impayée » ni « écriture existante », et ne crée pas d'écriture « à la volée » au sens du
     texte actuel. Le compte proposé est un compte **actif et imputable** ; un compte non imputable
     envoyé autrement est refusé (15-5b).
   - Éclatement : l'utilisateur répartit le montant en lignes, **un compte par ligne**
     (`TransactionSplitModal.svelte:265-268`) ; le tout produit **une seule** écriture à plusieurs
     lignes (`reconciliation.rs:3619`), pas « N écritures » ; le cas d'usage « virement groupé de trois
     factures » est remplacé par un cas que l'écran sert réellement (p. ex. un prélèvement à ventiler
     entre plusieurs charges), ou réécrit sans promettre de régler des factures.
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
     proposée, se modifie, mais ne se réactive pas sans changer de compte.
   Le dev décrit **ce qu'il voit** dans `RuleFormModal.svelte`, `RulesList.svelte` et `get_proposals`.
9. **AC9 — FAQ, PDF, CHANGELOG.**
   - FAQ *Une transaction bancaire ne propose aucun match automatique* (`:2095-2105`) : ajouter le cas
     d'une **règle écartée** — son compte est archivé ou non imputable, elle n'est plus proposée ;
     revérifier les trois puces existantes contre `get_proposals` (seuil, référence, libellé) et
     corriger ce qui ne tient pas.
   - Le PDF est régénéré (`latexmk -xelatex` dans `docs/manual/fr/`), commité, et **contrôlé aplati**
     (`pdftotext … - | tr '\n' ' ' | tr -s ' '`) : les phrases retirées sont absentes, les nouvelles
     présentes.
   - `docs/manual/fr/admin-manual.tex` : contrôle **sans objet** attendu (aucune description du
     rapprochement) — le refaire et l'écrire au Dev Agent Record.
   - `CHANGELOG.md`, section `## [0.13.0] — Non publié`, rubrique **Corrigé** : les refus d'une
     acceptation par lot s'affichent en clair (#492) ; le manuel du rapprochement décrivait un bouton,
     une opération atomique, un rapprochement par facture et un écran de règles qui n'existent pas
     (#481, #519) — même rubrique, le CHANGELOG n'ayant pas de rubrique *Documentation*.

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
- [ ] **T4 — Le manuel** (AC5–AC9) : les cinq passages et la FAQ ; PDF régénéré, commité, contrôlé
      aplati ; `admin-manual.tex` contrôlé.
- [ ] **T5 — Propagation** (règle *Propagation post-patch*) :
      `grep -rnE "auto-validate|atomique|CLAUDE\.md|Modifier.*facture|N écritures|par facture|brouillon" docs/manual/fr/*.tex`
      et le même motif sur le PDF aplati ; chaque occurrence qui décrit le rapprochement est réécrite ou
      justifiée au Dev Agent Record. `CHANGELOG.md` (AC9).
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

### Décisions consignées (registre `epic-15-choix-autonomes.md`)

- **C8** — libellés traduits pour **tous** les codes de `failed[]` ; ferme #492.
- **C11** — réécriture du § *Règles d'affectation automatique* ; ferme #519.
- **C15** — la création de cette story ; ferme aussi #481.
- **C16** — la forme `details.rejected[{accountId, accountNumber}]` qu'AC2 lit.

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
