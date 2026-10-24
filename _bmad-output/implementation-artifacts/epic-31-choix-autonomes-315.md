# Epic 31 — registre des choix autonomes de la Story 31-5 (quatre défauts courts)

Versé par union dans `epic-31-choix-autonomes.md` au merge.

## Story 31-5

### C-31-5-0 — Emplacement de ce registre

- **Contexte** : le prompt de l'orchestrateur nomme `_bmad-output/planning-artifacts/epic-31-choix-autonomes-315.md` ;
  les consignes communes (`docs/agents-orchestration.md` § 1) et tous les registres de l'epic (`-311`, `-312`, `-313`)
  sont dans `_bmad-output/implementation-artifacts/`.
- **Retenu** : `implementation-artifacts/`, à côté des autres, pour que l'union se fasse au même endroit.
- **Écartée** : `planning-artifacts/` — un registre isolé que l'union ne verrait pas.
- **Réversible** : oui (un `git mv`).

### C-31-5-1 — Une seule story, quatre volets indépendants

- **Contexte** : cinq modules de premier niveau (`kesh-db`, `kesh-seed`, `kesh-api`, `kesh-i18n`, `frontend`) ; la règle
  de découpage préventif découpe **au-delà** de cinq.
- **Retenu** : une fiche, quatre volets A-D sans fichier de code commun (seuls les catalogues `kesh-i18n` sont partagés
  par C et D, en lignes distinctes).
- **Écartées** : quatre stories (quatre boucles de validation et de revue pour des correctifs d'une à six lignes de
  production) ; deux stories frontend/backend (découpage sans rapport avec les défauts).
- **Réversible** : oui — un volet qui résiste en revue se détache en story propre sans toucher les autres.

### C-31-5-2 — #687 et #645 écartées

- **Contexte** : les deux sont réelles. #687 : l'issue ne propose « aucune [piste] mécanique » (arrondi à la création
  qui change HT/TVA et la complétion d'une pièce importée, ou refonte de `pay_in_tx`). #645 : définir les jetons change
  l'apparence de tout l'écran et l'accessibilité, l'issue demande une story propre.
- **Retenu** : écartées, restent ouvertes ; #687 est la plus utile à Guy des deux (factures fournisseur réelles) et
  mérite une story à elle.
- **Écartée** : forcer quatre parmi les six en y mettant #687 à la place de #504 ou #469 — ce serait prendre la moins
  courte et la plus risquée pour tenir un chiffre.
- **Réversible** : oui.

### C-31-5-3 — #707 : corriger la démonstration, pas le chemin production

- **Contexte** : `insert_with_defaults_in_tx` sert la démonstration **et** la finalisation production ; aucun des deux ne
  désigne les comptes de TVA (colonnes nullables depuis la 18-1a, `18-1a-comptes-tva.md:109-110` ; refus de validation
  sans TVA due décidé par la 18-1b, `18-1b-comptabilisation-ventes.md:122-125`, AC5 — la désignation par
  l'administrateur en est la conséquence ; « voulu depuis la 18-1a » jusqu'à la validation P3, F5). Pour un non-assujetti (Guy), le refus `CONFIGURATION_REQUIRED`
  d'une facture saisie par erreur avec un taux est un garde-fou.
