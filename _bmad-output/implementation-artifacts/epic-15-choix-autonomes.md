# Epic 15 — registre des choix faits en autonomie

Le 2026-10-08, Guy a confié l'Epic 15 en développement **entièrement autonome** : aucune
question en cours de route, les choix tranchés par l'orchestrateur et **présentés ici, pour
sa revue en fin d'epic**.

Chaque entrée est écrite **au moment où le choix est fait**, pas reconstituée à la fin. Elle
dit le contexte, l'option retenue, les options écartées, et ce que coûterait de revenir
dessus.

**Autorisations reçues le 2026-10-08** : pousser, ouvrir et merger (squash) les PR une fois
la CI verte ; publier la release v0.13.0 ; conduire seul la revue de projet après
sauvegarde ; traiter dans l'epic les constats de recette P1/P2 de la v0.12.1.

**Décompte des bugs au kickoff** (§ *Priorités des défauts* du `CLAUDE.md`) : **49**, dont
14 P1 (9 de la TVA, en report assumé), 11 P2, 16 P3, 8 P4. Commande :
`gh issue list --state open --limit 500 --json labels --jq '[.[]|select([.labels[].name]|any(.=="bug" or .=="known-failure"))]|length'`

---

## C1 — Périmètre de l'epic

- **Retenu** : le jalon GitHub « Épique 15 » tel qu'il est (#235, #458, #459, #460, #461),
  **plus** les cinq P1 hors TVA (#427, #429, #434, #473, #474), **plus** le lettrage, qui
  n'avait pas d'issue et en reçoit une. L'epic se clôt quand le jalon est à zéro.
- **Sortent de l'epic** : 15-3 (versioning des parseurs, modèles, manuels embarqués) et
  15-4 (journaux personnalisables). Aucune issue ne les porte, aucune n'est un défaut, et
  l'epic livre déjà deux fonctionnalités (lettrage, justificatifs). Elles retournent au
  backlog, sans epic.
- **Écarté** : tout le périmètre d'origine de `epics.md`. L'epic aurait compté douze
  stories, et l'engagement « le nombre de bugs baisse » passe avant.
- **Réversible** : oui, en remettant les deux stories dans le registre.

## C2 — Ordre des stories

Les trois corrections P1 d'abord (15-5, 15-6, 15-7) : elles sont courtes, indépendantes,
et elles font baisser le décompte dès le début. Puis le lettrage (15-1a → b → c), les
justificatifs (15-2), et enfin les quatre issues du jalon sur les factures fournisseurs et
l'import (#458–#461).

| Story | Objet | Issues |
|---|---|---|
| 15-5 | Gardes de postabilité côté serveur : réconciliation, règles, réglages de facturation | #427, #429 |
| 15-6 | La créance juste : l'avoir crédite le compte de la vente ; un règlement ne peut viser le compte débiteurs | #473, #474 |
| 15-7 | La piste de contrôle de l'onboarding | #434 |
| 15-1a/b/c | Lettrage — socle, vue « ce qui reste ouvert », proposition et écran | (nouvelle issue) |
| 15-2 | Pièces justificatives sur les écritures | #235 |
| 15-8 | Import automatique du dossier, et le manuel qui le promettait | #459, #458 |
| 15-9 | Cycle de paiement des factures fournisseurs ; état « envoyée » des factures clients | #460, #461 |

*(La numérotation 15-8 et 15-9 peut changer à leur spécification ; ce registre le dira.)*

## C3 — 15-5 : la forme du refus « compte non imputable »

- **Contexte** : le commentaire de #429 relève que toutes les gardes de postabilité refusent avec
  `InactiveOrInvalidAccounts` (« archivés ou invalides »), message que le dépôt avait déjà rejeté en
  16-1a pour ce motif. La 15-5 ajoute de nouvelles gardes : il faut décider sous quel nom elles
  refusent.
- **Retenu** : une variante `DbError::AccountsNotPostable(Vec<String>)` (numéros des comptes), code
  HTTP 400 `ACCOUNT_NOT_POSTABLE`, message traduit qui nomme la cause et les comptes. Appliquée aux
  nouvelles gardes **et** aux existantes — saisie manuelle (14-3b) et trois gardes de la 24-5. Les
  réglages de facturation gardent leur style `VALIDATION_ERROR` nommant le champ, déjà employé pour
  leurs comptes désignés.
- **Écartées** : garder `InactiveOrInvalidAccounts` (le défaut signalé reste) ; une variante par
  surface (trois codes pour une même cause) ; réutiliser `RevenueAccountRejection` (propre aux
  lignes de facture).
- **Conséquence visible** : pour un compte non imputable, la saisie manuelle rend désormais
  `ACCOUNT_NOT_POSTABLE` au lieu de `INACTIVE_OR_INVALID_ACCOUNTS` — changement de contrat pour une
  intégration par clé d'API, à écrire au CHANGELOG. Le test `reports_e2e` qui figeait l'ancien code
  (AC 13 de la 24-5) est réécrit à dessein.
- **Réversible** : oui, avant release ; après, le code d'erreur fait partie du contrat.

## C4 — 15-5 : où s'applique l'exemption d'un compte déjà en place

- **Contexte** : #427 demande de reprendre « le mécanisme d'exemption pour un compte déjà
  référencé » (#271), à juger surface par surface.
- **Retenu** : exemption « contrôlé seulement si la valeur change » pour les réglages de
  facturation (six champs historiques), le compte d'un compte bancaire (PUT, PATCH) et le PATCH d'une
  règle de rapprochement — patron de `resolve_designated_account` et de la décision D4 des articles.
  **Aucune** exemption pour les rapprochements manuels, ventilés et proposés : le compte y est choisi
  à l'instant. **Aucune** pour l'acceptation par règle : une règle dont le compte est devenu non
  imputable (scindé en sous-comptes) est périmée ; elle n'est plus proposée et son acceptation est
  refusée.
- **Écartées** : exempter l'acceptation par règle (ce serait poster sur un compte de regroupement,
  le défaut même) ; contrôler les réglages même inchangés (bloquerait tout enregistrement après
  l'ajout d'un sous-compte, sur un champ non touché).
- **Réversible** : oui, une condition par site.

## C5 — 15-5 : le compte comptable d'un compte bancaire entre dans la story, pour sa postabilité seule

- **Contexte** : #474 note que `bank_accounts.rs` accepte n'importe quel compte d'actif comme compte
  d'un compte bancaire. Deux défauts s'y mêlent : la postabilité (famille de #427/#429) et le compte
  débiteurs (fond de #474).
- **Retenu** : la 15-5 ajoute la garde de **postabilité** (avec exemption « inchangé ») aux trois
  routes du compte bancaire ; le refus du compte débiteurs reste à la 15-6.
- **Écartée** : tout laisser à la 15-6 (la même famille serait traitée par deux stories, et la 15-6
  hériterait d'un trou que la 15-5 prétend fermer).
- **Réversible** : oui.

## C6 — 15-5 : les angles morts assumés

- **Retenu, hors périmètre et écrit dans la story** : la fiche article (la décision D3 exclut
  `postable` ; la garde vit sur la ligne de facture — seule sa justification, devenue fausse, est
  réécrite) ; le compte bancaire **à l'usage** (D-A0 : un compte de configuration devenu non
  imputable continue de servir) ; les comptes désignés par **rôle** à la création des réglages
  (choix de l'application) ; l'avoir (snapshot de la facture, D5-bis).
- **Écartée** : étendre la garde à la fiche article — sans écriture comptable propre, elle aurait
  ajouté une surface et une exemption pour rien.
- **Réversible** : oui.

## C7 — 15-5 : découpage en 15-5a et 15-5b

- **Contexte** : la passe de validation P1 de la 15-5 (findings M5 et C-4) relève que la story touche
  plus de cinq modules — `kesh-db`, `kesh-api` (erreurs et cinq modules de routes), `kesh-i18n`,
  `frontend`, manuels — et que la fiche se dispensait de la règle de splitting préventif par l'argument
  « les gardes sont mécaniques », qui n'est pas une dérogation codifiée (la seule : les cycles Cargo).
- **Retenu** : découper selon le patron « story-zéro qui pose le patron + rollout ».
  **15-5a-refus-non-imputable** : variante `DbError::AccountsNotPostable`, code `ACCOUNT_NOT_POSTABLE`,
  message (4 locales, pluriel), conversion de la saisie manuelle, de l'écriture d'ouverture et des trois
  gardes de la 24-5, tests qui figeaient l'ancien code, ordre des causes, détail structuré, CHANGELOG ;
  `refs #427 refs #429`. **15-5b-gardes-surfaces-neuves** : rapprochement, règles, réglages de
  facturation, compte bancaire, écran des refus par lot, manuel ; `closes #427 #429 #492 #519`. La 15-5b
  dépend de la 15-5a. La 15-5 passe `split`, corps vidé.
- **Écartées** : garder la story unique avec une section « Dérogation » (l'argument n'est pas une
  dérogation prévue) ; découper par surface (cinq stories pour une même garde).
- **Conséquence** : la 15-5b reste au-dessus de cinq modules — c'est la nature d'un rollout, que la règle
  prévoit de revoir fichier par fichier ; c'est le mode de revue demandé dans la fiche.
- **Réversible** : oui, en refusionnant les fiches avant tout développement.

## C8 — 15-5b : les refus par lot du rapprochement, lisibles

- **Contexte** : findings B1/M2. `ReconciliationProposals.svelte:362` affiche `failed[]` en code brut ;
  les refus neufs de la 15-5b (`ACCOUNT_NOT_POSTABLE`) s'y liraient tels quels. L'issue #492 décrit le
  même défaut pour tous les codes.
- **Retenu** : la 15-5b ajoute un libellé traduit par `errorCode` pour **tous** les codes de `failed[]`
  (26 relevés), sur le patron `failedItemLabel` (clés littérales, repli avec le code brut) ; elle ferme
  #492. La phrase « Aucun changement d'écran n'est requis » est supprimée.
- **Écartées** : ne traduire que `ACCOUNT_NOT_POSTABLE` (un écran à moitié traduit, #492 resterait
  ouvert pour le reste) ; assumer l'angle mort (le refus neuf serait illisible).
- **Réversible** : oui.

## C9 — 15-5b : la règle de rapprochement dont le compte n'est plus imputable

- **Contexte** : findings C-2/M3. L'exemption « inchangé » du PATCH (C4) laissait un PATCH
  `active:true` ressusciter une règle dont le compte est devenu non imputable ; la fiche ne disait pas
  ce que l'utilisateur voit des règles déjà en base.
- **Retenu** : une telle règle n'est plus proposée (AC5) ; son acceptation est refusée (AC4) ; sa
  **réactivation** (`before.active = false`, `PATCH active:true`) est refusée en `ACCOUNT_NOT_POSTABLE`
  même si le compte est inchangé ; une règle déjà active sur un tel compte reste en base, listée, éditable
  et désactivable — **sans migration** ; le manuel le dit. Test dédié.
- **Écartées** : une migration qui désactive ces règles (écrit des données pour un cas que l'écran gère,
  et P7 imposerait un triage de rejeu) ; refuser tout PATCH d'une telle règle (on ne pourrait plus la
  désactiver ni la renommer).
- **Réversible** : oui, une condition dans `update_in_tx`.

## C10 — 15-5b : où se place le contrôle « inchangé »

- **Contexte** : findings B2/C-6. Lire la valeur en place **avant** la validation du compte, hors
  transaction, aurait changé l'ordre des erreurs existant (un compte bancaire inconnu aurait rendu une
  erreur de compte avant `BankAccountNotFound`) et lu une valeur possiblement périmée.
- **Retenu** : pour le compte bancaire (PUT, PATCH), le contrôle de postabilité se fait **dans la
  transaction**, dans le dépôt, sous le verrou `FOR UPDATE` que `update_for_company` et
  `set_journal_account_id_for_company` posent déjà, sur la valeur en place qu'ils lisent ; l'ordre des
  erreurs existant est conservé et le nouveau refus vient en dernier. Même principe pour la règle, dans
  `reconciliation_rules::update_in_tx`, à côté du contrôle « si changé » du projet par défaut. L'ordre est
  écrit dans les AC et testé.
- **Écartée** : la lecture préalable dans le handler (proposée par la fiche d'origine).
- **Réversible** : oui.

## C11 — 15-5b : le manuel des règles d'affectation

- **Contexte** : l'issue #519 (ouverte par l'orchestrateur, finding L5) constate que
  `user-manual.tex` § *Règles d'affectation automatique* décrit un écran et un comportement inexistants
  (écritures brouillon à l'import, option *auto-validate rules*).
- **Retenu** : la 15-5b réécrit la section sur le comportement réel — elle touche ces règles, et le
  manuel doit dire ce qu'elle change (C9) ; elle ferme #519.
- **Écartée** : laisser #519 à une story documentaire séparée (la 15-5b aurait écrit une phrase juste au
  milieu d'une section fausse).
- **Réversible** : sans objet (documentation).

## C12 — 15-5b : le compte lié reste visible dans le formulaire du compte bancaire

- **Contexte** : finding C-7 (« à vérifier »). Vérifié au code de Svelte 5.55 (`bindings/select.js`) :
  quand la valeur liée d'un `<select>` n'est dans aucune option, Svelte pose `selectedIndex = -1` —
  champ vide — mais ne réécrit pas la variable ; le PUT renvoie donc l'identifiant en place et
  l'exemption joue. Le défaut est d'affichage : le lien existe, le champ paraît vide (cas de #271).
- **Retenu** : la 15-5b passe les deux `<select>` de `bank-accounts/+page.svelte` à `withCurrentAccount`,
  comme `BankAccountJournalLinkForm.svelte` le fait déjà.
- **Écartée** : écrire le fait et ne rien changer (l'exemption serait juste, l'écran trompeur).
- **Réversible** : oui.

## C13 — 15-5a/15-5b : la forme du détail et l'ordre des causes

- **Contexte** : findings L2/B4 et M4. Le détail structuré du refus n'était pas fixé (la fiche demandait
  `accountIds` dans `failed[]`, la variante porte des numéros, le 404 voisin porte `missingAccountIds`) ;
  l'ordre entre « non imputable » et « mauvais type » n'était pas tranché.
- **Retenu** : `details.accountNumbers` (numéros triés, dédupliqués) **partout** où
  `ACCOUNT_NOT_POSTABLE` est émis — corps du 400 et `failed[].details`. Les numéros ne révèlent rien :
  le refus n'est émis que pour un compte de la société, actif. Ordre : inconnu / autre société / archivé
  / mauvais type d'abord (`InactiveOrInvalidAccounts` ou 404), non imputable ensuite ; sur une liste de
  lignes, le refus le plus bloquant gagne et le refus non imputable nomme tous les comptes.
- **Écartée** : des identifiants (`accountIds`) — l'utilisateur ne les voit nulle part.
- **Réversible** : oui avant release ; après, la clé fait partie du contrat.
- *Renvoi (ajouté le 2026-10-08, validation P3 de la 15-5a, finding R3-3 ; l'entrée n'est pas
  réécrite)* : la clé `details.accountNumbers` est **révisée par C16** —
  `details.rejected[{accountId, accountNumber}]`. L'ordre des causes, lui, reste en vigueur.

## C14 — 15-5b : les refus de l'écran des règles ne doivent pas s'afficher « [object Object] »

- **Contexte** : trouvé en refaisant l'axe « écrans » pendant la remédiation de P1. Le client d'API lève
  un `ApiError` objet simple, pas une instance d'`Error` ; `RuleFormModal.svelte:104` et
  `RulesList.svelte:64`, `:87` font `e instanceof Error ? e.message : String(e)` — le refus que la 15-5b
  ajoute à la création, à la modification et à la réactivation d'une règle s'afficherait
  « [object Object] ».
- **Retenu** : la 15-5b passe ces trois `catch`, ainsi que trois de `ReconciliationProposals.svelte` et
  celui de `reconciliation/rules/+page.svelte` (même motif, même module), au patron `isApiError` de
  `ManualMatchModal.svelte`. Les sites hors module (`reports/+page.svelte:210`,
  `settings/+page.svelte:236`, `:260`) sont signalés pour une issue, hors périmètre.
- **Écartée** : corriger tous les sites du dépôt dans la 15-5b (hors de son module, et de son sujet).
- **Réversible** : oui.

## C15 — 15-5 : troisième sous-story, 15-5c (libellés des refus par lot et manuel du rapprochement)

- **Contexte** : passe de validation P2 de la 15-5b, finding F-3 (MEDIUM, lentille F) — la 15-5b se
  présentait comme un rollout « strictement mécanique » alors qu'elle portait un module neuf de 26
  libellés traduits en quatre locales (#492) et la réécriture d'une section du manuel (#519). Le
  finding F-2 de la même passe a ajouté au manuel quatre passages faux du rapprochement (manuel,
  éclatement, bouton *Modifier*, FAQ) et l'acceptation par lot « atomique », qui recoupent l'issue
  **#481** — dont les trois points (Modifier, lot atomique, manuel et éclatement « par facture ») sont
  tous couverts.
- **Retenu** : une **15-5c** (`15-5c-rapprochement-libelles-et-manuel.md`) reprend le premier volet de
  l'ancien AC14 de la 15-5b (libellés de `failed[]`) et toute la réécriture du manuel du rapprochement ;
  elle **ferme #481, #492 et #519** et dépend de la 15-5b (qui émet `ACCOUNT_NOT_POSTABLE` dans
  `failed[]`). La 15-5b garde les gardes, le second volet de l'AC14 (C14), l'AC13, la levée des réserves
  du manuel (`user-manual.tex:380`, `:390`) et ajoute #521 (C25) ; elle ferme #427, #429, #521.
- **Écartées** : garder tout dans la 15-5b (le mode de revue « fichier par fichier » promis au rollout
  ne tient pas pour un manuel ni pour un module de libellés) ; mettre le manuel dans une story de
  documentation hors epic (le passage *Acceptation par lot* et la section des règles décrivent des
  comportements que la 15-5b change).
- **Réversible** : oui, tant qu'aucune des trois n'est développée.
- **Reste non strictement mécanique dans la 15-5b, et c'est assumé** : AC5 (filtre de `get_proposals`),
  AC8 b (réactivation refusée), AC13 (écran) et AC19 (#521). Ils restent couverts par les passes de
  validation et de revue de code ordinaires, pas par la seule revue fichier par fichier.

## C16 — 15-5a : le détail du refus a la forme de son jumeau `ACCOUNT_ARCHIVED` (révise C13)

- **Contexte** : finding F-1 (MEDIUM) de la P2 de la 15-5a. C13 fixait `details.accountNumbers:
  [String]`, alors que le refus jumeau `ACCOUNT_ARCHIVED` rend `details.rejected[{accountId,
  accountNumber}]` (`crates/kesh-api/src/errors.rs`, bras `ReversalAccountsArchived`) et que les autres
  corps du dépôt nomment un compte par `accountId` + `accountNumber`. Le client envoie des
  identifiants : sans eux, il ne peut pas désigner la ligne fautive.
- **Retenu** : `details.rejected: [{ accountId, accountNumber }]`, **au corps du 400 comme dans
  `failed[].details`** (15-5b). La variante porte les deux, par une structure **jumelle**
  `NonPostableAccount { account_id: i64, account_number: String }` — et non `ArchivedAccount` elle-même,
  dont le numéro est `Option` parce qu'un compte inconnu n'en a pas ; ici le compte est toujours de la
  société et actif, le numéro toujours connu (anti-énumération KF-002 inchangée). Le JSON a exactement
  la forme du jumeau.
- **Écartées** : garder `accountNumbers` (deux formes pour deux refus voisins, et des identifiants
  perdus) ; réutiliser `ArchivedAccount` (un `None` impossible deviendrait représentable).
- **Réversible** : oui avant la release v0.13.0 ; après, la clé fait partie du contrat.

## C17 — 15-5a : la variante ne se construit que triée, dédoublonnée et non vide

- **Contexte** : finding R-3 (MEDIUM) — l'AC1 promettait une variante « construite seulement par le
  constructeur trieur » alors que `DbError::AccountsNotPostable(Vec<String>)` était constructible par
  n'importe qui ; F-6 (LOW) — un itérable vide aurait produit un message sans numéro, accordé au
  singulier en français.
- **Retenu** : la variante porte un newtype `NonPostableAccounts` à **champ privé**, construit seulement
  par `NonPostableAccounts::new(impl IntoIterator<Item = NonPostableAccount>)`, qui trie par numéro et
  dédoublonne par identifiant ; précondition « non vide » vérifiée par `debug_assert!` (un appelant qui
  la viole est un bogue, pas une entrée utilisateur). La garantie tient par le système de types, pas par
  une consigne.
- **Écartée** : retirer l'affirmation et tester le tri à chaque site (la garantie redeviendrait une
  discipline).
- **Réversible** : oui.

## C18 — 15-5a : le pluriel du message passe par un sélecteur Fluent résolu côté serveur

- **Contexte** : finding R-1 (MEDIUM) — le sélecteur `[one]`/`*[other]` de l'AC2 aurait fait rougir le
  garde-fou `SELECTEURS_RESOLUS_COTE_SERVEUR` (`crates/kesh-i18n/src/loader.rs`, test associé), qui
  refuse tout sélecteur non inscrit parce que le dictionnaire servi au frontend le fige sur `*[other]`.
- **Retenu** : garder le sélecteur et **inscrire la clé** `error-account-not-postable` dans
  `SELECTEURS_RESOLUS_COTE_SERVEUR`, en disant où elle est résolue (bras `AccountsNotPostable` de
  `crates/kesh-api/src/errors.rs`, par `t_args` avec `count`). Le frontend n'affiche jamais cette clé
  depuis son dictionnaire : il affiche `err.message`, déjà résolu par le serveur. `loader.rs` entre aux
  fichiers touchés.
- **Écartée** : deux clés plates `-one` / `-other` (patron « et N autres », justifié là parce que le
  frontend résout la clé lui-même — ce n'est pas le cas ici).
- **Réversible** : oui.

## C19 — 15-5a/15-5b : un compte non imputable est « de regroupement, de résultat ou de clôture »

- **Contexte** : finding R-2 (MEDIUM) — la parenthèse « (compte de regroupement ou de clôture) » omet le
  compte de résultat (rôle `CurrentYearResult`, 2979), que `is_postable` exclut et que
  `test_create_manual_rejects_result_account` refuse. Le message existant de `validate_account_of`
  (`company_invoice_settings.rs`) porte le même défaut.
- **Retenu** : « (compte de regroupement, de résultat ou de clôture) » dans les quatre locales et le
  repli Rust de la 15-5a ; la 15-5b corrige de même le message de `validate_account_of`.
- **Écartée** : retirer la parenthèse (elle dit à l'utilisateur pourquoi, ce qui est le but de la story).
- **Réversible** : oui.

## C20 — 15-5a : le paramètre `exempt_ids` est retiré

- **Contexte** : findings R-8 / F-3 (LOW) — `validate_lines_accounts_in_tx(…, exempt_ids)` n'a plus
  qu'un appelant, qui passe `&[]` ; la modification d'une écriture n'existe plus depuis la 24-4b (la
  route rend `409 ENTRY_IS_POSTED`). Son doc-comment décrit encore un `update` inexistant.
- **Retenu** : le retirer dans la 15-5a, qui réécrit de toute façon la requête et le doc-comment ; le
  doc-comment dit pourquoi il a disparu.
- **Écartée** : le garder avec une doc corrigée (code mort qui complique la requête neuve).
- **Réversible** : oui — si une modification d'écriture revenait, l'exemption serait à repenser de
  toute façon.

## C21 — 15-5a : le verrou de `validate_lines_accounts_in_tx` reste hors périmètre

- **Contexte** : finding F-8 (LOW) — la saisie manuelle lit les comptes **sans verrou**, alors que les
  trois gardes de la 24-5 lisent `FOR UPDATE`. La 15-5a réécrit cette requête.
- **Retenu** : ne pas changer le verrouillage dans la 15-5a ; l'écrire comme hors périmètre. Raisons : le
  défaut est antérieur (14-3b) et ne concerne pas le *nom* du refus, objet de la story ; et poser un
  verrou sur les lignes `accounts` du chemin le plus fréquent (toute saisie manuelle, l'ouverture, le
  complément) change l'ordre d'acquisition des verrous face aux flux qui verrouillent déjà ces comptes —
  un risque d'interblocage qu'aucune mesure n'a évalué.
- **Écartée** : `FOR SHARE` / `FOR UPDATE` dans la 15-5a.
- **Réversible** : oui. **Signalé à l'orchestrateur** pour une issue de dette (course : un compte archivé
  ou rendu non imputable entre le contrôle et l'insertion d'une saisie manuelle).

## C22 — 15-5a : la boucle des lignes de facture fournisseur valide la forme d'abord, les comptes ensuite

- **Contexte** : findings R-4 / F-5 (LOW) — collecter les comptes non imputables « après la boucle »
  changeait l'ordre relatif des refus : une ligne 1 au compte non imputable et une ligne 2 de quantité
  nulle rendaient le refus de forme, là où une ligne 1 au compte archivé rendait le refus de compte.
- **Retenu** : deux passes. D'abord la **forme** de toutes les lignes (quantité et prix strictement
  positifs, taux de TVA dans 0–100), dans l'ordre des lignes, refus immédiat — l'ordre des refus de forme
  entre eux est celui d'aujourd'hui ; puis les **comptes**, ligne par ligne : (a) refus immédiat, (b)
  collecté et rendu après la passe. Le seul changement observable — une ligne i en défaut de compte et
  une ligne j > i en défaut de forme rendent désormais le refus de forme — est écrit à l'AC4 et testé.
- **Écartée** : garder une seule boucle et écrire l'ordre hybride (la règle « forme avant comptes » est
  plus simple à dire et à tester).
- **Réversible** : oui.

## C23 — 15-5a/15-5b : chaque commentaire est réécrit par une seule story

- **Contexte** : findings R-9 (15-5a), R-5 et F-10 (15-5b) — le doc-comment de
  `validate_lines_accounts_in_tx` et le commentaire du compte interne de `invoice_settlements_write.rs`
  étaient réécrits par les deux stories, désignés par des numéros de ligne que la 15-5a décale.
- **Retenu** : attribution par **contenu** —
  - 15-5a : le doc-comment de `validate_lines_accounts_in_tx` (cause nommée, `exempt_ids` retiré ; il
    renvoie au doc-comment de `create_in_tx` pour la liste des flux au lieu de la répéter) ; un
    commentaire **neuf** au `match` de chacune des trois gardes de la 24-5 (ordre (a)/(b)) ; la doc de
    module de `routes/opening_balances.rs`.
  - 15-5b : le paragraphe `enforce_postable` du doc-comment de `create_in_tx` (liste des flux) et le
    commentaire du compte interne de `invoice_settlements_write.rs` qui contient « restent ouverts et
    sont suivis par #427 » — que la 15-5a ne touche pas.
  - La 15-5b **refait son T0 des numéros de ligne après le merge de la 15-5a**.
- **Réversible** : oui.

## C24 — 15-5a/15-5b : la documentation des intégrateurs suit le contrat

- **Contexte** : finding F-4 (MEDIUM, 15-5b, transverse) — `docs/api-external.md` documente les codes
  de `failed[]` de `/reconciliation/accept` et tient la table des codes d'erreur (§ 10) ; aucune des deux
  fiches ne la nommait.
- **Retenu** : 15-5a (AC9) ajoute `ACCOUNT_NOT_POSTABLE` à la table du § 10, avec `details.rejected[]` et
  les routes qui le rendent ; 15-5b (AC18) l'ajoute aux codes de `failed[]` de l'acceptation et dit le
  refus des routes de rapprochement manuel et ventilé.
- **Réversible** : oui.

## C25 — 15-5b : le compte créanciers absent du corps est préservé (#521)

- **Contexte** : finding F-1 (MEDIUM) de la P2 de la 15-5b — l'écran *Paramètres → Facturation*
  n'envoie jamais `defaultPayableAccountId`, et le serveur le traite comme `None` : chaque
  enregistrement efface le compte créanciers, et la facture fournisseur échoue ensuite. Issue **#521**,
  créée par l'orchestrateur, confiée à la 15-5b.
- **Retenu** : `default_payable_account_id: Option<Option<i64>>` désérialisé par
  `crate::helpers::double_option` — **absent : préservé, sans contrôle ; `null` : effacé ; valeur :
  validée** (patron #216 et 25-4-c3-a1, `resolve_designated_account`) ; la garde de postabilité « si la
  valeur change » de l'AC10 s'y applique. Le champ **n'est pas exposé à l'écran** dans cette story.
- **Écartées** : exposer le champ à l'écran (élargit la 15-5b d'un formulaire, sans nécessité pour
  fermer le défaut) ; garder `Option<i64>` et faire envoyer le champ par l'écran (un onglet ouvert
  avant la mise à jour, ou tout client qui ignore le champ, l'effacerait encore).
- **Réversible** : oui. Les contournements des E2E (`payment-batches.spec.ts`, `inbox-import.spec.ts`)
  restent valides et ne sont pas retirés.

## C26 — 15-5b : correction de C12 — le formulaire de lien ne protégeait rien

- **Contexte** : finding R-1 (MEDIUM) de la P2 de la 15-5b. C12 disait que
  `BankAccountJournalLinkForm.svelte` « le fait déjà » ; or la page lui passe `accounts={linkableAccounts}`
  (`bank-accounts/+page.svelte`), liste déjà filtrée `active && postable` : `withCurrentAccount` n'y
  retrouve jamais le compte devenu non imputable, et le champ s'affiche vide (défaut #271) sur la
  troisième surface.
- **Retenu** : la 15-5b passe aussi la **liste complète** (`accounts={accounts}`) à
  `BankAccountJournalLinkForm`, qui filtre lui-même ses options ; test Vitest. C12 reste valable pour les
  deux `<select>` de la page.
- **Réversible** : oui.

## C27 — 15-5b : les comptes de réglage sont contrôlés à l'usage, pas seulement à la désignation

- **Contexte** : finding F-1 (MEDIUM) de la P3 de la 15-5b — l'exemption « inchangé » de l'AC11 laisse
  en place un réglage devenu non imputable, et les flux de validation d'une facture et de saisie d'une
  facture fournisseur y écrivent avec `enforce_postable = false` : le défaut de #429 subsistait alors
  que l'inventaire (b) le disait traité. Le compte d'arrondi, lui, est relu à l'usage
  (`rounding_account_for_write`).
- **Retenu** (décision de l'orchestrateur) : la 15-5b ajoute un contrôle **à l'usage** de la créance et
  de la TVA due (validation d'une facture) et des créanciers et de la TVA récupérable (création d'une
  facture fournisseur, donc aussi complétion d'un import), chacun **seulement s'il reçoit une ligne** ;
  refus `ACCOUNT_NOT_POSTABLE` avec `details.rejected` (forme : C28). L'exemption « inchangé » reste à
  la désignation — c'est l'usage qui refuse.
- **Compatibilité avec D-A0, vérifiée** : D-A0 (14-3b) exempte les flux automatiques de la garde de
  `create_in_tx` ; sa limite L2 prévoyait exactement cette remédiation (« re-vérifier `postable` à la
  résolution avec message dédié — amélioration future si un besoin se manifeste »,
  `14-3b-consommateurs-roles.md:188`). La garde est en amont, `create_in_tx` et son drapeau sont
  inchangés ; L2 est **révisée** pour ces quatre comptes. L'angle mort « compte bancaire à l'usage »
  (C6) n'est pas touché : il relève de D-A0 elle-même.
- **Laissé tel quel, et écrit** : le compte de produit par défaut (exemption délibérée D3-bis de la
  16-1a) ; le compte de décompte TVA (lu par aucun flux d'écriture).
- **Écartées** : écrire l'angle mort sans le fermer (#429 resterait ouverte en fait, fermée en titre) ;
  supprimer l'exemption « inchangé » (bloquerait l'enregistrement des réglages, #271).
- **Réversible** : oui (aucune migration) ; le code d'erreur fait partie du contrat dès v0.13.0.

## C28 — 15-5b : le refus à l'usage a son propre message, et le même contrat

- **Contexte** : C27. Le message de la 15-5a (« choisissez un compte imputable ») ne sert à rien à qui
  valide une facture : il n'a pas choisi ce compte sur la pièce ; il faut lui dire **où** agir.
- **Retenu** : une variante `DbError::DesignatedAccountsNotPostable(NonPostableAccounts)`, rendue sous le
  **même code** `ACCOUNT_NOT_POSTABLE` et le **même** `details` (`NonPostableAccounts::details()`, C29) —
  un seul contrat pour l'intégrateur —, avec une clé neuve `error-designated-account-not-postable`
  (sélecteur `[one]`/`*[other]` inscrit à `SELECTEURS_RESOLUS_COTE_SERVEUR`) qui renvoie à *Paramètres →
  Facturation* et demande d'y désigner un sous-compte imputable. Un compte archivé ou absent n'est pas
  refusé par cette garde (chemin actuel, `InactiveOrInvalidAccounts`).
- **Divergence assumée** : le solde du reste refuse une TVA due inutilisable en `ConfigurationRequired`
  (`vat_payable_account_for_write`, 25-4-d2a) ; il n'est pas aligné dans cette story.
- **Écartées** : réutiliser `AccountsNotPostable` (message inexact pour ce cas) ; un champ « contexte »
  dans `NonPostableAccounts` (alourdit le type de la 15-5a pour un seul consommateur) ; un code neuf
  (deux codes pour une même cause).
- **Réversible** : oui avant v0.13.0.

## C29 — 15-5a/15-5b : un seul constructeur du JSON `details.rejected`

- **Contexte** : findings R3-3 et F-4 (LOW) de la P3 de la 15-5b — le JSON `rejected` était construit à
  deux endroits (bras d'`errors.rs` de la 15-5a, `accept_one_split` / `accept_one_rule` de la 15-5b),
  libres de dériver.
- **Retenu** : la 15-5a pose `NonPostableAccounts::details(&self) -> serde_json::Value`
  (`{ "rejected": [{ "accountId", "accountNumber" }] }`, `serde_json` étant déjà une dépendance de
  `kesh-db`) ; le bras API, les deux `failed[].details` et le refus à l'usage (C28) l'appellent.
- **Écartée** : un test d'égalité entre deux constructions (la duplication resterait).
- **Réversible** : oui.

## C30 — 15-5b : la création d'une règle est gardée dans le dépôt, comme sa modification

- **Contexte** : finding F-5 (LOW) de la P3 de la 15-5b — le POST de règle n'était gardé que par le
  pré-vol du handler, hors transaction, alors que le PATCH l'est dans `update_in_tx`.
- **Retenu** : contrôle dans `reconciliation_rules::create_in_tx`, dans la transaction, après la
  validation du projet par défaut et avant l'`INSERT`, refus si `active && !postable` ; le pré-vol
  `validate_counterparty_account` reste inchangé (404). Ordre : forme → 404 compte → refus du projet →
  400 non imputable → 409 doublon.
- **Écartée** : pré-vol seul, course écrite (deux patrons pour une même règle métier).
- **Réversible** : oui.

## C31 — 15-5a : ordre des comptes nommés — lexicographique, puis identifiant

- **Contexte** : finding F-3 (LOW) de la P3 de la 15-5a — « trié par numéro » laissait ouvert l'ordre
  numérique ou lexicographique, et l'ordre à numéro égal.
- **Retenu** : ordre **lexicographique de la chaîne** (`String` de Rust), puis identifiant ; attendu
  fixé dans les tests (`["1000", "10000", "1010", "2000"]`).
- **Écartée** : ordre numérique (un numéro n'est pas garanti numérique ; un tri qui échoue sur un numéro
  alphanumérique est pire qu'un ordre lexicographique).
- **Réversible** : oui.

## C32 — 15-5c : l'écran réel du rapprochement, vérifié au code ; `PERIOD_LOCKED` sans détail

- **Contexte** : findings R-1 et F-1 (MEDIUM) de la P1 de la 15-5c, qui se contredisaient sur le filtre
  des sélecteurs de compte, et F-1 sur les boutons ; F-6 (LOW) sur `PERIOD_LOCKED`.
- **Constaté au code** : les deux modales réduisent la liste aux classes **5, 6, 7** par préfixe
  (`ManualMatchModal.svelte:65-69`, `TransactionSplitModal.svelte:71-73`), puis `AccountAutocomplete`
  ne garde que les comptes **actifs et imputables** (`:200-207`) — R-1 et F avaient chacun raison sur
  un des deux filtres. L'écran des propositions n'a que deux boutons **de lot** (« Accepter (N) »,
  « Rejeter », sur les cases cochées) et deux boutons **par ligne** (« Affecter manuellement »,
  « Éclater ») ; la candidate affichée est une facture **ou** une règle. Une transaction **rejetée**
  quitte la liste, et aucun écran ne la montre plus.
- **Retenu** : l'AC7 écrit les deux filtres ; l'AC5 fusionne *Acceptation des propositions* et
  *Acceptation par lot* en une sous-section qui décrit cet écran ; le manuel dit le sort d'une
  transaction rejetée sans promettre de chemin inexistant. Le libellé de `PERIOD_LOCKED` ne lit pas son
  `details` : limite assumée, écrite.
- **Signalé à l'orchestrateur** : une transaction rejetée n'est plus atteignable depuis l'interface
  (candidat à une issue).
- **Réversible** : oui.

## C-15-5a-1 — 15-5a (dev) : le test du compte de résultat ne passe plus à vide

- **Contexte** : l'AC7 demande de réécrire `test_create_manual_rejects_result_account` vers
  `AccountsNotPostable`. Sur la base de dev seedée (`scripts/seed-dev-db.sql`), la société n'a **aucun**
  compte de rôle `CurrentYearResult` : le test sortait par `return` et rendait vert sans rien exercer —
  constaté par requête sur la base remise à zéro. Le réécrire tel quel aurait certifié une réécriture
  jamais exécutée.
- **Retenu** : faute de compte de résultat, le test en crée un temporaire (`2979`, `Liability`, rôle
  `CurrentYearResult`, `postable = FALSE`) et le supprime en fin de test. La mutation M1 le fait rougir.
- **Écarté** : laisser le `return` (test muet) ; modifier le seed partagé (hors périmètre, effets sur
  d'autres tests).
- **Réversible** : oui (un test).

## C-15-5a-2 — 15-5a (dev) : vocabulaire DE/IT/EN de la parenthèse

- **Contexte** : l'AC2 fixe le texte FR exact et le terme « non imputable » des trois autres locales
  (bebuchbar, registrabile, postable), pas les mots de la parenthèse (regroupement, résultat, clôture),
  qu'aucune clé existante ne traduit.
- **Retenu** : DE « Sammel-, Ergebnis- oder Abschlusskonto », IT « conto di raggruppamento, di risultato
  o di chiusura », EN « grouping, result or closing account », avec accord singulier / pluriel.
- **Réversible** : oui (quatre lignes `.ftl`) ; à relire par un locuteur.

## C-15-5a-3 — 15-5a (dev) : `INACTIVE_OR_INVALID_ACCOUNTS` entre aussi dans la table de `api-external.md`

- **Contexte** : C24 demande une ligne `ACCOUNT_NOT_POSTABLE` au § 10 ; la ligne dit ce qu'il remplace
  et quand l'ancien code reste rendu, or `INACTIVE_OR_INVALID_ACCOUNTS` n'y figurait pas.
- **Retenu** : deux lignes — l'ancien code (inconnu, archivé, autre société, non nommé) et le nouveau.
- **Réversible** : oui.

## C-15-5a-4 — 15-5a (dev) : accesseurs `len()` / `is_empty()` sur `NonPostableAccounts`

- **Contexte** : l'AC1 nomme `iter()` et `numbers()` ; le bras API a besoin du nombre (`count`), et
  clippy (`len_without_is_empty`) exige `is_empty()` dès qu'un `len()` public existe.
- **Retenu** : `len()` et `is_empty()` en lecture seule ; le champ reste privé, l'invariant intact.
- **Réversible** : oui.

## C-15-5a-5 — 15-5a (revue de code) : clôture sur LOW acceptés

- **Contexte** : la passe P1 (Sonnet, trois lentilles) rend 0 CRITICAL, 0 HIGH, 0 MEDIUM et 13 LOW.
- **Retenu** : corriger A-2 (documentation seule) et accepter les douze autres, écrits au Change Log. Le
  plus tentant, B-2 (non-vacuité en `debug_assert!` seulement), toucherait la production : cela
  rouvrirait une passe et le gate pour une liste qu'aucun appelant ne peut produire vide.
- **Écartées** : durcir B-2 en erreur à l'exécution ; ajouter les tests HTTP E-2/E-3/A-3 (routes qui
  propagent sans remappage).
- **Réversible** : oui (les LOW restent tracés au Change Log de la fiche).
## C33 — 15-5b/15-5d : la garde à l'usage des comptes de réglage sort de la 15-5b

- **Contexte** : validation P4 de la 15-5b (lentilles Opus R et F). Finding F4-3 (MEDIUM) : l'AC20,
  ajouté en P3 (C27, C28), est une **règle métier neuve** — révision de la limite L2 de D-A0 —, pas un
  rollout ; et les deux HIGH de la passe (R4-1 = F4-1) en sont **nés**. Sévérité P3 → P4 : MEDIUM →
  HIGH, défauts nés du correctif précédent — le recyclage que l'amendement D5 désigne comme déclencheur.
- **Retenu** (décision de l'orchestrateur) : nouvelle fiche **15-5d-garde-usage-comptes-reglage**, qui
  reprend l'AC20 et tout ce qui s'y rattache (C27, C28, la révision de L2, la variante
  `DesignatedAccountsNotPostable`, la clé `error-designated-account-not-postable`, ses tests, ses
  passages de manuel, sa part de `docs/api-external.md` et du CHANGELOG), plus l'exposition du compte
  créanciers (C34) et l'exemption de l'avoir (C35). Elle **dépend de 15-5a et 15-5b**, est indépendante
  de la 15-5c, et porte `closes #429` ; la 15-5b passe à `refs #429` et garde `closes #427 closes
  #521`. La 15-5b n'est pas renumérotée (19 AC, AC1–AC19).
- **Écartées** : garder l'AC20 dans la 15-5b et l'y faire revoir en passes complètes (la § *Une story de
  rollout* ne le justifiait plus) ; retirer la garde à l'usage de l'epic (#429 resterait ouverte en
  fait).
- **Réversible** : oui (aucun code écrit).

## C34 — 15-5d : le compte créanciers est exposé à l'écran des réglages (révise C25)

- **Contexte** : findings R4-1 = F4-1 (HIGH) de la P4 de la 15-5b. C25 avait écarté l'exposition de
  `defaultPayableAccountId` « sans nécessité pour fermer le défaut » ; la garde à l'usage (C27) a créé
  cette nécessité : son refus renvoie à *Paramètres → Facturation*, où le champ n'existe pas — la
  saisie de toute facture fournisseur serait bloquée sans recours à l'écran.
- **Retenu** (décision de l'orchestrateur) : la 15-5d ajoute au formulaire un `<select>` *Compte
  créanciers (Passif)*, filtré comme la TVA due (`active && postable && Liability`), la valeur courante
  préservée par `withCurrentAccount` (#271), lue, relue sur conflit et envoyée ; types TypeScript
  complétés ; clé `settings-invoicing-payable-account`. L'AC19 de la 15-5b (absent du corps = préservé)
  reste le filet des clients qui n'envoient pas le champ. Les contournements E2E
  (`payment-batches.spec.ts`, `inbox-import.spec.ts`) sont retirés s'ils deviennent inutiles, sur
  constat (specs rejouées sans eux sur base fraîche).
- **Écartées** : sortir les créanciers de la garde à l'usage (angle mort de plus, pour un compte que
  l'utilisateur ne peut pas régler) ; un message distinct pour les créanciers (n'ouvre pas de recours).
- **Réversible** : oui.

## C35 — 15-5d : l'avoir est exempté de la garde à l'usage, avec sa vraie raison

- **Contexte** : findings R4-2 = F4-2 (MEDIUM) de la P4 de la 15-5b. L'avoir ne reprend de la facture que
  ses **comptes de produit** ; la **créance** et la **TVA due** sont relues dans les réglages **du
  moment** (`credit_notes.rs:360-364`, `:507-512`) et postées sans garde. La fiche disait « snapshot de
  la facture » (inventaire (a) #11) et l'AC17 (iv) « reprend les comptes de la facture d'origine » :
  faux.
- **Retenu** (décision de l'orchestrateur) : exemption **délibérée** et écrite. Ces lectures sont
  elles-mêmes le défaut à corriger : la créance sera lue **sur l'écriture de vente** par la 15-6a (#473,
  et #523 pour le compte d'arrondi), la TVA due relève de #525 (report TVA) ; une garde posée sur le
  compte des réglages serait défaite par ces corrections et bloquerait l'annulation d'une facture sur
  un compte que l'avoir ne devrait pas lire. Inventaire (a) #11, AC15 et AC17 de la 15-5b corrigés ; un
  test de la 15-5d fige l'exemption.
- **Écartée** : garder l'avoir à l'usage comme la validation (cohérent avec C27, mais transitoire et
  contraire à « une pièce émise reste annulable »).
- **Réversible** : oui.

## C36 — 15-5d : le message du refus à l'usage — « un compte imputable », « un administrateur doit »

- **Contexte** : findings R4-5 = F4-6 (LOW) de la P4 de la 15-5b. Le texte de C28 disait « désignez-y un
  sous-compte imputable » : le remède « sous-compte » ne vaut que pour un compte de regroupement, et la
  phrase s'adressait à un Comptable qui valide une facture sans accès à la page des réglages (Admin).
- **Retenu** : « … n'est pas imputable (…) : un administrateur doit y désigner à sa place un compte
  imputable » — vrai pour les deux rôles et pour un client d'API, sans branche d'écran par rôle.
- **Écartée** : une branche par rôle dans chaque `catch` (patron de `CONFIGURATION_REQUIRED` à la
  validation d'une facture) — trois écrans à modifier, et un client d'API n'en profiterait pas.
- **Réversible** : oui avant v0.13.0.

## C37 — 15-5c : une clé `error-*` existante est lue directement quand elle convient

- **Contexte** : findings R-7 et F-13 (LOW) de la P2 de la 15-5c — l'AC1 disait que le libellé de
  `ROUNDING_ACCOUNT_NOT_CONFIGURED` « reprend » `error-rounding-account-not-configured`, sans dire s'il
  fallait lire la clé ou en copier le texte ; certains messages serveur portent une variable que le
  client n'a pas.
- **Retenu** : une clé `error-*` existante qui convient **mot pour mot et sans variable** est **lue
  directement** (l'espace `error-` est global pour le lint d'ownership) — c'est le cas de la clé
  d'arrondi ; sinon, clé neuve `reconciliation-failed-*`, sans la variable. Le Dev Agent Record dit, par
  code, quelle clé est lue.
- **Écartée** : dupliquer toutes les traductions sous `reconciliation-failed-*` (quatre locales à tenir
  en double).
- **Réversible** : oui.

## C38 — 15-5c : les échecs partiels restent visibles, et désignent la transaction

- **Contexte** : findings F-5 et F-6 (LOW) de la P2 de la 15-5c — le bloc *Échecs partiels* vit dans la
  branche « liste non vide » et disparaît quand le lot vide la liste (cas de tous les refus de
  *Rejeter*) ; `TX #<id>` désigne un identifiant que l'écran n'affiche nulle part.
- **Retenu** : bloc et compteur sortis de la branche ; la ligne de refus affiche la date, le montant et la
  contrepartie relevés **avant** le rechargement, `TX #<id>` en repli ; un test où le second chargement
  rend une liste vide.
- **Écartée** : écrire la limite au manuel (l'AC6 promet déjà les refus de *Rejeter* en clair : il
  faut qu'ils s'affichent).
- **Réversible** : oui.

## C39 — 15-5d : le prédicat de la garde à l'usage — les rôles rendus par le générateur

- **Contexte** : findings R1-2 (MEDIUM) et F2 (LOW) de la validation P1 de la 15-5d. L'AC1 demandait de
  contrôler les comptes de réglage « effectivement présents dans `entry_lines` », sans mécanisme : la
  TVA totale est une variable locale du générateur (`invoices.rs:1851`, `supplier_invoices.rs:129`), et
  lire les `account_id` des lignes est ambigu dès qu'un même compte joue deux rôles ou coïncide avec un
  compte de produit ou de charge (les fixtures réutilisent `2000`).
- **Retenu** : les générateurs (`generate_invoice_journal_lines`, transmis par `_rounded`, et
  `generate_purchase_journal_lines`) rendent, avec les lignes, l'ensemble des **rôles** de réglage
  qu'ils ont effectivement écrits (créance, TVA due, créanciers, TVA récupérable), le rôle TVA étant
  posé dans la branche même `total_vat > 0` qui écrit sa ligne. La garde ne contrôle que les comptes de
  ces rôles. L'avoir, autre appelant, ignore les rôles (C35).
- **Écartées** : (a) inspecter les `account_id` de `entry_lines` — ambigu ; (b) recalculer la TVA
  totale hors du générateur — duplication (règle DRY) et risque de divergence avec l'arrondi par ligne ;
  (c) rendre seulement un booléen « TVA écrite » — même changement de signature, moins expressif.
- **Réversible** : oui (code non écrit).

## C40 — 15-5d : un même compte désigné pour deux rôles est nommé une fois

- **Contexte** : finding F1 (MEDIUM) de la validation P1 de la 15-5d : sans dédoublonnage, un compte
  qui porte à la fois la créance et la TVA due donnerait « Les comptes 2000, 2000 … ».
- **Retenu** : le dédoublonnage est celui de `NonPostableAccounts::new`, que la 15-5a définit comme
  trieur **et dédoublonneur par identifiant** (fiche 15-5a, AC1) ; l'accesseur construit la variante
  par ce seul constructeur, sans dédoublonnage propre. Un test le fige (« même compte pour deux rôles →
  nommé une fois, singulier »), et le test « créance et TVA due » utilise deux comptes distincts.
- **Écartée** : dédoublonner aussi dans l'accesseur — doublon de la garantie de type de la 15-5a.
- **Réversible** : oui.

## C41 — 15-5d : l'AC4 prouvé par un test de l'écran de validation

- **Contexte** : findings R1-3 (MEDIUM) et F4 (LOW) de la validation P1 de la 15-5d : l'AC4 (« les
  écrans affichent le refus ») n'avait aucun test ; l'écran de validation d'une facture a une branche
  propre à `CONFIGURATION_REQUIRED` qu'un futur ajout d'`ACCOUNT_NOT_POSTABLE` détournerait sans bruit.
- **Retenu** : un test Vitest neuf de l'écran de validation (`invoice-validate-page.test.ts`) — le
  message serveur affiché tel quel, pour un Comptable comme pour un Admin ; les deux `catch`
  fournisseurs, sans branche par code sur ce chemin, sont vérifiés à la lecture et consignés.
- **Écartée** : un test par écran (trois) — les deux `catch` fournisseurs affichent `err.message` sans
  `switch` sur ce code, un test n'y attraperait rien de plus qu'une lecture.
- **Réversible** : oui.

## C42 — 15-5d : révision de C39 — l'avoir n'appelle pas les générateurs de la garde

- **Contexte** : findings R2-2 (MEDIUM) et F2-3 (LOW) de la validation P2 de la 15-5d. C39 écrit
  « L'avoir, autre appelant, ignore les rôles (C35) » ; l'appelant cité (`credit_notes.rs:933`) est un
  test de 16-1a, dans `mod tests` (ouvert à `:743`). L'avoir de production passe par son propre
  générateur, `generate_credit_note_journal_lines` (`credit_notes.rs:187`, appelé à `:507-512`).
- **Retenu** : C39 tient pour son mécanisme (les rôles rendus par les deux générateurs de la garde) ;
  sa phrase sur l'avoir est remplacée par : l'avoir **n'appelle pas** ces générateurs, il a le sien,
  que la story ne touche pas — c'est la forme exacte de son exemption (C35). Ventilation des 25
  occurrences écrite à l'AC1 : 3 définitions, 1 transmission, 2 appels de production, 19 tests.
- **Écartée** : faire rendre des rôles au générateur de l'avoir — hors périmètre (C35 : #473, #523,
  #525).
- **Réversible** : oui (texte de fiche).

## C43 — 15-5d : les comptes de réglage verrouillés avant l'exercice

- **Contexte** : finding F2-1 (HIGH) = R2-1 (MEDIUM) de la validation P2 de la 15-5d. La garde,
  placée entre la génération et `create_in_tx`, verrouillait la créance et la TVA due **après**
  l'exercice ; le solde du reste verrouille la TVA due **avant** lui
  (`invoice_settlements_write.rs:487-491`) → cycle (1213, 500). L'ordre canonique écrit à
  `invoices.rs:1916-1925` met `accounts` (1 bis) avant `fiscal_years` (2) ; les règlements par compte
  interne le suivent aussi. Constat voisin, F2-8 : `supplier_invoices::create_in_tx` verrouille
  l'exercice (`:353`) avant les réglages (`:358-359`), à l'inverse de `validate_invoice`.
- **Retenu** (décision de l'orchestrateur) : l'accesseur en **deux temps** — verrouiller **tous** les
  comptes candidats du flux (vente : créance et TVA due ; achat : créanciers et TVA récupérable),
  une requête `ORDER BY id FOR UPDATE`, **avant** `find_open_covering_date` (après le compte d'arrondi
  côté vente, après les comptes de charge côté achat), sans refuser ; **contrôler** après la
  génération les seuls rôles écrits (C39 inchangé), sur l'instantané verrouillé. Côté achat, les
  réglages sont chargés **avant la boucle des comptes de charge** : ordre réglages → comptes →
  exercice, celui de la validation ; cela ferme l'inversion F2-8 (et sa variante réglages / comptes) —
  aucune KF à ouvrir. Les refus gardent leur priorité (l'exercice précède le refus de la garde) ; un
  test de non-interblocage à deux connexions (`attendre_une_requete_en_cours`) et un test d'ordre
  « exercice absent + compte non imputable » le figent.
- **Écartées** : (a) lecture non verrouillante (patron `validate_line_revenue_accounts_in_tx`) —
  rouvre la course que le verrou ferme, sur des comptes que le solde du reste verrouille déjà ;
  (b) générer les lignes et garder avant l'exercice — déplace des refus (la garde passerait avant
  `FiscalYearInvalid`, et l'ordre des refus que les AC figent changerait) ; (c) laisser l'inversion F2-8 à une KF — la correction la ferme sans coût propre.
- **Réversible** : oui (code non écrit).

## C44 — 15-5d : `DesignatedRole`, type neuf plutôt qu'`AccountRole`

- **Contexte** : finding F2-4 (LOW) de la validation P2 de la 15-5d — `AccountRole`
  (`crates/kesh-db/src/entities/account.rs:90-102`) porte déjà `Receivable`, `Payable`,
  `VatRecoverable`, `VatPayable` ; la règle DRY veut un type neuf justifié.
- **Retenu** : `DesignatedRole` à quatre variantes. Il désigne un **champ des réglages** effectivement
  écrit, non le rôle que le plan donne au compte (un compte désigné ne porte pas forcément ce rôle) ;
  ses quatre variantes se traduisent en champs par un `match` exhaustif, sans bras mort pour les neuf
  autres rôles ; `AccountRole` ne dérive pas `Ord`, qu'exige le `BTreeSet`.
- **Écartée** : réemployer `AccountRole` — bras `_ =>` ou `unreachable!()` pour neuf variantes
  (proscrit par le garde-fou défensif du `CLAUDE.md`), et confusion de deux notions.
- **Réversible** : oui.

## C45 — 15-5d : les contournements E2E du compte créanciers restent (révise C34)

- **Contexte** : findings R2-3 (MEDIUM), F2-2 (MEDIUM) et R2-4 (LOW) de la validation P2 de la 15-5d.
  C34 prévoyait leur retrait « sur constat (specs rejouées sans eux sur base fraîche) », sur la
  prémisse que le seed E2E désigne le compte créanciers. Il ne le fait pas : les presets
  `with-company` / `post-onboarding` appellent `seed_accounting_company`
  (`test_endpoints.rs:184`), dont l'`INSERT` (`test_fixtures.rs:156-170`) ne pose pas
  `default_payable_account_id`. Et les contournements sont **trois**, non deux
  (`supplier-invoices.spec.ts:86` en plus).
- **Retenu** : les trois restent ; leur commentaire est réécrit (« le seed `with-company` ne désigne
  pas le compte créanciers ») ; inventaire par la commande
  `grep -rn "defaultPayableAccountId == null" frontend/tests/e2e`. La suite E2E complète reste le
  juge de non-régression.
- **Écartée** : étendre `seed_accounting_company` au compte créanciers — rayon d'impact sur tous les
  tests `kesh-db` / `kesh-api` qui l'emploient, sans gain pour la garde.
- **Réversible** : oui.

## C46 — 15-5d : le dialogue de validation se ferme sur `ACCOUNT_NOT_POSTABLE`

- **Contexte** : finding F2-6 (LOW) de la validation P2 de la 15-5d — l'écran de validation ferme le
  dialogue pour les « erreurs non-retryables » (`invoices/[id]/+page.svelte:428-434` :
  `FISCAL_YEAR_INVALID`, `CONFIGURATION_REQUIRED`) ; `ACCOUNT_NOT_POSTABLE` n'y est pas.
- **Retenu** : le code est ajouté à la liste. Qu'il vienne d'un compte de réglage (15-5d) ou d'un
  compte de produit d'une ligne (15-5a), réessayer depuis le dialogue rend le même refus ; le message
  reste affiché par `notifyError`. Le test d'écran de l'AC7 l'assère.
- **Écartée** : le laisser ouvert — un bouton « Valider » qui ne peut que rééchouer.
- **Réversible** : oui.

## C47 — 15-5d : le signal de découpage D5 (MEDIUM → HIGH, recyclé) ne la découpe pas

- **Contexte** : à la passe P2 de la 15-5d, la sévérité est remontée (HIGH, ordre des verrous), et le défaut venait de
  la place de la garde fixée par la remédiation P4 de la 15-5b — c'est le « recyclage » que vise l'amendement D5 du
  CLAUDE.md, qui appelle normalement un découpage.
- **Retenu (orchestrateur)** : pas de découpage. La 15-5d porte une seule règle métier (la garde à l'usage) et l'écran
  qui la rend praticable ; la découper séparerait la garde de son ordre de verrouillage, qui est précisément l'objet du
  défaut. Le défaut a été corrigé à la racine (verrou avant l'exercice, conforme à l'ordre canonique écrit à
  `invoices.rs:1916-1925`) et une passe complète P3 suit.
- **Écartée** : découper en « garde » / « écran » — l'écran n'est pour rien dans le défaut.
- **Réversible** : oui ; si la P3 recycle encore, découpage.

## C48 — 15-5d : « l'arrondi d'abord » — le règlement client et le solde du reste réordonnés (révise C43)

- **Contexte** : findings R3-1 (MEDIUM) = F3-1 (MEDIUM) de la validation P3 de la 15-5d. C43 affirmait
  que « les règlements par compte interne » suivent l'ordre canonique. Le règlement client ne le suit
  pas pour la paire de comptes : il verrouille le compte interne (`invoice_settlements_write.rs:157`),
  puis le compte d'arrondi en cas d'écart (`:181-191`), puis l'exercice (`:200`) ; la validation, après
  la story, prend l'arrondi (`invoices.rs:2065`) puis la créance et la TVA due. Le compte interne
  pouvant être n'importe quel compte actif et imputable — créance ou TVA due comprises —, la garde
  formait un cycle neuf (1213, 500).
- **Retenu** (décision de l'orchestrateur) : le règlement client par compte interne calcule le
  classement du paiement (`amount_due`, `classify_payment`, lectures pures) avant l'étape (3), prend
  le compte d'arrondi (cas `SettlesWithRounding`) **avant** le compte interne, puis l'exercice ; le
  virement est inchangé (il ne verrouille que `bank_accounts`). Seul changement de priorité des
  refus : le compte d'arrondi inutilisable passe avant le compte interne invalide ; le trop-perçu
  reste rendu après le compte interne. **Ajout de la remédiation** : ce réordonnancement crée à son
  tour un cycle avec le **solde du reste** (nature `:433` puis arrondi `:475`) quand le compte interne
  choisi est le compte de la nature ; le solde prend donc lui aussi l'arrondi **avant** la nature
  (ses deux conditions — nature `Rounding`, reste hors centime — sont connues dès l'étape (2)). Règle
  écrite : parmi les comptes, l'arrondi d'abord. Un test de non-interblocage règlement ↔ validation
  (compte interne = TVA due), avec une sonde `FOR UPDATE NOWAIT` qui rend le test indépendant de
  l'ordre d'attribution des verrous ; tests d'ordre des refus des deux flux ; mutation.
- **Restent, écrits et non déclarés absents** : le lot de rapprochement (exercice puis arrondi, #536,
  finding F3-3) ; l'avoir d'une facture arrondie (exercice `credit_notes.rs:370`, puis arrondi `:525`
  — même inversion, préexistante, relevée par le grep de la remédiation, **à tracer**) ; un compte de
  nature désigné sur la créance ou la TVA due (angle mort assumé).
- **Écartées** : (a) déclarer le cycle comme angle mort assumé (configuration atypique) — l'API la
  permet, la conséquence est un 500, et le correctif ne coûte qu'un déplacement ; (b) réordonner le
  seul règlement — ouvre le cycle symétrique avec le solde du reste ; (c) faire verrouiller par le
  solde la nature et la TVA due dans une seule requête `ORDER BY id` — ne règle rien pour l'arrondi
  et élargit la story.
- **Réversible** : oui (code non écrit).

## C49 — 15-5d : l'accesseur refuse lui-même un compte désigné absent ou inactif

- **Contexte** : finding F3-4 (LOW) de la validation P3 de la 15-5d. L'accesseur laissait un compte
  archivé au contrôle de `create_in_tx` (`journal_entries.rs:96-99`), lecture non verrouillante qui,
  sous REPEATABLE READ, lit l'instantané ouvert avant le verrou : un compte archivé entre les deux y
  paraît actif, et l'écriture passe. L'argument (a) de C43 (« rouvre la course que le verrou ferme »)
  n'était donc vrai qu'à moitié.
- **Retenu** : au contrôle, sur la ligne fraîche tenue sous verrou, l'accesseur rend d'abord
  `InactiveOrInvalidAccounts` pour un compte d'un rôle écrit absent de l'instantané ou inactif, puis
  `DesignatedAccountsNotPostable` pour les comptes actifs non imputables. Même variante et même code
  qu'aujourd'hui pour le premier cas ; le contrat de la 15-5a (ne nommer qu'un compte actif de la
  société) est tenu.
- **Écartée** : écrire la course comme limite (fenêtre de quelques millisecondes) — le verrou tient
  déjà la ligne, la refuser ne coûte qu'une condition.
- **Limite écrite** : le test du compte archivé prouve le résultat, non qui le produit ; le refus de
  l'accesseur est vérifié à la lecture.
- **Réversible** : oui.

## C50 — 15-5d : la signature des générateurs — `GeneratedLines { lines, roles }`

- **Contexte** : finding F3-6 (LOW) de la validation P3 de la 15-5d — « les rôles rendus avec les
  lignes » laissait au dev le choix du type, dont dépendent 19 tests.
- **Retenu** : `struct GeneratedLines { pub lines: Vec<NewJournalEntryLine>, pub roles: BTreeSet<DesignatedRole> }`,
  visible dans `crate::repositories` ; vente : `Result<GeneratedLines, DbError>` ; achat :
  `Result<(GeneratedLines, Decimal), DbError>`. Les 19 tests lisent `.lines`, sans autre changement.
- **Écartées** : un triplet (lisibilité) ; une fonction compagnon qui recalculerait les rôles (DRY,
  C39).
- **Réversible** : oui.

## C51 — 15-5d : un identifiant d'une autre société dans le verrou des comptes désignés

- **Contexte** : finding F3-5 (LOW) de la validation P3 de la 15-5d. Le dépôt a mesuré qu'un
  `FOR UPDATE` par clé primaire verrouille la ligne d'une autre société avant que le filtre
  `company_id` ne l'écarte (`opening_complement.rs:431-440`), et qu'un plan par `filesort` verrouille
  tout ce qu'il parcourt (`:274-288`).
- **Retenu** : un test pose un compte d'une autre société dans les réglages (par SQL) et vérifie, par
  une seconde connexion `FOR UPDATE NOWAIT`, que l'accesseur ne le verrouille pas ; si le test montre
  le contraire, l'accesseur prend le patron `owned_account_ids` (identifiants de la société lus sans
  verrou, puis `FOR UPDATE` sur eux seuls). L'`EXPLAIN` de la requête est relevé au Dev Agent Record.
- **Écartée** : filtrer d'office — les identifiants viennent des réglages, gardés à la désignation
  par la 15-5b ; le filtre ne s'impose que si la mesure le demande.
- **Réversible** : oui.

## C52 — 15-5d : la clause de C47 joue, l'ordre des verrous sort en 15-5e

- **Contexte** : C47 avait écarté le découpage de la 15-5d « sauf si la P3 recycle encore ». La P3 a recyclé en partie : F3-2 (aucun test du changement d'ordre côté achat) naît de la remédiation P2, et la remédiation P3 a dû étendre le réordonnancement à deux flux de production de plus (règlement client par compte interne, solde du reste — C48).
- **Retenu (orchestrateur)** : la 15-5d est découpée. **15-5e-ordre-des-verrous-reglements** porte le réalignement des flux de règlement sur l'ordre canonique (arrondi → comptes → exercice) — règlement client par compte interne, solde du reste, règlement et saisie fournisseur — avec ses tests de non-interblocage ; elle passe **avant** la 15-5d, qui garde la garde à l'usage, son accesseur et l'écran du compte créanciers, et suppose l'ordre déjà en place.
- **Écartée** : poursuivre sans découper — C47 l'avait promis, et la story n'a cessé de grossir à chaque remédiation.
- **Réversible** : oui (refusionner les deux fiches).

## C53 — 15-5e / 15-5d : la ligne de partage du découpage C52

- **Contexte** : C52 sort l'ordre des verrous de la 15-5d. Trois éléments n'ont pas de place évidente :
  les tests de non-interblocage (deux d'entre eux n'existent que par le verrou de l'accesseur, que la
  15-5d crée), le commentaire « 5 bis » du solde du reste (sa réécriture affirmait que la validation
  verrouille la TVA due, ce qui n'est vrai qu'après la 15-5d) et la phrase du doc-comment canonique de
  `validate_invoice` (`invoices.rs:1920-1922`, « Aucun chemin ne verrouille `accounts` avant `invoices`
  ou `fiscal_years` »), fausse aujourd'hui : le règlement client et le solde du reste prennent leurs
  comptes **avant** l'exercice, comme l'ordre canonique le prescrit.
- **Retenu** :
  - **15-5e** réordonne les flux **existants** — règlement client par compte interne (arrondi avant
    compte interne), solde du reste (arrondi avant nature), saisie fournisseur (réglages après projet
    et fournisseur, avant les comptes de charge) — et porte les tests qui figent **ces** ordres, chacun
    par une sonde `FOR UPDATE NOWAIT` décisive : l'ancien test 3 de la 15-5d (règlement client ↔
    validation), et **deux sondes neuves**, solde du reste (arrondi tenu, compte de la nature libre) et
    saisie fournisseur (réglages tenus, compte de charge et exercice libres). Elle réécrit la phrase
    fausse du doc-comment canonique et le commentaire « 5 bis » **sans** y prêter à la validation un
    verrou qu'elle n'a pas encore. Elle porte une ligne de CHANGELOG (rubrique « Modifié ») pour le seul
    changement visible : la priorité des refus du règlement par compte interne et du solde du reste.
  - **15-5d** garde les tests de non-interblocage 1 (solde du reste ↔ validation) et 2 (règlement
    fournisseur ↔ saisie fournisseur), qui figent la **place de son accesseur** par rapport à
    l'exercice et rougissent sous sa mutation, et gagne un **test 3 neuf** — une sonde : la
    validation tient l'arrondi sans tenir encore la créance ni la TVA due —, seul à rougir si
    l'accesseur passe **avant** l'arrondi (la sonde du test de la 15-5e porte sur le règlement et ne
    voit pas cette mutation) ; elle complète le doc-comment canonique (créance et TVA due à l'étape 1 bis) et le
    commentaire « 5 bis » (la validation verrouille désormais la TVA due avant l'exercice), et compte le
    test de la 15-5e comme non-régression.
  - Fait relevé au découpage, **raisonné et non mesuré** : deux cycles existent **dès aujourd'hui**,
    sans la garde — règlement client avec écart sur un compte interne = TVA due (TVA due, puis arrondi)
    contre solde du reste à escompte d'un reste hors centime (arrondi, puis TVA due) ; validation
    (réglages `invoices.rs:2006`, puis exercice `:2172`) contre saisie fournisseur (exercice
    `supplier_invoices.rs:353`, puis réglages `:358-359`, finding F2-8 de la P2 de la 15-5d). La 15-5e
    les ferme ; elle n'est donc pas un simple préalable de la 15-5d.
- **Écartées** : (a) les trois tests de non-interblocage dans la 15-5e — les tests 1 et 2 y seraient
  verts par construction (sans l'accesseur, aucune tâche n'attend sur les comptes désignés) : des tests
  qui ne prouvent rien ; (b) les sondes neuves omises : côté achat, le déplacement des réglages ne change
  aucun refus, rien d'autre ne le figerait ; côté solde, l'ordre des refus ne fige l'ordre des verrous
  que tant que verrou et refus vivent dans le même appel (`write_off_account_for_write`) — un accesseur
  en deux temps, comme celui de la 15-5d, les découplerait sans que le test d'ordre des refus rougisse ;
  (c) faire écrire à la 15-5e le commentaire final, validation comprise — il serait faux entre les deux merges.
- **Réversible** : oui (fiches seulement, code non écrit).

## C54 — 15-5e : la défense contre l'interblocage est le rejeu, pas un ordre parfait des verrous

- **Contexte** : la validation P1 de la 15-5e (lentille F, F1-1 HIGH) établit que **tout** flux d'écriture reprend un verrou partagé sur `accounts` APRÈS l'exercice, par la clé étrangère `fk_jel_account` à l'insertion des lignes. Aucun ordre « comptes avant exercice » ne peut donc être tenu de bout en bout ; chaque passe de validation des 15-5d, 15-5e, 15-6a et 15-6b a trouvé un cycle de plus, et la recherche d'un ordre global sans cycle ne converge pas. Deux issues ouvertes décrivent précisément l'absence de rejeu : #463 (annulation d'un règlement client), #491 (règlement manuel contre rapprochement).
- **Retenu (orchestrateur)** : la 15-5e change d'objet. Elle devient **le rejeu sur interblocage (erreur 1213) de tous les flux d'écriture qui ne l'ont pas** — règlement client et fournisseur, annulation de règlement, avoir, validation de facture, saisie fournisseur, lots, rapprochement manuel et ventilé —, sur le patron existant `kesh_db::retry::retry_with` (déjà en place sur le solde du reste, l'onboarding, les soldes de départ et l'acceptation du rapprochement) ; un inventaire fermé des routes d'écriture, chacune rejouée ou exemptée avec raison ; un test qui prouve le prédicat sur une vraie 1213. Elle ferme **#463** et **#491**, et **#536** si le rejeu la couvre. Les réordonnancements simples déjà spécifiés qui ne coûtent rien sont gardés ; les affirmations d'absence de cycle sont retirées partout, et le doc-comment canonique dit la règle vraie : l'ordre réduit la fréquence des interblocages, le rejeu les rend invisibles à l'utilisateur.
- **Conséquence pour les 15-5d, 15-6a, 15-6b** : leurs verrous restent (ils ferment des courses de lecture, ce qui est leur vrai rôle) ; elles cessent d'affirmer l'absence de cycle et renvoient au rejeu de la 15-5e.
- **Écartée** : continuer à chercher un ordre global sans cycle — impossible tant que la clé étrangère reprend le compte après l'exercice, et chaque passe en trouve un nouveau.
- **Réversible** : oui.

## C55 — 15-5e : ce qui reste de l'ordre des verrous après C54

- **Contexte** : C54 garde « les réordonnancements simples déjà spécifiés qui ne coûtent rien ». La fiche validée en P1 en portait trois — règlement client par compte interne : l'arrondi avant le compte interne (ancien AC2) ; solde du reste : l'arrondi avant la nature (ancien AC3) ; saisie fournisseur : les réglages avant les comptes de charge (ancien AC4) — et la P1 demandait de trier les comptes de charge par identifiant (R1-3).
- **Retenu** : garder **la seule avance des réglages de la saisie fournisseur** (un appel déplacé, aucun refus ne bouge ; ordre « réglages → exercice » commun avec la validation ; la ligne des réglages sérialise les saisies, F1-6 ; la 15-5d en a besoin pour verrouiller ses candidats avant l'exercice), placée entre la passe de forme et la passe des comptes de la 15-5a (F1-7). **Retirer** les anciens AC2 et AC3 : ils ne ferment aucune course de lecture, déplacent deux priorités de refus visibles de l'intégrateur, avancent le classement du paiement et coûtaient trois tests à sonde dont l'un ne prouvait rien (F1-4) ; le cycle qu'ils visaient sur la créance n'était d'ailleurs pas fermé (F1-1). **Ne pas trier** les comptes de charge : le seul cycle que le tri fermerait (deux saisies aux comptes croisés) est déjà sérialisé par la ligne des réglages, et celui qui reste (compte de charge = compte d'arrondi, contre un règlement par compte interne) ne dépend pas de cet ordre. Les commentaires faux (doc-comment canonique, « 5 bis », `write_off_invoice_handler`, module `retry.rs`, Pattern 5) sont réécrits pour dire la règle vraie, sans changer de code.
- **Écartées** : tout garder (coût et refus déplacés pour un gain de fréquence que le rejeu rend invisible) ; tout retirer, avance des réglages comprise (casse la 15-5d) ; trier les comptes de charge (code sans effet sur un cycle réel).
- **Réversible** : oui — les réordonnancements retirés restent décrits au Change Log de la 15-5e.

## C56 — 15-5e : la forme du rejeu — deux enveloppes partagées, un registre des routes

- **Contexte** : quatre routes rejouent déjà leurs écritures au journal, chacune en recopiant son `retry_with` ; dix-sept sont à rejouer ; la story exige un inventaire fermé, et touche plus de cinq modules (signal de la règle de découpage).
- **Retenu** : `kesh_db::retry::retry_on_deadlock` (existante, inutilisée par les routes) pour les routes dont l'écriture est une fonction de dépôt qui possède sa transaction ; une enveloppe `AppError` neuve dans `kesh-api` pour les trois routes dont la transaction est ouverte dans le handler (extraites en fonctions « une tentative », patron `accept_once`) ; migration des trois sites existants à prédicat simple ; `post_accept` garde son prédicat élargi (1305) ; `onboarding::finalize`, qui n'écrit pas au journal, n'est pas touché. L'inventaire est gardé par un **registre testé** de toutes les routes mutantes (patron `audit_route_registry.rs`), qui vérifie aussi la présence de l'enveloppe dans le corps de chaque handler « rejoué ». Deux routes exemptées, raison écrite : l'import d'instance (`/admin/full-import`) et l'effacement de la démo (`/onboarding/reset`). **Dérogation de découpage** écrite à la fiche : le patron existe, la story en est le rollout mécanique, une découpe laisserait des routes non rejouées entre deux merges ; signal déclaré au Project Lead.
- **Écartées** : rejouer dans les dépôts (les fonctions `_in_tx` ne possèdent pas la transaction) ; un rejeu générique par middleware Axum (le corps de requête est consommé, et le handler peut faire des relectures hors transaction qu'on ne veut pas rejouer) ; découper en story-zéro + rollout (le patron existe déjà).
- **Réversible** : oui.

## C57 — 15-5e : #536 fermée par le rejeu

- **Contexte** : #536 (lot de rapprochement ↔ validation, exercice et arrondi pris en sens inverses) attend « l'ordre canonique, ou le rejeu sur interblocage ; un test à deux connexions ». L'acceptation rejoue déjà (#480) ; la validation, victime, rend 500. L'issue cite aussi l'avoir d'une facture arrondie (même inversion), qu'elle confie à la 15-6a.
- **Retenu** : `closes #536` — la validation est rejouée, avec un test où elle est la victime d'une vraie 1213 ; les deux côtés deviennent invisibles à l'utilisateur. La création d'avoir est rejouée au même titre : la 15-6a n'a plus besoin de réordonner l'avoir pour #536 (signalé à l'orchestrateur ; fiche 15-6a non touchée ici).
- **Écartée** : `refs #536` — le rejeu couvre tout ce que l'issue attend.
- **Réversible** : oui.

## C58 — 15-5e : le registre des routes rejouées est une seconde colonne du registre d'audit

- **Contexte** : la validation P2 de la 15-5e (finding F2-3 MEDIUM) relève que la fiche prescrivait un registre neuf (`rejeu_route_registry.rs`) « sur le patron » d'`audit_route_registry.rs` : sans autre consigne, l'extracteur de routes (`extract_counted`), les deux volets `lib.rs` / `test_endpoints.rs` et la garde du troisième fichier auraient été recopiés, et toute route ajoutée aurait dû l'être dans deux registres (règle DRY). Arbitrage de l'orchestrateur : étendre le registre existant ou partager son extracteur — trancher.
- **Retenu** : **une seconde colonne de statut** (`Rejouee` / `SansEcritureAuJournal` / `Exemptee`) dans `LIB_ROUTES` et `TEST_ENDPOINT_ROUTES` d'`audit_route_registry.rs`, plus deux tests dans le même fichier (présence d'une enveloppe dans le corps des handlers `Rejouee` ; partition recomptée : 21 / 4 / reste). Une route ajoutée s'examine une fois pour les deux propriétés ; les volets « absente du registre », « absente du code » et la garde du troisième fichier servent tels quels. Le doc-comment du module dit ce que la colonne n'établit pas (classification, atteinte du prédicat, angle mort de la victime sans écriture au journal).
- **Écartée** : un module partagé sous `tests/common/` utilisé par deux registres — supprime la copie de l'extracteur, mais garde **deux listes** de routes à tenir, donc deux examens par route ajoutée et deux listes qui peuvent dériver.
- **Réversible** : oui (fiche seulement ; au dev, scinder une colonne en fichier reste mécanique).

## C59 — 15-5e : les rejeux sont journalisés au niveau `warn`, avec le nom de l'opération

- **Contexte** : finding F2-8 (LOW) — `retry_with` journalise chaque rejeu en `debug` ; avec 21 routes rejouées, l'exploitant ne verrait plus jamais un interblocage, et la thèse de C54 (l'ordre des verrous réduit la fréquence) ne se mesurerait plus. Le serveur n'a pas de span de requête (aucun `TraceLayer`), donc un `warn` nu ne dirait pas quelle route.
- **Retenu** : le `tracing::debug!` de `retry_with` passe à `warn!` ; chaque enveloppe prend un nom d'opération (`&'static str`) et exécute le rejeu dans un span qui le porte ; `post_accept` et `onboarding::finalize`, qui appellent `retry_with` directement, s'instrumentent de même.
- **Écartées** : `info` (filtré par une configuration de production à `warn`) ; changer la signature de `retry_with` pour y passer le nom (touche tous les sites pour un gain nul face au span) ; un compteur de métriques (aucune infrastructure de métriques dans Kesh).
- **Réversible** : oui (niveau de journal).

## C60 — 15-5e : #484 (manuels « SERIALIZABLE ») fermée par la 15-5e

- **Contexte** : finding F2-4 (MEDIUM) — `user-manual.tex:905` et `admin-manual.tex:885-886` affirment des transactions `SERIALIZABLE` qui n'existent pas (isolation `REPEATABLE READ`, verrous de ligne, contrainte d'unicité) ; c'est l'issue ouverte #484. Le relevé des manuels de l'AC6, une liste de mots sans `SERIALIZABLE`, ne pouvait pas le voir. Décision de l'orchestrateur : la 15-5e la ferme.
- **Retenu** : `closes #484` ; les deux passages réécrits sur le mécanisme réel (compteur verrouillé, unicité, rejeu sur interblocage ; l'admin : `REPEATABLE READ` gardé, verrous de ligne et nommés, rejeu) ; le manuel admin gagne la consigne de laisser `innodb_deadlock_detect` à `ON` (finding F2-7) ; PDF régénérés et contrôlés aplatis ; relevé élargi (`SERIALIZABLE|isolation|concurren|simultan|en même temps|…`) trié occurrence par occurrence.
- **Écartée** : laisser #484 à une story ultérieure — la 15-5e écrit précisément la règle que ces passages devraient décrire, et le CHANGELOG qu'elle ajoute contredirait sinon les manuels.
- **Réversible** : oui.

## C61 — 15-5e : découpage en 15-5e1 (socle du rejeu) et 15-5e2 (rollout)

- **Contexte** : la validation P3 de la 15-5e (finding F3-2 MEDIUM) établit que la dérogation de découpage de C56 ne repose pas sur l'exception que la § *Règle de splitting préventif* prévoit (cycles Cargo, merges intermédiaires impossibles à tester) : « le patron existe » est contredit par l'AC2 (deux enveloppes, un changement de signature, la journalisation), et « des routes resteraient non rejouées entre deux merges » décrit l'état actuel, pas une impossibilité de tester. Décision de l'orchestrateur : découper selon le patron « story-zéro + rollout ».
- **Retenu** : **15-5e1-socle-rejeu** — les deux enveloppes nommées, le paramètre `operation` et le `warn!`, le registre (seconde colonne d'`audit_route_registry.rs`), l'inventaire, les trois routes des issues (règlement client #491, annulation de règlement #463, validation #536) et leurs tests « route victime » ; `closes #463 #491`, `refs #536`. S'y ajoute ce que la 15-5d attend de l'ordre des verrous (C65) : doc-comment canonique de `validate_invoice`, avance des réglages de la saisie fournisseur, « 5 bis », module `retry.rs`, et la ligne du CHANGELOG (C64). **15-5e2-rejeu-des-autres-flux** — les 18 autres routes `Rejouee` (14 à rejouer, 3 migrées, `post_accept` inchangée), les commentaires d'ordre restants, l'inventaire au symptôme, le Pattern 5, les manuels (#484), le CHANGELOG étendu, `api-external` ; `closes #536 #484`. La 15-5e devient la fiche index (`split`). **Révise C56** sur deux points : la dérogation de découpage (écartée) et « deux routes exemptées » (quatre depuis C58 / la P2).
- **Écartées** : (a) réécrire la dérogation sur le critère effectif de la règle — aucun cycle Cargo ni merge non testable ne la fonde ; (b) un troisième morceau pour la documentation (#484) — elle décrit le mécanisme complet, qui n'existe qu'après le rollout ; (c) laisser à la 15-5e2 le doc-comment canonique et l'avance des réglages — la 15-5d en dépendrait alors.
- **Réversible** : oui (fiches seulement, code non écrit).

## C62 — 15-5e1 : noms des enveloppes ; le nom d'opération est un champ de l'événement `warn`

- **Contexte** : findings F3-1 et R3-3 (le volet (c) du registre cherche les enveloppes par leur nom, laissé « au choix du dev » pour l'enveloppe `AppError`, sans visibilité fixée) et F3-3 (C59 portait le nom d'opération par un span `info`, désactivé sous `RUST_LOG=warn` — la configuration même que C59 invoquait pour écarter `info` ; l'événement `warn` perdait alors son nom).
- **Retenu** : `kesh_db::retry::retry_on_deadlock(operation, f)` et `retry_on_deadlock_with(operation, max_attempts, f)` ; module `crates/kesh-api/src/retry.rs` déclaré `pub mod retry;`, fonction `kesh_api::retry::retry_app_on_deadlock(operation, f)`, prédicat exposé `kesh_api::retry::is_app_deadlock(&AppError) -> bool`. `retry_with` gagne un premier paramètre `operation: &'static str`, porté par le `tracing::warn!` lui-même (champ `operation`). Conséquence assumée : les cinq sites directs de `retry_with` et les six appels de tests de `retry.rs` gagnent un nom dans la 15-5e1. **Révise C59** (le span est abandonné).
- **Écartées** : `warn_span!` (un span reste un mécanisme de plus pour une seule donnée, et dépend du niveau du span et non de l'événement) ; une variante `retry_with_named` à côté de `retry_with` (deux fonctions pour un même rôle, contraire au DRY).
- **Réversible** : oui.

## C63 — 15-5e1 / 15-5e2 : statut transitoire `ARejouer`, volet (c) robuste, `retry_with` restreint

- **Contexte** : le registre est posé par la 15-5e1 alors que quatorze routes qui écrivent au journal ne seront rejouées que par la 15-5e2 ; et le volet (c), textuel, laissait passer un handler sans enveloppe suivi du doc-comment d'une route qui la nomme (F3-1).
- **Retenu** : un statut **`ARejouer("15-5e2")`** dans la 15-5e1 (partition 7 `Rejouee` / 14 `ARejouer` / 4 `Exemptee` / 90 `SansEcritureAuJournal` = 115), **retiré** par la 15-5e2 (21 / 4 / 90). Volet (c) : commentaires retirés avant le match, fenêtre coupée au premier attribut / doc-comment / item qui suit le corps, recherche de `nom(` ou `nom::<` ; ces précautions testées sur un source synthétique ; mutations sur plusieurs familles. `retry_with` accepté pour toute route `Rejouee` dans la 15-5e1 (quatre sites l'appellent), restreint à `post_accept` dans la 15-5e2.
- **Écartées** : classer les quatorze routes `Rejouee` dès la 15-5e1 (le volet (c) rougirait) ou `SansEcritureAuJournal` (faux, et la 15-5e2 devrait reclasser sans garde) ; un analyseur syntaxique (`syn`) dans le test — dépendance de développement neuve pour un gain que les trois précautions et leur test donnent déjà.
- **Réversible** : oui.

## C64 — 15-5e1 / 15-5e2 : la ligne du CHANGELOG voyage avec les issues qu'elle ferme ; PDF de la brochure

- **Contexte** : le découpage demandé plaçait le CHANGELOG dans la 15-5e2, alors que la 15-5e1 ferme #463 et #491. La règle d'inclusion du `CLAUDE.md` veut la documentation dans la PR qui motive le changement, et le précédent de l'Epic 24 (neuf livraisons absentes du CHANGELOG) montre ce que coûte un correctif publié sans sa ligne. Finding F3-9 : `make fr` régénère aussi la brochure, dont la source ne change pas.
- **Retenu** : la 15-5e1 écrit la ligne « Corrigé » de ses trois routes (avec la réserve des trois tentatives, F3-9) ; la 15-5e2 l'étend aux autres opérations et ajoute la ligne #484. Après `make fr`, `marketing-brochure.pdf` est restauré (`git checkout`), seuls les PDF des deux manuels modifiés sont commités.
- **Écartées** : tout le CHANGELOG en 15-5e2 (une release entre les deux merges publierait #463 et #491 sans ligne) ; commiter la brochure régénérée (octets changés sans texte changé, bruit dans l'historique des PDF).
- **Réversible** : oui.

## C65 — 15-5d : dépend de la 15-5e1 seule

- **Contexte** : la 15-5d supposait « la 15-5e » en place : rejeu de la validation, de la saisie fournisseur et de la complétion d'import, avance des réglages de la saisie fournisseur, doc-comment canonique (étape (2 bis')), « 5 bis ». L'orchestrateur veut qu'elle ne dépende que du socle.
- **Retenu** : la 15-5e1 porte le rejeu de la validation, l'avance des réglages, le doc-comment canonique et « 5 bis » ; la 15-5d dépend d'elle seule. Le rejeu de la saisie fournisseur et de la complétion d'import vient avec la 15-5e2, dans un ordre de merge libre : si la 15-5d merge avant, ces deux routes portent ses verrous sans rejeu jusqu'au merge de la 15-5e2 — une victime y rend 500 comme aujourd'hui (fenêtre de même nature que l'état actuel, pas une régression de nature). Écrit dans les 15-5e1, 15-5e2, 15-5d et l'index.
- **Écartées** : (a) faire dépendre la 15-5d des deux sous-stories (contraire à la décision) ; (b) avancer en 15-5e1 le rejeu de la saisie fournisseur et de la complétion d'import — la seconde exige l'extraction d'une fonction « une tentative » à l'enveloppe `AppError`, c'est du rollout.
- **Réversible** : oui.

## C-15-5b-1 — 15-5b (dev) : un helper `errorMessageOf` plutôt que sept copies du motif

- **Contexte** : l'AC14 fait passer sept `catch` au motif `isApiError(e) ? e.message : (e instanceof
  Error ? e.message : String(e))`. Sept copies d'une même expression contredisent la règle DRY du
  `CLAUDE.md`, et une huitième copie divergente est le défaut le plus probable.
- **Retenu** : `errorMessageOf(err: unknown)` dans `frontend/src/lib/shared/utils/api-client.ts`, à côté
  d'`isApiError`, appelé par les sept sites ; même comportement que le patron de `ManualMatchModal`.
  La mutation du helper (branche `ApiError` retirée) fait rougir les sept tests AC14.
- **Écarté** : recopier l'expression à chaque site (lettre de l'AC14) ; migrer aussi les sites hors
  module (`reports/+page.svelte`, `settings/+page.svelte`) — signalés hors périmètre par la P1.
- **Réversible** : oui.

## C-15-5b-2 — 15-5b (dev) : tests de page nommés `*-page.test.ts`, pas `+page.test.ts`

- **Contexte** : T4 et T5 demandent un test neuf `bank-accounts/+page.test.ts` et un test de la page des
  règles. SvelteKit **réserve** le préfixe `+` dans `src/routes/` ; les tests de page existants s'y
  nomment `accounts-page.test.ts`, `contacts-page.test.ts`, etc.
- **Retenu** : `bank-accounts/bank-accounts-page.test.ts` et `reconciliation/rules/rules-page.test.ts`.
- **Réversible** : oui (renommage).

## C-15-5b-3 — 15-5b (dev) : un seul contrôle partagé dans le dépôt `accounts`

- **Contexte** : l'AC7, l'AC8 et l'AC12 placent la même lecture (`SELECT number, postable, active`,
  refus si `active && !postable`) dans deux dépôts (`reconciliation_rules`, `bank_accounts`), quatre
  sites.
- **Retenu** : `accounts::ensure_postable_if_active_in_tx`, appelé par les quatre sites ; il construit
  `DbError::accounts_not_postable`. La mutation M7d (condition `active` retirée) fait rougir le test du
  hors-périmètre « règle réactivée sur un compte archivé et non imputable ».
- **Écarté** : un helper privé par dépôt (deux copies).
- **Réversible** : oui.

## C-15-5b-4 — 15-5b (dev) : la mutation de #521 porte sur la résolution, pas sur le type

- **Contexte** : T6 demande, pour l'AC19, la mutation « champ remis en `Option<i64>` ». Remettre le type
  oblige à réécrire la résolution et la validation : la mutation ne retirerait plus une seule garde.
- **Retenu** : mutation `None => None` (absent → effacé, comportement d'avant #521) dans la résolution ;
  `absent_payable_account_is_preserved` rougit. Même pouvoir de détection, un seul point modifié.
- **Réversible** : oui (un test de mutation).

## C-15-5b-5 — 15-5b (revue) : clôture de la boucle sur 13 LOW acceptés

- **Contexte** : la passe P1 (Sonnet, trois lentilles) rend 0 CRITICAL, 0 HIGH, 0 MEDIUM et 13 LOW.
- **Retenu** : clore la boucle après P1. Seul A-1 = E1 (virgule de la documentation) est corrigé. B2
  (duplication, `errorMessageOf` non repris dans deux modales) et B4/E3 (lecture sans verrou des nouveaux
  contrôles) exigeraient de toucher la production, donc de rouvrir une passe de revue pour des défauts de
  niveau LOW ; ils sont tracés : B4 avec la dette de #522, B2 comme dette LOW de la fiche, B3/A-3 par #520,
  B1/E4 par la 15-5c (#492).
- **Écarté** : corriger B2/B4 dans cette story (la remédiation de production appelle une nouvelle passe,
  et « la sévérité se déplace vers ce qu'on vient d'écrire »).
- **Réversible** : oui (une story de dette).
## C-15-8-1 — La clé 15-8 passe à la modification d'une écriture (#532)
- **Contexte** : C2 réservait « 15-8 » à l'import automatique du dossier (#459, #458), en
  précisant que la numérotation pouvait changer. Le 2026-10-08, Guy a demandé en urgence la
  modification d'une écriture tant que l'exercice est ouvert (#532) ; l'orchestrateur a attribué
  la clé `15-8-modifier-une-ecriture`.
- **Retenu** : 15-8 = #532. L'import du dossier et le cycle fournisseurs prendront les numéros
  suivants à leur spécification (15-9, 15-10).
- **Réversible** : oui, ce n'est qu'un nom de clé ; aucun fichier de l'import n'existe encore.
## C-15-8-2 — La suppression est rouverte, dans le même cadre que la modification
- **Contexte** : #532 laisse la suppression « à trancher ». La 24-4b l'avait fermée avec la
  modification.
- **Retenu** : `DELETE /journal-entries/{id}` aboutit sous **exactement** les gardes de la
  modification (exercice ouvert, période non verrouillée, ni contre-passée ni contre-passation,
  aucune pièce), avec l'instantané complet au journal d'audit (`journal_entry.deleted`, déjà
  écrit par `delete_in_tx`). Le numéro n'est jamais réattribué (compteur de la 25-2-c) : le trou
  est visible et expliqué par l'audit.
- **Écartées** : (a) suppression toujours refusée — l'écriture d'ouverture saisie par erreur,
  ou une écriture en double, ne s'effacerait qu'en la contre-passant, trois écritures pour une ;
  et un cadre à deux règles pour deux gestes de même nature serait plus difficile à expliquer
  que le même cadre ; (b) suppression sans trace — contraire à #532 point 1.
- **Réversible** : oui — remettre le refus dans `delete_in_tx` (une garde).
## C-15-8-3 — Une écriture modifiée ne change ni de numéro ni d'exercice
- **Contexte** : le numéro est tiré d'un compteur **par exercice** (25-2-c) et l'unicité porte sur
  `(company_id, fiscal_year_id, entry_number)`. Déplacer une écriture dans un autre exercice
  obligerait à lui donner un numéro neuf.
- **Retenu** : la nouvelle date doit tomber dans l'exercice **de l'écriture** — sinon
  `400 DATE_OUTSIDE_FISCAL_YEAR` (message existant : « La date n'est pas dans l'exercice courant
  de cette écriture »). Le numéro, l'exercice et l'`id` sont immuables ; changer d'exercice se fait
  en supprimant puis en ressaisissant.
- **Écartée** : renuméroter dans l'exercice cible — un numéro changeant sous une écriture
  existante est ce que la 25-2-c a fermé.
- **Réversible** : oui, mais coûteux (renumérotation).
## C-15-8-4 — Les contrôles de saisie s'appliquent strictement, sans « grandfathering » de compte
- **Contexte** : l'ancien `update` (supprimé par la 24-4b) exemptait de la garde de postabilité
  les comptes déjà présents sur l'écriture (14-3b, D-A1). #532 exige « les mêmes contrôles qu'à
  la saisie (comptes actifs et imputables) ».
- **Retenu** : toutes les lignes, anciennes comme nouvelles, sont contrôlées comme à la création
  (`active = TRUE`, `postable = TRUE`). Un compte archivé ou devenu non imputable doit être
  remplacé pour enregistrer. Les **projets** déjà présents sur l'écriture restent tolérés même
  archivés (grandfathering de l'ancien `update`) : étiqueter n'est pas imputer — asymétrie déjà
  admise par la 24-4a.
- **Écartée** : reprendre l'exemption D-A1 — contraire à la lettre de #532, et elle laisserait une
  écriture modifiée aujourd'hui porter un compte qu'aucune saisie n'accepterait.
- **Réversible** : oui (paramètre `exempt_ids` déjà présent dans `validate_lines_accounts_in_tx`).
## C-15-8-5 — Le motif de refus réutilise l'inventaire de la contre-passation
- **Contexte** : il faut refuser la modification de toute écriture référencée par une pièce. La
  24-4a tient déjà l'inventaire exact (`reversal_blockers`) : contre-passation, contre-passée,
  facture, avoir, facture fournisseur (achat et règlement), règlement client (solde compris),
  transaction bancaire rapprochée, compte archivé.
- **Retenu** : la modification et la suppression réutilisent `reversal_blockers` **sans** le motif
  « compte archivé » (il ne gèle pas : on peut remplacer le compte). Codes : `ENTRY_IS_REVERSED`
  (409, existant, message élargi à « modifiée ni supprimée ») pour une écriture déjà contre-passée ;
  pour les autres motifs, un nouveau `DbError::EntryNotModifiable { blocker, document_id,
  document_label }` rendu en 409 **sous le code du motif** (`IS_A_REVERSAL`, `OWNED_BY_INVOICE`,
  …, `MATCHED_BANK_TRANSACTION`) avec `details.documentId` / `details.documentNumber` — la forme
  exacte du refus de contre-passation, que l'API externe documente déjà, et le même message (il
  nomme le chemin de la pièce). `ENTRY_IS_POSTED` et `DbError::EntryIsPosted` sont **retirés**
  (plus d'émetteur). Un garde-fou d'inventaire (`information_schema`) rougit si une table neuve
  référence `journal_entries` ou si `journal_entry_lines` gagne une colonne — c'est ce qui forcera
  le triage du lettrage (15-1a).
- **Écartée** : une seconde liste de propriétaires écrite à la main — deux inventaires divergent.
## C-15-8-6 — L'historique se lit au journal d'audit, depuis la fiche
- **Contexte** : #532 laisse à trancher l'affichage de l'historique des versions sur la fiche.
- **Retenu** : la fiche indique « Modifiée » quand `version > 1` (seule la modification fait
  bouger `version`) et porte un lien « Historique » vers `/audit-log?entityType=journal_entry&entityId={id}`
  (écran existant, réservé Comptable/Admin ; le lien est masqué pour Consultation). L'instantané
  avant/après, lignes comprises, est dans `details`.
- **Écartée** : un écran de versions avec différentiel ligne à ligne — hors de l'urgence ; le
  journal d'audit porte déjà l'information complète.
- **Réversible** : oui ; un écran de versions peut s'ajouter plus tard sans rien défaire.
## C-15-8-7 — Modifier et supprimer se font depuis la fiche, pas depuis la liste
- **Retenu** : la liste garde son seul lien vers la fiche (24-4b). La fiche porte « Modifier »
  (formulaire de saisie en mode édition, pré-rempli) et « Supprimer » (confirmation), absents —
  pas grisés — avec le motif quand l'écriture n'est pas modifiable. La fiche est l'endroit où le
  motif de refus et la contre-passation sont déjà expliqués.
- **Écartée** : remettre ✎ et 🗑 sur chaque ligne de la liste — il faudrait y calculer
  l'éditabilité ligne par ligne (cinq jointures), ce que la 24-4a a refusé pour la liste.
## C-15-8-8 — `PUT` et `DELETE` restent ouverts aux clés API en écriture
- **Retenu** : comme la création, la contre-passation et la dévalidation, ils sont ouverts aux clés
  `read-write` ; l'API externe et le manuel administrateur le disent. La trace d'audit porte la clé.
- **Écartée** : les réserver à l'interface — une intégration qui crée des écritures doit pouvoir
  corriger les siennes, et le précédent de la dévalidation (#440) va dans ce sens.
## C-15-8-9 — Pas de découpage : la règle n'est pas déclenchée
- **Contexte** : cinq zones touchées (`kesh-db`, `kesh-api`, `kesh-i18n`, `frontend`, `docs`) —
  le seuil est « plus de cinq ». Un seul mécanisme (deux verbes rouverts sous une garde commune),
  aucune migration.
- **Retenu** : story unique. **Ligne de découpe pré-déclarée** si la validation montre une
  non-convergence par recyclage : la suppression (AC 9–11, T3, son bouton et son E2E) part en
  15-8b ; la modification de l'écriture manuelle et d'ouverture reste en 15-8a.
- **Réversible** : oui.

## C-15-8-10 — Ordre des verrous du `PUT` : l'écriture seule, puis les projets, puis l'exercice

- **Contexte** : validation P1 (R1/F1, HIGH). Le `FOR UPDATE` de l'écriture doit être le premier acte de la
  transaction (décision de l'orchestrateur), sans quoi la garde lit une vue `REPEATABLE READ` antérieure à l'attente
  du verrou. Mais l'ancien `update` validait les projets (sentinelle `companies` puis `FOR UPDATE` des projets)
  **avant** de verrouiller l'écriture **avec son exercice**.
- **Retenu** : étape 1 = l'écriture **seule** (`FOR UPDATE` sans jointure) ; 1-bis = projets (sentinelle puis projets,
  seulement pour les tags nouveaux) ; 1-ter = l'exercice ; 1-quater = borne lue sans verrou. Le **refus** projet est
  gardé et rendu à l'étape 6, avec les comptes, pour que l'exercice clos et le gel parlent d'abord (AC 7).
- **Écartées** : verrouiller écriture et exercice ensemble puis les projets — inverse l'ordre de la création
  (`companies → projects → fiscal_years`) et fait se bloquer en croix deux saisies taguées du même exercice ; garder
  les projets avant l'écriture en lecture verrouillante — contraire à la décision « verrou de l'écriture d'abord ».
- **Vérifié** : aucun appelant de la sentinelle ne verrouille ensuite une écriture existante (grep, 2026-10-08) ; à
  revérifier au développement. Un cycle résiduel serait un 1213 d'InnoDB, pas une attente infinie.
- **Réversible** : oui (ordre interne d'une fonction).

## C-15-8-11 — Un `PUT` identique sur une écriture à compte archivé rend 400, pas 200

- **Contexte** : R2 — l'AC 3 (« `PUT` identique → 200 ») contredisait l'AC 4 (« compte archivé, y compris sur une ligne
  inchangée → 400 ») pour ce cas.
- **Retenu** : 400 `INACTIVE_OR_INVALID_ACCOUNTS`. Le no-op vient après toutes les gardes (héritage KF-004) ; le test
  `update_no_op_with_inactive_account_returns_inactive_error` de l'ancien `update` est rétabli tel quel.
- **Écartée** : 200 no-op — un « rien à faire » sur une écriture que l'enregistrement refuserait ferait croire
  qu'elle est saine.
- **Réversible** : oui.

## C-15-8-12 — Après suppression de l'ouverture seule, la nouvelle ouverture porte le numéro 2

- **Contexte** : R10 — le compteur de la 25-2-c ne réattribue jamais un numéro.
- **Retenu** : l'accepter, le dire au manuel (§ soldes de départ) et l'asserter (AC 13). Le trou est expliqué par
  l'instantané `journal_entry.deleted`.
- **Écartée** : réinitialiser le compteur quand la société redevient vierge — rouvrirait la réattribution que la
  25-2-c a fermée, pour un gain cosmétique.
- **Réversible** : oui.

## C-15-8-13 — Deux fonctions : la garde d'écriture et le motif d'écran

- **Contexte** : R5 — une seule `modification_blocker` devait servir l'écriture (filtre D2 seul) et l'écran (neuf
  motifs, dont l'exercice clos et la période), avec deux précédences et deux vocabulaires de codes.
- **Retenu** : `modification_guard` (D2 seule, rend `Option<ReversalBlockerHit>`, `AccountArchived` exclu) partagée
  par `update` et `delete_in_tx`, convertie en erreur par `modification_refusal` ; `modification_blocker` (écran)
  l'appelle, entre l'exercice clos et la période, et rend `Option<ModificationBlocker>`. Une table de correspondance
  écrite en dur (seul écart : `ALREADY_REVERSED` ↔ `ENTRY_IS_REVERSED`) fait l'assertion de l'AC 14.
- **Écartée** : renommer `ALREADY_REVERSED` à l'écran en `ENTRY_IS_REVERSED` — perdrait la réutilisation des clés
  `journal-entries-reverse-blocked-*` et créerait un second vocabulaire côté écran.
- **Réversible** : oui.

## C-15-8-14 — Rôle Consultation : « Modifier », « Supprimer » et « Contre-passer » masqués

- **Contexte** : R10 — « Contre-passer » est aujourd'hui affiché à Consultation et rend 403 au clic ; « Modifier » et
  « Supprimer » auraient hérité du même défaut.
- **Retenu** : les trois boutons, et le lien « Historique », absents pour ce rôle (lu dans `authState`) ; le 403 du
  serveur reste le refus qui fait autorité. « Contre-passer » est aligné dans la même story, puisque le geste est le
  même et sur la même page.
- **Écartée** : assumer des boutons qui échouent en 403 — un bouton qui ne peut qu'échouer ne renseigne pas.
- **Réversible** : oui.

## C-15-8-15 — Le formulaire se rétablit par inversion du commit du gel, sur cinq fichiers seulement

- **Contexte** : F3 — « rétablir depuis `d2910022` » écraserait la prop `booksLockedThrough` et le `min` de date de la
  24-4c (`54ae4a70`).
- **Retenu** : `git show 08e20353 -- <fichier> | git apply -R --3way` sur `form-helpers.ts`, `form-helpers.test.ts`,
  `journal-entries.api.ts`, `journal-entries.types.ts` (nets, vérifié par `--check`) et `JournalEntryForm.svelte`
  (conflit à résoudre en gardant la 24-4c). **Pas** sur la page de liste (le mode édition y vivait, C-15-8-7 l'écarte),
  ni sur la spec E2E de liste, ni sur `i18n-keys.test.ts` (cinq fois modifié depuis), ni à l'aveugle sur
  `e2e-selecteurs-traduits.test.ts`.
- **Écartée** : copier l'état `d2910022` — perte silencieuse de la protection de saisie 24-4c.
- **Réversible** : oui.

## C-15-8-16 — La préparation extraite du `POST` passe avant son pré-contrôle d'exercice

- **Contexte** : R11 / D4 — la préparation (trim, longueurs, parse, `accounting::validate`) est extraite dans
  `prepare_new_journal_entry`, commune au `POST` et au `PUT`. Dans le `POST` actuel, `find_covering_date` s'intercale
  entre le parse et `validate`.
- **Retenu** : `prepare_new_journal_entry` d'un bloc, puis `find_covering_date` : un corps à la fois déséquilibré
  **et** sans exercice rend désormais `ENTRY_UNBALANCED` avant `NO_FISCAL_YEAR`. Cohérent avec « les refus de forme
  précèdent toute lecture de la base ». Le développeur grepe les tests du `POST` qui cumuleraient les deux causes.
- **Écartée** : couper la fonction en deux (parse / validate) pour garder l'ordre exact — deux fonctions là où une
  suffit, pour un cas de double faute.
- **Réversible** : oui.

## C-15-8-17 — Découpage de la 15-8 en 15-8a (modifier) et 15-8b (supprimer)

- **Contexte** : validation P2, finding F6 (MEDIUM) — comptée à la granularité de la règle de découpage (« modules
  métier de premier niveau »), la story touche bien plus de cinq modules ; C-15-8-9 n'en comptait que cinq « zones » et
  omettait `kesh-report`. Ce n'est pas un recyclage, et l'amendement D5 ne couvre pas ce critère : le signal est déclaré.
  Décision de l'orchestrateur, pour livrer vite ce que Guy attend (corriger son ouverture inversée).
- **Retenu** : la ligne de découpe pré-déclarée par C-15-8-9. **15-8a** — la modification (`PUT`) de l'écriture manuelle
  et d'ouverture, sa garde, son audit avant/après, l'écran d'édition, le manuel et la doc de ce geste (`refs #532`).
  **15-8b** — la suppression (`DELETE`), l'historique visible sur la fiche (« Modifiée », « Historique »), le retrait
  d'`ENTRY_IS_POSTED`, le reste du manuel (`closes #532`). 15-8b dépend de 15-8a. La fiche 15-8 devient l'index
  (`split`). Révise C-15-8-9.
- **Écartées** : garder une story unique (signal levé, règle non dérogeable par l'argument « un seul mécanisme ») ;
  couper par couche (backend / écran) — livrerait un `PUT` sans écran, inutilisable par Guy.
- **Conséquence assumée** : entre les deux merges, le `DELETE` garde `409 ENTRY_IS_POSTED` ; le manuel de la 15-8a dit
  vrai pour cet état intermédiaire.
- **Réversible** : oui (refusionner deux fiches non développées).

## C-15-8-18 — La 15-8a passe après la 15-5a : deux refus de compte, pas un

- **Contexte** : validation P2, finding F1 (HIGH). La 15-5a (revue close, PR #535), ordonnée avant la 15-8 (C2), a
  retiré `exempt_ids` de `validate_lines_accounts_in_tx`, introduit `400 ACCOUNT_NOT_POSTABLE` avec
  `details.rejected[{accountId, accountNumber}]` (compte actif non imputable), laissé `400 INACTIVE_OR_INVALID_ACCOUNTS`
  au compte inconnu, d'une autre société ou archivé (qui prime), et créé `## [0.13.0] — Non publié`. La fiche 15-8
  attendait un seul refus et chargeait T2 de retirer `exempt_ids`.
- **Retenu** : la 15-8a s'écrit contre l'état « 15-5a mergée » et se rebase sur `main` après ce merge (T0). Partout,
  les deux refus sont séparés : AC 3, AC 4, AC 12 (table de correspondance), D4 étape 6. **C-15-8-11 est révisé** : un
  `PUT` identique sur une écriture à compte archivé rend `INACTIVE_OR_INVALID_ACCOUNTS`, à compte devenu non imputable
  `ACCOUNT_NOT_POSTABLE` (test jumeau neuf). `test_update_refuses_a_line_on_an_account_made_non_postable` attend
  `AccountsNotPostable` nommant le compte. T2 ne retire plus `exempt_ids`. Le formulaire garde le
  `case 'ACCOUNT_NOT_POSTABLE'` de la 15-5a ; `docs/api-external.md` ajoute le `PUT` à la liste des routes qui le rendent.
- **Correction d'une entrée antérieure** : la réversibilité écrite à C-15-8-4 (« paramètre `exempt_ids` déjà présent »)
  est périmée — le paramètre n'existe plus après la 15-5a ; revenir sur C-15-8-4 exigerait de le réintroduire (finding
  R2-9).
- **Écartée** : écrire la 15-8a contre `main` d'avant la 15-5a — conflit garanti sur `validate_lines_accounts_in_tx`
  et sur le formulaire, et un contrat de refus que le serveur ne rendrait plus.
- **Réversible** : oui.

## C-15-8-19 — Rejeu sur interblocage du `PUT` (et du `DELETE`), cycles nommés, projets lus en verrou partagé

- **Contexte** : validation P2, findings F2, R2-2, R2-3 (MEDIUM). L'ordre écriture → [companies → projets] → exercice
  n'entre en cycle avec aucune sentinelle, mais il entre dans trois cycles **hérités** : exercice ↔ compte (règlement,
  complément : compte puis exercice ; le `PUT` : exercice puis verrou partagé de clé étrangère sur le compte) ;
  exercice ↔ `companies` (le `PUT` tient la sentinelle et attend l'exercice ; une création tient l'exercice et prend le
  verrou partagé de clé étrangère `company_id`) ; projet ↔ exercice (avec la contre-passation d'une écriture taguée). Le
  `PUT` n'avait pas de `retry_with`, et la fiche affirmait « jamais une attente infinie », contre la doctrine de
  `retry.rs:7-10` et de Pattern 5 (50 s puis 500). Par ailleurs la lecture des projets existants, ordinaire, ouvrait la
  vue `REPEATABLE READ` avant les verrous des projets et de l'exercice.
- **Retenu** : le handler `PUT` est enveloppé dans `retry_with(DEFAULT_MAX_DEADLOCK_ATTEMPTS, is_deadlock_error, …)` ;
  les trois cycles sont nommés dans la fiche et le doc-comment ; la phrase fausse est retirée (1213 rejoué, 1205 non
  rejoué → 500) ; le `PUT` entre à la « Deny list » de Pattern 5 (`docs/MULTI-TENANT-SCOPING-PATTERNS.md`) avec ordre,
  raison et mitigation ; la lecture des projets existants se fait en `LOCK IN SHARE MODE` — la vue s'ouvre ainsi après
  le dernier verrou (une seconde mutation la tient, AC 8) ; un test à deux connexions établit que le cycle projet ↔
  exercice produit bien un interblocage (AC 9). Le `DELETE` de la 15-8b est enveloppé de même : son rejeu coûte trois
  lignes, sans effet hors de la transaction.
- **Écartées** : (b) de R2-2 — garder une lecture ordinaire et dire que la vue s'ouvre à 1-bis : plus faible pour un
  coût nul de la lecture verrouillante (l'écriture est déjà tenue en exclusif) ; réordonner les verrous pour supprimer
  les cycles — impossible sans contredire la règle « le verrou de l'écriture d'abord » (D2), et le dépôt n'a pas d'ordre
  unique (`opening_complement.rs:26-35`).
- **Réversible** : oui.

## C-15-8-20 — Le paiement détaché d'une facture fournisseur annulée reste gelé, par la trace d'audit

- **Contexte** : validation P2, finding F3 (MEDIUM). `supplier_invoices::cancel_in_tx` annule une facture **payée**
  en contre-passant l'achat, **sans** contre-passer le règlement, puis remet `settlement_journal_entry_id` à `NULL`
  (arbitrage de Guy du 2026-09-26). Plus aucune colonne ne référence l'écriture de règlement : sous le cadre de la 15-8,
  elle devenait modifiable et supprimable — une sortie de banque réelle. Consigne de l'orchestrateur : chercher un
  marqueur structurel de l'origine ; sinon, l'annulation doit laisser une trace qui gèle l'écriture, par la solution la
  moins invasive.
- **Constat au sol** : **aucun marqueur structurel d'origine** — `journal_entries` n'a que `journal` (partagé avec la
  saisie manuelle) et `reverses_entry_id` ; l'audit `journal_entry.created` est écrit pour tous les flux par
  `create_in_tx_inner`. La seule trace est l'audit du geste : `supplier_invoice.cancelled`,
  `details_json.settlementJournalEntryId` (`supplier_invoices.rs:931-935`), écrit depuis la 25-3-c.
- **Retenu** : la garde de modification (`modification_guard`, 15-8a D2) lit cette trace — jointure
  `supplier_invoices` (même société, `status = 'cancelled'`) × `audit_log` (index `idx_audit_log_entity`), sur
  `JSON_VALUE(details_json, '$.settlementJournalEntryId')` — et refuse en **409 `DETACHED_SUPPLIER_SETTLEMENT`**, avec
  l'id et le numéro de la facture. Le motif est **hors** `reversal_blockers` : la contre-passation du paiement reste
  offerte, comme le manuel le promet. Nouveau code d'écran (dix au lieu de neuf), nouvelle clé
  `journal-entries-modify-blocked-detached-settlement`, cas de test à l'AC 6 (15-8a) et à l'AC 4 (15-8b), manuel
  `user-manual.tex:1331-1337` et FAQ `:2140`. Ni migration, ni changement du geste d'annulation.
- **Écartées** : garder la colonne `settlement_journal_entry_id` à l'annulation — rendrait le paiement
  `OwnedBySupplierInvoice`, donc **non contre-passable**, et changerait le sens d'une colonne qu'une facture `cancelled`
  ne porte pas aujourd'hui (revient sur l'arbitrage du 2026-09-26) ; contre-passer aussi le règlement à l'annulation —
  idem ; une colonne ou une table neuve qui garde le lien — migration, hors du périmètre d'une story urgente ; accepter
  le paiement comme une écriture manuelle — écarté par la consigne.
- **Limite assumée, signalée pour une issue** : une référence lue dans un journal d'audit n'est pas une clé étrangère —
  le garde-fou d'inventaire D3 ne la voit pas, et un futur geste de « rattachement » d'un paiement détaché (le manuel
  l'annonce) devra la revoir. Une issue doit porter la colonne structurelle qui la remplacera.
- **Réversible** : oui (une branche de la garde).

## C-15-8-21 — Formulaire : la modale de conflit ne revient pas avec l'inversion du gel

- **Contexte** : validation P2, findings R2-1 et F4 (MEDIUM), F8 (LOW). L'inversion de `08e20353` sur
  `JournalEntryForm.svelte` (C-15-8-15) restaure la modale de conflit de la Story 3.3 (`showConflictDialog`,
  `case 'OPTIMISTIC_LOCK_CONFLICT'` qui l'ouvre, prop `onConflictReload`, `handleConflictReload`, balisage, clés
  `journal-entry-conflict-*`), que D8 remplace par un toast puis un rechargement. Les clés neuves du formulaire
  étaient nommées `journal-entry-…`, préfixe que `lint-i18n-ownership` refuse dans `features/journal-entries/`.
- **Retenu** : la modale est **écartée** du bloc inversé, nommément ; une prop neuve `onStale` (appelée après le toast
  par `FISCAL_YEAR_CLOSED`, les 409 de course et `OPTIMISTIC_LOCK_CONFLICT`) remplace `onConflictReload` ;
  `{#if !isEdit}` autour de l'assistant TVA est **repris** (l'assistant compose une écriture d'achat neuve, réservé à la
  création) ; les trois entrées périmées `journal-entry-conflict-*` de `KNOWN_VIOLATIONS` sont retirées ; les clés du
  formulaire prennent le préfixe `journal-entries-` (`journal-entries-edit-conflict`,
  `journal-entries-line-account-unusable`) ; `case 'PERIOD_LOCKED'` est nommé.
- **Écartées** : garder la modale (contraire à D8, et elle rouvre quatre clés retirées) ; inscrire les nouvelles clés à
  `KNOWN_VIOLATIONS` (agrandit la dette #30) ; offrir l'assistant TVA en édition (il ajoute des lignes d'achat à une
  écriture existante, cas non spécifié).
- **Réversible** : oui.

## C-15-8-22 — Modification et suppression refusées dès qu'un exercice postérieur est clos

- **Contexte** : validation P3, finding F1 de la 15-8b (MEDIUM), qui vaut pour la 15-8a. Le bilan est **cumulatif**
  (`kesh-report/src/balance_sheet.rs:9`, « tous exercices confondus ») et `fiscal_years::close` n'exige pas que
  l'exercice précédent soit clos (`fiscal_years.rs:739` ; seule `reopen` a une garde LIFO, `find_later_closed_in_tx`).
  L'état « N ouvert, N+1 clos » est atteignable : modifier ou supprimer une écriture de N réécrivait le bilan d'un N+1
  tenu pour clos. Décision de l'orchestrateur (consigne 1 de la passe).
- **Retenu** : nouvelle condition du cadre D1 (15-8a) : aucun exercice **postérieur** à celui de l'écriture n'est clos.
  Refus **400 `LATER_FISCAL_YEAR_CLOSED`** (`DbError::LaterFiscalYearClosed { fiscal_year_id, fiscal_year_name }`,
  `details.fiscalYearId` / `fiscalYearName`, clé `journal-entries-modify-blocked-later-fiscal-year-closed` partagée par
  le serveur et l'écran) ; précédence : juste après `FISCAL_YEAR_CLOSED`, avant tout 409 ; verrou : réutilisation de
  `find_later_closed_in_tx` (`FOR UPDATE` sur l'intervalle `start_date > ?`), **après** l'exercice de l'écriture —
  ordre écriture → [companies → projets] → exercice → exercices postérieurs (`PUT`), écriture + exercice → exercices
  postérieurs (`DELETE`, étape 2-bis, avant la première lecture ordinaire) ; motif d'écran `LATER_FISCAL_YEAR_CLOSED`
  (onze codes au lieu de dix) lu par une sœur non verrouillante, `find_later_closed`, au `SELECT` partagé ; tests de
  refus, de précédence et de concurrence (clôture de N+1 non commitée), avec mutation ; manuel (§ clôture).
- **Écartées** : l'inscrire au registre comme angle mort assumé — le défaut fausse un bilan clos, exactement ce que la
  clôture protège ; imposer l'ordre des clôtures dans `fiscal_years::close` — ferme la cause pour l'avenir, mais pas les
  installations où l'état existe déjà, et change un geste hors du périmètre de #532 ; un `400 FISCAL_YEAR_CLOSED`
  réutilisé — il dirait « l'exercice de cette date est clos », faux ici.
- **Coûts assumés** : le verrou d'intervalle sérialise brièvement le `PUT`/`DELETE` d'une écriture ancienne avec les
  créations de tous les exercices postérieurs ; les trois cycles hérités valent aussi pour les exercices postérieurs
  (rejoués par `retry_with`).
- **Signalé pour une issue** : le même trou existe pour la création, la dévalidation, le règlement et toute écriture
  datée dans un exercice antérieur à un exercice clos — défaut préexistant, hors périmètre.
- **Réversible** : oui (une étape de la garde).

## C-15-8-23 — Projets existants lus par une lecture ordinaire après le verrou de l'écriture (corrige C-15-8-19)

- **Contexte** : validation P3, findings R3-2 (15-8a) et F1 (15-8a), MEDIUM. Le `LOCK IN SHARE MODE` retenu par
  C-15-8-19 sur `journal_entry_lines` posait, sous `REPEATABLE READ`, un verrou d'intervalle jusqu'à l'enregistrement
  suivant (jusqu'au *supremum* pour la dernière écriture), **avant** le verrou de l'exercice : quatrième cycle, non
  hérité, avec une création du même exercice — dont la victime pouvait être la création, non rejouée (500). « Coût
  nul » était faux.
- **Retenu** (consigne 2 de l'orchestrateur) : lecture **ordinaire** des projets existants **après** l'étape 1 —
  l'écriture verrouillée, ses lignes sont stables. La vue `REPEATABLE READ` s'ouvre donc à 1-bis, avant les verrous des
  projets et des exercices : la garde reste juste (tout ce qui la change passe par la ligne verrouillée de l'écriture ;
  le paiement détaché est sûr par atomicité du commit) ; l'exercice et les exercices postérieurs sont lus en lecture
  verrouillante (état courant) ; la borne `books_locked_through` peut être périmée — même tolérance qu'à la création.
  La seconde mutation de l'AC 8 tombe. Les cycles restants (les trois hérités, étendus aux exercices postérieurs) sont
  nommés et couverts par `retry_with`.
- **Écartées** : garder le verrou partagé et nommer le quatrième cycle — il frappe une création non rejouée ;
  verrouiller tous les projets du corps puis trier « anciens / nouveaux » après la vue — exige une variante de
  `validate_taggable_in_tx` qui tolère l'archivé, pour un gain limité à la fraîcheur de la borne.
- **Correction d'une entrée antérieure** : C-15-8-19, point « la lecture des projets existants se fait en `LOCK IN
  SHARE MODE` » et option écartée « (b) de R2-2 … coût nul » — révisés ici.
- **Réversible** : oui.

## C-15-8-24 — `modification_guard` prend une connexion ; corrections de renvois de C-15-8-5, C-15-8-10, C-15-8-12, C-15-8-13

- **Contexte** : validation P3, finding R3-1 de la 15-8a (MEDIUM). `modification_guard(executor, …)` générique enchaîne
  deux lectures (`reversal_blockers`, puis la trace d'audit) ; `reversal_blockers` prend son exécuteur par valeur : un
  `E: Executor` ne sert qu'une fois. La correction R2-15 avait déplacé le défaut sur `modification_guard` au lieu de le
  corriger. LOW R3-3 (15-8a) et R3-3/F8 (15-8b) : renvois périmés du registre.
- **Retenu** (consigne 3) : `modification_guard(conn: &mut sqlx::MySqlConnection, company_id, id)` — `update` et
  `delete_in_tx` passent `&mut **tx` ; `modification_blocker(pool: &MySqlPool, …)` fait `pool.acquire()` et enchaîne ses
  cinq lectures sur la connexion.
- **Corrections d'entrées antérieures** (sans les réécrire) :
  - **C-15-8-5** : `DbError::EntryNotModifiable { blocker, document_id, document_label }` est devenu
    `DbError::EntryNotModifiable(ModificationGuard)` (tuple, `ModificationGuard::Owned { … }` ou
    `::DetachedSupplierSettlement { … }`, C-15-8-20) ; `ENTRY_IS_POSTED` n'est retiré qu'à la 15-8b.
  - **C-15-8-10** : la phrase « Un cycle résiduel serait un 1213 d'InnoDB, pas une attente infinie » est fausse
    (C-15-8-19 l'a retirée de la fiche) : un cycle non détecté finit en 1205 après 50 s, non rejoué, donc 500. L'ordre
    s'est aussi allongé des exercices postérieurs (C-15-8-22).
  - **C-15-8-12** : « l'asserter (AC 13) » — c'est l'**AC 7** de la 15-8b depuis le découpage.
  - **C-15-8-13** : `modification_guard` rend `Result<Option<ModificationGuard>, DbError>` (pas
    `Option<ReversalBlockerHit>`), prend une connexion ; la table de correspondance est l'assertion de l'**AC 12** de
    la 15-8a (pas l'AC 14) ; `modification_blocker` lit aussi l'exercice postérieur clos.
- **Écartée** : `modification_guard` sur `&mut Transaction` — l'écran n'a pas de transaction ; en ouvrir une pour lire
  serait plus lourd qu'un `acquire()`.
- **Réversible** : oui.

## C-15-8-25 — Paiement détaché lu dans l'audit : trois réserves écrites, dette #541, requête bornée par société

- **Contexte** : validation P3, finding F7 de la 15-8a (LOW, trois réserves non écrites).
- **Retenu** (consigne 6) : la fiche écrit (1) que la lecture d'audit **révise** le garde-fou de la 25-3-c (« ne pas le
  rechercher dans l'audit », `supplier-invoices/[id]/+page.svelte:409-411`, arbitrage Q1) — le commentaire se reformule ;
  (2) le faux positif possible après restauration (fusion d'`audit_log`, `backup.rs:438-455`) — limite assumée, dite au
  manuel administrateur ; (3) le coût de `JSON_VALUE`, non indexable, à chaque `GET` et `PUT`. Renvoi à **#541** (dette
  structurelle : la colonne qui garde le lien). Requête bornée par société des deux côtés : `si.company_id = ?` **et**
  `al.company_id = ?` (filtre strict, patron `audit_log.rs:204`) ; faux négatif résiduel nommé (trace sans société).
- **Écartée** : `al.company_id = ? OR al.company_id IS NULL` — rouvre le faux positif inter-sociétés que le filtre ferme,
  pour un cas (utilisateur sans société au moment de l'annulation) que l'écrivain ne produit pas en pratique.
- **Réversible** : oui (une condition de jointure).

## C-15-8-26 — `README.md:29` édité par la 15-8b seule

- **Contexte** : validation P3, findings R3-4 et F3 de la 15-8b (LOW) : les deux fiches éditaient la même ligne, et la
  15-8b citait « … et supprimables » comme s'il existait déjà.
- **Retenu** : la 15-8b seule écrit la forme finale — « écritures validées, modifiables et supprimables tant que
  l'exercice est ouvert ». La 15-8a n'y touche pas : entre les deux merges, la ligne tait la modification sans rien
  affirmer de faux. `README.md:219` (ligne de la v0.12.0) est historique et reste.
- **Écartée** : la 15-8a écrit « modifiables », la 15-8b complète — conflit de merge garanti sur une ligne, pour un
  état intermédiaire de quelques jours.
- **Réversible** : oui.

## C-15-8-27 — Contrôles de manuel sur les PDF aplatis, motifs complétés, `.ftl` contrôlé ; message d'`ENTRY_IS_POSTED` réécrit

- **Contexte** : validation P3, findings R3-6, F6, F9 (15-8a), R3-6, F9 (15-8b), F4 (15-8a) — tous LOW. Le balisage TeX
  (`\textbf{imposée}`) coupe les tournures dans le `.tex` ; des formulations échappaient aux motifs (« seul chemin »,
  « passent par contre-passation », « ni modifiée ni supprimée ») ; la tournure naturelle du paiement détaché heurtait un
  motif ; le `.ftl` n'était pas contrôlé, et le message d'`ENTRY_IS_POSTED` (« ne se modifie plus … contre-passez-la »)
  devenait faux dès la 15-8a pour le `DELETE` d'une écriture modifiable.
- **Retenu** (consigne 7) : le contrôle **fait foi sur les PDF aplatis** (`grep -oiE`, qui compte les occurrences) ;
  motifs complétés (`seul chemin`, `passent par (la )?contre-passation`, `seule voie de correction`, `ni modifiée ni
  supprimée`) ; `fr-CH/messages.ftl` grepé, les trois autres locales relues à la main ; lignes de base mesurées
  (15-8a : 11 / 5 / 2 ; 15-8b : 5 / 2 / 0) ; tournures **prescrites** pour le paiement détaché et pour la clé de
  l'exercice postérieur clos, qui ne heurtent aucun motif. La 15-8a réécrit la clé `journal-entries-blocked-posted`
  (« … ne se supprime pas. Pour la corriger, modifiez-la tant que son exercice est ouvert, ou contre-passez-la »), son
  repli Rust et le `#[error]` ; la 15-8b les retire.
- **Écartée** : garder le contrôle sur le `.tex` — passe à tort sur les phrases balisées ; exiger que 15-8a et 15-8b
  partent dans la même release au lieu de réécrire le message — contrainte de calendrier pour une story urgente.
- **Réversible** : oui.

## C-15-8-28 — 15-8a : dérogation écrite à la règle de splitting, pas de nouveau découpage

- **Contexte** : validation P3, finding F2 de la 15-8a (MEDIUM). La fiche touche encore plus de cinq modules de premier
  niveau ; l'exception du `CLAUDE.md` (cycle Cargo, merge non testable) ne s'applique pas — un `PUT` sans écran est
  testable.
- **Retenu** (consigne 4) : **pas** de découpage supplémentaire ; section « Dérogation règle de splitting » écrite dans
  la 15-8a — story URGENTE demandée par le Project Lead, déjà issue d'un découpage (15-8 → 15-8a / 15-8b), et un `PUT`
  sans écran ne rend aucun service à l'utilisateur qui l'attend ; risque accepté (revue d'un périmètre large), contenu
  par la passe ciblée de fin de boucle. **Repli écrit** si une passe recycle : 15-8a-1 (API : D1–D7, D9, D10 ; AC 1–13,
  15, 16, 18) / 15-8a-2 (écran et manuel : D8 ; AC 14, 17).
- **Écartée** : couper maintenant en 15-8a-1 / 15-8a-2 — retarde la correction que Guy attend, sans défaut de conception
  qui l'exige (les findings P3 sont distincts et ne recyclent pas).
- **Réversible** : oui (le repli est écrit).

## C-15-8-29 — 15-8b : l'exercice postérieur clos ne garde le `DELETE` que sur le chemin de la route

- **Contexte** : C-15-8-22 au `DELETE`. `delete_in_tx` a deux appelants : la route (`enforce_ownership = true`) et
  `invoices::unvalidate` (`false`), dont l'AC 3 de la 15-8b exige le comportement inchangé.
- **Retenu** : l'étape 2-bis (exercices postérieurs clos) ne s'exécute que si `enforce_ownership` ; la dévalidation garde
  son comportement, et le même trou y est signalé pour une issue avec la création et le règlement (C-15-8-22).
- **Écartée** : l'appliquer aussi à la dévalidation — change un flux de facturation hors du périmètre de #532, sans
  spécification de son message ni de son écran.
- **Réversible** : oui.

## C-15-8a-1 — 15-8a (dev) : bases dédiées recréées après un redémarrage du conteneur par l'autre agent

- **Contexte** : développement en parallèle de la 15-5b sur la même machine. Les bases `kesh_158` / `kesh_e2e_158`,
  créées au début, ont **disparu** en cours de route (`Unknown database 'kesh_158'`, MariaDB à 461 s d'uptime) : le
  conteneur `kesh-mariadb-dev` (datadir en tmpfs) a été redémarré par la « remise à zéro » de l'autre agent.
- **Retenu** : un script de remise à zéro propre à ce worktree (scratchpad `reset158.sh` : `DROP/CREATE`, `GRANT`,
  migrations, seed sur `kesh_158` seulement), rejoué avant chaque gate ; jamais de redémarrage du conteneur, jamais de
  geste sur `kesh` ni `kesh_e2e`.
- **Écartée** : redémarrer le conteneur moi-même — interdit, et aurait effacé les bases de l'autre agent.
- **Réversible** : oui. ⚠️ **À signaler à l'orchestrateur** : la « remise à zéro » du `CLAUDE.md` (redémarrage du
  conteneur) efface les bases de **tous** les agents ; un gate en cours chez l'un tomberait.

## C-15-8a-2 — 15-8a (dev) : l'identifiant du paiement détaché se compare en CHAÎNE à `JSON_VALUE`

- **Contexte** : D2 laissait ouvert « vérifier au premier test que MariaDB 10.11 compare numériquement, ou lier l'id en
  chaîne ».
- **Retenu** : `JSON_VALUE(al.details_json, '$.settlementJournalEntryId') = ?` avec l'id lié en **chaîne** : comparaison
  exacte, sans conversion implicite en `DOUBLE`. Vérifié par le **chemin réel** create → pay → cancel
  (`supplier_invoices_repository.rs`, `cancel_paid_invoice_detaches_its_settlement`, étendu), et non par une trace posée
  à la main.
- **Écartée** : lier un entier et compter sur la conversion — exacte pour des identifiants, mais implicite.
- **Réversible** : oui.

## C-15-8a-3 — 15-8a (dev) : les motifs traduits de la fiche extraits dans `blocker-messages.ts`, sous des noms en `…Label`

- **Contexte** : la fiche traduisait les huit motifs de contre-passation (`blockedLabel`) ; la modification en partage
  sept (D8, « réutiliser les clés »). Vitest exige un test par code (onze).
- **Retenu** : `lib/features/journal-entries/blocker-messages.ts` — `reversalBlockerLabel` (le `blockedLabel` déplacé) et
  `modificationBlockerLabel`, qui délègue au premier pour les sept codes communs (une seule source) ; testé code par
  code. Noms en `…Label` **à dessein** : un nom en `…Message` faisait sortir les deux fonctions du relevé de
  `i18n-libelle-en-dur.test.ts` (46 → 45, un compteur qui baisse parce que le détecteur ne voit plus).
- **Écartées** : un second `switch` dans la fiche (DRY, et non testable par Vitest) ; le nom `…Message` (angle mort).
- **Réversible** : oui.

## C-15-8a-4 — 15-8a (dev) : les refus en mode édition classés par une fonction pure, `editRefusalOutcome`

- **Contexte** : D8 veut une branche nommée par code (pas le `default`) pour les refus qui rechargent la fiche, et des
  tests Vitest de ces branches.
- **Retenu** : `form-helpers.ts::editRefusalOutcome(code) → 'stale' | 'stay' | 'other'`, un `case` par code ; le
  formulaire en déduit toast puis `onStale` (`stale`) ou toast seul (`stay`). Testée seule **et** par un test de
  composant (`JournalEntryForm.edit.test.ts`) : PUT avec la version, `FISCAL_YEAR_CLOSED` sans
  `notifyMissingFiscalYearOrFallback`, quatre 409/400 de course, conflit de version sans modale, refus de saisie qui
  laissent le formulaire ouvert, ligne à compte archivé signalée, bornes de date.
- **Écartée** : un `switch` monolithique dans `handleSubmit` — même effet, mais seule l'interface le testerait.
- **Réversible** : oui.

## C-15-8a-5 — 15-8a (dev) : la fiche passe au formulaire la liste COMPLÈTE des comptes

- **Contexte** : D8 — une ligne pré-remplie sur un compte archivé ou non imputable doit s'afficher (numéro et nom) avec
  un avertissement, sans redevenir sélectionnable.
- **Retenu** : la fiche, qui charge déjà `fetchAccounts(true)`, passe cette liste au formulaire :
  `AccountAutocomplete` résout le libellé sur la liste complète et ne **propose** que les comptes actifs et imputables
  (comportement existant, 16-1b D11) ; l'avertissement `line-account-unusable` suit `isAccountUnusable` (16-1b), la
  source unique du verdict.
- **Écartée** : une seconde requête « comptes actifs » au clic sur « Modifier » — le libellé d'un compte archivé ne se
  résoudrait plus.
- **Réversible** : oui.

## C-15-8a-6 — 15-8a (dev) : le garde-fou d'inventaire porte ses mutations EN PERMANENCE

- **Contexte** : AC 13 demande que les mutations du garde-fou (colonne ou clé factice vers `journal_entries`, clé
  factice vers `journal_entry_lines`) soient tuées « une fois, déclarées au Dev Agent Record ».
- **Retenu** : un test permanent, `the_inventory_guard_turns_red_on_each_mutation`, qui pose chacune des quatre
  mutations (clé vers `journal_entries`, clé vers `journal_entry_lines`, colonne neuve sur les lignes, colonne au nom
  d'écriture sans clé), vérifie que le garde-fou rougit, la retire, puis vérifie qu'il redevient vert.
- **Écartée** : la mutation manuelle unique — sa preuve ne survit pas à la session.
- **Réversible** : oui.

## C-15-8a-7 — 15-8a (dev) : le montage des pièces devient un helper partagé, avec le solde `write_off`

- **Contexte** : AC 6 / finding R2-11 — réutiliser le montage de `every_document_owned_entry_is_refused` et y ajouter
  un solde.
- **Retenu** : `monter_les_pieces` (sept chemins, dont le `write_off`) rend pour chaque pièce l'écriture, le code et
  l'identifiant de la pièce ; consommé par le test de contre-passation (doc passé à « sept chemins »), par celui de la
  modification (`details.documentId`, I3), par la précédence et par la table de correspondance (AC 12). Le paiement
  détaché, propre à la modification, a son montage (`detacher_un_paiement`, trace posée par l'écrivain réel du journal
  d'audit) ; son chemin réel est tenu côté dépôt (C-15-8a-2).
- **Réversible** : oui.

## C-15-8a-8 — 15-8a (dev) : le motif de modification n'est affiché que s'il diffère de celui de la contre-passation

- **Contexte** : D8 — « si le motif de contre-passation et celui de modification sont le même code, ne l'afficher
  qu'une fois ».
- **Retenu** : la fiche affiche `modification-blocked-reason` sauf quand la contre-passation est elle aussi refusée
  sous le **même** code (cas des pièces et de la contre-passée) ; testé par Playwright (« écriture de facture »).
- **Réversible** : oui.

## C-15-8a-9 — 15-8a (revue P1) : le rejeu du `PUT` prouvé en forçant le `PUT` à perdre l'interblocage

- **Contexte** : finding B-5 de la revue de code P1 — le test de cycle de `kesh-db` accepte que la victime soit
  l'autre transaction, si bien que `retry_with` n'était exercé par aucun test.
- **Retenu** : `the_put_replays_a_deadlock_it_lost` (`journal_entry_reversal_e2e.rs`) monte le cycle projet ↔ exercice
  **à travers HTTP**, en rendant la transaction concurrente B **plus lourde** (500 lignes d'undo dans `audit_log`, table
  que le `PUT` ne verrouille pas) : InnoDB sacrifie la plus légère, donc le `PUT`. La preuve que la 1213 a eu lieu est
  structurelle : B obtient le projet en **exclusif** alors que le `PUT` le tenait et attendait l'exercice que B tient
  encore — seule l'annulation du `PUT` le permet. Le `PUT` doit ensuite rendre 200 et une seule trace. Mutation
  « `retry_with` à une seule tentative » → 500 `INTERNAL_ERROR`, rouge ; restauré, `touch`, trois runs verts.
- **Écartées** : injecter une fausse 1213 (ne prouve pas que l'erreur réelle est reconnue) ; lire le compteur global
  `Innodb_deadlocks` (pollué par les gates parallèles d'autres agents).
- **Réversible** : oui. ⚠️ Le test repose sur la règle de choix de la victime d'InnoDB (poids = undo + verrous) ; si une
  version de MariaDB la changeait, il rougirait à l'`expect` de B, avec un message qui le dit.

## C-15-8a-10 — 15-8a (revue P1) : conflits du rebase sur la 15-5b, résolus par fusion des deux intentions

- **Contexte** : rebase sur `origin/main` (`12e75d23`, 15-5b mergée). Conflits : registre et `sprint-status.yaml`
  (union) ; `docs/api-external.md` (ligne `ACCOUNT_NOT_POSTABLE`) ; `user-manual.tex` (encadré `keshnote` du
  *postable*) ; les deux PDF. Le code (`kesh-db/errors.rs`, `accounts.rs`, `journal_entries.rs`) a fusionné sans conflit.
- **Retenu** : `api-external.md` — la ligne de la 15-5b (liste complète des routes, rapprochement et comptes bancaires
  compris) **plus** la mention `PUT /journal-entries/{id}` de la 15-8a ; les deux lignes neuves
  (`LATER_FISCAL_YEAR_CLOSED`, `DETACHED_SUPPLIER_SETTLEMENT`) conservées. `user-manual.tex` — le texte de la 15-5b
  (quatre cas non contrôlés), dont la dernière phrase « ne se modifie plus du tout » (écrite sous le gel) est remplacée
  par celle de la 15-8a, au vocabulaire de la 15-5b (« non imputable »). PDF régénérés depuis les `.tex` fusionnés,
  zéro « ?? ».
- **Écartée** : prendre un côté entier — perdait soit la couverture de la 15-5b, soit la levée du gel.
- **Réversible** : oui.

## C-15-8a-11 — 15-8a (revue P1) : huit LOW acceptés sans correction

- **Contexte** : revue de code P1 (Sonnet, trois lentilles) : 0 CRITICAL/HIGH, un MEDIUM (E1) reclassé LOW par
  l'orchestrateur — suivi par #543 —, 14 LOW. Remédiation bornée à ce qui ne change pas le comportement de production.
- **Retenu** : corrigés E1 (documentation), E3, B-5, A-1 à A-4. **Acceptés** : B-1 (doublon de l'inventaire au `GET`
  — optimisation qui touche la production) ; B-2 (`project_id` d'en-tête inatteignable aujourd'hui — l'aligner touche
  `update_in_tx`) ; B-3 (message générique d'un projet archivé, hérité du `POST`) ; B-4 (perte de saisie sur conflit de
  version — choix D8 « pas de modale ») ; E2 (`PERIOD_LOCKED` sur l'ancienne date, message et classement — changerait un
  message et le classement d'écran) ; E4 (borne de verrou en ISO brut, préexistant côté serveur) ; E5 (422 de
  l'extracteur, commun à toutes les routes) ; E6 (coût du `GET` et faux négatif résiduel, déjà déclarés).
- **Réversible** : oui — chacun peut faire l'objet d'une issue ; B-2 et E2 sont les deux à reprendre en premier.

## C-15-5c-1 — 15-5c (dev) : la réutilisation de C37 s'étend aux clés `reconciliation-*` existantes

- **Contexte** : C37 dit de lire une clé `error-*` existante quand elle convient mot pour mot et sans
  variable. Trois codes de `failed[]` ont déjà un message exact dans l'espace `reconciliation-`, propre
  à la fonctionnalité (donc permis par `lint-i18n-ownership`) : `RECONCILIATION_ALREADY_RECONCILED`
  (`reconciliation-errors-already-reconciled`), `RECONCILIATION_INVOICE_NOT_ELIGIBLE`
  (`reconciliation-errors-invoice-not-eligible`), `RECONCILIATION_SPLIT_IMBALANCE`
  (`reconciliation-split-error-imbalance`). Par ailleurs `FISCAL_YEAR_INVALID` et
  `RECONCILIATION_FISCAL_YEAR_CLOSED` naissent du même constat (`find_open_covering_date` ne trouve aucun
  exercice ouvert) : `error-fiscal-year-invalid` (« Aucun exercice ouvert ne couvre cette date. ») est
  exact pour les deux, alors que `error-fiscal-year-closed-generic` affirmerait une clôture qui peut être
  une absence.
- **Retenu** : lire ces clés existantes (8 codes sur 26 lisent une clé existante, 7 clés distinctes :
  `error-rounding-account-not-configured`, `error-fiscal-year-invalid` ×2, `error-internal`,
  `error-validation`, et les trois `reconciliation-*`) ; 20 clés neuves `reconciliation-failed-*` pour les 18 autres codes
  (dont la variante sans numéros d'`ACCOUNT_NOT_POSTABLE` et le repli `-unknown`, qui porte `{ $code }`,
  valeur que le client possède). `BANK_ACCOUNT_NOT_CONFIGURED` et `RECONCILIATION_RULE_NOT_FOUND` ont une
  clé neuve : les messages existants (`reconciliation-manual-bank-account-not-configured`, qui cite le
  chemin brut `/bank-accounts` ; `reconciliation-rules-error-not-found`, « Règle introuvable. », alors que
  le code couvre aussi une règle désactivée) ne conviennent pas mot pour mot.
- **Écarté** : dupliquer les traductions existantes (deux copies divergent) ; une clé par code même
  quand le constat est identique.
- **Réversible** : oui (une clé par `case`).

## C-15-5c-2 — 15-5c (dev) : le manuel décrit l'écran, et nomme les trois défauts cités sans les promettre corrigés

- **Contexte** : AC5–AC9. Le manuel décrivait des fonctions inexistantes ; la story ne corrige ni #526,
  ni #527, ni #529.
- **Retenu** : sous-section fusionnée *Accepter ou rejeter les propositions* ; *Rejeter* dit que la
  transaction quitte la liste, reste visible sans action dans le détail de son import, et qu'aucun geste
  ne la rapproche ensuite (conseil : ne rejeter que ce qu'on ne comptabilisera pas par ce chemin) ; la FAQ
  est réécrite en deux temps (pourquoi rien n'est proposé ; que faire selon la nature de la transaction),
  avec l'interdiction explicite d'une règle pour un paiement de client ; la règle dont le compte devient
  non imputable « laisse la place à la règle suivante qui correspond » — fait relevé au code
  (`first_matching_rule` filtre sur l'ensemble des comptes imputables, `reconciliation.rs:586-591`), que la
  fiche ne disait pas. `user-manual.tex:1861` (« rapprochement manuel », section des projets) est aligné
  sur « affectation manuelle ». Seul `user-manual.pdf` est régénéré (`make user`) : l'administrateur et la
  brochure ne changent pas.
- **Écarté** : citer l'annulation d'un rapprochement comme chemin de retour d'une transaction rejetée
  (elle ne s'y applique pas) ; citer le seuil de 0,5 (API seulement).
- **Réversible** : oui (texte).

## C-15-5c-3 — 15-5c (revue P1, E1) : un libellé dédié au « paiement antérieur à la facture », le moteur inchangé

- **Contexte** : finding E1 (MEDIUM) de la revue de code P1. La proposition retient les factures datées
  de 30 jours avant à 30 jours après la transaction (`find_unpaid_invoices_for_window`, sur la date de
  comptabilisation), alors que l'acceptation refuse un paiement antérieur de plus d'un jour à la facture
  (`accept_one_invoice`, sur la date de valeur à défaut de comptabilisation) : une facture postérieure au
  paiement peut être proposée puis refusée. L'alignement du moteur est l'issue #548 (P3), hors story.
- **Retenu** : documenter et expliquer. `details.reason` porte la chaîne stable
  `payment_date_before_invoice_date`, posée par un seul site (`reconciliation.rs`) : le cas se distingue
  sûrement. D'où une clé neuve `reconciliation-failed-payment-before-invoice` (4 locales + repli), lue par
  `failedProposalLabel` derrière une garde de type (`failureReason`) ; les cinq autres raisons du code
  gardent le libellé générique. Le manuel (§ Algorithme de matching, paragraphe « Une facture proposée peut
  être refusée à l'acceptation », et la FAQ « aucun match automatique ») explique le refus et précise que
  le règlement depuis la fiche applique la même borne (`invoice_settlements_write.rs:96`).
- **Écarté** : modifier la fenêtre de proposition (#548) ; un libellé par raison (`race_during_update`,
  `invoice_already_paid`… : hors du finding, et le choix C37 d'un libellé par code reste la règle) ;
  laisser le libellé générique alors que la raison est lisible sans ambiguïté.
- **Réversible** : oui (un `if` et une clé).

## C-15-5c-4 — 15-5c (intégration) : la planification rejouée d'un bloc, les commits de la 15-5c un à un

- **Contexte** : rebase de la branche sur `origin/main` (`52a9b19b`, 15-8a mergée). La branche
  portait 18 commits de planification (`1920381e..98846b5e`) bâtis sur un main **antérieur** aux
  squashes de la 15-5a et de la 15-5b — squashes qui en contenaient déjà une partie —, puis le
  commit de fusion `f289414e` qui réintégrait main. Un `git rebase` les aurait rejoués un à un sur
  un registre et un `sprint-status.yaml` qui les portaient déjà en partie : conflits d'ajout à
  chaque commit, sans valeur.
- **Retenu** : la branche est reconstruite sur `origin/main` par `cherry-pick -m 2 f289414e` (le
  delta exact de la planification par rapport au main d'alors : story files 15-5d, 15-5e, 15-5e1,
  15-5e2, prompts, C42–C65), puis les sept commits de la 15-5c rejoués un à un. L'arbre obtenu
  diffère de l'ancien exactement par les 61 fichiers du delta de la 15-8a (contrôlé par
  `git diff --stat` des deux côtés). Ancienne tête conservée sous `backup/15-5c-avant-rebase`.
- **Résolutions** : registre et en-têtes de `sprint-status.yaml` par **union** (entrées
  C-15-8-* et C-15-8a-* de main, puis C-15-5c-*) ; CHANGELOG `[0.13.0]`, `.tex` et les quatre
  `messages.ftl` fusionnés sans conflit ; PDF pris côté main à chaque conflit puis **régénéré**
  (`make user`, 79 pages, contrôle sur texte aplati des deux paragraphes neufs).
- **Écarté** : rebase commit par commit des 18 commits de planification (conflits d'ajout répétés,
  aucun gain : la PR est fusionnée en squash) ; rebase des seuls commits 15-5c sur `f289414e`
  (aurait **perdu** la planification 15-5d/15-5e, absente de main).
- **Réversible** : oui (`backup/15-5c-avant-rebase`).

## C-15-5c-5 — 15-5c (intégration) : compteurs i18n recomptés sur l'état rebasé

- **Contexte** : la 15-8a et la 15-5c ajoutaient chacune des sites `i18nMsg(` et une déclaration
  candidate de l'angle « libellés en dur ».
- **Retenu** : `sitesTotal` **1868 → 1876** (15-8a) **→ 1904** (15-5c : +27 au dev, +1 en revue
  P1) ; `CANDIDATES_ATTENDUES` **46 → 48** (`reversalBlockerLabel`/`modificationBlockerLabel` de la
  15-8a déplacent `blockedLabel` et ajoutent un `ecartee`, `failedProposalLabel` de la 15-5c un
  `conforme`) : `ecartee: 7, conforme: 41`. Les commentaires des deux stories sont conservés, la
  15-5c datée « avant le rebase ». Vérifié par les tests eux-mêmes (`vitest run src/lib/shared/`,
  194/194), pas par addition seule.

## C-15-8b-1 — 15-8b (clôture de la validation) : `journal-entries.api.ts` tranché DEDANS, l'AC 10 refermée sur `HEAD`

- **Contexte** : la P4 ciblée laissait un MEDIUM (M1) — la liste « fermée » de l'AC 10 portait `journal-entries.api.ts:58`
  « effacé par l'inversion de la 15-8a — à vérifier ». Vérifié sur `52a9b19b` : le site **existe** (`:70`), dans le
  doc-comment de `deleteJournalEntry` que la 15-8a a rétabli en annonçant la 15-8b.
- **Retenu** : le site est **dans** la liste des disparitions ; T4 réécrit le doc-comment quand la fonction gagne son
  appelant. L'inventaire complet a été refait sur `HEAD` (38 lignes hors `_bmad-output/`) : deux sites neufs de la
  15-8a — `CHANGELOG.md:15` (dans `[0.13.0]`, réécrit) et `docs/api-external.md:221` (fait historique, **reste**) — et un
  sortant (`journal-entries.spec.ts:285`, reformulé par la 15-8a). Numéros de ligne remesurés dans une table de
  décalages en tête des Dev Notes plutôt que réécrits un à un dans la prose (la table fait foi).
- **Écartées** : le mettre dans les résidus « qui restent » (la phrase devient fausse au merge de cette story) ;
  réécrire chaque numéro dans la prose (plus de cent sites, risque d'erreur supérieur au bénéfice).
- **Réversible** : oui.

## C66 — 15-5e1 : la saisie fournisseur rejouée dès la 15-5e1, avec l'avance de ses réglages

- **Contexte** : finding **F4-1 MEDIUM** de la validation P4 de la 15-5e1. L'AC5 avance l'appel des réglages (`get_or_create_default_in_tx` : `INSERT IGNORE` puis `SELECT … FOR UPDATE`) avant les comptes de charge de `supplier_invoices::create_in_tx`. Deux saisies concurrentes d'une même société, qui s'attendaient jusqu'ici sur l'exercice, se disputent désormais d'abord la ligne des réglages. Si un `INSERT IGNORE` en doublon y pose un verrou partagé (hypothèse R3-7), elles s'interbloquent — alors que le rejeu de `POST /supplier-invoices` n'arrivait qu'à la 15-5e2 : une régression possible entre les deux merges. Décision de l'orchestrateur.
- **Retenu** : la **15-5e1 rejoue aussi `POST /supplier-invoices`** (enveloppe `DbError`, `"supplier_invoices::create"`, `supplier_invoices::create` ouvrant et concluant sa transaction), avec un test « route victime » sur une vraie 1213 comme les trois autres (test 5 : la transaction de test tient l'exercice, demande le compte de charge). L'avance des réglages et sa défense arrivent ensemble. La 15-5e2 perd cette route : partitions **8 `Rejouee` / 13 `ARejouer` / 4 `Exemptee` / 90 `SansEcritureAuJournal` = 115** à la 15-5e1, **17** routes (13 à rejouer, 3 migrées, `post_accept`) à la 15-5e2, **21 / 4 / 90** au final. L'hypothèse **R3-7 n'est pas tranchée** (ni mesurée, ni contournée par un `SELECT … FOR UPDATE` préalable) : le rejeu la rend sans conséquence pour l'utilisateur. **Révise C55** : « la ligne des réglages sérialise les saisies » et « déjà sérialisé par la ligne des réglages » sont faux en l'état des connaissances — elle les **ordonne** sans les sérialiser à coup sûr. **Révise C65** : la 15-5d s'appuie sur le rejeu de la validation **et de la saisie fournisseur** posé par la 15-5e1 ; seule la complétion d'import reste à la 15-5e2 (fenêtre assumée : geste interactif, rarement concurrent).
- **Écartées** : (a) une sonde T0 conditionnant le déplacement à la mesure de R3-7 — déplace la décision au développement et laisse la régression possible si la sonde est mal lue ; (b) réécrire `get_or_create_default_in_tx` sur le patron `SELECT … FOR UPDATE` d'abord (`invoice_number_sequences::next_number_for`) — un module de plus dans une story déjà au-delà du seuil de découpage, pour une hypothèse que le rejeu neutralise ; (c) avancer aussi la complétion d'import — elle exige l'extraction d'une fonction « une tentative » à l'enveloppe `AppError`, c'est du rollout (motif de C65 (b), maintenu pour elle seule).
- **Réversible** : oui (fiches seulement, code non écrit).

## C67 — 15-5e1 : dérogation de découpage écrite sur le décompte réel ; signal D5 déclaré

- **Contexte** : finding **F4-2 MEDIUM** de la validation P4 de la 15-5e1 : la fiche se déclarait « sous le seuil de la règle de découpage pour le code qui se conçoit », critère que la § *Règle de splitting préventif* ne connaît pas. Décompte selon la méthode de la règle (modules distincts au grain `kesh-api/routes/invoices`, tests et CHANGELOG exclus, sites forcés compris) : **10 modules de production** dans 2 crates (patron 2, sites pilotes 2 dont la saisie fournisseur de C66, sites forcés par la signature 3, ordre des verrous attendu par la 15-5d 3) — 11 avec la déclaration `pub mod retry;` de `lib.rs`.
- **Retenu** : une section « Dérogation règle de splitting » dans la 15-5e1, qui écrit le décompte, la justification — la 15-5e1 **est** la story-zéro du patron « story-zéro + rollout » ; un découpage de plus séparerait le registre et les enveloppes de leurs premiers utilisateurs, ou livrerait l'avance des réglages sans sa défense (C66) ; les sites forcés ne se découpent pas — et le risque accepté (revue adversariale sur dix modules, atténuée : cinq ne changent qu'un commentaire ou un argument). **Ce n'est pas une exception de la règle** mais un arbitrage, déclaré comme tel. **Signal D5 déclaré au Project Lead** (amendement D5) au Change Log de la fiche.
- **Écartées** : (a) un troisième découpage (patron seul / sites pilotes) — un patron livré sans site qui l'exerce ; (b) déplacer les trois dépôts de l'AC5 dans la 15-5d — rouvre C61 (c) et C65 pour deux commentaires et un appel.
- **Réversible** : oui.

## C68 — 15-5e2 : le Pattern 5 renvoie aux doc-comments canoniques au lieu de recopier l'ordre

- **Contexte** : findings **F4-3 MEDIUM = R4-1 MEDIUM** de la validation P4 de la 15-5e2 : la table « Where This Applies » du Pattern 5 recopiait l'ordre des verrous de `POST /supplier-invoices` et de `validate_invoice`, deux flux où la 15-5d ajoute ses comptes désignés ; la 15-5d ne touche pas au Pattern 5. Dans un sens de merge la ligne omet un verrou déjà en place, dans l'autre elle devient fausse au merge suivant — l'« ordre libre » de C65 n'était pas sûr pour la documentation. Décision de l'orchestrateur.
- **Retenu** : les lignes du Pattern 5 pour ces deux flux **renvoient au doc-comment canonique** de la fonction (`validate_invoice`, `supplier_invoices::create_in_tx`), **seul lieu de vérité**. Vérifié : l'AC9 de la 15-5d met déjà ces deux doc-comments à jour quand elle y place ses verrous — elle le dit désormais, et précise qu'elle n'a pas à toucher le Pattern 5 (Change Log de la 15-5d). **Révise C65** : l'ordre de merge 15-5d / 15-5e2 est sûr dans les deux sens, code et documentation. La nouvelle ligne du lot de rapprochement, que la 15-5d ne modifie pas, dit l'ordre du lot (par proposition et entre propositions, cycle de #536).
- **Écartées** : (a) écrire les lignes depuis le code relevé en T0 et donner à la 15-5d une tâche conditionnelle « si la 15-5e2 est mergée » — deux lieux à tenir, une coordination entre fiches ; (b) imposer un ordre de merge — contraire à C65.
- **Réversible** : oui.

## C69 — 15-5e1 / 15-5e2 : remédiations P4 de moindre portée

- **Contexte** : les findings LOW et les MEDIUM restants des quatre rapports de la validation P4 (`target/gate-logs/15-5e{1,2}-p4-{R,F}.md`), traités sur instruction de l'orchestrateur ; plus deux constats sur le registre (R4-6 de la 15-5e1).
- **Retenu** :
  - **Volet (c) du registre** (R4-1, R4-9, F4-4 de la 15-5e1) : fenêtre d'un handler de `pub async fn <nom>(` à la première ligne réduite à `}` en colonne 0 (rustfmt imposé par la CI) — plus d'équilibrage d'accolades, donc insensible aux `{`/`}` des littéraux ; retrait des commentaires qui saute les littéraux ; limite de mot avant le nom d'enveloppe ; handler `Rejouee` introuvable → échec. La fiche dit quelle preuve exerce quelle précaution : sur le source de la 15-5e1, aucune route `Rejouee` n'est suivie d'un item qui nomme une enveloppe, seules la mutation « nom en commentaire » (précaution 1) et le source synthétique (toutes) exercent les précautions ; le cas qui les motive naît à la 15-5e2.
  - **L'enveloppe `AppError` exercée dès la 15-5e1** (F4-5 de la 15-5e1) : la saisie fournisseur de C66 prend l'enveloppe `DbError`, pas `retry_app_on_deadlock` — F4-5 n'est donc **pas** fermé par C66 ; le test 1 gagne un volet « enveloppe » (une vraie 1213 puis `Ok` : deux appels ; erreur métier : un appel) et une mutation `max_attempts` à 1.
  - **CHANGELOG** : ligne « Modifié » à la 15-5e1 pour le passage de `debug!` à `warn!` (F4-11 de la 15-5e1) ; texte final complet de la ligne « Corrigé » écrit dans la 15-5e2 (R4-9 de la 15-5e2).
  - **Frontière des mentions de `retry_with`** : `opening_complement.rs:36` et `:405` (R4-10 de la 15-5e1 = R4-2, F4-6 de la 15-5e2), trou antérieur au découpage, **attribués à la 15-5e2**, qui migre `complete_opening_balances` — et ajout d'un contrôle `grep -rn "retry_with"` après migration.
  - **Documents faux entre les deux merges** (F4-8 de la 15-5e1) : écrits à l'AC6 de la 15-5e1 (exemple du Pattern 5, `api-external.md`, `user-manual.tex:909`), pour qu'une passe ne les prenne pas pour un oubli.
  - **Constat sur C57** (R4-6 de la 15-5e1) : son « Retenu : `closes #536` » a été révisé par C61 — `refs #536` à la 15-5e1, `closes #536` à la 15-5e2. Écrit ici plutôt que dans C57, que ce registre ne réécrit pas.
  - **Numéros de ligne** des deux fiches recalés sur `f289414e` (merge des 15-5a et 15-5b), `grep -nF` / `sed -n` à l'appui.
- **Écartées** : un test « route victime » sur `post_manual` dans la 15-5e2 (F4-8 de la 15-5e2, « idéalement ») — C56 exclut les tests dynamiques du rollout ; l'angle mort de la famille `AppError` est écrit (AC2 et Dev Notes de la 15-5e2) et les trois extractions reçoivent une lentille adversariale de la revue de code (F4-9).
- **Réversible** : oui.

## C70 — 15-5e1 / 15-5e2 : remédiation P5 — le volet (c) analysé par `syn`, les réglages de facturation rejoués

- **Contexte** : validation P5 (Sonnet ×2 par fiche ; `target/gate-logs/15-5e{1,2}-p5-{R,F}.md`). 15-5e1 : **R5-1 = F5-1 MEDIUM** — le volet (c) du registre, scanner textuel maison, ignore les durées de vie (`'_`, `&'static str`) et les littéraux octets (`b';'`), présents dans les fichiers visés : un `'` lu comme ouverture de caractère désynchronise le retrait des commentaires pour tout le reste du fichier (faux rouge ou faux vert). C'est le **deuxième trou successif** du même scanner (le premier, les accolades des littéraux, fermé en P4 par C69) : un **recyclage**, au sens de l'amendement D5. Plus onze LOW (15-5e1) et neuf LOW (15-5e2, qui converge). Décisions de l'orchestrateur, appliquées ici.
- **Retenu** :
  - **Volet (c) par l'analyseur du langage** : le test parse chaque fichier de routes avec **`syn` 2** (`syn::parse_file`), déjà au `Cargo.lock` (2.0.118, transitif) et ajouté aux `[dev-dependencies]` de `kesh-api` (features `full`, `visit`) ; il retrouve le `fn` du handler par son nom (`syn::visit`, `visit_item_fn`) et cherche dans son **corps** un `ExprCall` dont le dernier segment du chemin est `retry_on_deadlock`, `retry_on_deadlock_with`, `retry_app_on_deadlock` ou `retry_with`, ou un `ExprMethodCall` de ce nom. Commentaires, doc-comments (attributs de l'item suivant), littéraux, durées de vie et octets ne sont pas des appels : la classe de défauts est fermée par construction. Les quatre précautions textuelles et leur source synthétique sont **retirés**, remplacés par : (i) échec explicite si le handler est introuvable (fichier absent, aucun ou plusieurs `fn` de ce nom, fichier qui ne parse pas) ; (ii) un test du visiteur sur source synthétique (appel réel → trouvé ; nom en commentaire, chaîne, doc-comment du handler suivant, corps d'une autre fonction, nom préfixé → non trouvé ; `'_`, `&'static str`, `b';'`, `r#"…"#` → parse sans erreur) ; (iii) les mutations (retirer l'enveloppe d'une route `Rejouee` → rouge). Limite écrite : un appel dans une macro rend un faux rouge, jamais un faux vert. ⚠️ **Signal D5 déclaré au Project Lead** : sévérité égale (MEDIUM → MEDIUM) **et** recyclage ; la story n'est pas redécoupée parce que le recyclage tient à la **méthode** du volet (c), non au périmètre — il est fermé par un changement de méthode.
  - **`PUT /api/v1/company/invoice-settings` rejouée par la 15-5e1** (F5-6) : `update_invoice_settings` enveloppe `company_invoice_settings::update` (transaction propre, `DbError`, `"company_invoice_settings::update"`, `CompanyInvoiceSettingsUpdate` cloné par tentative). L'avance des réglages de la saisie fournisseur (C55, C66) l'expose : si R3-7 se vérifie, elle entre dans le cycle des saisies et, n'ayant rien modifié, en est la victime. **Tenue au registre** : elle reste **`SansEcritureAuJournal`**, comme `onboarding::finalize` (rejouée elle aussi sans écrire au journal) — la colonne dit l'inventaire de l'AC1, pas la présence d'une enveloppe ; elle est nommée au point (vi) du doc-comment du registre et prouvée par un **test 7** « route victime » (le test tient un `S` sur la ligne des réglages, la route attend l'`X` de son `UPDATE`, le test demande l'`X` : cycle documenté d'InnoDB, qu'il y ait ou non le `S` de R3-7), avec sa mutation. C'est l'option la plus simple : ni cinquième valeur, ni changement de partition (8 / 13 / 4 / 90 = 115, puis 21 / 4 / 90).
  - **R3-7** : **hypothèse forte** (un `INSERT IGNORE` en doublon pose un verrou partagé sur l'enregistrement existant, comportement documenté d'InnoDB, confirmé par la lentille F) ; **mesurée en T0** (deux connexions, une dizaine de lignes), résultat au Dev Agent Record. La réécriture éventuelle de `get_or_create_default_in_tx` (`SELECT … FOR UPDATE` d'abord) reste hors de la 15-5e1 (C66 (b)) ; si T0 confirme, l'orchestrateur ouvre une issue.
  - **Test 4 sur le cycle de #536** (F5-5) : validation avec écart d'arrondi, le test tient l'exercice et demande le compte d'arrondi ; la justification « le motif d'attente change » est retirée (fausse).
  - **Harnais E2E recopié** (F5-4) : le fichier neuf recopie `test_config`, `spawn_app`, `forge_jwt`, `create_company`, `create_user`, comme chacun des fichiers E2E de `kesh-api/tests` (`forge_jwt` y existe en 23 copies) — copie assumée, déclarée au Dev Agent Record ; la factorisation du harnais est une dette antérieure, à ouvrir en issue par l'orchestrateur.
  - **Révise C67** : onze modules de production (`kesh-api/routes/company_invoice_settings` en plus), 5 + 1 + 1 + 4 ; les sites forcés « suivent du choix de signature » (C62), ce n'est pas une impossibilité (F5-8). **Remplace** le volet (c) textuel de **C63** et **C69**.
  - **15-5e2** (convergée) : PDF et CHANGELOG en conflit au second merge avec la 15-5d — relocaliser par le texte, régénérer après rebase ; Pattern 5 sans numéros de ligne ; `complete_import` sans verrou nommé ; corps passés par référence ; `innodb_deadlock_detect` **vérifiée** `ON` sur `10.11.16-MariaDB` (base de dev) ; contrôle des `Overfull` ; « trois tentatives » à `api-external.md:307` ; ligne « Corrigé » étendue aux réglages de facturation, « #536 en partie » dans celle de la 15-5e1.
- **Écartées** : (a) compléter le scanner maison (durées de vie, octets, `r##"…"##`, commentaires imbriqués) — troisième rustine sur une méthode qui a déjà recyclé deux fois, et rien ne dit que la liste est close ; (b) une cinquième valeur de statut (`RejoueeHorsJournal`) pour les réglages — change la partition des deux fiches et le doc-comment pour une route, alors que `onboarding::finalize` fournit déjà le précédent ; (c) laisser les réglages non rejoués, l'angle mort (iv) écrit — l'exposition naît de cette story même ; (d) extraire le harnais dans `tests/common/mod.rs` pour le seul fichier neuf — une vingt-quatrième variante, sans migrer les vingt-trois autres.
- **Réversible** : oui (fiches seulement, code non écrit ; `syn` est une dépendance de test).

## C71 — 15-11 : une liste explicite sous `environment:`, les ajouts en clé sans valeur — pas d'`env_file`

- **Contexte** : #550 — les deux compose distribués ne transmettent au conteneur que les variables listées sous `environment:` ; 16 des 41 variables lues par le code n'y sont pas (SMTP, `KESH_PRODUCTION_RESET`, `KESH_LANG`…), et `.env` les reçoit sans effet ni message. Deux voies : `env_file: .env`, ou compléter la liste.
- **Retenu** : la **liste explicite**, complétée ; les **16 ajouts en clé sans valeur** (`KESH_SMTP_HOST:`), mesuré par `docker compose config` et `run … env` (Compose 2.40.3) : la clé reprend la valeur de `.env` si elle y est, et **n'existe pas** dans le conteneur sinon — le défaut du code s'applique sans être recopié, et aucune chaîne vide n'est transmise (`${X:-}` en transmet une, que `KESH_SMTP_PORT` et `KESH_ADMIN_BACKUP_DIR` ne traitent pas comme une absence). Les entrées existantes gardent leur forme. `env_file` est **interdit** sur `kesh-api` par le test.
- **Écartées** : (a) `env_file: .env` — transmettrait `MARIADB_ROOT_PASSWORD` et toute variable étrangère à l'application, laisserait une ligne `KESH_STATIC_DIR=frontend/build` décommentée par erreur écraser le chemin de l'image, et viderait de sens le test de transmission ; (b) ajouts en `${X:-défaut}` — recopie les défauts du code dans deux fichiers (DRY), à tenir alignés à chaque changement ; (c) ajouts en `${X:-}` — chaîne vide transmise.
- **Réversible** : oui (fichiers compose ; une installation existante doit de toute façon re-télécharger son compose, écrit au CHANGELOG et au manuel).

## C72 — 15-11 : le test garde-fou est un test Rust de `kesh-api`, `syn` pour le code, un vrai analyseur YAML pour les compose

- **Contexte** : #550 demande un test qui empêche l'écart de se reformer. Il doit tourner dans les gates existants et inventorier les lectures **sans** liste tenue à la main (règle « inventorier les sites non résolus »).
- **Retenu** : `crates/kesh-api/tests/configuration_transmise.rs`, sans base, exécuté par `cargo test --workspace` (CI) et `scripts/test-fast.sh`. Lectures recalculées par **`syn` 2** sur `crates/*/src` (hors `#[cfg(test)]`) : littéraux lus directement ; sites non littéraux résolus par une liste fermée d'indirections (`opt_trimmed_env`, `parse_strict_bool`, `env_flag_enabled`, `EnvFilter::DEFAULT_ENV` → `RUST_LOG`), dont toute entrée périmée rougit ; une macro contenant `env :: var` rougit (faux rouge possible, faux vert jamais). Compose lus par **`yaml-rust2`** (dépendance de test ; aucun analyseur YAML au `Cargo.lock`, `serde_yaml` archivé). Six familles d'assertions (lectures, transmission, valeurs, `.env.example`, fantômes, auto-test), seize mutations. Complété par `docker compose config -q` dans le job `docker-build` de la CI, pour la validité que seul Compose juge.
- **Écartées** : (a) script shell appelé par la CI — ne tourne pas dans le gate local, et un `grep` d'`env::var` est exactement la méthode textuelle dont la 15-5e1 a dû sortir (C70) ; (b) test appelant `docker compose config` — dépend du binaire Docker dans le job `backend` et dans nextest ; (c) analyseur YAML maison ligne à ligne — fragile aux deux formes de `environment:` (dictionnaire, liste).
- **Réversible** : oui (test et dépendances de test seulement).

## C73 — 15-11 : périmètre — ce que la story corrige au-delà de la liste de #550, et ce qu'elle laisse

- **Contexte** : l'inventaire a révélé, dans la même classe de défaut (« réglage documenté, sans effet ») : `KESH_COOKIE_SECURE` absente de `docker-compose.prod.yml` ; les trois `KESH_*_HOST_DIR` ignorées par `docker-compose.prod.yml` (chemins figés) ; `KESH_ADMIN_RESET`, variable **inexistante** donnée comme recours dans un message de refus de démarrage, un journal et `.env.example` ; et `docker-compose.yml` sans `image:`, si bien que l'installation du manuel (télécharger ce seul fichier) échoue sur le `Dockerfile` absent — vérifié.
- **Retenu** : les quatre sont traités dans la 15-11 — ce sont des lignes du fichier même que l'exploitant doit re-télécharger, ou des textes que le test « fantômes » refuserait. `image: gcorbaz/kesh:latest` **avec** `build:` conservé, prouvé par `docker compose --dry-run up -d` avant/après ; montages de P en `${KESH_*_HOST_DIR:-<chemin actuel>}` (aucun changement sans `.env`). Le bloc `KESH_PRODUCTION_RESET` de `.env.example` et les lignes `:691`/`:1314` du manuel restent à la 15-7b2.
- **Écartées (signalées à l'orchestrateur pour issue)** : (a) un volume pour `KESH_ADMIN_BACKUP_DIR` (défaut `/tmp`, perdu au redémarrage) — changement de comportement par défaut, à concevoir ; (b) MariaDB publiée sur `3306:3306` par `docker-compose.yml` — durcissement distinct ; (c) contraindre `docker-compose.dev.yml` — pile de développement non distribuée.
- **Réversible** : oui.

## C74 — 15-5e1 : remédiation P6 — un témoin du rejeu aux tests « route victime » ; le cycle du test 7 borné à la version épinglée

- **Contexte** : validation P6 de la 15-5e1 (Opus ×2 ; `target/gate-logs/15-5e1-p6-{R,F}.md`) : **F6-1 MEDIUM** — le test 7 (réglages de facturation, ajouté en P5, C70) repose sur un cycle « S tenu, X en attente, le détenteur du S demande le X » que C70 disait « documenté d'InnoDB, qu'il y ait ou non le S de R3-7 ». C'est l'**ancien** comportement : MySQL 8.0.18 (bogue #11745929) puis MariaDB 11.4.5 / 11.7.2 (MDEV-34877) accordent le X au détenteur du S sans attendre. Sur ces versions, si R3-7 est fausse, il n'y a pas de cycle : la route réussit au premier essai et le test reste vert (200, une version, un audit) sans qu'aucun rejeu ait eu lieu. La version épinglée (`mariadb:10.11` en CI, à la release et dans `docker-compose.yml` ; base de dev 10.11.16) forme bien le cycle, mais le manuel admin conseille une migration vers MariaDB 11.5+. Plus 10 LOW distincts (R6-1 à R6-7, F6-2 à F6-5, R6-1 = F6-2).
- **Retenu** (décisions de l'orchestrateur) :
  - **Un témoin du rejeu aux tests 2 à 5 et 7** : chaque test installe pour sa durée un abonné `tracing` de capture et exige au moins un événement `WARN` de cible `kesh_db::retry` portant l'`operation` attendue. Couche maison d'une trentaine de lignes sur `tracing-subscriber` (déjà dépendance normale de `kesh-api`, 0.3.23 : **aucune crate ajoutée**), dans un module de test partagé `crates/kesh-api/tests/common/capture_rejeu.rs` ; `tracing::subscriber::set_default` suffit, `#[sqlx::test]` tournant sur un runtime tokio à un seul fil (`sqlx-core-0.8.6/src/rt/mod.rs:119`) — un passage au multi-fil ferait rougir le témoin, jamais un faux vert. **Mutation** : aux tests 2 et 7, la transaction de test annule sans fermer le cycle → la requête aboutit sans rejeu → seul le témoin rougit.
  - **Le cycle du test 7 écrit pour ce qu'il est** : un cycle sur MariaDB 10.11 ; sur MySQL ≥ 8.0.18 et MariaDB ≥ 11.4.5, il dépend de R3-7. **T0 le forme à la main** sur la version épinglée avant que le test s'écrive ; un échec remonte à l'orchestrateur. Un rouge du témoin après une montée de version dira que le montage ne forme plus son cycle, non que le rejeu est cassé.
  - **Corrige C70** sur ce seul point (« qu'il y ait ou non le S de R3-7 ») ; C70 n'est pas réécrite. Le Change Log P5 de la fiche reçoit une ligne de correction.
  - **LOW** : AC3 dit que les lectures préalables lisent la ligne verrouillée, déjà hors transaction, rejugées par `WHERE version = ?` (R6-1 = F6-2) ; « tests 2 à 5 et 7, cinq routes » au doc-comment du registre et dans la 15-5e2 (R6-2) ; montage par `seed_accounting_company` (réglages, taux, exercice), `designate_rounding_account` de `kesh_db` vérifié après appel, ligne des réglages semée et `version` lue par `GET` au test 7 (R6-3) ; chemins `routes/onboarding.rs` (R6-4) ; « cinq routes » à l'AC2 et à T0 (R6-5) ; appels imbriqués au source synthétique du visiteur `syn` (R6-6) ; titres (R6-7) ; règlement partiel concurrent écrit dans « Pourquoi rejouer est sûr », sans garde neuve (F6-3) ; forme des entrées du CHANGELOG (F6-4) ; `disable_rounding_to_5_centimes` aux tests 2 et 3 (F6-5).
  - **Signal D5 déclaré au Project Lead** : MEDIUM → MEDIUM, et recyclage (F6-1 naît du test 7 ajouté par la remédiation P5). Traité par un renforcement des preuves (témoin, mutation, mesure en T0), sans découpage ni module de production de plus (onze, inchangé).
- **Écartées** : (a) mesurer l'attente de la requête `FOR UPDATE` du test (qu'elle n'aboutisse qu'après l'annulation de la route) — preuve indirecte et sensible au temps, là où l'événement du rejeu est la chose même ; (b) une crate de capture (`tracing-test`) — dépendance neuve pour ce qu'une couche de trente lignes fait ; (c) un abonné global installé une fois (`set_global_default`) — impose un filtrage par opération entre tests d'un même processus, inutile sur un runtime à un fil ; (d) remonter le test 7 sur un cycle à deux ressources — la route n'en tient qu'une, la ligne des réglages ; (e) épingler la version de MariaDB par un test — hors du périmètre ; le témoin suffit à rendre la dérive visible.
- **Réversible** : oui (fiches seulement, code non écrit).

## C-15-5e1-1 — 15-5e1 / T0 : R3-7 mesurée VRAIE sur MariaDB 10.11.16, cycle du test 7 formé à la main

- **Contexte** : T0 de la 15-5e1 (choix C70, C74) — mesurer l'hypothèse R3-7 et former à la main le cycle du test 7 avant de l'écrire, sur la base de dev (`10.11.16-MariaDB-ubu2204`, `innodb_deadlock_detect = 1`), base dédiée `kesh_155e1`.
- **Mesures** : (1) **R3-7** — connexion A : `START TRANSACTION; INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (1)` sur une ligne existante (`ROW_COUNT() = 0`), sans conclure ; connexion B (`innodb_lock_wait_timeout = 2`) : `SELECT … WHERE company_id = 1 FOR UPDATE` → **1205** ; `information_schema.INNODB_LOCKS` pendant l'attente : A tient un verrou **`S` RECORD** sur `PRIMARY` (ligne 1), B demande `X`. **R3-7 est vraie** sur la version épinglée. (2) **Cycle du test 7** — A alourdie (500 lignes de lest), `LOCK IN SHARE MODE` sur la ligne ; B `UPDATE company_invoice_settings … WHERE company_id = 1` attend ; A `SELECT … FOR UPDATE` → **B sort en 1213, A obtient son verrou**. Le montage de la fiche tient : aucune adaptation.
- **Conséquences** : la fiche (AC5) le prévoyait : la réécriture de `get_or_create_default_in_tx` (`SELECT … FOR UPDATE` d'abord, `INSERT IGNORE` seulement si la ligne manque) **reste hors de cette story** (C66 (b)) ; **l'orchestrateur ouvre une issue**. À noter : `company_invoice_settings::update` commence lui-même par un `INSERT IGNORE` ; sous R3-7, la route du test 7 tient donc un `S` avant son `UPDATE`, et le cycle du test se formerait aussi sur MariaDB ≥ 11.4.5 / MySQL ≥ 8.0.18 — par R3-7, pas par l'ancien comportement. Le témoin du rejeu reste la garde.
- **Réversible** : sans objet (mesure).

## C-15-5e1-2 — 15-5e1 : les doc-comments canoniques écrivent l'ordre sans numéros de ligne

- **Contexte** : l'AC5 donne le bloc de l'ordre de `validate_invoice` avec, en colonne de droite, les numéros de ligne relevés (`:1953`, `:2004-2006`, …).
- **Retenu** : le doc-comment reprend l'ordre, les étapes et leurs conditions **sans** les numéros de ligne ; idem pour celui de `supplier_invoices::create_in_tx`. Les étapes du code portent déjà les mêmes numéros (`// (1)`, `// (2 bis')`, …), ce qui suffit à la 15-5d pour y renvoyer.
- **Écartée** : recopier les numéros — le doc-comment est au-dessus du code qu'il décrit, sa propre longueur les décale dès l'écriture, et rien ne les recontrôle : une référence fausse au premier commit.
- **Réversible** : oui.

## C-15-5e1-3 — 15-5e1 : technique des mutations de l'AC4

- **Contexte** : les mutations « retirer l'enveloppe » doivent faire rougir le test « route victime » **et** le volet (c) ; celle du témoin doit faire rougir le seul témoin.
- **Retenu** : (a) l'enveloppe est remplacée par une fonction `sans_rejeu(op, f)` ajoutée au fichier muté, qui appelle la fermeture une fois — la route garde sa forme et compile, son corps ne nomme plus aucune enveloppe ; pour `post_cancel_reconciliation`, `retry_with` est remplacé de même par `sans_rejeu_w` ; (b) la mutation du témoin est posée dans l'aide commune `victime` (la demande qui ferme le cycle est retirée) : elle couvre d'un coup les tests 2, 3, 4, 5 et 7, au-delà des 2 et 7 que la fiche exige ; (c) la variante « témoin muté **et** enveloppe retirée » n'a pas été lancée séparément (la fiche la donne pour équivalente). Chaque mutation : copie de sauvegarde, application, `binary(rejeu_interblocage_e2e) | binary(audit_route_registry)`, restauration, `touch`.
- **Écartée** : supprimer l'appel à la main (fermeture déballée) — plus long, et sujet à erreur de reconstitution ; le shim produit le même comportement observable.
- **Réversible** : oui (rien n'est versionné du script de mutation).

## C-15-5e1-4 — 15-5e1 / revue de code P1 : formes retenues pour la remédiation

- **Contexte** : revue de code P1 de la 15-5e1 (Sonnet ×3, `target/gate-logs/15-5e1-review-p1-{B,E,A}.md`) : B-1, B-2 = E-1 MEDIUM ; E-3, A1, A2 et LOW. Les décisions de fond sont de l'orchestrateur ; ce qui suit consigne les formes.
- **Retenu** :
  - **B-1** : la connexion qui pose `SET SESSION innodb_lock_wait_timeout = 1` (test 1, volet (b)) est **détachée du pool et fermée** (`detach().close()`) après son `ROLLBACK`, plutôt que remise à `DEFAULT` : la remise dépend d'une requête de plus qui peut elle-même échouer, la fermeture ne rend rien au pool quoi qu'il arrive. Aucun autre `SET SESSION` dans le fichier ni dans le module de capture (`grep -rln "SET SESSION" crates/` : ce test et `kesh-db/src/pool.rs`, production, hors sujet).
  - **E-3** : le rejeu épuisé est journalisé en **`error!`** (même cible `kesh_db::retry`, champs `operation` et `attempts`), non en `warn!` : le témoin du rejeu compte les `WARN` de cette cible comme des rejeux, et un `warn!` d'épuisement après une seule tentative (`max_attempts = 1`) y passerait pour un rejeu — un faux vert possible. Seul `kesh_db::retry::retry_with` a une boucle ; l'enveloppe `kesh_api::retry` la réutilise, donc une seule ligne couvre les deux. Test : volet (f) du test 1 (trois vraies 1213, deux `warn!` puis un `error!` nommé), le témoin gagnant une liste `epuisements` ; mutation « `error!` retiré » → (f) rouge.
  - **A1** : le doc-comment de `supplier_invoices::create_in_tx` suit les étiquettes du code ; la passe des comptes, qui fait partie de l'étape `(2)` du code mais vient après `(2 bis)`, est écrite **`(2, suite)`** au doc-comment **et** au code (commentaire posé en tête de la boucle) — plutôt que de renuméroter le code, dont les étiquettes sont citées ailleurs. `(4)` ajoutée.
  - **B-2 = E-1** : seuls le paragraphe « Why Lock Ordering Matters » et l'exemple « How to use the retry helper » du Pattern 5 sont corrigés ici ; le reste du Pattern 5 (tableau, « Used on `finalize` », « Global Lock Order ») reste à la 15-5e2.
- **Écartées** : (a) `SET SESSION … = DEFAULT` pour B-1 (voir ci-dessus) ; (b) `warn!` d'épuisement ; (c) renuméroter `(2)` du code en `(2)` / `(2 ter)` — change les étiquettes de l'étape que la 15-5d et les Dev Notes citent.
- **Réversible** : oui.

## C-15-5e1-5 — 15-5e1 (intégration) : reconstruite sur `origin/main` (`ef39dd54`) ; le `PUT` de la 15-8a nommé et classé `Rejouee`

- **Contexte** : `origin/main` porte la 15-8a (`52a9b19b`) et la 15-5c (`ef39dd54`, dont le squash contenait déjà le delta de planification de `f289414e`, cf. C-15-5c-4). La branche portait les 18 commits de planification antérieurs, la fusion `f289414e`, puis neuf commits de planification (`597e4126..778513aa` : validations P4 à P7 des 15-5e1/15-5e2, spécification 15-11) absents de main, et six commits propres à la 15-5e1. La 15-8a a ajouté un **`retry_with` sur le `PUT /journal-entries/{id}`** (C-15-8-19), alors que la 15-5e1 change la signature de `retry_with` (nom d'opération en premier argument, C62).
- **Retenu** : branche recréée sur `origin/main` ; delta `f289414e..778513aa` rejoué **d'un bloc** (registre et en-têtes de `sprint-status.yaml` par union) ; les six commits de la 15-5e1 rejoués un à un. Ancienne tête conservée sous `backup/15-5e1-avant-integration`. Résolutions : CHANGELOG `[0.13.0]` — les deux entrées « Corrigé » de la 15-5c et celle de la 15-5e1 gardées ; `sprint-status.yaml` — union ; **registre des routes** — version à deux colonnes de la 15-5e1, où le `PUT` passe de `NoMatter … SansEcritureAuJournal` à **`Traced, Rejouee`** (il écrit au journal depuis la 15-8a, et son handler appelle `retry_with`) ; partition d'audit reprise de la 15-8a (95 tracées, 2 `NoMatter`) ; partition de rejeu **9 / 13 / 4 / 89 = 115** (au lieu de 8 / 13 / 4 / 90) ; volet (c) : **9** routes examinées ; limite (iii bis) : `the_put_replays_a_deadlock_it_lost` cité comme preuve dynamique du `PUT`. Le site reçoit le nom **`"journal_entries::update"`** (forme `module::opération` des autres sites). Pattern 5 (`docs/MULTI-TENANT-SCOPING-PATTERNS.md`, ligne du `PUT` écrite par la 15-8a) aligné sur la signature nommée.
- **Écartées** : classer le `PUT` `ARejouer("15-5e2")` (faux : il est déjà rejoué, le volet (c) le confirme) ; rebase commit par commit (conflits d'ajout répétés sans valeur, précédent C-15-5c-4) ; ne pas rejouer la planification `597e4126..778513aa` (la fiche 15-5e1 et les prompts versionnés de ses validations P5–P7 manqueraient à main).
- **Conséquence pour la 15-5e2** (à reporter dans sa fiche par l'orchestrateur) : il existe désormais **six** sites `retry_with` (`onboarding::finalize`, `opening_balances::complete`, `invoices::write_off`, `reconciliation::accept` et `reconciliation::cancel`, et le `PUT` des écritures) ; sa cible de registre devient **22 `Rejouee` / 4 / 89** et non 21 / 4 / 90, et le `PUT` est un site `retry_with` de plus à migrer vers une enveloppe (ou à justifier).
- **Réversible** : oui (`backup/15-5e1-avant-integration`).
## C-15-8b-2 — 15-8b (dev) : cible cargo DÉDIÉE au worktree, la cible partagée mélange les worktrees

- **Contexte** : la consigne 8 prescrit `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh/target` (cible partagée). Sur ce
  worktree, `cargo build -p kesh-api` a compilé `kesh-api` contre un `kesh-db` **d'une autre branche** (erreurs
  « no `ModificationGuard` », signatures de `retry_with` différentes) sans recompiler `kesh-db` : cargo hache les
  dépendances de chemin **relativement à la racine du workspace**, si bien que deux worktrees produisent les mêmes
  artefacts et se les volent, le fingerprint pointant les sources de l'autre arbre. Les deux premiers `cargo build`
  « verts » de cette story ne prouvaient donc rien.
- **Retenu** : `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh/target-158` (le nom que le prompt prévoyait), compilation à
  froid (3 min 19). Tous les gates déclarés ici ont tourné sur cette cible.
- **Écartée** : continuer sur la cible partagée — résultats non attribuables à l'arbre testé.
- **Réversible** : oui. ⚠️ **À signaler à l'orchestrateur** : tout agent d'un autre worktree sur la cible partagée est
  exposé au même mélange — un gate vert peut y avoir testé le code d'une autre branche.

## C-15-8b-3 — 15-8b (dev) : tests d'ordre de `mod tests` montés dans UNE transaction annulée

- **Contexte** : AC 4-bis demande les paires de précédence aussi dans `mod tests`, qui travaillent sur la base
  **partagée** (`test_pool`). Clore un exercice, poser une borne ou lier une facture à une écriture y laisserait un
  résidu qui fait rougir le gate suivant (KF-039, cas b).
- **Retenu** : helper `supprimer_avec(Causes)` — l'écriture est créée normalement, puis **toutes** les causes (facture
  brouillon + contact, contre-passation par `reverse_in_tx`, exercice postérieur clos, exercice clos, borne) sont
  posées **dans la transaction** passée à `delete_in_tx`, le résultat lu, puis la transaction **annulée**. La « facture »
  est un brouillon inséré en SQL avec `journal_entry_id` (suffisant pour `reversal_blockers`). Sept tests d'ordre, plus
  `la_route_refuse_une_ecriture_manuelle_de_periode_verrouillee` (cas séparé, AC 4-bis) et
  `la_devalidation_ne_voit_pas_l_exercice_posterieur` (fige C-15-8-29 par écrit).
- **Écartée** : monter via l'API (déjà fait au niveau HTTP, `the_precedence_of_the_delete_refusals_is_fixed`) ; poser
  les causes par le pool puis nettoyer (un test qui rougit laisse le résidu).
- **Réversible** : oui.

## C-15-8b-4 — 15-8b (dev) : « Modifiée » visible à tous les rôles, « Historique » dès la création

- **Contexte** : D4 dit « Modifiée » quand `version > 1`, et « Supprimer » et « Historique » absents au rôle
  Consultation ; il ne dit ni si « Modifiée » l'est aussi, ni si le lien attend une modification.
- **Retenu** : la mention « Modifiée » est affichée **à tous les rôles** (c'est un fait sur l'écriture, pas un geste) ;
  le lien « Historique » est affiché **aux rôles Administrateur et Comptable, même avant toute modification** (la
  création est déjà au journal d'audit, et une suppression future d'une autre écriture ne s'y voit pas autrement).
- **Écartée** : lier le lien à `version > 1` — l'historique d'une écriture jamais modifiée existe (sa création).
- **Réversible** : oui, une condition dans `[id]/+page.svelte`.

## C-15-8b-5 — 15-8b (dev) : AC 7 testé dans `opening_balances_e2e.rs`, pas dans `journal_entry_reversal_e2e.rs`

- **Contexte** : l'AC 7 asserte le statut `READY`, la génération sous le numéro 2 et `completableAccounts` après
  suppression de l'ouverture. Les helpers de ces routes (`seed_ready`, `get_status`, `post_complete`) vivent dans
  `opening_balances_e2e.rs`.
- **Retenu** : deux tests neufs là (`deleting_the_only_opening_entry_reopens_the_generation_under_number_2`,
  `deleting_the_opening_among_other_entries_makes_its_unmoved_accounts_completable`, ce dernier asserte aussi que la
  banque mouvementée ailleurs n'est **pas** complétable — finding F10 — et que le complément se supprime en 204) ; le
  test de la 15-8a `the_opening_entry_is_modifiable_and_still_reversable` est **renommé**
  `the_opening_entry_is_modifiable_reversable_and_deletable` et sa moitié `DELETE` inversée (204). La « porte de la
  contre-passation » y est désormais vérifiée par `reversable = true` au détail (contre-passer puis supprimer est
  impossible : une écriture contre-passée ne se supprime pas).
- **Réversible** : oui.

## C-15-8b-6 — 15-8b (dev) : pas de test de rejeu d'interblocage propre au `DELETE`

- **Contexte** : D2 enveloppe le `DELETE` dans `retry_with` « par uniformité, aucun cycle connu ». La 15-8a a un test
  qui force le `PUT` à perdre un interblocage réel (`the_put_replays_a_deadlock_it_lost`), monté sur le cycle
  projet ↔ exercice — que le `DELETE` ne prend pas (aucun projet verrouillé).
- **Retenu** : l'AC 6 est tenue par le `grep -nF "retry_with"` (le `PUT` **et** le `DELETE`) et la ligne de Pattern 5 ;
  pas de test de rejeu : sans cycle connu, il faudrait fabriquer un interblocage artificiel, qui prouverait le
  montage plutôt que le handler.
- **Écartée** : un test à interblocage fabriqué (deux transactions croisées sur l'écriture et l'exercice) — coûteux,
  fragile, et la règle de choix de la victime d'InnoDB en déciderait.
- **Réversible** : oui — à ajouter si un cycle est un jour identifié.

## C-15-8b-7 — 15-8b (clôture de la revue P1) : E2E sur le port 3008, le 3001 étant pris par un autre projet

- **Contexte** : le prompt de clôture fixait le port E2E 3001. Au moment du gate, `127.0.0.1:3001` était tenu par
  `opengmao-server` (`/home/gcorbaz/devel/opengmao`), un autre projet de la station — pas un agent Kesh.
- **Retenu** : backend E2E sur **3008** (libre, vérifié par `ss -ltn`), `KESH_BACKEND_URL=http://127.0.0.1:3008` côté
  runner ; même binaire (`target-158`, copié dans le scratchpad), même base `kesh_e2e_158` remise à zéro, répertoires
  inbox/documents neufs (`/tmp/kesh-e2e-158b`).
- **Écartées** : arrêter le processus d'`opengmao` (hors périmètre, pas le nôtre) ; attendre qu'il libère le port
  (aucune échéance connue).
- **Réversible** : oui — le port n'est qu'un paramètre de montage, rien n'est versionné.

## C-15-8b-8 — 15-8b (revue P1, E-2) : le message générique d'exercice clos étendu à « supprimée », pas de clé dédiée au `DELETE`

- **Contexte** : `DbError::FiscalYearClosed` rend `error-fiscal-year-closed-generic` (« … ne peut y être ajoutée ou
  modifiée »), que le `DELETE` refusé sur exercice clos rend aussi.
- **Retenu** : reformuler la clé existante dans les quatre locales et son repli Rust (`errors.rs`) — « ajoutée, modifiée
  ou supprimée » / « hinzugefügt, geändert oder gelöscht » / « added, modified or deleted » / « aggiunta, modificata o
  eliminata ». Aucune clé neuve : `sitesTotal` et l'inventaire des sites inchangés. La variante datée
  (`error-fiscal-year-closed`, `{ $date }`) n'est pas touchée : elle est rendue par la saisie et la modification, pas par
  la suppression.
- **Écartée** : une clé dédiée au `DELETE` — il faudrait distinguer l'erreur au niveau de `DbError` ou de la route pour
  un gain nul (la phrase étendue reste vraie pour les trois gestes).
- **Réversible** : oui (texte seul).

## C-15-8b-9 — 15-8b (intégration) : rebasée sur `origin/main` (`de1e1c26`, 15-5e1) ; le `DELETE` nommé et classé `Rejouee`

- **Contexte** : `origin/main` porte la 15-5e1 (`de1e1c26`) : `retry_with` prend un nom d'opération en premier
  argument (C62), le registre des routes a une colonne de rejeu et un volet (c) qui vérifie par `syn` que chaque route
  `Rejouee` appelle une enveloppe, et Pattern 5 a été réécrit (signature nommée, ligne du `PUT`). La 15-5e1 avait
  classé `DELETE /journal-entries/{id}` **`ARejouer("15-5e2")`** — et non `SansEcritureAuJournal`, comme le prompt
  d'intégration le supposait : le `DELETE` existait déjà (il rendait `ENTRY_IS_POSTED`) et la remontée de l'AC1 de la
  15-5e1 l'avait compté parmi les routes qui écrivent au journal.
- **Retenu** : rebase des sept commits de la story ; conflits résolus par union (registre des choix — C66 à
  C-15-5e1-5 avant les C-15-8b —, en-têtes `last_updated` de `sprint-status.yaml`, ligne 15-11 conservée, une seule
  ligne 15-8b, YAML rechargé) ; CHANGELOG `[0.13.0]` fusionné sans conflit (les entrées des deux côtés présentes) ;
  aucun `.ftl`, `.tex`, PDF ni fichier frontend touché par la 15-5e1, donc ni recompte de `sitesTotal` /
  `CANDIDATES_ATTENDUES` ni régénération de PDF (vérifiés par les tests eux-mêmes). Le `retry_with` du `DELETE` reçoit
  **`"journal_entries::delete"`** (forme `module::opération`, comme `"journal_entries::update"`), au commit de
  développement même pour que chaque commit rebasé compile. Pattern 5 : la ligne du `PUT` de la 15-5e1 gardée, celle
  du `DELETE` alignée sur la signature nommée. Registre des routes : `DELETE` → **`Traced, Rejouee`** ; volet (c) :
  **10** routes examinées ; partition de rejeu **10 `Rejouee` / 12 `ARejouer` / 4 `Exemptee` / 89
  `SansEcritureAuJournal` = 115** (et non 10 / 13 / 4 / 88 : le `DELETE` quitte `ARejouer`, pas
  `SansEcritureAuJournal`) ; limite (iii bis) : le `DELETE` relève de la revue fichier par fichier (pas de test de
  rejeu propre, C-15-8b-6). Les empreintes citées dans la fiche (`8cfb3759`) sont celles d'avant ce rebase ; le
  commit correspondant est désormais `a6e06547`.
- **Écartées** : garder `ARejouer("15-5e2")` (faux : le handler est déjà rejoué, le volet (c) le confirme) ;
  migrer le `DELETE` vers l'enveloppe `retry_on_deadlock` (c'est le rollout de la 15-5e2, qui migre aussi le `PUT`).
- **Conséquence pour la 15-5e2** (à reporter dans sa fiche par l'orchestrateur) : une route de moins à rejouer
  (**12** `ARejouer`), un site `retry_with` de plus à migrer (**sept** : les six de C-15-5e1-5 et le `DELETE`) ; sa
  cible finale de registre reste **22 `Rejouee` / 4 / 89** si toutes les `ARejouer` y passent.
- **Réversible** : oui.

## C75 — 15-11 : remédiation P1 — les ajouts en `${KESH_X:-}`, une fonction de lecture unique (vide = absent) ; révise C71

- **Contexte** : validation P1 de la 15-11 (Sonnet ×2 ; `target/gate-logs/15-11-p1-{R,F}.md`) : 13 MEDIUM bruts, 8 distincts, 13 LOW. Le cœur : la **clé sans valeur** retenue par C71 (a) ne protège pas du vide — une ligne `KESH_X=` vide de `.env` est transmise comme chaîne vide (mesuré, Compose 2.40.3), et `KESH_SMTP_PORT`, `KESH_ADMIN_BACKUP_DIR`, `KESH_LANG` ne la traitent pas comme une absence ; (b) n'a été mesurée que sur une version de Compose, alors que la cible (Synology Container Manager) en embarque une inconnue (R2/F6, R3/F2, F1). Et l'opt-out documenté `KESH_LOG_FILE_PATH=` vide est sans effet sous `${…:-défaut}` (F3).
- **Retenu** (décisions de l'orchestrateur) :
  - **Compose** : chaque variable ajoutée s'écrit `KESH_X: ${KESH_X:-}` (ou `${KESH_X:-défaut}` pour un défaut de déploiement voulu et documenté) — l'interpolation depuis `.env` est le mécanisme de base, documenté depuis Compose v1. La clé sans valeur est **interdite** par le test (V). **Révise C71** sur ce seul point ; `env_file` reste interdit, la liste explicite reste (C71 n'est pas réécrite).
  - **Code** : une fonction unique `config::env_nonempty(name) -> Option<String>` dans `kesh-api` (trim ; vide ou espaces → `None` ; non-UTF-8 → `None` avec avertissement ; absorbe `opt_trimmed_env`). Les **36** sites de lecture de production (`config.rs` 32, `main.rs` 2, `logging.rs` 1, `routes/onboarding.rs` 1) se réduisent à **un** ; `EXCEPTIONS_LECTURE` vide. Le test (L) rougit sur toute lecture hors d'elle, sur tout import de `std::env::var*` ou renommage de `std::env`, et la règle `cfg(test)` est écrite (items, instructions, `all` seul exclu).
  - **`KESH_LOG_FILE_PATH`** : seul « vide signifiant » trouvé (recensement de `.env.example` et des 36 lecteurs ; `KESH_ADMIN_*` « vide » = absent dans le code). **Écart avec la consigne** : l'orchestrateur la voulait « exception écrite de la fonction unique » ; elle n'en a pas besoin — dans le code, vide et absent signifient déjà tous deux « pas de journal fichier » (`LogConfig::from_raw` filtre le vide, test `config.rs:2315`). La distinction ne vit que dans le compose : forme `${KESH_LOG_FILE_PATH-/var/log/kesh/kesh.log}` (sans deux-points), liste fermée `VIDE_SIGNIFIANT` contrôlée par (V) et par la documentation du vide dans `.env.example`, mutation M20.
  - **Trim** : `KESH_COOKIE_SECURE` et `KESH_TEST_MODE` acceptent désormais `" true "` (le test `config.rs:2501` et trois commentaires sont mis à jour) — alignement sur `parse_strict_bool`, qui trime déjà ; `"True"`/`"yes"` restent refusés.
  - **AC15** (lecture unique) et **AC16** (propagation `restart` → `up -d` : `admin-manual.tex:1239`, `:1289`, `DOCKER_START.md:62`, `:106` ; la recette de la 15-7b2 reste à la 15-7b2, rebasée après la 15-11, C-15-7-51) ; garde-fous structurels `image:` et montages (M18, M19) ; « Action requise » avec bloc exact et liste des lignes de `.env` qui deviennent actives ; #551/#552 citées (15-12) ; écart avec l'« Attendu » de #550 déclaré (`KESH_STATIC_DIR`/`KESH_LOCALES_DIR` conservées dans `.env.example`, avec note). Mutations 16 → 21, AC 14 → 16.
  - **Découpage** : un crate, cinq modules de code — seuil non franchi ; coupe 15-11a (lecture unique) / 15-11b (compose, docs) signalée, non appliquée.
- **Écartées** : (a) garder la clé sans valeur et durcir seulement trois lecteurs — laisse la dépendance de version et la règle du vide à la discipline de chaque lecteur futur ; (b) `${X:-}` sans fonction unique — transmettrait le vide à des lecteurs qui ne le traitent pas ; (c) une exception de code pour `KESH_LOG_FILE_PATH` — sans objet (même sens dans le code) ; (d) ne pas trimer dans `env_nonempty` — deux politiques coexisteraient, et les lecteurs d'`opt_trimmed_env` trimaient déjà.
- **Réversible** : oui (fiche seulement, code non écrit).

## C76 — 15-11 : remédiation P2 — `KESH_ADMIN_PASSWORD` sans défaut dans `docker-compose.yml`, liste « relisez » fermée, `dotenvy` et macros vus par le test

- **Contexte** : validation P2 de la 15-11 (Opus ×2 ; `target/gate-logs/15-11-p2-{R,F}.md`) : 9 MEDIUM bruts, 7 distincts, 17 LOW. Le plus lourd (F1) : `docker-compose.yml` transmet `KESH_ADMIN_PASSWORD: ${…:-changeme}` ; absent de `.env`, il vaut `changeme`, que `Config::from_env` refuse (`InsecureAdminPassword`) — l'onboarding `/setup` que le manuel recommande est impossible sur ce compose, et le retrait des variables après un break-glass fait refuser le démarrage. La fiche l'écartait par un argument faux.
- **Retenu** (décisions de l'orchestrateur, appliquées par l'agent de remédiation) :
  - **F1** : dans Y, `KESH_ADMIN_PASSWORD: ${KESH_ADMIN_PASSWORD:-}` (vide = absent → `None` → setup-required, vérifié au code : `config.rs:619-637`, `auth/bootstrap.rs:62-69`) ; P, déjà `${KESH_ADMIN_PASSWORD}`, inchangé sur la valeur, ses commentaires « obligatoire » corrigés ; `KESH_ADMIN_USERNAME` reste `${…:-admin}` (le bootstrap exige le couple). Le risque — `/setup` ouvert au premier venu sur base vide, Y publiant `80:80` — est **écrit comme comportement existant et documenté** (`admin-manual.tex:980`, flux recommandé depuis v0.1.2, celui de P depuis toujours), non comme un risque nouveau.
  - **Garde-fou ajouté par l'agent** (choix propre, non demandé) : liste fermée `SANS_DEFAUT` = {`KESH_ADMIN_PASSWORD`} au test (V) — formes `${NOM:-}` ou `${NOM}` seules ; contrôle de la raison : la ligne de `.env.example` est commentée ; mutation M22. Motif : une correction sans test se défait en silence (« un patch vient avec son test »). Écarté : ne rien garder (régression muette possible) ; interdire tout défaut égal à une valeur refusée par le code (exige de recopier les règles de `Config::from_env` dans le test).
  - **R2-1** : (L) compte comme noms lus les arguments littéraux des sites directs (`env::var("X")`, `dotenvy::var("X")`), qui restent rouges comme sites ; **choix propre** : entrée transitoire `opt_trimmed_env` dans `INDIRECTIONS` du T1 au T3, pour que les cinq noms qu'elle lit ne passent pas pour fantômes au T1 (écarté : laisser le T1 rougir sur ces cinq noms en le déclarant — rouge plus large que le défaut de #550, preuve brouillée).
  - **R2-2 = F2, R2-3 = F3, R2-4, R2-5** : propagation du trim (`admin-manual.tex:1306`, `.env.example:110-111`, `espaces` au grep) ; liste « relisez » fermée depuis les variantes de `ConfigError` ; trois gestes au lieu d'un bloc à coller, `docker compose config -q` avant `up -d` ; commentaire `KESH_LOG_FILE_PATH` réécrit, (V) contrôlé par le marqueur « contrairement aux autres variables » et le refus de « ou absent ».
  - **F4** : `dotenvy::var*` est un site, son import interdit, (S) et M23 ; `TMPDIR` angle mort.
  - **O-1 (trouvé à la remédiation)** : (F) parcourt les littéraux de chaîne des macros — les deux messages porteurs du fantôme sont dans `write!` et `tracing::info!`, invisibles à `syn` ; sans cela M11 restait verte.
  - **Signal D5** : MEDIUM → MEDIUM, plusieurs défauts nés de la remédiation P1 (R2-1, R2-2, R2-4, R2-5), traités localement ; un crate, cinq modules : pas de découpage. Déclaré au Project Lead.
  - **Issue à ouvrir par l'orchestrateur** (F14) : `docker-compose.dev.yml` ne démarre pas sans `.env` (`KESH_ADMIN_PASSWORD: ${…:-admin}`, 5 caractères → `WeakAdminPassword`), alors que le README et le site y mènent.
- **Écartées** : (a) pour F1, la voie (b) du rapport — ouvrir une issue et écrire le défaut sans le corriger : laisserait la procédure recommandée en échec sur le fichier que l'AC5 fait devenir le fichier d'installation générique ; (b) passer aussi `KESH_ADMIN_USERNAME` à `${…:-}` : sans effet (couple exigé), et change une ligne de plus chez l'exploitant.
- **Réversible** : oui (fiche seulement, code non écrit).

## C77 — 15-11 : découpage en 15-11a (compose, documentation, test à liste fermée) et 15-11b (lecture unique, test qui lit le code)

- **Contexte** : validation P3 de la 15-11 (Sonnet ×2 ; `target/gate-logs/15-11-p3-{R,F}.md`) : R 2 MEDIUM / 7 LOW, F 4 MEDIUM / 6 LOW — 4 MEDIUM distincts plus F-4 (processus). Le **signal D5 est levé pour la deuxième fois, et par recyclage** : P1 → P2, 4 des 8 MEDIUM distincts naissaient de la remédiation P1 ; P2 → P3, 3 des 4 MEDIUM distincts naissent de la remédiation P2 (R3-1 = F-1, rouge exact du T1 ; F-2, exclusion `cfg(test)` de (F) ; F-3, `proc-macro2` non déclaré — les deux derniers nés du parcours des macros ajouté en P2). Ils se concentrent dans la machinerie du test qui lit le code (règles L/F/E, `syn`/`proc_macro2`). L'exception de l'amendement D5 (défauts distincts **et** non issus d'une remédiation) ne s'applique pas.
- **Retenu** (décision de l'orchestrateur, appliquée par l'agent de découpage) :
  - **15-11a-compose-transmet-la-configuration** (`closes #550`, `refs #534`) : compose (ajouts `${KESH_X:-}`, `KESH_LOG_FILE_PATH` en `${X-défaut}`, `KESH_ADMIN_PASSWORD` sans défaut), `image:`, montages de P, fantôme `KESH_ADMIN_RESET` (textes du code compris), `.env.example`, manuel, `DOCKER_START.md`, `up -d`, « relisez », gestes de mise à jour, CHANGELOG **Corrigé**, `docker compose config -q` en CI, `docs/ci.md`. Test **simple**, qui ne lit pas le code Rust : liste fermée `LUES` des 41 variables, écrite en dur, reproduite par une commande `grep` documentée ; (T), (V), (E), (F) sur les corpus texte, (S). Dépendance de test : `yaml-rust2` seule. 15 AC, 10 tâches, 19 mutations. La 15-7b2 dépend d'elle seule.
  - **15-11b-lecture-unique-des-variables** (`refs #550`) : `config::env_nonempty`, migration des 36 sites, test (L) par `syn` et (F) étendu au code et aux macros, qui **remplace** `LUES` (égalité assertée au T1) ; `proc-macro2` en dev-dépendance ; CHANGELOG **Modifié**. Dépend de la 15-11a et de la 15-5e1 (un seul `syn`, lock régénéré). 6 AC, 7 tâches, 11 mutations.
  - **15-11** devient une fiche index (`split`), qui garde son Change Log P1–P3 ; version complète au commit d69fdcca.
  - **Choix propre de l'agent — sûreté d'un merge de la 15-11a seule** : vérifié au code que les 16 ajouts, arrivés vides, ne font refuser le démarrage d'aucune installation (`KESH_COOKIE_SECURE` vide → `true`, booléens stricts → défaut, SMTP → `None`) ; effets transitoires écrits (avertissements « invalide » pour cinq variables numériques et `KESH_LANG`, sauvegarde pré-import dans `/app` pour `KESH_ADMIN_BACKUP_DIR` vide). La 15-11b est « attendue avant le tag v0.13.0 », non bloquante. Écarté : déplacer dans la 15-11a un correctif ponctuel des trois lecteurs (`KESH_SMTP_PORT`, `KESH_ADMIN_BACKUP_DIR`, `KESH_LANG`) — c'est la règle « vide = absent » de la 15-11b, morcelée, et l'inventaire du test ne la garderait pas.
  - **Choix propre — partage des textes** : la règle générale « une ligne vide vaut une ligne absente » et le trim (`.env.example` en-tête et `:109-111`, `admin-manual.tex:1306`, entrée CHANGELOG **Modifié**) vont à la 15-11b, qui les rend vrais ; la 15-11a garde le seul sens du vide qu'elle établit (`KESH_LOG_FILE_PATH`, marqueur « contrairement aux autres variables »).
  - **Choix propre — mutations** : la 15-11a ne mute aucun `.rs` de production (M3 simule une variable ajoutée en l'ajoutant à `LUES`) ; la 15-11b reprend les mutations du code et en ajoute deux (M10 : entrée d'`INDIRECTIONS` retirée ; M11 : doc-comment fantôme).
  - **R3-5 (registre)** : précise **C75** — les ajouts s'écrivent **tous** en `${KESH_X:-}` (R2-9) ; la parenthèse « ou `${KESH_X:-défaut}` pour un défaut de déploiement voulu » de C75 ne vaut que pour les entrées existantes. Le titre de C71 (« ajouts en clé sans valeur ») et les « seize mutations » de C72 sont historiques (révisés par C75, C76 et ce découpage). C71, C72, C75 ne sont pas réécrites.
- **Écartées** : (a) poursuivre la validation de la 15-11 entière avec une passe P4 — le recyclage montre que chaque remédiation de la machinerie (L)/(F) fait naître le défaut suivant, et la 15-7b2 attend ; (b) la coupe signalée en P1 (15-11a = lecture unique, 15-11b = compose) — met en premier la partie instable et retarde ce dont dépend la 15-7b2 ; (c) un test 15-11a qui lirait le code par expression régulière — c'est la méthode textuelle dont la 15-5e1 a dû sortir (C70), et elle serait jetée par la 15-11b.
- **Réversible** : oui (fiches seulement, code non écrit ; la version complète reste au commit d69fdcca).

## C78 — 15-11b : remédiation P1 — le test qui lit le code devient LEXICAL (sortie d'un recyclage de quatre passes)

- **Contexte** : validation P1 de la 15-11b (Opus 5.5 ×2 ; `target/gate-logs/15-11b-p1-{R,F}.md`) : R 5 MEDIUM / 8 LOW, F 3 MEDIUM / 8 LOW — 6 MEDIUM distincts (F2 ≈ R-2 + R-3 ; F3 = R-4). **Signal D5 levé, par recyclage** : R-1 naît de la remédiation R3-7 ; R-1, R-2, R-3 = F2 et F1 portent sur la même machinerie sémantique (reconnaissance des formes d'appel, imports, indirections « à un site », macros à motifs) que les passes P1-P3 de la 15-11. Quatre passes successives ont trouvé chacune la forme que la remédiation précédente ne voyait pas.
- **Retenu** (décision de l'orchestrateur, appliquée par l'agent de remédiation) : le test ne reconnaît plus de formes. Il parcourt le **flux de jetons** complet (`proc_macro2`, groupes, macros et attributs compris) de chaque `.rs` de production, relève **chaque occurrence** des jetons surveillés — `env` (hors `env!`), `dotenvy`, `from_default_env`, `try_from_default_env`, et les indirections `env_nonempty`, `parse_strict_bool`, `env_flag_enabled`, `init_tracing` (`opt_trimmed_env` du T1 au T2) —, la rattache à son **emplacement** (fichier + élément englobant le plus intérieur, visiteur `syn`, plages `cfg(test)` exclues) et à sa **fenêtre** (jetons suivants jusqu'au premier groupe `( … )`), et la confronte à une liste fermée `EMPLACEMENTS_AUTORISES` (fichier, emplacement, jeton, forme `Exacte`/`Littéral`, nombre exact ; 17 entrées attendues). Occurrence hors liste → rouge ; entrée au nombre différent → rouge. Faux rouge possible, faux vert impossible pour toute lecture par `std::env`, `dotenvy` ou un jeton surveillé — et rien de plus n'est affirmé. Les noms lus restent tirés des littéraux (fenêtres de lecture, autorisées ou non) plus `RUST_LOG` par `EnvFilter::DEFAULT_ENV`. (F) code devient lexical aussi : tout littéral de chaîne du flux, donc doc-comments, macros **et attributs** (`#[error]`).
- **Choix propres de l'agent** : (a) surveiller l'identifiant `env` **en entier** plutôt que `var`/`var_os`/`vars`/`vars_os` après `env ::` — un `use std::env::{self, var_os}` place `var_os` dans un groupe où il n'est pas précédé de `env ::`, et un `use std::env as e` ferait disparaître le préfixe ; surveiller `env` couvre les deux, au prix d'inventorier `std::env::temp_dir()` (`routes/admin.rs:80`), qui devient une entrée autorisée ; (b) ajouter `from_default_env` / `try_from_default_env` aux jetons (F8 c, coût nul) ; (c) rattachement par position (`proc-macro2` feature `span-locations`, côté cible seulement) plutôt que par réémission des jetons de chaque élément ; (d) `main.rs` appelle `env_nonempty` par chemin, sans `use` (un `use` serait une occurrence à autoriser, de fenêtre `env_nonempty` nue, indiscernable d'une référence non appelée) ; (e) R-5 : tests « valeur vide » discriminants, constatés rouges avant le T2, et capture `tracing` locale avec témoin positif pour les deux cas qui ne diffèrent que par l'avertissement ; (f) mutations 11 → 15.
- **Écartées** : (a) corriger chaque finding dans la machinerie sémantique (R-1 : site = paramètre de la fonction ; R-2 : liste de motifs de macro élargie ; R-3 : règles de référence et d'alias) — c'est la cinquième itération du même recyclage ; (b) découper encore — la story est déjà le produit d'un découpage, et le défaut est de méthode, non de taille ; (c) un test par expression régulière sur le texte — il verrait les commentaires et ne saurait pas exclure `cfg(test)`.
- **Fiche 15-11a non modifiée** : aucun finding ne l'exige.
- **Réversible** : oui (fiche seulement, code non écrit).

## C79 — 15-11a : remédiation P1 — troisième source de la liste « relisez » (chemins d'hôte de P), garde du placeholder `GENERATE_ME` du secret JWT (#557)

- **Contexte** : validation P1 de la 15-11a (Opus 5.5 ×2 ; `target/gate-logs/15-11a-p1-{R,F}.md`) : R 2 MEDIUM / 6 LOW, F 1 HIGH / 1 MEDIUM / 6 LOW ; R1-1 = F1-1, R1-7 = F1-7 — 14 findings distincts (1 HIGH, 2 MEDIUM, 11 LOW). Le HIGH : la liste « relisez votre `.env` », déclarée fermée, ne tirait ses entrées que du code (`ConfigError`, `process::exit`) et ignorait ce que le **compose** change — d'abord les `KESH_*_HOST_DIR` de `docker-compose.prod.yml`, que le manuel conseille de poser sur Synology et qui, honorées après la mise à jour, montent `/data/documents` sur un autre dossier (justificatifs et PDF figés « disparus », un refigement produisant un nouveau document). R1-2 : le placeholder actif du gabarit, `KESH_JWT_SECRET=<GENERATE_ME: openssl rand -hex 32>` (35 caractères), est accepté par `Config::from_env` — issue #557 ouverte par l'orchestrateur.
- **Retenu** (décisions de l'orchestrateur, appliquées par l'agent de remédiation) : (1) troisième source fermée « interpolations du compose nouvellement prises en compte ou dont la forme change » (T0, AC12 f, AC13), revérifiée au T8 contre `git diff main -- docker-compose*.yml` ; avertissement **en tête et en gras** du CHANGELOG et de la procédure de mise à jour, avec recette `grep`/`sed`/`rsync -a`/`diff -rq` (rejouée au scratchpad) ; (2) **AC16** : refus de tout secret contenant `GENERATE_ME` (sans égard à la casse), test qui lit la ligne réelle du gabarit, mutations M20 et M23, CHANGELOG **Sécurité** ; la fiche passe à `closes #550`, `closes #557` ; (3) `DOCKER_START.md:13, 23, 38-40` corrigés (F1-2) ; (4) tous les LOW.
- **Choix propres de l'agent** :
  - **Même variante `InsecureJwtSecret`** plutôt qu'une neuve : même défaut (placeholder non remplacé), même action (`openssl rand -hex 32`), appelants et tests existants inchangés ; une variante neuve dupliquerait message et traitement sans différence d'action. Variante unitaire (ne porte ni la valeur ni la sous-chaîne).
  - **Test unitaire qui lit `.env.example` par `include_str!`** (dans `mod tests`) plutôt qu'une copie du placeholder : le symptôme est « la ligne active du gabarit, recopiée, est acceptée » ; un test sur une copie resterait vert si le gabarit changeait de placeholder (M23 le prouve). Assertion de montage : valeur ≥ 32 caractères, sinon `WeakJwtSecret` masquerait le contrôle.
  - **R1-5 : `KESH_ADMIN_PASSWORD` de P passe aussi à `${…:-}`** (plutôt qu'écrire au manuel que l'avertissement de Compose est attendu) ; `SANS_DEFAUT` n'admet plus que `${NOM:-}` ; M22. Même valeur, une seule forme, plus d'avertissement sur le flux `/setup` recommandé.
  - **F1-8 : liste fermée `AJOUTS`** des 28 couples (variable, compose), forme `${NOM:-}` seule, vérifiée **quand la clé est présente** (l'absence reste le rouge de (T) : un seul rouge par défaut) ; M21. Écarté : règle de revue non outillée.
  - **F1-4 : `$` → `$$`** (consigne de l'orchestrateur), **mesuré** avant d'être écrit (Compose 2.40.3 : `pa$$word`, `'pa$word'`, `"pa$$word"` → `pa$word` ; `pa$word` et `"pa$word"` → `pa`, avec avertissement) ; le manuel cite aussi les apostrophes simples.
  - **Hors périmètre, signalé** : `KESH_ADMIN_PASSWORD=<GENERATE_ME: …>` (`.env.example:82`, commenté) serait accepté si décommenté tel quel — même défaut, moindre ; l'orchestrateur décide (extension de #557 ou issue). La fiche 15-7b2 (worktree `kesh-15-7`) n'est pas modifiée : ses lignes 540-541 disent encore « 15-11 » (R1-6), et son motif de contrôle n'énumère pas les nouvelles mentions de `KESH_PRODUCTION_RESET` au manuel et au CHANGELOG (F1-5) — report par l'orchestrateur.
  - **Signal D5 non déclencheur** : le HIGH est un défaut d'origine (la troisième source manquait depuis la conception de la liste), distinct, non né d'une remédiation. Modules de code recomptés : 3 (`config`, `main`, `lib`), l'AC16 vivant dans `config.rs`.
- **Comptes** : AC 15 → **16**, tâches 10, mutations 19 → **23**, modules 3.
- **Écartées** : (a) une variante `ConfigError` neuve pour `GENERATE_ME` (ci-dessus) ; (b) refuser tout secret commençant par `<` — plus large que le défaut constaté et sans gain (un placeholder court est déjà refusé par la longueur) ; (c) traiter `KESH_ADMIN_PASSWORD` au placeholder dans la même AC — hors de la décision de l'orchestrateur, signalé.
- **Réversible** : oui (fiche seulement, code non écrit).

## C80 — 15-11b : remédiation P2 — appels qualifiés d'`env_nonempty`, API de lecture de `tracing-subscriber` surveillée, promesse bornée au code du workspace

- **Contexte** : validation P2 de la 15-11b (Sonnet ×2 ; rapports `target/gate-logs/15-11b-p2-{R,F}.md`) — 2 MEDIUM distincts (R-1 = F-2 : un `use` d'`env_nonempty` dans `logging.rs`/`onboarding.rs` rougirait sans que la fiche le prescrive ; F-1 : `EnvFilter::from_env("X")` et le `Builder` lisent une variable sans jeton surveillé), 11 LOW. Non recyclés : aucun ne naît du patch de la P1 (amendement D5, pas de découpage).
- **Retenu** (décisions de l'orchestrateur) : (1) appel par chemin qualifié hors de `config.rs`, aucun `use` ; M12 réécrite ; (2) `EnvFilter` devient un jeton surveillé, entrées de `logging.rs` recomptées ; (3) la promesse de l'AC3 devient « faux vert impossible pour le code du workspace », les lectures internes aux dépendances sont un angle mort écrit avec la liste connue et sa méthode ; (4) tous les LOW traités.
- **Choix propres de l'agent** :
  - **`kesh_api::config::…` dans `main.rs`, non `crate::config::…`** : `main.rs` est le crate binaire, sans module `config` (il importe `kesh_api::…`) ; `crate::config` n'y compilerait pas. `crate::config::…` dans `logging.rs` et `routes/onboarding.rs`. La fenêtre commence à `env_nonempty` : les 17 entrées de la P1 restent exactes.
  - **Jetons surveillés étendus au-delà d'`EnvFilter`** : `Builder`, `with_env_var`, `from_env_lossy`, `try_from_env` — le constructeur s'atteint par `tracing_subscriber::filter::Builder` sans le jeton `EnvFilter` (ré-export `filter::env::Builder`, `Default` implémenté), et sa méthode `from_env` est homonyme de `Config::from_env` ; **`init` et `try_init`** — `tracing_subscriber::fmt::init()`/`try_init()` lisent `RUST_LOG` par `EnvFilter::from_default_env()` interne (`fmt/mod.rs:1200-1204`), trouvé en lisant les sources pendant la remédiation. Coût : 0 occurrence pour tous sauf `init` (1, `logging.rs:162`). Faux rouge futur possible sur un `fn init` ailleurs : prix accepté.
  - **Entrées : 17 → 22** (`EnvFilter` ×4 : `use`, type de retour et `EnvFilter::new(raw)` de `build_log_filter`, `DEFAULT_ENV` d'`init_tracing` ; `init` ×1). Le `[`EnvFilter`]` du doc-comment `:96` est un littéral, non un identifiant.
  - **Règle de fenêtre amendée** : un groupe `{ … }` ou `[ … ]` termine la fenêtre (exclu) ; sans cela, le type de retour `-> EnvFilter { … }` s'étendait jusqu'à la fonction suivante. Les fenêtres existantes ne changent pas (toutes s'arrêtent avant sur `( … )`, `;` ou `,`).
  - **Mutations M16** (`EnvFilter::from_env("…")`, (L)+(T)+(E)) et **M17** (`filter::Builder::default().from_env_lossy()`, (L) seule) : 15 → 17.
  - **F-4 rectifié au code** : les quatre `KESH_LOG_FILE_*` sont déjà trimés en aval (`from_raw`, `LogRotation::parse`, `LogFormat::parse`, `parse_max_files`) ; l'exemple des rapports (« `" daily"` devient valide ») est faux ; seul le vide change (avertissement « invalide » → défaut silencieux). Écrit tel quel dans la table avant/après.
  - **Vérification de la liste des dépendances (F-6)** : `cargo tree -p kesh-api --depth 1 -e normal` (28 dépendances directes hors `kesh-*`) + `sqlx-core`/`sqlx-mysql` 0.8.6, `grep -rlE 'env::var|var_os\(|getenv'` dans le `src/` de chaque paquet du registre, chaque site lu. API appelable : `dotenvy` et `tracing-subscriber` seulement (tous deux surveillés). Lectures internes à nom fixe : `NO_COLOR` (`fmt::Layer::default`, **lu en production** par `fmt::layer()`), `TOKIO_WORKER_THREADS`, `TZ` (`chrono`, `time`). Transitives non examinées. **Nuance pour l'orchestrateur** : « autres : aucun » est vrai pour les API de lecture, pas pour les lectures internes, d'où la liste.
- **Écartées** : (a) un `use` autorisé par deux entrées supplémentaires (19) — ouvre une forme de plus, l'appel qualifié n'en ouvre aucune ; (b) écrire `Builder`/`fmt::init` en angles morts plutôt que les surveiller — coût nul à surveiller, et `fmt::init()` est la ligne canonique des exemples de `tracing-subscriber`, la plus probable à être écrite ; (c) une garde de classement des dépendances nouvelles (suggestion F-6) — hors du périmètre de la story, angle mort écrit à la place.
- **Réversible** : oui (fiche seulement, code non écrit).

## C81 — 15-11a : remédiation P2 — placeholder refusé aussi pour `KESH_ADMIN_PASSWORD`, recette de déplacement réécrite et rejouée, tableaux de `sec:env-vars` repris de la 15-7b2

- **Contexte** : validation P2 de la 15-11a (Sonnet 5.5 ×2 ; `target/gate-logs/15-11a-p2-{R,F}.md`) : R 4 MEDIUM / 7 LOW, F 3 MEDIUM / 8 LOW (le bilan du rapport F en annonce 7 ; recompté F2-4 à F2-11) ; recoupements R2-3 = F2-1, R2-2 = F2-2 (+ F2-9), R2-4 ≈ F2-5 + F2-6 — 17 findings distincts (5 MEDIUM, 12 LOW). MEDIUM : `KESH_ADMIN_PASSWORD=<GENERATE_ME: …>` décommenté accepté (crée un administrateur au mot de passe publié) ; recette de P1 qui réécrit les droits de la destination (`rsync -a`) et copie au mauvais endroit en affichant « copie identique » (guillemets, `\r`) ; propagation du placeholder incomplète (README de crate hors du grep) ; tableaux de `sec:env-vars` rognés dans le PDF, rendant intenables les contrôles PDF de l'AC12.
- **Retenu** (décisions de l'orchestrateur, appliquées par l'agent de remédiation) : (1) AC16 étendue au mot de passe admin, constante commune, variante `InsecureAdminPassword` réutilisée, test calqué sur l'AC16 c, M24, AC12 e et AC13, manuel `:537-539` (mot de passe admin optionnel) ; placeholder contrôlé avant la longueur pour les deux (F2-8) ; (2) recette de déplacement réécrite et rejouée au scratchpad sur cas piégés ; (3) la 15-11a reprend la correction des dix tableaux (AC12 j) — la 15-7b2 n'a plus à le faire ; (4) propagation complète du symptôme, grep du T8 étendu à `README.md` et `crates/*/README.md` ; (5) tous les LOW, dont `DOCKER_START.md` en `docker compose` v2 et la brochure.
- **Choix propres de l'agent** :
  - **M25 en plus de M24** : le changement d'ordre (F2-8) est un comportement neuf, asserté par un cas du test `…_generate_me_case_insensitive` (secret court `GENERATE_ME`) ; sans mutation qui le voie rouge, cette assertion pourrait être muette. Coût : une mutation.
  - **Constante `TEMPLATE_PLACEHOLDERS = ["generate_me"]`** commune, les refus propres à chaque variable (`change-me` en sous-chaîne pour le secret, `changeme` en égalité pour le mot de passe) restant à leur place : les deux formes historiques n'ont pas la même sémantique (sous-chaîne / égalité), les fusionner aurait changé le refus existant du mot de passe.
  - **Le test qui lit le gabarit asserte que la valeur extraite contient `GENERATE_ME`** et qu'il y a exactement une ligne en colonne 0 (R2-9) : sans la première assertion, un gabarit sans placeholder ferait passer le test à vide ; l'ancienne assertion « ≥ 32 caractères » tombe avec le nouvel ordre.
  - **Recette** : plus de `mkdir -p` (la destination, dossier partagé DSM, se crée dans File Station, avec les droits du partage ; c'est `mkdir -p` qui fabriquait le dossier `"`) ; `S=""`/`S=sudo` explicite plutôt que `sudo` partout (`sudo` seulement pour `docker compose`, et pour la copie sur « Permission denied » — le conteneur tourne en root mais ses fichiers sont lisibles) ; sous-shell pour qu'un `exit 1` ne ferme pas la session SSH ; messages ASCII ; lignes ≤ 76 caractères parce que le style `kesh` de `lstlisting` replie les lignes longues (`breaklines = true`, `kesh-style.sty:224`), ce qui rendrait la commande non recopiable ; gestion de `export` et d'un commentaire de fin de ligne. Rejouée sous `dash`, `bash` et BusyBox sur dix `.env` piégés ; **non mesurée sur DSM**.
  - **CHANGELOG sans ligne de commande de copie** : une ligne `rsync` isolée de ses gardes reproduirait le défaut de la P1 ; il renvoie à la recette du manuel.
  - **F2-7 : clé i18n `error-invoice-pdf-gone` non modifiée** — le message est juste hors mise à jour (fichier réellement perdu), et les quatre locales sortent du périmètre ; la nuance « ne refigez pas avant d'avoir vérifié le montage » est au manuel et au CHANGELOG.
  - **F2-11 : brochure corrigée** plutôt que déclarée hors périmètre — une phrase, et le PDF est régénéré par le même `make fr`.
  - **`crates/kesh-api/README.md`** : seules les affirmations fausses sur les refus (`:38`, `:55-57`, `:62-65`) sont corrigées ; l'inventaire partiel de variables et l'exemple de requête restent hors périmètre, écrit.
  - **AC12 j** : le geste (`\paragraph{…}\mbox{}\\` ou `\subsubsection*{…}`) est **laissé à l'essai** au développement, comme dans la fiche 15-7b2 — non mesuré ici (compilation LaTeX hors du périmètre d'une remédiation de spec) ; le critère de choix est écrit (supprimer l'`Overfull` sans toucher la table des matières ni la numérotation).
  - **Signal D5** : HIGH → MEDIUM, critère « égale ou supérieure » non atteint ; 3 des 5 MEDIUM nés de la remédiation P1 (recette, propagation de l'AC16), 2 d'origine ; une seule zone (procédure de mise à jour du manuel), 3 modules de code : pas de découpage, **déclaré**.
- **Comptes** : AC 16 (AC12 gagne (j)), tâches 10, mutations 23 → **25**, tests unitaires de l'AC16 3 → **5**, modules 3.
- **À faire par l'orchestrateur** : étendre le texte de #557 au mot de passe admin ; ajuster la fiche 15-7b2 (worktree `kesh-15-7` : retirer la mise en page des dix tableaux de son AC 11 ligne `:691` et de son T7, et ajouter à sa liste de motif les mentions nouvelles de `KESH_PRODUCTION_RESET`).
- **Écartées** : (a) une variante `ConfigError` neuve pour le mot de passe admin au placeholder — même défaut, même action ; (b) modifier `\titleformat{\paragraph}` dans `kesh-style.sty` — touche les trois manuels ; (c) garder `mkdir -p` — il crée la destination sans les droits du partage et masquait l'erreur d'extraction ; (d) corriger la clé i18n — hors périmètre, message juste hors mise à jour ; (e) reporter le contrôle PDF des tableaux à la 15-7b2 (option b de F2-3) — décision contraire de l'orchestrateur.
- **Réversible** : oui (fiche seulement, code non écrit).

## C82 — Une cible cargo par worktree (incident de la cible partagée)

- **Contexte** : les consignes des agents (n° 8) imposaient `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh/target` à tous
  les worktrees, pour éviter des compilations à froid. L'agent de la 15-8b a constaté que `kesh-api` y avait été
  compilé contre le `kesh-db` d'une autre branche, sans recompilation ni erreur : cargo calcule l'empreinte des
  crates du workspace par chemin relatif, si bien que deux worktrees partagent leurs artefacts. Un gate peut donc
  avoir testé le code d'une autre branche.
- **Portée évaluée** : les PR fusionnées (#545 15-5b, #553 15-8a, #556 15-5c) ont toutes eu une CI GitHub verte,
  qui compile à neuf et rejoue la suite backend complète : leur code fusionné est validé indépendamment. La 15-8a et
  la 15-8b ont utilisé une cible propre (`target-158`). Le risque résiduel porte sur les **E2E locaux** des 15-5b,
  15-5c et 15-5e1 (la CI ne lance qu'un smoke) : un backend d'une autre branche a pu servir. La 15-5c ne touchait
  aucun fichier Rust ; la 15-5e1 refait ses gates sur sa propre cible avant sa PR.
- **Retenu** : chaque worktree a sa cible (`<worktree>/target`) ; consigne 8 réécrite, consigne 9 (attente des
  gates) filtrée sur le répertoire courant des processus (`wait-kesh.sh`).
- **Écarté** : garder la cible partagée en forçant des `cargo clean -p` (fragile, oubliable).
- **Réversible** : oui. À présenter au Project Lead en fin d'epic comme incident de méthode.

## C83 — 15-11a : remédiation P3 — AC4 abandonnée (montages fixes de `docker-compose.prod.yml`, recette de déplacement retirée, #558), gabarits `<…>` refusés ; révise C73, C77, C79 et C81

- **Contexte** : validation P3 de la 15-11a (Opus 5.5 ×2 ; `target/gate-logs/15-11a-p3-{R,F}.md`) : R 3 MEDIUM / 10 LOW, F 4 MEDIUM / 8 LOW ; 20 findings distincts (6 MEDIUM, 14 LOW). La recette de déplacement des dossiers montés (AC12 f, née en P1 de l'AC4) est en défaut pour la **troisième passe de suite** : elle lit `.env` avec `grep`/`sed` alors que Compose admet `KEY = v`, `KEY: v`, un commentaire après tabulation… — « rien à faire » en silence, ou copie au mauvais endroit affichée « copie identique » (R3-1 = F3-1) ; une destination non vide l'arrête sans consigne (F3-2) ; ses lignes se replient dans l'encadré (R3-2). Signal D5 levé par **recyclage** (même machinerie qu'en R2-1/R2-2). F3-3 : le déplacement sortait les PDF figés de la sauvegarde Hyper Backup documentée. F3-4 : les gabarits entre chevrons du manuel (`<mot de passe fort, …>`, `<nouveau-mdp-12+>`…) sont acceptés comme mot de passe admin.
- **Retenu (décisions de l'orchestrateur)** : (1) **abandon de l'AC4** — P garde `./documents`, `./inbox`, `./log` ; la recette et son encadré disparaissent ; les `KESH_*_HOST_DIR` restent au gabarit et au manuel (elles servent `docker-compose.yml`, et les règles (E)/(F) du test les admettent), marquées « sans effet avec `docker-compose.prod.yml` » ; version configurable renvoyée à l'**issue #558** ; (2) **AC16** : refus, pour `KESH_JWT_SECRET` et `KESH_ADMIN_PASSWORD`, de toute valeur de la forme `<…>` (après trim), en plus de `GENERATE_ME` ; (3) contrôle `Overfull` borné aux dix tableaux ; (4) tous les LOW.
- **Choix propres de l'agent** :
  - **Le test GARDE l'état des montages** plutôt que de les ignorer : (T) exige des sources exactement `./log`, `./inbox`, `./documents` dans P et `${KESH_*_HOST_DIR:-` dans Y (M14 réécrite sur Y, M28 neuve sur P, message qui renvoie à #558). Raison : sans garde, une contribution future pourrait rendre P configurable « en passant » — exactement le changement dont trois passes ont montré le danger. La story #558 retirera la garde délibérément. Écarté : supprimer toute contrainte de montage (silence) ; exiger `${NOM` « dans un des deux compose » (moins lisible que nommer Y).
  - **(E) cherche `${NOM` dans les `volumes:` de `docker-compose.yml` seul** — le rouge (E) du T1 disparaît ; (E) reste exercé par M3, M7, M12, M14.
  - **Numéro d'AC4 conservé** (« montages inchangés ») pour ne pas décaler seize renvois.
  - **La vérification des placeholders passe par Compose** (`sudo docker compose config | grep -iE 'KESH_(JWT_SECRET|ADMIN_PASSWORD): .?(<|.*generate_me)'`) au lieu d'un `grep` sur `.env` (R3-13 = F3-11) : c'est la leçon même de R3-1 — ne pas réimplémenter le lecteur de `.env`. Sur-ensemble assumé (une valeur commençant par `<` sans `>` final apparaît), écrit au manuel ; motif à mesurer au T6 sur `.env` piégés, et à corriger — non à déclarer en limite — si le rendu YAML de Compose y échappe.
  - **Fonction commune `is_template_placeholder`** (sous-chaînes de `TEMPLATE_PLACEHOLDERS` **ou** forme `<…>` après trim local) : une seule définition pour les deux contrôles ; le trim est local parce que le secret JWT n'est pas trimé à la lecture avant la 15-11b. Le contrôle `change-me` et celui du placeholder du secret forment une seule condition, avant la longueur (M25 la redescend entière ; cas `xchange-mex`, F3-8).
  - **M27** (ordre du contrôle admin, R3-7) : même motif que M25 en C81 — une assertion d'ordre sans mutation qui la voie rouge peut être muette.
  - **Manuel** : `:243-244` (exemple de l'*Étape 3*) passent en commentaire, « optionnel » — la phrase « au minimum les variables suivantes » cessait d'être vraie depuis C81 ; les quatre gabarits restent des gabarits manifestes, avec « remplacez la valeur entre chevrons ».
  - **`:1384` et `:1518`** (sauvegarde) précisés par compose : seul résidu de F3-3, qui disparaît pour P ; la précision pour `docker-compose.yml` décrit une situation antérieure à la story.
  - **`Overfull` de `sec:inbox-import` (`821--825`)** : laissé hors périmètre (préexistant, hors des dix tableaux, non touché par l'abandon) — couvert par « aucun `Overfull` nouveau ».
- **Révisions explicites** : **C73** — « montages de P en `${KESH_*_HOST_DIR:-…}` » est **retiré** (le reste de C73 tient) ; **C77** — la 15-11a ne livre plus les chemins d'hôte de P ; **C79** — la troisième source de la liste « relisez » perd les montages (garde `KESH_LOG_FILE_PATH`, `KESH_ADMIN_PASSWORD`) et l'écarté (b) « refuser tout secret commençant par `<` » est **renversé** (forme `<…>` complète, pour les deux variables) ; **C81** — la recette de déplacement réécrite et rejouée, son choix « messages ASCII, lignes ≤ 76, `S=sudo` », et la nuance F2-7 (clé `error-invoice-pdf-gone`) deviennent **sans objet**.
- **Comptes** : AC 16, tâches 10, mutations 25 → **28**, tests unitaires de l'AC16 5 → **6**, lignes ajoutées aux compose 28, modules 3.
- **À faire par l'orchestrateur** : (a) **#558** : y reporter la recette retirée et les findings P1-P3 qui la concernent (R1-1, R2-1, R2-2, R3-1 = F3-1, F3-2, R3-2, F3-3) comme cahier des charges ; (b) **#557** : étendre le texte à la forme `<…>` ; (c) issue de F14 (`docker-compose.dev.yml` sans `.env`) : y ajouter `website/index.html:189` (F3-10) ; (d) fiche 15-7b2 (worktree `kesh-15-7`) : cellule `:1314` et T7 en `docker compose up -d` (F3-12) ; (e) signaler au Project Lead `CLAUDE.md:180` (`KESH_ADMIN_PASSWORD='<12+ caractères>'`, refusé recopié tel quel) ; (f) vérifier que la 15-7b3 ne touche pas `admin-manual.tex:1384`/`:1518`.
- **Écartées** : (a) corriger la recette une quatrième fois en lisant la source par `docker compose config` (proposition de R3-1/F3-1) — elle résout la lecture mais garde la copie, la destination non vide, la sauvegarde et la largeur : la machinerie reste ; (b) sortir la recette dans une story sœur liée à l'AC4 — l'AC4 n'apporte rien que les exploitants de P attendent avant #558 ; (c) retirer les `KESH_*_HOST_DIR` du gabarit — `docker-compose.yml` les emploie réellement ; (d) réécrire les exemples du manuel sans chevrons (option b de F3-4) — la règle de forme ferme la classe, présente et à venir.
- **Réversible** : oui (fiche seulement, code non écrit) ; #558 reprend la fonctionnalité.

## C84 — 15-11a : clôture de la validation (P4, 0 au-dessus de LOW) — `$` entre apostrophes simples, motif de vérification élargi, coordination avec la 15-7b3 ; marque C73, C77, C79, C81 comme révisés par C83

- **Contexte** : validation P4 de la 15-11a (Sonnet 5.5 ×2 ; `target/gate-logs/15-11a-p4-{R,F}.md`) : R 14 LOW, F 9 LOW, 0 au-dessus de LOW ; 20 LOW distincts. La validation **converge** ; les LOW sont appliqués à la fiche, sans nouvelle passe.
- **Révisions par C83, annotées ici (F4-1)** — les entrées d'origine ne sont pas réécrites ; qui les lit seules doit savoir : **C73** — « montages de P en `${KESH_*_HOST_DIR:-…}` » est **retiré** (P garde ses montages fixes) ; **C77** — la 15-11a ne livre plus les chemins d'hôte de P, et ses comptes « 15 AC, 10 tâches, 19 mutations » sont périmés (16 AC, 10 tâches, 28 mutations à la clôture) ; **C79** — la troisième source de la liste « relisez » perd les montages, et l'écarté (b) « refuser tout secret commençant par `<` » est **renversé** (forme `<…>` refusée) ; **C81** — la recette de déplacement « réécrite et rejouée » n'est **plus une décision vivante** : elle est retirée avec l'AC4 (cahier des charges de #558).
- **Choix propres de l'agent** :
  - **`$` dans une valeur : apostrophes simples d'abord** (R4-13). Mesuré au scratchpad : Compose 2.40.3 et `dotenvy` 0.15.7 (lu par `cargo run`, `main.rs:44`) lisent `'pa$word'` littéralement ; `pa$$word` vaut `pa$word` pour Compose mais `pa` pour `dotenvy` ; `"pa$word"`/`"pa$$word"` sont des erreurs d'analyse pour `dotenvy` ; `"it's pa\$word"` vaut `it's pa$word` dans les deux. `.env.example` (qui sert aux deux) ne dit plus `$$` ; le manuel le cite en second, comme échappement propre à Compose ; `docker compose config` réaffiche tout `$` en `$$`, écrit. Écarté : garder `$$` en tête (faux hors Docker) ; `\$` sans guillemets (Compose le lit `pa\`, mesuré).
  - **Motif de vérification `: .? *(<|.*generate_me)`** (R4-14) : `.? *` au lieu de `.?`, conformément à la consigne de C83 (« corriger le motif, non le déclarer en limite ») ; rejoué sur dix-sept `.env` (quatorze piégés montrés, trois témoins muets) ; seconde ligne de 70 caractères (≤ 76). Le sur-ensemble s'élargit d'un cas (un caractère, des espaces, puis `<`), écrit. Écarté : `.{0,6}` (borne arbitraire).
  - **M23 réécrite sur `REMPLACER_MOI: openssl rand -hex 32`** (R4-5) : ni `<…>`, ni `GENERATE_ME`, ni `change-me`, ≥ 32 — le refus manquerait aussi si l'assertion de montage tombait. Écarté : garder `<A_GENERER: …>` en corrigeant seulement le rationnel (la mutation n'exercerait plus que l'assertion de montage).
  - **Listes `HOTE` et `MARIADB` écrites en dur** (R4-11), avec contrôle de raison et rouge sur entrée inutilisée — comme les autres listes fermées. Écarté : les dériver des `volumes:` (un montage supprimé ferait disparaître la variable de la liste en silence).
  - **Extraction de la source d'un montage** (R4-12 = F4-8) : recherche de `:<cible>` final (ou suivi de `:ro`/`:rw`), forme longue rouge, chaque cible exactement une fois.
  - **Coordination avec la 15-7b3** (R4-1 = F4-2) : ordre 15-11a d'abord ; la 15-7b3 relocalise par le texte et place son texte après l'énumération et l'encadré de la 15-11a. Clôt l'action (f) de C83 : `:1384`/`:1518` libres ; `:1704-1717` partagé ; `:1734-1749` voisin.
- **À faire par l'orchestrateur** : (a) **#558** : remplacer « les retire de la documentation » par « les marque *sans effet* avec le compose de production » et « 70 caractères » par 76 (R4-4) ; (b) **fiche 15-7b3** (worktree `kesh-15-7`) : déclarer la zone partagée — voir le rapport de clôture ; (c) les actions (a) à (e) de C83 restent dues si elles ne sont pas faites.
- **Comptes** : AC 16, tâches 10, mutations 28, tests unitaires de l'AC16 6, lignes ajoutées aux compose 28, modules 3.
- **Réversible** : oui (fiche seulement, code non écrit).

## C85 — 15-5d et 15-5e2 : alignement sur le livré (15-5e1, 15-8a, 15-8b)

- **Contexte** : la 15-5e1 (PR #559, `de1e1c26`) et la 15-8a (PR #553) sont mergées, la 15-8b est en
  cours de rebase (worktree `kesh-15-8`). Les fiches 15-5d et 15-5e2, écrites avant, citaient une
  partition, des sites `retry_with`, des passages du Pattern 5 et des numéros de ligne périmés. Code
  relu sur `cecd5d1d` (branche de planification qui intègre `de1e1c26`). Fiches seules, aucun code.
- **Retenu** :
  1. **Partition d'arrivée de la 15-5e2 : 22 `Rejouee` / 0 / 4 / 89 = 115, dans les deux ordres de
     merge avec la 15-8b.** Mesuré : 9 / 13 / 4 / 89 sur `cecd5d1d` (recompté sur le registre) ;
     **10 / 12 / 4 / 89** sur le worktree de la 15-8b (diff non commité de son rebase) — le `DELETE`
     était `ARejouer("15-5e2")`, il passe `Rejouee`, et le nombre de `SansEcritureAuJournal` ne bouge
     pas. ⚠️ **La prévision transmise à l'agent (10 / 13 / 4 / 88, puis 23 / 4 / 88) ne correspond pas
     au registre** : elle supposait le `DELETE` classé `SansEcritureAuJournal` ; il ne l'est pas. La
     fiche écrit le mesuré. Écarté : écrire la prévision (fausse sur le source).
  2. **Tous les sites `retry_with` directs des routes migrent vers une enveloppe, sauf
     `post_accept`.** Six sur `cecd5d1d` (`invoices::write_off`, `opening_balances::complete`,
     `journal_entries::update` de la 15-8a, `reconciliation::cancel`, `onboarding::finalize`,
     `reconciliation::accept`), sept avec le `DELETE` de la 15-8b. Chaque migration est une
     équivalence exacte (même `DEFAULT_MAX_DEADLOCK_ATTEMPTS`, même prédicat, même nom) ;
     `post_accept` garde `retry_with` pour son prédicat élargi au 1305, réécrit
     `is_app_deadlock(err) || matches!(…TransactionAborted)`. C'est aussi la migration des
     prédicats en ligne laissée à la 15-5e2 (finding B-3 de la revue de code de la 15-5e1). **Révise
     la fiche 15-5e2** (« `onboarding::finalize` garde son `retry_with` », finding F4-11) : la garder
     aurait laissé un prédicat en ligne identique à `is_app_deadlock` pour un gain nul. Écarté :
     garder `finalize` en `retry_with` avec `is_app_deadlock` comme prédicat (DRY partiel, un
     `retry_with` direct de plus à justifier).
  3. **La rubrique « Deny list » du Pattern 5, posée par la 15-8a (et complétée par la 15-8b), est
     versée à « Where This Applies »** quand la 15-5e2 retire la rubrique : ordre, raison, cycles
     hérités et tests gardés ; seule la colonne « Mitigation » change (enveloppe). Raison : sans
     ordre global, « divergent » n'a plus de référence, mais l'analyse des cycles du `PUT` est la
     plus précise du document. Écarté : retirer les lignes avec la rubrique (perte d'information) ;
     garder une « Deny list » d'un ordre qui n'existe plus.
  4. **Étiquettes des comptes désignés de la 15-5d** : `(2 bis', suite)` à la validation (juste après
     le bloc `let rounding` qui clôt `(2 bis')`, avant `(2 ter)`, `(2 quater)` et `(3)`) ; `(2 ter)`
     à la saisie fournisseur (entre `(2, suite)` et `(3)`, étiquette libre dans cette fonction).
     Chacune présente au doc-comment **et** au code (leçon A1 de la revue de code de la 15-5e1).
     Écarté : réutiliser `(2 ter)` à la validation (déjà pris, sans verrou) ; laisser le choix au dev
     sans le fixer (deux passes l'auraient relu chacune à sa façon).
  5. **Inventaire au symptôme de la 15-5e2 recompté : 347 lignes / 52 fichiers** sur `cecd5d1d`
     (205 / 42 sur `f289414e`) — l'écart vient surtout des fichiers écrits par la 15-5e1 et la 15-8a ;
     chaque occurrence reste triée, bloc par bloc. Relevé élargi des manuels : **17 lignes** (la ligne
     « atomique » réécrite par la 15-5c ne sort plus).
  6. **15-5d** : rien de la 15-5c n'y est (libellés de `failed[]` propres au rapprochement) ;
     `sitesTotal` (1904 sur `cecd5d1d`) se recompte sur l'état rebasé ; aucun test de rejeu prévu,
     témoin `tests/common/capture_rejeu.rs` nommé pour le cas où.
- **À faire par l'orchestrateur** : (a) informer l'agent de la 15-8b que sa partition attendue est
  **10 / 12 / 4 / 89** (et non 10 / 13 / 4 / 88) — son diff de rebase l'écrit déjà ainsi ; (b) passe
  ciblée de validation sur cet alignement (sections listées aux Change Logs des deux fiches) ;
  (c) la 15-5e1 a laissé « à ajuster par l'orchestrateur » la mention de la 15-5e2 sur `:289-291` et
  `:338-351` du Pattern 5 : fait ici.
- **Réversible** : oui (fiches seulement).

## C86 — 15-5e2 : remédiation de la validation P6 — test « route victime » sur `post_manual` (révise C56 et l'écart de C69), volet (c bis), contrôle `retry_with` étendu aux tests

- **Contexte** : validation P6 de la 15-5e2 (Opus ×2, première passe après l'alignement C85 ; `target/gate-logs/15-5e2-p6-{R,F}.md`) : 2 MEDIUM distincts (R6-1, F6-1), 15 LOW. R6-1 : le contrôle `grep -rn "retry_with" crates/*/src docs` de l'AC1 ne voit pas `crates/*/tests`, où six mentions deviennent fausses du fait même des migrations décidées par C85 — critère de clôture qui certifie l'absence de résidus qu'il ne peut pas voir. F6-1 : C56 (et l'écart consigné en C69) excluait un test dynamique au rollout ; le harnais livré par la 15-5e1 (`victime`, `transaction_lourde`, `CaptureRejeu`) rend désormais un test « route victime » sur `post_manual` abordable (~70 lignes), et c'est précisément le chemin que l'AC2 déclarait non exercé. Décisions de l'orchestrateur, appliquées par l'agent de remédiation.
- **Retenu** :
  - **R6-1** : le contrôle couvre `crates/*/src crates/*/tests docs` ; six sites nommés et réécrits (`journal_entry_reversal_e2e.rs:1609`, `journal_entries_modification.rs:441`, `audit_route_registry.rs:89` — limite (vi), nommée à côté de (iii bis) —, `:119`, `:165`, `:600`) ; occurrences légitimes listées (`capture_rejeu.rs:6`, `reconciliation_e2e.rs:4414`, sources synthétiques du banc du visiteur).
  - **F6-1 — C56 révisé** : la 15-5e2 ajoute le **test 8** (`manual_match_is_replayed_when_it_is_the_deadlock_victim`, `rejeu_interblocage_e2e.rs`), montage avec projet — la transaction de test, lourde, tient la sentinelle `companies` puis demande l'exercice ; la route tient l'exercice (étape 6) et attend la sentinelle (`validate_taggable_in_tx`, étape 6bis) —, témoin `exiger_un_rejeu("reconciliation::manual")`, mutation « enveloppe de `post_manual` retirée » → rouge. T0 forme d'abord le cycle à la main sur MariaDB 10.11 ; s'il ne se forme pas, le test est écrit comme angle mort et la fiche le dit. L'angle mort « famille `AppError` non exercée » est retiré de l'AC2 et des Dev Notes, ramené à `post_split` et `complete_import`. Noms d'opération des trois routes `AppError` fixés : `imported_supplier_invoices::complete`, `reconciliation::manual`, `reconciliation::split`.
  - **LOW, tous appliqués** : R6-2 = F6-2 (commentaires de `post_accept` → `is_app_deadlock`) ; R6-3 ; R6-4 = F6-6 (forme fixée : ordre dans « Lock sequence », paragraphe « Notes » sous la table, « diverges » reformulé) ; R6-5 (cinq sites migrés, six avec le `DELETE`) ; R6-6 (§ 10 d'`api-external.md`, phrases sous la table) ; R6-7 ; **F6-4** (`ENVELOPPES` sans `retry_with`, liste `RETRY_WITH_AUTORISE = ["post_accept"]`, banc transposé, cas négatif) ; **F6-5** (garde durable de la migration de `finalize` : volet (c bis), test `no_route_calls_retry_with_except_post_accept` qui parse par `syn` tout `crates/kesh-api/src/routes/` et refuse `retry_with` hors de `post_accept` quel que soit le statut de la route, plus une mutation) ; F6-7 (motif de l'inventaire étendu : 385 lignes / 62 fichiers sur `cecd5d1d`) ; F6-8 ; F6-9 (routes rejouées énumérées dans `api-external.md`, phrase converse) ; F6-10 (consigne de rebase de la 15-8b dans l'en-tête).
  - **Signal D5** déclaré : MEDIUM après une P5 à 0 ; R6-1 né de la remédiation C85, F6-1 contestation d'un choix dont la prémisse a changé ; aucun recyclage, pas de découpage.
- **Écartées** : maintenir C56 en écrivant pourquoi le test resterait prohibitif (F6-1, option 2) — le harnais existe, le coût est d'une page ; un test « route victime » aussi pour `post_split` et `complete_import` — `post_split` suit le chemin de `post_manual`, exercé sur la route sœur, et le montage de `complete_import` (`staging` ↔ réglages) n'est pas établi ; restreindre le contrôle `is_deadlock_error` au code hors commentaires (R6-2, option 2) — les commentaires de `post_accept` doivent nommer le prédicat réel.
- **Réversible** : oui (fiche seule ; aucun code écrit).
- **À propager par l'orchestrateur** : la consigne de rebase de F6-10 dans la fiche 15-8b (worktree `kesh-15-8`).

## C87 — 15-5d : remédiation de la validation P5 — verrou partagé des comptes désignés (révise C43 et le mode fixé depuis), étiquette d'achat `(2, désignés)` (révise C85), développement après la 15-8b

- **Contexte** : validation P5 de la 15-5d (Opus ×2, première passe complète après l'alignement C85 ;
  `target/gate-logs/15-5d-p5-{R,F}.md`) : 1 HIGH, 4 MEDIUM, 8 LOW distincts. **F5-1 (HIGH)** : le
  `FOR UPDATE` que C43 posait sur les comptes désignés **avant** l'exercice formait un cycle
  **systématique** avec tous les flux qui prennent l'exercice puis reprennent la créance, la TVA due ou
  les créanciers en verrou **partagé** par `fk_jel_account` (règlement client par virement, solde du
  reste, rapprochement, avoir, règlement fournisseur) — et la fiche disait que ces places « réduisent
  la fréquence ». Décisions de l'orchestrateur, appliquées par l'agent de remédiation. Les entrées
  antérieures (C43, C51, C53, C85) ne sont pas réécrites.
- **Retenu** :
  1. **Verrou partagé** : `… WHERE company_id = ? AND id IN (…) ORDER BY id LOCK IN SHARE MODE`
     (MariaDB 10.11 ; pas `FOR SHARE`). Il suffit au but de l'accesseur : l'archivage, la case
     *imputable*, le retypage (`accounts::update`, `accounts::archive`) et la création d'un sous-compte
     **écrivent** la ligne du compte par un `UPDATE`, qui prend un exclusif et attend. **Vérifié au
     code** (`accounts.rs:466-470`, `:558`, `:654-658`, `:669`, `:194`) : ces routes ne verrouillent
     **pas** la ligne avant leur propre contrôle (lecture simple de l'instantané) ; c'est l'`UPDATE`
     lui-même qui pose l'exclusif. Le verrou partagé garantit donc « aucune écriture de la ligne entre
     le verrou de l'accesseur et le commit », et la lecture verrouillante lit la dernière version
     validée. Compatible avec les partagés de clé étrangère : le cycle de F5-1 ne se forme plus.
  2. **Plus aucune affirmation de baisse de fréquence** : les Dev Notes énumèrent les cycles examinés —
     ne se forment plus ou pas : (a) F5-1, (b) validation ↔ solde du reste sur la TVA due, (c') saisie
     ↔ règlement fournisseur par compte interne, (g) flux de même nature ; **restent possibles**, rares
     et rejoués : (c) validation arrondie ↔ règlement client par compte interne = créance ou TVA due
     avec écart (exclusif contre partagé — aucune place ne ferme à la fois (b) et (c)), (b') solde du
     reste dont la nature est un compte de produit des lignes validées (né de l'accesseur), (d) trois
     parties dont une modification du compte (partagé en file derrière un exclusif en attente) ; non
     examinés : lot de paiement, acceptation par lot du rapprochement. Établi par lecture, non
     reproduit.
  3. **Tests de l'AC7 réécrits** : chaque test tient lui-même le verrou concurrent (un `UPDATE …
     postable = FALSE` non validé, un `FOR UPDATE`, un `LOCK IN SHARE MODE`) et constate l'attente sur
     une requête nommée ; test 1 (vente) et test 2 (achat) : attente sur l'accesseur, sonde de
     l'exercice `NOWAIT` qui réussit, puis lecture fraîche (refus après le commit de la bloqueuse) ;
     test 3 inchangé (arrondi avant) ; **test 4 neuf, de mode** (la bloqueuse tient l'exercice et les
     comptes en partagé ; la validation est vue en attente sur l'exercice — seul test qui rougit si
     l'accesseur repasse en `FOR UPDATE`). **L'ancien test 1** (escompte ↔ validation, deux flux réels)
     est **retiré** : sous l'exclusif, il formait lui-même le cycle de F5-1. Mutations du mode
     ajoutées (exclusif → test 4 ; sans verrou → tests 1 et 2).
  4. **Étiquette d'achat `(2, désignés)`** au doc-comment et au code (finding R5-4) — révise C85,
     item 4 : `(2 ter)` venait d'être retirée de cette fonction (finding A1 de la 15-5e1), et le
     doc-comment de `validate_invoice`, auquel celui d'achat renvoie, dit « (2 ter) ne prend aucun
     verrou ».
  5. **La 15-5d se développe sur `main` après le merge de la 15-8b** (PR #560), ou se rebase dessus
     (finding R5-1 : la 15-8b touche `invoices.rs` — +5 lignes avant `validate_invoice` —, les deux
     `errors.rs`, les quatre `messages.ftl`, `i18n-keys.test.ts`, `docs/api-external.md`,
     `CHANGELOG.md`, les manuels et leurs PDF) ; relocalisation par le texte. Avec la 15-5e2, l'ordre
     de merge reste indifférent pour l'ordre des verrous, **pas** pour les fichiers communs (PDF à
     régénérer, `user-manual.tex`, `CHANGELOG.md`, « 5 bis » — R5-6).
  6. Corrections de fait : le compte de produit d'une ligne non imputable rend
     `INVOICE_LINE_REVENUE_ACCOUNT_INVALID` (`InvalidRevenueAccounts`, 16-1a), pas
     `ACCOUNT_NOT_POSTABLE` — AC1, justification de C46 à l'AC4, test d'ordre de l'AC7 (F5-2) ; deux
     phrases du CHANGELOG `[0.13.0]` réécrites par l'AC9, motifs ajoutés au grep de T6 (F5-3) ;
     montage `kesh-api` complété par des fixtures existantes — `seed_accounting_company`,
     `disable_rounding_to_5_centimes`, contact, Comptable par `users::create` + login,
     `init_error_i18n` (F5-4 = R5-3) ; angle mort « type » écrit à l'AC3 (F5-8) ; dérivés de
     `GeneratedLines` / `DesignatedRole` et leur module (F5-7) ; « Quatre cas » repris ensemble (F5-6) ;
     « devenu non imputable » élargi (F5-5) ; numéros relocalisés (R5-2, F5-9, R5-5).
- **Écartées** : (a) garder `FOR UPDATE` en écrivant le cycle comme introduit — un cycle systématique
  sur la ligne la plus chaude d'une société pour aucun gain de garantie ; (b) déplacer l'accesseur
  avant l'arrondi — fermerait (c) mais ouvrirait le symétrique de (b) contre le solde du reste avec
  écart ; (c) découper la story sur le signal D5 — le défaut recyclé (ordre des verrous, né de C43)
  se traite localement, sans toucher d'autre module ; (d) `(2 quater)` côté achat — libre mais sans
  rapport de sens avec le `(2 quater)` de la vente.
- **Signal D5** : la sévérité **monte** (P4 ciblée 0 → P5 HIGH) et le HIGH est **recyclé** (thème
  « ordre des verrous », né de la remédiation C43) : déclaré au Project Lead ; traité localement, pas
  de découpage.
- **Réversible** : oui (fiche seulement ; code non écrit).

## C-15-5e2-1 — 15-5e2 / T0 : le cycle du test 8 se forme sur MariaDB 10.11.16 ; relevés sur `HEAD`

- **Contexte** : T0 de la 15-5e2 (AC2, choix C86) — former à la main, avant de l'écrire, le cycle du test 8 (`post_manual` victime), sur la base dédiée `kesh_155e2` (`10.11.16-MariaDB-ubu2204`, `innodb_deadlock_detect = ON`, `innodb_deadlock_report = full`) ; refaire les relevés de l'en-tête sur `HEAD` (`688fed25`, qui porte la 15-8b).
- **Mesures** : session A alourdie (500 lignes de lest), `SELECT id FROM companies WHERE id = 1 FOR UPDATE` ; session B — la requête réelle de `find_open_covering_date` (`… FROM fiscal_years WHERE company_id = 1 AND start_date <= … AND end_date >= … AND status = 'Open' LIMIT 1 FOR UPDATE`) puis `SELECT id FROM companies WHERE id = 1 FOR UPDATE`, qui attend ; A demande `SELECT id FROM fiscal_years WHERE id = 1 FOR UPDATE` → **B reçoit 1213, A obtient son verrou** (`SHOW ENGINE INNODB STATUS` : la transaction 2, B, est la victime). Le montage de l'AC2 tient sans adaptation ; le test 8 est écrit **en vert**, non comme angle mort. Relevés : partition de départ **10 / 12 / 4 / 89** (la 15-8b est mergée) ; **sept** sites `retry_with` dans `src/routes` ; inventaire au symptôme **394 lignes / 62 fichiers** (272 / 42 sous `src` et `docs`, 122 / 20 sous `tests`).
- **Réversible** : sans objet (mesure).

## C-15-5e2-2 — 15-5e2 : noms d'opération des routes rejouées ici

- **Contexte** : l'AC1 fixe les trois noms de la famille `AppError` ; les neuf routes `DbError` n'en ont pas.
- **Retenu** : `"<module>::<action>"` sur le module de la route et le verbe du dépôt : `invoices::unvalidate`, `credit_notes::create`, `supplier_invoices::pay`, `supplier_invoices::cancel`, `supplier_invoices::cancel_settlement`, `payment_batches::confirm`, `journal_entries::create`, `journal_entries::reverse`, `opening_balances::generate`. Les six sites migrés gardent leur nom (équivalence exacte).
- **Écartée** : le nom du handler (`create_journal_entry`…) — plus long, et incohérent avec les noms déjà posés par la 15-5e1 (`invoices::settle`, `supplier_invoices::create`).
- **Réversible** : oui (seule la journalisation porte ces noms).

## C-15-5e2-3 — 15-5e2 : frontière des fonctions « une tentative » de `post_manual` et `post_split`

- **Contexte** : l'AC1 demande une fonction « une tentative » qui ouvre la transaction, prend le verrou nommé, écrit et conclut, avec les contrôles qui lisent la transaction **dedans**, dans leur ordre.
- **Retenu** : les contrôles 0 à 4bis (`post_manual`) et 0 à 7 (`post_split`) — validation du corps, lectures **sur le pool**, hors transaction et non verrouillantes — restent dans le handler, avant l'enveloppe ; la tentative commence au `begin()` et reprend tel quel le bloc sous `with_account_lock` et son `match`. Ce que la tentative consomme est reconstruit en elle (libellé pour le manuel ; lignes de ventilation, détails d'audit et libellé de l'écriture pour la ventilation). `complete_import_once` reçoit tout le corps de l'ancien handler sauf la résolution de la société (le verrou du `staging` est son premier acte).
- **Écartée** : faire entrer les pré-vols dans la tentative — ils ne lisent rien que la transaction verrouille, et les rejouer ne changerait que la latence ; ils auraient aussi changé la place des refus (AC6).
- **Réversible** : oui.

## C-15-5e2-4 — 15-5e2 : forme du volet (c bis) et de son banc

- **Contexte** : l'AC1 fixe le test `no_route_calls_retry_with_except_post_accept` (analyse `syn` de chaque fichier de `src/routes/`, échec nommant fichier et fonction).
- **Retenu** : un visiteur qui tient la **pile** des fonctions (`ItemFn` et méthodes `ImplItemFn`) et attribue chaque appel `retry_with` (`ExprCall` au dernier segment, ou `ExprMethodCall`) à la fonction la plus proche — un appel hors fonction est relevé `<hors fonction>` ; balayage **récursif** de `src/routes/` (aucun sous-répertoire aujourd'hui) ; garde « détecteur cassé » si moins de onze fichiers. Un **banc** neuf, `the_primitive_visitor_names_the_enclosing_function`, éprouve le visiteur sur un source synthétique (commentaire, chaîne et doc-comment ignorés ; fermeture, méthode et chemin qualifié vus) — au-delà de l'AC, pour la même raison que le banc du volet (c) : un détecteur qui ne s'éprouve qu'en mutant le dépôt ne s'éprouve pas.
- **Écartée** : réutiliser `CherchePlusieursFn` — il cherche un nom donné et ne verrait pas une fonction inconnue.
- **Réversible** : oui.

## C-15-5e2-5 — 15-5e2 : relevés de l'inventaire au symptôme — précisions « pour cette paire »

- **Contexte** : AC3, tri bloc par bloc ; les commentaires « gardés, vrais pour la paire qu'ils nomment » reçoivent une précision si le texte laisse entendre plus.
- **Retenu** : précisés — `projects.rs` (« Ordre de verrouillage global » → convention de Pattern 5, « évite, **pour cette paire** »), `supplier_invoices.rs` étape (0) et `reconciliation_rules.rs` (« anti-ABBA avec l'archivage d'un projet »), `invoices.rs` (`update` : « l'ordre de la création » au lieu de « l'ordre de verrous global »), `reconciliation_cancel.rs` (le côté fiche facture est rejoué depuis la 15-5e1, #463) ; deux phrases de tests décrites comme vraies **dans leur montage** (`opening_complement_repository.rs:746`, `fiscal_years_repository.rs:1071`), assertions et messages d'`expect` intacts. Gardés sans retouche : `journal_entry_number_sequences.rs` (entre créations), `invoices.rs:1074-1078` (borné aux `invoices` / `invoice_lines`), `:2014`, `:2124` (paire avec création et modification), `company_invoice_settings.rs:312` (ordre entre comptes désignés, sans prétention d'absence de cycle), `onboarding.rs:232`, `:683`, `:850`, `:907` (sérialisation et déterminisme de sélection, hors sujet).
- **Réversible** : oui (commentaires).

## C-15-5e2-6 — 15-5e2 : Pattern 5 et manuel d'administration — choix de forme

- **Contexte** : AC3 (Pattern 5) et AC5 (`99-kesh.cnf`).
- **Retenu** : (a) le Pattern 5 reste **en anglais**, langue du document ; « Global Lock Order » devient « a frequency convention, not a guarantee », nomme les trois verrous partagés de clé étrangère et les quatre flux inversés ; la table gagne la ligne du lot de rapprochement (`accept_batch`, ordre par proposition et entre propositions) et les lignes du `PUT` et du `DELETE` (ordre seul), leurs raisons, cycles, mitigation et tests passant dans **« Notes »** sous la table ; la puce « Resolution status » nomme les deux enveloppes et le registre ; l'exemple « How to use » de la 15-5e1 est gardé, et « Required » dit que la forme générique `retry_with` est réservée à `post_accept` dans `src/routes/`. (b) Au manuel d'administration, la consigne `innodb_deadlock_detect` est un **commentaire** du listing, sans ligne `innodb_deadlock_detect = ON` : « laisser à sa valeur par défaut » ne demande rien d'écrire.
- **Réversible** : oui.

## C-15-5e2-7 — 15-5e2 : remédiation de la revue de code P1 — choix de forme

- **Contexte** : revue P1 (Sonnet ×3 ; `target/gate-logs/15-5e2-review-p1-{B,E,A}.md`) : 1 MEDIUM (A1), 14 LOW, dont trois doublons (L-1 = B-2, L-3 = B-4 = A5).
- **Retenu** :
  - **L-1 = B-2** : `was_previously_rejected` et le montant de l'audit sont lus dans la tentative, sur la transaction bancaire re-lue sous verrou (`post_manual_once`, `post_split_once`), et ne sont plus des paramètres. Preuve par **extension du test 8** plutôt que par un test neuf : entre l'attente de la route sur la sentinelle et la fermeture du cycle, une écriture validée pose `auto_match_rejected_at` ; l'audit doit dire `true`. Mutation (lecture neutralisée à `false`, ce qu'écrivait la pré-lecture) : rouge, `Some("false")` contre `Some("true")`. Ceci amende **C-15-5e2-3** : les pré-lectures du handler n'alimentent plus aucune écriture.
  - **B-3** : `conclude_locked_attempt(tx_outer, lock_result, flow)` porte le `match` commun ; seule différence conservée, le nom de route dans le 500 défensif des variants `Rule*` (même texte qu'avant).
  - **B-5** étendu aux **cinq** fermetures migrées qui clonaient le pool (les trois nommées, plus `onboarding::finalize` et `reconciliation::cancel`), pour une seule forme ; `post_accept` (non migré, `retry_with`) inchangé.
  - **L-3 = B-4 = A5** : angles morts écrits (point (vii) du doc-comment du registre, doc du volet, Dev Notes), non fermés — la méthode lexicale de la 15-11b (C78) est nommée, non introduite.
  - **A4** : le « Required » du Pattern 5 renvoie aux doc-comments de `kesh_db::retry::retry_on_deadlock` et du module `kesh_api::retry`, sans recopier les clauses.
  - **B-6** : liste orpheline de l'étape 0-bis refondue en phrase ; ligne longue de la doc de `update_journal_entry` recoupée ; `{settlementId}` de `docs/api-external.md:488` **gardé** — c'est la convention du document (camelCase, même graphie à `:321`, `{reminderId}`, `{documentNumber}`), non celle des chemins Axum.
  - **L-5** : inventaire nommé (`PUT /invoices/{id}` avec changement de projet, archivage de projet, clôture et réouverture d'exercice) ajouté au point (iv) du registre comme angle mort, sans cycle démontré. Issue à ouvrir par l'orchestrateur (amélioration, `v0.2-milestone`), pour que la limitation soit tracée.
- **Écartées** : un test dynamique neuf pour `split` et `complete_import` (B-1 = L-4) — accepté LOW, angle mort (iii bis) déjà écrit ; fermer le volet (c bis) par un relevé lexical maintenant — hors périmètre, c'est la machinerie de la 15-11b.
- **Réversible** : oui.

## C-15-11a-1 — 15-11a (dev) : forme du test `configuration_transmise` — fonctions pures, deux listes de plus que la fiche, garde de tri par octets

- **Contexte** : l'AC8 décrit les contrôles (T, V, E, F, S) et leurs listes fermées ; elle laisse la structure du test à l'implémentation.
- **Retenu** : chaque contrôle est une **fonction pure** sur des chaînes (`controle_transmission`, `controle_valeur`, `controle_raisons_valeurs`, `controle_env_example`, `controle_fantomes`, `source_montage`/`sources_montages`), appelée par les tests du dépôt **et** par les tests (S) sur sources synthétiques — l'auto-test exerce le code même qui juge le dépôt. Deux listes non nommées par la fiche, toutes deux avec contrôle de raison : **`VALEURS_COMPOSEES`** (l'exception « `DATABASE_URL` de `docker-compose.yml` » de (V), contrôlée par la présence de `${MARIADB_` dans la valeur) et **`MONTAGES`** (les trois cibles et leurs sources exigées, Y préfixe / P exacte). La garde `garde_liste_lues` vérifie aussi que `LUES` est **triée par octets et sans doublon** et que `AJOUTS` compte 28 couples. `HOTE` se contrôle sur les **entrées analysées** de `volumes:` de `kesh-api` (un commentaire qui nomme la variable ne compte pas — c'est ce qui fait rougir M14 en (E)) ; `MARIADB` sur le texte de `docker-compose.yml`.
- **Constaté en route** : le tri de `sort -u` sous la locale française place `FILE_BYTES` avant `FILES_PER_RUN` ; Rust compare les octets (`S` < `_`). La commande de l'en-tête du test porte donc `LC_ALL=C sort -u`.
- **Effet de bord, écrit** : M10 (`${KESH_SMTP_HSOT:-}`) rougit **deux fois** — (V) et (F), le jeton `KESH_SMTP_HSOT` étant un fantôme du corpus compose. La fiche n'annonçait que (V).
- **Écartées** : un test monolithique par fichier (l'auto-test aurait dû recopier la logique) ; dériver `VALEURS_COMPOSEES` d'une détection « contient un `${` étranger » (une interpolation d'une autre variable sous une clé passerait en silence).
- **Réversible** : oui (test seul).

## C-15-11a-2 — 15-11a (dev) : l'ordre du test du secret JWT collecte les écarts au lieu de s'arrêter au premier

- **Contexte** : sous M25 (contrôle des placeholders redescendu après la longueur), `config_rejects_jwt_secret_generate_me_case_insensitive` s'arrêtait sur `GENERATE_ME` ; le cas `xchange-mex` (F3-8 : remontée de `change-me` avant la longueur) n'était alors **jamais exercé** par la mutation.
- **Retenu** : la boucle du test collecte les écarts et asserte la liste vide à la fin ; M25 nomme désormais les **deux** cas (`"GENERATE_ME" → WeakJwtSecret { 11 }`, `"xchange-mex" → WeakJwtSecret { 11 }`). Commit séparé (`80231b62`), M20 et M25 rejouées après.
- **Écartées** : un test par cas (six tests de plus pour une seule règle) ; laisser tel quel (le cas `xchange-mex` n'aurait été prouvé par aucune mutation).
- **Réversible** : oui.

## C-15-11a-3 — 15-11a (dev) : mise en page de `sec:env-vars` — `\paragraph{…}\mbox{}\\`, `sloppypar` et deux `\par` ; un seul des deux gestes essayé

- **Contexte** : AC12 j demande zéro `Overfull \hbox` entre `\label{sec:env-vars}` et `\subsubsection{Import de factures depuis un dossier}`, par un même geste sur les dix titres, choisi « après essai » entre `\paragraph{…}\mbox{}\\` et `\subsubsection*{…}`, sans toucher `kesh-style.sty`.
- **Retenu** : `\paragraph{…}\mbox{}\\` sur les dix titres (texte inchangé, `\paragraph` hors table des matières, numérotation inchangée). Il supprime l'essentiel des débordements ; **trois causes de plus** restaient dans la section, sans lien avec le titre en ligne : (a) les paragraphes de prose — dont les deux nouveaux de l'AC12 a — qui portent de longs `\texttt` insécables (`crates/kesh-api/src/config.rs`, `KESH_SMTP_PASSWORD='pa$word'`) : enveloppés dans `sloppypar` ; (b) les deux tableaux précédés d'un texte dans le même paragraphe (SMTP, chemins d'hôte) : `\par\noindent` avant `\begin{tabularx}` ; (c) la cellule `DATABASE_URL` : `\allowbreak` après `PASS@`. Résultat mesuré : **0** `Overfull` dans les bornes (67 → 52 au total), **aucun** nouveau ailleurs (comparaison par contenu des boîtes, avant/après). Les cinq débordements que les nouveaux paragraphes de la sous-section « Passer à la 0.13.0 » et des items `:1020`/`:1260`/`:2235` créaient ont été traités de même (`sloppypar`).
- **Écart déclaré** : `\subsubsection*{…}` **n'a pas été essayé** — le premier geste a suffi ; il aurait aussi changé la taille et l'espacement des titres.
- **Réversible** : oui.

## C-15-11a-4 — 15-11a (dev) : la procédure de mise à jour gagne une sous-section, l'encadré « relisez » avant les gestes ; quatre sites de plus que la fiche

- **Contexte** : AC12 f décrit le contenu (voie recommandée, vérification par Compose, deux gestes, encadré, vérification fonctionnelle) sans fixer la place exacte.
- **Retenu** : le point 3 de la *Procédure de mise à jour standard* devient « Mettre à jour le fichier compose — obligatoire pour passer à la 0.13.0 » et renvoie à une nouvelle `\subsubsection{Passer à la 0.13.0 : …}\label{sec:maj-0-13}`, placée après l'encadré de sauvegarde (la 15-7b3, rebasée, placera son texte après elle). L'encadré « Relisez votre `.env` » vient **avant** la voie recommandée et les gestes, donc avant tout `up -d`. Les lignes des deux `lstlisting` sont **extraites du compose final** par script (15 + 13 + 2), aucune n'est repliée dans le PDF (`pdftotext -layout` : 30 lignes `KESH_…: ${…}` entières).
- **Sites ajoutés par la propagation** (grep du symptôme) : `admin-manual.tex` § *Reset du mot de passe administrateur (break-glass)* (« Retirez ensuite `KESH_ADMIN_PASSWORD` » → « puis `docker compose up -d kesh-api` ») ; *Étape 5* Synology (« se connecter avec le compte admin renseigné dans `.env` » → `/setup` sur base vide) ; une `keshnote` après la méthode GUI de Container Manager (AC12 i) ; la ligne `:1239` perd « (Optionnel) » — c'est le geste qui retire les variables du conteneur.
- **Correction en cours de route** : la réécriture du bloc Synology `:537-539` avait d'abord supprimé la ligne `openssl rand -base64 32 # → MARIADB_…` ; rétablie (« mot de passe MariaDB, repris dans `DATABASE_URL` »).
- **Réversible** : oui (documentation).

## C-15-11a-5 — 15-11a (dev) : montage E2E — port 3004, répertoires inbox/documents dans le scratchpad, secrets générés

- **Contexte** : plusieurs worktrees de l'Epic 15 font tourner leurs E2E sur le même MariaDB ; la recette du dépôt emploie `/tmp/kesh-e2e/*` et des secrets fixes.
- **Retenu** : base `kesh_e2e_1511a`, port **3004** (vérifié libre), `KESH_INBOX_DIR`/`KESH_DOCUMENTS_DIR` dans `scratchpad/e2e-1511a/` (aucun partage de répertoire avec un autre agent), `KESH_JWT_SECRET` = `openssl rand -hex 32`, `KESH_ADMIN_PASSWORD` = 24 caractères aléatoires (l'AC16 refuse désormais tout gabarit), les quatre `KESH_SMTP_*` de la recette ; `/health` contrôlé (`smtpConfigured: true`) avant la suite.
- **Réversible** : oui (montage local).

## C-15-11a-6 — 15-11a (revue de code P1) : vide = défaut pour sept variables, par une garde de `match` et non par une nouvelle fonction

- **Contexte** : B1 = E-1 = A-L3 (MEDIUM) — les compose transmettent `${NOM:-}` ; `KESH_ADMIN_BACKUP_DIR` vide devenait le chemin `""` (sauvegarde pré-import écrite dans `/app`), `KESH_LANG` vide avertissait « Locale '' non reconnue », et les cinq numériques (`KESH_PASSWORD_MIN_LENGTH`, `KESH_BANK_IMPORT_MAX_MB`, `KESH_ADMIN_EXPORT_INMEM_MB`, `KESH_ADMIN_IMPORT_MAX_MB`, `KESH_SMTP_PORT`) avertissaient « invalide » à chaque démarrage. Décision de l'orchestrateur : corriger ces sept dans la 15-11a, la 15-11b généralisant à toutes.
- **Retenu** : `KESH_ADMIN_BACKUP_DIR` et `KESH_LANG` passent par `opt_trimmed_env` (vide ou blanc = absent, valeur trimée) ; les cinq numériques gardent leur `env::var` et gagnent un bras `Ok(val) if val.trim().is_empty() => <défaut>` — une valeur non blanche garde exactement son comportement (pas de trim : `" 12 "` reste « invalide » pour `KESH_PASSWORD_MIN_LENGTH`, comme avant). Tests : `from_env_empty_or_blank_vars_take_code_default_silently` (vide et blanc, valeurs ET absence de tout message nommant la variable, capture `tracing` locale avec assertion de montage) et son témoin `from_env_non_empty_invalid_values_still_warn`. Mutations : chacune des sept lectures remise dans sa forme d'avant → rouge (7/7).
- **Écartées** : une fonction `non_blank_env` (écrite puis retirée) — elle aurait ajouté un jeton de lecture que l'inventaire de `LUES` (grep `env::var|opt_trimmed_env|parse_strict_bool|env_flag_enabled`) ne voit pas, et changé l'inventaire de la 15-11b plus que nécessaire ; `${KESH_ADMIN_BACKUP_DIR:-/tmp}` dans les compose (recopie le défaut, contraire à C75 et rouge au test (V)) ; `opt_trimmed_env` pour les numériques (trimerait une valeur non blanche : changement de comportement que la 15-11b assume, pas la 15-11a).
- **Effet de bord écrit** : `KESH_LANG=" de "` donne désormais `de` (trim d'`opt_trimmed_env`) au lieu de l'avertissement puis `fr` ; `KESH_ADMIN_BACKUP_DIR` est trimé.
- **Propagation** : le doc-comment d'`is_loopback_host`, détaché de sa fonction et collé au-dessus d'`opt_trimmed_env` **avant** la story (même symptôme que B4), est rattaché à sa fonction dans le même commit.
- **Frontière avec la 15-11b**, et ce qu'il faut reporter dans sa fiche : voir le Change Log de la 15-11a (revue P1).
- **Réversible** : oui.

## C-15-11a-7 — 15-11a (intégration sur `9cb5083b`) : union du registre avec dédoublonnage, PDF utilisateur restauré, six rouges E2E jugés au rejeu isolé

- **Contexte** : rebase de la 15-11a sur `origin/main` après le merge de la 15-5e2. Le commit de planification
  reporté de la branche (`63c73e59`) portait un bloc C66–C74 **déjà présent** sur `main` ; l'union brute des deux
  côtés l'aurait mis deux fois. `make admin user` régénère aussi `user-manual.pdf`, que la branche ne modifie pas.
  L'E2E rend 13 échecs, dont six hors de la liste de `docs/testing.md`.
- **Retenu** : (a) registre — union des deux côtés, puis suppression du second bloc C66–C74 après contrôle
  qu'il est **identique octet pour octet** au premier (`diff` vide) ; ensemble des titres égal à l'union de
  `origin/main` et de la branche, aucun doublon ; (b) `sprint-status.yaml` — les deux en-têtes `last_updated`
  gardés, celui de la 15-11a passé en « (19) » ; (c) `user-manual.pdf` régénéré au texte identique à celui de
  `main` → version de `main` gardée (aucun octet changé sans raison) ; `admin-manual.pdf` régénéré et commité ;
  (d) les six rouges hors liste (`contact-duplicate-probe.spec.ts:78`, `:113`, `contacts.spec.ts:39`,
  `onboarding.spec.ts:33`, `:119`, `invoice-frozen-pdf.spec.ts:74`) rejoués **seuls** sur le même backend :
  six verts ; signature KF-053 (#478 : `page.fill('#username')`, run allongé à 13,5 min) plus une pollution.
  Pas de second run complet.
- **Écartées** : garder les deux blocs C66–C74 (doublon de titres, recherche par numéro ambiguë) ; commiter le
  PDF utilisateur régénéré (bruit binaire sans changement de texte) ; relancer la suite E2E entière (la règle
  du dépôt juge un rouge au rejeu isolé, et les six passent).
- **Réversible** : oui (sauvegarde `backup/15-11a-pre-rebase-2` sur `7d0fd45b`).

## C88 — 15-5d : remédiation de la validation P6 — cycles (a bis) examinés, dérogation écrite au découpage (signal D5 de la P5), `owned_account_ids` adopté d'emblée (tranche C51), montage et sondes des tests

- **Contexte** : validation P6 de la 15-5d (Sonnet ×2 ; `target/gate-logs/15-5d-p6-{R,F}.md`) :
  R 0 MEDIUM / 5 LOW, F 2 MEDIUM / 3 LOW — 2 MEDIUM, 8 LOW distincts, aucun né de la remédiation C87.
  **F6-1** : la fiche renvoyait au développeur, comme « non examinés », le lot pain.001 et
  l'acceptation par lot du rapprochement, qui se tranchent à la lecture. **F6-2** : le critère D5
  (recyclage) n'était pas appliqué à la lettre au HIGH F5-1 de la P5 — le constat « traité
  localement » décrivait l'étendue du correctif, non la nature du défaut. Décisions de
  l'orchestrateur, appliquées par l'agent de remédiation. Les entrées antérieures (C51, C87) ne sont
  pas réécrites.
- **Retenu** :
  1. **Cycle (a bis)** aux Dev Notes : lot pain.001 (`confirm_batch` → `pay_in_tx` en
     `BankTransfer` seul : facture, `bank_accounts`, exercice, jamais de `FOR UPDATE` sur les comptes
     désignés) et acceptation par lot (exercice tenu, créance en partagé par `fk_jel_account` ; seul
     exclusif après l'exercice : le compte d'arrondi, cycle préexistant couvert par #536 / 15-5e2) —
     classe (a), aucun cycle neuf ; T0 **confirme**. **Remplacement du plan et désarchivage** relus au
     code (axe non exercé de la lentille R) : la plage `accounts.rs:1070-1216` est
     `delete_all_by_company`, **sans appelant**, suivie des tests ; `reset_demo` supprime en
     autocommit (aucun verrou tenu entre deux instructions) ; le chargement d'un plan n'écrit que des
     `INSERT` sur une société sans réglages ; `reactivate` ne verrouille rien avant son `UPDATE` —
     ni chemin de défaut, ni cycle (un attendeur qui ne tient rien n'est maillon d'aucun cycle).
  2. **Dérogation règle de splitting** (section neuve de la fiche) : signal D5 écrit tel qu'il est
     (F5-1 né de C43 ; thème « ordre des verrous » en P2, P3, P5) ; la seule coupe disponible —
     AC5/AC6, l'écran du compte créanciers et les contournements E2E — ne porte pas l'axe recyclé, qui
     est au cœur de la garde : découper ne traiterait pas la cause. Risque accepté ; P6 dernière passe
     complète, P7 ciblée ; si une passe ciblée trouve encore un défaut de verrou né d'une remédiation,
     la coupe AC5/AC6 s'applique sans nouvelle délibération.
  3. **`owned_account_ids` adopté d'emblée** (R6-5) : lecture non verrouillante des identifiants de
     la société dans la transaction, puis verrou partagé sur eux seuls — tranche ce que C51 laissait
     au résultat du test ; le test « autre société » reste, mutation « patron retiré » ajoutée ; il
     asserte aussi le refus `InactiveOrInvalidAccounts` (F6-5), avec la limite du test « archivé »
     écrite (résultat, non auteur).
  4. **Montage** (R6-1) : tous les tests de vente sauf le test 3 sous `disable_rounding_to_5_centimes`
     — choisi plutôt qu'un TTC multiple de 0.05, que le test « TVA arrondie à zéro » ne peut pas tenir ;
     achat sans objet (aucune étape d'arrondi).
  5. Finitions : `NOWAIT` premier emploi, erreur attendue `1205` (mesurée en 10.11.16 par la lentille
     R), test discriminant sur ce code (R6-4) ; doc-comment d'`attendre_une_requete_en_cours` réécrit,
     `test_fixtures.rs` aux fichiers touchés (R6-2) ; lettres des cycles expliquées, non renumérotées
     (R6-3) ; rubrique `### Ajouté` à créer en tête de `[0.13.0]` (F6-3) ; verrous d'intervalle au
     doc-comment de l'accesseur, `EXPLAIN` à la main en T0, renvoi à C-15-8-23 (F6-4).
- **Écartées** : (a) découper sur la coupe AC5/AC6 — elle laisse l'axe recyclé entier d'un côté ;
  (b) un TTC multiple de 0.05 au lieu de désactiver l'arrondi — inapplicable au test « TVA arrondie à
  zéro » ; (c) laisser le patron `owned_account_ids` au rouge du test — décision certaine différée
  pour rien ; (d) renuméroter les cycles — C87 cite les lettres.
- **Signal D5 — à présenter au Project Lead (Guy) en fin d'epic** : la 15-5d a franchi le critère de
  recyclage en P5 (HIGH F5-1 né de C43) et n'est pas découpée ; la dérogation est écrite dans la fiche
  (« Dérogation règle de splitting »). En P6, la sévérité baisse (HIGH → MEDIUM) et rien n'est recyclé.
- **Signalé à l'orchestrateur, hors périmètre** : le doc-comment d'`accounts::delete_all_by_company`
  (`accounts.rs:1063`) annonce « utilisé par reset_demo », ce qui est faux.
- **Réversible** : oui (fiche seulement ; code non écrit).

## C-15-5d-1 — 15-5d (dev) : la bloqueuse du test de mode lit `name`, et vérifie elle-même ce qu'elle tient

- **Contexte** : l'AC7 (test 4) fait tenir à la bloqueuse la créance et la TVA due « en partagé » par
  `SELECT id FROM accounts WHERE id IN (…) LOCK IN SHARE MODE`. Écrit ainsi, le test **passait sous la mutation
  « accesseur en `FOR UPDATE` »** — celle qu'il existe pour attraper. Mesuré au développement (MariaDB 10.11.16, base
  de gate à cinq comptes) : le plan de cette requête est `index` sur **`fk_accounts_parent`**, *Using index* — un
  index secondaire couvrant —, et un verrou **partagé** posé par un index secondaire couvrant ne verrouille **pas**
  la ligne de la clé primaire ; une sonde `FOR UPDATE NOWAIT` sur la créance réussit alors que la bloqueuse est
  censée la tenir. La clé étrangère `fk_jel_account` et l'accesseur, eux, verrouillent la clé primaire.
- **Retenu** : la bloqueuse lit `SELECT id, name …` (`name` n'est dans aucun index secondaire : plan `range` sur
  `PRIMARY`), et le test **vérifie son propre montage** par deux sondes `NOWAIT` qui doivent échouer (`1205`) avant
  de lancer la validation. Sous la mutation `FOR UPDATE`, le test rougit désormais (attente sur les comptes, panique
  au bout de dix secondes). Commentaire écrit au test.
- **Écartées** : `FORCE INDEX (PRIMARY)` (lie le test à un nom d'index et cache la raison) ; garder la requête de la
  fiche (test vert à vide).
- **Portée** : le même piège guette **tout** test qui simule un verrou partagé par `SELECT id … LOCK IN SHARE MODE`
  sur `accounts` : à signaler à la revue (axe « bloqueuses des tests de verrou »). L'accesseur n'est pas concerné :
  il lit `active` et `postable`, hors de tout index secondaire (`EXPLAIN` : `range`/`const` sur `PRIMARY`).
- **Réversibilité** : totale (test seul).

## C-15-5d-2 — 15-5d (dev) : le test « identifiant d'une autre société » passe par le vrai flux

- **Contexte** : l'AC7 fait appeler l'accesseur de verrou par une connexion de test. L'accesseur est
  `pub(in crate::repositories)` (même visibilité que les générateurs, F5-7) : un test d'intégration ne l'atteint pas.
- **Retenu** : une bloqueuse tient l'exercice (`FOR UPDATE`) ; la **validation réelle** est lancée et vue en attente
  sur l'exercice — donc passée l'accesseur, ses verrous posés ; la sonde `NOWAIT` sur la ligne étrangère doit
  réussir, et une sonde témoin sur la créance doit échouer (`1205` : l'accesseur tient bien la créance). Puis la
  bloqueuse annule et la validation rend `InactiveOrInvalidAccounts`, rien d'écrit. Mutation « patron
  `owned_account_ids` retiré » : rouge.
- **Écartées** : rendre l'accesseur `pub` pour le seul test (élargit une surface que C50/F5-7 ont voulue étroite).
- **Réversibilité** : totale.

## C-15-5d-3 — 15-5d (dev) : forme du contrôle, réponse HTTP commune, sonde partagée, test de l'avoir

- **Contrôle** : le second temps est une méthode de l'instantané, `DesignatedAccountsSnapshot::check_written(roles,
  settings)` ; la traduction rôle → identifiant est le `match` exhaustif de `DesignatedRole::designated_id` (C44), les
  candidats `DesignatedRole::SALE` / `PURCHASE`. Un rôle écrit sans compte désigné (impossible : le générateur a
  refusé `ConfigurationRequired`) rend `DbError::Invariant` plutôt qu'un `continue` muet.
- **HTTP** : les bras `AccountsNotPostable` et `DesignatedAccountsNotPostable` partagent
  `account_not_postable_response(key, fallback, accounts)` (`kesh-api/src/errors.rs`) — même code, même détail, seule
  la clé du message diffère (règle DRY).
- **Sonde** : `test_fixtures::sonde_verrou_nowait(pool, sql, id)` — `true` si la sonde réussit, `false` sur `1205`,
  panique sur toute autre erreur ; partagée par les tests de vente et d'achat.
- **Avoir** : le test « l'avoir est exempté » (C35) vit dans le module `garde_usage_comptes_reglage` de
  `invoices_validate_vat.rs` (il en réutilise le montage), et non dans `credit_notes_repository.rs` comme la fiche
  le prévoyait.
- **Réversibilité** : totale.

## C-15-5d-4 — 15-5d (dev) : le manuel passe de « quatre cas » à « trois », la garde à l'usage écrite à part

- **Contexte** : l'AC8 réécrit le cas (4) de l'encadré *Rôles des comptes* (« un compte désigné … reste utilisé ») et
  la phrase « Si vous scindez un tel compte … ».
- **Retenu** : la garde à l'usage est écrite dans le **premier** paragraphe de l'encadré (où sont les contrôles), avec
  ses deux exceptions (avoir, compte de produit par défaut) et le remède « un compte imputable — l'un de ses
  sous-comptes, si vous l'avez scindé » ; l'énumération des cas qui échappent devient **« Trois cas »** (valeur
  recomptée, `grep` du `.tex` et du PDF aplati). La note des comptes de clôture renvoie désormais à cette section pour
  « le contrôle à l'usage … et les cas qui échappent encore » (sa phrase précédente ne disait plus tout). Le passage
  de l'avoir (« l'inverse exact », `user-manual.tex:1234`) n'est pas réécrit (AC8 : #473, #525) ; la brochure n'est
  pas commitée (régénérée par `make fr`, sans changement de source).
- **Réversibilité** : totale (texte).

## C-15-5d-5 — 15-5d (revue P1) : le plan de l'accesseur épinglé par `FORCE INDEX (PRIMARY)`

- **Contexte** : finding B-1 (LOW) de la revue de code P1 — l'absence de verrous d'intervalle de la requête
  verrouillante de `lock_designated_accounts_in_tx` reposait sur un plan `range`/`const` sur `PRIMARY` **mesuré**, que
  rien ne garantissait sur une table réelle (index secondaires `uq_accounts_company_number` et
  `uq_accounts_company_singleton_role`, tous deux préfixés par `company_id`).
- **Retenu** : `FROM accounts FORCE INDEX (PRIMARY) WHERE company_id = ? AND id IN (…) ORDER BY id LOCK IN SHARE MODE`
  sur la seule requête **verrouillante** (la lecture non verrouillante des identifiants de la société n'en a pas
  besoin : elle ne pose aucun verrou). `EXPLAIN` relevé sur `kesh_155d` : `range` sur `PRIMARY`, *Using where*, deux
  identifiants ; `const` pour un seul. Les motifs `ACCESSEUR` des tests de place 1 et 2 suivent le texte de la requête
  (sans cela, ils attendraient en vain et paniqueraient).
- **Écarté** : un test qui épinglerait le plan par `EXPLAIN` (le plan dépend des statistiques de la base de test, à
  cinq comptes ; l'indice le fixe à la source) ; laisser le risque écrit seulement.
- **Ce que les tests voient, et ce qu'ils ne voient pas** : la suppression de l'indice fait rougir les tests de place
  1 et 2 — parce que leur motif ne reconnaît plus la requête, **non** parce qu'un verrou d'intervalle apparaîtrait.
  L'effet sur le plan n'est établi que par l'`EXPLAIN`.
- **Réversibilité** : totale (une clause SQL, deux constantes de test).

## C-15-5d-6 — 15-5d (revue P1) : les LOW acceptés, et ce qui reste écrit comme angle mort

- **Contexte** : revue de code P1, Sonnet ×3 — B 3 LOW, E 1 MEDIUM et 5 LOW, A 2 LOW. E1 (MEDIUM), B-1, B-2, B-3, A-1
  et la moitié de E2 sont corrigés (Change Log de la fiche).
- **Retenu, sans correction** :
  - **E2, reste** : côté achat, ni test « archivé », ni « compte étranger », ni « deux rôles ». Accepté : l'accesseur
    et `check_written` sont **communs** aux deux flux et couverts côté vente (archivé, étranger avec sonde, deux rôles,
    priorité en mélange) ; ce que le site d'achat a en propre — ses candidats (`DesignatedRole::PURCHASE`), sa place,
    son mode — est couvert par les tests créanciers, TVA récupérable, place 2 et le **test de mode d'achat** ajouté.
  - **E3** : la garantie « aucun archivage entre le contrôle et l'insertion » n'est testée que pour `postable`.
    Accepté : `active` et `postable` sont lus par la **même** lecture verrouillante de la même ligne ; un `UPDATE` de
    l'un ou l'autre prend le même verrou exclusif de ligne. Le test de place 1 exerce ce chemin.
  - **E4** : la lecture non verrouillante des identifiants de la société (patron `owned_account_ids`) se fait dans
    l'instantané REPEATABLE READ de l'appelant. Un compte **créé et désigné** par un `PUT` des réglages après
    l'ouverture de cet instantané, mais avant le verrou des réglages de l'appelant, en serait absent : refus
    `InactiveOrInvalidAccounts` d'un compte valide. **Angle mort écrit** : refus sûr (rien n'est écrit), réessayable,
    fenêtre de l'ordre de la milliseconde. Ajouté au doc-comment de l'accesseur.
  - **E5** : la complétion d'une facture importée (`routes/imported_supplier_invoices.rs`) prend désormais les
    verrous partagés de l'accesseur sans être rejouée sur interblocage. **Dépendance écrite** : son rejeu relève de la
    **15-5e2** (rollout du rejeu, closes #536 #484), déjà prévu par la fiche de la 15-5e1 et la phrase des cycles du
    doc-comment de `supplier_invoices::create_in_tx`. Rien à faire dans la 15-5d.
  - **E6** : aucun test ne rougit si `ORDER BY id` disparaît. Accepté : deux verrous partagés sont compatibles,
    l'ordre est ici d'hygiène.
  - **A-2** : le test de l'avoir exempté vit dans `invoices_validate_vat.rs` et non `credit_notes_repository.rs` ;
    écart déjà déclaré (C-15-5d-3).
  - **B-3** : la disjonction par type n'est pas contrôlée après la désignation. Les doc-comments disent désormais
    pourquoi la vente n'a pas de cycle (deux partagés, quel que soit le type) et, côté achat, écrivent le cycle étroit
    d'un compte désigné retypé en charge comme angle mort couvert par le rejeu de la route.
- **E1, ce qui a été corrigé et ce qui ne l'était pas** : l'encadré *Rôles des comptes* (`user-manual.tex`, « Deux
  exceptions, voulues ») disait déjà l'exemption de l'avoir ; le paragraphe de la validation, non. Une phrase y est
  ajoutée, avec la raison (« une facture émise doit rester annulable ») et le renvoi.
- **Réversibilité** : totale.

## C-15-5d-7 — 15-5d (intégration) : rebasée sur `origin/main` (`9cb5083b`, 15-5e2) ; « 5 bis » fusionné, fiche de la branche retenue

- **Contexte** : `origin/main` porte la 15-5e2 (rejeu des douze autres routes, commentaires d'ordre, Pattern 5,
  `api-external.md` § 10, CHANGELOG, manuels #484). Rebase des huit commits de la 15-5d.
- **Option retenue** : (1) fiche de la story — la version de `main` (`41f41e60`, validation P5) est un ancêtre de
  celle de la branche : version de la branche prise entière ; (2) registre et `sprint-status.yaml` — union, la ligne
  `last_updated` de la 15-5d renumérotée (19) au-dessus de celle de la 15-5e2 (18) ; (3) commentaire « 5 bis » — la
  formulation de la 15-5e2 (« enveloppe `retry_on_deadlock` ») gardée et le paragraphe de la 15-5d (verrou partagé
  de la TVA due à la validation, même ordre « arrondi, puis TVA due ») placé à sa suite ; (4) PDF — régénérés sur
  l'état rebasé, jamais fusionnés. Les doc-comments canoniques, le CHANGELOG, `api-external.md` et les `.tex` ont
  fusionné sans conflit et ont été relus ; aucune mention `retry_with` ajoutée par la story.
- **Écartées** : reprendre la fiche de `main` et y rejouer les passes P6–P7 (perte du texte validé) ; garder
  `retry_with` dans « 5 bis » (contredit la migration de la 15-5e2).
- **Réversible** : oui (rebase local, branche poussée seulement après les gates).

## C-15-5d-8 — 15-5d (intégration) : rebasée sur `origin/main` (`8f9811d8`, 15-11a) ; registre et sprint-status par union, PDF régénérés

- **Contexte** : `origin/main` porte la 15-11a (compose, `config.rs` qui refuse les gabarits de secrets, manuel
  d'administration remis en page, CHANGELOG avec rubrique Sécurité). PR #565 en conflit.
- **Option retenue** : registre — union par ordre d'arrivée (`C-15-11a-1..7` de `main`, puis `C88` et
  `C-15-5d-1..7`), contrôlée par comparaison des titres `## ` aux deux bornes ; `sprint-status.yaml` — union des
  lignes `last_updated`, celles de la 15-5d renumérotées (21) et (22) ; PDF — régénérés par `make admin user` après
  `touch` des `.tex`, jamais fusionnés. CHANGELOG et `.tex` ont fusionné sans conflit et ont été relus (une rubrique
  de chaque, mise en page des tableaux de la 15-11a intacte). Montage E2E à secrets générés par `openssl rand`.
- **Écartées** : renuméroter les entrées de la 15-5d (la fiche les cite) ; garder les numéros (19)/(20) de la
  branche dans `sprint-status.yaml` (doublons avec ceux de la 15-11a).
- **Réversible** : oui (rebase ; poussé par `--force-with-lease` après les gates).

## C-15-7-1 — 15-7 : pas de découpage préventif

- **Contexte** : la règle de splitting préventif du `CLAUDE.md` découpe au-delà de cinq modules
  de premier niveau. La 15-7 touche `kesh-api` (routes d'onboarding), `kesh-api` (libellés
  d'audit et registre des routes), `kesh-db` (variantes `_in_tx` de cinq repositories),
  `kesh-seed` (peuplement et remise à zéro) et `kesh-i18n` (quatre catalogues) — **cinq**,
  comptés comme la 15-5a les a comptés (un crate de persistance = un module, quel que soit le
  nombre de ses repositories). Manuel et CHANGELOG ne sont pas des modules.
- **Retenu** : une story unique. Les onze routes forment **une seule famille** (un fichier,
  un parcours) ; c'est précisément le nombre de familles qui avait fait écarter #434 de la 25-1b.
- **Écarté** : découper en « parcours production » / « démonstration et remise à zéro ». Le
  seuil n'est pas franchi, et la remise à zéro partage avec les autres routes le patron qu'elle
  devrait autrement réinventer.
- **Réversible** : oui — si la validation P1 compte six modules, le découpage naturel est
  celui qui vient d'être écarté (la 15-7b prendrait `kesh-seed`).

## C-15-7-2 — 15-7 : chaque étape franchie s'inscrit (`installation.step_completed`)

- **Contexte** : trois routes ne touchent que `onboarding_state` (`start-production`,
  `skip-bank`, et `language`/`org-type` quand la valeur ne change pas). Le registre n'offre que
  `Traced`, `Exempt("issue #…")` et `NoMatter` — défini comme « ne mute rien », ce qui serait
  faux pour une route qui fait avancer l'assistant.
- **Retenu** : toute route d'onboarding qui réussit écrit **au moins** une entrée,
  `installation.step_completed` (`entity_type = "installation"`, `entity_id = 0`,
  `details = {from, to, step}`), dans la transaction qui fait avancer l'étape ; les faits de
  domaine (société, plan comptable, compte bancaire, réglages, taux) ont **en plus** leur
  propre entrée. Les onze routes deviennent `Traced` sans changer la sémantique du registre.
- **Écarté** : (a) `NoMatter` pour `skip-bank` et `start-production` — il aurait fallu
  redéfinir `NoMatter`, et « passer en production » est justement le geste qui rend
  l'installation non réinitialisable ; (b) une action par étape (`installation.production_started`,
  `installation.bank_skipped`…) — huit libellés ×4 locales pour une information que
  `details.step` porte déjà.
- **Réversible** : oui, à bas coût (une action, quatre libellés).

## C-15-7-3 — 15-7 : le plan comptable s'inscrit en UNE entrée agrégée

- **Contexte** : `bulk_create_from_chart` crée 83 à 86 comptes selon le plan livré.
- **Retenu** : une entrée `account.chart_loaded` (`entity_type = "account"`, `entity_id = 0`),
  `details` = plan (`org_type`), langue, nombre, et la liste `[{id, number}]` des comptes créés.
- **Écarté** : ~85 entrées `account.created` — exactes entité par entité, mais elles noient le
  début du journal et la création d'un compte par l'utilisateur ne se distingue plus de celle
  du plan livré.
- **Coût du retour** : le filtre « numéro d'entité » de l'écran ne retrouve pas la création d'un
  compte du plan livré ; il faut chercher l'entrée agrégée. Assumé, écrit dans le manuel.

## C-15-7-4 — 15-7 : la démonstration s'inscrit par une entrée de synthèse

- **Contexte** : `kesh_seed::seed_demo` enchaîne cinq transactions (société, plan, exercice,
  taux, réglages) puis l'étape ; le rendre atomique est KF-002-H-002 (#43), hors périmètre.
- **Retenu** : `installation.demo_seeded` (+ `installation.step_completed` 2→3), écrites dans
  la **dernière** transaction, celle de l'étape, avec les identifiants et décomptes de ce qui a
  été créé. Un peuplement interrompu ne laisse pas de trace — il laisse les orphelins que le
  code documente déjà, et que la remise à zéro efface.
- **Écarté** : une entrée par fait de domaine (`company.updated`, `account.chart_loaded`,
  `fiscal_year.created`, `vat_rate.created`…) pour des données de démonstration, qui
  n'appartiennent à aucune comptabilité réelle.
- **Réversible** : oui.

## C-15-7-5 — 15-7 : la remise à zéro efface la piste ET y inscrit son propre geste

- **Contexte** : `reset_demo` exécute `DELETE FROM audit_log` (#279) sur une installation non
  finalisée, en `autocommit`, sans aucune trace.
- **Retenu** : les `DELETE` et la réinitialisation d'`onboarding_state` passent dans **une**
  transaction, qui écrit en dernier `installation.reset` — première entrée du journal neuf —,
  avec le nombre d'entrées effacées et la plage d'identifiants `[min, max]` qu'elles occupaient
  (l'auto-incrément n'étant pas remis à zéro, le trou dans la numérotation s'explique par
  l'entrée qui le suit). La transaction relit `onboarding_state` `FOR UPDATE` pour les détails
  et **refuse** si `step_completed >= 7` — ce qui ferme au passage la fenêtre résiduelle notée
  KF-002-H-002 sur ce chemin.
- **Écarté** : (a) l'exemption, « c'est de la démo » — effacer la piste sans le dire est
  exactement ce que la piste existe pour empêcher ; (b) conserver les entrées antérieures — la
  remise à zéro sert à repartir de rien, et l'installation n'est pas finalisée.
- **Réversible** : oui.

## C-15-7-6 — 15-7 : une transaction par route, extraite en variantes `_in_tx`

- **Retenu** : chaque route mène UNE transaction — mutation de domaine, avancée d'étape
  (`update_step_in_tx`) et entrées d'audit —, sur le patron de la 25-1b (AC 9) : extraire un
  `_in_tx` là où le repository commite seul, garder l'enveloppe pool pour ses autres appelants.
  Effet de bord voulu : une société modifiée dont l'étape n'avance pas (conflit de version) n'est
  plus commitée à moitié.
- **Écarté** : une transaction d'audit séparée après la mutation (trace non atomique).
- **Réversible** : non sans perdre l'atomicité ; c'est la contrainte de la piste.

## C-15-7-7 — 15-7 : finalisation — réglages et taux nommés par ce qui a réellement été inséré

- **Retenu** : une action neuve `company_invoice_settings.created` si la ligne a été insérée
  (pas si elle préexistait — `INSERT IGNORE`) ; une entrée `vat_rate.created` **par taux
  réellement inséré**, au format de `routes/vat.rs`. `insert_with_defaults_in_tx` et
  `seed_default_swiss_rates_in_tx` rendent désormais ce qu'ils ont inséré.
- **Laissé tel quel** : `fiscal_year.created`, déjà écrit par `create_if_absent_in_tx` avec
  `NewAuditLogEntry::user` — l'attribution par clé d'API y est la dette #431, hors périmètre.
- **Réversible** : oui.

## C-15-7-8 — 15-7 : découpage en 15-7a / 15-7b (renverse C-15-7-1)

- **Contexte** : passe de validation P1, findings R7 et F-5 — le décompte « cinq modules » de
  C-15-7-1 comptait `kesh-api` deux fois et `kesh-db` une fois pour cinq repositories ; il donne
  quatre par crate, neuf à la granularité des exemples de la règle (`kesh-api/routes/invoices`).
- **Retenu** (décision de l'orchestrateur) : granularité des exemples, donc seuil franchi, donc
  découpage — **15-7a-trace-installation-production** (les neuf routes de production, registre,
  libellés, manuel ; `refs #434`) puis **15-7b-trace-demo-et-remise-a-zero** (`seed-demo`, `reset`,
  `kesh-seed` ; `closes #434`, la dernière). La 15-7 devient l'index (`split`).
- **Écarté** : maintenir la story unique en écrivant une convention de comptage à la mesure du
  résultat voulu — c'est le défaut que la passe a relevé.
- **Réversible** : oui avant tout développement.

## C-15-7-9 — 15-7b : #528 — la remise à zéro préserve l'identité de la société

- **Contexte** : #528 (ouverte par l'orchestrateur après la P1) — `reset_demo` efface `companies`
  sous `FOREIGN_KEY_CHECKS=0`, rien ne repointe `users.company_id`, le JWT désigne une société morte
  et toute route scopée répond 500. Attendu de l'issue : rattacher l'utilisateur à la société
  recréée, **ou** préserver l'identité de la société.
- **Retenu** : préserver l'identité. La société (exactement une, verrouillée) est remise **en place**
  à l'état stub du premier démarrage (`companies::reset_to_stub_in_tx`, mêmes valeurs que
  `insert_stub_company`, constantes descendues dans `kesh-db`) ; `users` et `api_keys` dont le
  `company_id` diffère y sont rattachés (ce qui répare aussi une installation déjà atteinte).
  Le JWT en cours reste valide sans reconnexion.
- **Écarté** : rattacher les utilisateurs à la société recréée par `set_language` — le JWT en vol
  garde l'ancien id jusqu'au rafraîchissement (500 entre-temps), et il faudrait un rattachement dans
  `set_language`, hors du geste qui casse.
- **Conséquence nécessaire** : C-15-7-10.
- **Réversible** : oui.

## C-15-7-10 — 15-7b : #279 fermée — vidage dérivé de la liste canonique

- **Contexte** : préserver l'id de la société (C-15-7-9) **sans** vider toutes ses tables rattacherait
  à la société remise à zéro les factures, contacts, taux et réglages de la démonstration, et
  `finalize` trouverait des réglages de facturation pointant des comptes effacés. #279 demandait
  justement l'exhaustivité, en recommandant la liste canonique.
- **Retenu** : `RESET_PRESERVED_TABLES` (`api_keys`, `companies`, `onboarding_state`,
  `password_reset_tokens`, `refresh_tokens`, `users`) ; toutes les autres tables de
  `kesh_db::backup::TABLES_TO_TRUNCATE` sont vidées dans l'ordre de la liste. Un test de partition
  garde la dérivation. `closes #279` sur la PR de la 15-7b.
- **Écarté** : compléter la liste en dur (option 2 de #279) — elle dériverait comme celle d'aujourd'hui.
- **Réversible** : oui.

## C-15-7-11 — 15-7b : les trois gardes de la remise à zéro vivent sous le verrou de l'effacement

- **Contexte** : F-1 — le handler applique trois gardes (`step >= 7`, `!is_demo && step > 2`,
  `KESH_PRODUCTION_RESET`) puis relâche son verrou avant `reset_demo`.
- **Retenu** : une seule copie des gardes, dans `reset_demo`, évaluées sur `onboarding_state`
  verrouillé **par la transaction qui efface** ; le handler lit le drapeau d'environnement une fois et
  le passe, n'ouvre plus de transaction, mappe les erreurs (400 et 403 d'aujourd'hui).
- **Écarté** : garder les gardes au handler **et** les rejouer dans `reset_demo` — deux copies qui
  peuvent diverger, pour aucun gain : la seconde est la seule qui compte.
- **Réversible** : oui.

## C-15-7-12 — 15-7 : ordre des verrous Pattern 5 et étape revérifiée sous verrou

- **Contexte** : R3/F-3 — fusionner mutation et étape dans une transaction verrouillerait
  `companies` avant `onboarding_state`, à l'inverse de `finalize` ; l'étape est lue hors verrou ;
  `count_by_company` (garde du plan comptable) est lu hors transaction.
- **Retenu** : chaque transaction d'onboarding ouvre par `onboarding_state FOR UPDATE` et y revérifie
  l'étape (helper `lock_state_at_step`), puis `companies`, puis `accounts` / `bank_accounts` ; le
  comptage des comptes se fait dans la transaction. Même ordre pour la dernière transaction de
  `seed_demo` et pour `reset_demo`. La table du Pattern 5 est mise à jour.
- **Réversible** : non sans rouvrir les cycles de verrous.

## C-15-7-13 — 15-7 : l'atomicité se prouve par un déclencheur SQL de test

- **Contexte** : R1/F-2 — un conflit de version « posé avant l'appel » est relu par la route et ne
  produit aucun échec.
- **Retenu** : le test pose `CREATE TRIGGER … BEFORE INSERT ON audit_log … SIGNAL SQLSTATE '45000'`
  (l'insertion d'audit vient **après** les mutations de domaine), asserte que rien n'est commité, puis
  `DROP TRIGGER` et rejoue avec succès. Absence du privilège `TRIGGER` ⇒ le test échoue, il ne se
  saute pas.
- **Écarté** : deux requêtes concurrentes (non déterministe) ; un crochet d'injection dans le code de
  production.
- **Réversible** : oui.

## C-15-7-14 — 15-7b : la dernière transaction de `seed_demo`, et la boucle de retry

- **Contexte** : R2/F-4 — `demo_seeded` doit compter les taux et réglages insérés, que les enveloppes
  pool ne rendent pas ; la boucle de retry existe pour une visibilité entre transactions.
- **Retenu** : la dernière transaction appelle `seed_default_swiss_rates_in_tx` et
  `insert_with_defaults_in_tx` (posées par la 15-7a), y reçoit aussi l'effacement d'`is_stub`
  (retiré du handler), l'étape et les deux entrées ; la boucle (trois essais, 50 ms, sur
  `InactiveOrInvalidAccounts`) est **conservée** et enveloppe toute cette transaction, rollback
  explicite entre deux essais. `seed_demo` reçoit l'acteur et ne reçoit plus `onboarding_version`.
- **Écarté** : supprimer la boucle au motif qu'une transaction ouverte après le commit du plan voit
  les comptes — vraisemblable, non mesuré ; la garder coûte une enveloppe.
- **Réversible** : oui.

## C-15-7-15 — 15-7a : helpers société sur `companies::update_in_tx`, helper d'étape dans `kesh-db`

- **Contexte** : F-12 — trois `UPDATE` bruts à doter d'une comparaison, alors que
  `companies::update_in_tx` court-circuite déjà le no-op (KF-004) ; l'entrée d'étape s'écrirait à onze
  sites, dont un dans `kesh-seed`, qui ne peut pas employer un helper de `kesh-api`.
- **Retenu** : `CompanyUpdate` reconstruit depuis la ligne verrouillée, `update_in_tx`, trace si la
  version change ; `is_stub` par `companies::clear_stub_in_tx` (servira aussi `seed_demo`) ;
  `onboarding::record_step_completed_in_tx` dans `kesh-db`, action en littéral (forme `for_actor`
  reconnue par la garde des libellés). `insert_with_defaults` (pool) appelle sa variante `_in_tx`.
- **Réversible** : oui.

## C-15-7-16 — 15-7b : `onboarding_state` remis à zéro en place

- **Retenu** : `onboarding::reset_state_in_tx` — `UPDATE` de la ligne déjà verrouillée (étape 0,
  `is_demo` faux, `ui_mode` nul, `version + 1`), plutôt que `DELETE` + `INSERT` dans la transaction.
- **Écarté** : variantes `_in_tx` de `delete_state` / `init_state` — un `DELETE` de la ligne qu'on
  tient, suivi d'un `INSERT` qui change son id, pour le même état final.
- **Réversible** : oui.

## C-15-7-17 — 15-7a : taux TVA insérés ligne à ligne

- **Contexte** : F-13 — l'`INSERT IGNORE` multi-lignes ne dit pas quels taux il a insérés ;
  `vat_rate.created` porte l'id du taux.
- **Retenu** : quatre `INSERT IGNORE` d'une ligne ; inséré ssi `rows_affected == 1`, id par
  `last_insert_id`. L'assertion `count >= 4` reste.
- **Écarté** : pré-lire les taux existants sous verrou puis insérer — deux requêtes pour une.
- **Réversible** : oui.

## C-15-7-18 — 15-7 : ordre des entrées au sein d'une route

- **Contexte** : R6 — les tests assertent la séquence exacte ; l'ordre n'était pas fixé.
- **Retenu** : entrées de domaine dans l'ordre d'exécution des mutations, `installation.step_completed`
  en dernier ; pour `finalize` : réglages, taux (ordre du seed), exercice, étape. `company.created`
  porte `entity_id` = id inséré, `details = {instance_language, is_stub: true}`.
- **Réversible** : oui (tests à réordonner).

## C-15-7-19 — 15-7a : découpage en 15-7a1 (socle) / 15-7a2 (routes et audit)

- **Contexte** : validation P2 de la 15-7a, findings R2-3 et F-1 — à la granularité que C-15-7-8 a
  retenue, la 15-7a touchait encore neuf modules (six repositories `kesh-db`, routes d'onboarding,
  libellés d'audit, `kesh-i18n`) sans section de dérogation ; la seule dérogation codifiée est le cycle
  de dépendance Cargo, absent.
- **Retenu** (décision de l'orchestrateur) : patron « story-zéro + rollout » du `CLAUDE.md` —
  **15-7a1-socle-transactions-onboarding** (variantes `_in_tx`, `clear_stub_in_tx`, `lock_state_in_tx`,
  sans changement de comportement ni d'audit, tests de non-régression) puis
  **15-7a2-trace-installation-production** (routes, audit, registre, libellés, manuel). 15-7a2 dépend
  de 15-7a1 ; 15-7b de 15-7a2. La 15-7a devient une fiche `split` à corps vidé.
- **Décomptes déclarés** (signal D5) : 15-7a1 sept modules — rollout mécanique, revu fichier par
  fichier ; 15-7a2 quatre ; 15-7b huit, non découpée.
- **Écarté** : écrire une dérogation pour garder la 15-7a entière — aucun motif codifié ne la porte.
- **Réversible** : oui avant tout développement.

## C-15-7-20 — 15-7a1/15-7a2 : où vivent `lock_state_*` et `record_step_completed_in_tx`

- **Contexte** : la décision de découpage plaçait `record_step_completed_in_tx` et `lock_state_at_step`
  dans le socle. Deux contraintes mécaniques l'interdisent : (1) la garde bilatérale
  `audit_label_registry.rs` exige qu'une action écrite par un site de production soit déclarée dans
  `ACTIONS` et libellée dans les quatre `.ftl` — le helper d'étape tirerait donc le travail d'audit
  dans la 15-7a1, qui doit en être exempte ; (2) `lock_state_at_step` est une fonction privée de
  `kesh-api` : sans appelant, `clippy -D warnings` échoue sur `dead_code`.
- **Retenu** : la primitive `onboarding::lock_state_in_tx` (verrou seul, rend `Option`) dans `kesh-db`
  à la 15-7a1 — publique, servira aussi `kesh-seed` (finding L-4 de la 15-7b) ; `lock_state_at_step`
  (comparaison d'étape, l'appelant annule) et `record_step_completed_in_tx` à la 15-7a2.
- **Écarté** : libeller `installation.step_completed` dès la 15-7a1 (audit dans le socle) ;
  `#[allow(dead_code)]` provisoire (une exemption qui survit à son motif).
- **Réversible** : oui.

## C-15-7-21 — 15-7a1/15-7a2 : `is_stub` dans la trace de `coordinates`

- **Contexte** : R2-2 / F-5 — l'AC 3 disait « `company.updated` ssi la version change » puis « le
  changement d'`is_stub` seul suffit » ; `companies::update_in_tx` n'écrit pas `is_stub`, et l'effet de
  `clear_stub_in_tx` sur `version` n'était pas fixé.
- **Retenu** : `clear_stub_in_tx` = `UPDATE companies SET is_stub = FALSE, version = version + 1 WHERE
  id = ? AND is_stub = TRUE`, rend `rows_affected == 1` ; `company.updated` ssi (version rendue par
  `update_in_tx` ≠ version verrouillée) **ou** stub levé ; `after` lu par relecture après les deux
  écritures. Coordonnées changées sur stub ⇒ `version + 2` ; identiques sur stub ⇒ `+1`, une entrée.
- **Écarté** : faire écrire `is_stub` par `update_in_tx` (modifie le contrat KF-004 de toutes les
  routes société pour un seul appelant).
- **Réversible** : oui.

## C-15-7-22 — 15-7b : ordre des verrous de la remise à zéro (amende C-15-7-12)

- **Contexte** : R-2 / F-3 — C-15-7-12 annonçait « même ordre » pour `reset_demo`. Or le vidage suit
  `TABLES_TO_TRUNCATE` (enfants → parents), qui inverse le Pattern 5 après les deux premiers verrous ;
  un interblocage est possible avec une transaction qui ne prend pas `companies`
  (`invoices::validate_invoice`).
- **Retenu** : « même ordre » ne vaut que pour `onboarding_state` puis `companies`. `reset_demo` est
  inscrite à la liste d'exceptions du Pattern 5 (`docs/MULTI-TENANT-SCOPING-PATTERNS.md`, « Deny
  list ») avec sa justification ; **et elle est rejouée sur interblocage** par
  `kesh_db::retry::retry_with` (1213, trois essais) — chaque essai acquiert sa connexion, ouvre et ferme
  sa transaction. Motif du rejeu : le corps est rejouable de bout en bout (gardes réévaluées), la
  victime est annulée en entier, et le précédent `finalize` / `opening_balances` / `reconciliation`
  coûte une enveloppe ; sans lui, une collision rend 500 à l'administrateur.
- **Limite** : le rejeu n'est pas exercé par un test (interblocage non déterministe).
- **Écarté** : vider dans l'ordre du Pattern 5 — il faudrait une seconde liste ordonnée à maintenir à
  côté de la canonique, ce que C-15-7-10 écarte ; laisser la victime rendre 500 sans rejeu.
- **Réversible** : oui.

## C-15-7-23 — 15-7b : #528 sur une installation déjà sans société

- **Contexte** : F-1 (+ R-8) — une installation atteinte sous v0.12.x se trouve **sans société** après
  la remise à zéro ; une nouvelle remise à zéro rendait 500 (« exactement une »), et `language` recrée
  une société par la branche « aucune société » sans rattacher personne.
- **Retenu** : un helper unique `companies::attach_all_principals_in_tx` (utilisateurs et clés d'API
  rattachés à la société courante, rend les deux comptes), appelé par `reset_demo` **et** par la branche
  « aucune société » d'`ensure_company_with_language` ; un seul site d'insertion du stub,
  `companies::insert_stub` (générique sur l'exécuteur), employé par le bootstrap, cette branche et
  `reset_demo` — qui **insère** le stub quand il n'y a aucune société (`Invariant` seulement au-delà
  d'une). `company.created` et `installation.reset` portent les comptes de rattachement.
- **Écarté** : rattacher seulement dans `reset_demo` (laisse la branche reproduire le défaut).
- **Réversible** : oui.

## C-15-7-24 — 15-7b : la partition des tables se garde sur le schéma

- **Contexte** : F-2 — `RESET_CLEARED_TABLES` étant dérivé, le test de partition est vert par
  construction ; une future table enfant de `users` seulement serait vidée en silence.
- **Retenu** : test `#[sqlx::test]` sur `information_schema.KEY_COLUMN_USAGE` — toute table vidée atteint
  `companies` par clés, sauf une liste **fermée** d'exceptions (`audit_log`, sans FK sur `company_id`)
  dont chaque membre est vérifié ne pas l'atteindre ; toute table qui référence `users` sans atteindre
  `companies` est préservée. Recompté sur le squash : 32 des 33 tables vidées atteignent `companies`.
  `RESET_*` vivent dans `kesh_db::backup` (R-10).
- **Écarté** : admettre « porte une colonne `company_id` » comme critère (une colonne n'est pas une
  clé : rien ne la garde).
- **Réversible** : oui.

## C-15-7-25 — 15-7b : contrôles FK rétablis avant le commit ; 15-7a2 et 15-7b dans la même release

- **Contexte** : R-6 — le cas « transaction validée puis `SET FOREIGN_KEY_CHECKS=1` en échec » était
  indécis ; F-11 (15-7a) — entre la 15-7a2 et la 15-7b, une remise à zéro à l'étape ≤ 2 efface les
  entrées neuves sans trace.
- **Retenu** : sur le patron de `restore_tables_in_tx`, `SET FOREIGN_KEY_CHECKS=1` **avant** le
  `COMMIT` ; un échec du `SET` fait annuler et rend l'erreur — le cas indécis n'existe plus ; la
  connexion est détachée si le rétablissement échoue sur le chemin d'erreur. La 15-7a2 et la 15-7b
  partent dans la **même** release (0.13.0).
- **Écarté** : rendre `Ok` après un commit suivi d'un `SET` en échec (un succès qui laisse une
  connexion douteuse) ; documenter au manuel la fenêtre entre les deux stories.
- **Réversible** : oui.

## C-15-7-26 — 15-7a1 : le test 8 asserte l'état du parcours de production, et la séquence d'audit vaut `["fiscal_year.created"]`, non zéro

- **Contexte** : R1/F-1 de la P1 — le test 8 (« séquence d'audit relevée avant modification »)
  affirmait `[] == []` dans un fichier (`onboarding_e2e.rs`) qui n'emprunte pas le code touché. La
  consigne de l'orchestrateur demandait « `audit_log` à zéro déclaré » ; or `finalize` écrit déjà
  `fiscal_year.created` par `fiscal_years::create_if_absent_in_tx` (`fiscal_years.rs:316`), vérifié au
  code et conforme à l'issue #434 (« seul `fiscal_year.created` apparaît »). Un « zéro » serait rouge
  dès avant la story.
- **Retenu** : assertions **ajoutées** à `onboarding_path_b_e2e.rs::full_path_b_flow` — quatre taux,
  une ligne de réglages, séquence d'audit **exactement** `["fiscal_year.created"]`, c'est-à-dire zéro
  entrée de toute autre action — ; mutation : retirer `insert_with_defaults_in_tx` de `finalize`. Les
  enveloppes pool sont fixées branche par branche, et le rollback best-effort sur sortie d'erreur est
  écrit comme **exception assumée** au « sans changement de comportement ».
- **Écarté** : `COUNT(*) = 0` (faux sur le code actuel) ; retirer le test 8 sans remplacement (laisse
  l'AC 1 sans preuve sur le chemin de production).
- **Réversible** : oui (texte de fiche).

## C-15-7-27 — 15-7b : le prédicat de rejeu de la remise à zéro est gardé par le type, et testé sur un vrai 1213

- **Contexte** : R-2/F-1 de la P3 — `is_deadlock_error` ne voit que `DbError::Sqlx` ; `SeedError` a un
  `From<sqlx::Error>`, si bien qu'un `?` brut rend `SeedError::Sqlx` et que le rejeu de C-15-7-22
  serait du code mort, sans test pour le révéler.
- **Retenu** : le corps d'un essai rend `ResetAttemptError { Db(DbError), AlreadyFinalized,
  ResetForbidden }`, **sans** `From<sqlx::Error>` — toute erreur sqlx (begin et commit compris) passe
  par `map_db_error` ou ne compile pas ; prédicat `is_reset_retryable` (`#[doc(hidden)] pub`) ; test 13
  qui provoque un vrai 1213 entre deux connexions dédiées (ordre de verrouillage croisé, victime
  désignée par InnoDB) et un 1205, avec mutations qui mordent dans les deux sens.
- **Écarté** : prédicat sur les deux variantes de `SeedError` (exige de rendre `is_deadlock_sqlx`
  publique et ne garde pas la conversion) ; retirer `SeedError::Sqlx` (touche tout `kesh-seed`) ;
  test de source interdisant `?` (fragile, contournable).
- **Réversible** : oui.

## C-15-7-28 — 15-7b : « rejoindre `companies` » sans traverser une table préservée ; valeurs du stub ; pas de découpage

- **Contexte** : R-3/F-3 de la P3 — par fermeture transitive ordinaire, `refresh_tokens → users →
  companies` « atteint » `companies` : la règle 2 de C-15-7-24 était vide et sa mutation ne rougissait
  pas. R-1/F-2 — `reset_to_stub_in_tx` écrivait `NULL` dans quatre colonnes `NOT NULL DEFAULT ''`.
  F-6 — huit modules sans section de dérogation.
- **Retenu** : (a) la chaîne ne traverse aucune table de `RESET_PRESERVED_TABLES` autre que
  `companies` ; recalculé sur le squash : 32/33 tables vidées la rejoignent, `audit_log` seule
  exception (aucune FK depuis `20260910000001`), règle 2 = `password_reset_tokens`, `refresh_tokens` ;
  la mutation `refresh_tokens` déplacée rougit réellement (amende C-15-7-24). (b) Le stub remis à zéro
  porte les défauts du schéma : `NULL` pour les sept nullables, `''` pour quatre `address_*`, `'CH'`
  pour `address_country` et `country` ; `insert_stub` reçoit `Language::Fr` dans `reset_demo`. (c) Pas
  de découpage : section « Dérogation règle de splitting » écrite (défauts de la P3 distincts de ceux de
  la P2 et d'origine, amendement D5) ; coupe Volet A / Volet B automatique si un défaut né d'une
  remédiation revient à sévérité égale.
- **Écarté** : exclure seulement `users` de la traversée (une future table préservée rouvrirait le
  trou) ; découper dès maintenant.
- **Réversible** : oui.

## C-15-7-29 — 15-7a2 : test 10 conservé avec sa mutation ; test à pool d'une connexion ; `bank_accounts` avant les réglages au Pattern 5

- **Contexte** : P3 de la 15-7a2, convergée (19 LOW bruts). F3-3 — test 10 vert avant et après ;
  F3-9 — aucun test ne révèle une lecture du pool laissée dans la transaction ; F3-10 — position de
  `bank_accounts` par rapport à `company_invoice_settings` non dite.
- **Retenu** : test 10 **conservé**, car il mord dès que la lecture non verrouillée disparaît (sans la
  comparaison de `lock_state_at_step`, la route rend 200) — mutation partagée avec le test 11 ;
  test 13 sur pool `max_connections(1)` ; `bank_accounts` inséré au Pattern 5 après `accounts` et avant
  `company_invoice_settings` (donnée avant réglages ; aucune route ne prend les deux).
- **Écarté** : retirer le test 10 ; laisser le test 13 optionnel.
- **Réversible** : oui.

## C-15-7-30 — 15-7a1 : le test 8 appelle `finalize`, la séquence d'audit commence par `user.created`, le verrou d'état a un SQL littéral

- **Contexte** : P2 de la 15-7a1. R-1 = F-1 (HIGH) : le test 8 que C-15-7-26 avait fixé visait
  `onboarding_path_b_e2e.rs::full_path_b_flow`, qui s'arrête à l'étape 7 et **n'appelle jamais
  `finalize`** — assertions rouges sur le code inchangé, mutation 8 sans prise. C'est un **recyclage**
  du défaut que C-15-7-26 corrigeait. R-2 = F-2 : la séquence « `["fiscal_year.created"]` exactement »
  est fausse — le montage passe par `ensure_admin_user` (cas `(0, true)`), qui écrit `user.created`
  (`auth/bootstrap.rs:148`). R-3 = F-3 : « `SELECT_SQL` suivie de `FOR UPDATE` » n'est pas la requête
  `WHERE singleton = TRUE FOR UPDATE` des trois copies. R-6/F-11 : aucun des deux appels de `finalize`
  n'a à être adapté.
- **Retenu** : **corrige C-15-7-26** (non réécrit). Le test 8 **ajoute** `POST /onboarding/finalize`
  (200, étape 8) à la fin de `full_path_b_flow` — seul test qui traverse alors les quatre fonctions
  touchées (`bulk_create_from_chart`, que `fiscal_years_e2e` ne traverse pas, `upsert_primary`,
  `insert_with_defaults_in_tx`, `seed_default_swiss_rates_in_tx`) ; mutation 8 prouvée à la lecture
  (aucun autre écrivain de `company_invoice_settings` sur ce parcours). Séquence exacte
  `["user.created", "fiscal_year.created"]`, chaque entrée nommée par son écrivain. Correction **de
  fait** à la 15-7a2 (consigne héritée et test 1 : `user.created` en tête ; montage du test 1 écrit),
  sans rouvrir sa validation. AC 7 : constante neuve `LOCK_SQL`, SQL littéral, `SELECT_SQL` non
  réutilisée. `routes/onboarding` non touché ⇒ **six** modules. Pas de découpage : le défaut recyclé
  tient dans le texte d'un test, et la fiche est déjà la story-zéro d'un découpage (C-15-7-19).
- **Écarté** : un test dédié qui traverserait les quatre fonctions (doublon de montage de
  `full_path_b_flow`) ; une séquence filtrée sur les actions d'onboarding (plus faible qu'une séquence
  exacte globale) ; découper la 15-7a1.
- **Réversible** : oui (texte de fiche).

## C-15-7-31 — Découpage de la 15-7b en 15-7b1 (Volet A, démonstration) et 15-7b2 (Volet B, remise à zéro)

- **Contexte** : P4 de la 15-7b. R4-2 : la dérogation au découpage écrite à la P3 (C-15-7-28, point c)
  affirmait que les MEDIUM de la P3 n'étaient nés d'aucun correctif de la P2 ; c'est faux pour trois sur
  quatre (vérifié par versions de la fiche aux commits `3846b206` et `3c82e58f`). R4-1 = F-1 : un MEDIUM
  né de la remédiation P3. La fiche prévoyait la coupe Volet A / Volet B « sans nouvel arbitrage » dans
  ce cas.
- **Retenu** : **corrige le point (c) de C-15-7-28** (non réécrit). 15-7b1 = AC 1 et la part
  « démonstration » des AC 7 à 12, tests 1, 2, 3, 11, `refs #434` ; 15-7b2 = AC 2 à 6 et le reste,
  tests 4 à 10b, 12, 13, `closes #434`, `closes #528`, `closes #279`. Numérotation des AC et des tests
  conservée (traçabilité des passes). Ordre 15-7b1 → 15-7b2 ; la release commune de C-15-7-25 couvre
  désormais 15-7a2, 15-7b1 et 15-7b2. La 15-7b2 garde huit modules : nouvelle dérogation (une seule
  transaction, #528 et #279 inséparables) avec **coupe prédéclarée** — story-zéro `kesh-db` sans
  appelant, puis `reset_demo`, handlers, bootstrap et audit — si un défaut recyclé revient, le constat
  étant vérifié par versions de la fiche. Fiche 15-7b vidée (`split`). Pour R4-8 : C-15-7-24 est amendé
  par C-15-7-28, ce que disent les fiches et l'index (le registre ne se réécrit pas).
- **Écarté** : garder la 15-7b entière avec une dérogation rectifiée (la clause de la fiche s'appliquait
  sans arbitrage) ; couper la 15-7b2 dès maintenant en story-zéro + rollout (aucun défaut ne le motive
  encore).
- **Réversible** : oui (deux fiches à refondre).

## C-15-7-32 — 15-7b2 : connexion fermée à la libération, règle 3 de la partition, colonnes du stub gardées par le schéma

- **Contexte** : P4 de la 15-7b. F-2 : une future abandonnée (déconnexion, arrêt, délai) rend la
  connexion au pool avec `FOREIGN_KEY_CHECKS=0`. F-3 : aucune garde contre une clé d'une table préservée
  vers une table vidée. F-4/R4-1 : le test 5 comparait la même énumération ouverte que
  `reset_to_stub_in_tx`, et sa ligne de référence insérée au montage faisait deux sociétés (500).
- **Retenu** : `conn.close_on_drop()` juste après chaque `acquire` de `reset_demo` (sqlx-core 0.8.6) :
  la connexion n'est jamais rendue, sur succès, erreur ou abandon ; le `SET` de rétablissement du chemin
  d'erreur et la branche `detach()` de C-15-7-25 sont **retirés** (sans objet ; `detach()` laissait
  dépasser `max_connections`) — le `SET FOREIGN_KEY_CHECKS=1` avant le `COMMIT` reste ; test 8 :
  `CONNECTION_ID()` différent après l'échec. La même faille de `restore_tables_in_tx` reste à **#540**.
  Règle 3 du test 10b (ensemble vide), mutation `invoices` préservée. Test 5 : inventaire
  `information_schema.COLUMNS` = colonnes perturbées au montage ∪ {`id`, `version`, `created_at`,
  `updated_at`}, comparaison de toutes les colonnes sauf ces quatre à une référence construite **après**
  la remise à zéro dans une transaction annulée.
- **Écarté** : `tokio::spawn` de l'appel dans le handler pour le rendre non annulable (ne couvre pas
  l'arrêt du serveur) ; garder `detach()` ; comparer aux seules valeurs de l'AC 4 (énumération ouverte).
- **Réversible** : oui.

## C-15-7-33 — 15-7a1 (P3) : `LOCK_SQL` publique, fichier de test neuf pour `accounts`, ordre d'appel du verrou d'état

- **Contexte** : P3 de la 15-7a1 (R3-1/F-1, R3-2, R3-3/F-4, F-2). Le test 7, binaire d'intégration
  externe, exécute `LOCK_SQL` ; les tests 1-2 n'avaient pas de fichier ; la garde `existing == 0` de la
  15-7a2 dépend de l'ordre des lectures sous REPEATABLE READ.
- **Retenu** : `pub const LOCK_SQL` (doc : partagée avec le test 7, pas une API à étendre) et une
  mutation 7b (« `lock_state_in_tx` lit `SELECT_SQL` ») qui distingue la fonction de la constante ;
  fichier neuf `crates/kesh-db/tests/accounts_repository.rs` en squash `"./test-schema"` ; doc-comments
  de `lock_state_in_tx` et `count_by_company` : verrou d'état **en premier** dans la transaction.
- **Écarté** : littéral recopié dans le test (dérive) ; tests 1-2 dans `mod tests` d'`accounts.rs`
  (`#[tokio::test]` sur la base de dev partagée, pollution type KF-039).
- **Réversible** : oui (visibilité réductible si un autre moyen de partage apparaît).

## C-15-7-34 — 15-7b2 ferme #542 : garde du bootstrap, et réparation des installations déjà touchées par la remise à zéro

- **Contexte** : P1 de la 15-7b2, R1 (HIGH) = F-1 : #542 (un stub inséré à chaque démarrage sans
  utilisateur ni variable d'administrateur) désigne la 15-7b2, qui la renvoyait hors périmètre (L-7 de la
  P4 de la 15-7b). F-3 : la garde n'agit que pour l'avenir ; les installations déjà atteintes ont
  plusieurs sociétés et la remise à zéro y rend 500. Décision de l'orchestrateur : la 15-7b2 ferme #542 et
  répare.
- **Retenu** : AC 13 — branche `(0, false)` d'`ensure_admin_user` gardée par `company_count == 0`
  (compteur déjà lu en tête) ; test 14 « deux démarrages sans utilisateur → une société ». Étape 2 de
  `reset_demo` : s'il y a plus d'une société, suppression des sociétés `is_stub = TRUE` sans utilisateur
  ni clé d'API rattachés ; si toutes le sont, `MIN(id)` est conservée (celle que `routes/setup.rs:130`
  rattache) ; ensuite, plus d'une société restante ⇒ `Invariant`. Nombre rendu
  (`stub_companies_removed`, `ResetOutcome` et `details`). Tests 6d (i)/(ii) et 6e. `closes #542` dans
  les fiches index.
- **Écarté** : (i) assumer et écrire « corriger à la main » (laisse des installations bloquées) ;
  supprimer **toute** société provisoire superflue même rattachée (perte de rattachement d'un
  utilisateur) ; conserver la plus grande (`MAX(id)`), incohérent avec `setup.rs`.
- **Réversible** : oui (le code de suppression est local à l'étape 2).

## C-15-7-35 — 15-7b1 (P1) : `ui_mode` relu sous verrou, déclencheur sélectif, rejeu impossible écrit et testé

- **Contexte** : P1 de la 15-7b1 (R-1/F-2, R-2, F-1, F-3/R-4, R-5, F-4/R-7, F-5/R-6).
- **Retenu** : `seed_demo(pool, locale, actor)` — `ui_mode` et version lus sur l'état verrouillé ;
  montage des tests 1-3 par `ensure_admin_user` (stub) avec `is_stub = TRUE` asserté avant ; test 2 par
  déclencheur **sélectif** sur `installation.step_completed` (variante sur `installation.demo_seeded`),
  posé après la montée à l'étape 2 ; résidu (comptes, exercice, société renommée) et rejeu en 500
  assertés, écrits dans la doc de `seed_demo`, renvoyés à #538 ; mutation `clear_stub_in_tx` sortie de
  la dernière transaction et garde de source (test 11 b) contre le retour de l'`UPDATE is_stub` du
  handler ; jeton `read-write` créé par JWT d'administrateur, helper remonté dans `tests/common/mod.rs`,
  `api_key.created` dans la séquence ; boucle de retry conservée, **non testée** (angle mort déclaré :
  sa suppression défait C-15-7-14, réservée à Guy).
- **Écarté** : déclencheur global (ne distingue pas l'écriture hors transaction) ; rejeu réussi après
  `DROP TRIGGER` (C-15-7-13 dans sa forme d'origine : impossible, inventaire § 2) ; supprimer la boucle.
- **Réversible** : oui.

## C-15-7-36 — 15-7b2 (P1) : `reset_cleared_tables()` en fonction, correspondance d'erreurs, gardes prouvées sous verrou

- **Contexte** : P1 de la 15-7b2 (R2, R3, R5, R6, R7, F-2).
- **Retenu** : `pub fn reset_cleared_tables() -> Vec<&'static str>` (une `const` ne filtre pas une
  `const`) ; `SeedError::Db(d)` ⇒ `AppError::Database(d)`, `Sqlx` ⇒ `Internal`, 400/403 inchangés ;
  erreur de rollback journalisée, jamais substituée (sinon un 1213 masqué n'est plus rejoué) ; test 7b
  déterministe (connexion A tient `LOCK_SQL`, attente bornée sur `INNODB_TRX`, A pose l'étape 7 ⇒
  `AlreadyFinalized`, tables intactes) ; test 7c (`None` ⇒ `Invariant`) ; garde `rows_affected` de
  l'étape 4 non testée (non provocable, angle mort déclaré) ; test 4 avec relevé avant et liste fermée
  des tables non amorcées.
- **Écarté** : `LazyLock` (sans gain pour un geste rare) ; `sleep` nu dans le test 7b.
- **Réversible** : oui.

## C-15-7-37 — 15-7b1 (P2) : la démonstration rejoue sur interblocage, son manuel dit ce qu'elle crée (#544)

- **Contexte** : P2 de la 15-7b1 (F2-1, F2-2, F2-3, F2-6, R2-3). Le manuel promettait une entrée
  d'audit absente après un échec, et décrivait une démonstration (contacts, produits, écritures) que
  `seed_demo` ne crée pas ; la dernière transaction ne rejouait pas sur 1213 alors que ses voisines
  (`finalize`, la remise à zéro) le font.
- **Retenu** : la 15-7b1 **ferme #544** (ligne `user-manual.tex:179-189` à l'AC 11, `closes #544` sur la
  PR) ; paragraphe du journal borné à « lorsqu'il aboutit ». La dernière transaction est enveloppée dans
  `retry_with` (décision C54 de l'epic : tout flux d'écriture rejoue) ; le corps d'un essai rend
  `SeedAttemptError { Db, StepAlreadyCompleted }` sans `From<sqlx::Error>` (patron C-15-7-27), prédicat
  `is_seed_retryable`, que la 15-7b2 étend et dont son test 13 fait la preuve. La boucle
  `InactiveOrInvalidAccounts` (C-15-7-14, arbitrage réservé à Guy) **n'est pas touchée** : elle reste
  autour du rejeu ; elle compte trois rejeux, soit quatre essais (C-15-7-14 disait « trois essais » :
  corrigé ici, le registre ne se réécrit pas). Ordre de la transaction aligné sur `finalize` (réglages
  puis taux). Le handler garde le repli `AppError::Internal` (500) pour toute erreur autre que
  `StepAlreadyCompleted`.
- **Écarté** : réaffecter la boucle `InactiveOrInvalidAccounts` au 1213 (défait C-15-7-14) ; renvoyer
  #544 à une issue de suite (la story est celle qui inscrit l'inventaire exact de la démonstration) ;
  reprendre la correspondance `Db ⇒ AppError::Database` de la 15-7b2 (rejeu du test 2 en 409).
- **Réversible** : oui.

## C-15-7-38 — 15-7b2 (P2) : #528 réparée au démarrage ; un type d'essai et un prédicat pour les deux flux

- **Contexte** : F-1 de la P2 — le parcours démo → remise à zéro (v0.12.x) → `language` → production
  → `finalize` passe et laisse une installation finalisée à une société, `users.company_id` mort, que
  ni la remise à zéro (refusée à l'étape ≥ 7) ni `language` ne réparent. R2-6 : `AlreadyFinalized`
  doublait `StepAlreadyCompleted` (même 400), et la correspondance « complète » l'omettait.
- **Retenu** : `repair_orphan_principals` dans `ensure_admin_user`, avant la répartition par cas, en
  une transaction (`companies FOR UPDATE`, puis, si **exactement une** société et des utilisateurs ou
  clés désignant une société inexistante, `attach_all_principals_in_tx` — le helper unique de
  C-15-7-23 — et une entrée `installation.principals_reattached`, acteur = l'administrateur actif de
  plus petit id) ; rien d'écrit sinon ; test 15 (orphelins + une société, variante saine, variante deux
  sociétés) et trois mutations ; `closes #528` maintenu. « Plusieurs sociétés » laissé à la remise à
  zéro, écrit en limite (atteint par aucun chemin connu). `AlreadyFinalized` retirée ;
  `ResetAttemptError`/`is_reset_retryable` deviennent l'extension (`ResetForbidden`) de
  `SeedAttemptError`/`is_seed_retryable` posés par la 15-7b1 (C-15-7-37) — amende C-15-7-27 et
  C-15-7-36 sur les noms, sans en changer le principe. Test 7b sur
  `kesh_db::test_fixtures::attendre_une_requete_en_cours` au lieu d'`INNODB_TRX` (privilège `PROCESS`
  absent du gate local, table non filtrable par base) — amende C-15-7-36.
- **Écarté** : écrire la limite et passer `closes #528` en `refs` (option (b) de F-1 : la réparation
  coûte une fonction et un test, et toute installation touchée redémarre pour passer à la version) ;
  réparation par une migration (une migration ne voit pas « une seule société » comme une condition
  métier, et P7 l'obligerait au registre de rejeu) ; deux types d'essai et deux prédicats (DRY).
- **Réversible** : oui (la réparation n'écrit rien sur une installation saine).

## C-15-7-39 — 15-7b2 : la clause de coupe vise la sévérité maximale de la passe précédente

- **Contexte** : P2 de la 15-7b2 — trois MEDIUM nés de la remédiation P1 (test 7b, cellule `:1314` du
  manuel, AC 13 sans ligne au manuel), après une P1 à HIGH. La clause de coupe prédéclarée disait
  « à sévérité égale ou supérieure » sans dire à quoi : la sévérité de la passe précédente (HIGH), ou
  celle du défaut remédié (F-2 de la P1, MEDIUM) — F-2 de la P2 l'a relevé.
- **Retenu** : la clause vise la **sévérité maximale de la passe précédente** ; elle n'est **pas**
  déclenchée à la P2. Motif de fond : les MEDIUM nés de la remédiation sont des défauts de **test** et de
  **manuel**, qu'une story-zéro `kesh-db` (la coupe prévue) ne fermerait pas — couper ne traiterait pas
  la cause. Formulation précisée dans la Dérogation de la fiche. Signal déclaré au Project Lead.
- **Écarté** : la lecture « sévérité du défaut remédié », qui aurait coupé une fiche dont les défauts
  recyclés ne relèvent pas de l'axe de coupe.
- **Réversible** : oui — une P3 qui ramènerait un recyclage de sévérité ≥ MEDIUM après cette P2 à
  MEDIUM déclencherait la coupe.


## C-15-7-40 — 15-7b2 (P3) : coupe déclenchée, faite selon la cause — la réparation sort en 15-7b3

- **Contexte** : P3 de la 15-7b2 — F3-1 (la réparation au démarrage repointe des clés d'API inertes) et
  F3-2 (la restauration d'une sauvegarde rouvre #528), MEDIUM, nés de la remédiation P2 (vérifié par
  versions de la fiche : `repair_orphan_principals|principals_reattached` → 0 à `327ea9df`, 9 à
  `ffcdcf8d`), après une P2 à MEDIUM. La clause de coupe (C-15-7-39) est déclenchée ; décision de
  l'orchestrateur (amendement D5).
- **Retenu** : coupe **selon la cause**, non selon l'axe prédéclaré (story-zéro `kesh-db`) : les deux
  MEDIUM portent sur la réparation des installations déjà atteintes. **15-7b2** garde la prévention
  (remise à zéro et `language` ne laissent plus d'orphelin, #542, tests 6a-6c) et passe à `refs #528` ;
  **15-7b3** (`15-7b3-reparation-des-installations-atteintes.md`) reprend l'ex-AC 14, le test 15, ses
  trois mutations, l'action `installation.principals_reattached`, les lignes de manuel et de CHANGELOG,
  y ajoute la réparation dans la transaction de restauration (F3-2) et porte `closes #528`. AC 14 de
  la 15-7b2 réduit à un renvoi (numéro conservé). La clause de la 15-7b2 reste valable pour les passes
  suivantes, par rapport à la P3 (MEDIUM).
- **Écarté** : la story-zéro `kesh-db` prédéclarée (n'aurait fermé ni F3-1 ni F3-2 — motif de
  C-15-7-39) ; garder la réparation et remédier sur place (la clause l'interdit).
- **Réversible** : oui (deux fiches de spec, aucun code écrit).

## C-15-7-41 — 15-7b3 : une clé d'API orpheline est révoquée, jamais repointée

- **Contexte** : F3-1 de la P3 de la 15-7b2. Une clé dont la société est effacée est inerte
  (`get_company_for` répond 500) ; la repointer au démarrage la réactiverait sans geste de l'exploitant,
  alors qu'elle a pu naître en démonstration. Décision de l'orchestrateur.
- **Retenu** : au démarrage et à la restauration, les clés **actives** orphelines sont **révoquées** par
  le mécanisme existant `api_keys::revoke_in_tx(tx, key.company_id, key.id, key.version)` (`revoked_at =
  NOW(3)`, `version + 1`), avec le `company_id` mort de la clé — non repointées ; une clé déjà révoquée
  n'est pas touchée. Leurs ids vont dans `details.api_key_ids_revoked` de
  `installation.principals_reattached` ; le manuel dit qu'elles sont à recréer et que la page des clés
  ne les affiche pas. Seuls les utilisateurs sont rattachés (`attach_users_in_tx`). **Nom d'action
  conservé** ; libellés changés pour dire le sort des clés (« Utilisateurs rattachés à la société, clés
  orphelines révoquées », et trois traductions).
- **Écarté** : repointer et signaler (option (a)/(b) de F3-1 : la clé devient vivante avant que
  quiconque lise le journal) ; laisser les clés inertes sans les révoquer (option (c) : une remise à
  zéro ultérieure les repointerait et les réveillerait) ; révoquer **et** repointer, pour qu'elles
  paraissent dans l'historique de la page des clés (contraire à la décision, qui exclut tout repointage ;
  écart signalé à l'orchestrateur) ; une entrée `api_key.revoked` par clé (une entrée agrégée suffit, sur
  le patron d'`account.chart_loaded`) ; renommer l'action.
- **Réversible** : oui avant livraison ; après, une clé révoquée ne se réactive pas (par conception).

## C-15-7-42 — LOW de la P3 de la 15-7b2 ; la réparation ne rejoue pas ; acteur du démarrage

- **Contexte** : P3 de la 15-7b2, R3-1 à R3-8 et F3-3 à F3-7 (13 LOW).
- **Retenu** : tous appliqués, à la fiche où ils tombent après la coupe — 15-7b2 : R3-1, R3-2, R3-3,
  R3-5, R3-6, F3-4, F3-5, F3-6, F3-7 ; index : R3-4, R3-5 ; 15-7b1 : R3-5 ; 15-7b3 : F3-3, F3-4 (texte du
  manuel), R3-7, R3-8. **R3-8** : au démarrage, **pas de `retry_with`** — angle mort assumé, la
  réparation précède `TcpListener::bind` (`main.rs:181` puis `:369`), un 1205/1213 n'y viendrait que
  d'une seconde instance ou d'un client SQL externe, et un redémarrage le lève ; à la restauration, la
  réparation s'exécute dans la transaction de `run_backup_and_restore`, qui **n'est pas** sous
  `retry_with` (`pool.begin()`, `routes/admin.rs:232-236`) et cette fiche ne l'y met pas : un interblocage
  annule l'import entier, relancé par l'administrateur ; la réparation n'y ajoute aucun verrou neuf (les
  lignes de `companies`, `users`, `api_keys` sont déjà tenues en exclusif après `restore_tables_in_tx`).
  **F3-3** : l'entrée du démarrage reste signée par l'administrateur actif de plus petit id, choix écrit
  (doc-comment, manuel « signée … bien que personne ne l'ait faite », `details.trigger = "startup"`),
  patron de `books.restored` ; à la restauration, par l'acteur d'`admin.full_import`.
- **Écarté** : envelopper le démarrage dans `retry_with` (code pour un cas sans trafic) ; mettre la route
  de restauration sous `retry_with` (hors périmètre : le rejeu d'un import entier, sauvegarde pré-import
  comprise, est une décision à part) ; un acteur « système » (n'existe pas dans `ActorType`).
- **Réversible** : oui.

## C-15-7-43 — 15-7b1 : validation close à la P3 ciblée

- **Contexte** : P3 ciblée (Haiku) de la 15-7b1 sur la remédiation P2 (`ffcdcf8d`) : 0 finding ; seul axe
  non exercé, le PDF, sans objet (`ffcdcf8d` ne touche pas `docs/manual/`).
- **Retenu** : validation **close** ; statut `ready-for-dev` (déjà porté, convention de l'epic), Change
  Log P3 avec le trend P1 4 MEDIUM → P2 2 MEDIUM → P3 0. Le résidu R3-5 de la 15-7b2 (`:314`) y est
  reporté sans rouvrir la validation (renvoi de ligne d'une doc, pas de conception).
- **Écarté** : une passe complète de plus (la remédiation P2 relue ne touche aucun code de production).
- **Réversible** : oui.

## C-15-7-44 — 15-7b3 : une fonction `kesh-db` partagée par le démarrage et la restauration

- **Contexte** : la réparation doit être la même aux deux endroits (`auth/bootstrap`, `routes/admin`),
  dont l'un ouvre sa transaction et l'autre s'exécute dans celle de l'import.
- **Retenu** : `companies::repair_orphan_principals_in_tx(tx, RepairTrigger) -> Option<OrphanRepair>`
  dans `kesh-db`, sans `begin` ni `commit` ; `RepairTrigger::{Startup, Restore { actor_user_id,
  triggered_by_user }}` porte l'acteur ; `attach_users_in_tx` extrait d'`attach_all_principals_in_tx`
  (15-7b2), qui le compose (DRY) ; entrée d'audit écrite dans la fonction (précédent
  `record_step_completed_in_tx`). Cinq modules : sous le seuil. **Statut `backlog`** jusqu'à la
  convergence de sa validation (consigne de l'orchestrateur ; les fiches sœurs gardent `ready-for-dev`
  pendant leurs passes, convention antérieure). **Même release visée** (v0.13.0) que la 15-7b2, sans
  contrainte de sûreté (entre les deux, l'état d'aujourd'hui, sans trou de piste).
- **Écarté** : deux implémentations dans `kesh-api` (règle dupliquée) ; une migration (une société
  unique n'est pas une condition qu'une migration sait poser, et P7 l'obligerait au registre de rejeu) ;
  placer la réparation dans `replay_post_restore_backfills` (registre des migrations de données, pas du
  code de démarrage).
- **Réversible** : oui.

## C-15-7-45 — Une règle et une fonction pour les principaux orphelins : révoquer, puis repointer

- **Contexte** : P4 de la 15-7b2 (R4-1 = F4-1, MEDIUM) et P1 de la 15-7b3 (F1 = R1-2, HIGH ; F2,
  MEDIUM). C-15-7-41 (« une clé orpheline est révoquée, jamais repointée ») n'avait été appliquée qu'au
  démarrage : `attach_all_principals_in_tx` (15-7b2) repointait toujours les clés dans `reset_demo` et la
  branche « aucune société » de `language` — une installation v0.12.x restée sans société voyait ses clés
  de démonstration **réveillées** au premier choix de langue. Et une clé révoquée sans repointage restait
  une clé étrangère pendante, invisible sur la page des clés (F2). Décision de l'orchestrateur.
- **Retenu** : la 15-7b2 pose dans `kesh-db` **la** fonction des principaux orphelins,
  `companies::reattach_orphan_principals_in_tx(tx, target: Option<i64>) -> OrphanPrincipals` — clés
  d'API **actives** orphelines **révoquées d'abord** (`api_keys::revoke_in_tx`, `company_id` mort de la
  clé), quel que soit `target` ; puis, si `target` est donné, utilisateurs orphelins rattachés et
  **toutes** les clés orphelines (révoquées) repointées vers la société. Elle remplace
  `attach_all_principals_in_tx` ; appelée par `reset_demo`, la branche « aucune société » de `language`
  et, dans la 15-7b3, au démarrage et à la restauration (quel que soit le nombre de sociétés pour la
  révocation). Aucune clé orpheline ne redevient active par aucun chemin ; repointée, elle paraît
  révoquée, sous son nom, sur la page des clés. **Révise C-15-7-41** (révoquer **et** repointer — option
  écartée alors comme « contraire à la décision » : F2 a montré que le rejet était circulaire) et
  **C-15-7-44** (plus d'extraction d'`attach_users_in_tx` : la 15-7b3 appelle la fonction de la 15-7b2 ;
  la 15-7b3 **dépend** d'elle pour cette fonction). Prédicat d'orphelin `NOT EXISTS (… companies …)` au
  lieu de `NOT (company_id <=> ?)` (identiques à une société ; la 15-7b3 l'appelle à zéro ou plusieurs).
- **Clause de coupe de la 15-7b2** : formellement atteinte (MEDIUM après une P3 à MEDIUM, avec R4-2/F4-2
  né de la remédiation P3) ; **non appliquée** — R4-1/F4-1 est un **résidu de propagation** d'une décision,
  non un défaut de conception, et l'axe prédéclaré (story-zéro `kesh-db`) ne fermerait ni lui ni une
  ligne de manuel (motif de C-15-7-39, C-15-7-40). **Signal D5 déclaré au Project Lead** au Change Log P4
  de la 15-7b2.
- **Écarté** : écrire l'asymétrie comme choix (option (b) de R4-1 : une clé de démonstration réveillée
  par un choix de langue est précisément ce que C-15-7-41 refusait) ; une révocation propre à la 15-7b3
  « quel que soit le nombre de sociétés » sans toucher la 15-7b2 (option F1 (a) seule : `language`
  repointerait encore une clé active née après le démarrage) ; un seul `UPDATE … SET revoked_at, company_id`
  (le mécanisme existant `revoke_in_tx` est réemployé, DRY).
- **Réversible** : oui avant livraison (spec) ; après, une clé révoquée ne se réactive pas.

## C-15-7-46 — #542 : la réparation des installations déjà touchées passe à la 15-7b3, au démarrage

- **Contexte** : F4-3 de la P4 de la 15-7b2 (MEDIUM, d'origine — C-15-7-34). Plusieurs sociétés
  provisoires, étape ≤ 2, hors démonstration : `seed_demo` y rend 500, le bouton de remise à zéro ne
  vit que dans le bandeau de démonstration, la route est fermée aux clés d'API — la réparation « par la
  réinitialisation » n'était atteignable que par un appel HTTP forgé.
- **Retenu** (décision de l'orchestrateur) : la **15-7b3** supprime au **démarrage** les sociétés
  provisoires superflues (`is_stub`, sans utilisateur ni clé, `MIN(id)` conservée si toutes le sont),
  dans `repair_installation_in_tx`, avant le traitement des principaux (un cumul #542 + #528 se répare en
  un démarrage) ; à la **restauration**, non (une archive est l'état choisi, un 1451 y annulerait
  l'import ; le démarrage suivant le fait). Sur une base **sans utilisateur** : suppression sans entrée
  d'audit (personne pour la signer), `info!`. La 15-7b2 garde la prévention (garde `company_count == 0`,
  AC 13), passe à **`refs #542`** ; sa remise à zéro rend `Invariant` sur plus d'une société (test 6d (ii)) ;
  l'ex-test 6e devient le test 4 de la 15-7b3, qui porte **`closes #528`, `closes #542`**. Manuel et
  CHANGELOG des deux fiches corrigés. **Révise C-15-7-34.**
- **Écarté** : garder la réparation dans la remise à zéro et écrire l'appel par l'API (option (b) de
  F4-3 : un manuel qui prescrit un `curl` à un utilisateur bloqué) ; l'écrire en angle mort (c) ; la faire
  aussi à la restauration.
- **Réversible** : oui (spec).

## C-15-7-47 — 15-7b3 : réparation au démarrage non bloquante ; fonction et action renommées

- **Contexte** : F6 de la P1 de la 15-7b3 (MEDIUM : une erreur persistante de la réparation rendait
  l'instance non démarrable, sans recours écrit) ; R1-4 (LOW : `Invariant` « aucun utilisateur ») ; R1-9
  (LOW : libellé affirmant un rattachement qui n'a pas toujours lieu) ; la réparation porte désormais aussi
  les sociétés superflues (C-15-7-46).
- **Retenu** : au démarrage, une erreur de la réparation **ne refuse pas le boot** — `rollback`
  *best-effort*, `tracing::error!` avec le détail, `ensure_admin_user` poursuit comme aujourd'hui
  (mode dégradé : connexion et export disponibles) ; test 5 (déclencheur sur `audit_log`), mutation
  « propager l'erreur » ; manuel admin : quoi faire (exporter, redémarrer une fois, ticket, retour à la
  version précédente possible). À la restauration, inchangé : l'erreur annule l'import. Base sans
  utilisateur : `Ok(None)` (sauf sociétés superflues), plus d'`Invariant`. Fonction
  `repair_orphan_principals_in_tx` → **`repair_installation_in_tx`**, `OrphanRepair` →
  `InstallationRepair` ; action `installation.principals_reattached` → **`installation.repaired`**,
  libellé neutre « Réparation de l'installation » (de « Reparatur der Installation », it « Riparazione
  dell'installazione », en « Installation repaired »). **Révise** le « nom conservé » de C-15-7-41 et le
  nom de C-15-7-44.
- **Écarté** : garder le refus et écrire le recours (option (b) de F6 : rendre l'installation moins
  utilisable qu'avant la mise à jour) ; un `warn!` au lieu d'`error!` (l'exploitant doit le voir) ;
  garder l'ancien nom d'action (il décrirait une entrée qui peut ne porter que des sociétés supprimées).
- **Réversible** : oui (spec).

## C-15-7-48 — Sortir de la démonstration exige `KESH_PRODUCTION_RESET` : le manuel le dit

- **Contexte** : R4-2 = F4-2 de la P4 de la 15-7b2 (MEDIUM, né de la remédiation P3, F3-7). Une
  démonstration est à l'étape 3 ; la garde `step > 2 && !KESH_PRODUCTION_RESET` refuse toute
  réinitialisation sur une installation par défaut, avec un message qui accuse le rôle (#534), et
  aucune autre route ne sort de la démonstration. Le manuel offrait le bouton comme disponible.
- **Retenu** : les manuels (`user-manual.tex:177-189`, `admin-manual.tex:1314` et `:691`) et le renvoi
  de la 15-7b1 (`user-manual.tex:179-189`) disent la vraie marche — l'exploitant pose
  `KESH_PRODUCTION_RESET` (`1`, `true`, `yes`, `on`, sans casse — `routes/onboarding.rs:45-53`), redémarre,
  la réinitialisation fonctionne, puis il retire la variable ; sans elle, refus (403) même à
  l'administrateur. L'impasse (aucune sortie sans geste d'exploitation) est un **choix de sécurité
  existant**, conservé et écrit ; l'orchestrateur commente #534. La 15-7b1 reçoit le report sans rouvrir
  sa validation (renvoi de doc, patron C-15-7-43). L4-2 : « CR à ouvrir » → `refs #534`.
- **Écarté** : changer la garde (décision produit, hors du périmètre d'une story de trace) ; ne corriger
  que la 15-7b2 en laissant le renvoi de la 15-7b1 muet.
- **Réversible** : oui.

## C-15-7-49 — 15-7b3 : détail des clés révoquées, lignes orphelines de #279, sessions ; LOW

- **Contexte** : P1 de la 15-7b3, F3, F4, F5 (MEDIUM) et LOW ; P4 de la 15-7b2, LOW.
- **Retenu** : **F3** — les détails d'audit (`installation.repaired`, et de même `installation.reset`
  et `company.created` de la 15-7b2) portent pour chaque clé révoquée `{id, name, created_by_user_id,
  created_at, last_used_at}` ; **pas de préfixe** : vérifié au schéma
  (`20260605000001_api_keys.sql:18-38`), `api_keys` ne garde que l'empreinte `key_hash` (jamais
  journalisée) et le libellé `name`. **F4** — les autres lignes orphelines d'une remise à zéro v0.12.x
  (`vat_rates`, `company_invoice_settings`, contacts, produits, factures, avoirs) ne sont **pas**
  nettoyées : limite écrite (inventaire, « ne fait pas », manuel, CHANGELOG), renvoi à **#546** (ouverte
  par l'orchestrateur). **F5** — « l'import déconnecte déjà tout le monde » était faux : seul l'importateur
  est redirigé (par son client), les autres sessions gardent l'ancien `company_id` jusqu'à expiration
  (JWT sans état), voient erreurs ou listes vides, et doivent se reconnecter — fiche et manuel corrigés.
  **R1-1** — test 2 : un second administrateur B émet l'import, acteur A ≠ `triggered_by_user` B ;
  test 3 (c) à deux ids distincts. **LOW** : tous appliqués, à la fiche où ils tombent (Change Logs P4 de
  la 15-7b2 et P1 de la 15-7b3). **R4-4** (P4 de la 15-7b2) : le « 9 » de C-15-7-40 compte des **lignes**
  (10 occurrences) — corrigé dans la fiche, pas ici (le registre ne se réécrit pas). **R1-8 / F12, en
  partie faux** : le renvoi `routes/admin.rs:346-353` du commentaire d'`audit_uid` était **juste** (relu
  par `grep -n` : commentaire `:346-353`, requête `:354-358`) ; les deux lentilles le décalaient chacune
  dans un sens — précisé en trois plages.
- **Écarté** : purger les lignes orphelines dans la réparation (effacement de données, à arbitrer
  séparément — #546) ; un préfixe calculé (le secret n'est pas stocké).
- **Réversible** : oui.

## C-15-7-50 — 15-7b3, P2 : sociétés provisoires supprimées seulement sans aucune référence ; note d'exploitation à la mise à jour ; LOW

- **Contexte** : validation P2 de la 15-7b3 (Sonnet ×2), F-2, F-1, R2-1, R2-2 (MEDIUM) et 11 LOW
  distincts ; décisions de l'orchestrateur.
- **Retenu** : **F-2** — une société provisoire n'est supprimée au démarrage que si **aucune ligne
  d'aucune table** ne la désigne ; la liste des colonnes qui désignent `companies(id)` est **lue à
  l'exécution** dans `information_schema.KEY_COLUMN_USAGE` (schéma courant,
  `REFERENCED_TABLE_NAME = 'companies'`), par `companies::company_referencing_columns` — elle couvre les
  quatre tables en `ON DELETE CASCADE` (`users`, `bank_profiles`, `contact_persons`, `email_templates`)
  et les tables futures ; test de schéma par **inclusion** d'une liste écrite (non par égalité avec une
  seconde lecture, verte par construction) et test de comportement (une seule référence suffit,
  variante CASCADE obligatoire). Chaque suppression sous `SAVEPOINT` : une erreur laisse la société en
  place (`warn!`) sans annuler le reste de la réparation ; un `ROLLBACK TO SAVEPOINT` en échec
  (interblocage) rend l'erreur, qui suit la règle non bloquante du démarrage. **F-1** — la note
  d'exploitation va au § *Procédure de mise à jour standard* (`admin-manual.tex:1704-1717`) et au
  § *Dépannage* (`:2054`, sous-section neuve) ; `:1314` ne garde qu'un renvoi. **R2-1** — le manuel dit la
  suppression aussi sans utilisateur (seule l'entrée d'audit exige un utilisateur). **R2-2** — test 2 :
  assertion du `company_id` limitée à `admin.full_import` ; **aucune** variante de recul du verrou pour
  `books.restored` (même sous-SELECT, rien de plus à apprendre). **R2-3 = F-3** — réparation **avant la
  lecture des compteurs** d'`ensure_admin_user`, invariant écrit (ni insertion ni suppression dans
  `users`, jamais zéro société) ; deux instances simultanées sur base vide : angle mort assumé (Kesh
  tourne en une instance). **F-4** — `user-manual.tex:297`. **F-5** — l'entrée CHANGELOG s'ajoute à la
  section `## [0.13.0]` de `main` après rebase. **LOW R2-4 à R2-11** : tous appliqués (Change Log P2 de
  la fiche). Fiche 15-7b2 **non modifiée**.
- **Écarté** : élargir le `NOT EXISTS` à une liste recopiée des 29 tables (dérive silencieuse à la
  prochaine table) ; garder le 1451 comme seul garde-fou (ne voit pas les cascades) ; une variante de
  test faisant reculer le verrou de période ; poser la réparation après les compteurs avec relecture.
- **Réversible** : oui (spec seulement).

## C-15-7-51 — 15-7b2, P5 : la recette de sortie de la démonstration dépend de la 15-11 (révise C-15-7-48)

- **Contexte** : F5-1 de la P5 de la 15-7b2 (MEDIUM). `docker-compose.yml` (celui que le manuel fait
  télécharger) et `docker-compose.prod.yml` portent une liste `environment:` explicite, sans `env_file`,
  et ne transmettent pas `KESH_PRODUCTION_RESET` : la recette écrite par C-15-7-48 (« poser la variable
  dans `.env`, redémarrer, réinitialiser, la retirer ») n'atteint pas le processus. Le défaut est plus
  large (SMTP et une vingtaine de variables) : l'orchestrateur a ouvert **#550** et la story
  **15-11-configuration-transmise**.
- **Retenu** : **ordre de merge imposé — la 15-11 avant la 15-7b2**. La 15-7b2 ne modifie pas les
  compose ; elle déclare la dépendance (en-tête, Dev Notes, « ne fait pas »), porte `refs #550`, étend le
  grep de l'AC 10 à `docker-compose.yml` et `docker-compose.prod.yml`, et son T8 bloque le merge tant que
  `grep -n PRODUCTION_RESET` ne rend pas une ligne dans chacun et que `docker compose config` ne montre pas
  la variable transmise. Le manuel garde la recette par `.env`, vraie une fois la 15-11 mergée.
- **Écarté** : corriger les compose dans la 15-7b2 (doublon partiel de la 15-11, conflit au merge) ; une
  formulation vraie dans les deux ordres (« ajouter la variable à la section `environment:` du compose »)
  — elle fait éditer un fichier que l'exploitant retélécharge à chaque mise à jour, et deviendrait une
  recette concurrente du `.env` une fois la 15-11 livrée ; laisser le manuel promettre une recette
  inexécutable.
- **Réversible** : oui (spec seulement). Si la 15-11 devait glisser hors de la release de la 15-7b2, la
  formulation écartée redevient le repli, à décider alors.

## C-15-7-52 — 15-7b2, P5 : LOW appliqués ; tableaux du manuel corrigés dans leur section ; `user_ids` non ajouté

- **Contexte** : R5-1 à R5-5, F5-2 à F5-4 de la P5 de la 15-7b2 (tous LOW).
- **Retenu** : R5-3 — T7 corrige les **dix** tableaux de `sec:env-vars` du manuel d'administration
  (huit rognés, deux décalés : recensés au `.log`, `Overfull \hbox` de 96 à 263 pt, et au PDF aplati ;
  aucun autre tableau du manuel ne déborde), par un même geste **local à la section** (titre hors
  ligne), sans changer `\titleformat{\paragraph}` partagé par les trois manuels. R5-4 — la phrase est
  restreinte aux champs `api_keys_*` ; `user_ids` **n'est pas** ajouté à `installation.reset` (aucun
  besoin de lecture établi, la fiche reste stable). R5-5 — `assert_eq!(log_bin, 0, …)`. F5-4 —
  `CONNECTION_ID()` est la preuve, `foreign_key_checks` une ceinture. R5-1, R5-2, F5-2, F5-3 appliqués
  tels que proposés.
- **Écarté** : corriger le seul tableau `:683-693` (laisse sept tableaux rognés dans le même PDF) ;
  changer le style global des `\paragraph` (effet sur 38 titres du seul manuel d'administration, hors
  périmètre) ; ajouter `user_ids` aux trois sites.
- **Réversible** : oui (spec seulement).

## C-15-7-53 — 15-7b3, P3 : mutation d'exclusion sur une table en CASCADE ; service `kesh-api` au manuel ; archive sans société en limite ; LOW

- **Contexte** : validation P3 de la 15-7b3 (Opus ×2) — R3-1 = F3-1 et R3-2 = F3-3 (MEDIUM), 13 LOW
  distincts ; décisions de l'orchestrateur.
- **Retenu** : **R3-1 = F3-1** — la troisième variante de la mutation 9 (« exclure une table de la liste
  des références ») porte sur **`bank_profiles`** (`ON DELETE CASCADE`) et rougit le test 6 (b) ; la
  variante `api_keys` (RESTRICT) était devenue muette, le 1451 étant absorbé par le `SAVEPOINT` de la P2.
  `bank_profiles` plutôt que `users` : le test 6 (b) la monte déjà (aucun montage neuf), alors qu'une
  variante `users` exigeait un stub désigné par un seul utilisateur non administrateur. Les Dev Notes
  écrivent que, pour les 25 tables en RESTRICT, la garde `EXISTS` et le `SAVEPOINT` se recouvrent, que
  seules les quatre CASCADE rendent la garde observable, et que c'est voulu (défense en profondeur) ; le
  test 4 (ii) prouve le comportement, non la garde. **Signal D5** déclaré au Change Log P3 : recyclage
  d'une remédiation, traité localement, pas de découpage. **R3-2 = F3-3** — `docker compose logs
  kesh-api` dans la sous-section neuve du Dépannage ; l'AC 6 corrige **au passage** les lignes fausses
  préexistantes du manuel d'administration : `:2063` (`logs kesh | tail -50`), `:2058` (« container
  `kesh` », deux occurrences) et **`:1905`, `:1906`** (`--tail=100 kesh`, `--since=1h kesh`). ⚠️ Écart
  avec la consigne : l'orchestrateur tenait `:1905-1906` pour justes « si le grep le confirme » ; le
  `grep -n "compose logs" docs/manual/fr/admin-manual.tex` à `cf40085f` les montre fausses (service
  `kesh`, qui n'existe que dans `docker-compose.dev.yml`) — elles rejoignent la correction, même fichier,
  même geste. `:560` (« Nom du projet : `kesh` », projet Synology) est juste et reste. **F3-2** — archive
  sans société restaurée sur une installation onboardée : **limite assumée**, sans mécanisme neuf
  (inventaire § 1, AC 2 « Cas laissés » borné, « ne fait pas », phrase au § *Reprises* du manuel).
  **F3-6** — l'erreur d'origine du `DELETE` est rendue, jamais le 1305 du `ROLLBACK TO` ; forme
  d'émission du précédent `reconciliation.rs:1026-1072` ou `sqlx::raw_sql` ; `is_savepoint_lost`
  (R3-4) inutile ici, rien à descendre dans `kesh-db`. **F3-7** — mutation 14 (`MIN(id)`). **F3-4** —
  ligne neuve pour le § *Rollback en cas d'échec* (`:1734-1749`). **F3-8** — `### Corrigé` à créer s'il
  est absent après rebase (présent à `origin/main` le 2026-10-08). **R3-8** — dépendance à la 15-11 ;
  pré-requis `log_bin` alignés sur la 15-7b2 (`assert_eq!`). Autres LOW (R3-3 à R3-7, R3-9, F3-5)
  appliqués tels que proposés. Fiche 15-7b2 **non modifiée**.
- **Écarté** : variante `users` (montage neuf pour le même signal) ; capturer le `warn!` au test 4 (ii)
  (capture `tracing` coûteuse pour une redondance voulue) ; refuser à l'import une archive à utilisateurs
  et sans société (mécanisme neuf, cas étroit) ; laisser `:1905-1906` faux faute de confirmation par la
  consigne ; ouvrir une issue séparée pour les lignes préexistantes (même section, même geste).
- **Réversible** : oui (spec seulement).

## C-15-7-54 — 15-7b2 : la mise en page des tableaux de `sec:env-vars` passe à la 15-11a (révise C-15-7-52)

- **Contexte** : C-15-7-52 chargeait la 15-7b2 (AC 11, T7) de corriger les dix tableaux rognés ou décalés de `sec:env-vars`. La remédiation P2 de la 15-11a (choix C81, F2-3) en reprend la charge (AC12 (j)), parce que ses propres contrôles du PDF (AC12 b) en dépendent et qu'elle merge avant la 15-7b2.
- **Retenu** : AC 11 et T7 de la 15-7b2 renvoient à la 15-11a (AC12 (j)) et ne gardent que le contrôle, après rebase, que la cellule `KESH\_PRODUCTION\_RESET` (`:691`, réécrite par la 15-7b2) se lit entière dans le PDF aplati. Le motif de contrôle `KESH.{1,2}PRODUCTION.{1,2}RESET` et le T8 énumèrent les mentions nouvelles du manuel et du CHANGELOG (AC12 f, AC13 de la 15-11a) pour ne pas rougir à tort, et notent que la 15-11a interdit de nommer la variable dans un commentaire des compose (une ligne par compose). Le geste de mise en page (titre hors ligne, dans la section seule, sans toucher `\titleformat{\paragraph}`) reste celui de C-15-7-52, exécuté par l'autre story. Les autres volets de C-15-7-52 (R5-4 `user_ids`, R5-5, F5-4) sont inchangés.
- **Écarté** : garder la mise en page dans les deux fiches (double édition des mêmes hunks, conflit au rebase) ; faire merger la 15-7b2 d'abord (inverse l'ordre imposé par C-15-7-51).
- **Réversible** : oui (spec seulement).

## C-15-7-55 — 15-7b2 et 15-7b3 : recettes de redémarrage en `docker compose up -d`, zone `:1704-1717` partagée avec la 15-11a

- **Contexte** : la 15-11a (AC12 f, C83, C84), validée et close, établit que `docker compose restart` ne relit pas `.env`. La recette de sortie de la démonstration de la 15-7b2 disait « redémarrer » ; la 15-7b3 et la 15-11a écrivent toutes deux dans `admin-manual.tex:1704-1717`.
- **Retenu** : 15-7b2 (`:1314`, `:691`, T7) prescrit `docker compose up -d` après chaque changement de `KESH_PRODUCTION_RESET`, avec la raison ; `user-manual.tex:177-189` inchangé (aucune consigne d'exploitant). 15-7b3 : recette du *Dépannage* en `docker compose up -d` ; coordination écrite pour `:1704-1717` (15-11a d'abord, relocalisation par le texte, texte placé après l'énumération et l'encadré de la 15-11a sans les réécrire, `:1734-1749` décalé, PDF régénéré par la seconde à merger).
- **Écarté** : laisser « redémarrer » en prose (ambigu, mène à `restart`) ; faire réécrire le point 3 par la 15-7b3 (double édition des mêmes hunks).
- **Réversible** : oui (spec seulement).

## C-15-7a1-1 — 15-7a1, T0 : alignement sur le livré — dérives de lignes seulement, aucun AC changé ; le dev enchaîne

- **Contexte** : la fiche 15-7a1 a été écrite avant les fusions 15-5e1/15-5e2 et 15-8b. Chaque référence a été relocalisée par le texte sur `HEAD` (`0f6dfabb`, sur `origin/main` `9cb5083b`).
- **Constat** : (a) **dérives de lignes** — `accounts.rs` : `bulk_create_from_chart` en-tête `:967-976` (fiche `:925-934`), court-circuit `:983-985` (`:941-943`), rollback sur `last_insert_id == 0` `:1033` (`:991`) ; `routes/onboarding.rs` : verrous `:250`, `:649`, `:802` (fiche `:250`, `:653`, `:806`), appel `insert_with_defaults_in_tx` `:717` et bloc `:716-734` (`:721`, `:720-738`), appel des taux `:741` (`:745`), commentaire `:678` (`:682`) ; `fiscal_years.rs` : `fiscal_year.created` de `create_if_absent_in_tx` `:320` (`:314`) ; `journal_entries::count_by_company` `:527` (`:495`) ; ordre des verrous `docs/MULTI-TENANT-SCOPING-PATTERNS.md:298` et `:317` (`:322`). Toutes les autres références (`bank_accounts.rs`, `company_invoice_settings.rs`, `vat_rates.rs`, tests `:953`/`:1084`/`:1174`, `bootstrap.rs:83/:148/:347`, `test_schema_guard.rs:62`, squash `:841`, `onboarding_path_b_e2e.rs:216/:219`) sont exactes. (b) **`finalize` est désormais enveloppé** par `crate::retry::retry_app_on_deadlock("onboarding::finalize", …)` (15-5e2) : la fiche ne touche pas la route ; la variante `insert_with_defaults_in_tx` reste appelée dans la fermeture rejouée, à l'identique — **aucun effet sur la 15-7a1** (le booléen « inséré » d'une tentative annulée n'est consommé qu'à partir de la 15-7a2, qui devra le lire tentative par tentative). (c) Les copies du verrou d'état de la route lisent `id, singleton, …` ; `LOCK_SQL` (AC 7) omet `singleton`, que l'entité `OnboardingState` ne porte pas — conforme à la fiche.
- **Retenu** : aucun écart ne change une règle ni un AC ⇒ le développement enchaîne sans arrêt. Les références de la fiche ne sont pas réécrites (texte validé) ; la table de correspondance est au Change Log de la fiche (« Alignement sur le livré »).
- **Écarté** : réécrire chaque numéro de ligne dans le corps de la fiche (bruit dans un texte validé ; les numéros dériveront encore au rebase).
- **Réversible** : oui (aucun code).

## C-15-7a1-2 — 15-7a1, développement : choix d'exécution non fixés par la fiche

- **Contexte** : quelques détails de forme n'étaient pas fixés par la fiche validée.
- **Retenu** : (a) `UpsertPrimaryOutcome::into_account()` porte la projection sur `BankAccount` de l'enveloppe pool (une seule écriture de la règle « `Created`, `after` ou `Unchanged` ») ; (b) les quatre taux du seed vivent dans une constante `DEFAULT_SWISS_RATES` du module, liés en `Decimal::new(mantisse, 2)` (pas de littéral SQL, pas de `CAST`) ; (c) `insert_with_defaults_in_tx` calcule `let inserted = rows == 1` et le rend sur **les deux** sorties de succès — la mutation 4 (« `true` en dur ») porte ainsi sur une seule ligne ; (d) test 6 : la seconde société du montage a `ide_number = None`, le numéro IDE de `sample_new_company` étant unique (`uq_companies_ide_number`) ; (e) test 4 : l'erreur `InactiveOrInvalidAccounts` est vérifiée à travers l'enveloppe pool sur une société sans plan.
- **Écarté** : un `match` dupliqué dans l'enveloppe pool pour projeter l'issue ; garder l'`INSERT` multi-lignes et rendre les taux par relecture (ne distingue pas inséré/préexistant).
- **Réversible** : oui (local au code de la story).

## C-15-7a1-3 — 15-7a1, revue de code P1 : sort des neuf LOW

- **Contexte** : la revue P1 (Sonnet ×3, B/E/A) rend 0 au-dessus de LOW et neuf LOW (B 3, E 4, A 2 ; B-1 = A-1).
- **Retenu** : (a) **corrigés** — B-1/A-1 (les deux renvois « cf. variante pool » / « cf. pool variant » de `company_invoice_settings.rs` remplacés par la justification d'origine, portée dans la variante `_in_tx`, seule à avoir un corps) ; B-3 par deux tests neufs dans `bank_accounts_repository.rs` (`Updated` non commité ; erreur d'origine de l'enveloppe `upsert_primary`, provoquée par une clé étrangère) et un dans `accounts_repository.rs` (rollback de l'enveloppe `bulk_create_from_chart` sur collision du **dernier** compte de l'ordre topologique — mutation « `commit` au lieu de `rollback` » exécutée : rouge, `left: 86`). (b) **A-2 en angle mort assumé, écrit dans le test** : la branche `OptimisticLockConflict` de `upsert_primary_in_tx` suit un `SELECT … FOR UPDATE` dans la même transaction et n'est pas atteignable depuis un test sans modifier le code. (c) **B-2 et E-4 acceptés** (code sans appelant jusqu'à la 15-7a2 ; contrat « l'appelant annule » documenté). (d) **E-1, E-3 et le booléen relu par tentative** reportés au Change Log de la 15-7a2, qui les prévoit déjà dans son AC 8.2/AC 5 ; **E-2** au Change Log de la 15-7b1, dont c'est le périmètre (§ 2).
- **Écarté** : provoquer `OptimisticLockConflict` par un déclencheur SQL de test (modifie le schéma de test pour une branche inatteignable en production) ; câbler E-1/E-3 dès la 15-7a1 (la fiche exclut `routes/onboarding.rs`, la 15-7a2 les câble avec l'audit).
- **Clôture** : la remédiation ne touche que des commentaires et des tests — aucune ligne de production exécutable ⇒ pas de passe ciblée (§ « Ce qui permet de CLORE la boucle »).
- **Réversible** : oui.

## C89 — 15-12 : clôturer dans l'ordre (#543) — invariant « les clos forment un préfixe », filet non verrouillant aux deux points de passage, détection à l'écran, story entière avec coupe de repli

- **Contexte** : #543 (P1) — `fiscal_years::close` ne regarde aucun autre exercice ; le bilan est
  cumulatif ; dans l'état « N ouvert, N+1 clos », seuls le `PUT` et le `DELETE` d'une écriture
  refusent (15-8a/15-8b). Relevé au code sur `origin/main` `9cb5083b` : les 22 routes `Rejouee` du
  registre passent par **trois** points de passage — `create_in_tx_inner` (19), `update` (1, gardé),
  `delete_in_tx` (2, dont la dévalidation non gardée, C-15-8-29). Sources de l'état fautif : clôture
  hors ordre, création d'un exercice antérieur à un clos, restauration d'une sauvegarde ; la
  réouverture (LIFO), la démo, la graine de test et `onboarding::finalize` ne le produisent pas.
- **Retenu** :
  1. **Invariant I** (les exercices clos forment un préfixe chronologique) tenu par trois transitions :
     clôture refusée tant qu'un antérieur est ouvert (variante neuve `EarlierFiscalYearOpen`, code
     **`409 EARLIER_FISCAL_YEAR_OPEN`**, nomme le **plus ancien** antérieur ouvert) ; création refusée
     sous un postérieur clos (`400 LATER_FISCAL_YEAR_CLOSED`, variante existante) ; réouverture LIFO
     inchangée.
  2. **Code dédié plutôt que `ILLEGAL_STATE_TRANSITION`** (patron de la garde LIFO) : l'écran traduit
     tout `ILLEGAL_STATE_TRANSITION` de la clôture en « déjà clôturé » ; une intégration doit
     distinguer les deux refus.
  3. **Ordre des verrous de la clôture** : lecture non verrouillante de `start_date` (immuable), puis
     antérieurs ouverts `FOR UPDATE` en parcours ascendant, puis Y — toutes les acquisitions de lignes
     d'exercice deviennent ascendantes. Création et clôture passent par `retry_on_deadlock` (la course
     création/clôture se résout par un interblocage attendu, à mesurer en T0).
  4. **Filet généralisé** (la garde de la 15-8a **se généralise**) aux deux points de passage :
     `create_in_tx_inner` par une lecture **non verrouillante** (`find_later_closed`) — preuve écrite :
     sous I, la clôture d'un postérieur exige l'exercice de l'écrivain clos et le lit sous verrou, que
     l'écrivain tient ; la lecture n'a à voir que l'état hérité, stable ; aucun verrou neuf sur les 19
     flux — et `delete_in_tx` sans la condition `enforce_ownership` (lecture verrouillante existante).
     Bras `LATER_FISCAL_YEAR_CLOSED` dans le lot de rapprochement ; message serveur neutre (clé neuve
     `error-later-fiscal-year-closed`, l'ancienne ne valait que pour la modification).
  5. **Installations déjà fautives** : ni migration, ni détection au démarrage, ni refus à l'import ;
     le filet rend l'état inoffensif, l'écran des exercices l'**affiche** (bandeau calculé depuis la
     liste, sans changement d'API) avec la réparation : rouvrir le plus récent clos (gestes existants),
     puis clôturer dans l'ordre.
  6. **Pas de découpage** malgré 8 modules : la coupe A (l'ordre) / B (le filet) séparerait les deux
     moitiés d'une seule preuve (filet non verrouillant ⇐ lecture verrouillante de la clôture, test
     13 c). Dérogation écrite dans la fiche ; **coupe de repli 15-12a / 15-12b sans nouvelle
     délibération** au premier défaut recyclé sur l'ordre des verrous ou à la non-convergence (D5).
- **Écartées** : (a) garder seulement l'écriture (seconde voie de l'issue) — état fautif toujours
  atteignable, garde sur chaque flux ; (b) filet verrouillant (`find_later_closed_in_tx`) dans
  `create_in_tx_inner` — verrous d'intervalle neufs sur 19 routes, cycles neufs entre écrivains de N et
  de N+1, pour aucun gain sous I ; (c) détection au démarrage — journal non lu, l'état est déjà
  inoffensif ; (d) refuser une sauvegarde fautive à l'import — rendrait inutilisables les sauvegardes
  v0.12.x ; (e) migration de réparation — rouvrir N+1 ou clôturer N est une décision comptable ;
  (f) nommer le plus **proche** antérieur ouvert au refus de clôture — la même garde le refuserait à
  son tour ; (g) découper d'emblée en 15-12a/15-12b — cf. point 6.
- **Réversible** : oui (fiche seulement ; code non écrit). La coupe de repli est prête.

## C90 — Lettrage (15-1a) : la marque en deux colonnes de `journal_entry_lines`, sans table ; révise l'arbitrage du 2026-08-25
- **Contexte** : la reprise du lettrage (#518) relit les fiches d'août contre `origin/main`. L'arbitrage
  du Project Lead du 2026-08-25 avait retenu une table `letterings` (compteur `seq` par société). Depuis,
  l'import d'installation (#386, v0.12.1) exige un inventaire de tables **identique** dans les deux sens
  (`admin_backup/import.rs:118-136`) : une table neuve rend **inimportable toute sauvegarde antérieure** —
  motif pour lequel la 25-4-d2a a déjà préféré une colonne. Une colonne nullable passe `check_schema_compat`.
- **Retenu** : `journal_entry_lines.lettering_key BIGINT NULL` (= plus petit `id` de ligne du groupe) et
  `lettering_origin VARCHAR(10) NULL` (`document`, `reversal`, `manual`), deux `CHECK`, deux index, **sans
  FK** (le garde-fou d'inventaire de la 15-8a interdit toute référence vers les lignes, que le `PUT`
  réinsère). La clé sans compteur supprime le verrou de compteur et sa classe de défauts (P5-1, P6-1, P8-3
  d'août). Auteur et date : au journal d'audit. **Pas** de bump `min_required` (P1 : aucune opération P3) —
  risque nommé : un binaire antérieur pourrait modifier une écriture manuelle lettrée et casser un groupe.
- **Écartées** : la table `letterings` (sauvegardes antérieures perdues) ; une table de liaison vers les
  lignes (interdite par le point 1 bis du garde-fou) ; un compteur sur `companies` (verrou de plus, ABBA).
- **Réversible** : oui tant que non développé ; après migration appliquée, revenir à une table coûte une
  migration de données.

## C91 — Lettrage (15-1a) : le code affiché est la clé en base 26 bijective
- **Contexte** : sans compteur par société (C90), il n'y a plus de suite `A, B, C…` par société.
- **Retenu** : `code = base26_bijective(lettering_key)` (fonction pure de `kesh-core`, avec son inverse) :
  4 lettres jusqu'à 475 254 lignes, 5 jusqu'à ~12 millions ; stable, dictable. Un groupe dissous puis reformé
  avec la même plus petite ligne reprend le même code — jamais deux groupes vivants à la fois.
- **Écartées** : `A, B, C…` par société (exige un compteur sérialisé) ; l'identifiant numérique brut (moins
  lisible sur une ligne) ; un ULID (indictable).
- **Réversible** : oui — le code n'est jamais stocké, une autre projection se substitue sans migration.

## C92 — Lettrage, question 1 du dégel : pas de lettrage partiel ; un groupe à somme nulle de N lignes
- **Contexte** : la fiche d'août lettrait des **paires** (D2 : une facture, un règlement). La 24-2 permet
  plusieurs règlements par facture, la 25-4-d2a un solde du reste.
- **Retenu** : un lettrage = **groupe** de ≥ 2 lignes d'un même compte lettrable, somme `Σ(débit − crédit)`
  **exactement nulle**. Un règlement partiel laisse ses lignes ouvertes ; la vue (15-1b) les montre avec le
  motif « partiellement réglée ». L'écart d'un règlement amputé se traite par le solde du reste (facture) ou
  par une écriture d'ajustement (hors pièce), jamais par une tolérance.
- **Écartées** : le lettrage partiel (groupe à somme non nulle) — il ferait mentir l'invariant « somme des
  ouverts = solde » ; une tolérance de 5 centimes (Kesh proposerait ce qu'il refuse).
- **Réversible** : oui — un état « partiel » pourrait s'ajouter (valeur d'origine ou colonne nullable).

## C93 — Lettrage, question 2 du dégel : trois origines ; Kesh lettre ce que l'utilisateur a déjà apparié, et propose le reste
- **Contexte** : règle du `CLAUDE.md` « un appariement automatique propose, il ne crée jamais ».
- **Retenu** : `document` — posé d'office quand une pièce est soldée (facture client par ses règlements,
  solde ou avoir ; facture fournisseur par son paiement) : le rattachement a été **déclaré** par l'utilisateur
  en saisissant le règlement, Kesh ne devine rien (15-1a2). `reversal` — posé d'office entre une ligne libre
  et sa contre-passation (`reverses_entry_id`, lien déclaré). `manual` — choisi par l'utilisateur ; Kesh
  **propose** des paires (même compte, sens opposés, montants égaux) et n'écrit rien sans clic. Les lignes
  d'une pièce sont **exclues** du lettrage manuel (motifs de `reversal_blockers`) : une facture est soldée au
  grand livre si et seulement si son reste dû est nul.
- **Écartées** : tout manuel (double saisie de ce que `invoice_settlements` dit déjà, et vue fausse sur le cas
  le plus fréquent) ; lettrage automatique par similarité de montant (faux rattachement muet).
- **Réversible** : oui pour `reversal` (on peut cesser de le poser) ; `document` engage la cohérence avec les
  pièces.

## C94 — Lettrage, question 3 du dégel : lettrer toujours ; délettrer refusé si TOUTES les lignes sont sur exercice clos ; vue « au » d'une date
- **Contexte** : D3 d'août (arbitrage de Guy) : lettrer sur exercice clos oui, délettrer non dès qu'un
  exercice est clos. Appliquée telle quelle, elle bloquerait l'annulation (en exercice ouvert) d'un règlement
  d'une facture d'un exercice clos.
- **Retenu** : lettrer reste permis quel que soit l'exercice (D3 conservée). Le délettrage manuel est refusé
  si **toutes** les lignes du groupe sont sur des exercices clôturés. La vue des postes ouverts prend une date
  `asOf` : une ligne est ouverte à `X` si elle n'est pas lettrée ou si son groupe a une ligne postérieure à
  `X` ; la somme des ouverts égale alors le solde cumulatif à `X`. Un groupe ayant une ligne en exercice ouvert
  ne change donc rien de ce qui était ouvert à la fin d'un exercice clos (les clos forment un préfixe, C89).
  La contre-passation ne défait jamais un groupe (C97).
- **Écartées** : D3 stricte ; délettrage libre (changerait la liste des postes ouverts d'un exercice clos).
- **Réversible** : oui (règle de garde).

## C95 — Lettrage, question 4 du dégel : un écran dédié « Postes ouverts »
- **Retenu** : `/open-items`, menu **Mensuel**, entre Réconciliation et Rapports ; liens depuis le Grand livre
  d'un compte lettrable et depuis le code de lettrage d'une fiche d'écriture ; bandeau de frontière avec la
  réconciliation (D6 d'août).
- **Écartés** : un onglet du Grand livre (un rapport n'est pas le lieu d'une action d'écriture) ; un écran
  « compte » (inexistant) ; la fiche facture (le lettrage des pièces s'y fait déjà par les règlements).
- **Réversible** : oui.

## C96 — Lettrage : comptes lettrables = Actif/Passif non rattachés à un compte bancaire
- **Contexte** : la fiche d'août bornait la vue aux rôles `Receivable`/`Payable` ; #518 vise aussi les comptes
  de passage, acomptes, avances, compensations, qui n'ont pas de rôle ; le compte de créance des réglages peut
  être n'importe quel compte `Asset`.
- **Retenu** : type `Asset` ou `Liability`, et aucun `bank_accounts.journal_account_id` ne le désigne ; un
  compte archivé reste lettrable. Une seule fonction (`is_letterable_account`), exposée à l'écran.
- **Écartées** : borne par rôle (trop étroite) ; drapeau « lettrable » par compte (colonne et écran de plus,
  pour un besoin non établi).
- **Réversible** : oui (une fonction).

## C97 — Lettrage : la contre-passation lettre ce qui est libre et ne défait rien ; une écriture lettrée ne se modifie ni ne se supprime
- **Contexte** : la 15-8a/b rend modifiables et supprimables les écritures sans pièce ; `update_in_tx` réécrit
  les lignes. Toutes les annulations passent par `reverse_in_tx_inner`.
- **Retenu** : motif `Lettered` dans `modification_guard` → 409 `ENTRY_LETTERED`, lecture verrouillante après
  le verrou d'en-tête (ferme la course P8-2 d'août) — y compris pour une modification d'en-tête seul (révise la
  clause (ii) d'AC7 d'août). La contre-passation lettre `{L, L'}` pour toute ligne libre sur compte lettrable ;
  une ligne déjà lettrée garde son groupe et son miroir reste ouvert (mouvement nouveau). Les annulations de
  pièce dissolvent leur groupe `document` **avant** le socle. Le lettrage verrouille les en-têtes des
  écritures concernées (ordre `id`), puis les lignes, puis (délettrage) les exercices en mode partagé ;
  rejeu sur interblocage.
- **Écartées** : chemin « en-tête seul » dans `update` (seconde manière de modifier, pour un cas étroit) ;
  délettrage automatique par la contre-passation (bute sur l'exercice clos, et réécrit le passé) ; refus de
  contre-passer une écriture lettrée (correction d'une erreur d'un exercice clos rendue impossible).
- **Réversible** : oui.

## C98 — Lettrage (15-1a2) : synchronisation des pièces et rattrapage par migration classe A
- **Retenu** : `sync_invoice_in_tx` et `sync_supplier_invoice_in_tx`, idempotentes, appelées par l'inventaire
  fermé des écrivains (`invoice_settlements::create_in_tx` — qui couvre règlement, solde et rapprochement —,
  annulation de règlement client et dé-rapprochement, avoir, paiement fournisseur et lots pain.001, annulation
  du règlement ou de la facture fournisseur). Critère du groupe : la somme au grand livre sur le compte de la
  pièce, pas le reste dû (test d'accord entre les deux). Rattrapage des données existantes par une migration
  qui écrit des données, **classe A** (toutes les écritures gardées `lettering_key IS NULL`), première entrée
  de `POST_RESTORE_BACKFILLS` (rejouée après restauration d'une sauvegarde antérieure) ; la duplication de la
  règle (SQL figé par P8, Rust) est tenue par un test d'accord rattrapage ↔ synchronisation. 15-6a (#473)
  recommandée avant.
- **Écartées** : rattrapage en Rust au démarrage (écriture au boot, concurrence, aucun précédent) ; pas de
  rattrapage (les pièces déjà soldées resteraient ouvertes à jamais).
- **Réversible** : la migration, une fois appliquée, non (P8) ; la synchronisation, oui.

## C99 — Lettrage : découpage en 15-1a → 15-1a2 → 15-1b → 15-1c
- **Retenu** : 15-1a socle (schéma, primitive, routes manuelles, gardes, contre-passation) ; **15-1a2** (neuve)
  lettrage des pièces et rattrapage ; 15-1b postes ouverts à une date **et** moteur de proposition (backend,
  repris de l'ancienne 15-1c) ; 15-1c l'écran, le manuel, l'E2E. Chaque story ≤ 5 modules ; la 15-1a avant
  tout (socle). Les propositions se limitent aux paires (le lettrage manuel à N lignes reste possible). Les
  fiches réécrites gardent leur Change Log d'août, précédé d'une entrée « Reprise du 2026-10-08 ».
- **Écartées** : garder trois stories (la 15-1a aurait porté la synchronisation des pièces : au-delà du
  seuil de taille qui avait fait diverger la 15-1 d'août) ; renommer les fiches (les clés du registre et du
  `sprint-status` y renvoient).
- **Réversible** : oui (planification).

## C-15-11b-1 — 15-11b (dev, T0) : alignement sur le livré de la 15-11a — inventaire recompté, tests « valeur vide » remplacés, capture de la 15-11a réutilisée

- **Contexte** : la revue de code P1 de la 15-11a (C-15-11a-6) a fait passer `KESH_ADMIN_BACKUP_DIR` et `KESH_LANG` à `opt_trimmed_env` et donné aux cinq numériques un bras « vide = défaut ». Recompté sur `HEAD` `b2b09f34` : 34 sites (et non 36), `Config::from_env` 24 lectures littérales, 7 appels `opt_trimmed_env`, identifiant `env` 38 fois en production (`config.rs` 33), ensemble lu 41, table `EMPLACEMENTS_AUTORISES` 22 entrées (le nombre 31 des appels `Littéral` de `Config::from_env` est inchangé : 24 + 7). Trois des tests « valeur vide » prescrits (`KESH_ADMIN_BACKUP_DIR`, `KESH_SMTP_PORT`, `KESH_LANG` vides) seraient verts avant le changement : ils ne prouveraient rien. Une capture `tracing` locale existe déjà dans le module de test de `config.rs` (`from_env_with_logs`), avec son témoin.
- **Retenu** : fiche mise à jour (Change Log « Alignement sur le livré ») ; tests remplacés par des cas qui discriminent sur `HEAD` — `KESH_INBOX_DIR=""`, `KESH_PASSWORD_MIN_LENGTH=" 14 "`, `KESH_SMTP_PORT=" 2525 "` (valeur + capture), `KESH_COOKIE_SECURE="   "`, `KESH_LOG_FILE_ROTATION=""` (avertissement collecté par `LogConfig::from_env`) — en plus de `KESH_HOST`, `KESH_JWT_SECRET`, `DATABASE_URL`, `KESH_PORT`, `KESH_DOCUMENTS_DIR` ; capture de la 15-11a réutilisée (DRY) au lieu d'une seconde couche `Layer` ; `reset_env()` complété de six noms (`KESH_DOCUMENTS_DIR`, `KESH_INBOX_DIR`, quatre `KESH_LOG_FILE_*`).
- **Écartées** : garder les trois tests non discriminants comme preuves (ils seraient verts avant le T2 — mémoire « tests qui prouvent moins ») ; écrire la couche `Layer` de la fiche (duplication d'un outil présent) ; s'arrêter après le T0 (aucun écart ne change une règle ni un AC sur le fond : l'AC2 change de cas, pas de règle).
- **Réversible** : oui.

## C-15-11b-2 — 15-11b (dev) : `quote` en dev-dépendance pour l'exclusion `cfg(test)`, garde sans le nombre d'entrées, redondances de trim retirées

- **Contexte** : la règle `cfg(test)` de l'AC3 vaut pour tout élément **et toute instruction** (`syn::Stmt`) ; `syn` 2 n'offre pas d'accès générique aux attributs d'un `Item` ni d'une expression-instruction, et ne ré-exporte pas `ToTokens` (seulement sous `__private`). Par ailleurs, la garde du test assertait `EMPLACEMENTS_AUTORISES.len() == 22`, ce qui faisait rougir une seconde famille sur M13/M14, dont la fiche attend « (L) seule ».
- **Retenu** : `quote = "1"` en dev-dépendance de `kesh-api` (déjà au `Cargo.lock` comme dépendance de `syn` : aucun paquet neuf) — les attributs de tête de tout élément, élément d'`impl`/de trait et instruction se lisent en re-parsant ses jetons (`Attribute::parse_outer`), une seule règle pour tous ; assertion du nombre d'entrées retirée de la garde (le contrôle à nombre exact de chaque entrée rend toute liste vidée ou tronquée rouge de toute façon) ; `.trim()` redondants des numériques d'inbox et du port SMTP retirés (la valeur arrive trimée) ; doc-comment d'`is_template_placeholder` corrigé (« le secret JWT n'est pas trimé à la lecture » devenu faux).
- **Écartées** : énumérer à la main les variantes d'`Item`/`ImplItem`/`TraitItem`/`Expr` (une liste ouverte, précisément ce que le test lexical a abandonné) ; `syn::__private::ToTokens` (API cachée) ; garder l'assertion `== 22` (redondante, et fausserait le relevé des familles des mutations).
- **Réversible** : oui.

## C-15-11b-3 — 15-11b (revue de code P1) : `RUST_LOG` vide testé en lançant le binaire ; `KESH_STATIC_DIR`/`KESH_LOCALES_DIR` vides laissés en angle mort

- **Contexte** : B3 demande un test de comportement du défaut appliqué au vide de `RUST_LOG`, `KESH_STATIC_DIR` et `KESH_LOCALES_DIR`, sous la contrainte de ne toucher aucune ligne de code de production exécutable. `init_tracing` installe un abonné global (non testable en unitaire) ; les deux répertoires sont lus par `main` après la connexion à la base.
- **Retenu** : pour `RUST_LOG`, un test d'intégration qui lance le binaire `kesh-api` (`CARGO_BIN_EXE_kesh-api`, environnement vidé, répertoire temporaire, obligatoires vides → sortie en refus de configuration) et observe si l'avertissement rejoué de `KESH_LOG_FILE_ROTATION` passe ; témoin `RUST_LOG=error`. Pour les deux répertoires, angle mort écrit (en-tête du test, § *Angles morts* de la fiche). Le dédoublonnage des plages exclues (E3) et la normalisation `r#` (E1) sont faits dans le test.
- **Écartées** : extraire une fonction `niveau_de_log()` / `repertoire_statique()` (code de production, interdit à cette remédiation) ; démarrer le binaire complet contre une base pour observer `KESH_LOCALES_DIR` (coût, base partagée, processus à arrêter) ; renommer le champ `exclusions` au lieu de dédoublonner.
- **Réversible** : oui.

## C-15-11b-4 — 15-11b (clôture) : rebasée sur `origin/main` (`5e4bec50`, 15-5d) ; registre et sprint-status par union, PDF régénéré

- **Contexte** : la 15-5d a été mergée pendant la revue de la 15-11b ; elle touche le registre,
  `sprint-status.yaml`, le CHANGELOG et `admin-manual.tex`/`.pdf`.
- **Retenu** : rebase (non merge) ; registre par union (les entrées C89–C99 reportées par la branche après
  C-15-5d-8) ; `sprint-status.yaml` par union, la ligne `last_updated` de la branche renumérotée (24),
  celle de la clôture (25) ; `admin-manual.tex` fusionné par git sans conflit, le PDF binaire pris de la
  branche pendant le rebase puis **régénéré** sur le `.tex` fusionné et contrôlé aplati. CHANGELOG `[0.13.0]`
  sans conflit, une seule rubrique de chaque. Aucun conflit de code ; le test lexical (L) vert sur l'état
  rebasé. Gates complets rejoués (backend, frontend, E2E).
- **Écartées** : merge de `main` dans la branche (historique moins lisible) ; garder le PDF de l'un des deux
  côtés (il aurait omis l'apport de l'autre).
- **Réversible** : oui (rebase ; branche poussée).

## C-15-6-1 — 15-6 : découpée d'emblée en trois (a, b, c)

- **Contexte** : #473 et #474 réunis touchent `kesh-db` (avoir, règlement client, règlement
  fournisseur, comptes bancaires, erreurs), `kesh-api` (rapprochement, comptes bancaires, réglages de
  facturation, erreurs), `kesh-i18n`, trois écrans et le manuel : bien plus de cinq modules — le
  critère de périmètre de la § *Règle de splitting préventif* est franchi avant toute validation.
- **Retenu** : trois sous-stories, comptées en modules métier (la plomberie d'erreur et d'i18n
  suit le geste qu'elle sert, comme pour la 15-5a) :
  - **15-6a** — l'avoir crédite la créance de la vente (#473) : avoir, règlement, rapprochement
    (3 modules) ; elle pose le lecteur partagé du compte de créance.
  - **15-6b** — un règlement ne vise pas le compte qu'il solde (#474, cœur) : règlement client,
    règlement fournisseur, rapprochement, et les deux écrans de règlement (5 modules).
  - **15-6c** — la configuration ne prépare pas l'écriture nulle (#474, voisin) : compte comptable
    d'un compte bancaire, réglages de facturation, écran des comptes bancaires (3 modules).
- **Ordre** : a → b (mêmes fichiers) ; c **après le merge de la 15-5b** (mêmes routes
  `bank_accounts.rs` et `company_invoice_settings.rs`) ; b de préférence après la 15-5b aussi (le
  libellé du nouveau code dans `failed[]` passe par l'écran de #492 que la 15-5b pose).
- **Écartées** : une story unique (règle franchie) ; deux stories « serveur / écrans » (le filtre
  d'écran se séparerait de la garde qu'il reflète, et la story serveur dépasserait encore cinq
  modules).
- **Réversible** : oui, tant qu'aucune n'est développée.

## C-15-6-2 — 15-6a : l'avoir lit la créance sur l'écriture de vente ; arrondi et TVA restent aux réglages

- **Contexte** : l'avoir lit dans les réglages du moment trois comptes que la vente a déjà
  mouvementés : créance (#473), compte de différences d'arrondi (même défaut, non signalé), TVA due.
  Le compte de produit est déjà recopié par ligne (16-1a, D5).
- **Retenu** : la créance se lit sur l'écriture de vente par un **lecteur partagé**
  (`invoice_settlements::sale_receivable_account`) qui remplace aussi les trois copies de la même
  requête (règlement, solde du reste, rapprochement). L'avoir ne dépend plus du réglage
  `default_receivable_account_id`.
- **Laissés, en angles morts écrits dans la fiche** :
  - la **TVA due** reste débitée sur le compte **courant** — c'est la convention écrite du solde du
    reste (doc de `write_off_invoice` : « comme l'avoir ») et la changer toucherait le décompte TVA ;
  - le **compte d'arrondi** reste lu dans les réglages : le lire sur la vente casserait le refus
    nommé de #486 (`a_credit_note_is_refused_when_the_rounding_account_was_archived`, dont le message
    renvoie aux réglages) et demanderait un refus neuf « réactivez tel compte » ; l'écart en jeu est
    d'au plus 2,5 centimes par facture. **Issue à ouvrir** par l'orchestrateur ;
  - le repli sur le produit par défaut des lignes sans compte (D-B2), inchangé.
- **Écartée** : construire l'avoir en miroir ligne à ligne de l'écriture de vente (exact par
  construction, mais refonte de la génération, de la garde des comptes archivés de la 16-1a et de
  leurs tests, hors de la mesure d'un P1).
- **Réversible** : oui.

## C-15-6-3 — 15-6b : un refus dédié, sur les deux modes de règlement, après les contrôles existants

- **Retenu** : nouvelle variante `DbError::SettlementCounterpartyIsClaimAccount`, code stable
  `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` (HTTP 400), `details = { accountId, accountNumber,
  claim: "receivable" | "payable" }`, message 4 locales (deux clés : client, fournisseur). Elle
  s'applique au **virement** autant qu'au **compte interne** (un compte bancaire lié au 1100 produit la
  même écriture nulle), au règlement client, au règlement fournisseur et à l'acceptation d'un
  rapprochement de facture (`failed[]`, même code). Elle vient **après** les refus existants (404,
  configuration, inactif/non imputable) : leur ordre ne change pas.
- **Résolus par le type, sans garde neuve** : compte d'arrondi et comptes des natures de solde,
  revérifiés « charge ou produit » au moment d'écrire — jamais 1100 ni 2000.
- **Écartées** : `InvalidInput("…")` (code générique `INVALID_INPUT`, pas de `details`, contraire à
  la consigne) ; `ACCOUNT_NOT_POSTABLE` de la 15-5a (autre motif : 1100 est imputable).
- **Réversible** : oui.

## C-15-6-4 — 15-6b : le filtre d'écran se fait par rôle, sans champ d'API nouveau

- **Contexte** : l'écran ne connaît pas le compte de créance d'une facture ; l'échéancier ouvre le
  même dialogue depuis une liste.
- **Retenu** : le dialogue de règlement client écarte les comptes de **rôle `Receivable`** et les
  comptes bancaires liés à un tel compte ; la fiche fournisseur, ceux de rôle `Payable`. Les rôles
  sont des singletons par société (`uq_accounts_company_singleton_role`) et les réglages en sont
  dérivés. La garde serveur reste exacte (compte de l'écriture de vente) ; l'écart possible — réglage
  changé depuis — se solde par un refus clair, non par une écriture fausse.
- **Écartée** : exposer `receivableAccountId` sur la fiche et sur les lignes de l'échéancier (deux
  DTO, une sous-requête de plus sur une liste tenue à parité par `invoice_amount_due_parity.rs`).
- **Réversible** : oui.

## C-15-6-5 — 15-6c : la configuration refuse le couple, dans les deux sens, avec exemption « inchangé »

- **Retenu** : le compte comptable d'un compte bancaire ne peut être ni le compte débiteurs ni le
  compte créanciers **des réglages** ; symétriquement, les réglages refusent un compte débiteurs ou
  créanciers lié à un compte bancaire non archivé. Exemption quand la valeur ne change pas (patron
  C4/C10 de la 15-5b) ; contrôle des routes bancaires dans la transaction, comme la 15-5b. Deux
  codes : `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`, `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT`.
- **Assumé** : la course entre les deux gestes (deux administrateurs, deux tables) n'est pas
  verrouillée ; la garde à l'usage de la 15-6b reste le filet. Écrit dans la fiche.
- **Écartée** : ne garder que l'usage (la configuration fautive resterait offerte par l'écran, et
  chaque rapprochement échouerait ensuite un par un).
- **Réversible** : oui.

## C-15-6-6 — Voisin relevé, hors périmètre : la contrepartie égale au compte de banque

- **Constat (lu au code, non exécuté)** : le rapprochement **ventilé** refuse une contrepartie égale
  au compte de banque (`reconciliation.rs:1941-1950`, `:3462-3474`) ; le rapprochement **manuel**
  (`post_manual`, `:2942`) et l'acceptation **par règle** (`accept_one_rule`, `:2207`) ne le font pas,
  et `build_journal_entry_for_counterparty` (`kesh-reconciliation/src/manual.rs:67`) non plus — même
  écriture nulle `D banque / C banque`.
- **Retenu** : hors de #474 (le compte soldé n'est pas une créance) ; signalé à l'orchestrateur pour
  une issue, non traité ici.
- **Réversible** : sans objet.

## C-15-6-7 — 15-6a absorbe #523 : l'arrondi de l'avoir se lit sur la vente ; un compte archivé est refusé par `create_in_tx` (révise C-15-6-2)

- **Contexte** : validation P1 de la 15-6a (findings F1 = R1, HIGH). C-15-6-2 laissait le compte
  d'arrondi de l'avoir aux réglages pour ne pas casser le refus nommé de #486 ; l'orchestrateur a
  ouvert #523 et décidé de le traiter dans la 15-6a (`closes #473, closes #523`).
- **Retenu** : un second lecteur de l'écriture de vente, sur sa **dernière** ligne
  (`ORDER BY jel.id DESC LIMIT 1`), **recoupée** avec l'arrondi figé sur la facture (sens et valeur
  absolue), sinon `DbError::Invariant` ; appelé seulement si l'arrondi ≠ 0. Compte de la vente archivé
  depuis : **voie (b)** — `journal_entries::create_in_tx` refuse en `INACTIVE_OR_INVALID_ACCOUNTS`,
  comme pour la créance. Le test `invoices_validate_vat.rs:846` change d'assertion, délibérément, et
  la fiche l'écrit comme tel. Trois tests neufs : réglage redésigné, réglage vidé, recoupement.
- **Écartées** : (a) garder `RoundingAccountNotConfigured { Issuance }` en vérifiant le compte de la
  vente — son message renvoie aux paramètres, remède **faux** puisque l'avoir ne les lit plus ; (c) une
  variante neuve nommant le compte de la vente — un refus de plus pour un cas que le refus générique
  existant couvre déjà, comme pour la créance ; réutiliser `usable_designated_account` — contraire à la
  doctrine « mêmes comptes que l'origine, seule l'inactivité bloque ».
- **Réversible** : oui, tant que la 15-6a n'est pas développée.

## C-15-6-8 — 15-6a : la TVA due de l'avoir reste un angle mort, tracé par #525 ; le manuel dit la limite

- **Contexte** : finding F3 (MEDIUM) de la 15-6a — l'avoir débite la TVA due sur le compte **courant**
  des réglages ; même classe que #473. L'orchestrateur a ouvert **#525** (P1, jalon de la TVA, report
  assumé).
- **Retenu** : la fiche cite #525 comme angle mort tracé. Le manuel **ne présente pas** l'écriture
  comme juste : il dit que la TVA de l'avoir suit le compte de TVA due actuellement désigné, et que
  changer ce réglage entre une facture et son avoir est une limite connue de cette version.
- **Écartée** : le traiter dans la 15-6a (toucherait le décompte TVA et la convention partagée avec le
  solde du reste, `vat_payable_account_for_write` — c'est l'objet du jalon de la TVA).
- **Réversible** : oui.

## C-15-6-9 — #524 devient la sous-story 15-6d ; même refus que le flux ventilé existant (révise C-15-6-6)

- **Contexte** : validation P1 de la 15-6b — finding F1 (HIGH) : #524 doit être traité ; finding F4
  (MEDIUM) : avec lui et les lots de paiement, la 15-6b dépasse cinq modules. C-15-6-6 le laissait
  hors périmètre ; **il est retiré** par la présente entrée.
- **Retenu** : sous-story **15-6d-contrepartie-distincte-de-la-banque** (`closes #524`).
  `post_manual` et `accept_one_rule` refusent une contrepartie égale au compte de la banque **avec le
  même refus que le flux ventilé existant** : manuel → `AppError::Validation` comme `post_split` (400,
  message français en dur — limite assumée de ce flux existant, écrite comme telle) ; règle →
  `FailedProposal VALIDATION_ERROR`, `details.reason = "counterparty_equals_bank_ledger"`, HTTP 200,
  comme `accept_one_split`. Ordre : 404, puis `ACCOUNT_NOT_POSTABLE` (15-5b), puis ce refus. L'écran du
  rapprochement manuel ne propose pas le compte de la banque (test Vitest). Dépend de la 15-5b.
- **Écartées** : un code dédié et traduit (divergerait des deux chemins ventilés existants ; s'il
  devient souhaitable, il vaudra pour les quatre chemins, story à part) ; garder #524 dans la 15-6b
  (règle de découpage franchie).
- **Réversible** : oui, tant qu'elle n'est pas développée.

## C-15-6-10 — 15-6b : les lots de paiement refusent le compte créanciers à la création, et gardent le refus à la confirmation

- **Contexte** : findings F2 (HIGH) / R4 de la 15-6b — `confirm_batch` est le troisième appelant de
  `pay_in_tx`. Avec la seule garde de `pay_in_tx`, un compte bancaire lié au compte créanciers produit
  un lot **et son fichier pain.001**, puis un lot inconfirmable — alors que le fichier est peut-être
  déjà à la banque.
- **Retenu** : `create_batch` compare, **par facture**, la dette de l'écriture d'achat au compte lié du
  compte bancaire source **avant** de produire le fichier : égalité → la facture va dans `failed[]` du
  lot (`SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, `details` avec `claim: "payable"`), patron batch ;
  aucune facture retenue → aucun lot, aucun fichier. `confirm_batch` garde le refus de `pay_in_tx`
  (le compte bancaire a pu être relié entre-temps) : 400, lot toujours `generated`. Libellé à l'écran
  des lots, manuel (§ pain.001), `docs/api-external.md`.
- **Écartées** : refuser le lot entier en erreur globale (contraire au patron batch : une facture
  fautive ne doit pas bloquer les autres) ; ne garder que la confirmation (le défaut même du finding).
- **Réversible** : oui.

## C-15-6-11 — 15-6b : les gardes réutilisent les lecteurs ; côté fournisseur, un lecteur sœur

- **Contexte** : finding R7 (MEDIUM) de la 15-6b — la dépendance au lecteur de la 15-6a était déclarée
  mais inemployée.
- **Retenu** : côté client (`settle_invoice`, `accept_one_invoice`), la garde compare l'identifiant que
  le lecteur `sale_receivable_account` de la 15-6a a **déjà** rendu — aucune requête neuve. Côté
  fournisseur, la requête est **différente** (ligne de **crédit** de l'écriture d'achat **et** le TTC,
  `supplier_invoices.rs:582-593`) : elle est extraite en lecteur sœur `purchase_payable_line`, partagé
  par `pay_in_tx` et la création de lot (C-15-6-10).
- **Écartée** : faire lire la dette par le lecteur de la créance (autre sens, autre montant).
- **Réversible** : oui.

## C-15-6-12 — #492 relève de la 15-5c, non de la 15-5b (révise l'ordre de C-15-6-1)

- **Contexte** : findings F3 = R5 de la 15-6b. C-15-6-1 disait « le libellé du nouveau code dans
  `failed[]` passe par l'écran de #492 que la 15-5b pose » ; depuis C15, #492 est dans la **15-5c**
  (`frontend/src/lib/features/reconciliation/failed-proposal-label.ts`).
- **Retenu** : la 15-6b est « de préférence après la 15-5c » ; son développeur ajoute le code au module
  si la 15-5c est mergée, sinon il le signale (pas de second mécanisme). Fiches 15-6 et 15-6b corrigées.
- **Réversible** : sans objet (correction d'une référence).

## C-15-6-13 — 15-6c : contrôles dans la transaction, et les deux gestes se sérialisent (révise C-15-6-5)

- **Contexte** : findings F1 et F2 (HIGH) de la 15-6c — le contrôle des réglages se faisait hors
  transaction, contre une valeur lue sans verrou ; la course entre les deux gestes était déclarée en
  angle mort alors qu'elle se ferme à bon marché.
- **Retenu** : deux lecteurs uniques — `company_invoice_settings::claim_accounts_for_share` (`FOR
  SHARE` sur la ligne des réglages) et `bank_accounts::first_active_bank_account_linked_to` (`FOR
  SHARE`, premier compte bancaire non archivé par `id`) — et un **ordre de verrous unique** : la ligne
  des réglages d'abord, les comptes bancaires ensuite. Côté compte bancaire, la lecture des réglages
  est la première de la transaction (après le verrou sentinelle, avant tout `FOR UPDATE` de ligne
  bancaire) ; côté réglages, `company_invoice_settings::update` lit `before` en `FOR UPDATE`, compare
  contre lui (exemption « inchangé »), puis interroge les comptes bancaires. **L'angle mort « course »
  est supprimé.** Variantes nommées : `DbError::BankAccountLedgerIsClaimAccount`,
  `DbError::ClaimAccountLinkedToBankAccount`. **Ordre des erreurs** : routes bancaires — celui de C10
  (le refus neuf en dernier, **après** le 409 de version et le non-imputable) ; route des réglages —
  forme → refus → 409, tous les 400 de cette route précédant déjà le 409.
- **Écartées** : contrôle dans le handler (valeur lue hors verrou, F1) ; verrou sentinelle `companies`
  sur les quatre gestes (aurait sérialisé aussi, mais le PATCH et le PUT des réglages ne le prennent
  pas, et la sentinelle sérialise bien plus que ces deux gestes) ; lire les réglages **après** la
  ligne bancaire (ordre inverse → interblocage entre les deux gestes).
- **Réversible** : oui, tant qu'elle n'est pas développée.

## C-15-6-14 — 15-6b/15-6c : des fonctions d'écran par ensemble d'identifiants ; la 15-6b les nourrit par le rôle, la 15-6c par les réglages (précise C-15-6-4)

- **Contexte** : finding F3 (MEDIUM) de la 15-6c — un écran filtré par **rôle** et un serveur qui
  refuse selon les **réglages** divergent ; finding R2 (MEDIUM) — la signature du helper de la 15-6b
  (un rôle) ne servait pas la 15-6c (deux comptes, et pas par rôle).
- **Retenu** : trois fonctions pures dans `account-options.ts`, posées par la 15-6b :
  `accountIdsWithRole`, `withoutAccountIds`, `bankAccountsNotLinkedTo`, et des identifiants calculés
  sur la liste **complète** des comptes. La 15-6b les nourrit par le rôle (C-15-6-4 maintenu : le
  dialogue de règlement ne connaît pas la créance de la facture) ; la 15-6c par les identifiants
  **désignés dans les réglages** (`getInvoiceSettings`, lisible par tout rôle), comme son serveur, et
  l'écran des réglages par les comptes liés à un compte bancaire non archivé.
- **Écartée** : un helper par story (deux filtres voisins dans le même module, qui dériveraient).
- **Réversible** : oui.

## C-15-6-15 — 15-6c : la spec E2E qui liait un compte bancaire au 1100 le lie au 1000

- **Contexte** : finding R4 (MEDIUM) de la 15-6c. Vérifié au code : le préréglage E2E `with-company`
  (`seed_accounting_company`, `test_fixtures.rs:88-107`) désigne **1100 « Banque CI »** comme compte
  débiteurs ; `bank-account-journal-link.spec.ts:105-119` lie un compte bancaire au 1100 par l'écran.
  Après la 15-6c, le menu ne le propose plus et le serveur le refuse. Le seed de dev
  (`scripts/seed-dev-db.sql`) ne crée pas de réglages : non concerné.
- **Retenu** : la spec lie au **1000 « Caisse CI »** (actif, imputable, non désigné comme créance).
- **Écartée** : changer la fixture partagée (désigner un autre compte débiteurs) — elle sert des
  204 appels dans 41 fichiers (`grep -rho "seed_accounting_company(" crates/`, relevé le 2026-10-08),
  dont beaucoup supposent la créance sur 1100.
- **Réversible** : oui.

## C-15-6-16 — 15-6a : les comptes lus sur la vente sont verrouillés ; un compte archivé est refusé par le refus nommé de la contre-passation (révise la voie (b) de C-15-6-7)

- **Contexte** : validation P2 de la 15-6a — R-2 = F-1 (MEDIUM) : remplacer `rounding_account_for_write`
  par un lecteur sans verrou retirait le `FOR UPDATE` que #486 posait sur le compte d'arrondi ; la garde
  `active` de `create_in_tx` est une lecture simple, qui sous REPEATABLE READ ne voit pas un archivage
  validé entre-temps. R-3 = F-2 (MEDIUM) : la voie (b) rendait un refus anonyme
  (`INACTIVE_OR_INVALID_ACCOUNTS`), que la 6 ter de la même fonction avait justement écarté pour les
  comptes de produit. Décisions de l'orchestrateur : fermer la course, nommer le compte par la forme
  la plus simple.
- **Retenu** : après les deux lecteurs et la garde 6 ter, `create_credit_note` verrouille en une
  instruction les lignes `accounts` de la créance et de l'arrondi de la vente
  (`SELECT id, number, active … ORDER BY id FOR UPDATE`, **sans** filtre `active`) et lit l'état actif
  **dans** cette lecture verrouillante. Un compte archivé → `DbError::ReversalAccountsArchived`
  (400 `ACCOUNT_ARCHIVED`, `details.rejected[]`, clé `journal-entries-reverse-account-archived`
  existante) : l'avoir est une contre-passation, c'est le refus que la contre-passation rend pour le
  même cas. Aucune variante, aucun code, aucune clé neuve. La garde de `create_in_tx` reste, en filet.
  Ordre des verrous : facture → règlements → avoir existant → comptes (celui de `validate_invoice`).
  Test neuf de la course (attente puis refus nommé). Manuel : `keshwarning` au § *Avoirs*,
  avertissement du plan comptable étendu, phrase du manuel d'administration sur l'arrondi.
- **Concilie** deux consignes : « le refus reste celui de `create_in_tx` » (pas de refus neuf — tenu :
  aucun code neuf) et « nommer le compte » (tenu par le refus nommé existant). Lire `active` par la
  garde de `create_in_tx` après le verrou n'aurait pas suffi : sa lecture simple rend l'instantané.
- **Écartées** : une variante neuve propre à l'avoir (un refus de plus pour un cas que la
  contre-passation nomme déjà) ; garder `INACTIVE_OR_INVALID_ACCOUNTS` anonyme et le compenser au
  manuel (l'utilisateur ne saurait pas quel compte réactiver) ; assumer la course en dette LOW (un site
  aujourd'hui verrouillé l'aurait perdu).
- **Conséquence assumée** : deux avoirs simultanés sur des factures de même compte débiteurs se
  sérialisent sur la ligne de ce compte, le temps d'une transaction. Le code du refus change pour
  l'arrondi (`ROUNDING_ACCOUNT_NOT_CONFIGURED` → `ACCOUNT_ARCHIVED`) : l'esprit de la contrainte de
  #523 (refus nommé) est tenu, sa lettre non — à commenter sur #523.
- **Réversible** : oui, tant que la 15-6a n'est pas développée.

## C-15-6-17 — 15-6b : les écrans renoncent aux comptes archivés ; clés d'écran conformes au lint

- **Contexte** : validation P2 de la 15-6b — R-1 = F1 (MEDIUM) : l'AC8 promettait d'écarter un compte
  bancaire lié à un compte de rôle `Receivable` **archivé**, mais les trois écrans chargent le plan sans
  les archivés ; F2 (MEDIUM) : la clé `invoice-settle-no-eligible-bank-account` faisait rougir
  `lint-i18n-ownership` dans `src/lib/features/invoices/`.
- **Retenu** : l'AC8 **renonce** aux archivés — un compte archivé ne peut recevoir aucune écriture (la
  garde `active` de `create_in_tx` est inconditionnelle), la garde serveur suffit ; les ids se
  calculent avant le filtre `active && postable` (cas non imputable, qui lui arrive à l'écran). Clés
  renommées `invoices-settle-no-eligible-bank-account` et `supplier-invoices-pay-no-eligible-bank-account`
  (alignée sur ses voisines).
- **Écartées** : charger `fetchAccounts(true)` dans trois écrans pour un cas que le serveur refuse
  déjà ; allonger `KNOWN_VIOLATIONS` (dette #30) au lieu de nommer la clé correctement.
- **Réversible** : oui.

## C-15-6-18 — 15-6b : les comptes d'écart sont comparés à la créance par identifiant (révise le « résolu par le type » de C-15-6-3)

- **Contexte** : finding F3 (MEDIUM) de la 15-6b — le « résolu par le type » reposait sur un invariant
  qu'un geste ultérieur défait : le type d'un compte porteur d'écritures se change avec confirmation
  (`accounts.rs:497-512`, `confirm_retype`), et le 1100 devenu `Expense` porte toujours la créance des
  ventes antérieures ; `write_off_invoice` écrirait `D 1100 / C 1100`, l'arrondi du règlement de même.
- **Retenu** : le même refus `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, par comparaison
  d'identifiants avec la créance lue sur la vente, garde le compte d'arrondi de `settle_invoice` (4bis)
  et de `accept_one_invoice` (c-bis) et les comptes de `write_off_invoice` (nature, reste d'arrondi, TVA
  due) — AC3 bis, trois tests. Le contrôle de type existant reste. Le test qui figeait le « résolu par
  le type » est retiré.
- **Écartée** : écrire le chemin du changement de type en angle mort avec une issue (trois `if` sur des
  identifiants déjà lus coûtent moins qu'une dette tracée).
- **Réversible** : oui.

## C-15-6-19 — 15-6b : lots — refus contextualisé à la confirmation, écriture d'achat malformée en `failed[]`, écran de création non filtré

- **Contexte** : findings F4 (MEDIUM), R-7 = F10 et F6 (LOW) de la 15-6b. Le refus à la confirmation
  d'un lot disait « par un autre compte » alors que l'utilisateur n'en choisit aucun, et que le fichier
  pain.001 a pu être exécuté par la banque ; le cas « écriture d'achat sans ligne de crédit » à la
  création n'était pas dit ; l'écran de création d'un lot propose un compte bancaire lié au compte
  créanciers.
- **Retenu** : la variante porte un contexte de lot optionnel (`SettlementBatchContext`), rempli par
  `confirm_batch` ; clé propre `error-settlement-counterparty-is-payable-in-batch` qui dit les deux
  issues (relier de nouveau le compte bancaire puis confirmer, ou annuler le lot et régler depuis la
  fiche), `details.paymentBatchId` / `supplierInvoiceId`, manuel ; test du remède. Le refus à la
  création reste la défense principale. À la création, une écriture d'achat sans ligne de crédit est
  une donnée de **cette** facture : item `failed[]` `SUPPLIER_INVOICE_PURCHASE_ENTRY_MALFORMED`
  (patron `INVOICE_SALE_ENTRY_MALFORMED`, § *Pattern batch*), libellé à l'écran des lots. L'écran de
  création n'est **pas** filtré : il ne charge pas le plan comptable, et le refus y arrive par facture,
  libellé, avant tout fichier — angle mort écrit.
- **Écartées** : le message du règlement unitaire à la confirmation (remède faux) ; une erreur globale
  (500) pour l'écriture d'achat malformée (contraire au patron batch) ; charger le plan comptable sur
  l'écran de création pour un filtre que le serveur double.
- **Réversible** : oui.

## C-15-6-20 — 15-6b : décompte des modules refait, signal D5 déclaré, pas de découpage

- **Contexte** : finding F5 (MEDIUM) de la 15-6b — la P1 avait ajouté les lots sans recompter.
- **Retenu** : décompte écrit dans la fiche — huit modules au barème des fichiers, cinq au barème du
  geste (patron de la 15-5a). Signal **déclaré au Project Lead** ; pas de découpage : les gestes ne sont
  pas indépendants (une variante, un `ClaimSide`, un lecteur sœur partagé par `pay_in_tx` et les lots,
  trois fonctions d'écran ; les lots n'appellent que la garde de `pay_in_tx`), et aucun défaut de la P2
  ne recycle un défaut de la P1.
- **Écartée** : sortir les lots en sous-story (elle naîtrait sans sa garde, ou dupliquerait la
  plomberie d'erreur et d'i18n).
- **Réversible** : oui — un découpage reste possible tant que la story n'est pas développée, sur
  arbitrage de Guy.

## C-15-6-21 — 15-6c : `LOCK IN SHARE MODE` ; route des réglages, le 409 d'abord ; son code n'est pas pour les intégrateurs (révise C-15-6-13)

- **Contexte** : validation P2 de la 15-6c — R2-1 = F1 (HIGH) : `FOR SHARE`, écrit par C-15-6-13, est
  une erreur de syntaxe (1064) sur MariaDB 10.11 ; F2 (MEDIUM) : le refus des réglages avant le 409
  jugeait un changement contre un état que le client n'avait pas vu ; R2-3 = F3 (MEDIUM) : la route des
  réglages est fermée aux clés d'API.
- **Retenu** : `LOCK IN SHARE MODE` dans les deux lecteurs (graphie du dépôt,
  `opening_complement.rs:38-39`), lecteur renommé `claim_accounts_in_share_mode` ; route des réglages :
  forme → 409 version → 400 `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT` → no-op ; `docs/api-external.md` ne
  documente que `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT` (routes bancaires, ouvertes aux clés). La graphie
  `FOR SHARE` de C-15-6-13 est **fausse** ; cette entrée la corrige (l'entrée d'origine n'est pas
  réécrite).
- **Écartées** : garder le refus avant le 409 « parce que les autres 400 de la route le précèdent »
  (ceux-là ne dépendent que du corps) ; lister le second code au guide comme « interne » (le guide
  documente l'API par clé : un code qu'elle ne peut pas rendre n'y a pas sa place).
- **Réversible** : oui.

## C-15-6-22 — 15-6c : appels directs au dépôt, fichiers de test de la 15-5b, code mort, signal D5

- **Contexte** : validation P2 de la 15-6c — R2-4 (MEDIUM) : 14 sites de test appellent directement les
  deux fonctions du dépôt dont la signature change ; R2-2 = F4 (MEDIUM) : deux fichiers de test
  doublaient ceux que crée la 15-5b ; F6 (LOW) : `BankAccountList.svelte`, second appelant du formulaire
  de lien, n'est importé nulle part ; F8 : signal de découpage (le HIGH de la P2 naît de la remédiation
  P1). Consigne de l'orchestrateur : tenir compte de la 15-5d, qui expose `defaultPayableAccountId` à
  l'écran des réglages.
- **Retenu** : les appels directs passent `&ClaimAccounts::default()` (la garde n'est l'objet d'aucun
  d'eux ; aucun test ne change de sens) ; les tests 6 et 14 étendent `company_invoice_settings_postable_e2e.rs`
  et `bank-accounts/+page.test.ts` de la 15-5b ; `BankAccountList.svelte` est supprimé, la prop
  `claimAccountIds` obligatoire ; le menu créanciers de la 15-5d est filtré comme celui des débiteurs.
  Signal D5 **déclaré au Project Lead**, pas de découpage : le défaut est une graphie de mot-clé, non
  une conception qui tourne en rond, et séparer les deux sens casserait l'ordre de verrous qui les
  tient ensemble. Références de fixture de C-15-6-15 (`test_fixtures.rs:88-107`) **fausses** : lire
  `:80-172`, désignation `:157-172`.
- **Écartées** : passer les réglages réels de la fixture aux appels directs (ferait rougir un test qui
  lie au 1100, sans rapport avec son objet) ; prop optionnelle (laisserait le composant mort non
  filtré, en silence).
- **Réversible** : oui ; la suppression de `BankAccountList.svelte` se défait par l'historique.

## C-15-6-23 — 15-6d : l'égalité avec le compte de banque testée d'abord ; règle sur ce compte plus proposée ; filtre d'écran local (révise l'ordre de C-15-6-9)

- **Contexte** : validation P1 de la 15-6d — F1 (MEDIUM) : la fiche plaçait le refus après le 404 et
  `ACCOUNT_NOT_POSTABLE`, à l'inverse du flux ventilé qu'elle prétendait reproduire ; F2 (MEDIUM) :
  `get_proposals` proposait une règle sur le compte de banque, que l'acceptation refuserait toujours,
  et qui masquait une règle suivante ; R-1 = F3, R-3 (MEDIUM) : câblage de l'écran non testé, filtre
  conditionné à la 15-6b. Décisions de l'orchestrateur.
- **Retenu** : `post_manual` compare juste après l'étape 2 bis, `accept_one_rule` juste après l'étape 4
  — avant 404 et non imputable, et avant la recherche de la transaction en attente (une transaction
  déjà rapprochée rend 400) ; même réponse sur tous les chemins. `get_proposals` retire le compte de
  banque de l'ensemble passé à `first_matching_rule` (patron 15-5b AC5). Écran : `ReconciliationProposals`
  résout le compte de banque par `listBankAccounts()` (effet dépendant de `bankAccountId`, garde de
  génération ; repli `null` = aucun filtrage), testé dans son fichier existant (mock de
  `bank-accounts.api`) ; filtre **local** d'une condition dans `ManualMatchModal`. L'écran de
  ventilation reste non filtré (angle mort écrit). Course résiduelle de `post_manual` écrite.
- **Écartées** : garder l'ordre « après la 15-5b » (réponse différente selon le chemin, message
  trompeur) ; faire passer le compte de banque par la page (`/companies/current` le porte, mais la page
  n'a pas de test : le câblage resterait non vérifié) ; `withoutAccountIds` de la 15-6b (dépendance pour
  un seul id).
- **Réversible** : oui, tant qu'elle n'est pas développée.

## C-15-6-24 — 15-6a : comptes de la vente verrouillés EN PARTAGE et AVANT l'exercice (rectifie C-15-6-16)

- **Contexte** : validation P3 de la 15-6a — R3-1 (HIGH, né de la remédiation P2) : C-15-6-16 plaçait
  le verrou des comptes de la vente (créance, arrondi) **après** `fiscal_years` et la séquence, et
  affirmait « c'est l'ordre de `validate_invoice` » ; c'est l'inverse de l'ordre canonique
  (`invoices.rs:1916-1925` : `invoices` → `accounts` (1 bis) → `fiscal_years` → séquences →
  écriture), et la 15-5d verrouille la créance dans `validate_invoice` avant l'exercice : cycle sur la
  ligne 1100 entre chaque validation et chaque avoir. F-1 (MEDIUM) : un `FOR UPDATE` (X) sur la
  créance ouvrait un cycle neuf avec `accept_one_invoice`, qui tient S sur 1100 (clé étrangère des
  lignes insérées) avant de demander X sur la facture, que l'avoir tient. F-8 : la « conséquence
  assumée » (deux avoirs se sérialisent sur la ligne du compte) était fausse. Décision de
  l'orchestrateur.
- **Retenu** : les deux lecteurs sont appelés juste après les réglages (étape (3)) ; les comptes de la
  vente sont verrouillés en une instruction `… ORDER BY id LOCK IN SHARE MODE`, **avant** l'exercice
  (4) et la séquence (5) — ordre : facture → règlements → avoir existant → réglages → comptes de la
  vente (S) → exercice → séquence → écriture, conforme à la validation, au règlement et à la 15-5d. Un
  S suffit à faire attendre `accounts::archive` (X) et rend l'état courant de `active` ; S + S ne
  bloque ni le rapprochement ni un second avoir. Ordre des refus qui en découle : compte de la vente
  archivé avant `FISCAL_YEAR_INVALID`, `creditNoteTotalZero` et `CREDIT_NOTE_REVENUE_ACCOUNT_ARCHIVED`
  (test 14). Test de course sur `attendre_une_requete_en_cours` (motif `LOCK IN SHARE MODE`). Ce que
  le verrou ne couvre pas — comptes de produit (6 ter) et TVA due, lus sans verrou — est écrit comme
  angle mort assumé, préexistant.
- **Rectification de C-15-6-16** (entrée non réécrite) : ses phrases « ordre des verrous : … comptes
  de la vente. C'est celui de `validate_invoice` » et « deux avoirs simultanés … se sérialisent sur la
  ligne de ce compte » sont **fausses** ; le mode `FOR UPDATE` qu'elle prescrivait est remplacé par
  `LOCK IN SHARE MODE`. Le reste de C-15-6-16 (comptes verrouillés sans filtre `active`, état lu sous
  verrou, refus nommé) tient, le refus étant précisé par C-15-6-25.
- **Écartées** : garder `FOR UPDATE` en le déclarant risque connu (cycle avec le rapprochement sur le
  compte le plus mouvementé du grand livre) ; garder la place après l'exercice en écrivant l'inversion
  comme angle mort (cycle avec chaque validation dès la 15-5d) ; joindre les comptes de produit et de
  TVA due au même verrou (exige de faire lire la 6 ter sous verrou ; hors sujet de #473/#523).
- **Réversible** : oui, tant qu'elle n'est pas développée.

## C-15-6-25 — 15-6a : un seul vocabulaire de refus pour l'avoir — variante propre, code `ACCOUNT_ARCHIVED`, libellé « Impossible d'émettre l'avoir » (révise C-15-6-16)

- **Contexte** : validation P3 de la 15-6a — F-9 (LOW) : sur `POST /api/v1/credit-notes`, un compte
  de produit archivé rend `CREDIT_NOTE_REVENUE_ACCOUNT_ARCHIVED` (« Impossible d'émettre l'avoir —
  … ») et un compte de la vente archivé rendait, par C-15-6-16, `ACCOUNT_ARCHIVED` avec le message de
  la contre-passation (« Impossible de contre-passer — … ») : deux vocabulaires pour un même geste.
  L'orchestrateur demande de trancher entre étendre `CREDIT_NOTE_REVENUE_ACCOUNT_ARCHIVED` et un
  libellé « Impossible d'émettre l'avoir » pour `ACCOUNT_ARCHIVED` sur cette route.
- **Retenu** : la seconde voie. Variante `DbError::CreditNoteAccountsArchived(Vec<ArchivedAccount>)`,
  **même code** `ACCOUNT_ARCHIVED`, même `details.rejected[]` (`accountId`, `accountNumber`) que la
  contre-passation ; clé neuve `credit-note-account-archived` dans les quatre locales — fr :
  « Impossible d'émettre l'avoir — compte(s) archivé(s) : { $detail }. Réactivez le ou les comptes
  concernés. » Le manuel et le CHANGELOG disent les deux codes de la route (par ligne / par compte de
  la vente). Révise le « aucune variante, aucun code, aucune clé neuve » de C-15-6-16 : aucun **code**
  neuf, une variante et une clé.
- **Écartées** : étendre `CREDIT_NOTE_REVENUE_ACCOUNT_ARCHIVED` (sa structure `RejectedRevenueAccount`
  désigne une **ligne de facture** ou le produit par défaut, et son nom dit « produit » : y loger la
  créance mentirait au lecteur de l'API et exigerait un sujet neuf dans `format_rejected_revenue_accounts`) ;
  garder le message de la contre-passation (« contre-passer » face au bouton « Créer un avoir »,
  incohérent avec le refus voisin de la même route).
- **Réversible** : oui, tant qu'elle n'est pas développée ; après, retirer la variante ne change pas le
  code rendu.

## C-15-6-26 — 15-6b : un code, un rôle, deux remèdes ; tables de refus du guide ; comparaison séparée de la construction du refus (révise C-15-6-18 ; C-15-6-3 révisé par C-15-6-18 et ce choix)

- **Contexte** : validation P3 de la 15-6b (lentilles R et F, Sonnet ; remédiation Opus 5.5).
  R1 = F1 (MEDIUM) : l'AC3 bis ajoutée en P2 (C-15-6-18) rendait, pour un compte d'arrondi, de nature
  ou de TVA due **désigné dans les réglages**, le message du règlement unitaire, qui renvoie à « un
  autre compte (banque, caisse…) » alors que l'utilisateur n'a choisi aucun compte — cas du dialogue
  de solde, qui affiche `err.message` (`invoices/[id]/+page.svelte:509`). R2 = F2 (MEDIUM) : l'AC10
  visait le § 10 générique de `docs/api-external.md` et oubliait la table des refus du solde
  (`:274-283`). F3 (MEDIUM) : six à sept sites répétaient comparaison, lecture du numéro et
  construction du refus. Le corps de C-15-6-3 dit encore « deux clés » et « résolu par le type » pour
  les comptes d'écart : révisé par C-15-6-18 puis par ce choix, non réécrit. Décisions de
  l'orchestrateur.
- **Retenu** : (a) un seul code `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, discriminant
  `role: SettlementAccountRole` dans la variante et en `details.role` (`counterparty`, `rounding`,
  `write_off_nature`, `vat_payable`) ; quatrième clé `error-settlement-designated-account-is-receivable`
  (sélecteur Fluent sur `$role`) qui renvoie à *Paramètres → Facturation* ; deux cas dans le libellé
  `failed[]` du rapprochement et au manuel ; `claim: Payable` ne se combine qu'avec `counterparty`.
  (b) Guide : une ligne dans la table du solde, une phrase « Refus » sous le § des règlements client et
  sous celui du règlement fournisseur (qui couvre les deux routes de lot) ; le § 10 n'est pas touché.
  (c) Helper commun dans `invoice_settlements.rs` : `ensure_not_claim_account` (comparaison **pure**,
  seule partie empruntée par la 15-6d), `claim_account_refusal` (lit le numéro à l'échec seulement),
  `claim_account_refusal_details` (mapping HTTP, `FailedProposal`, `PaymentBatchFailedItem`).
  (d) `confirm_batch` relit `supplier_invoice_number` par une `SELECT` ; la variante ne gagne pas de
  champ. (e) Signal D5 levé (MEDIUM → MEDIUM, deux défauts nés de la remédiation P2, confinés à
  l'AC3 bis), déclaré, sans découpage ; la P4 est une passe complète.
- **Écartées** : un second code pour le compte désigné (même défaut `D X / C X`, un intégrateur le
  traite d'un seul bras ; la différence de remède est une information) ; un message unique reformulé
  (vague pour les deux cas) ; inscrire le code au § 10 (rompt la convention) ; un helper qui compare et
  construit en une fonction (inempruntable par la 15-6d, dont le refus est le `VALIDATION_ERROR` du flux
  ventilé) ; ajouter `supplier_invoice_number` à `pay_in_tx` ou à la variante.
- **Réversible** : oui avant le développement ; après, `details.role` et la quatrième clé sont des
  ajouts compatibles, retirer `role` romprait un contrat d'API documenté.

## C-15-6-27 — 15-6c : le `FOR UPDATE` de `before` testé, attente prouvée, pas de verrou pour une dé-liaison, place du manuel, routes bancaires au guide

- **Contexte** : validation P3 de la 15-6c (lentilles R et F, Sonnet ; remédiation Opus 5.5) — 3
  MEDIUM, 11 LOW. F1 : le `FOR UPDATE` de `before` dans `company_invoice_settings::update` ferme une
  course, mais aucun test ne l'épinglait (les tests 11 et 12 passent avec un `SELECT` simple). F2 : les
  attentes des tests étaient bornées à 300 ms. R3-1 : le paragraphe du manuel d'administration était
  placé juste avant un paragraphe sans en-tête, qui en serait devenu la suite. F8 : les routes
  `/bank-accounts` ne figurent nulle part dans `docs/api-external.md`. Décisions de l'orchestrateur.
- **Retenu** : test 12 bis — une transaction tenue pose S sur les réglages par
  `claim_accounts_in_share_mode`, le PUT doit attendre au `FOR UPDATE` (rougit si on le retire) ;
  tests 11, 12 et 12 bis prouvent l'attente par `attendre_une_requete_en_cours`, jamais par un délai ;
  une cible `NULL` (dé-liaison, création ou remplacement sans compte) ne lit pas les réglages et ne pose
  aucun verrou S (`ClaimAccounts::default()`) ; `$account` = numéro du compte lu sans verrou après la
  décision du refus, repli `#<id>` ; `$bank` = `bank_name` ; paragraphe du manuel d'administration
  après `:2033` (borne `:2017-2033`), qui présente d'abord les comptes débiteurs et créanciers puis la
  règle réciproque ; `docs/api-external.md` : une ligne *Comptes bancaires* au § 7 et le code au § 10,
  sans section neuve ; portée réelle des verrous dite (next-key et intervalle, attente transitoire
  possible, non mesurée) ; tests Vitest existants adaptés (`BankAccountJournalLinkForm.test.ts`,
  `settings-invoicing-page.test.ts`), aucun ne change de sens.
- **Écartées** : garder le délai de 300 ms (passe à vide sur machine chargée) ; verrouiller aussi pour
  une dé-liaison (attente inutile, le contrôle ne joue jamais sur `NULL`) ; insérer le paragraphe après
  *Montant minimum* (`:2031`) ; documenter toute la ressource bancaire au guide (hors périmètre) ; ne
  rien ajouter au § 7 (le § 10 renverrait à des routes que le guide ne présente pas).
- **Réversible** : oui, la fiche n'est pas codée.

## C-15-6-28 — 15-6d : contrôle du compte de banque actif sur le chemin par règle ; gardes ventilées figées ; pas de garde de génération ; comparaison empruntée au helper de la 15-6b (complète C-15-6-23)

- **Contexte** : validation P2 de la 15-6d (lentilles R et F, Opus ; remédiation Opus 5.5) — R-1
  (MEDIUM) = F-4 (LOW) : `accept_one_rule` ne contrôle pas l'activité du compte de banque, que les
  trois autres chemins contrôlent avant leur garde d'égalité ; « même réponse sur tous les chemins »
  était faux pour un compte de banque archivé. F-2 (MEDIUM) : l'ordre « égalité avant postabilité » de
  C-15-6-23 reposait sur deux gardes ventilées qu'aucun test n'exerce. F-6 (LOW) : la garde de
  génération protégeait un cas que la page ne produit pas (`{#key selectedId}`). F3 de la P3 de la
  15-6b : helper commun dans `invoice_settlements`. Décisions de l'orchestrateur.
- **Retenu** : `accept_one_rule` gagne, juste après l'étape 1, l'étape « compte de banque actif »
  d'`accept_one_split` (`reconciliation.rs:1912-1938` ; `DATABASE_ERROR`,
  `BANK_ACCOUNT_NOT_CONFIGURED` + `details.bankAccountId`), avant la garde d'égalité — même réponse
  sur les quatre chemins (actif → `VALIDATION_ERROR`, non imputable → `VALIDATION_ERROR`, archivé →
  `BANK_ACCOUNT_NOT_CONFIGURED`) ; changement délibéré d'un refus existant, relevé en T0, testé
  (test 7). Deux témoins figent les gardes ventilées (`post_split`, `accept_one_split`). Écran :
  `listBankAccounts()` une fois au montage, sans garde de génération (le `{#key}` de la page remonte
  le composant ; son retrait exigerait la garde, dit au doc-comment). Backend : la **comparaison**
  emprunte `ensure_not_claim_account` (fonction pure de la 15-6b, C-15-6-26) ; la **construction** du
  refus reste celle du flux ventilé, ramenée à deux fonctions locales. La 15-6d passe donc après la
  15-6b. L'angle mort « course résiduelle » est retiré : la valeur comparée est celle qui construit
  l'écriture, aucun verrou à ajouter.
- **Écartées** : écrire la divergence du chemin par règle comme angle mort (un même défaut rendrait
  trois réponses) ; garder la garde de génération « par cohérence avec `loadGen` » (justification
  fausse, test d'un cas impossible) ; emprunter aussi la construction de refus de la 15-6b (changerait
  la forme `VALIDATION_ERROR` fixée par C-15-6-9 / C-15-6-23) ; `withoutAccountIds` à l'écran (écarté
  par C-15-6-23).
- **Réversible** : oui, tant qu'elle n'est pas développée.

## C-15-6-29 — 15-6a : le verrou partagé couvre tous les comptes que l'avoir écrit ; rejeu de la route sur interblocage ; ligne absente → `Invariant` (rectifie C-15-6-24 ; étend C-15-6-25)

- **Contexte** : validation P4 de la 15-6a (lentilles R et F, Opus ; remédiation Opus 5.5). F-1
  (MEDIUM) : la TVA due et les comptes de produit demandaient leur S **après** l'exercice, face au X
  que le solde du reste y prend **avant** le sien — cycle préexistant que la 15-5d décrit et ferme pour
  la validation, non pour l'avoir. R4-1 ≈ F-2 (MEDIUM) : l'analyse de C-15-6-24 (« S + S ne bloque ni
  le rapprochement ni un second avoir ») était fausse pour l'arrondi — le rapprochement prend X sur
  l'arrondi et sur l'exercice avant son `UPDATE invoices`, et le cycle avoir ↔ rapprochement de la
  même facture préexiste par l'exercice (**#536**, ouverte par l'orchestrateur) ; la route de l'avoir
  ne rejoue pas un 1213 (500). F-3 (LOW) : cycle à trois neuf par un X en attente. R4-9 = F-5 (LOW) :
  ligne absente du résultat du verrou non spécifiée. Décisions de l'orchestrateur.
- **Retenu** : (1) le `LOCK IN SHARE MODE`, pris avant l'exercice, en **une** requête `ORDER BY id`,
  couvre la créance et l'arrondi de la vente, la TVA due des réglages (si l'avoir porte de la TVA et
  que le réglage est posé) et les comptes de produit effectifs (repli D-B2 compris) — règle écrite :
  un compte qu'un autre flux tient en X avant l'exercice se verrouille avant l'exercice ; (2) la 6 ter
  lit `active` dans le résultat du verrou (l'instantané REPEATABLE READ, établi au snapshot des lignes,
  précède le verrou), en gardant son code, son message par ligne et sa place ; (3) un compte de TVA
  due archivé rend `CreditNoteAccountsArchived` (`ACCOUNT_ARCHIVED` nommé) au lieu de
  `INACTIVE_OR_INVALID_ACCOUNTS` anonyme — même remède que la créance et l'arrondi ; (4) un id que le
  verrou ne rend pas → `DbError::Invariant` ; (5) la route `POST /api/v1/credit-notes` enveloppe le
  dépôt dans `retry_with` (`is_deadlock_error`), patron de `write_off_invoice_handler`, figé par un
  test qui provoque un interblocage déterministe dont l'avoir est la victime ; (6) trois cycles
  résiduels inscrits à l'inventaire (avoir ↔ rapprochement, #536 ; avoir ↔ solde du reste par ordres
  opposés entre deux comptes ; X en attente), tous détectés par InnoDB et rejoués. L'angle mort « un
  compte lu sans verrou pendant l'avoir » disparaît. C-15-6-24 reste vrai sur le mode (S) et la place
  (avant l'exercice) ; sa justification « S + S ne bloque pas le rapprochement » est rectifiée ici.
- **Écartées** : garder le verrou à la créance et à l'arrondi et inscrire le cycle de la TVA due et
  des produits en angle mort (la 15-5d a fermé la même forme pour la validation ; l'avoir serait le
  dernier chemin à l'ouvrir) ; un refus `ACCOUNT_ARCHIVED` pour les produits aussi (changerait le
  refus par ligne de la 6 ter, figé par ses tests et son message) ; laisser la ligne absente au filet
  anonyme de `create_in_tx` ; laisser la route sans rejeu (un 500 opaque sur « Créer un avoir » quand
  l'avoir est la victime).
- **Réversible** : oui, tant qu'elle n'est pas développée.

## C-15-6-30 — 15-6b : le compte d'arrondi porte le rôle `rounding` quel que soit le geste ; validation et avoir en angles morts écrits ; message de liste vide conditionné

- **Contexte** : validation P4 de la 15-6b (lentilles R et F, Opus ; remédiation Opus 5.5). R1 = F1
  (MEDIUM) : le reste d'arrondi du solde (5 bis) et la nature `rounding` étaient classés
  `write_off_nature`, alors qu'ils lisent `default_rounding_account_id`, la colonne du compte d'arrondi
  du règlement — le message renvoyait au champ d'escompte (recyclage du discriminant posé en P3,
  C-15-6-26). F2 (MEDIUM) : « résolu par le type » faux pour la validation (créance des réglages sans
  type contrôlé) et pour l'avoir (TVA due, produit de repli). F3 (MEDIUM) : le message de liste vide
  accusait le compte débiteurs dans une société sans compte bancaire. F4 (LOW) : « désignez » s'adresse
  à un Comptable sans accès aux réglages. Décisions de l'orchestrateur.
- **Retenu** : `role: rounding` pour tout compte lu dans `default_rounding_account_id` (règlement,
  rapprochement, reste d'arrondi et nature `rounding` du solde) ; `write_off_nature` pour `discount`,
  `bank_fees`, `bad_debt` seulement ; le rôle suit la colonne lue, pas la route ; ordre des refus au
  solde : nature, reste d'arrondi, TVA due. Angles morts écrits, sans garde neuve : créance retypée à
  la validation (**#537**, ouverte par l'orchestrateur), TVA due et produit de repli de l'avoir
  (#525). Message de liste vide seulement si au moins un compte bancaire a été écarté par le filtre.
  Messages : « un administrateur doit désigner », comme C36. Sélecteur Fluent multi-lignes, repli Rust
  par rôle. **Signal D5 levé (recyclage de F1), pas de découpage** : recyclage contenu au même
  discriminant (une ligne de classement, un test), F2/F3 d'origine, aucun module de plus ; un troisième
  recyclage sur l'AC3 bis en P5 en déciderait la sortie en story propre.
- **Écartées** : un rôle distinct « reste d'arrondi » (deux valeurs pour une même colonne, un
  intégrateur verrait deux rôles pour un même compte désigné) ; garder la validation et l'avoir
  (sixième et septième modules, hors de la classe « règlement ») ; un second message pour « aucun
  compte bancaire » (aucun état vide n'existe aujourd'hui, le comportement actuel suffit) ;
  « désignez » avec une branche d'écran par rôle (écartée par C36).
- **Réversible** : oui, tant qu'elle n'est pas développée.

## C-15-6-31 — 15-6d : clôture de la validation ; C-15-6-28 révise C-15-6-23 (garde de génération, course résiduelle) ; texte du refus selon le champ

- **Contexte** : validation P3 de la 15-6d (lentilles R et F, Sonnet ; remédiation Opus 5.5) — 10 LOW,
  0 au-dessus. F3-1 : C-15-6-23 « Retenu » dit toujours « effet dépendant de `bankAccountId`, garde de
  génération » et « course résiduelle de `post_manual` écrite », que C-15-6-28 retire en se disant
  « complète » ; le registre ne se réécrit pas. F3-2 : « le même refus que `post_split` » n'est pas le
  même texte (`splits[{idx}].…`). R1 : l'étape ajoutée à `accept_one_rule` est atteignable depuis
  l'écran.
- **Retenu** : C-15-6-28 **révise** C-15-6-23 sur deux points — plus de garde de génération à l'écran,
  plus d'angle mort « course résiduelle » — ; le reste de C-15-6-23 tient. Une fonction de refus
  `counterparty_is_bank_ledger_error(field)` pour `post_split` et `post_manual`, même code, texte selon
  le champ ; tests sur le code seul. Compte de banque archivé avant l'ouverture de l'écran : angle mort
  écrit, aucun contrôle ajouté à `get_proposals`. Manuel : le refus seul est affirmé sans réserve.
  CHANGELOG : `### Modifié` pour le changement délibéré de l'AC2. Validation close.
- **Écartées** : réécrire C-15-6-23 (interdit par la consigne du registre) ; un message identique sur
  les deux routes (`post_manual` n'a pas d'index de ligne) ; contrôler l'activité du compte lié dans
  `get_proposals` (hors périmètre, la garde d'acceptation suffit).
- **Réversible** : oui, tant qu'elle n'est pas développée.

## C-15-6-32 — 15-6a : alignement sur la 15-5e réécrite par C54 — le rejeu de la route vient de la 15-5e, le verrou reste pour la seule course de lecture, aucun réordonnancement, cycles nommés sans prétention d'absence ; helper de verrou de liste partagé avec la 15-5d ; « émet de la TVA » à source unique (révise C-15-6-29 sur le rejeu et les cycles)

- **Contexte** : validation P5 de la 15-6a (lentilles R et F, Sonnet ; remédiation Opus 5.5). R5-1 =
  F5-1 (MEDIUM) : la fiche ignorait la 15-5e (C52). R5-2 (MEDIUM) et F5-3 (LOW) : cycles non
  inscrits (avoir ↔ solde du reste par TVA due ↔ arrondi ; avoir ↔ règlement par compte interne).
  F5-5 (LOW) : deux helpers jumeaux (15-5d en X, 15-6a en S). R5-4 = F5-2 (LOW) : condition « l'avoir
  émet de la TVA » calculée après le verrou. Consigne initiale de l'orchestrateur : « une seule règle
  d'ordre des verrous pour l'epic, celle de la 15-5e », avec « l'arrondi d'abord ». **Correction de
  consigne reçue pendant la remédiation** : la 15-5e est réécrite selon **C54** (la défense contre
  l'interblocage est le rejeu ; la 15-5e rejoue toutes les routes d'écriture, avoir compris, et ne pose
  plus « l'arrondi d'abord », C55) ; ne pas réordonner l'avoir pour fermer des cycles, garder les
  verrous qui ferment une course de lecture, retirer toute affirmation d'absence de cycle.
- **Retenu** : (1) dépendances « après 15-5a à 15-5e » ; (2) l'AC7 ne réécrit ni le doc-comment
  canonique, ni le « 5 bis », ni le doc-comment de la route, que la 15-5e réécrit ou pose ; elle ne
  corrige qu'une phrase qui décrirait l'ancien ordre de l'avoir, s'il y en a une ; (3) la route
  `POST /api/v1/credit-notes` est rejouée **par la 15-5e** — la 15-6a ne spécifie plus de `retry_with`
  (révise C-15-6-29) ; son test 15 fige que ce rejeu couvre l'attente neuve créée par le verrou ;
  (4) le verrou `LOCK IN SHARE MODE` des comptes écrits reste **en une instruction `ORDER BY id`**,
  avant l'exercice, pour la seule **course de lecture** (archivage concurrent, `active` lu frais) ;
  aucun réordonnancement ; (5) les cycles connus — #536 (rejeu, C57), solde du reste (nature
  commune, ou TVA due ↔ arrondi), règlement par compte interne, X en attente — sont **nommés, non
  exhaustifs**, couverts par le rejeu ; plus d'« ensemble clos » ; (6) **un seul** helper de verrou
  d'une liste de comptes, paramétré par le mode, créé par la première story mergée (a priori la
  15-5d), réutilisé par la 15-6a ; `EXPLAIN` et test d'identifiant étranger comme la 15-5d (C51) ;
  (7) agrégation de la TVA de l'avoir extraite en fonction pure partagée par le générateur et le
  calcul des ids ; refus `ConfigurationRequired(TVA)` laissé à sa place ; (8) une ligne pour l'avoir
  au Pattern 5, dans la forme que la 15-5e lui donne.
- **Écartées** : « l'arrondi de la vente d'abord » (demandé par la consigne initiale, retiré par la
  correction : un réordonnancement pour fermer un cycle, contraire à C54/C55) ; un « ensemble clos »
  de cycles (affirmation d'absence, contraire à C54) ; garder un `retry_with` propre à la route (doublon
  de la 15-5e) ; un second helper en S (DRY) ; avancer le générateur avant le verrou (déplacerait le
  refus de TVA devant `FISCAL_YEAR_INVALID`) ; dupliquer `total_vat > 0`.
- **Réversible** : oui, tant qu'elle n'est pas développée. **À répercuter hors du worktree** (par
  l'orchestrateur) : la fiche 15-5d peut dire que son helper prend un mode.

## C-15-6-33 — 15-6b : alignement sur la 15-5e réécrite par C54 (aucun ordre ne change ; chaque comparaison suit sa lecture) ; trois clés plates ; critère de T2 par nom sur liste fermée ; sujet de refus typé (révise C-15-6-26 et C-15-6-30 sur la forme des messages)

- **Contexte** : validation P5 de la 15-6b (lentilles R et F, Sonnet ; remédiation Opus 5.5). F1
  (MEDIUM) : la 15-5e (C52) réordonnait `settle_invoice` et `write_off_invoice` ; sa réécriture par
  C54, reçue en cours de remédiation, retire ce réordonnancement et rejoue les routes. R5-1 (MEDIUM) :
  le sélecteur Fluent `$role` rougit un test de `kesh-i18n` et se résout mal côté frontend. R5-2
  (MEDIUM) : la borne `sitesTotal` existe déjà. R5-3 = F2 (MEDIUM) : le critère de T2 (P4) ne voyait
  pas un littéral coupé par rustfmt. F3 (MEDIUM) : `user-manual.tex:1754`. F5, F6, F7, R5-4 à R5-8 (LOW).
- **Retenu** : (1) dépendances « après 15-5a à 15-5e », étapes citées par leur nom ; (2) **chaque
  comparaison suit aussitôt la lecture de son compte**, dans l'ordre des lectures du code, que la
  15-5e ne change pas : au solde, nature → reste d'arrondi → TVA due ; au règlement, contrepartie →
  trop-perçu → compte d'arrondi ; une nature égale à la créance précède les refus de configuration
  des comptes lus après elle (test 10 ter) ; (3) trois clés plates
  (`error-settlement-{rounding,write-off,vat-payable}-account-is-receivable`), un rôle une clé, sans
  `match` de repli ; (4) `sitesTotal` relevé pour tous les sites neufs (+4, +6 avec la 15-5c), recompté
  sur l'état rebasé ; (5) critère de T2 : `grep -rnF` du nom, commentaires exclus, sur une liste fermée
  de cinq fichiers, chaque occurrence lue ; (6) helper de construction à sujet typé
  (`ClaimSubject::{Counterparty(ClaimSide), Designated(DesignatedRole)}`) ; (7) `:1754` au manuel ;
  phrase « Refus » du paiement fournisseur avant la ligne d'annulation ; (8) produit de repli de l'avoir
  écrit « non tracé » (#525 ne le couvre pas), signalé à l'orchestrateur. **Signal D5 non déclenché** :
  F1 vient d'un changement extérieur (la 15-5e), les recyclages (R5-1, F2) sont hors de l'AC3 bis.
- **Écartées** : suivre l'ordre « l'arrondi d'abord » de la première version de la 15-5e (retiré par
  C54/C55) ; inscrire la clé à sélecteur à `SELECTEURS_RESOLUS_COTE_SERVEUR` (ajoute `loader.rs` et un
  repli Rust par `match` ; la 15-5c a choisi les clés plates pour la même raison) ; toutes les
  comparaisons après la dernière lecture (ordre des refus dépendant de refus de configuration
  étrangers au défaut) ; un critère `rg -U` multi-lignes (plus fragile qu'une liste fermée) ; un
  `debug_assert!` pour `Payable` + rôle désigné (muet en production).
- **Réversible** : oui, tant qu'elle n'est pas développée.

## C-15-6-34 — 15-6b : clôture de la validation à la P6 ; produit de repli et validation d'achat tracés (#525, #537) ; règle de placement « après son compte et la créance » ; manuel d'administration ; dispersion déclarée sans découpage

- **Contexte** : validation P6 de la 15-6b (Opus, lentilles R et F) : 0 au-dessus de LOW (R 11 LOW,
  F 7 LOW, R6-4 = F6-6). Après la P5, l'orchestrateur a tracé le produit de repli de l'avoir sur #525
  et la forme côté dette de la validation d'achat sur #537 (commentaires du 2026-10-08) ; la fiche les
  disait « non tracé » / « signalé ». F6-1 : le manuel d'administration n'avait aucun verdict. F6-7 :
  le critère de dispersion de l'amendement D5 est rempli au barème de la règle (neuf modules).
- **Retenu** : (1) **validation close** (0 > LOW, § *Review Iteration Rule*) ; tous les LOW appliqués ;
  (2) « tracé par #525 / #537 » aux trois sites de la fiche et dans la 15-6a ; (3) règle de placement :
  chaque comparaison suit la lecture de son compte **et** celle de la créance (la nature est lue avant
  la créance au solde) — aucun réordonnancement (C55) ; (4) manuel d'administration : `:2027` inchangée,
  une phrase à `:2029`, une condition à `:2033`, frontière de la 15-6c alignée ; (5) présélection du
  dialogue de règlement dans un effet distinct qui n'efface pas la saisie ; (6) clés de lot de
  `details` ajoutées par le mapping HTTP seul ; (7) `DesignatedRole` avec `ClaimSubject` dans
  `invoice_settlements.rs` ; (8) **dispersion déclarée au Project Lead, non découpée** : le « barème du
  geste » n'est plus invoqué ; raison — fiche convergée en six passes, gestes liés par une variante, un
  sujet typé, un lecteur sœur et trois fonctions d'écran ; le risque visé (non-convergence) ne s'est
  pas réalisé ; arbitrage de Guy en revue de fin d'epic.
- **Écartées** : une P7 (aucun finding > LOW ; la boucle s'arrête) ; un angle mort écrit pour le
  manuel d'administration au lieu de deux phrases (le remède du message envoie l'administrateur à ce
  manuel) ; découper la story à ce stade (rouvrirait deux validations pour séparer ce que la
  conception lie) ; reconstruire le `default` de `t_args` par un `match` (le `format!` du patron
  `errors.rs:88-92` suffit).
- **Réversible** : oui, tant qu'elle n'est pas développée ; le découpage reste ouvert à l'arbitrage
  de Guy.

## C-15-6-35 — 15-6a : un seul contrat pour le helper de verrou de liste (mode, `pub`, ne refuse rien, type nommé distinct de `LockedAccount`) ; critère DRY à exception écrite ; partenaires réels du cycle (iv) ; dépendance ferme à la 15-5d

- **Contexte** : validation P6 de la 15-6a (Opus, lentilles R et F) : 4 MEDIUM distincts — F6-1 = R6-1
  (deux contrats pour la fonction de verrou), R6-2 (critère DRY faux : `opening_complement.rs:518-537`),
  R6-3 (doctrine retirée restée aux Dev Notes et en T0), F6-2 = R6-4 (validation impossible comme
  partenaire du cycle (iv)) ; trois recyclent la remédiation P5.
- **Retenu** : (1) la puce « Un seul helper » est le **seul** contrat : paramétré par le mode, à
  l'emplacement de la 15-5d, `id, number, active, postable`, ne refuse rien (« ligne absente » et
  partage chez `create_credit_note`), **`pub`** (test 17 d'intégration), type de ligne nommé et public
  **distinct** de `opening_complement::LockedAccount` (p. ex. `LockedAccountState`), ou celui de la
  15-5d s'il existe ; le paragraphe « Refus nommé » y renvoie ; (2) critère DRY : aucune autre requête
  de verrou d'une liste d'ids de comptes **hors** `opening_complement.rs:518-537`, exception écrite
  (forme distincte, hors périmètre) ; (3) Dev Notes et T0 : plus de « règle de l'AC6 » ni de
  conformité à vérifier — le verrou avant l'exercice sert la course de lecture, la défense est le
  rejeu (C54) ; (4) cycle (iv) : partenaires = flux qui prennent l'exercice sans la ligne des réglages
  (saisie manuelle, règlement, solde du reste, rapprochement) ; validation et saisie fournisseur
  exclues ; (5) la 15-5d devient une dépendance **ferme** (la story attend) ; (6) test 17 : issue
  attendue rouge d'après la mesure du dépôt, puis `owned_account_ids`, sauf adoption par la 15-5d ;
  (7) **signal D5 levé** (MEDIUM → MEDIUM, recyclage de la P5) et dispersion (six modules métier, plus
  la plomberie) **déclarés**, pas de découpage : recyclage contenu au texte de l'AC6, sans règle métier
  ni module ; P7 complète (Sonnet) due.
- **Écartées** : réutiliser `opening_complement::LockedAccount` (privé, autres colonnes, flux hors
  epic) ou lui prendre son nom (homonymie) ; refondre `create_opening_complement` sur le helper
  (changement sans motif d'un flux livré) ; restreindre le critère DRY aux seuls flux de facturation
  (moins décidable qu'une exception nommée) ; garder la branche « si la 15-5d n'est pas mergée »
  (deux chemins à spécifier pour un ordre d'epic déjà fixé) ; découper l'AC6 (le défaut est une
  propagation incomplète, que le grep du symptôme traite).
- **Réversible** : oui, tant qu'elle n'est pas développée. **À faire par l'orchestrateur** :
  commentaire sur #536 (l'avoir est couvert par le rejeu de la 15-5e ; la 15-6a ne ferme aucun cycle —
  finding F6-6).

## C-15-6-36 — 15-6a : un seul type de ligne pour le helper de verrou (révise C-15-6-35 (1)) ; C-15-6-32 (6) et C-15-6-29 marqués révisés ; manuel : le rôle repris avant la réactivation

- **Contexte** : validation P7 de la 15-6a (Sonnet, lentilles R et F) : 1 MEDIUM (R7-1), 10 LOW. R7-1 : la fiche
  (AC6) disait « la 15-6a prend le sien » si la 15-5d a nommé le type, le registre (C-15-6-35 (1)) « ou celui de
  la 15-5d s'il existe » ; deux consignes qui s'excluent, la première produisant deux types publics de même forme.
- **Retenu** : (1) **un seul type, jamais deux** : si la 15-5d a nommé **et exposé (`pub`)** le type de ligne de
  son helper, la 15-6a l'emploie ; sinon elle le rend public sous le nom qu'il porte (ou `LockedAccountState`
  s'il n'en a pas). **Cette entrée révise C-15-6-35 (1)** (« type nommé et public distinct de `LockedAccount`,
  ou celui de la 15-5d s'il existe »), la fiche (AC6) étant alignée ; (2) **C-15-6-32 (6)** (« créé par la
  première story mergée, a priori la 15-5d ») est **révisé** : la 15-5d est une dépendance ferme (C-15-6-35 (5)) ;
  **C-15-6-29** (« angle mort : cycle avec chaque validation dès la 15-5d ») est **révisé** : la validation
  n'est pas partenaire du cycle (iv) (C-15-6-35 (4)) ; (3) le cycle (iv) ne cite `invoices.rs:2006` et `:2172`
  que pour la validation ; l'ordre de la saisie fournisseur est relevé par T0 ; (4) test 17 : appel direct du
  helper avec des ids explicites, sans `UPDATE` des réglages ; tests 2 et 7 : « par SQL direct » ; (5) le
  `keshwarning` du manuel dit de retirer d'abord le rôle *Créances clients* du nouveau compte si nécessaire avant
  de réactiver (`user-manual.tex:358-359`) ; (6) numéros de ligne relevés de `credit_notes.rs` et
  `opening_complement.rs` corrigés (T0 les refait de toute façon).
- **Écartées** : deux types (un par story) ; remplacer le type de la 15-5d par celui de la 15-6a (change le retour
  d'un helper d'une autre story).
- **Réversible** : oui, tant qu'elle n'est pas développée. **Pour le message de PR** (F7-6) : `closes #523`
  renvoie au commentaire du 2026-10-08 sur #523 — la contrainte « sans casser le refus nommé de #486 » est levée
  en esprit, pas à la lettre (l'assertion de `invoices_validate_vat.rs:846` change).
