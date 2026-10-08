# Story 15.5 : Gardes de postabilité côté serveur — réconciliation, règles, réglages de facturation, compte bancaire

## Status

split

⛔ **CORPS VIDÉ — cette fiche ne contient plus ni critères, ni tâches, ni inventaire.** Elle ne garde
que les pointeurs vers ses quatre sous-stories et l'historique des passes qui ont conduit au découpage.
*(La définition du statut `split` l'impose ; précédents : 15-1, 17-2.)* La version complète d'avant
découpage se lit au commit `428985c8`.

## Les quatre sous-stories

| | fiche | ce qu'elle porte | issues |
|---|---|---|---|
| **15-5a** | `15-5a-refus-non-imputable.md` | **Socle** : la variante `DbError::AccountsNotPostable` (newtype trié, non vide), le code `ACCOUNT_NOT_POSTABLE`, son message (4 locales, pluriel résolu côté serveur), sa conversion sur la saisie manuelle, l'écriture d'ouverture et les trois gardes de la 24-5 ; les tests qui figeaient l'ancien code ; l'ordre des causes ; le détail `rejected[{accountId, accountNumber}]` ; `docs/api-external.md` | `refs #427`, `refs #429` |
| **15-5b** | `15-5b-gardes-surfaces-neuves.md` | **Rollout** : les gardes neuves — rapprochement manuel et ventilé, acceptation `split` et `rule`, `get_proposals`, création / modification / réactivation des règles, six réglages de facturation, compte comptable d'un compte bancaire ; le compte créanciers préservé (#521) ; les `catch` de l'écran des règles ; les `<select>` du compte bancaire ; les deux encadrés du manuel (désignation) | `closes #427`, `refs #429`, `closes #521` |
| **15-5c** | `15-5c-rapprochement-libelles-et-manuel.md` | Les libellés traduits des refus par lot (`failed[]`) ; la réécriture du manuel du rapprochement (bouton *Modifier*, lot « atomique », manuel et éclatement, règles d'affectation, FAQ) | `closes #481`, `closes #492`, `closes #519` ; cite #526, #527, #529 |
| **15-5d** | `15-5d-garde-usage-comptes-reglage.md` | La garde **à l'usage** des comptes de réglage (créance, TVA due, créanciers, TVA récupérable) à la validation d'une facture et à la saisie d'une facture fournisseur — variante `DesignatedAccountsNotPostable`, révision de la limite L2 de D-A0 ; le compte créanciers exposé à l'écran des réglages ; l'avoir exempté, raison écrite | `closes #429` |

⚠️ **L'ordre n'est pas indifférent** : 15-5a → 15-5b → (15-5c, 15-5d). La 15-5b émet la variante que
la 15-5a pose ; la 15-5c affiche le refus que la 15-5b émet dans `failed[]` et décrit le comportement
des règles qu'elle fixe ; la 15-5d s'appuie sur l'AC19 et l'AC10 de la 15-5b. La 15-5c et la 15-5d
sont **indépendantes** l'une de l'autre (seule la borne `sitesTotal` est commune). Chacune ne commence
qu'après le merge de celles dont elle dépend.

## Pourquoi le découpage

La passe de validation P1 (trois lentilles Sonnet) n'a rien trouvé au-dessus de MEDIUM, mais deux
lentilles (A : M5, C : C-4) ont relevé que la story franchissait le **critère de périmètre** de la
§ *Règle de splitting préventif* — `kesh-db`, `kesh-api` (erreurs + cinq modules de routes),
`kesh-i18n`, `frontend`, manuels, CHANGELOG — et que la fiche s'en dispensait par l'argument « les
gardes sont mécaniques », qui n'est pas une dérogation prévue (la seule codifiée : les cycles de
dépendance Cargo). L'orchestrateur a découpé selon le patron que la règle prescrit — **story-zéro qui
pose le patron, puis rollout** (choix **C7** de `epic-15-choix-autonomes.md`). Les remédiations de la
passe P1 ont été appliquées **dans les deux fiches filles**, pas ici.

## Décisions prises pour la story et où elles vivent

| choix | objet | fiche |
|---|---|---|
| C3 | forme du refus : variante dédiée, code `ACCOUNT_NOT_POSTABLE` | 15-5a (pose), 15-5b (emploie) |
| C4 | exemption « inchangé » : réglages, compte bancaire, PATCH de règle | 15-5b |
| C5 | compte d'un compte bancaire : postabilité seule ; le reste de #474 à la 15-6 | 15-5b |
| C6 | angles morts assumés (fiche article, compte bancaire à l'usage, rôles, avoir) | 15-5b |
| C7 | découpage | les deux |
| C8 | libellés traduits pour tous les codes de `failed[]` (#492) | 15-5c (depuis C15) |
| C9 | règle périmée : plus proposée, réactivation refusée, pas de migration | 15-5b |
| C10 | contrôle « inchangé » dans la transaction, ordre des erreurs conservé | 15-5b |
| C11 | réécriture du § *Règles d'affectation automatique* (#519) | 15-5c (depuis C15) |
| C12 | `withCurrentAccount` sur les `<select>` du compte bancaire (corrigé par C26) | 15-5b |
| C13 | ordre des causes ; sa clé `accountNumbers` est **révisée par C16** | 15-5a, 15-5b |
| C14 | les `catch` de l'écran des règles lisent `ApiError` (« [object Object] ») | 15-5b |
| C15 | troisième sous-story 15-5c (libellés de `failed[]`, manuel du rapprochement, #481) | 15-5b, 15-5c |
| C16 | détail `rejected[{accountId, accountNumber}]`, forme du jumeau `ACCOUNT_ARCHIVED` | 15-5a, 15-5b, 15-5c |
| C17 | newtype `NonPostableAccounts` à champ privé, trié, non vide | 15-5a |
| C18 | sélecteur Fluent inscrit à `SELECTEURS_RESOLUS_COTE_SERVEUR` | 15-5a |
| C19 | « de regroupement, de résultat ou de clôture » | 15-5a, 15-5b |
| C20 | `exempt_ids` retiré | 15-5a |
| C21 | verrou de `validate_lines_accounts_in_tx` hors périmètre | 15-5a |
| C22 | boucle des factures fournisseur : forme d'abord, comptes ensuite | 15-5a |
| C23 | chaque commentaire réécrit par une seule story | 15-5a, 15-5b |
| C24 | `docs/api-external.md` | 15-5a, 15-5b |
| C25 | #521 : compte créanciers absent du corps → préservé (révisé par C34) | 15-5b |
| C26 | `BankAccountJournalLinkForm` reçoit la liste complète | 15-5b |
| C27 | garde à l'usage des comptes de réglage | 15-5d (depuis C33) |
| C28 | forme de ce refus : `DesignatedAccountsNotPostable`, même code, message propre (texte révisé par C36) | 15-5d (depuis C33) |
| C29 | `NonPostableAccounts::details()`, seul constructeur de `rejected` | 15-5a, 15-5b, 15-5d |
| C30 | création de règle gardée dans le dépôt | 15-5b |
| C31 | ordre des comptes nommés | 15-5a |
| C32 | écran réel du rapprochement (corrigé en P2 de la 15-5c) | 15-5c |
| C33 | quatrième sous-story 15-5d | 15-5b, 15-5d |
| C34 | compte créanciers exposé à l'écran (révise C25) | 15-5d |
| C35 | avoir exempté de la garde à l'usage, raison écrite | 15-5d |
| C36 | message du refus à l'usage | 15-5d |
| C37 | clés `error-*` réutilisées | 15-5c |
| C38 | échecs partiels toujours visibles, ligne désignée | 15-5c |

## Change Log

- 2026-10-08 — Spécification initiale (bmad-create-story, en autonomie), commit `428985c8`. Statut
  `ready-for-dev`. Choix C3–C6 consignés.
- 2026-10-08 — **Passe de validation P1** (prompt versionné `15-5-validate-prompt-p1.md` ; trois
  lentilles Sonnet en contexte frais : A auditeur d'acceptation, B chasseur de chemins non gardés,
  C régressions et bords). **0 CRITICAL, 0 HIGH.** Bruts : A 5 MEDIUM + 6 LOW, B 2 MEDIUM + 2 LOW, C 4
  MEDIUM + 4 LOW (11 MEDIUM, 12 LOW). Après fusion des doublons inter-lentilles : **7 MEDIUM et 8 LOW
  distincts** :

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | M1 = C-1 | MEDIUM | A, C | grep de T2 ratant `invoice_settlement.rs:354` ; `supplier_invoices_repository.rs:424`, `:470` non nommés | 15-5a AC7, T0 |
  | M2 = B1 | MEDIUM | A, B | « Aucun changement d'écran requis » faux : `failed[]` en code brut | 15-5b AC14 (C8, #492) |
  | M3 ≈ C-2 | MEDIUM | A, C | PATCH `active:true` ressuscitant une règle périmée ; état visible non dit | 15-5b AC8 (b), AC9, AC17 (C9) |
  | M4 | MEDIUM | A | ordre « non imputable ET mauvais type » non tranché | 15-5a AC4 |
  | M5 = C-4 | MEDIUM | A, C | règle de splitting franchie | **découpage (C7)** |
  | B2 (+ C-6 LOW) | MEDIUM | B, C | lecture de la valeur en place avant validation : ordre des erreurs changé, hors transaction | 15-5b AC8, AC12 (C10) |
  | C-3 | MEDIUM | C | `## [Unreleased]` inexistant au CHANGELOG | 15-5a AC9, 15-5b AC18 |
  | L1 | LOW | A | chemins frontend incomplets | 15-5b |
  | L2 + B4 | LOW | A, B | détail structuré du 400 et de `failed[]` non fixé | `accountNumbers` (C13) |
  | L3 | LOW | A | pluriel et apostrophe du message | 15-5a AC2 |
  | L4 | LOW | A | priorité 404 > 400 de `post_split` et messages des 4 locales non testés | 15-5a T2, 15-5b AC16 |
  | L5 + C-8 | LOW | A, C | manuel : § *Règles d'affectation* décrit un écran inexistant ; `admin-manual` sans objet | 15-5b AC17 (C11, #519) |
  | L6 | LOW | A | inventaire (a) à ±1 | 15-5b, recompté |
  | B3 + C-5 | LOW | B, C | numéros de ligne (`fn` vs appel ; dépôt vs route homonyme ; `:1656`) | 15-5b |
  | C-7 | LOW | C | comportement du `<select>` Svelte sur une valeur absente des options | 15-5b AC13 (C12), vérifié au code |

  **Trouvés pendant la remédiation** (en refaisant les axes manuel et écrans) : le § *Acceptation par
  lot* du manuel (`user-manual.tex:1532`) dit l'opération « atomique » et renvoie au `CLAUDE.md` →
  15-5b AC17 ; `RuleFormModal.svelte:104` et `RulesList.svelte:64`, `:87` affichent « [object Object] »
  pour un `ApiError` → 15-5b AC14 (C14). Le même motif hors module (`reports/+page.svelte:210`,
  `settings/+page.svelte:236`, `:260`) est signalé pour une issue.

  **Découpage (C7)** : statut `split`, corps vidé. Les deux fiches filles portent chacune la remédiation
  qui leur revient et leur propre Change Log ; la passe P2 se lance **sur chacune**.
- 2026-10-08 — **Passe de validation P2** sur les deux fiches filles (prompts versionnés
  `15-5a-validate-prompt-p2.md` et `15-5b-validate-prompt-p2.md` ; deux lentilles **Opus** par fiche,
  R chasseur de régressions et F adversaire de périmètre complet). **0 CRITICAL, 0 HIGH** sur l'une et
  l'autre ; **15-5a : 5 MEDIUM, 9 LOW distincts ; 15-5b : 6 MEDIUM, 8 LOW distincts** (détail et
  origine de chaque MEDIUM selon l'amendement D5 dans le Change Log de chaque fiche). Sur le finding
  F-3 de la 15-5b, l'orchestrateur a **découpé une troisième fois** : la **15-5c** reprend les libellés
  de `failed[]` (#492) et le manuel du rapprochement (#519, et #481 qu'elle ferme entièrement) (choix
  **C15**). L'issue **#521** (compte créanciers effacé à l'enregistrement des réglages), créée par
  l'orchestrateur sur le finding F-1 de la 15-5b, est confiée à la 15-5b (choix C25). Choix C15 à C26
  consignés.
- 2026-10-08 — **Passes de validation P3** (15-5a, 15-5b) **et P1** (15-5c), commit `007c4eb1` : détail
  dans le Change Log de chaque fiche ; choix C27 à C32. La remédiation P3 de la 15-5b y a ajouté la garde
  à l'usage des comptes de réglage (AC20, C27, C28).
- 2026-10-08 — **Passes de validation P4** (15-5b) **et P2** (15-5c), deux lentilles **Opus** par fiche.
  **15-5b : 1 HIGH, 2 MEDIUM, 9 LOW distincts** ; **15-5c : 0 HIGH, 3 MEDIUM, 14 LOW distincts**. Sur le
  finding F4-3 de la 15-5b (l'AC20 est une règle métier, pas un rollout, et le HIGH en est né),
  l'orchestrateur a **découpé une quatrième fois** : la **15-5d** reprend la garde à l'usage, expose le
  compte créanciers à l'écran (C34, sur le HIGH) et exempte l'avoir avec sa vraie raison (C35) ; elle
  porte `closes #429`, la 15-5b passe à `refs #429`. La 15-5c cite sans les corriger #526, #527 et
  #529. Choix C33 à C38 consignés. Signaux de la règle de découpage (amendement D5) déclarés au Project
  Lead dans le Change Log de chaque fiche.