- **Retenu** : une fonction distincte appelée par `seed_demo` seulement (deux depuis la validation P1, C-31-5-13 : la lecture verrouillante des comptes, puis l'application) ; le chemin production garde `NULL`, et le manuel
  le dit (AC3) ; un test fige ce comportement production.
- **Écartées** : désigner les comptes dans `insert_with_defaults_in_tx` (change le comportement de toute installation
  neuve, retire le garde-fou du non-assujetti) ; manuel seul pour la démonstration (l'issue admet cette voie, mais elle
  attend de la démonstration un essai sans réglage — formule de l'issue, non du produit, annotation de la validation
  P2 —, et le correctif est court).
- **Réversible** : oui.

### C-31-5-4 — #707 : n'agir que sur une ligne de réglages née dans la transaction

- **Contexte** : `insert_with_defaults_in_tx` rend `inserted` ; si la ligne existait (`false`), elle a pu être lue et
  modifiée par quelqu'un, et une mise à jour sans `version + 1` casserait le verrou optimiste.
- **Retenu** : appel conditionné à `invoice_settings_created` ; `COALESCE` en plus (aucun écrasement). Mutation M4
  probablement équivalente, à déclarer.
- **Écartée** : mettre à jour toujours, avec `version + 1` — gère un cas que la démonstration (étape 2 → 3) ne produit
  pas.
- **Réversible** : oui.

### C-31-5-5 — #707 : l'entrée d'audit `installation.demo_seeded` reste telle quelle

- **Contexte** : elle porte `invoice_settings_created`, pas le détail des comptes.
- **Retenu** : inchangée (périmètre minimal).
- **Écartée** : y ajouter les trois comptes — utile mais hors du défaut.
- **Réversible** : oui.

### C-31-5-6 — #589 : la capture montre l'écran, la valeur de l'infobulle se lit au DOM

- **Contexte** : la recette visuelle exige une capture de tout contrôle visible touché ; une infobulle native `title`
  n'apparaît pas dans une capture Playwright.
- **Retenu** : capture de l'écran à trois exercices clos (boutons désactivés/actif) + valeur de `title` relevée au DOM et
  recopiée au Dev Agent Record ; Vitest l'assure.
- **Écartée** : remplacer l'infobulle native par un composant d'infobulle pour pouvoir la capturer — change le contrôle.
- **Réversible** : oui.

### C-31-5-7 — #504 : la traduction ne touche pas le rapprochement

- **Contexte** : l'issue demande de vérifier qu'aucun rapprochement ne s'appuie sur « Facture ».
- **Retenu** : établi au code — `reference_score` (`kesh-reconciliation/src/matching.rs:231-256`) compare le numéro de
  facture normalisé par contenance ; le préfixe n'y entre pas. Aucun test de rapprochement à ajouter.
- **Écartée** : garder le français « pour la banque » — la communication est lue par le débiteur, dans sa langue.
- **Réversible** : oui.

### C-31-5-8 — #504 : une clé dédiée, sans variable

- **Contexte** : réutiliser `invoice-pdf-title` (même valeur aujourd'hui) couplerait le texte lu par la banque au titre
  imprimé ; une clé `Facture { $number }` ferait entrer les marques d'isolation Fluent (U+2068/U+2069) dans la QR,
  aucun `set_use_isolating(false)` n'étant posé.
- **Retenu** : `invoice-qr-message-prefix` sans placeable, `format!("{prefix} {n}")` côté Rust, dans `kesh-api` (pas dans
  `I18N_KEYS` de `kesh-qrbill`, qui n'est pas touché).
- **Écartées** : `invoice-pdf-title` ; clé à variable ; clé ajoutée à `kesh-qrbill` (un sixième module pour rien).
- **Réversible** : oui.

### C-31-5-9 — #504 : pas de refigeage des PDF émis

- **Contexte** : depuis la 25-6-b, le PDF d'une facture est figé au premier rendu ; refigeage manuel seulement.
- **Retenu** : les PDF figés gardent leur communication française ; manuel le dit (AC10).
- **Écartée** : refigeage d'office (change une pièce émise, ce que la 25-6-b interdit).
- **Réversible** : sans objet.

### C-31-5-10 — #469 : le détail `chrono` d'une date mal formée n'est plus montré

- **Contexte** : `parse_date` renvoie aujourd'hui `{nom} invalide ({e})`, `e` étant le message anglais de `chrono`.
- **Retenu** : message traduit qui nomme le paramètre et le format attendu (`AAAA-MM-JJ`), sans `e`.
- **Écartée** : garder `e` en variable — un fragment anglais dans un message allemand ou italien.
- **Réversible** : oui.

### C-31-5-11 — #469 : `t_args` passe en `pub(crate)`

- **Contexte** : seule `t` est exposée ; `t_args` retire déjà les marques d'isolation.
- **Retenu** : élargir sa visibilité (DRY : pas de seconde fonction qui ferait la même chose dans `audit_log.rs`).
- **Écartée** : `format!` sur le rendu de `t` — perd l'ordre des mots propre à chaque langue.
- **Réversible** : oui.

### C-31-5-12 — Classe voisine non traitée : les autres `AppError::Validation` en français

- **Contexte** : des dizaines de routes rendent des `AppError::Validation` écrits en français ; #469 ne couvre que le
  journal d'audit et aucune issue ouverte ne couvre le reste.
- **Retenu** : hors périmètre ; issue demandée à l'orchestrateur (« on trace toujours »), même si elle fait remonter
  le compte — **ouverte le 2026-10-24 : #710** (annotation de la validation P1 ; comprise dans le recompte de 141).
- **Écartée** : les traduire ici — ni mécanique ni court (283 lignes hors du journal d'audit au relevé grossier du
  2026-10-24, `grep -rn "AppError::Validation(" crates | grep -v "t(" | grep -v audit_log.rs | wc -l` ; 289 sans le
  dernier filtre, chiffre écrit ici jusqu'à la validation P2, qui comptait les six d'`audit_log.rs`).
- **Réversible** : sans objet.

### C-31-5-13 — #707 : les comptes de TVA se verrouillent AVANT la ligne de réglages (validation P1, R4/F4)

- **Contexte** : la fonction prescrite à la rédaction verrouillait les trois comptes de TVA **après** l'`INSERT IGNORE`
  de `insert_with_defaults_in_tx`, d'où `accounts → company_invoice_settings → accounts`, contraire à l'ordre écrit
  pour `seed_demo` (`docs/MULTI-TENANT-SCOPING-PATTERNS.md:326`, doc de `final_body`). L'argument de la fiche (aucun
  concurrent sur une ligne neuve) ne couvrait pas une transaction tenant l'un des comptes.
- **Retenu** : deux fonctions — `lock_vat_role_accounts_in_tx` (lecture verrouillante des trois comptes, appelée avant
  `insert_with_defaults_in_tx` ; sa forme — une requête `IN (…) ORDER BY id FOR UPDATE` à la validation P1 — est
  remplacée par trois lectures ponctuelles à la validation P2, C-31-5-18) et `designate_vat_accounts_in_tx` (un `UPDATE ... COALESCE` de la
  ligne de réglages, sans lecture de compte, appelée après, si `invoice_settings_created`). L'ordre documenté est tenu
  sans exception ; la ligne 326 et le doc de `final_body` sont précisés (comptes de TVA inclus) dans la même PR.
- **Écartées** : garder l'ordre inverse et écrire l'exception (risque faible, mais une exception de plus au Pattern 5
  pour un gain nul) ; ajouter un paramètre à `insert_with_defaults_in_tx` (toucherait le chemin production, C-31-5-3).
- **Réversible** : oui.

### C-31-5-14 — Contrôles de l'AC16 décidables (validation P1, R1/F2/F3)

- **Contexte** : `grep -rn 'format!("Facture' crates` rend deux lignes, dont `supplier_invoices.rs:512` (libellé
  d'écriture de production), et les deux « exceptions » nommées (`:816`, `qr_decode.rs:225`) ne contiennent pas
  `format!` ; le contrôle d'`audit_log.rs` était ligne à ligne, faux dès que rustfmt met l'argument à la ligne suivante,
  et contredisait l'AC11.
- **Retenu** : (a) contrôle #504 = zéro ligne hors `crates/kesh-db/src/repositories/supplier_invoices.rs` **et** compte
  de `format!("Facture fournisseur` dans ce fichier égal à 1 — l'exception unique et nommée, avec sa raison ;
  (b) contrôle #469 multi-ligne par aplatissement (`tr '\n' ' '`, jeton qui suit `AppError::Validation(`), six jetons,
  tous `t`/`t_args` après correctif ; le **garde** reste le test de l'AC12, l'AC11 renvoie à l'AC16.
- **Écartées** : `grep -Pzo` (moins lisible, dépend de PCRE) ; supprimer le contrôle et ne garder que le test.
- **Réversible** : oui.

### C-31-5-15 — Les six clés du journal d'audit entrent à la garde G9 (validation P1, F9)

- **Contexte** : `les_replis_rust_suivent_le_catalogue` (G9, `crates/kesh-api/tests/textes_coherents.rs:680`) est une
  table fermée ; sans entrée, les replis Rust des six `t`/`t_args` pourraient dériver du catalogue `fr-CH`.
- **Retenu** : une quatrième entrée `crates/kesh-api/src/routes/audit_log.rs` avec les six clés (paramètres
  `{ $field }` → `{nom}`, `{ $max }` → `{maximum}`). La clé `invoice-qr-message-prefix` n'y entre pas : sans repli Rust.
- **Écartée** : laisser G9 telle quelle (facultatif, mais une dérive silencieuse pour quelques lignes).
- **Réversible** : oui.

### C-31-5-16 — Le budget d'issues n'est pas tenu par cette story : déficit déclaré, arbitrage au Project Lead (validation P1, R3/F1 ; rendu inconditionnel à la validation P2, F2 ; recompté à la validation P3, F4 = R8)

- **Contexte** : *(relevé de la validation P1, daté ; recompte vivant à l'Origine de la fiche : 143 ouvertes à la
  validation P3, #711 et #712 ouvertes, 138 attendues au tag)* recompte du 2026-10-24 (validation P1) : 141 ouvertes (#708, #709, #710 ouvertes ce jour) ; − #503 − 4 = 136, pas sous
  le plafond « moins de 136 » ; + l'issue annoncée des LOW résiduels de la 31-1c = 137 ; + l'issue des libellés
  d'écriture (C-31-5-17), non ouverte, si elle s'ouvre = 138.
- **Retenu** : écrire les deux comptes dans l'*Origine* — **137** attendues au tag si rien d'autre ne s'ouvre (deux
  retraits manquent), **138** si l'issue des libellés s'ouvre (trois manquent) — et les soumettre au Project Lead par
  l'orchestrateur : d'autres défauts corrigés ou publication au-dessus du plafond. Aucune fermeture pour le chiffre.
  *(La validation P1 n'écrivait que 137 et « deux retraits », sans l'issue que la fiche soumet elle-même : chiffre
  conditionnel sans le dire, F2 de la P2.)*
- **Recompte de la validation P3** (2026-10-24, `gh issue list --state open --limit 500 --json number --jq length`) :
  **143** ouvertes, #711 (LOW résiduels de la 31-1c) et #712 (libellés d'écriture, C-31-5-17) comprises ;
  143 − #503 − les quatre = **138** attendues au tag ; plafond < 136 : **trois** retraits manquent ; arbitrage au
  Project Lead. Plus aucune hypothèse : les deux issues annoncées sont ouvertes, et la règle de type des comptes de TVA
  n'en demande pas (C-31-5-19). Les comptes 137/138 ci-dessus sont des relevés datés de la P2.
- **Écartée** : ajouter d'office #687 ou #645 (écartées pour risque, C-31-5-2) ; taire le déficit.
- **Réversible** : sans objet (constat).

### C-31-5-17 — Libellés d'écriture en français en dur : hors périmètre, tracés par #712 (validation P1 ; issue ouverte à la validation P3)

- **Contexte** : `supplier_invoices.rs:512` et `:941`, `invoice_settlements_write.rs:277`, `credit_notes.rs:590`
  composent en dur des descriptions d'écritures en français ; ce ne sont pas des textes adressés au débiteur (hors
  classe de #504) ; aucune issue ouverte ne les couvre.
- **Retenu** : inventoriés au *Hors périmètre* ; tracés par **#712** (« Les libellés des écritures générées (facture
  fournisseur, règlement, avoir) sont écrits en dur en français »), ouverte le 2026-10-24 par l'orchestrateur en
  application de « on trace toujours » (F2 de la validation P3), comprise dans le recompte de 143 (C-31-5-16). La
  question de savoir s'ils doivent suivre la langue comptable de la société reste à trancher sur l'issue. Seul
  `supplier_invoices.rs:512` intéresse l'AC16 (exception nommée). *(Jusqu'à la validation P2 : « non ouverte, soumise au
  Project Lead ».)*
- **Écartée** : les traduire ici (ni mécanique ni dans le défaut).
- **Réversible** : oui.

### C-31-5-18 — #707 : trois lectures ponctuelles, l'ordre des verrous est celui du code (validation P2, R1 = F1)

- **Contexte** : la P1 prescrivait `… singleton_role IN (?, ?, ?) ORDER BY id FOR UPDATE` en annonçant un « ordre des
  verrous par clé primaire ». C'est faux : InnoDB pose ses verrous au fil de l'index parcouru —
  `uq_accounts_company_singleton_role (company_id, singleton_role)`, ordre alphabétique des rôles —, le tri vient après ;
  et l'ensemble même des verrous dépend du plan (accès par plages sur l'index unique, ou par le seul préfixe
  `company_id`, verrous *next-key* sur tous les comptes de la société). Le remède du dépôt, `FORCE INDEX (PRIMARY)`
  (`company_invoice_settings.rs:897-905`), suppose une liste d'`id`, que la requête n'a pas.
- **Retenu** : trois lectures `SELECT id, account_type, postable FROM accounts WHERE company_id = ? AND singleton_role = ?
  FOR UPDATE`, sur le patron d'`insert_with_defaults_in_tx` (`:1076-1105`), dans l'ordre écrit dans le code —
  `VatPayable`, `VatRecoverable`, `VatSettlement` (ordre permuté à la validation P3 en `VatRecoverable`, `VatPayable`,
  `VatSettlement`, C-31-5-21) ; chacune est un accès `const` à l'index unique, verrouille l'entrée de
  l'index puis l'enregistrement du compte, et ne laisse qu'un verrou d'intervalle pour un rôle absent. Le doc-comment, les
  AC1, Dev Notes et T2 disent cet ordre-là, sans promettre l'ordre des identifiants. L'ordre **entre tables**
  (`accounts → company_invoice_settings`, Pattern 5) reste tenu.
- **Écartées** : garder l'`IN` en corrigeant seulement le texte (plan dépendant de l'optimiseur, verrous possiblement
  étendus à toute la société) ; la forme en deux temps du précédent (lecture non verrouillante des `id`, puis
  `FORCE INDEX (PRIMARY) … id IN (…) ORDER BY id FOR UPDATE`) — rien, dans cette transaction, ne requiert l'ordre
  par `id`, et la lecture préalable non verrouillante ouvre une fenêtre entre les deux temps.
- **Réversible** : oui.

### C-31-5-19 — #707 : le filtre de type est celui du réglage, écrit une fois en `kesh-db` ; test et mutations des filtres (validation P2, R2, F10)

- **Contexte** : R2 — la branche « un rôle sans compte utilisable donne `None` » n'avait ni test ni mutation, alors
  qu'elle peut se produire (`VatPayable` admis sur un actif par la règle du rôle ; fenêtre de #538 entre la validation
  du plan et la dernière transaction). F10 — la fiche recopiait les types de la route, troisième copie d'une règle de
  type ; l'orchestrateur demandait de réutiliser `accepts_account_type` (`accounts.rs:89`). Établi au code :
  `accepts_account_type` (`kesh-core/src/chart_of_accounts/mod.rs:161`) est la règle du **rôle** — `Asset` ou
  `Liability` pour les trois rôles de TVA, appliquée à l'écriture de tout compte qui porte un rôle ; la réappliquer est
  une tautologie et laisse passer un `VatPayable` porté par un actif, que la route refuse (`Liability` exigé,
  `routes/company_invoice_settings.rs:337-364`). La règle du **réglage** n'existe qu'à la route, dans `kesh-api`, que
  `kesh-db` n'importe pas.
- **Retenu** : les trois types du réglage écrits **une fois** dans `kesh-db`, en une constante nommée
  (`VAT_SETTING_ACCOUNT_TYPES`, C-31-5-22), **employée par la route** depuis la validation P3 (F1 = R4) : la route
  (`kesh-api/src/routes/company_invoice_settings.rs`) importe déjà `kesh_db::entities::account::AccountType` (l. 25-26)
  et `kesh_db::repositories::company_invoice_settings` (l. 28), et ses trois appels de `validate_account` (l. 337-364)
  passent `expected: AccountType` par valeur (`Copy`) : trois arguments remplacés, comportement inchangé, une seule
  copie de la règle. Le sens de dépendance utile est `kesh-api → kesh-db`, que la P2 n'avait pas examiné. Précédent à
  ne pas reproduire : `designated_account_id` (`kesh-db/src/repositories/company_invoice_settings.rs:495-520`), deux
  copies (« Mêmes exigences que sa validation (`validate_account_of`, `kesh-api`) ») qu'aucun garde ne tient d'accord.
  *(Jusqu'à la validation P2 : « la route n'est pas touchée », constante « jumelle » de la route.)* Test `kesh-db` de la lecture à
  quatre cas (heureux ; `VatPayable` retypé en actif ; `VatRecoverable` non imputable ; `VatSettlement` archivé) ;
  mutations M7a (filtre de type retiré) et M7b (filtre `postable` retiré), chacune tuée par un cas nommé ; M8 pour le
  test de garde du chemin production (R3).
- **Écartées** : réutiliser `accepts_account_type` (filtre inopérant, cf. contexte — **écart assumé à la consigne de
  l'orchestrateur**, signalé dans le rapport) ; déplacer la règle du réglage en `kesh-core` et y brancher la route (un
  crate de plus pour un résultat qu'atteint la constante de `kesh-db`, déjà importable par la route) ; garder deux
  copies avec un test de garde `kesh-api` (deux copies et un test pour ce qu'un import supprime) ; garder deux copies
  et ouvrir une issue (dette évitable, F1 de la P3).
- **Réversible** : oui.

### C-31-5-20 — #469 : préfixe `audit-log-query-error-*` pour les six refus du serveur (validation P2, R9)

- **Contexte** : `audit-log-filter-*` (`fr-CH` l. 2719-2726) désigne les libellés du formulaire de filtre de l'écran, et
  `audit-log-error-date-range` (l. 2739) le contrôle de plage fait par l'écran ; `audit-log-api-key` (l. 2734) occupe
  déjà `audit-log-api-`. `grep -rn "audit-log-query" crates frontend/src` ne rend rien. Le précédent serveur du même
  fichier est `audit-log-export-error-too-large`.
- **Retenu** : `audit-log-query-error-*` (les refus portent sur les paramètres de la requête de consultation), dans
  l'espace `audit-log-` comme le précédent.
- **Écartées** : `audit-log-filter-error-*` (mêle refus du serveur et libellés de l'écran) ; `audit-log-api-error-*`
  (voisin de `audit-log-api-key`, sans rapport) ; `error-audit-log-*` (rompt avec le précédent du fichier).
- **Réversible** : oui (clés neuves, aucune lue par le frontend).

### C-31-5-21 — #707 : ordre des trois lectures permuté, `VatRecoverable` d'abord (validation P3, R3)

- **Contexte** : l'ordre retenu à la P2 (`VatPayable`, `VatRecoverable`, `VatSettlement`, soit 2200, 1171, 2206 au plan
  PME) est l'inverse, pour ses deux premiers comptes, du verrou partagé par identifiant croissant de
  `lock_designated_accounts_in_tx` (`company_invoice_settings.rs:865-916`, 1171 puis 2200), seul ordre opposé identifié *(prémisse de la P3, réfutée en P4 : voir « Rectifié » ci-dessous)* ;
  `bulk_create_from_chart_in_tx` crée les comptes par longueur puis numéro, d'où 1171 < 2200 < 2206 en identifiants.
- **Rectifié (validation P4, P4-1)** : l'« opposition » invoquée n'existe pas — les appelants de
  `lock_designated_accounts_in_tx` prennent `SALE` (créance, `VatPayable`) ou `PURCHASE` (dette, `VatRecoverable`),
  jamais 1171 et 2200 ensemble (`company_invoice_settings.rs:696-700`, vérifié par l'orchestrateur). La permutation est
  gardée pour la seule lisibilité (ordre des identifiants au plan standard), sans effet sur les interblocages.
- **Retenu** : `VatRecoverable`, `VatPayable`, `VatSettlement` — sans coût, il suit au plan standard l'ordre des
  identifiants ; le doc-comment dit qu'il coïncide avec l'ordre des identifiants au plan
  standard sans le garantir (un rôle réattribué peut le défaire), et le rejeu `is_seed_retryable` reste le filet.
- **Écartée** : garder l'ordre de la P2 (aussi sûr, l'opposition invoquée n'existant pas — P4-1 ; écartée pour la seule lisibilité).
- **Réversible** : oui.

### C-31-5-22 — #707 : définitions de `VatRoleAccounts` et de la constante des types (validation P3, F6)

- **Contexte** : l'AC1 prescrivait un type de retour et une constante sans nom, champs ni emplacement.
- **Retenu** : dans `crates/kesh-db/src/repositories/company_invoice_settings.rs`, `pub`, atteignables par
  `kesh_db::repositories::company_invoice_settings::…` : `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] pub
  struct VatRoleAccounts { pub payable, pub recoverable, pub settlement: Option<i64> }` (les trois réglages, dans
  l'ordre des colonnes) ; `pub struct VatSettingAccountTypes { pub payable, pub recoverable, pub settlement: AccountType }`
  et `pub const VAT_SETTING_ACCOUNT_TYPES` = `Liability` / `Asset` / `Liability`. Lecture typée
  `(i64, AccountType, bool)` (`AccountType: Decode<MySql>`, `entities/account.rs:43-66`).
- **Écartées** : un tableau `[AccountType; 3]` indexé (la position d'un réglage n'a pas de nom, une erreur d'indice ne
  rougit pas) ; trois constantes isolées (trois noms à importer pour une seule règle).
- **Réversible** : oui (noms internes, aucun n'est d'API).
