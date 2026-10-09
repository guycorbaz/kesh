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

## C-15-6a-1 — 15-6a : T0 fait, développement suspendu jusqu'au merge de la 15-5d ; helper employé tel quel (partagé, sans mode), rendu `pub` sans second type

- **Contexte** : T0 de la 15-6a sur `origin/main` `9cb5083b`. La 15-5d (helper `lock_designated_accounts_in_tx`,
  test de C35) est développée et revue (branche `story/15-5d-garde-usage-comptes-reglage`, `5624ca78`) mais **non
  mergée**. La fiche en fait une dépendance **ferme** (« si l'une n'est pas mergée au moment de T0, la story
  attend », R6-10 ; C-15-6-35 a écarté la branche « si la 15-5d n'est pas mergée »). Sur la branche de la 15-5d,
  le helper est déjà en **partagé seul** (C87), déjà sur le patron `owned_account_ids` (C88), `FORCE INDEX
  (PRIMARY)`, `pub(in crate::repositories)`, type de ligne privé `LockedDesignatedAccount` dans le newtype
  `DesignatedAccountsSnapshot` ; son test de C35 est dans `invoices_validate_vat.rs`.
- **Retenu** : (1) **arrêt après le T0**, relevés et écarts consignés (Change Log de la fiche, « Alignement sur le
  livré (T0) ») ; (2) une fois la 15-5d mergée, la 15-6a **emploie le helper tel quel, sans paramètre de mode**
  (aucun appelant exclusif ne reste ; un mode à une seule valeur serait du code mort) et rend `pub` la fonction,
  `LockedDesignatedAccount` et l'accès aux lignes de `DesignatedAccountsSnapshot` sous leurs noms (C-15-6-36 :
  un seul type) ; (3) le test 13 se ré-ancre dans `invoices_validate_vat.rs`, le test 17 est attendu vert
  d'emblée ; (4) la ligne Pattern 5 de l'avoir prend la forme « par renvoi » (C68).
- **Écartées** : fusionner la branche de la 15-5d dans celle de la 15-6a (les consignes interdisent tout merge ;
  et la résolution du conflit 15-5d ↔ 15-5e2 sur `invoice_settlements_write.rs` reviendrait à qui merge la 15-5d) ;
  écrire un helper jumeau (contraire au critère DRY de l'AC6, conflit certain au merge) ; développer T1/T2 seuls
  (story livrée en deux temps, sans motif — T1/T2 ne prennent qu'une fraction du travail).
- **Réversible** : oui. **À faire par l'orchestrateur** : merger la 15-5d, puis relancer le développement de la
  15-6a sur `main` à jour (le T0 n'aura à refaire que les numéros de ligne des fichiers que la 15-5d touche :
  `company_invoice_settings.rs`, `invoices.rs`, `invoice_settlements_write.rs`, `invoices_validate_vat.rs`, manuels).

## C-15-6a-2 — 15-6a (dev) : rebase sur `5e4bec50` par union ; helper de la 15-5d rendu `pub` avec un accesseur, sans mode

- **Contexte** : la 15-5d est mergée (`5e4bec50`) ; la branche de la 15-6a portait la planification (fiches 15-6*,
  registre) et le T0. Rebase : conflit du registre (entrées disjointes — 15-11a/15-5d côté `main`, C-15-6-1 à 36
  côté branche) et de l'en-tête de `sprint-status.yaml` (deux lignes `(19)`). Au code mergé, le helper
  `lock_designated_accounts_in_tx` est partagé seul, privé au module `repositories`, et rend un newtype à champ privé.
- **Retenu** : (1) union sans dédoublonnage nécessaire (aucun titre commun, vérifié par `uniq -d`) ; la ligne
  `last_updated` du T0 renumérotée `(24)` et placée en tête, la mention de sa numérotation d'origine gardée ;
  (2) le helper est employé **tel quel, sans paramètre de mode** (C-15-6a-1 confirmé) ; sont rendus `pub` la
  fonction, `LockedDesignatedAccount` (champs `pub`) et `DesignatedAccountsSnapshot`, auquel s'ajoute un
  accesseur en lecture `accounts()`. Le champ du newtype reste privé : on ne fabrique pas d'instantané hors du
  helper, et `check_written` reste `pub(in crate::repositories)`.
- **Écartées** : rendre le champ du newtype `pub` (permettrait de construire un faux instantané hors du verrou) ;
  un second type pour l'avoir (C-15-6-36) ; ajouter un mode à une seule valeur.
- **Réversible** : oui (visibilité seule).

## C-15-6a-3 — 15-6a (dev) : placement des tests, bras HTTP factorisé, passages du manuel laissés tels quels

- **Contexte** : développement de la 15-6a (T1–T5). Plusieurs points laissés « au choix » ou non tranchés par la fiche.
- **Retenu** : (1) **test 15** dans `crates/kesh-api/tests/rejeu_interblocage_e2e.rs` (la fiche laissait le choix avec
  `invoice_echeancier_e2e.rs`) : le fichier porte déjà le harnais du patron (`transaction_lourde`, `victime`,
  `CaptureRejeu`), ce qui évite une troisième copie et ajoute le témoin `exiger_un_rejeu("credit_notes::create")` ;
  « premier numéro » vérifié par la séquence (un seul numéro tiré), non par la forme du numéro. (2) **Tests 4 et 17** :
  seconde société par `INSERT INTO companies` direct (patron du test `foreign_account_is_never_locked_and_is_refused`
  de la 15-5d), non par `companies::create` — le lecteur et le helper n'en lisent que l'identifiant. (3) **Test 10** :
  montage léger prévu par la fiche (en-tête d'écriture sans ligne, même société), nom de la fiche gardé. (4) **Test 13**
  renommé `credit_note_credits_a_non_postable_sale_receivable` et doté d'une assertion sur le compte crédité (sans
  elle, il ne figeait rien de la 15-6a). (5) **Bras HTTP** : `ReversalAccountsArchived` et `CreditNoteAccountsArchived`
  partagent `archived_accounts_response(archived, clé, amorce du repli)` (DRY) — corps et statut inchangés pour la
  contre-passation. (6) **Manuel** : `user-manual.tex:928` (« L'avoir n'est pas soumis à ce contrôle … même si sa
  créance ou sa TVA due est devenue non imputable ») **reste vrai** — la créance de la vente non imputable est toujours
  créditée — et n'est pas réécrit ; seule `:380` (« relit la créance … dans les réglages ») l'est. La balance âgée
  (`:1846-1851`) et « un avoir le reprend et l'annule » restent vrais. (7) **CHANGELOG** : l'entrée de la 15-5d qui
  annonçait « ce que corrigeront #473 et #525 » est réécrite dans la même version non publiée (0.13.0), pour ne pas
  contredire l'entrée neuve.
- **Écartées** : un fichier de test neuf pour le rejeu de l'avoir ; une `companies::create` complète pour un identifiant ;
  dupliquer le bras HTTP ; réécrire `:928`.
- **Réversible** : oui.

## C-15-6a-4 — Revue de code P1 de la 15-6a : remédiation sans code de production, une dette écrite

- **Contexte** : la revue P1 (Sonnet ×3) rend 0 au-dessus de LOW et 14 LOW. Plusieurs se corrigent par des
  tests, des doc-comments ou de la documentation ; un seul (B-3, double dérivation de l'ensemble des comptes de
  produit dans `create_credit_note`) ne se corrige que par du code de production.
- **Option retenue** : corriger tout ce qui se corrige hors production exécutable (renommage du test 10, test 13
  discriminant constaté rouge sous mutation, test 19 sur l'`Invariant` « écriture de vente sans ligne de débit »,
  doc-comments, section « Émettre un avoir » de `docs/api-external.md`, manuel et PDF) ; écrire **B-3 comme dette**
  (P3, à ouvrir en issue par l'orchestrateur : dériver `sites` de `revenue_ids` par un seul helper, ou un
  `debug_assert!` d'inclusion) ; écrire comme **angle mort** le second `Invariant` de l'AC3 (« facture validée sans
  écriture de vente »), que `chk_invoices_validated_has_je` empêche de monter. Le test 19 est placé dans
  `credit_notes_repository.rs` (patron du test 3, helpers `emit` / `assert_nothing_written`).
- **Écartées** : poser le garde de B-3 maintenant (rouvrirait la boucle — la remédiation toucherait la production,
  ce qui interdit de clore après elle) ; un test du second `Invariant` par désactivation de la contrainte
  (`SET check_constraint_checks = 0`) — il monterait un état que la base interdit, sans valeur de preuve.
- **Réversibilité** : totale ; la dette se solde par une story de quelques lignes.
## C100 — 15-12, remédiation de la validation P1 : révise C89 (preuve, ordre des verrous, réparation, dérogation)
- **Contexte** : validation P1 de la 15-12 (lentilles R et F, Opus) — 1 HIGH, 4 MEDIUM, 14 LOW distincts,
  aucun recyclé. Trois points de C89 sont faux ou mal fondés : la preuve du filet (point 4) prêtait à la
  lecture verrouillante de la clôture un rôle dans le scénario « écrivain dans N », où elle n'en a
  aucun ; l'ordre des verrous (point 3) affirmait que toutes les acquisitions d'exercice deviennent
  ascendantes ; la réparation (point 5) prescrivait de rouvrir le plus proche postérieur clos, que la
  garde LIFO refuse dès qu'un plus récent est clos. L'argument de non-découpage (point 6) reposait sur
  la preuve réfutée.
- **Retenu** :
  1. **Preuve** : la sûreté du filet repose sur (α) l'invariant I sur tout état validé — c'est lui qui
     dépend de la lecture verrouillante de la clôture, face à la course réouverture/clôture — et (β) le
     verrou de son exercice tenu par l'écrivain. Test 13 a **ordonné** (réouverture de N simulée et non
     validée, `close(L)` lancée, bloquée, puis validation de la réouverture → refus
     `EarlierFiscalYearOpen(N)`), qui tue la mutation (ii) sur l'état final ; l'ancien 13 c (écrivain dans
     N) est retiré ; la course libre `reopen_close_concurrent_is_serialized` est remplacée.
  2. **Test « refus parasite »** (ex-13 d, désormais 13 c) : forme **ordonnée** plutôt que l'ensemble
     des issues admises — seule à garder contre le refus parasite, son objet.
  3. **Ordre des verrous** : la clôture est ascendante comme `reopen`, la garde 15-8a et
     `find_open_covering_date` pris seuls ; la contre-passation (`reverse_in_tx_inner`) et les quatre
     annulations qui la portent verrouillent l'exercice de l'origine puis parcourent depuis le premier
     exercice (ordre inverse) : cycle possible avec la clôture, résolu par le rejeu des deux côtés (cinq
     routes `Rejouee`, clôture enveloppée par l'AC 6) ; test 13 d à deux connexions, victime forcée si
     InnoDB le permet (mesure en T0), mutation (ix). Le cycle réouverture/contre-passation, préexistant,
     reste nommé au point (iv) du registre.
  4. **Réparation de l'état hérité** : d'abord clôturer l'exercice ouvert le plus ancien (l'AC 1
     l'accepte, geste du Comptable), puis les suivants dans l'ordre ; sinon, un administrateur rouvre
     les exercices clos **à partir du plus récent** (seul ordre que LIFO accepte). Bandeau à trois noms
     (`$open`, `$closed`, `$latest`) ; message global neutre, sans prescription de réouverture et
     valable à la création d'un exercice.
  5. **Restauration en vol** : angle mort assumé, écrit (le verrou de `full_import` ne sérialise qu'avec
     les autres imports) — et non plus « exclue ».
  6. **Clé du lot** : `reconciliation-failed-later-fiscal-year-closed` avec `$name` lu dans
     `details.fiscalYearName` (troisième code lu par `details`).
  7. **Base partagée** : les tests qui posent un postérieur clos le retirent ; mode d'échec écrit dans
     `docs/testing.md` ; pas de réparation dans `ensure_open_fiscal_year` (masquerait un résidu).
  8. **Pas de découpage**, sur un argument neuf : décompte refait, **11** modules ; la coupe A/B laisse
     7 et 8 modules (aucune moitié sous le seuil) et le remède de la règle (patron puis déploiement
     mécanique) n'a pas de prise ; les deux moitiés réécriraient les mêmes paragraphes de doc ; les
     défauts de P1 sont d'origine et se rangent par thème. **Déclencheur de repli élargi** : premier
     défaut recyclé, quel qu'en soit l'objet, ou non-convergence (D5) → 15-12a / 15-12b.
- **Écartées** : asserter l'ensemble des issues au 13 c (ne garde plus contre le refus parasite) ;
  forcer l'entrelacement du 13 a par un verrou d'intervalle sur `audit_log` (proposition F1 — plus
  fragile que la réouverture simulée, et dépendante de la place de l'audit dans `reopen`) ; garder « tout
  est ascendant » en rendant les annulations ascendantes (refonte de cinq flux rejoués pour un cycle
  que le rejeu résout déjà) ; prescrire la seule réouverture (geste réservé à l'administrateur, plus
  long, et faux dans l'ordre d'origine) ; découper maintenant en 15-12a/15-12b (cf. point 8).
- **Réversible** : oui (fiche seulement ; code non écrit). Le Project Lead peut imposer la coupe.

## C101 — 15-1a : la migration du lettrage relève `kesh_version_min_required` à 0.13.0 (révise C90), et ce que cela fait à la release
- **Contexte** : validation P1 de la 15-1a (Opus ×2), findings R-1 = F1 (HIGH). C90 avait conclu « pas de
  bump » sur la liste P3 et nommé un risque inexact (une v0.12.1 ne modifie ni ne supprime aucune écriture
  manuelle — `PUT`/`DELETE` y rendent `ENTRY_IS_POSTED`, vérifié sur le tag). Le risque réel : une v0.12.1
  relancée sur une base où la 15-1a2 a posé des groupes `document` annule un règlement, un paiement ou un
  rapprochement **sans dissoudre le groupe** — facture due, grand livre « soldée », faux rattachement
  muet. Précédent `20260814000001` : bump sur un changement de sémantique d'écriture, sans opération P3.
- **Retenu** (décision de l'orchestrateur) : dernière instruction `UPDATE _kesh_version SET
  kesh_version_min_required = '0.13.0' WHERE id = 1;` dans la migration de la **15-1a** (c'est elle qui
  introduit l'état) ; P2-bis : les dix crates à `0.13.0` dans le même commit ; P7 : `EXEMPT_MIGRATIONS`,
  `Durable`, patron `post_restore.rs:512-525` ; gate runtime complet ; `migrations_fresh_install.rs:244`
  passe de `0.10.0` à `0.13.0` ; CHANGELOG : avertissement de non-retour (précédent v0.10.0).
- **Conséquence pour la release** (lue dans `scripts/prepare-release.sh`, non exécuté) : le script refuse
  quand la version des crates égale la cible (« version cible identique », `exit 1`) **avant** son
  pré-vol. `prepare-release.sh 0.13.0` refusera donc, et **ne lancera ni** la datation du CHANGELOG **ni**
  le contrôle des exemptions périssables. La v0.13.0 se publiera comme la v0.10.0 (`a80a36c1`) :
  CHANGELOG daté à la main **et** contrôle périssable lancé à la main (`cargo run -q -p kesh-db --example
  perishable_exemptions`, puis la recherche de tag dans l'intervalle). ⚠️ À trancher hors story (chore) :
  apprendre au script le cas « déjà bumpé » (sauter l'étape 1, garder le pré-vol), ou écrire la
  procédure manuelle dans la checklist de release.
- **Écartées** : maintenir « pas de bump » (contraire au précédent et à la définition P1) ; bumper dans la
  15-1a2 seulement (la 15-1a aurait livré des colonnes qu'un ancien binaire ignore, et la 15-1a2 écrit déjà
  des données, P7 classe A) ; risque accepté sans bump (faux rattachement muet).
- **Réversible** : oui tant que non développé ; une fois publiée, une migration ne se modifie plus (P8).

## C102 — 15-1a : la garde `ENTRY_LETTERED` hors du drapeau `enforce_ownership`, et son motif d'écran (révise C97)
- **Contexte** : findings R-3 = F2 (la garde posée dans `modification_guard` n'est pas évaluée par
  `invoices::unvalidate`, qui appelle `delete_in_tx(…, false)`) et R-2 = F3 (HIGH : `modification_guard`
  sert aussi `modification_blocker`, le `GET` de la fiche, hors transaction — le motif sortirait vide à
  l'écran et un `FOR UPDATE` y attendrait derrière tout lettrage).
- **Retenu** : variante `ModificationGuard::Lettered { code }` (`code()` = `ENTRY_LETTERED`, `label()` = le
  code), rendue par une fonction **distincte** `lettering_guard(conn, company_id, id, Lecture)` ; appel
  **inconditionnel** dans `delete_in_tx` (étape 3-ter-bis) et `update_in_tx`, `Lecture::Conseil` (sans
  verrou) dans `modification_blocker`. Frontend dès la 15-1a : union `ModificationBlocker`,
  `modificationBlockerLabel`, `editRefusalOutcome` → `'stale'`, `ATTENDU` 11 → 12, clé
  `journal-entries-modify-blocked-lettered` quatre locales, « onze » → « douze ». Test
  `delete_in_tx(…, false)` sur écriture lettrée → refus.
- **Écartées** : le motif dans `modification_guard` (manque la dévalidation) ; une garde dans
  `unvalidate` seul (un point de passage de plus à tenir) ; exclure le motif de l'écran (bouton offert,
  `PUT` refusé) ; deux fonctions de lecture (elles divergeraient) — un paramètre.
- **Réversible** : oui (fiche seulement).

## C103 — 15-1a : séquence des verrous du lettrage — premier acte verrouillant, exercices exclusifs dans le sens de la clôture, exercice tenu en mode système (révise R7 de la reprise)
- **Contexte** : findings R-5 = F4 (découvrir les en-têtes par une lecture ordinaire ouvre la vue avant
  le verrou ; un nombre de lignes affectées inattendu finissait en `Invariant` → 500), R-6 = F7 (verrou
  partagé suivi d'un exclusif : interblocage d'escalade structurel ; `ORDER BY id` contraire à la clôture
  de la 15-12, `start_date`), R-4 (R7 et AC5 en désaccord sur le mode système).
- **Retenu** : (1) premier acte = une lecture `FOR UPDATE` **jointe** lignes ⋈ en-têtes, scopée par
  société, qui découvre et verrouille d'un coup — aucune lecture ordinaire avant ; plan vérifié par
  `EXPLAIN` ; (2) mode `Manual` : exercices des lignes `FOR UPDATE ORDER BY start_date, id` ; (3) mode
  `System { held_open_fiscal_year_id }` : **aucun** verrou d'exercice dans la primitive — l'appelant tient
  déjà un exercice ouvert couvrant une ligne et le passe ; vérification sans requête, manquement →
  `Invariant` (défaut d'appelant) ; (4) compte d'`UPDATE` inattendu → 409 `LETTERING_CONCURRENT_CHANGE`.
  ⚠️ **Affinement de la décision 7 de l'orchestrateur** (« exercices `FOR UPDATE ORDER BY start_date` ») :
  appliquée telle quelle au mode système, la contre-passation — qui tient l'exercice **du jour** quand elle
  lettre — reprendrait ensuite les exercices **antérieurs** de l'origine, à rebours de la clôture de la
  15-12, qui n'est pas rejouée. Interblocage résiduel avec `update_in_tx`/`delete_in_tx` (ligne avant
  en-tête dans l'acte 1) : nommé, absorbé par le rejeu (les trois routes sont `Rejouee`).
- **Écartées** : lecture ordinaire de découverte puis comparaison (contraire à la doctrine « le verrou
  d'abord ») ; lignes `FOR UPDATE` puis en-têtes en deux actes (même cycle, une requête de plus) ;
  verrouiller les exercices en mode système (cycle avec la clôture) ; `Invariant` sur le compte d'`UPDATE`.
- **Réversible** : oui (fiche seulement).

## C104 — 15-1a : la lettrabilité est exigée à la création d'un groupe, jamais à sa dissolution (révise C96)
- **Contexte** : finding F8 (+ R-21) — un compte lettrable peut cesser de l'être : retypage d'un compte
  mouvementé (`confirm_retype`, Story 25-2-a) ou rattachement d'un `bank_accounts`. La fiche ne disait pas
  ce que deviennent les groupes existants (mémoire « invariant dans le temps »).
- **Retenu** : les groupes existants restent **intacts** (le solde constaté reste vrai) et
  **dissolubles** ; la dissolution ne contrôle pas la lettrabilité ; l'invariant `lettering_invariants`
  ne la contrôle pas, par décision ; test : retyper 1100 en `Expense` puis `DELETE` → 204, idem après
  rattachement bancaire. Ce que la 15-1b affiche pour un tel compte est sa question.
- **Écartées** : dissoudre d'office au retypage (écriture cachée dans un geste de plan comptable) ;
  refuser le retypage d'un compte lettré (bloque une correction légitime) ; groupe indissoluble.
- **Réversible** : oui.

## C105 — 15-1a : lettrer comme délettrer exige au moins une ligne sur un exercice ouvert ; la 15-12 passe avant (révise C94 et D3)
- **Contexte** : findings F6 (lettrer deux lignes d'exercices clos réécrit les postes ouverts « au » d'une
  date close — la vue ne connaît pas la date du lettrage — exactement ce que C94 refuse au délettrage) et
  R-7 = F5 (C94 repose sur « les clos forment un préfixe », C89, Story 15-12, non déclarée en dépendance ;
  l'état « N ouvert, N+1 clos » est atteignable aujourd'hui).
- **Retenu** : règle **symétrique** en mode `Manual` — un groupe entièrement dans des exercices clos ne se
  crée ni ne se défait (409 `LETTERING_FISCAL_YEARS_CLOSED`, une clé) ; à cheval, permis. En mode
  système, garanti par l'exercice tenu (C103). **La 15-12 est un prérequis** : ordre **15-12 → 15-1a →
  15-1a2 → 15-1b → 15-1c**, écrit dans la fiche, l'index `15-1-lettrage.md`, `epics.md` et
  `sprint-status.yaml`. FR86 (« tant que l'exercice est ouvert ») est **interprété** pour un groupe à
  cheval, consigné aux Dev Notes. ⚠️ **D3 (« lettrer toujours permis ») était un arbitrage de Guy
  (août)** : cette révision, prise en autonomie, est à lui présenter à la revue finale. Point laissé à la
  15-1a2 : sa migration de rattrapage poserait-elle des groupes historiques entièrement clos ?
  (recommandation du socle : non, par symétrie).
- **Écartées** : asymétrie acceptée (lettrer libre) — la réécriture du passé est la même dans les deux
  sens ; dater le lettrage (une colonne de plus, et une vue « au » plus lourde) ; durcir AC5 sans la 15-12
  (« refus si une ligne est antérieure à un exercice clos ») — duplique la 15-12 au lieu d'en dépendre.
- **Réversible** : oui (fiche seulement) ; revenir à D3 coûte le retrait d'une branche de refus.

## C106 — 15-1a : un groupe `reversal` contenant une ligne de pièce n'est pas dissoluble à la main ; la promesse « paiement à lettrer » quitte la documentation (révise C97 et C93)
- **Contexte** : findings F9 (la contre-passation d'une écriture de pièce lettre `{L, L'}` ; une
  dissolution manuelle ouvrirait la paire pour toujours, R5 interdisant de la relettrer) et F10 (le
  manuel `:1169`, `:1696` et `api-external.md` `:325`, `:386` disent le règlement d'une facture créditée
  « paiement **à lettrer** » ; R5 l'interdit — motif `OwnedBySettlement` — et le groupe `document` ne peut
  le prendre).
- **Retenu** : dissolution manuelle d'un groupe `reversal` dont une ligne relève d'une pièce (motifs
  `OwnedBy*`, rangs 3 à 6) → 409 `LETTERING_LINE_OWNED_BY_DOCUMENT` (clé réutilisée) ; test nommé. Le
  manuel et l'API sont réécrits **dans la 15-1a** sans promesse de lettrage manuel ; le **traitement**
  du règlement d'une facture créditée est renvoyé à la 15-1a2 (section « Reçu de la 15-1a » de sa fiche :
  exception à R5, groupe incluant l'avoir, ou cas ouvert et dit).
- **Écartées** : exempter de R5 la relettre `{L, L'}` (une seconde voie de lettrage manuel de lignes de
  pièce) ; laisser la promesse au manuel jusqu'à la 15-1a2 (documentation fausse dès la 15-1a).
- **Réversible** : oui.

## C107 — 15-12 découpée en 15-12a (l'ordre) et 15-12b (le filet) à la validation P2 (révise C89 point 6 et C100 point 8)
- **Contexte** : validation P2 de la 15-12 (Sonnet ×2 ; rapports `target/gate-logs/15-12-p2-{R,F}.md`) —
  F3 (MEDIUM) : la dérogation au découpage ne relève pas de l'exception que le `CLAUDE.md` codifie (cycles
  Cargo, merges intermédiaires intestables) ; R3 (LOW) : « la dépendance B → A est à sens unique » est
  fausse (l'AC 5 de A rendait le message de l'AC 9, rangé dans B ; doc-comments de A citant le filet).
  **Décision de l'orchestrateur**, inscrite ici : découper maintenant, quand la coupe ne coûte que des
  fiches (aucun code écrit), plutôt qu'au premier défaut recyclé, où elle coûterait une branche entamée.
- **Retenu** : coupe de repli déjà décrite par la fiche, avec l'AC 9 (message neutre et sa clé) déplacé
  dans la **15-12a**, et toute dépendance A → B réglée (A livrable et testable seule) :
  - **15-12a** « l'ordre » (`refs #543`) : AC 1-7, 9, 13, 14, 17, 22, et la part A des AC 19, 21, 23 ;
  - **15-12b** « le filet » (`closes #543`) : AC 8, 10, 11, 12, 15, 16, 18, 20, et la part B des AC 19,
    21, 23.
  Numérotation des AC **conservée** de la 15-12 (références croisées stables : « AC 13 b », « mutation
  (ii) ») ; un AC partagé porte la mention « part A » / « part B ». Tâches renumérotées par fiche, chacune
  avec son origine (« ex-T4 »). La fiche `15-12-cloture-dans-l-ordre.md` devient un index ; sa version
  complète avant découpage est au commit `dae3a618`.
- **Ce que la coupe ne règle pas, et qui est dit** : recomptés au même critère (un module dont seul un doc-comment change compte), A touche 8
  modules de premier niveau et B 9 ; les deux dépassent encore 5. Le remède de la règle (patron puis déploiement mécanique) n'a pas de
  prise ; un découpage plus fin séparerait des AC qui se testent ensemble (l'ordre de `close` et sa
  concurrence ; le filet et son inventaire). Signal déclaré au Project Lead, dans les deux fiches.
- **Écartées** : garder la story entière (dérogation hors de l'exception codifiée) ; découper en trois
  (écran à part) — l'écran des exercices porte à la fois le bouton (A) et le bandeau (B), un tiers écran
  dépendrait des deux autres.
- **Réversible** : oui (fiches seulement).

## C108 — 15-12b : le refus `LATER_FISCAL_YEAR_CLOSED` du lot, trois voies, un seul constructeur, code littéral
- **Contexte** : R1 = F1 (MEDIUM) — `accept_one_rule` (`routes/reconciliation.rs:2528-2555` sur `8f9811d8`)
  n'emprunte pas `project_error_to_failed_proposal` : son repli en ligne rend `DATABASE_ERROR`. F8 (LOW) :
  le décompte « 26 codes » de `failed-proposal-label.ts` ne compte que les littéraux + `ACCOUNT_NOT_POSTABLE`.
- **Retenu** : un constructeur unique `later_fiscal_year_closed_failed_proposal` (patron de
  `period_locked_failed_proposal`, `:186-193` : « un seul constructeur pour les DEUX sites »), qui pose le
  code par un **littéral** `"LATER_FISCAL_YEAR_CLOSED"` (visible au `grep` du décompte, qui passe à 27 :
  26 littéraux + `ACCOUNT_NOT_POSTABLE`), appelé par un bras de `project_error_to_failed_proposal`
  (facture, ventilé) **et** par la branche en ligne de `accept_one_rule`, à côté de son `PeriodLocked`.
  Un test par voie ; mutations (vii-a) et (vii-b).
- **Écartées** : aiguiller l'`Err(e)` de la règle vers `project_error_to_failed_proposal` — son bras
  `NotFound → PROJECT_NOT_FOUND` mal-étiquetterait un `NotFound` étranger au projet, ce que le commentaire
  de la règle (`:2531-2534`) et le doc-comment du mapper (`:180-181`) écartent expressément ; poser le code par `err.error_code()` (invisible au
  `grep` du décompte).
- **Réversible** : oui.

## C109 — 15-12a : la preuve de l'enveloppe de la clôture est un test HTTP ; celle de la création, un test HTTP si T0 trouve un interblocage forçable, sinon la revue
- **Contexte** : R2 (MEDIUM) — la mutation (ix) « clôture hors de son enveloppe » ne peut être tuée par un
  test `kesh-db` : l'enveloppe y serait écrite par le test lui-même. `close_fiscal_year` et
  `create_fiscal_year` restent `SansEcritureAuJournal`, que le volet (c) du registre n'examine pas.
- **Retenu** : test `fiscal_year_close_is_replayed_when_it_is_the_deadlock_victim` dans
  `crates/kesh-api/tests/rejeu_interblocage_e2e.rs`, patron des tests 2 à 5 (transaction lourde qui tient N,
  la route vue en attente à l'étape (c), la transaction demande M que la requête (b) a verrouillé,
  témoin `warn!` de `kesh_db::retry` nommant `fiscal_years::close`). Pour la création : même patron si la
  mesure de l'AC 13 b montre un interblocage dont la victime se laisse forcer ; sinon la mutation (x) est
  écrite « tenue par revue » au Dev Agent Record et au point (vi) du registre, comme `onboarding::finalize`.
  Le test 13 d (`kesh-db`) garde son objet — le cycle existe et se résout — sans prétendre tuer (ix).
- **Écartées** : retirer (ix) et tenir les deux enveloppes par la revue (un rouge possible vaut mieux
  qu'une relecture) ; classer les deux routes `Rejouee` (elles n'écrivent pas au journal : le registre
  mentirait sur sa première colonne).
- **Réversible** : oui.

## C110 — 15-12a : `OPEN_COVERING_DATE_SQL` reçoit `ORDER BY start_date ASC`
- **Contexte** : F2 (MEDIUM) — l'ordre « ascendant » de `find_open_covering_date` était affirmé (AC 3, 13 d,
  14) sans que la requête le tienne : pas d'`ORDER BY`, trois index utilisables sur `company_id`
  (`uq_fiscal_years_company_name`, `uq_fiscal_years_company_start_date`, index de la FK), plan au choix de
  l'optimiseur.
- **Retenu** : `… AND status = 'Open' ORDER BY start_date ASC LIMIT 1` (constante partagée avec
  `has_open_covering_date`, lecture seule : résultat inchangé, une seule ligne peut correspondre, les
  exercices ne se chevauchant pas). **Sans `, id`** : `(company_id, start_date)` est unique
  (`uq_fiscal_years_company_start_date`), le départage ne peut pas servir — et les deux autres requêtes du
  module (`FIND_LATER_CLOSED_SQL`, la neuve `FIND_EARLIER_OPEN_SQL`) s'écrivent `ORDER BY start_date ASC`.
  `EXPLAIN` des trois requêtes mesuré en T0 (index `uq_fiscal_years_company_start_date`, pas de
  `filesort`), écrit au Dev Agent Record.
- **Écartées** : écrire que l'ordre dépend du plan (le doc-comment canonique de l'AC 14 et le test 13 d
  reposeraient sur un fait non tenu) ; `FORCE INDEX` (fige un nom d'index dans une requête).
- **Réversible** : oui.

## C111 — 15-12a : le message neutre garde le conseil de la contre-passation et prescrit l'ordre LIFO ; l'ancienne clé est alignée
- **Contexte** : F5 (LOW) — le message neutre de l'AC 9 faisait perdre au `PUT`/`DELETE` le conseil de la
  contre-passation, et la fiche renvoyait à un bandeau qui ne s'affiche pas à la création d'un exercice
  (état sain). R4 (LOW) — l'ancienne clé `journal-entries-modify-blocked-later-fiscal-year-closed`
  prescrit encore « Un administrateur peut rouvrir cet exercice », le geste que C100 a réfuté.
- **Retenu** : message de la clé neuve terminé par la marche à suivre valable dans tous les cas (corriger
  une écriture par contre-passation ; sinon, un administrateur rouvre les exercices clôturés en commençant
  par le plus récent) ; l'ancienne clé, toujours lue par la fiche d'écriture, reçoit la même prescription,
  aux quatre catalogues et dans le repli de `blocker-messages.ts`.
- **Écartées** : garder l'ancienne clé telle quelle et l'écrire comme limite (texte faux dans l'état même
  où il s'affiche) ; renvoyer au bandeau (absent à la création).
- **Réversible** : oui.

## C112 — Ordre 15-12a → 15-12b → 15-1a ; le prérequis réel de la 15-1a est la 15-12a
- **Contexte** : C105 faisait de la 15-12 un prérequis de la 15-1a (« les clos forment un préfixe »). R6 et
  F11 (LOW) : la 15-1a dit la clôture « non rejouée » (vrai sur `main`, faux dès la 15-12a), et l'invariant I
  n'est garanti que pour les états sains.
- **Retenu** : **la 15-12a est le prérequis réel** (elle tient l'invariant et rend la clôture rejouée). La
  15-12b n'est pas requise par la 15-1a — son filet couvre la création et la suppression d'écritures, pas
  le lettrage — mais **passe avant de préférence** : les deux touchent `journal_entries::delete_in_tx`
  (la 15-12b lève la condition `enforce_ownership` sur la lecture des postérieurs clos, la 15-1a y pose
  `ENTRY_LETTERED` hors de ce drapeau) ; la seconde mergée écrit la précédence des deux refus. Ordre écrit :
  **15-12a → 15-12b → 15-1a → 15-1a2 → 15-1b → 15-1c**. Ce que la 15-1a doit porter (fiche en validation,
  non modifiée ici) est rendu à l'orchestrateur : corriger « la clôture n'est pas rejouée » et dire ce que
  devient sa règle des exercices dans l'état hérité (tolérer, garder comme 15-8a, ou écrire la limite).
- **Écartées** : faire de la 15-12b un prérequis dur (rien dans la 15-1a ne lit le filet).
- **Réversible** : oui.

## C113 — 15-1a : une ligne est « en période ouverte » si son exercice est ouvert, qu'aucun exercice postérieur n'est clos et que sa date dépasse le verrou de période ; lettrer comme délettrer en exigent une (révise C105)
- **Contexte** : validation P2 de la 15-1a (Sonnet ×2 ; `target/gate-logs/15-1a-p2-{R,F}.md`). F-3 (MEDIUM) :
  R7 permettait de lettrer et délettrer dans une période verrouillée (« la marque n'est pas une écriture »),
  au rebours de l'argument F6 qui fonde le refus pour les exercices clos — la vue « au » d'une date (15-1b)
  compte soldé à X tout groupe entièrement daté ≤ X, quelle que soit la date de pose ; lettrer deux lignes
  du 15 mars sous un verrou au 31 mars réécrit les « postes ouverts au 31.03 » remis au fiduciaire. Le
  manuel (`user-manual.tex:578-583`) dit que verrouiller « fige » la période. S'y ajoute le point reçu de
  la 15-12 (C112) : dans l'**état hérité** (sauvegarde v0.12.x, SQL direct), un groupe tout entier dans
  N ouvert sous N+1 clos passait la règle « au moins une ligne sur un exercice ouvert », et ni la 15-12a
  ni la 15-12b ne le gardent.
- **Retenu** (décision de l'orchestrateur pour le verrou de période ; choix de l'agent pour l'état
  hérité) : une ligne est **« en période ouverte »** ssi (i) son exercice est `Open`, (ii) aucun exercice
  postérieur n'est `Closed` (patron de la garde 15-8a — on **garde** l'état hérité plutôt que de le
  tolérer ou d'écrire la limite), (iii) sa date est strictement postérieure à `books_locked_through`
  (seuil inclusif, comme partout). En mode `Manual`, lettrer comme délettrer exigent au moins une telle
  ligne ; sinon 409 **`LETTERING_ALL_LINES_IN_CLOSED_PERIODS`** (une clé, qui **remplace**
  `LETTERING_FISCAL_YEARS_CLOSED` — elle ne disait pas le verrou). La borne se lit ordinairement, après
  les verrous d'exercices (même tolérance qu'à la création et au `PUT`). Les informations (ii) viennent
  de la même lecture verrouillante que les exercices (C114). En mode `System`, non évaluée par la
  primitive : la ligne de l'exercice tenu vient d'être écrite et a passé les gardes de la création ; le
  (ii) n'y est garanti qu'avec la 15-12b — limite écrite. Manuel : phrase symétrique au § du verrou de
  période ; tests nommés (borne exacte, état hérité posé par SQL). Nom « en période ouverte » choisi pour
  ne pas se confondre avec « ligne ouverte » (non lettrée).
- **Écartées** : garder « permis » sous le verrou et écrire que la vue « au D » est réécrite après coup
  (contredit le manuel et l'argument F6) ; deux codes distincts (exercice / verrou) — une ligne du groupe
  peut être close pour une raison, une autre pour l'autre, le refus porte sur le groupe ; tolérer l'état
  hérité (la vue « au » de N+1 serait réécrite, précisément ce que la règle protège) ; écrire la limite
  sans garde (une requête suffit à la fermer).
- **Réversible** : oui (fiche seulement) ; revenir à « permis » sous le verrou retire la condition (iii).

## C114 — 15-1a : les exercices se verrouillent par un parcours ascendant de `(company_id, start_date)`, pas par un `ORDER BY` sur une liste d'`id` (révise C103 point 2)
- **Contexte** : R2-2 = F-5 (MEDIUM / LOW) — `SELECT … WHERE id IN (…) … ORDER BY start_date, id FOR
  UPDATE` ne fixe pas l'ordre d'acquisition : InnoDB verrouille au fil du parcours (ici la clé
  primaire) et trie ensuite, soit l'ordre des `id`, celui que C103 voulait écarter. Décision de
  l'orchestrateur : s'aligner sur la 15-12a (C110) ou verrouiller un par un après tri en Rust ; choisir la
  forme qui garantit l'ordre et écrire la mesure T0.
- **Retenu** : (a) `MIN(start_date)` des exercices des lignes, lu **sans verrou** (`start_date` immuable,
  15-12a AC 3 (a)), après l'acte 1 ; (b) `SELECT id, start_date, status FROM fiscal_years WHERE company_id
  = ? AND start_date >= ? ORDER BY start_date ASC FOR UPDATE` — parcours d'intervalle de
  `uq_fiscal_years_company_start_date`, qui verrouille l'exercice le plus ancien du groupe **et tous les
  postérieurs** dans l'ordre chronologique, et rend d'un coup l'information « exercice postérieur clos »
  de C113. `EXPLAIN` mesuré en T0 (attendu : `range` sur cet index, sans `filesort`), écrit au Dev Agent
  Record ; **repli écrit d'avance** si le plan diffère : `(id, start_date)` lus sans verrou, tri en Rust,
  verrous un par un. Test avec deux exercices aux `id` inversés par rapport aux dates, face à `close`.
  Pas de `, id` (`(company_id, start_date)` est unique, C110).
- **Écartées** : la forme « un par un » d'emblée (plus de requêtes, et elle ne rend pas l'information
  « postérieur clos » sans une requête de plus) ; `FORCE INDEX` (fige un nom d'index — écarté aussi par
  C110) ; garder `ORDER BY` sur la liste d'`id` en se fiant au plan.
- **Réversible** : oui.

## C115 — 15-1a : le motif du bump `min_required` réécrit au plus juste, et ce que `sqlx` fait déjà (révise le motif de C101, pas sa décision)
- **Contexte** : F-1 (MEDIUM) — C101 et la fiche disaient la v0.12.1 « seul binaire antérieur publié »
  ne modifiant aucune écriture manuelle. Faux : v0.10.0, v0.11.0, v0.11.1 sont publiés, routent `PUT` et
  `DELETE /journal-entries/{id}` (`v0.11.1:crates/kesh-api/src/lib.rs:337-338`, sans `ENTRY_IS_POSTED`)
  et démarrent contre `min_required = '0.10.0'`. Le motif ira dans l'en-tête de la migration, que P8 fige.
  En vérifiant, l'agent a relevé un fait que ni C101 ni la passe n'avaient écrit : `kesh_db::MIGRATOR`
  garde `ignore_missing = false` (défaut de `sqlx` 0.8.6, `Migrator::run` →
  `validate_applied_migrations` → `MigrateError::VersionMissing`), si bien qu'un binaire antérieur
  refuse **déjà** de démarrer contre une base portant une migration qu'il ne connaît pas
  (`kesh-api/src/main.rs:138`) ; et l'import de la v0.12.1 refuse une colonne inconnue
  (`check_schema_compat` (c1), `unknownColumns`).
- **Retenu** : la décision de bumper tient (P1/P2 ne présument pas de `sqlx`, et le bump rend le refus
  explicite, précoce — avant `sqlx`, en nommant les versions — et porté au manifeste des sauvegardes).
  Le motif écrit : tout binaire publié antérieur (v0.10.0 à v0.12.1) ignore le lettrage ; les v0.10.0 à
  v0.11.1 modifient et suppriment des écritures manuelles, les v0.12.x annulent règlements, paiements et
  rapprochements, sans dissoudre les groupes. **En-tête de migration arrêté dans la fiche** (cinq lignes)
  pour être recopié tel quel. ⚠️ **Signalé à l'orchestrateur** : la prémisse de la § « Migration breaking
  policy » du `CLAUDE.md` (« un binaire antérieur pourrait démarrer et corrompre ») n'est pas vraie sous
  le réglage actuel de `sqlx` — le bump n'est pas la seule barrière, il en est la forme lisible. Ce n'est
  pas à cette story de réécrire la politique.
- **Écartées** : garder le motif « v0.12.1 seule » (faux, et figé par P8) ; renoncer au bump au motif
  que `sqlx` refuse déjà (la politique l'exige, et un futur `set_ignore_missing(true)` rouvrirait le
  risque en silence).
- **Réversible** : oui tant que la migration n'est pas écrite ; ensuite, non (P8).

## C116 — 15-1a : la contre-passation relit les lignes de l'écriture inverse après les avoir lettrées
- **Contexte** : F-4 (MEDIUM) — `reverse_in_tx_inner` rend `created`, lu par `create_in_tx_inner` avant
  tout lettrage ; la route en fait son `201`. L'API, ouverte aux clés, dirait `letteringCode: null` sur une
  ligne lettrée en base ; aucun test d'AC9 ne lisait le corps.
- **Retenu** : si au moins un groupe a été posé, `created.lines` est remplacé par une relecture `SELECT
  {LINE_COLUMNS} … WHERE entry_id = ? ORDER BY line_order` dans la transaction, **dans
  `reverse_in_tx_inner`** (tous les appelants de `reverse_in_tx` en profitent) ; AC9 (a) asserte le corps
  du `201`.
- **Écartées** : relire dans la route seule (les autres appelants garderaient une valeur fausse) ;
  recalculer les champs en mémoire depuis le retour de `create_group_in_tx` (deux sources de la même
  vérité — la base et un calcul).
- **Réversible** : oui.

## C117 — 15-1a / 15-12b : précédence des refus dans `delete_in_tx`, écrite et testée par la seconde des deux à merger
- **Contexte** : reçu de la remédiation de la 15-12 (`9b403aaf`) — la 15-12b rend l'étape 2-bis
  (« exercice postérieur clos ») inconditionnelle ; la 15-1a pose 3-ter-bis (`ENTRY_LETTERED`),
  inconditionnelle aussi. Les deux se recouvrent sur la dévalidation et sur la route.
- **Retenu** : ordre **2-bis avant 3-ter-bis** (l'état des exercices parle avant la marque, comme
  `FISCAL_YEAR_CLOSED` avant `LATER_FISCAL_YEAR_CLOSED`). La seconde story mergée l'écrit au doc-comment
  « Ordre des refus » de `delete_in_tx` et le teste par une paire (écriture lettrée dans N, N+1 clos,
  `enforce_ownership` à `false` et à `true` → `LaterFiscalYearClosed`), mutation « permuter les étapes ».
  Dans l'ordre préféré (C112), c'est la 15-1a (T0 relève l'état, T4 l'applique). ⚠️ Si la 15-1a est
  mergée la première, la paire revient à la 15-12b : **à reporter dans la fiche 15-12b par
  l'orchestrateur** (fiche en validation P3, non modifiée ici).
- **Écartées** : 3-ter-bis avant 2-bis (dirait « délettrez d'abord » à qui ne pourrait de toute façon
  rien supprimer — et le délettrage serait lui-même refusé, C113 (ii)).
- **Réversible** : oui.

## C118 — 15-1a : pas de découpage malgré le premier critère franchi au grain des modules métier ; couture écrite
- **Contexte** : F-13 (LOW) — l'argument de P1 (« amendement D5 : aucun recyclage ») ne qualifiait que le
  second critère ; au grain des modules métier, le premier est franchi (une dizaine) ; F-1 et F-2 sont
  des faits écrits par la remédiation de P1.
- **Retenu** : pas de découpage. La couture naturelle — (i) schéma, primitive, routes, audit, exports ;
  (ii) gardes d'AC8, R6, frontend, manuel — produit deux stories qui touchent toutes deux
  `repositories/journal_entries` et `kesh-api`, la seconde intestable sans la première ; les défauts nés
  de P1 sont des faits recopiés (un motif, une liste de tests), non une règle métier qui ne converge
  pas. **Déclencheur écrit** : si la P3 trouve un défaut né d'un correctif de P2 sur une règle métier
  (R7, AC4, AC5, AC8), découper selon cette couture avant toute P4. Signal déclaré au Project Lead.
- **Écartées** : découper maintenant sur le modèle de C107 (la 15-12 avait deux moitiés indépendantes ;
  ici la seconde dépend entièrement de la première).
- **Réversible** : oui (fiches seulement).

## C119 — 15-12a : les exercices antérieurs se verrouillent un par un par clé primaire, dans l'ordre lu sans verrou, puis relecture verrouillante après l'exercice clôturé (révise C110 ; à reporter sur C114)
- **Contexte** : validation P3 de la 15-12 (Opus ×2 ; `target/gate-logs/15-12-p3-{R,F}.md`). F2 (MEDIUM) —
  « ascendant **par construction** » était faux : un `ORDER BY` fixe l'ordre du résultat, InnoDB verrouille
  dans l'ordre du **parcours** choisi par l'optimiseur, et le dépôt a déjà mesuré un `filesort` qui
  verrouille tous les exercices d'une société (`opening_complement.rs:278-281`). L'ordre des `id` diverge
  de celui des `start_date` dès qu'un exercice antérieur est créé après un postérieur. **Décision de
  l'orchestrateur** : garantir l'ordre indépendamment du plan ; choisir entre `FORCE INDEX` + `EXPLAIN`
  mesuré et testé, et une lecture des `id` triés sans verrou suivie de verrous un par un — de préférence la
  forme dont la garantie ne dépend d'aucune mesure.
- **Retenu** : `close` en six temps — (a) `start_date` de Y sans verrou ; (b) `LIST_EARLIER_SQL` : `SELECT
  id … WHERE company_id = ? AND start_date < ? ORDER BY start_date ASC`, sans verrou, tous statuts ; (b')
  `LOCK_EARLIER_BY_ID_SQL` : `SELECT id … WHERE id = ? AND company_id = ? FOR UPDATE`, un par un dans cet
  ordre, par une boucle Rust (un exercice disparu est ignoré) ; (c) Y par clé primaire `FOR UPDATE`
  (`fetch_optional` → `NotFound`), verdict « déjà clos » ; (d) `FIND_EARLIER_OPEN_SQL … FOR UPDATE`,
  **relecture verrouillante** qui rend le verdict `EarlierFiscalYearOpen` sur l'état validé ; (e) `UPDATE`,
  audit. L'ordre des antérieurs **connus** est celui du code. `OPEN_COVERING_DATE_SQL` reste **sans**
  `ORDER BY` (C110 révisé : un `ORDER BY` n'aurait fixé que le résultat, déjà unique, et rien ne dépend de
  l'ordre de ses verrous). `EXPLAIN` de (d) et de `FIND_LATER_CLOSED_SQL` mesuré en T0 à titre
  **descriptif**, dans les deux régimes (une / plusieurs sociétés).
- **Ce qui est perdu, écrit dans la fiche** : (1) les verrous de clé suivante et d'intervalle d'un
  parcours — un exercice antérieur **créé** après la vue de (a) (fantôme) n'est ni listé ni verrouillé ;
  (d) le lit (lecture verrouillante : état validé), et une création postérieure à la prise de Y l'attend
  (sa garde `find_later_closed_in_tx` examine Y), sauf si un exercice clos s'interpose, auquel cas elle
  est refusée ; (2) une requête par antérieur ; (3) (d) reste un parcours dont les acquisitions **nouvelles**
  dépendent du plan — fantômes, ou autres lignes de la société sous un mauvais plan —, mais elle vient
  quand la clôture tient Y et les antérieurs connus : un cycle qui y naît est absorbé par le rejeu de la
  clôture ; l'invariant I n'en dépend pas. Effets de bord favorables : le point d'arrêt des tests 13 d et
  de l'AC 6 devient (c), déterministe ; l'hypothèse « le parcours garde le verrou d'une ligne qu'il ne
  retient pas » n'a plus d'objet (M est verrouillé explicitement).
- **Pour la 15-1a (C114, à réviser par l'orchestrateur)** : même forme — lire sans verrou les `id` (et
  `start_date`) des exercices concernés, `ORDER BY start_date ASC` ; les verrouiller **un par un** par
  `SELECT … WHERE id = ? AND company_id = ? FOR UPDATE` dans cet ordre, en lisant `status` sous le verrou ;
  puis une relecture verrouillante **seulement** si un fantôme peut changer le verdict. Pour la règle
  « en période ouverte » de C113, l'information « postérieur clos » se lit sur les lignes verrouillées ; un
  exercice postérieur **créé** pendant ce temps naît `Open` et ne peut être clôturé tant qu'un antérieur
  ouvert tenu par le lettrage l'est (la clôture, C119, le verrouille avant) — à vérifier par la 15-1a sur
  sa propre liste, non supposé ici. Le « repli écrit d'avance » de C114 devient la forme principale ; le
  parcours `start_date >= ? ORDER BY start_date ASC FOR UPDATE` et son `EXPLAIN` attendu sont abandonnés.
- **Écartées** : `FORCE INDEX (uq_fiscal_years_company_start_date)` avec test sur l'`EXPLAIN` — la garantie
  y reste une mesure, valable pour la version de MariaDB testée, et fige un nom d'index (le dépôt emploie
  `FORCE INDEX (PRIMARY)` en 15-5d pour **borner** des verrous partagés, pas pour ordonner des verrous
  exclusifs) ; garder le parcours et écrire que l'ordre dépend du plan (l'argument contre le cycle avec
  `reopen`, non rejouée, n'aurait plus de fondement) ; passer la transaction de clôture en `READ
  COMMITTED` pour relire sans verrou l'état validé (nouveau patron d'isolation dans un dépôt qui raisonne
  partout en `REPEATABLE READ`).
- **Réversible** : oui (fiches seulement).

## C120 — 15-12b : les prédicteurs d'annulation ne connaissent pas le filet — angle mort assumé, non un rang neuf
- **Contexte** : F1 (MEDIUM) — `settlement_cancellation::settlement_entry_cancel_blocker` (queue commune
  de `cancelBlockedBy`, `settlementCancelBlockedBy`) contrôle au rang 5 qu'un exercice ouvert couvre le
  jour, pas qu'aucun postérieur à celui-ci n'est clos ; dans cet état hérité, quatre écrans annoncent
  l'annulation possible et le clic rend `400 LATER_FISCAL_YEAR_CLOSED`. Option préférée de
  l'orchestrateur : un rang après `NoOpenFiscalYearToday`, avec test ; sinon l'angle mort écrit.
- **Retenu** : **angle mort assumé, écrit** — au doc-comment de `settlement_entry_cancel_blocker`, à côté
  de la « Limite assumée » existante du verrou de période du jour (même nature : refus du socle au clic,
  non reproduit par la lecture) ; dans `docs/api-external.md` (`:319`, `:353`, `:361`) ; un test qui fixe le
  comportement (prédicteur `None`, annulation `LaterFiscalYearClosed`), patron C-15-8-29, pour qu'il
  rougisse le jour où le rang est ajouté. **Issue à ouvrir** (P3) pour le rang.
- **Pourquoi pas l'option préférée** : le rang ajoute une variante à `SettlementCancelBlocker`, trois bras
  aux `match` exhaustifs de `kesh-api/src/errors.rs`, trois `switch` exhaustifs (`never`) côté écran
  (`settlement-cancel-blocked.ts`, `reconciliation-cancel.ts`, `invoice-cancel.ts`) et leurs types, des
  clés ×4 et la documentation des champs : cinq modules de premier niveau de plus, sur une fiche que la
  dérogation (C121) vient d'accepter à dix — pour un état hérité étroit (un exercice **futur** clôturé
  d'avance) dont le refus au clic est exact et porte sa marche à suivre (AC 9 de la 15-12a). Le module a
  déjà un précédent d'angle mort de même nature.
- **Écartées** : le rang (ci-dessus) ; ne rien écrire (site non résolu et non écrit — la règle du
  `CLAUDE.md` l'interdit).
- **Réversible** : oui — l'issue porte le rang ; le test qui fixe l'angle mort rougira alors.

## C121 — 15-12a et 15-12b : dérogation à la règle de découpage, écrite dans la forme codifiée
- **Contexte** : F9 (LOW) — les deux fiches filles franchissent encore le premier critère (8 et 9
  modules ; 10 pour la 15-12b après C120) avec un « signal déclaré » qui n'avait pas la forme que le
  `CLAUDE.md` exige (section « Dérogation règle de splitting », risque accepté). **Décision de
  l'orchestrateur** : dérogation **acceptée** — la story a déjà été découpée une fois selon la seule couture
  naturelle (C107) ; un second découpage ferait des stories non testables isolément (exception écrite du
  `CLAUDE.md`).
- **Retenu** : section « Dérogation règle de splitting » dans chaque fiche (modules recomptés, motif,
  risque accepté — une revue moins fine par module ; mitigation : passe ciblée sur chaque remédiation,
  inventaires refaits à chaque passe), signal déclaré à Guy.
- **Écartées** : redécouper (sous-stories intestables seules) ; garder le seul « signal déclaré » (forme
  non codifiée).
- **Réversible** : oui (fiches seulement).

## C122 — 15-12a : les autres textes qui prescrivent une réouverture sans ordre restent hors périmètre, avec une issue
- **Contexte** : F8 (LOW) — le symptôme que C111 corrige pour l'ancienne clé (« rouvrez l'exercice »,
  geste refusé par LIFO dès qu'un exercice plus récent est clos) vit aussi dans les messages d'annulation
  refusée pour exercice clos (règlement, rapprochement, facture fournisseur), dans
  `error-fiscal-year-reopen-blocked`, `error-opening-balances-first-year-closed`, leurs replis Rust et
  frontend, et quatre phrases du manuel utilisateur (relevé par la valeur sur `5e4bec50`, liste dans la
  fiche 15-12a, § « Hors périmètre »).
- **Retenu** : hors périmètre, préexistant, **écrit** dans la 15-12a avec la liste des sites ; **issue à
  ouvrir** (P3) pour les aligner sur la prescription de C111.
- **Écartées** : les aligner ici (cinq clés ×4, trois replis, trois modules frontend et le manuel de plus,
  hors du défaut #543) ; ne rien écrire.
- **Réversible** : oui.
- **Suivi** *(ajouté en revue P1 de la 15-12a, A3)* : l'issue est
  [#569](https://github.com/guycorbaz/kesh/issues/569) ; la fiche 15-12a et les trois replis Rust
  d'`errors.rs` la citent.

## C123 — 15-12b / 15-12a : deux tranchages de forme (projectId de la voie « règle », rubriques du CHANGELOG)
- **Contexte** : R4 (LOW) — la fiche promettait `projectId` « s'il y a un projet » alors que la branche en
  ligne d'`accept_one_rule` passe `None` à son `PeriodLocked` (la règle a un `default_project_id`). F10
  (LOW) — la 15-12a rangeait sous `### Corrigé` un changement de contrat d'API (refus neufs, clôture ouverte
  aux clés).
- **Retenu** : (1) voie « règle » : `projectId` **omis** (`None`), comme son `PeriodLocked` — les deux refus
  de la même branche publient les mêmes `details` ; le test l'asserte ; (2) CHANGELOG : le défaut sous
  `### Corrigé`, le changement de contrat sous `### Modifié`, patron des entrées #532 de `[0.13.0]`.
- **Écartées** : (1) passer `default_project_id` au seul refus neuf (deux refus d'une même branche aux
  `details` divergents) ; (2) tout sous `Corrigé` (une intégration par clé chercherait le changement de
  comportement sous `Modifié`).
- **Réversible** : oui.

## C-15-12a-1 — 15-12a (T0) : un message propre au refus de création d'un exercice
- **Contexte** : F3 de la validation P4 (LOW) — le texte de l'AC 9 (`error-later-fiscal-year-closed`),
  qui sert au `PUT` et au `DELETE` d'une écriture, servait aussi à `POST /fiscal-years` (AC 5) ; sa phrase
  « une écriture se corrige alors par une contre-passation » ne répond à rien à l'écran de création.
  L'AC 9 exige de garder ce conseil pour les écritures (F5 de P2) ; le message ne pouvait donc pas devenir
  commun en le perdant. L'orchestrateur demandait un « message adapté à la création ».
- **Retenu** : une clé dédiée, `error-fiscal-year-create-later-closed` (×4 locales, repli Rust), rendue
  par une variante d'`AppError` (`FiscalYearBeforeClosedYear { fiscal_year_id, fiscal_year_name }`) que
  `map_create_error` produit à partir de `DbError::LaterFiscalYearClosed`. **Code (`LATER_FISCAL_YEAR_CLOSED`),
  statut (400) et `details` inchangés** — une intégration ne voit que le texte changer. La 15-12b ne lit
  que la clé de l'AC 9, qui ne bouge pas.
- **Écartées** : un seul message sans conseil de contre-passation (contredit l'AC 9) ; un message unique
  allongé des deux cas (plus long, et à moitié hors sujet à chaque fois) ; un champ de plus dans
  `DbError::LaterFiscalYearClosed` pour choisir le texte (fait porter à la couche base une décision
  d'affichage).
- **Réversible** : oui (une variante et une clé à retirer).

## C-15-12a-2 — 15-12a (revue P1) : la preuve du verrou de la garde de création isole la garde
- **Contexte** : A1 (MEDIUM) = B-2 = E1 — dans les 13 b1 / b2, la création bute dès `find_overlapping`
  (verrou de borne de son parcours) et ne lit Y clos dans sa garde qu'après ; la mutation « garde non
  verrouillante » y reste verte. Le verrou de la garde n'était prouvé par aucun test.
- **Retenu** : un test 13 b3 où W rejoue la garde puis l'`INSERT` de `create` **sans** le pré-contrôle
  `find_overlapping`, dont le verrou masquerait celui de la garde ; la clôture lancée entre les deux doit
  attendre en (c) et nommer X. Mutation (xi) (`FOR UPDATE` retiré de `find_later_closed_in_tx`) jouée :
  seul le 13 b3 rougit. Le cas « fantôme validé entre (a) et (c) » reste sans test, écrit comme tel.
- **Écartées** : rejouer tous les gestes de `create` (le pré-contrôle tiendrait Y et le test ne
  prouverait rien de la garde) ; seulement écrire le `FOR UPDATE` de la garde « défensif, non éprouvé »
  (E1) — le test est bon marché et déterministe.
- **Réversible** : oui (un test).

## C-15-12a-3 — 15-12a (revue P1) : le message neutre ne promet que ce que le code garde
- **Contexte** : B-1 (MEDIUM) = E4 — `error-later-fiscal-year-closed` disait « rien ne peut être
  enregistré, modifié ou supprimé avant sa date de début », alors que seuls `PUT` et `DELETE` le gardent
  (la saisie le sera par la 15-12b), contrairement au manuel (`user-manual.tex:712`) et à
  `api-external.md`. L'AC 9 avait écrit le texte large en comptant sur la 15-12b avant le tag.
- **Retenu** : « aucune écriture datée avant sa date de début ne peut être modifiée ni supprimée tant
  qu'il l'est. Une telle écriture se corrige par une contre-passation ; … », aux quatre locales et au
  repli Rust ; assertion ajoutée au test HTTP de contre-passation (`journal_entry_reversal_e2e.rs`) qui
  refuse « enregistr ». **La 15-12b, en posant sa garde de saisie, élargira ce texte** (à reporter dans
  sa fiche par l'orchestrateur : la fiche 15-12b dit le message « inchangé ici »).
- **Deux clés voisines, gardées** (B-5) : `journal-entries-modify-blocked-later-fiscal-year-closed` est
  l'explication d'écran d'une écriture affichée (« cette écriture ») ; `error-later-fiscal-year-closed`
  la réponse du serveur, qui ne présuppose pas l'objet. Le pourquoi est écrit au mapping
  (`kesh-api/src/errors.rs`). **Registre italien** : les clés neuves de l'écran des exercices
  (`Chiudi prima…`) restent au tutoiement, registre que le glossaire mesure comme celui de l'italien
  (`docs/i18n-glossaire.md` § Registre) et qu'emploient leurs voisines (`Riapri prima…`) ; la famille
  `journal-entries-modify-blocked-*` (« Correggetela ») est au registre de courtoisie depuis avant la
  story. Chaque message tient un seul registre ; l'alignement du fichier relève des rollouts, comme le dit
  le glossaire.
- **Écartées** : garder le texte large jusqu'à la 15-12b (faux entre deux merges, et contraire au
  manuel) ; unifier les deux clés (l'une présuppose l'écriture, l'autre non) ; passer les clés neuves au
  vouvoiement (elles détonneraient sur leur propre écran, et contre le glossaire).
- **Réversible** : oui (textes).

## C-15-12a-4 — 15-12a (revue P1) : l'infobulle d'un bouton désactivé portée par une enveloppe
- **Contexte** : B-3 (LOW) — `button.svelte` pose `disabled:pointer-events-none` : le `title` d'un bouton
  désactivé ne s'affiche pas au survol, alors que le manuel promet l'infobulle de « Clôturer ». Même
  défaut, préexistant, sur « Réouvrir ».
- **Retenu** : chaque bouton enveloppé d'un `<span class="inline-flex" title=…>` qui reçoit le survol ; le
  bouton garde son `title` (tests existants). Appliqué aux **deux** boutons (propagation du symptôme).
  Vitest et E2E assertent l'attribut de l'enveloppe ; un `title` natif ne se voit pas dans Playwright.
- **Écartées** : retirer la promesse du manuel (le défaut resterait) ; retirer `pointer-events-none` du
  composant partagé (effet sur tout le produit) ; un composant d'infobulle (hors proportion).
- **Réversible** : oui.

## C-15-7a2-1 — Forme des neuf handlers : corps transactionnel + `conclude_step`

- **Contexte** : la 15-7a2 (développement) réunit chaque route d'étape en une transaction (AC 8). `lock_state_at_step` reçoit `tx` par référence et ne peut pas annuler (R2-7).
- **Retenu** : chaque handler valide le corps, garantit la ligne d'état (`get_or_init_state`), ouvre la transaction, exécute un corps (`async fn …_in_tx` ou bloc `async` borrowant `&mut tx`) qui rend `Result<OnboardingState, AppError>`, puis `conclude_step(pool, tx, result)` : `COMMIT` puis `response_with_stub` après commit, ou annulation et l'erreur d'origine. Deux helpers partagés : `complete_step` (`update_step_in_tx` à la version verrouillée + `record_step_completed_in_tx`, en dernier) et `update_company_in_tx` (`companies::update_in_tx` + `company.updated` ssi `version` a bougé, projection par route). Une macro `company_select!` porte la liste de colonnes de `Company` (deux requêtes : première société `FOR UPDATE`, société par id `FOR UPDATE`) et remplace les copies en ligne de `ensure_company_with_language` et de `finalize_inner`.
- **Écarté** : une fermeture générique `run_step(|tx| async {…})` (emprunt d'une transaction par une fermeture asynchrone : signatures HRTB lourdes pour un gain nul) ; réécrire `finalize_inner` sur `conclude_step` (son `retry_app_on_deadlock`, ses chemins d'erreur et son retour idempotent sont préservés tels quels — AC 8 : « `finalize` garde sa transaction »).
- **Réversible** : oui (forme interne au fichier).

## C-15-7a2-2 — Écarts de détail tranchés au développement

- **Contexte** : la fiche laisse ouverts quelques points de forme.
- **Retenu** : (a) `account.chart_loaded` ne s'écrit que si la variante a **réellement** inséré des comptes (`Vec` non vide ; un plan vide n'est pas « chargé ») ; (b) `details.language` de cette entrée est la `Language` sérialisée (`"DE"`), comme `accounting_language` dans `company.updated`, et non la clé de traduction minuscule ; (c) société absente sous verrou dans `org-type`, `accounting-language`, `coordinates`, `bank-account` ⇒ `AppError::Internal` (message de `finalize`), là où l'ancien `fetch_one` rendait un `NotFound` — cas inatteignable hors corruption, un seul message pour les cinq routes ; (d) `language` passe aussi par `companies::update_in_tx` : une langue inchangée ne bumpe plus `version` (règle de no-op de l'AC 3, étendue à `language` par l'AC 1) ; (e) la relecture de la société après `coordinates` ne se fait que si une entrée s'écrit (elle ne sert qu'à `after`).
- **Écarté** : écrire une entrée `account.chart_loaded` à `count = 0` ; garder l'`UPDATE` inconditionnel de la langue.
- **Réversible** : oui.

## C-15-7a2-3 — Mutations « deux commits » par `COMMIT` SQL en cours de transaction

- **Contexte** : la fiche demande de prouver que les tests 9 (a) et 9 (b) mordent sur « deux commits » / « société commitée avant le plan ». Le corps transactionnel tient `&mut Transaction` et ne peut pas commiter au milieu.
- **Retenu** : la mutation insère `sqlx::query("COMMIT")` juste après `companies::update_in_tx` (dans `update_company_in_tx` pour 9 (a), dans `update_company_coordinates_in_tx` pour 9 (b)) : la mutation de la société est commitée **avant** son entrée d'audit, que le déclencheur fait échouer — exactement la non-atomicité que l'AC 8 interdit. Un `COMMIT` placé après l'écriture d'audit ne mordrait pas (le déclencheur fait échouer l'audit avant).
- **Écarté** : réintroduire une transaction séparée pour la société (réécriture large, mutation moins locale).
- **Réversible** : sans objet (mutations jouées puis restaurées, fichier touché).

## C-15-7a2-4 — Pas de rejeu sur interblocage pour les huit routes d'étape de l'onboarding

- **Contexte** : revue de code P1 de la 15-7a2 (B-4 = E-4, LOW). Les huit routes d'étape autres que `finalize` tiennent désormais une transaction unique (`onboarding_state → companies → accounts` ou `bank_accounts`, puis 2 à 4 entrées d'audit) sans `retry_app_on_deadlock` ; seule `finalize` est rejouée. Un 1213 y rend 500.
- **Retenu** : ne pas les envelopper, et l'écrire comme angle mort assumé au point (iv) du doc-comment de `tests/audit_route_registry.rs` (renvoi au point (vi)). L'échec est sûr — annulation complète, ni mutation ni trace —, l'administrateur rejoue l'étape, et une installation en cours de configuration n'a pas de trafic concurrent : aucun cycle n'est démontré.
- **Écarté** : envelopper les huit routes (modification de code de production en remédiation de revue, pour un risque non démontré ; la règle de la passe interdisait de toucher le code de production) ; les classer `AvecEcritureAuJournal` (elles n'écrivent pas au journal comptable, convention du registre).
- **Réversible** : oui — envelopper une route est local (le patron `finalize` existe) ; à reprendre si un 1213 est observé sur l'onboarding.

## C-15-7a2-5 — 15-7a2 (clôture) : rebasée sur `origin/main` (`de285ea8`, 15-11b) ; registre par union, PDF admin régénéré

- **Contexte** : clôture de la 15-7a2 après la revue de code (P1 Sonnet ×3, 2 MEDIUM → remédiation `23b3e46e`, devenue `004341d0` → P2 ciblée Haiku 0). `origin/main` avait avancé d'un commit (la 15-11b : `config::env_nonempty`, test lexical (L), manuel admin).
- **Retenu** : rebase sur `de285ea8`. Deux conflits seulement : `admin-manual.pdf` (binaire — pris de `main` pendant le rebase puis **régénéré** sur le `.tex` fusionné sans conflit par git, contrôlé aplati : « 104 des 112 routes », « 104 + 6 + 2 = 112 », la réserve OLICo réécrite et les apports de la 15-11b sur `.env` sont tous présents) et ce registre (union, les entrées C-15-11b avant les C-15-7a2). `CHANGELOG.md` et `sprint-status.yaml` fusionnés sans conflit ; `last_updated` (27) pris après le (26) de `main`. Aucun conflit de code : `routes/onboarding.rs` passe déjà par `config::env_nonempty` (15-11b) et le test (L) est vert. **Partition du registre recomptée depuis `LIB_ROUTES`** (et non relue) : 112 routes = 104 `Traced` + 6 `Exempt` + 2 `NoMatter` ; colonne `Rejeu` sur les 115 entrées : 22 rejouées + 4 exemptées + 89 sans écriture — inchangée par le rebase (`main` n'a pas touché `audit_route_registry.rs`), le manuel reste juste. Gates complets rejoués sur l'état rebasé (backend, frontend, E2E).
- **Écartées** : merge de `main` dans la branche (historique moins lisible) ; garder l'un des deux PDF (il aurait omis l'apport de l'autre).
- **Réversible** : oui (rebase ; branche poussée, sans PR).


## C-15-6b-1 — 15-6b : une garde d'un tenant, `refuse_if_claim_account`, au-dessus des deux fonctions de la fiche

- **Contexte** : l'AC1 sépare la comparaison pure (`ensure_not_claim_account`) de la construction
  du refus (`claim_account_refusal`, numéro lu à l'échec) et veut que chaque site appelle l'une
  puis, sur `Err` seulement, l'autre. Neuf sites (règlement : contrepartie et arrondi ; solde :
  nature, reste d'arrondi, TVA due ; règlement fournisseur ; création d'un lot ; rapprochement :
  banque et arrondi) auraient chacun écrit le même `match` de quatre lignes.
- **Retenu** : une troisième fonction publique, `invoice_settlements::refuse_if_claim_account`,
  qui fait exactement les deux gestes dans cet ordre et rend `Result<(), DbError>`. Les sites
  l'appellent ; le rapprochement et la création de lot convertissent son `Err` (refus → item
  `failed[]`, toute autre erreur → `DATABASE_ERROR` / propagation). Les deux fonctions de la fiche
  restent publiques — la 15-6d emprunte la comparaison pure.
- **Écarté** : le `match` recopié site par site (DRY, et un site qui oublierait la lecture à
  l'échec seulement).
- **Réversibilité** : totale (une fonction d'enrobage).

## C-15-6b-2 — 15-6b : `GapAccountRole` au lieu du `DesignatedRole` de la fiche

- **Contexte** : la fiche (AC1, signature indicative) nomme `DesignatedRole` le sous-ensemble
  `{ Rounding, WriteOffNature, VatPayable }`. Un `DesignatedRole` existe déjà dans
  `company_invoice_settings.rs` (Story 15-5d) : les quatre **champs** que la validation et la
  saisie fournisseur écrivent (`Receivable`, `VatPayable`, `Payable`, `VatRecoverable`) — un autre
  ensemble, au sens voisin. Deux types homonymes dans `kesh_db::repositories`, dont un
  `VatPayable` de chaque côté, inviteraient à l'import du mauvais.
- **Retenu** : `invoice_settlements::GapAccountRole` (« compte d'écart », le vocabulaire de la
  fiche : « comptes d'écart du même geste »), dans le même fichier et au même usage que la fiche
  décrit (`ClaimSubject::Designated(GapAccountRole)`). Le doc-comment dit pourquoi.
- **Écarté** : garder l'homonyme (confusion à l'import) ; renommer celui de la 15-5d (hors
  périmètre, 26 occurrences).
- **Réversibilité** : totale (renommage).

## C-15-6b-3 — 15-6b : les LOW de la revue P1 qui exigent du code exécutable sont écrits en dette, pas corrigés

- **Contexte** : la revue de code P1 (Sonnet ×3) rend 0 au-dessus de LOW et 20 LOW. La consigne de
  clôture borne la remédiation aux lignes non exécutables (tests, doc-comments, commentaires,
  `.ftl`, fiche, manuel). Quatre LOW ne se corrigent qu'en touchant du code de production :
  B-1 (« le seul compte lié » faux quand plusieurs comptes bancaires sont écartés — le texte vit
  aussi dans les replis `i18nMsg` de `SettleInvoiceDialog.svelte` et de la fiche fournisseur, et
  dans deux tests qui le comparent au repli) ; B-2 = E2 (remède du message HTTP quand la
  contrepartie vient d'un compte bancaire : il faudrait porter `bankAccountId` jusqu'au mapping, ou
  retoucher le repli Rust `format!` avec la clé) ; B-4 (apostrophe droite de
  `payment-batches-failed-purchase-entry-malformed`, dont le repli est dans
  `payment-batch-helpers.ts`) ; B-6 (DRY : deux `match` d'extraction des champs du refus).
- **Retenu** : les quatre écrits comme dette au Change Log de la fiche, sans modifier le seul
  catalogue — corriger le `.ftl` sans le repli ferait diverger le texte affiché selon que la clé
  est chargée ou non, défaut pire que celui qu'on corrige. B-4 est de plus discutable sur le fond :
  le catalogue `fr-CH` écrit l'apostrophe droite sur 316 lignes et la typographique sur 70 ; la
  clé neuve suit la majorité. Le doc-comment de `claim_account_refusal_details` dit désormais que
  seules les clés sont construites une fois, pas l'extraction (B-6).
- **Écarté** : corriger les replis malgré la consigne (elle est explicite) ; corriger le `.ftl`
  seul (divergence repli/catalogue).
- **Réversibilité** : totale ; la dette se solde dans une story de rattrapage i18n ou dans la
  15-6c, qui retouche les écrans de liaison bancaire.

## C124 — 15-1a découpée en 15-1a-i (la marque) et 15-1a-ii (les gardes) à la validation P3, selon la couture écrite à C118
- **Contexte** : validation P3 de la 15-1a (Opus ×2 ; `target/gate-logs/15-1a-p3-{R,F}.md`). R3-1 (= F3-1),
  R3-2 et R3-3 naissent de correctifs de P2 sur des règles métier (C113, C114, et le test que C114 a
  ajouté) : le déclencheur écrit à C118 (« défaut né d'un correctif de P2 sur une règle métier — R7, AC4,
  AC5, AC8 —, découper avant toute P4 ») est atteint au sens littéral ; c'est le signal D5 de **recyclage**
  (le défaut naît du correctif), non la découverte de défauts d'origine neufs. **Décision de
  l'orchestrateur** : découper selon la couture de C118, avant toute P4.
- **Retenu** : **15-1a-i — la marque** (schéma et bump, code, primitive et règle des périodes, routes,
  audit, exposition et export, i18n des refus du lettrage, documentation de l'API et du manuel propre au
  lettrage) ; **15-1a-ii — les gardes** (gel des écritures lettrées `ENTRY_LETTERED` sur les trois chemins,
  écran de la fiche d'écriture, lettrage `reversal` de la contre-passation, réserves du manuel et de
  l'API sur la modification, entrées #532 du CHANGELOG). `15-1a-socle-lettrage.md` devient l'index
  (statut `split`, corps vidé, table de correspondance, historique). **Numérotation conservée** : R1–R7,
  AC1–AC15 et T0–T12 gardent leur numéro dans la sous-fiche qui les porte — les fiches sœurs (« 15-1a R5 »,
  « 15-1a AC5 ») restent justes sans réécriture ; un AC partagé est écrit « AC15 (part i) / (part ii) ».
  **Dépendance résiduelle, écrite** : la 15-1a-ii suppose la 15-1a-i mergée (colonnes, primitive, mode
  `System`, `JournalEntryLineResponse`) ; entre les deux merges, une écriture lettrée par l'API reste
  modifiable et supprimable (le gel est dans la seconde) — d'où : la 15-1a-ii suit la 15-1a-i
  **immédiatement**, et **la v0.13.0 ne se tague pas entre les deux** (même règle de publication que la
  15-12a pour la 15-12b). Ordre : 15-12a → 15-12b → **15-1a-i → 15-1a-ii** → 15-1a2 → 15-1b → 15-1c.
- **Écartées** : ne pas découper et écrire pourquoi (le déclencheur était écrit d'avance ; le contourner
  aurait fait de C118 une règle contournée dès sa première occasion) ; placer les gardes backend d'AC8 dans
  la 15-1a-i pour supprimer la fenêtre (ce n'est plus la couture de C118, et la 15-1a-i franchirait de
  nouveau le premier critère) ; renuméroter les AC par sous-fiche (casse les renvois des fiches sœurs et du
  registre) ; nommer les sous-fiches `15-1a1`/`15-1a3` (collision de lecture avec la 15-1a2, qui est une
  autre story).
- **Réversible** : oui (fiches seulement) — refusionner revient à concaténer les deux corps, la table de
  correspondance de l'index disant où chaque élément est allé.

## C125 — 15-1a-i : exercices verrouillés un par un par clé primaire, bornés à ceux du groupe ; « postérieur clos » lu sans verrou (révise C114 ; aligne sur C119)
- **Contexte** : R3-2 (MEDIUM) — le parcours `start_date >= ? ORDER BY start_date ASC FOR UPDATE` de C114
  verrouillait **tous** les exercices postérieurs au plus ancien du groupe, dont celui du jour : il ne
  gardait aucune transition (sous la 15-12a, aucun postérieur ne se clôt tant qu'un antérieur est ouvert)
  et coûtait une sérialisation avec toute écriture de l'exercice du jour, un cycle neuf avec la
  contre-passation d'écritures sans rapport, et un verrou de fin d'intervalle sur la société suivante (R L10).
  Et C119 (15-12a) a établi qu'un parcours d'intervalle ne fixe pas l'ordre d'acquisition (il dépend du
  plan ; `opening_complement.rs:278-281` l'a mesuré). **Décision de l'orchestrateur** : la forme de C119
  devient la forme principale (le « repli » de C114), parcours et `EXPLAIN` abandonnés ; verrous bornés
  aux exercices du groupe ; « postérieur clos » par `find_later_closed`, non verrouillant, à vérifier.
- **Retenu** : (a) `SELECT id, start_date, name FROM fiscal_years WHERE id IN (<exercices des lignes>) AND
  company_id = ? ORDER BY start_date ASC`, sans verrou (`start_date` immuable) ; (b) verrou de **chacun**,
  un par un, dans cet ordre, par une boucle Rust — constante `LOCK_LETTERING_FISCAL_YEAR_SQL` : `SELECT
  id, start_date, status FROM fiscal_years WHERE id = ? AND company_id = ? FOR UPDATE`, `status` lu sous
  verrou ; absent → `Invariant` (clé étrangère `fk_journal_entries_fiscal_year` sans cascade, écritures
  tenues) ; (c) le (ii) de la règle des périodes par `fiscal_years::find_later_closed` (`:678`), sur
  `&mut **tx`, pour chaque exercice ouvert du groupe. **« Ceux du groupe »** = les exercices qui portent
  une ligne du groupe (tous compris entre le plus ancien et le plus récent) ; un exercice intermédiaire
  sans ligne du groupe n'est **pas** verrouillé : aucune de ses lignes ne change, et il n'entre pas dans le
  verdict. **Pourquoi la lecture sans verrou du (ii) suffit** (même preuve que la 15-12b, AC 8 (α)/(β)) :
  (α) sous la 15-12a, aucune transition ne crée « X ouvert, postérieur clos » — la clôture d'un postérieur
  verrouille ses antérieurs (b') et relit sous verrou (d), donc refuse tant qu'un antérieur est ouvert ;
  (β) le lettrage tient X jusqu'à son `COMMIT`, si bien que cette clôture attend X puis le trouve ouvert.
  Une lecture périmée ne peut donc manquer aucune clôture postérieure ; elle ne peut que voir encore clos un
  postérieur rouvert entre-temps (état hérité) : refus à tort, sans dommage. Un exercice créé entre-temps
  naît `Open`. Aucun fantôme ne change le verdict : pas de relecture verrouillante. Remarque écrite : le
  verdict du groupe ne dépend que de son exercice le plus récent (une ligne plus ancienne en période
  ouverte implique qu'une ligne du plus récent l'est). Tests : celui de l'ordre (R3-3, sonde `NOWAIT`) et
  un test du (ii) par lecture non verrouillante (état hérité posé par SQL).
- **Écartées** : garder le parcours de C114 (ordre dépendant du plan, verrous inutiles, fin d'intervalle) ;
  le parcours borné `start_date BETWEEN ? AND ? … FOR UPDATE` proposé par R3-2 (toujours un parcours :
  l'ordre reste au plan — C119) ; verrouiller aussi les exercices intermédiaires (aucune ligne ne change,
  aucun verdict n'en dépend) ; `find_later_closed_in_tx` (verrouillant : réintroduit le verrou d'intervalle
  sur les postérieurs que R3-2 retire) ; ne verrouiller que le plus récent (suffit au verdict, mais
  laisserait le lettrage modifier des lignes d'un exercice ouvert non tenu, contre la règle commune à tous
  les écrivains de lignes et le P8-1 d'août).
- **Réversible** : oui (fiches seulement).

## C126 — 15-1a-ii : `ENTRY_LETTERED` parle en dernier sur les trois chemins (révise le rang de C102 et de C117)
- **Contexte** : R3-1 = F3-1 (MEDIUM) — la garde de lettrage était placée avant le verrou de période
  (`delete_in_tx` 3-ter-bis, `update_in_tx` juste après `modification_guard`, `modification_blocker` avant
  `PeriodLocked`). Depuis C113, délettrer est refusé quand toutes les lignes du groupe sont sous le verrou :
  « délettrez-la d'abord » envoyait alors vers un geste refusé (boucle de refus), ou vers un délettrage qui
  réussit puis un `PERIOD_LOCKED` de toute façon (un lettrage juste détruit pour rien). C117 avait écarté
  pour la même raison « 3-ter-bis avant 2-bis » sans l'appliquer au verrou de période.
- **Retenu** : la marque parle **après tout refus que le délettrage ne peut lever** : `delete_in_tx`
  étape **3-quinquies**, après le verrou de période (3-quater), toujours hors du drapeau ;
  `update_in_tx` étape **7-bis**, après le verrou de période sur l'ancienne et la nouvelle date, avant
  l'instantané de l'étape 8 (les refus du corps et `OPTIMISTIC_LOCK_CONFLICT` parlent donc avant elle) ;
  `modification_blocker` : après `PeriodLocked`, dernier motif. **Conséquence écrite** : quand
  `ENTRY_LETTERED` parle sur la route, l'écriture est en période ouverte (exercice ouvert, aucun
  postérieur clos, date après la borne) et n'appartient à aucune pièce — son groupe est `manual` (R5) et
  sa propre ligne satisfait la règle des périodes : le délettrage qu'il prescrit **aboutit**. Les doc-
  comments « le verrou de période parle en dernier » de `delete_in_tx` et du `PUT` sont réécrits ; la
  précédence de C117 (2-bis avant la marque) tient, l'étape s'appelant désormais 3-quinquies ; la chaîne
  de `unvalidate` (15-12b AC 10) finit par `… → PERIOD_LOCKED → ENTRY_LETTERED`. Tests par paire sur
  chaque chemin (écriture lettrée datée ≤ borne → `PERIOD_LOCKED` ; datée après → `ENTRY_LETTERED`),
  mutation « permuter » nommée.
- **Écartées** : garder le rang et conditionner la marque à « date > borne » (deux lectures de la borne,
  et la règle « tout refus non levable d'abord » ne serait tenue que pour un refus) ; placer la marque
  avant `OPTIMISTIC_LOCK_CONFLICT` (un conflit de version ne se lève pas en délettrant) ; écrire pourquoi la
  marque devrait parler d'abord (aucun motif : elle n'indique pas où corriger, contrairement à une pièce,
  `api-external.md:279`).
- **Réversible** : oui (fiches seulement).

## C127 — 15-1a-i / 15-1a-ii : trois tranchages de forme de la validation P3 (compte bancaire archivé, exercice des lignes, rubriques du CHANGELOG)
- **Contexte** : F3-7 (LOW) — R4 ne disait pas si un compte bancaire archivé laisse son compte non
  lettrable ; F3-8 (LOW) — `entryNumber` repart à 1 à chaque exercice, ambigu dans un groupe à cheval
  (réponse et audit) ; F3-2 (MEDIUM) — le CHANGELOG `[0.13.0]` énumère déjà les refus du `PUT`/`DELETE`
  (#532) et ne disait rien des changements de contrat du lettrage.
- **Retenu** : (1) **tout** `bank_accounts` qui désigne le compte, archivé compris, le rend non lettrable
  (il a été un compte bancaire ; ses lignes relèvent de la réconciliation) — même fonction pour la 15-1b ;
  test ; (2) chaque ligne de la réponse des routes de lettrage et des `details` d'audit porte
  `fiscalYearId` et `fiscalYearName` ; (3) CHANGELOG, patron C123 : la section *Ajouté* existante est
  **complétée** (créée seulement si absente) ; sous *Modifié*, les changements de contrat — champs neufs
  des lignes et deux colonnes CSV (15-1a-i), `ENTRY_LETTERED` au `PUT`/`DELETE` et `201` de la
  contre-passation portant des lignes lettrées (15-1a-ii) — ; les deux entrées #532 réécrites par la
  15-1a-ii (refus énuméré à son rang, dernier ; « écriture lettrée » parmi ce qui ne se modifie pas tel
  quel).
- **Écartées** : (1) seuls les comptes bancaires non archivés (un compte redeviendrait lettrable à
  l'archivage, ses lignes de banque mêlées au lettrage) ; (2) le seul `entryId` (l'audit se lit sans
  recouper) ou le nom seul (renommable) ; (3) tout sous *Ajouté* (une intégration cherche sous *Modifié*).
- **Réversible** : oui.

## C128 — 15-1a-i / 15-1a-ii : en mode `System`, le nom des exercices se lit sans verrou (validation P4)
- **Contexte** : R4-2 (15-1a-i) = R4-1 (15-1a-ii), MEDIUM. C127 exige `fiscalYearId` **et**
  `fiscalYearName` par ligne dans la réponse et l'audit, toutes origines ; mais R7 réservait la lecture
  des exercices (point 2 (a), qui porte `name`) au mode `Manual`, et disait la primitive en mode
  `System` « sans requête ». Le lettrage `reversal` (15-1a-ii) et les groupes `document` (15-1a2)
  n'avaient donc aucune source pour le nom.
- **Retenu** (décision de l'orchestrateur) : en mode `System`, après l'acte 1, une lecture **ordinaire,
  non verrouillante** — `LETTERING_FISCAL_YEAR_NAMES_SQL` : `SELECT id, name FROM fiscal_years WHERE
  company_id = ? AND id IN (…)` — pour le seul affichage. Le « sans requête » ne vaut plus que pour le
  contrôle de l'exercice tenu. Écrit en R7 point 3, AC6, AC10, T3 (15-1a-i), au « Coût » de R6 et à
  AC10 part ii (15-1a-ii), reporté à la 15-1a2 (Reçu, point 18).
- **Écartées** : une jointure de `fiscal_years` dans l'acte 1 (sous `FOR UPDATE`, elle verrouillerait les
  exercices hors de l'ordre de la clôture ; MariaDB n'a pas de `FOR UPDATE OF`) ; un nom absent en mode
  `System` (le test `reversal_lettering_is_audited_by_the_reverser` l'attend, et l'audit doit se lire
  sans recouper) ; une lecture verrouillante (inverserait l'ordre `start_date` derrière l'exercice du
  jour).
- **Réversible** : oui (une requête de lecture).

## C129 — 15-1a-ii : la contre-passation marque l'origine ; « intacte » veut dire montants, comptes, dates et libellés (validation P4)
- **Contexte** : F-3 (MEDIUM, 15-1a-ii). R6 écrit `lettering_key`/`lettering_origin` sur les lignes de
  l'**origine** ; la doctrine « crée une écriture, n'en modifie aucune — l'origine reste intacte » est
  écrite au doc-comment de la route (`routes/journal_entries.rs:477`), au texte du dialogue de
  confirmation (quatre locales, repli Svelte), et un test (`…_leaves_the_origin_intact`) resterait vert
  en le disant, sur un compte lettrable.
- **Retenu** (décision de l'orchestrateur) : l'origine reste **intacte dans ses montants, comptes, dates
  et libellés** ; seules ses lignes reçoivent la marque de lettrage qui les apparie à la
  contre-passation (sans bump de `version`). Doc-comment, clé `journal-entries-reverse-dialog-body`
  (quatre locales, texte arrêté à R6) et repli réécrits ; test renommé
  `reverse_creates_the_opposite_entry_and_marks_the_origin_without_altering_it`, qui asserte montants,
  comptes et `version` inchangés **et** la marque posée ; mutations nommées. Publié (`CHANGELOG.md:192`,
  `[0.12.0]`) : non réécrit. Avoirs (`user-manual.tex:1230/1235`) : portés à la 15-1a2 (point 20).
- **Écartées** : ne pas marquer l'origine (le groupe `{L, L'}` exige ses deux lignes) ; bumper la
  `version` de l'origine (ferait échouer un `PUT` concurrent sans motif, et l'origine n'est de toute façon
  plus modifiable — `ENTRY_IS_REVERSED`) ; laisser le texte tel quel (vrai au sens comptable, faux à la
  lettre, et lu à chaque contre-passation).
- **Réversible** : oui (textes et test).

## C130 — 15-1a-i : la promesse « paiement à lettrer » quitte l'écran aussi, dans cette story (validation P4)
- **Contexte** : R4-1 (MEDIUM, 15-1a-i). C106 retirait la promesse d'un lettrage manuel du règlement
  d'une facture créditée (que R5 interdit), mais n'inventoriait que le manuel et `api-external.md`. Le
  même texte est affiché à l'écran : deux clés i18n × quatre locales, replis Rust (`errors.rs:2966`,
  `:3495`) et frontend, quatre tests qui l'assertent, un doc-comment (`kesh-db/src/errors.rs:327`).
- **Retenu** (décision de l'orchestrateur, emplacement tranché ici) : **dans la 15-1a-i**, là où vit
  R5 — sinon l'écran dirait de lettrer ce que l'API refuse dès son merge. Texte : « … ce règlement **reste
  ouvert au compte débiteurs**, il ne s'annule pas » (un constat, sans geste promis), arrêté en
  FR/DE/EN/IT à AC15 part i ; contrôle final par `git grep` par la valeur dans les quatre langues.
  `CHANGELOG.md:74` est sous `[0.12.1]` **publié** : non réécrit (des notes de version publiées ne se
  corrigent pas après coup), le changement est annoncé sous *Modifié* de `[0.13.0]`.
  `supplier-invoices-cancel-confirm-paid` (« to be matched », DE/EN/IT) parle du paiement fournisseur
  **détaché**, qui reste lettrable à la main : promesse vraie, non touchée.
- **Écartées** : renvoyer à la 15-1a2 (l'écran mentirait entre les deux merges) ; « il se traite
  ailleurs » ou une promesse de traitement futur (la 15-1a2 n'a pas tranché le cas) ; réécrire
  `CHANGELOG.md:74` (l'orchestrateur le listait ; écarté pour la raison dite, à son arbitrage).
- **Réversible** : oui (textes).

## C131 — 15-1a-i / 15-1a-ii : tranchages de forme de la validation P4
- **Contexte** : LOW des deux lentilles, sur des points que la fiche laissait au développeur.
- **Retenu** : (1) `key_from_code` : `to_ascii_uppercase` **puis** validation `A-Z` (F4-2 : `to_uppercase`
  ferait de `ß` un `SS`) ; (2) champs de lettrage du type frontend **requis et nullables**, fixtures
  typées nommées (R4-3 = F4-3) ; (3) **`LETTERING_CONCURRENT_CHANGE` testé par une fonction pure**
  `check_rows_affected` et par le mapping 409, le chemin de bout en bout étant un angle mort assumé
  (F4-4). ⚠️ **Écart à la consigne de l'orchestrateur** (« déclencheur SQL posé par le test ») :
  `sqlx-mysql` 0.8.6 pose `CLIENT_FOUND_ROWS` (`connection/stream.rs:46`), si bien que
  `rows_affected()` compte les lignes **trouvées** ; un déclencheur `BEFORE UPDATE` ne change pas ce
  nombre (il ne peut ni écarter une ligne trouvée ni écrire dans sa propre table — erreur 1442), donc
  ne provoque pas le refus. (4) Test d'ordre : `test_fixtures::sonde_verrou_nowait` réutilisé, `1205`
  déjà mesuré (F4-5), requête bloquée observée deux fois à 100 ms (R4-5). (5) AC9 de la 15-1a-ii : noms
  `snake_case` fixés (R4-8). (6) Contre-passation dans `api-external.md` : une phrase à `:255`, pas de
  section (F-7). (7) Message `ENTRY_LETTERED` inchangé malgré l'absence d'écran de délettrage avant la
  15-1c : fenêtre jamais publiée, l'epic sortant en une release (R4-9 = F-8).
- **Écartées** : (2) champs optionnels (mentirait sur le contrat) ; (3) un crochet de test en code de
  production ; (6) une section complète de la route (hors périmètre) ; (7) un message qui renvoie à
  l'API (devient faux à la 15-1c).
- **Réversible** : oui.

## C132 — 15-1a-ii : la marque de lettrage est nommée, non comptée, parmi les refus de la dévalidation (validation P5)
- **Contexte** : R5-7 = L-7 (LOW). La remédiation de P4 (R4-2, F-7) faisait passer de « trois » à
  « quatre » les doc-comments qui comptent les gardes de `delete_in_tx` tenues hors du drapeau
  (`invoices.rs:1471-1473`, `kesh-db/src/errors.rs:241-244`), alors que les totaux écrits ailleurs —
  « huit » à `invoices.rs:1376`, `invoices/[id]/+page.svelte:355`, `admin-manual.tex:1919` et `:1961`,
  et le tableau d'`api-external.md:288-299` — restaient à huit : neuf d'un côté, huit de l'autre. Le
  motif est **inatteignable** par la dévalidation (une facture lettrée a un règlement ou un avoir, que
  `unvalidate` refuse avant).
- **Retenu** : les totaux comptent les refus que la dévalidation **peut rendre** et restent « huit » ;
  les deux doc-comments gardent « trois » et **nomment** la marque (« plus la marque de lettrage, tenue
  au même point de passage mais inatteignable par la dévalidation, qui parle en dernier »). Aucune règle
  ne change : la garde reste inconditionnelle dans `delete_in_tx` (AC8).
- **Écartées** : compter partout (« neuf », en le disant inatteignable) — six sites à réécrire, dont deux
  du manuel d'administration et un tableau d'API, pour annoncer à l'utilisateur un refus qu'il ne peut
  pas rencontrer ; laisser « quatre » d'un côté et « huit » de l'autre (incohérent, le défaut signalé).
- **Réversible** : oui (texte de doc-comments).

## C-15-12b-1 — 15-12b (T0) : le message de `LATER_FISCAL_YEAR_CLOSED` élargi à la saisie, la contre-passation dite « d'une écriture existante »
- **Contexte** : la 15-12a avait borné `error-later-fiscal-year-closed` (4 locales + repli Rust) à
  « modifiée ni supprimée » (C-15-12a-3) ; le filet de cette story refuse aussi la **création**. Le reçu
  du 2026-10-09 demande d'élargir le texte et d'inverser l'assertion « enregistr ».
- **Retenu** : « … aucune écriture datée avant sa date de début ne peut être **enregistrée, modifiée ni
  supprimée** tant qu'il l'est. **Une écriture existante** se corrige par une contre-passation ; sinon,
  un administrateur rouvre les exercices clôturés, en commençant par le plus récent. » — « Une telle
  écriture » devient « Une écriture existante » : pour un refus de saisie, il n'y a rien à corriger, et
  le conseil ne vaut que pour une écriture déjà passée. Même retouche dans les trois autres locales.
  L'assertion de `journal_entry_reversal_e2e.rs` est **inversée** (elle exige désormais « ne peut être
  enregistrée, modifiée ni supprimée »), et le test neuf `filet_bilan_clos_e2e.rs` l'exige sur le refus
  de la saisie et de la contre-passation.
- **Écartées** : un second message propre à la création (deux clés pour un même code et un même état —
  le message neutre de la 15-12a est fait pour ne pas présupposer le geste) ; ajouter au message la
  marche à suivre du bandeau (« clôturez d'abord l'exercice ouvert le plus ancien ») — le message est
  déjà long et la 15-12a l'a validé tel quel ; le bandeau la porte.
- **Réversible** : oui (texte).

## C-15-12b-2 — 15-12b (T0) : `GET /opening-balances/status` ne prédit pas le filet — angle mort assumé, écrit et fixé
- **Contexte** : l'inventaire des prédicteurs de l'AC 18 (P3, C120) part des champs d'annulation
  (`cancelBlockedBy`…). Rejoué par le symptôme (« un champ d'API qui annonce un geste »), il trouve un
  sixième prédicteur : `canComplete` / `completeReason` (et `canEnter`) de l'écran des soldes de départ
  (`opening_complement::complement_status`), qui ne lit pas les exercices postérieurs. Dans l'état hérité,
  l'écran annonce le complément possible et le `POST` rend `400 LATER_FISCAL_YEAR_CLOSED`.
- **Retenu** : même traitement que C120 — **angle mort assumé, écrit** au doc-comment de
  `complement_status`, au Dev Agent Record et dans `api-external.md` (section des soldes de départ), et
  **fixé** par le test `le_complement_sous_un_exercice_posterieur_clos_est_refuse`
  (`opening_complement_repository.rs`), qui asserte `refusal = None` **et** le refus du `POST` : il
  rougira le jour où le statut le prédira. Le refus au clic est exact et son message porte la marche à
  suivre. Un module de premier niveau de plus au décompte de la dérogation (11 au lieu de 10 : un
  doc-comment).
- **Écartées** : un motif neuf `LATER_FISCAL_YEAR_CLOSED` dans `OpeningComplementRefusal` (variante,
  mapping de la route, `switch` de l'écran, clés ×4 — le coût que C120 a refusé pour les annulations, sur
  un état hérité étroit) ; ne rien écrire (le défaut que la règle « inventorier les sites non résolus »
  interdit). **À signaler à l'orchestrateur** : l'issue #568 pourrait couvrir aussi ce prédicteur.
- **Réversible** : oui.

## C-15-12b-3 — 15-12b (T2) : le libellé du lot a deux clés, avec et sans nom
- **Contexte** : AC 11 (C100) — clé neuve `reconciliation-failed-later-fiscal-year-closed`
  (« Exercice postérieur « { $name } » clôturé »), `$name` lu dans `details.fiscalYearName`, « repli sans
  nom si le champ manque ».
- **Retenu** : le repli sans nom est une **seconde clé**, `reconciliation-failed-later-fiscal-year-closed-generic`
  — patron d'`ACCOUNT_NOT_POSTABLE` / `-generic` du même fichier : un message Fluent dont la variable
  manque rendrait les marques d'isolation ou un nom vide. Texte complété d'une demi-phrase qui dit la
  conséquence : « Exercice postérieur « { $name } » clôturé : aucune écriture ne peut être datée avant
  lui. » La lecture de `details` passe par une aide pure `fiscalYearName` (même prudence que
  `rejectedAccountNumbers`).
- **Écartées** : une seule clé avec `$name` vide (affiche « « » ») ; renvoyer le code brut (le repli
  « Refus non reconnu ») quand le nom manque.
- **Réversible** : oui.

## C-15-12b-4 — 15-12b (revue de code P1) : seize LOW traités sans toucher une ligne de production Rust ni un catalogue
- **Contexte** : revue de code P1 (Sonnet ×3, lentilles B / E / A ; rapports
  `target/gate-logs/15-12b-review-p1-{B,E,A}.md`) — 0 au-dessus de LOW, 16 LOW (B 5, E 4, A 7). Consigne
  de l'orchestrateur : remédier sans code exécutable de production (ni repli Rust, ni catalogue de
  messages), le reste écrit.
- **Retenu** :
  - **B-4** — le bandeau passe de `role="alert"` à `role="status"` : un attribut ARIA du gabarit, qui ne
    change aucun comportement (aucune logique Svelte touchée) ; une information d'état rendue au
    chargement n'a pas à être annoncée de façon assertive à chaque visite. Test mis à jour, AC 15 annoté.
  - **B-1 = E-1** — le message (« Une écriture existante se corrige par une contre-passation ») renvoie à un
    geste que le filet peut refuser à son tour : **message non réécrit**, suivi par l'issue #569 (messages
    de réparation) — écrit au Change Log.
  - **A-2, E-3** — deux tests neufs, sans code de production : la branche `projectId` du constructeur du
    lot (test unitaire dans `period_lock_tests`), et le repli `default` du formulaire de saisie en création
    sur `LATER_FISCAL_YEAR_CLOSED` (`JournalEntryForm.create.test.ts`, mutation du `default` observée
    rouge). Le `case` explicite qu'E-3 proposait aurait touché le composant : écarté, le test écrit
    l'intention.
  - **A-4** — **rectification de C-15-12b-2**, sans réécrire l'entrée : l'angle mort du statut des soldes
    de départ n'est pas dans une « section des soldes de départ » d'`api-external.md` — il n'y en a pas —,
    mais dans la ligne `LATER_FISCAL_YEAR_CLOSED` du tableau des erreurs (`:504`). Aucune section ajoutée :
    la route `GET /opening-balances/status` n'est pas documentée dans ce fichier.
  - **E-2** (apostrophe droite du repli Rust contre typographique des catalogues, préexistant) et **E-4**
    (« avant lui » du libellé de lot) : écrits, non traités — ils toucheraient le repli et les catalogues.
- **Écartées** : réécrire le message (B-1) dans cette story — hors consigne, et l'issue #569 couvre la
  famille ; ajouter le `case 'LATER_FISCAL_YEAR_CLOSED'` au `switch` (E-3) — code de composant pour un
  comportement déjà correct.

## C-15-13-1 — 15-13 (spécification) : le port de MariaDB n'est plus publié du tout, pas même en loopback

- **Contexte** : `docker-compose.yml:13` publie `"3306:3306"` sur toutes les interfaces (#551). `kesh-api` atteint MariaDB par le réseau interne de Compose ; aucune commande du manuel n'emploie le port publié (toutes passent par `docker compose exec`). Un port publié par Docker contourne UFW, que le manuel recommande (`admin-manual.tex:1998-2009`).
- **Retenu** : aucune clé `ports:` au service `mariadb` ; un commentaire donne la forme loopback (`127.0.0.1:3306:3306`) à décommenter pour un dépannage ponctuel ; le test `configuration_transmise.rs` rougit sur toute clé `ports:` de `mariadb`.
- **Écartées** : `"127.0.0.1:3306:3306"` actif (inutile au fonctionnement, et une ouverture que personne n'a demandée) ; garder `3306:3306` en avertissant (l'issue est un P2 de sécurité).
- **Réversible** : oui (une ligne à décommenter).

## C-15-13-2 — 15-13 (spécification) : les mots de passe MariaDB sont exigés par Compose (`${VAR:?message}`) ; Kesh avertit, sans refuser, sur un mot de passe publié

- **Contexte** : défauts `kesh_dev_root` / `kesh_dev` dans `docker-compose.yml` (`:8`, `:11`, `:53`) et lignes **actives** de `.env.example` (`:336`, `:339`). Kesh ne voit pas le mot de passe root (non transmis, C71) ; refuser `kesh_dev` dans Kesh casserait `docker-compose.dev.yml`, le montage E2E et toutes les recettes de test, qui l'emploient ; `KESH_TEST_MODE` ne distingue pas (`TestModeWithPublicBind` interdit le mode test sur la pile de dev).
- **Retenu** : `${MARIADB_ROOT_PASSWORD:?…}` et `${MARIADB_PASSWORD:?…}` (y compris dans la `DATABASE_URL` composée) — Compose refuse de démarrer en nommant la variable ; lignes du gabarit commentées et sans valeur ; avertissement Kesh au démarrage si le mot de passe de `DATABASE_URL` est `kesh_dev` ou `kesh_dev_root` ; indice explicite sur l'erreur 1045 (le piège de la mise à jour : MariaDB ne lit `MARIADB_PASSWORD` qu'à la création de la base) ; CI qui exige le refus sans les variables. Le manuel écrit les anciennes valeurs (pour qu'une installation qui s'y appuyait redémarre à l'identique avant de les changer) ; les fichiers distribués n'en portent plus aucune trace.
- **Écartées** : refus Kesh de `kesh_dev` (casse la pile de dev et la CI) ; placeholder `<GENERATE_ME>` actif pour les mots de passe MariaDB (la base s'initialiserait avec le gabarit, et `<`, espace, `:` cassent la `DATABASE_URL` composée) ; message de refus qui inviterait à « choisir » un mot de passe (une valeur neuve dans `.env` sur une base déjà créée = panne 1045).
- **Réversible** : oui.

## C-15-13-3 — 15-13 (spécification) : sauvegarde pré-import par défaut dans `/data/backup`, montée en dur sur `./backup` par les deux compose

- **Contexte** : `KESH_ADMIN_BACKUP_DIR` vaut `/tmp` par défaut (`config.rs:947`), transmise en `${…:-}` sans aucun montage (#552). Le gabarit suggère déjà `#KESH_ADMIN_BACKUP_DIR=/data/backup`, que rien ne monte.
- **Retenu** : défaut du code `/data/backup` (constante nommée), comme `/data/documents` et `/data/inbox` ; montage fixe `./backup:/data/backup` dans `docker-compose.yml` et `docker-compose.prod.yml` ; transmission `${KESH_ADMIN_BACKUP_DIR:-}` inchangée (C75, liste `AJOUTS` intacte) ; `MONTAGES` du test étendu, égalité exigée pour une source fixe. Singulier, pour que l'exploitant qui a décommenté la suggestion du gabarit soit couvert sans rien changer.
- **Écartées** : défaut de déploiement dans le compose (`${KESH_ADMIN_BACKUP_DIR:-/data/backup}`, contraire à C75 et sans effet pour un compose tiers) ; ranger les sauvegardes sous `/data/documents` ou `/var/log/kesh` (dossiers que le manuel invite à partager en SMB, et le fichier est un secret) ; variable d'hôte `KESH_BACKUP_HOST_DIR` (relève de #558) ; `/data/backups` au pluriel (laisserait dans le conteneur l'exploitant qui a suivi le gabarit).
- **Réversible** : oui (aucune donnée déplacée : l'ancien emplacement mourait avec le conteneur).

## C-15-13-4 — 15-13 (spécification) : la sauvegarde pré-import est écrite en `0600`, sans écrasement, dossier créé en `0700`

- **Contexte** : rendre la sauvegarde persistante sur l'hôte change sa durée de vie ; le `.keshbackup` contient condensés de mots de passe et jetons de session ; le dossier d'hôte créé par Docker est `root:root` `0755`, et `tokio::fs::write` crée le fichier selon l'umask (`0644`).
- **Retenu** : `OpenOptions` avec `create_new(true)` et `mode(0o600)` (unix), `sync_all` avant le journal ; `DirBuilder` récursif en `0o700` quand Kesh crée le dossier ; un dossier existant n'est pas modifié. Test avec assertion de montage sur l'umask.
- **Écartées** : chiffrer la sauvegarde (changement de format, hors périmètre) ; `chmod` du dossier de l'hôte par Kesh (il appartient à l'exploitant).
- **Réversible** : oui.

## C-15-13-5 — 15-13 (spécification) : périmètre du manuel — `openssl rand -hex 32` pour MariaDB, `exec db` corrigé, section Synology laissée à une issue

- **Contexte** : le manuel fait générer le mot de passe MariaDB de Synology en `openssl rand -base64 32` (`admin-manual.tex:546`), repris dans `DATABASE_URL` — `/` y apparaît une fois sur deux et casse l'URL. Trois commandes visent un service `db` qui n'existe pas (`:1460`, `:1511`, `:2216`) avec un `-p"${MARIADB_ROOT_PASSWORD}"` que rien ne pose. La section Synology (pré-script Hyper Backup `:1578`, aperçu Container Manager `:571`) suppose un service `mariadb` que `docker-compose.prod.yml` n'a pas ; le nom de volume `kesh_db_data` (`:1422`, `:1744`) est faux.
- **Retenu** : traités ici, parce qu'ils sont le sujet même de la story (accès à MariaDB, mot de passe MariaDB) : `hex` partout pour MariaDB ; `exec mariadb` avec le mot de passe lu dans le conteneur. **Non traités**, signalés pour une issue P3 : la section Synology sans service `mariadb`, le nom de volume.
- **Écartées** : réécrire la section Synology (autre sujet, autre installation, risque de déborder) ; laisser `base64` (défaut actif sur l'installation de référence).
- **Réversible** : oui (texte).
- **Rectificatif** (validation P2, 2026-10-09) : l'aperçu Container Manager est à `admin-manual.tex:573`, non `:571`.

## C-15-13-6 — 15-13 (spécification) : l'emplacement de la sauvegarde n'est pas renvoyé par l'API ni affiché à l'écran

- **Contexte** : #552 demande que « le manuel dise où les trouver ». La réponse de l'import ne porte que `backupCreated`.
- **Retenu** : le manuel dit l'emplacement (`./backup` du dossier du compose, nom de fichier) ; ni l'API ni l'écran ne changent.
- **Écartées** : ajouter le nom du fichier à la réponse et l'afficher (frontend, i18n ×4, contrat d'API : trois modules de plus pour une information que l'exploitant trouve au manuel ; un chemin du conteneur affiché à l'écran serait d'ailleurs trompeur — ce n'est pas celui de l'hôte).
- **Réversible** : oui.

## C-15-13-7 — 15-13 (spécification) : une seule story pour #551 et #552, pas de découpage ; ligne du registre et paragraphe de la fiche d'epic

- **Contexte** : règle de découpage préventif (plus de cinq modules). Les deux issues touchent les mêmes fichiers (compose, gabarit, test `configuration_transmise.rs`, manuel § « Passer à la 0.13.0 », CHANGELOG `[0.13.0]`). Le worktree part d'`origin/main` (`de285ea8`), où la fiche d'epic ne porte pas encore le paragraphe « Story 15-13 — à spécifier » écrit sur la branche de planification du dépôt principal.
- **Retenu** : une story, trois modules de code dans un crate (`config`, `main`, `routes/admin`) — même décompte que la 15-11a ; ligne `15-13-mariadb-et-sauvegarde: ready-for-dev` (convention « en validation » de l'epic) ajoutée après la 15-11b ; paragraphe « Story 15-13 » ajouté à la fiche d'epic après celui de la 15-11 — à fusionner par union avec la branche de planification.
- **Écartées** : deux stories (#551 / #552), qui doubleraient les conflits sur les mêmes sections sans réduire le risque.
- **Réversible** : oui (planification).
- **Décompte supersédé** (ajout de la validation P2, 2026-10-09) : « trois modules » ne vaut plus depuis l'absorption de #576 ; le décompte en vigueur est celui de **C-15-13-9** (cinq modules, seuil atteint, non franchi). La décision « pas de découpage » tient sur ce nouveau décompte.

## C-15-13-8 — 15-13 (validation P1) : la sauvegarde pré-import reste en `0600` ; le manuel donne le geste de rapatriement

- **Contexte** : R-2 / F4 de la validation P1. L'import se fait par le navigateur ; un fichier `root` en `0600` sur l'hôte ne se lit ni par File Station, ni par SMB, ni par un `scp` sans `sudo`. La procédure « restaurer depuis la sauvegarde = l'importer par le même écran » échouait au moment où l'on en a besoin. L'affirmation « lisible du seul propriétaire » n'était pas mesurée sur Synology (ACL du dossier partagé).
- **Retenu** (décision de l'orchestrateur) : le mode `0600` reste — la sauvegarde contient toute la comptabilité et ses secrets ; le manuel donne le rapatriement en SSH (`sudo cp` vers un dossier partagé, `sudo chown`, import, suppression de la copie), rejoué sur un conteneur jetable au T0 ; « créé en mode `0600`, propriétaire `root` », avec la limite ACL écrite non mesurée ; le tableau des cas gagne « restaurer après un import raté ».
- **Écartées** : un mode `0640`/`0644` ou un `chown` vers un utilisateur de l'hôte (élargit l'accès à un secret, et l'UID de l'exploitant n'est pas connu du conteneur) ; renvoyer le fichier par l'API (C-15-13-6).
- **Réversible** : oui (mode et texte du manuel).

## C-15-13-9 — 15-13 (validation P1) : #576 absorbée — variante `AdminPreImportBackupFailed`, conversion à l'appel, cinq modules

- **Contexte** : F7 de la validation P1. Avec le défaut `/data/backup`, toute instance lancée hors Docker (montage E2E du dépôt, développement) refuse chaque import, et le message affiché annonçait « un backup automatique a été créé ». La story faisait de ce message faux le cas nominal. L'orchestrateur a décidé que la story ferme #576.
- **Retenu** : une variante neuve (500, code `ADMIN_PRE_IMPORT_BACKUP_FAILED`, clé `error-admin-pre-import-backup-failed`, quatre locales et repli Rust) pour **tout** échec antérieur à l'écriture réussie — y compris transaction et verrou, dont le message mentait aussi ; les deux variantes nées dans `admin_backup` (`check_schema_compat`, `build_keshbackup`) converties **à l'appel** par une fonction pure `avant_sauvegarde`, pour ne pas toucher `admin_backup` ; le texte d'`error-admin-full-import-failed` inchangé (il dit vrai pour les douze sites postérieurs). Les recettes `cargo run` du dépôt posent `KESH_ADMIN_BACKUP_DIR` (`CLAUDE.md` : la seule ligne de commande, `docs/testing.md`). Décompte : cinq modules (`config`, `main`, `routes/admin`, `errors`, catalogues `kesh-i18n`) — seuil atteint, non franchi.
- **Écartées** : modifier `admin_backup/import.rs` et `export.rs` (sixième module, seuil franchi) ; un champ « étape » dans `AdminFullImportFailed` (même texte pour deux vérités différentes, à trier par le client) ; un contrôle du dossier au démarrage (`warn!`) — le message dit la cause au moment où elle compte, et un dossier monté tardivement le ferait mentir ; une story séparée pour #576 (elle naît du changement de défaut de #552, la séparer laisserait la 15-13 livrer un message faux).
- **Réversible** : oui (une variante et une clé).

## C-15-13-10 — 15-13 (validation P1) : le contrôle de fin de mise à jour se connecte à la base avec les anciens mots de passe

- **Contexte** : F3 de la validation P1. La vérification prévue (`docker compose config | grep kesh_dev`) ne lit que `.env` : un mot de passe root neuf écrit sans `ALTER USER` la rend muette alors que la base garde `kesh_dev_root`, et fait échouer la sauvegarde nocturne sans que rien ne rougisse (Kesh n'utilise pas root, le healthcheck passe par le compte `healthcheck`). La phrase « aucun de ces cas ne casse en silence » était fausse.
- **Retenu** (décision de l'orchestrateur) : connexion avec les **anciens** mots de passe publiés → refus attendu ; avec ceux de `.env` → succès ; pour `root` et `kesh`, en TCP (`--protocol=TCP -h 127.0.0.1`) pour que l'authentification par socket de `root@localhost` ne fausse pas la mesure — forme exacte rejouée au T7. Plus : lancer une fois à la main le script de sauvegarde après toute mise à jour ou tout changement de mot de passe. Le tableau des cas gagne la ligne « root neuf sans `ALTER USER` » et la conclusion est réécrite (deux cas silencieux nommés, avec leur geste de détection).
- **Écartées** : garder le `grep` (faux négatif sur le cas même qu'il devait couvrir) ; transmettre `MARIADB_ROOT_PASSWORD` à Kesh pour qu'il vérifie (C71 : pas d'`env_file`, et Kesh n'a pas à connaître root).
- **Réversible** : oui (texte du manuel).

## C-15-13-11 — 15-13 (validation P1) : `init-demo.sh` lit l'identifiant MariaDB dans le conteneur

- **Contexte** : R-3 / F6. `init-demo.sh` (versionné) vise par défaut `kesh-mariadb`, le conteneur de `docker-compose.yml`, avec `kesh_dev` en dur : après la story, il échoue en 1045 sur toute installation conforme ; c'était aussi le dernier outil distribué qui supposait le mot de passe publié. L'inventaire « fermé » l'avait omis.
- **Retenu** : `docker exec -i "$CONTAINER" sh -c 'mariadb -u "$MARIADB_USER" -p"$MARIADB_PASSWORD" "$MARIADB_DATABASE"'` — les deux compose à service MariaDB posent ces variables dans le conteneur ; en-tête qui dit le caractère destructif et l'administrateur de démonstration ; vérifié contre le conteneur jetable du T7 (jamais `kesh-mariadb-dev`, dont la base `kesh` est partagée).
- **Écartées** : le marquer « développement seulement » sur `kesh-mariadb-dev` (il resterait faux pour sa cible par défaut) ; le supprimer (outil documenté par son en-tête, décision de produit hors story).
- **Réversible** : oui.
- **Remplacé** (validation P2, 2026-10-09) : par **C-15-13-13** — le script est retiré du dépôt ; la lecture de l'identifiant dans le conteneur le rendait opérant, sans transaction ni confirmation, sur toute installation conforme.

## C-15-13-12 — 15-13 (validation P1) : forme du message `:?` et scission de l'interpolation obligatoire

- **Contexte** : R-1 / F1 et F2. Le message prescrit contenait `": "`, qui rend les deux compose invalides en YAML (scalaire non cité) ; et le test confondait `${VAR:?}` (refuse le vide) et `${VAR?}` (ne refuse que l'absence), si bien qu'une ligne `MARIADB_PASSWORD=` vide passait.
- **Retenu** : deux formes admises — message sans `": "` ni `" #"`, ou valeur entière entre guillemets doubles —, tranchées au T0 par `docker compose config -q` **avec** les variables posées ; message qui dit d'abord le cas existant, puis le neuf ; `Interpolation::Obligatoire` scindée en `ObligatoireNonVide` / `ObligatoireSiAbsente`, la première exigée (mutations M28, M29) ; contrôle de la `DATABASE_URL` par resserrement du motif de `VALEURS_COMPOSEES` (DRY).
- **Écartées** : guillemets simples (l'apostrophe de « d'administration ») ; un message sans indication pour l'installation existante (il pousserait à écrire une valeur neuve).
- **Réversible** : oui.

## C-15-13-13 — 15-13 (validation P2) : `init-demo.sh` est retiré du dépôt, pas corrigé

- **Contexte** : F2 de la validation P2. La correction de la P1 (C-15-13-11 : identifiant lu dans le conteneur) rendait le script **opérant** sur toute installation conforme ; or il enchaîne des `DELETE` (`init-demo.sh:57-62`) sans transaction ni confirmation, et sur une base peuplée `DELETE FROM companies` échoue sur les 25 clés étrangères `ON DELETE RESTRICT` **après** que `users` et `onboarding_state` ont été vidés. L'orchestrateur a demandé de vérifier d'abord s'il est redondant.
- **Constat** : redondant. La démonstration est semée par l'application (`POST /api/v1/onboarding/seed-demo`, `routes/onboarding.rs:177-190` → `kesh_seed::seed_demo` : plan comptable par les repositories, exercice, `is_demo`, réinitialisation par `kesh_seed::reset_demo`). Le script écrit en SQL brut dix comptes sans rôle, sans exercice, et un administrateur `admin`/`admin123`. `git grep -n init-demo` hors `_bmad-output` ne rend que le fichier lui-même : aucun script, aucune CI, aucune documentation ne l'appelle (dernier commit qui le touche : `b63dc4e1`, Story 7-1).
- **Retenu** : `git rm init-demo.sh` (AC 12 b) ; entrée « Retiré » au CHANGELOG ; le T7 ne le lance plus ; inventaire « résolu par retrait ».
- **Écartées** : le garder en le durcissant (refus si la base porte des données, `START TRANSACTION`, test T7 sur un conteneur peuplé) — maintient un second chemin de démonstration, hors des repositories, que rien n'appelle et qui dérive du schéma à chaque migration ; le garder tel que la P1 l'avait corrigé (risque F2 entier).
- **Réversible** : oui (l'historique git garde le fichier).

## C-15-13-14 — 15-13 (validation P2) : les dossiers montés par défaut sont ignorés de git et du contexte de build, contrôle dérivé de `MONTAGES`

- **Contexte** : F1 de la validation P2. `docker compose up` se lance depuis un clone ; `./backup` y apparaît au premier import et contient un secret. Grepé par la valeur, le symptôme vaut aussi pour `./inbox` et `./documents` (justificatifs) : seul `log/` est dans `.gitignore`, aucun des quatre dans `.dockerignore`.
- **Retenu** : `.gitignore` gagne `/inbox/`, `/documents/`, `/backup/` (ancrés à la racine ; `log/` gardé tel quel) ; `.dockerignore` gagne `log/`, `inbox/`, `documents/`, `backup/`. Un test (`montages_hors_du_depot`, test 19) **dérive** la liste des dossiers des sources de `MONTAGES` (colonne `docker-compose.prod.yml`), si bien qu'un montage ajouté demain est contrôlé sans retouche ; mutations M39, M40. `git ls-files` sous ces dossiers : 0 (aucun contenu masqué).
- **Écartées** : `backup/` seul (laisse le même défaut sur deux dossiers voisins — propagation par le symptôme, `CLAUDE.md`) ; motifs non ancrés (`documents/` masquerait tout sous-dossier homonyme du dépôt).
- **Réversible** : oui.

## C-15-13-15 — 15-13 (validation P2) : `docs/ci.md` — le décompte des jobs est corrigé, la description d'un job `e2e` inexistant est laissée à une issue

- **Contexte** : F6 de la validation P2. `docs/ci.md:9-10` annonce « 4 jobs (`backend`, `frontend`, `e2e`, `docker-build`) » ; `ci.yml` en a trois. Grepé par la valeur, le job `e2e` est décrit aussi à `:23`, `:30`, `:131`, `:164`, `:186`, et le `CLAUDE.md` affirme un smoke E2E en CI que `ci.yml` ne porte pas.
- **Retenu** : la story corrige `:9-10` (à une ligne de `:12`, qu'elle réécrit déjà) et ajoute sous `:10` une phrase de renvoi vers une issue que l'orchestrateur ouvre ; les autres lignes et le `CLAUDE.md` restent (défaut antérieur, hors du sujet de #551/#552/#576, et la recette du `CLAUDE.md` est la seule ligne que la story y touche).
- **Écartées** : réécrire toute la description de la CI (hors sujet, et elle dépend d'une décision — rétablir le job ou retirer sa description) ; ne rien toucher (la story réécrit `:12` à côté d'un décompte qu'elle saurait faux).
- **Réversible** : oui.

## C-15-13-16 — 15-13 (validation P3) : découpage en 15-13a (MariaDB, #551) et 15-13b (sauvegarde, #552 et #576)

- **Contexte** : validation P3 (deux lentilles Opus 5.5) : 5 MEDIUM distincts, dont trois nés de la remédiation P2 (R3-1, R3-2, F-P3-2), après deux nés de la remédiation P1 en P2. Signal D5 de recyclage levé deux passes de suite — la forme qui découpe selon l'amendement D5 de la rétrospective de l'Epic 25.
- **Retenu** (décision de l'orchestrateur) : deux fiches filles selon la couture naturelle — **15-13a** « MariaDB non publiée, mots de passe obligatoires » (`closes #551`, refs #577 ; modules `config`, `main`) et **15-13b** « sauvegarde avant import persistante » (`closes #552`, `closes #576`, refs #558 ; modules `config`, `routes/admin`, `errors`, catalogues `kesh-i18n`) ; `15-13-mariadb-et-sauvegarde.md` devient fiche index (`split`), la version complète restant au commit `8a9bcd27`. Numérotation de la fiche unique conservée dans les filles (AC, tests, mutations, tâches) ; recompte aux deux bornes écrit dans l'index. Ordre suggéré : 15-13a d'abord (#551 P2) ; chaque fille ne réécrit que ce que son changement rend faux, si bien qu'une seule mergée au tag laisse un CHANGELOG et un manuel vrais ; la seconde recompte les gestes de `admin-manual.tex:1793` et de `CHANGELOG.md:46`.
- **Écartées** : une troisième fiche pour #576 (elle naît du défaut `/data/backup` : la séparer de #552 laisserait un message faux dans le cas nominal hors Docker, C-15-13-9) ; une coupe « code / documentation » (chaque moitié ne serait ni livrable ni testable seule) ; continuer sans découper (troisième recyclage probable : les défauts naissent désormais dans les ajouts de la passe précédente).
- **Réversible** : oui (planification ; les deux fiches se refondent par simple concaténation, la numérotation étant commune).

## C-15-13-17 — 15-13a (validation P3) : Kesh avertit aussi pour un mot de passe applicatif resté au gabarit du manuel

- **Contexte** : F-P3-3. Le manuel fait recopier `MARIADB_PASSWORD=<mot de passe utilisateur applicatif>` (`admin-manual.tex:254`) et `DATABASE_URL=mysql://kesh:<MARIADB_PASSWORD>@…` (`:259`) ; recopiées telles quelles, ces chaînes — imprimées dans le PDF distribué — deviennent le mot de passe applicatif. La fiche les disait « hors de portée de Kesh pour la même raison » que root ; c'est faux : Kesh lit `DATABASE_URL`, et `is_template_placeholder` (`config.rs:1405`, 15-11a) existe.
- **Retenu** (décision de l'orchestrateur) : AC 5 a-bis — un `warn!` **distinct** quand le mot de passe décodé satisfait `is_template_placeholder` (réemploi, DRY), renvoi à la procédure de changement (AC 11 g) ; test 7 étendu (formes encodée et non encodée, chevrons intérieurs en témoin), mutation **M42** ; doc-comment du prédicat complété ; tableau des cas et Dev Notes rectifiés. Le **root** gabarit reste un angle mort (Kesh ne voit pas root, C71), tracé par une issue que l'orchestrateur ouvre.
- **Écartées** : refuser le démarrage (même raison que C-15-13-2 : un mot de passe faible n'empêche pas de fonctionner, et le refus casserait des installations existantes sans gain sur root) ; un message commun avec `kesh_dev` (l'exploitant ne saurait pas que la valeur vient du manuel) ; un second prédicat propre aux mots de passe (duplication).
- **Réversible** : oui.

## C-15-13-18 — 15-13a et 15-13b (validation P3) : les gestes « pour qui garde son compose » se comptent par fichier ; le contrôle des placeholders du CHANGELOG suit le manuel

- **Contexte** : F-P3-1 et F-P3-2. `admin-manual.tex:1793` (« Pour qui garde son fichier compose : deux gestes. », puis « Sous `environment:` du service `kesh-api` : ») et `CHANGELOG.md:46` (« décrit les deux gestes ») deviennent faux : la 15-13 ajoute des gestes hors d'`environment:`, différents selon le fichier. `CHANGELOG.md:52` recopie le `docker compose config | grep` « doit rester muette », piège corrigé au manuel en P2 (R2-9) mais non propagé.
- **Retenu** : le titre perd son décompte global ; le paragraphe donne le nombre **par fichier**, recompté par la fille mergée en second (aujourd'hui 2 / 2 ; 15-13a seule 4 / 2 ; 15-13b seule 3 / 3 ; les deux 5 / 3), chaque geste écrit avec son fichier et une ligne de contrôle ; `deux gestes` devient un contrôle négatif du PDF aplati. `CHANGELOG.md:52` reçoit la même forme que le manuel (mots de passe d'abord, `config -q && echo 'compose lisible'`) ou un renvoi au manuel — choix laissé au développement, écrit au Dev Agent Record. Les « Effets sans refus » de la sauvegarde sont conditionnés au compose (0.13.0 : `./backup` ; compose gardé : `/data/backup` du conteneur, éphémère).
- **Écartées** : un nombre unique (faux pour l'un des deux fichiers) ; garder « deux gestes » en ajoutant un troisième paragraphe (le titre resterait faux) ; supprimer la commande du CHANGELOG sans renvoi (perte de l'action requise).
- **Réversible** : oui (texte).

## C-15-13-19 — 15-13b (validation P3) : motifs ancrés dans `.gitignore`, une seule tolérance nommée (`log/`)

- **Contexte** : R3-1. Le test 19 admettait `x/` aussi bien que `/x/` pour toute entrée de `MONTAGES`, alors que C-15-13-14 écarte les motifs non ancrés. Mesuré : `backup/` ignore `frontend/src/routes/(app)/admin/backup/` (versionné : l'écran de la sauvegarde) — tout fichier neuf de cette route serait ignoré en silence ; `/backup/` non.
- **Retenu** : `.gitignore` exige `/x/` ; **seule** tolérance, une constante qui ne compte que `log` : la ligne `log/` existante (`.gitignore:34`) — aucun dossier `log` versionné nulle part (`git ls-files | grep -E '(^|/)log/'` → 0), et l'ancrer ferait réapparaître des `log/` d'outils de dev sous les crates ; un `log/` versionné ajouté demain serait masqué : risque écrit. Mutation **M41** (`/backup/` → `backup/`). `.dockerignore` inchangé dans sa forme (motifs relatifs à la racine du contexte).
- **Écartées** : ancrer aussi `log/` (change un comportement existant hors du sujet de la story) ; tolérer `x/` pour tous (le défaut mesuré).
- **Réversible** : oui.

## C-15-13-20 — 15-13 (validation P3) : rectificatifs de C-15-13-13, C-15-13-14 et C-15-13-15 ; attribution de M5

- **Rectificatifs** (les entrées d'origine ne sont pas réécrites) : **C-15-13-13** — les `DELETE` d'`init-demo.sh` sont à `:70-75` (commentaire `:70`, instructions `:71-75`), non `:57-62` (`check_container()`, `:55-63`) — R3-7. **C-15-13-14** — l'homonyme réel d'un motif non ancré est `frontend/src/routes/(app)/admin/backup/` (versionné), non un hypothétique `documents/` ; la décision d'ancrer est tenue par C-15-13-19 — R3-1. **C-15-13-15** — l'issue à laquelle renvoie `docs/ci.md` existe : **#577** ; la liste des lignes qui décrivent le job `e2e` absent compte aussi `:210` — R3-3/F-P3-4, R3-5.
- **Attribution de M5** (constat du remédiateur) : `VALEURS_COMPOSEES` n'est lu que par `controle_valeur` (`configuration_transmise.rs:592-603`), donc par le test `valeurs` (famille (V), `:1743`) ; la fiche unique rangeait son resserrement sous `transmission` et M5 sous le test 1. Retenu : ligne 3a de la 15-13a (`valeurs`, existant modifié) porte M5 ; le décompte des fonctions passe de 19 à 20 (fonction modifiée jamais comptée). Écarté : faire porter le contrôle de la `DATABASE_URL` par le test neuf `mariadb` (second contrôle sur la même valeur, contraire au DRY de l'AC 10 a).
- **Réversible** : oui.

## C-15-13-21 — 15-13b (validation P4) : la sauvegarde pré-import s'écrit sous `.partial`, puis est renommée

- **Contexte** : F-P4-6. `write_backup_file` créait le fichier au nom final puis écrivait : un client déconnecté (hyper abandonne le futur du handler), un OOM ou un `docker stop` en cours d'écriture laissaient un `kesh-pre-import-….keshbackup` tronqué — désormais **persistant**, de même nom qu'une sauvegarde valide, et refusé en « corrompu » à l'import, au moment même où l'on en a besoin.
- **Retenu** (décision de l'orchestrateur) : écriture dans `<chemin>.partial` du **même dossier** (`create_new`, `0600`), `sync_all`, puis `rename` vers le nom final (atomique sur un même système de fichiers) ; un arrêt pendant l'écriture ne laisse qu'un `.partial`, jamais un fichier nommé comme une sauvegarde. `rename` remplaçant une cible existante sur Unix, le refus d'écraser passe par une vérification du nom final avant le renommage (`try_exists`) ; la fenêtre entre les deux reste ouverte, sans portée pratique (nom unique par construction) — angle mort écrit. Test `write_backup_file_passe_par_un_partiel` (un `.partial` préexistant fait échouer l'appel, le nom final n'apparaît pas) ; mutation **M46** « écrire directement le nom final » ; M26 réécrite (« vérification du nom final retirée »). Le manuel dit qu'un `.partial` est une écriture interrompue et peut être supprimé.
- **Écartées** : écrire au nom final et documenter l'angle mort (le fichier mentirait sur sa nature) ; `hard_link` sans écrasement puis suppression du `.partial` (atomique, mais dépend du système de fichiers du montage — NAS, SMB) ; `renameat2(RENAME_NOREPLACE)` (hors `std`, Linux seulement).
- **Réversible** : oui (une fonction privée).

## C-15-13-22 — 15-13a (validation P4) : le mot de passe de `DATABASE_URL` est décodé par `decode_utf8_lossy`, et sa non-divulgation est gardée par mutation

- **Contexte** : F4 et F6. `percent_decode_str` rend un `PercentDecode` ; `decode_utf8()` renvoie une erreur sur des octets non UTF-8 (`%FF`), qu'un `?` ou un `unwrap` transformerait en échec ou en panique du démarrage — contraire à l'AC 5 b (« aucun `ConfigError` neuf »). Et la propriété « ni le mot de passe ni l'URL dans le journal » n'était gardée par aucune mutation.
- **Retenu** : `decode_utf8_lossy()` (une valeur non UTF-8 n'est ni publiée ni gabarit ; sa forme décodée ne déclenche rien) ; témoin `%FF` au test 7 ; **M43** (l'avertissement interpole le mot de passe décodé, rouge au test 7) et **M44** (le message d'échec de connexion interpole `database_url`, rouge au test 10, qui exige l'absence de `mauvais-15-13` dans la sortie). La forme non encodée du gabarit, établie à la source (`url` 2.5.8), devient un cas positif ferme du test 7 ; le repli conditionnel est retiré.
- **Écartées** : `decode_utf8()` avec gestion explicite de l'erreur (même effet, une branche de plus à tester) ; garder le repli du test 7 (branche morte qui laisse croire à une incertitude levée).
- **Réversible** : oui.

## C-15-13-23 — 15-13a (validation P4) : la recette de changement de mot de passe MariaDB n'expose pas le nouveau mot de passe, et commence par arrêter Kesh ; la rubrique « Retiré » du CHANGELOG est légitime

- **Contexte** : F7. La recette (`ALTER USER`, puis `.env`, puis `up -d`) ne disait pas comment le nouveau mot de passe atteint la requête : tapé en ligne de commande, il resterait dans l'historique du shell et dans la liste des processus de l'hôte, alors que toute la story lit les mots de passe dans le conteneur. Entre l'`ALTER USER kesh` et le `up -d`, les connexions neuves de Kesh échouent en 1045. Annexe : la rubrique « Retiré » n'existe dans aucune version du CHANGELOG.
- **Retenu** : nouveau mot de passe engendré dans une variable (`NEW=$(openssl rand -hex 32)`, jamais tapé), transmis à `mariadb` par l'entrée standard (document en ligne), reporté dans `.env` à l'éditeur ; la recette commence par `docker compose stop kesh-api` ; forme de référence en Dev Notes, forme finale fixée au T7. Rubrique « Retiré » gardée : *Removed* est l'une des six rubriques de Keep a Changelog 1.1.0 (vérifié en ligne le 2026-10-09), que l'en-tête du CHANGELOG déclare suivre en traduisant les intitulés.
- **Écartées** : `sed -i` pour écrire `.env` (le mot de passe passerait un instant dans les arguments de `sed`) ; laisser la forme au seul T7 (l'exigence d'hygiène doit être écrite avant la mesure, sinon la mesure ne la vérifie pas) ; ranger le retrait d'`init-demo.sh` sous « Modifié » (moins exact).
- **Réversible** : oui (texte du manuel et du CHANGELOG).

## C-15-13-24 — 15-13b (validation P4) : le défaut `/data/backup` est lié par test au littéral et au montage ; la comparaison des sources de montage devient une fonction pure

- **Contexte** : R4-1 et F-P4-2. `DEFAULT_ADMIN_BACKUP_DIR` n'était comparée qu'à elle-même : changée en `/data/backups`, elle laissait tous les tests verts et la sauvegarde retournait dans le conteneur ; M15, appliquée à la valeur de la constante, restait verte. La règle d'égalité des sources de montage vivait dans la boucle de `controle_transmission`, que l'auto-test (S) n'atteint pas : M14 n'avait aucun test capable de rougir.
- **Retenu** : constante `pub` ; le test 6 b la compare au **littéral** `"/data/backup"` (le seul littéral voulu côté test), le test `transmission` exige qu'elle soit la cible d'une entrée de `MONTAGES` (donc d'un montage des compose) ; M15 réécrite « valeur de la constante changée en `/tmp` », rouge aux tests 3 et 6 b. Fonction pure `source_conforme(compose, source, attendue_y, exacte_p) -> bool`, appelée par la boucle et exercée par (S) avec `./backups` refusé en `Y` — rouge sous M14.
- **Écartées** : faire porter le lien par le test 19 (il traite d'exclusion git, pas de montage) ; construire des `Service` synthétiques pour atteindre la boucle (plus lourd, même garantie).
- **Réversible** : oui.

## C-15-13-25 — 15-13b (validation P4) : le message de #576 n'oriente pas vers le seul dossier, et sa négation est exigée par test

- **Contexte** : F-P4-5 et F-P4-4. Trois des cinq échecs antérieurs à l'écriture sont des pannes de base (transaction, verrou, lecture du schéma ou des données) ; un texte « dossier de sauvegarde inscriptible ? » oriente vers une fausse piste. Et les tests 15/16 n'exigeaient que l'absence de l'ancienne promesse : un texte « Échec de l'import. » passait, sans plus dire qu'aucune copie n'existe — l'objet même de #576.
- **Retenu** : texte qui nomme les deux pistes à égalité (« dossier de sauvegarde inscriptible, base de données accessible ») ou aucune ; code et variante inchangés. Les tests 15 et 16 exigent la négation (« aucune sauvegarde n'a été créée » et ses trois traductions, à la lettre) ; mutation **M45**.
- **Écartées** : deux variantes et deux textes (dossier / base) — un module de plus à toucher pour un tri que les journaux font déjà ; un code d'erreur neuf pour la base (même raison).
- **Réversible** : oui (quatre catalogues et un repli).

## C-15-13-26 — 15-13 (validation P4) : rectificatifs de C-15-13-9 et de C-15-13-14

- **Rectificatifs** (les entrées d'origine ne sont pas réécrites) : **C-15-13-9** — « cinq modules » ne vaut plus depuis le découpage (C-15-13-16) : la 15-13a en compte deux (`config`, `main`), la 15-13b quatre (`config`, `routes/admin`, `errors`, catalogues `kesh-i18n`) — R4-10 de la validation P4 de la 15-13b. **C-15-13-14** — « un montage ajouté demain est contrôlé sans retouche » n'était vrai que si l'assertion de montage du test 19 ne fixait pas le nombre d'entrées : elle exige désormais « au moins quatre », dont `/data/backup` (R4-5). **Inventaire de la 15-13a** — la ventilation « 29 + 6 → 36 » de la fiche index ne se refaisait pas ; seuls les totaux sont gardés (R4-7).
- **Réversible** : oui.

## C-15-13-27 — 15-13b (validation P5) : le message de #576 nomme toujours les deux pistes — rectifie C-15-13-25 et le motif de C-15-13-9

- **Contexte** : R5-1 de la validation P5, **né de la remédiation P4**. C-15-13-25 admettait un texte qui « nomme les deux pistes à égalité, ou aucune ». Or l'écartement du contrôle du dossier au démarrage (C-15-13-9, angle mort de la 15-13b) repose sur un message qui « dit la cause au moment où elle compte » : avec l'option « aucune », le cas que #576 rend nominal (instance hors Docker sans `KESH_ADMIN_BACKUP_DIR`) rendrait un 500 qui ne mentionne plus le dossier, et l'écartement n'aurait plus de fondement — sans qu'aucun test le voie.
- **Retenu** (décision de l'orchestrateur) : le message d'échec antérieur à la sauvegarde nomme **toujours** les deux pistes — le dossier de sauvegarde (inscriptible ?) **et** la base de données (joignable ?) — dans les quatre locales et le repli Rust, par des jetons fixés à la lettre (`dossier de sauvegarde`/`base de données`, `Sicherungsordner`/`Datenbank`, `backup folder`/`database`, `cartella di backup`/`database`). Les tests 15 (repli) et 16 (catalogues) exigent leur présence ; mutations **M49** (`en-CH` sans « database ») et **M50** (repli sans « base de données »). La liste des refus du manuel nomme les mêmes deux pistes (F-P5-7).
- **Rectificatifs** (entrées d'origine non réécrites) : **C-15-13-25** — « ou aucune » est retiré ; « trois des cinq échecs antérieurs » devient « cinq des **sept** » (R5-2 : `:172`, `:236`, `:240`, `:246`, `:259` pour la base ; `:473`, `:487` pour le dossier). **C-15-13-9** — le motif d'écartement du `warn!` au démarrage se lit désormais : « le message **nomme les deux causes possibles** au moment où elles comptent » ; il tient tant que le message nomme le dossier.
- **Écartées** : garder « ou aucune » et rouvrir le contrôle au démarrage (plus coûteux, et un dossier monté tardivement ferait mentir le `warn!`) ; deux variantes dossier/base (déjà écartées par C-15-13-25).
- **Réversible** : oui (quatre catalogues, un repli, deux assertions).

## C-15-13-28 — 15-13b (validation P5) : le branchement d'`avant_sauvegarde` est gardé par un test lexical

- **Contexte** : F-P5-1 de la validation P5 (d'origine P1, `07e168e1`). Le test 14 exerce la fonction pure `avant_sauvegarde`, mais rien ne vérifiait qu'elle est appliquée aux appels de `check_schema_compat` (`routes/admin.rs:172`) et de `build_keshbackup` (`:259`) : un `.map_err(avant_sauvegarde)` oublié laissait tous les tests verts et rendait le message faux de #576 (ou « l'export n'a pas pu être généré »). Les deux pannes ne sont pas injectables à bon compte.
- **Retenu** : un garde **lexical**, test 20 `avant_sauvegarde_branchee_aux_appels_de_l_import` (`mod tests` de `routes/admin.rs`) — `include_str!("admin.rs")` tronqué au `#[cfg(test)]`, commentaires écartés, chaque appel des deux fonctions rattaché à sa fonction de premier niveau ; dans `full_import` et `run_backup_and_restore`, l'instruction (jusqu'au `;`) contient `.map_err(avant_sauvegarde)` ; dans `full_export` (`:39`), elle ne le contient pas ; assertion de montage : exactement un appel de `check_schema_compat` et deux de `build_keshbackup`. Forme d'appel prescrite à l'AC 15 b (`….await.map_err(avant_sauvegarde)?`). Mutations **M47** (`:172`) et **M48** (`:259`) **couvertes**.
- **Pourquoi c'est fiable** : la forme est unique et prescrite ; toute écriture que le test ne reconnaît pas (fermeture, conversion différée, alias) le fait rougir **à tort**, jamais passer à tort — un faux rouge coûte une ligne, un faux vert est muet. Le précédent du dépôt est le contrôle (L) de `configuration_transmise.rs`, lexical lui aussi. L'appel de l'export est contrôlé dans l'autre sens, pour que le garde ne pousse pas à convertir partout. Reste angle mort : aucune panne effective ne traverse jusqu'à la réponse HTTP.
- **Écartées** : l'angle mort seul, avec mutation déclarée non couverte (le garde est bon marché et ferme la mutation) ; extraire `sauvegarde_pre_import(pool, dir)` et la tester sur une base dégradée (lourd, change la frontière du verrou, et une table supprimée fait d'abord rougir `check_schema_compat` en 400).
- **Réversible** : oui (un test).

## C-15-13-29 — 15-13b (validation P5) : précisions de C-15-13-21 — on ne supprime que ce qu'on a créé, `try_exists` en erreur vaut échec

- **Contexte** : F-P5-3, R5-3/F-P5-8 de la validation P5 (nés de la remédiation P4). C-15-13-21 ne disait pas si l'échec de `create_new` nettoyait le `.partial` (qui n'est alors pas celui de l'appel), ni ce que vaut un `try_exists` en `Err`, ni les détails journalisés des étapes neuves.
- **Retenu** : l'échec de `create_new` ne supprime **rien** (test 12 : le `.partial` préexistant garde son contenu ; mutation **M51**) ; seul le `.partial` créé par l'appel est nettoyé ; `try_exists` en `Err` = **échec** (jamais « absent », qui rouvrirait l'écrasement) ; un détail journalisé par étape, qui nomme le chemin (AC 9 c), non testé à la lettre.
- **Écartées** : nettoyer sur toute erreur (supprimerait le fichier d'un autre écrivain) ; traiter `Err` comme absent.
- **Réversible** : oui.

## C-15-13a-1 — 15-13a (T0) : le manuel dit le refus de Compose tel qu'il est mesuré — `config`, `pull` et `up` refusent, `ps`, `logs`, `exec`, `stop` restent utilisables, et la variable nommée change d'un lancement à l'autre

- **Contexte** : l'AC 11 f affirmait, sous réserve de mesure au T0, que « le refus de Compose frappe toute sous-commande » (`ps`, `logs`, `exec`, `stop`, `down`) et que le script de sauvegarde lancé par `cron` échoue dès que `.env` perd une des deux lignes ; l'AC 11 j, que Compose nomme « la première » variable et que « la seconde apparaît au lancement suivant ». Mesuré le 2026-10-09 (Docker Compose 2.40.3, projet jetable `kesh1513a-t0`, service `mariadb` en marche, `env -u MARIADB_ROOT_PASSWORD -u MARIADB_PASSWORD`) : `config`, `pull`, `up -d` et `up -d kesh-api` sortent en code 1 avec `required variable … is missing a value` ; `ps`, `logs`, `exec -T mariadb true` et `stop` sortent en 0 et font leur travail. Et, sur 12 lancements de la même commande, la variable nommée varie : `MARIADB_ROOT_PASSWORD`, `MARIADB_PASSWORD` ou `DATABASE_URL` (de `kesh-api`, qui nomme `MARIADB_PASSWORD`) — ordre non déterministe.
- **Retenu** : le manuel, le CHANGELOG et `DOCKER_START.md` disent la mesure — refus de `config`, `pull`, `up` ; `ps`, `logs`, `exec`, `stop` encore utilisables avec la version mesurée (diagnostic et arrêt possibles ; le script de sauvegarde, qui passe par `exec`, ne dépend pas de `.env`, il lit le mot de passe dans le conteneur) ; Compose nomme **une** variable à la fois, **pas toujours la même** : poser les deux avant de relancer. La version mesurée est nommée, faute de pouvoir garantir les autres.
- **Écartées** : écrire l'affirmation de la fiche (fausse sur la version mesurée) ; ne rien dire des sous-commandes (l'exploitant face au refus doit savoir s'il peut encore diagnostiquer).
- **Réversible** : oui (texte). La règle (refus par Compose, avertissement par Kesh) n'est pas touchée : seul le constat documentaire change.

## C-15-13a-2 — 15-13a (T4/T8) : l'étape CI retire aussi chaque mot de passe MariaDB SEUL, l'autre posé

- **Contexte** : l'AC 4 b exige que `docker compose config -q` **sans les deux** variables échoue en nommant l'une d'elles. Jouée localement, la mutation **M27** de la fiche (« remettre `:-kesh_dev_root` ») **survit** à cette forme : `MARIADB_PASSWORD` reste obligatoire, Compose refuse en la nommant, et l'étape passe — le retour du défaut publié de root ne se voit pas en CI.
- **Retenu** : l'étape garde la vérification de l'AC 4 b et ajoute une boucle : pour chaque variable, retirer les deux puis poser **l'autre** ; Compose doit refuser **en nommant celle qui manque** (`required variable <nom> is missing`). M27 rougit (rejouée sous `bash -eo pipefail` sur une copie). Reste vert, à dessein, le retour d'un défaut sur le seul `MARIADB_PASSWORD` du service `mariadb` : la `DATABASE_URL` de `kesh-api` exige encore la variable, Compose refuse toujours son absence — aucun changement de comportement ; le test Rust `mariadb` le voit.
- **Écartées** : s'en tenir à l'esquisse de la fiche (M27 survit) ; ne compter que sur le test Rust (l'AC 4 veut le refus exercé par Compose lui-même).
- **Réversible** : oui.

## C-15-13a-3 — Recette `ALTER USER` : sous-shell `set -e`, mots de passe affichés avant, compte lu dans le conteneur (revue P1, B-L1, E-3 = A-5)

- **Contexte** : la revue de code P1 relève que la recette n'affichait les nouveaux mots de passe qu'**après** l'`ALTER USER` (session coupée = mots de passe perdus), sans arrêt à la première erreur, et avec le compte `kesh` en dur alors que `MARIADB_USER` est réglable.
- **Retenu** : la recette tient dans un sous-shell `( set -e … )` — arrêt à la première erreur sans fermer le terminal interactif ; `echo` des deux valeurs **avant** l'application, puis `read` (pause pour les recopier) ; compte applicatif lu par `docker compose exec -T mariadb printenv MARIADB_USER` et passé **en premier** (seule ligne sans `IF EXISTS` : s'il est refusé, rien n'a changé) ; ligne témoin `CHANGÉS DANS LA BASE` ; `up -d` dans un **second** bloc, pour qu'un collage du premier ne le lance pas avant l'enregistrement de `.env`. Rejouée sur un projet Compose jetable `kesh1513ap1` (`MARIADB_USER=compta`) : nominal, échec de connexion root (rc 1, pas de ligne témoin), compte absent (rc 1, ERROR 1396, root et compte inchangés).
- **Écartées** : `set -e` nu (fermerait le terminal sur une erreur) ; écrire les mots de passe dans un fichier sous `umask 077` (un fichier de secrets de plus à effacer) ; dire seulement « adaptez `kesh` » (le lecteur ne le fera pas).
- **Réversible** : oui (texte du manuel).

## C-15-13a-4 — Volume `kesh_db_data` du manuel laissé à #575 (revue P1, B-L4)

- **Contexte** : B-L4 relève le nom de volume `kesh\_db\_data` (`admin-manual.tex`, procédure de mise à jour standard et stratégie de sauvegarde), faux contre `kesh-mariadb-data`.
- **Retenu** : laissé à **#575**, dont le constat cite précisément ces deux sites ; la story 15-13a ne les touche pas. Le reste de B-L4 (`cd /opt/kesh` de la restauration) est traité : renvoi à `COMPOSE_DIR` du script de sauvegarde.
- **Écartée** : corriger ici (double traitement d'une issue ouverte, et la section Synology de #575 demande une refonte plus large).
- **Réversible** : oui.

## C-15-7b1-1 — 15-7b1 (T0) : colonne `Rejeu` de `seed_demo` inchangée, bras `422` du handler conservé

- **Contexte** : au T0 de la 15-7b1, deux points que la fiche ne tranche pas. (1) Le registre des routes porte une seconde colonne, `Rejeu` ; `seed_demo` y est `SansEcritureAuJournal`, et va désormais écrire au journal d'audit et rejouer sa dernière transaction sur 1213 — dans `kesh-seed`, pas dans le handler. (2) Le handler rend `422` sur `InactiveOrInvalidAccounts` ; la fiche dit « `StepAlreadyCompleted` ⇒ 400, toute autre erreur ⇒ 500 ».
- **Retenu** : (1) `SansEcritureAuJournal` conservé, comme les neuf routes d'onboarding tracées par la 15-7a2 (C-15-7a2-4 : la colonne grave l'inventaire de l'AC1 de la 15-5e1, au sens du journal comptable) ; compteurs de la colonne inchangés ; le point (vi) du doc-comment du registre nomme `seed_demo` parmi les routes rejouées quand même, rejeu **dans `kesh-seed`**, que ni le volet (c) ni le (c bis) ne voient. (2) Le bras `422` est conservé tel quel ; seul s'ajoute `StepAlreadyCompleted ⇒ 400`, le repli `500` reste.
- **Écarté** : classer `seed_demo` `Rejouee` (le volet (c) exige l'appel d'une enveloppe dans le corps du handler ; déplacer le rejeu dans le handler contredirait l'AC 1, qui le place autour de la dernière transaction de `kesh-seed`) ; retirer le bras `422` (changement de code de réponse non demandé).
- **Réversible** : oui (une ligne du registre, un bras de `match`).

## C-15-7b1-2 — 15-7b1 (revue de code P1) : la garde sous verrou et le rejeu de `seed_demo` prouvés par déclencheurs

- **Contexte** : revue de code P1 de la 15-7b1, E-1 = A-1 (MEDIUM) et B-3 (LOW). La revérification de l'étape sous verrou de `seed_demo` est masquée par la pré-vérification non verrouillée du handler, et l'interblocage de la dernière transaction était déclaré non provocable de façon déterministe. La remédiation ne doit toucher aucune ligne de code de production exécutable.
- **Retenu** : (1) appel **direct** de `kesh_seed::seed_demo` aux étapes 3 et 4 (contourne la pré-vérification) ; (2) la course `start-production` / `seed-demo` rendue déterministe par un déclencheur `AFTER INSERT ON fiscal_years` qui pose l'étape 3 — entre la pré-vérification et la dernière transaction —, pour prouver le bras `400` du handler ; (3) une **vraie** 1213 par `SIGNAL SQLSTATE '40001' SET MYSQL_ERRNO = 1213` (vérifié : MariaDB rend bien `ERROR 1213 (40001)`), pour le prédicat, et levée **une seule fois** par un déclencheur dont le compteur vit dans une table MyISAM (non transactionnelle : l'annulation de l'essai ne l'efface pas), pour le rejeu de bout en bout. Mutations M6 à M9 rouges.
- **Écarté** : un faux `DatabaseError` construit en Rust (le prédicat descend vers `MySqlDatabaseError`, non constructible hors de sqlx) ; deux connexions en interblocage réel (patron de `rejeu_interblocage_e2e.rs`, plus lourd, et il ne vise pas la transaction de `seed_demo` elle-même) ; laisser l'angle mort écrit (la fiche le permettait, mais il était testable à faible coût).
- **Réversible** : oui (tests seuls).

## C-15-7b1-3 — 15-7b1 (revue de code P1) : le message du `422` de `seed-demo` laissé tel quel, écrit comme dette

- **Contexte** : E-2 (LOW). Le message « … avant de relancer la démo » invite à relancer, alors qu'après l'échec de la dernière transaction le plan et l'exercice sont commités et un nouveau `seed-demo` échoue (500). Le corriger exige de toucher une chaîne du code de production, que la remédiation s'interdit.
- **Retenu** : dette écrite à la fiche (« Ce que la story ne fait pas »), rattachée à **#538** — l'atomicité des quatre premières validations fait disparaître l'état non relançable, et le message redevient juste. Atteinte pratiquement impossible avec le plan PME embarqué. B-2 (boucle `InactiveOrInvalidAccounts`) reste à l'arbitrage de Guy (C-15-7-14).
- **Écarté** : changer le texte maintenant (code de production en remédiation de fin de boucle, qui rouvrirait la boucle de revue pour un LOW).
- **Réversible** : oui (une chaîne).

## C-15-7b2-1 — 15-7b2 (T0) : `onboarding::reset` passe à `Traced` / `SansEcritureAuJournal` ; partitions recomptées

- **Contexte** : au T0 de la 15-7b2, le registre des routes compte 114 routes (107 / 5 / 2), non 112 (105 / 5 / 2) comme le dit la fiche ; et sa colonne `Rejeu` porte la remise à zéro en `Exemptee` (« un 1213 annule sa transaction unique et la relance manuelle est sûre »), alors que l'AC 2 l'enveloppe désormais dans `retry_with` — dans `kesh-seed`, hors de `src/routes/`.
- **Retenu** : `Traced`, et `SansEcritureAuJournal` comme `seed_demo` (C-15-7b1-1 : la colonne grave l'inventaire de l'AC1 de la 15-5e1, au sens du journal comptable ; le volet (c) exige une enveloppe dans le corps du handler). Le point (vi) du doc-comment nomme la remise à zéro parmi les routes rejouées quand même. Partition d'audit 108 / 4 / 2 = 114 ; partition de rejeu 24 / 3 / 90 = 117. Le manuel dit « 108 des 114 ».
- **Écarté** : `Rejouee` (déplacer le rejeu dans le handler contredirait l'AC 2) ; garder `Exemptee` (la raison écrite serait fausse).
- **Réversible** : oui (une ligne du registre, quatre nombres).

## C-15-7b2-2 — 15-7b2 (T0) : le rejeu de `reset_demo` prouvé de bout en bout, et le prédicat sur un vrai cycle

- **Contexte** : la fiche déclare angle mort l'interblocage de `reset_demo` lui-même (« non reproductible de façon déterministe ») et demande, au test 13, un vrai cycle de verrous pour le prédicat. La 15-7b1 a depuis montré (C-15-7b1-2) qu'une 1213 levée **une fois** par un déclencheur — compteur en table MyISAM, que l'annulation n'efface pas — prouve un rejeu de bout en bout.
- **Retenu** : (1) test 13 tel qu'écrit (deux connexions hors du pool, `reset_retry_probe`, cycle réel, 1213 ⇒ vrai ; 1205 ⇒ faux ; variantes ⇒ faux, `ResetForbidden` compris) ; (2) **en plus**, un test 13b : déclencheur `BEFORE INSERT ON audit_log` qui lève une 1213 à la première écriture d'`installation.reset` ; la remise à zéro aboutit au second essai, une seule entrée, tables vidées. L'angle mort se réduit à l'interblocage **naturel** (non provoqué), écrit au Dev Agent Record.
- **Écarté** : laisser l'angle mort entier (testable à faible coût) ; remplacer le vrai cycle par `SIGNAL` (la fiche le demande, et il prouve la conversion par `map_db_error` d'une erreur réellement émise par InnoDB).
- **Réversible** : oui (tests seuls).

## C-15-7b2-3 — 15-7b2 (développement) : l'exception au Pattern 5 écrite en note, la liste « Deny list » n'existant plus

- **Contexte** : l'AC 10 prescrit d'inscrire `kesh_seed::reset_demo` à la « liste d'exceptions » du Pattern 5 (« Deny list », `*(none)*`, `docs/MULTI-TENANT-SCOPING-PATTERNS.md:330-336`). À `181efa3c`, cette liste n'existe plus : le document porte une table *Where This Applies* et des **notes** par route (journal_entries, letterings).
- **Retenu** : la ligne `reset` de la table dit l'ordre complet et renvoie à une **note** « exception to the Global Lock Order », écrite sur le patron des notes voisines (motif, atténuation, tests, ce qui n'est pas provoqué) ; le paragraphe *Known Risk — KF-002-H-002* ne cite plus `reset` parmi les lock-and-release.
- **Écarté** : recréer une section « Deny list » pour une seule entrée (structure abandonnée par le document).
- **Réversible** : oui (texte).

## C-15-7b2-4 — 15-7b2 (développement) : montage du test 4 par une facture sans TVA ; `credit_note_number_sequences` peuplée

- **Contexte** : le test 4 exige une facture **validée** et un avoir sur une démonstration. La démonstration ne désigne pas de compte de TVA due : la validation d'une ligne à 8,1 % rend `400 CONFIGURATION_REQUIRED`.
- **Retenu** : la ligne de facture est à `0.00` % — la facture se valide, l'avoir se crée, et l'écriture existe ; le relevé réel a montré `credit_note_number_sequences` peuplée par l'avoir : elle sort de la liste fermée des tables vides (17 tables, assertée égale à l'ensemble relevé).
- **Écarté** : désigner un compte de TVA due au montage (geste de plus, sans rapport avec la remise à zéro) ; laisser `vat_rates` seule témoin de la TVA (elle est peuplée par le seed).
- **Réversible** : oui (montage de test).

## C-15-7b2-5 — 15-7b2 (revue de code P1, A-1 = E-3) : le manuel borne la réparation de #528 à zéro ou une société

- **Contexte** : revue de code P1 (Sonnet ×3, prompt `e3368ba7`) — A-1 (MEDIUM) = E-3 (LOW). Le manuel d'administration disait que la réinitialisation répare une installation touchée par #528 « quel que soit son nombre de sociétés » ; le code (`reset_body`, AC 2.2) rend `Invariant` (500, rien d'effacé) dès deux sociétés. La formule venait de la ligne de l'AC 11 (R4-5 de la P4), vraie de `reattach_orphan_principals_in_tx`, pas de la remise à zéro.
- **Retenu** (décision de l'orchestrateur) : `admin-manual.tex` dit « si elle compte aucune ou une société », et qu'à deux sociétés ou plus — réelles ou provisoires — le bouton répond par une erreur interne sans rien effacer, la réparation relevant de la 15-7b3 ; PDF régénéré (`make -B fr`), contrôlé aplati ; même bornage au CHANGELOG `[0.13.0]` ; ligne de l'AC 11 de la fiche corrigée. Aucune ligne de code touchée.
- **Écarté** : faire réparer la remise à zéro sur N ≥ 2 (périmètre de la 15-7b3, C-15-7-46).
- **Réversible** : oui (texte).

## C-15-7b2-6 — 15-7b2 (revue de code P1) : LOW documentaires appliqués, LOW de code écrits en dette

- **Contexte** : P1 : B 6 LOW, E 7 LOW (dont E-3, absorbé par C-15-7b2-5), A 3 LOW. Consigne de l'orchestrateur : ne toucher aucun fichier de code ni de test, pour que le dernier commit de code (`5fe1f918`) et ses gates restent valables.
- **Retenu** : appliqués — A-2 (`docker compose config` exécuté en lecture, sortie au Dev Agent Record), A-3 (grep de l'AC 10 rejoué sur l'état rebasé, sortie collée, intégrale au journal des gates), A-4 (Overfull comptés par passe). E-5 (champ d'audit) et E-7 (catalogues i18n) ne sont pas du texte de manuel : dette. En dette, avec leur motif, à la fiche : B-1 à B-6, E-1 (rattachée à #534), E-2, E-4 (comportement hérité, conservé), E-5, E-6, E-7.
- **Écarté** : corriger maintenant les LOW de code (rouvrirait la boucle de revue et les gates pour des LOW).
- **Réversible** : oui.

## C-15-13b-1 — 15-13b (T0) : le rapatriement n'est pas rejoué au T0, le démon Docker étant bloqué

- **Contexte** : le T0 prescrit de rejouer, sur un conteneur jetable, l'écriture d'un fichier `0600` par un conteneur root dans un dossier monté, puis le rapatriement `sudo cp` + `sudo chown`. Le 2026-10-09 vers 07:10, toute création de conteneur expire (`docker run` → 124 après 60 s) : le noyau signale des tâches `dockerd` en état D, bloquées sur un rw-semaphore (`journalctl`, « blocked for more than 122 seconds »). Le démon sert encore `ps`/`info`. Les créations interrompues laissent des noms réservés sans conteneur. `sudo` n'est pas utilisable sans mot de passe depuis l'agent.
- **Retenu** : faire au T0 ce qui ne dépend pas du démon (`docker compose config`, client seul : graphie et contrôle rouge mesurés ; `make fr`), écrire le manuel selon la fiche, et **retenter** le rapatriement au T10 ; s'il est toujours impossible, l'écrire au Dev Agent Record comme **non mesuré**, à rejouer par l'orchestrateur ou en recette. `sudo` y est joué par un conteneur root (même effet de droits), ce qui sera dit.
- **Écartées** : redémarrer `dockerd` (emporterait `kesh-mariadb-dev` et les bases des autres agents — interdit) ; attendre sans fin ; déclarer le geste vérifié par raisonnement (« une hypothèse éliminée par raisonnement n'est pas une hypothèse testée »).
- **Réversible** : oui — la mesure se rejoue à tout moment.

## C-15-13b-2 — 15-13b (T6) : `CLAUDE.md` — seule la ligne de commande de la recette change, pas la mention de date

- **Contexte** : l'AC 16 a prescrit deux modifications du `CLAUDE.md` (la variable ajoutée à la recette E2E, et une mention « `KESH_ADMIN_BACKUP_DIR` ajoutée le … » accolée à « vérifié le 2026-08-04 »). La consigne de l'orchestrateur pour ce développement dit : « ne change QUE la ligne de commande de la recette E2E, rien d'autre dans ce fichier ».
- **Retenu** : la consigne de l'orchestrateur prime sur la fiche — `KESH_ADMIN_BACKUP_DIR=target/kesh-backup` ajoutée à la ligne `KESH_PORT=3000 KESH_STATIC_DIR=frontend/build`, rien d'autre. La date de vérification reste celle du 2026-08-04 ; le Dev Agent Record dit que la recette modifiée a tourné au gate E2E.
- **Écartées** : appliquer l'AC 16 a à la lettre (contrevient à la consigne).
- **Réversible** : oui (une phrase à ajouter si l'orchestrateur le souhaite).

## C-15-13b-3 — 15-13b (T6) : la phrase « deux gestes » du CHANGELOG devient un renvoi sans nombre

- **Contexte** : AC 13 c — la phrase partagée avec la 15-13a (« le manuel d'administration décrit les **deux gestes** ») est réécrite par la première fiche mergée, soit avec le décompte par fichier, soit par un renvoi sans nombre.
- **Retenu** : « décrit les gestes à faire, fichier par fichier (§ *Passer à la 0.13.0*) » — un renvoi qui n'a rien à recompter au rebase de la 15-13a ; le manuel, lui, porte le décompte (« Trois gestes pour chacun des deux compose » avec la 15-13b seule).
- **Écartées** : « trois gestes pour chacun » au CHANGELOG (à recompter par la seconde mergée, conflit certain sur la même ligne).
- **Réversible** : oui.

## C-15-13b-4 — 15-13b (T5) : la brochure reste telle quelle ; le nom du fichier est décrit, non écrit en entier

- **Contexte** : AC 11 n (brochure `:398`, « restauration sans accès SSH ») et AC 11 e (nom `kesh-pre-import-<horodatage>-….keshbackup`). Écrit d'un bloc dans la prose, ce nom insécable produisait un `Overfull \hbox` de 155 pt ; deux autres débordements (14 et 18 pt) venaient de la ligne `KESH_ADMIN_BACKUP_DIR` de la liste des variables et d'une incise ajoutée au premier geste.
- **Retenu** : brochure **inchangée** — elle vend l'export/import par l'écran, qui reste sans SSH ; l'exception de la sauvegarde pré-import est dite au manuel d'administration (ouverture de la section). Le nom est décrit « commence par `kesh-pre-import-` (suivi de l'horodatage) et finit par `.keshbackup` » ; `\sloppy` sur l'entrée de la liste ; l'incise « sous `environment:` » du premier geste retirée (la phrase d'ouverture le dit). Résultat : 55 `Overfull`, la même liste qu'avant, aucun nouveau.
- **Écartées** : nuancer la brochure (promesse vraie pour ce qu'elle vend) ; garder le nom entier avec des `\allowbreak` (rendu haché, contrôle aplati fragilisé).
- **Réversible** : oui.

## C-15-13b-5 — 15-13b (clôture) : rebasée sur `origin/main` (`bcded0c8`, 15-13a) ; union partout, décompte du manuel recompté à cinq et trois

- **Contexte** : clôture de la 15-13b après la revue de code (P1 Sonnet ×3, 15 LOW → remédiation `7b1db902` → P2 ciblée Haiku 0 au-dessus de LOW, rapport perdu au `cargo clean`). Elle devait passer **après** la 15-13a, mergée en `bcded0c8`. Conflits annoncés aux Dépendances de la fiche : paragraphe « Action requise » du CHANGELOG et « Pour qui garde son fichier compose » du manuel ; s'y sont ajoutés les fiches 15-13/15-13b (ajout/ajout), le registre, `sprint-status.yaml`, `DOCKER_START.md` et le PDF.
- **Retenu** : rebase (branche de sauvegarde `backup/15-13b-avant-rebase-bcded0c8`). Fiches : version de la branche (celle de `main` était l'état d'avant la validation P5, sans ligne propre). Registre et `sprint-status.yaml` par union. CHANGELOG : paragraphe de `main`, la phrase « la sauvegarde pré-import reste dans `/tmp` » remplacée par celle de la 15-13b ; l'entrée Sécurité « dossiers montés » placée avant #557 et #551. Manuel : les deux paragraphes conservés ; « Cinq gestes pour `docker-compose.yml`, trois pour `docker-compose.prod.yml` », montage `./backup` en dernier ; une seule phrase « Puis `docker compose config -q` » qui réunit les deux contrôles (aucun `ports` sous `mariadb`, `target: /data/backup` compté à 1). PDF régénéré, contrôlé aplati. Gates complets et E2E rejoués sur l'état rebasé, compilation à froid.
- **Écartées** : merge de `main` dans la branche (historique moins lisible) ; garder l'un des deux PDF (il aurait omis l'apport de l'autre) ; deux phrases « Puis … » successives (deux `config -q` à lancer, l'un sans l'autre).
- **Réversible** : oui (rebase ; branche non poussée, sauvegarde gardée).

## C-15-6c-1 — 15-6c : le numéro du compte refusé se lit par un lecteur partagé avec la 15-6b

- **Contexte** : les deux refus neufs nomment le compte par son numéro, lu « une fois le refus
  décidé, sans verrou » (AC1). La 15-6b lit déjà ce numéro par la même requête, écrite en ligne dans
  `invoice_settlements::claim_account_refusal`. La recopier deux fois de plus contredirait la règle
  DRY du dépôt.
- **Retenu** : `accounts::number_in_company(conn, company_id, account_id) -> Option<String>`, seule
  occurrence de la requête ; `claim_account_refusal` (15-6b) l'emprunte, sans changement de
  comportement (son doc-comment est ajusté). Côté comptes bancaires, une garde unique
  `bank_accounts::refuse_if_ledger_is_claim_account` sert la création (route) et les deux fonctions
  du dépôt ; `ClaimAccounts::side_of` dit le réglage occupé, compte débiteurs d'abord.
- **Écarté** : trois copies de la requête ; une comparaison écrite dans chaque route.
- **Réversibilité** : totale (extraction de fonctions).

## C-15-6c-2 — 15-6c : la spec E2E du lien vérifie aussi l'absence du compte débiteurs

- **Contexte** : C-15-6-15 fait lier la spec `bank-account-journal-link.spec.ts` au `1000` au lieu
  du `1100`. Ce faisant, la spec ne dit plus rien du filtre — or l'E2E est le seul test qui voie la
  désignation traverser la frontière HTTP (`GET /company/invoice-settings` → menu).
- **Retenu** : la spec affirme en plus que le `1100`, compte débiteurs désigné par le seed, est
  **absent** du menu. Aucune fixture partagée n'est touchée.
- **Écarté** : un scénario E2E neuf (refus serveur à l'écran) — couvert par les tests Vitest et
  HTTP, coût d'une spec de plus sans valeur de frontière supplémentaire.
- **Réversibilité** : totale (une assertion).

## C-15-6c-3 — 15-6c : le manuel d'administration et le CHANGELOG nomment aussi le rapprochement

- **Contexte** : l'AC9 demande la règle dans les deux sens ; l'angle mort « données antérieures »
  (un lien fautif existant n'est pas défait) n'était pas dit au lecteur.
- **Retenu** : le paragraphe *Comptes débiteurs et créanciers.* dit qu'un lien antérieur à la
  v0.13.0 n'est pas défait, ne bloque pas les autres réglages, mais fait refuser **chaque règlement
  et chaque rapprochement** de facture par ce compte bancaire (garde de la 15-6b) — avec le remède.
  Le CHANGELOG **complète** l'entrée #474 de la 15-6b (AC11 : une entrée par issue et par release).
- **Écarté** : une entrée CHANGELOG séparée ; taire l'angle mort au manuel.
- **Réversibilité** : totale (texte).

## C-15-6c-4 — 15-6c : le test de rejeu du PUT des réglages attend au `FOR UPDATE` de `before`

- **Contexte** : le gate complet rougit sur `rejeu_interblocage_e2e`
  `invoice_settings_update_is_replayed_when_it_is_the_deadlock_victim` (Story 15-5e1) : il prouve
  l'attente de la route sur le motif `UPDATE company_invoice_settings`. L'AC2 de la 15-6c fait lire
  `before` en `FOR UPDATE` : la route attend désormais le `S` du test **à cette lecture** (promotion
  S → X après son `INSERT IGNORE`), non plus à l'`UPDATE`. Le cycle est le même — c'est
  exactement l'interblocage préexistant que la note KF-004 réécrite décrit — ; seul le point
  d'attente a bougé. L'inventaire de la fiche (§ *Tests existants qui changent de sens*) ne
  l'avait pas vu : il cherchait les liens bancaires et les appels directs au dépôt, pas les motifs
  d'attente couplés à la forme du verrou des réglages.
- **Retenu** : le motif devient `["FROM company_invoice_settings", "FOR UPDATE"]`, commentaire et
  doc-comment du test ajustés ; le test reste ce qu'il était (rejeu d'un 1213 vrai, une version,
  une entrée d'audit). Grep des autres motifs couplés aux réglages : aucun autre site.
- **Écarté** : garder la lecture simple de `before` (rouvrirait la course que l'AC2 ferme, et le
  test 12 bis rougirait).
- **Réversibilité** : totale (un motif de test).

## C-15-6c-5 — 15-6c : pas de test neuf du filet de la 15-6b pour la sauvegarde et l'onboarding

- **Contexte** : revue de code P1, F3 (LOW) — l'import d'une sauvegarde et la création des
  réglages à l'onboarding peuvent produire le couple fautif ; la fiche les écrit comme angles morts,
  avec pour filet la garde à l'usage de la 15-6b. La consigne : un test du filet dans l'un des deux
  scénarios « si le montage est simple ».
- **Retenu** : pas de test neuf. L'état que produisent ces deux chemins — un compte bancaire lié au
  compte débiteurs désigné, écrit **sans** passer par les routes — est exactement celui que posent
  déjà les tests de la 15-6b, par `UPDATE bank_accounts SET journal_account_id = …` en SQL brut
  (`reconciliation_e2e.rs`, « sans passer par la 15-6c », tests 7 et 8 de la 15-6b) : le filet y est
  prouvé sur l'état, quel que soit le chemin qui l'a écrit. Monter un `.keshbackup` complet (inventaire
  de tables identique, manifeste, rejeu) pour reproduire le même état n'est pas un montage simple.
- **Écarté** : un test d'import de sauvegarde portant le couple fautif (montage lourd, ne prouverait
  rien de plus sur la garde) ; un test d'onboarding (même état final).
- **Réversibilité** : totale (un test pourra s'ajouter quand le produit tiendra une comptabilité réelle).

## C-15-6c-6 — 15-6c : la duplication création / `claims_for_target` est écrite comme dette, non refactorée

- **Contexte** : revue de code P1, B3 = A4 (LOW) — `create_bank_account` lit les comptes de créance
  par `claim_accounts_in_share_mode` puis appelle `refuse_if_ledger_is_claim_account` en ligne, dans un
  `if let Some(account_id)`, alors que le remplacement et le lien passent par `claims_for_target`.
  Deux sites de la même règle d'acquisition (« pas de lecture ni de verrou S sans cible ») : l'un peut
  dériver de l'autre.
- **Retenu** : **dette écrite**, pas de refactorisation à la clôture — la remédiation de la revue P1
  ne touche aucune ligne de code exécutable (consigne de l'orchestrateur), et un changement de code de
  production rouvrirait la boucle de revue. **Propriétaire** : l'orchestrateur de l'Epic 15 ; remède
  attendu : remplacer le bloc en ligne de la création par `claims_for_target` (une ligne), à la
  prochaine story qui touche `routes/bank_accounts.rs`. Le comportement est aujourd'hui identique sur
  les trois sites (test 1, tests 2 à 4).
- **Écarté** : refactorer maintenant (code de production après une revue close) ; taire la duplication.
- **Réversibilité** : totale.

## C-15-1a-i-1 — 15-1a-i (T0) : le verrou d'intervalle « plus grande clé » est démenti par la mesure, le texte de R7 suit
- **Contexte** : R7 (F4-1 de P4, « raisonné sur le moteur, non exécuté ») annonçait qu'une dissolution du
  groupe de plus grande clé tenait le trou `(Kmax, +∞)` d'`idx_jel_lettering` et faisait attendre
  l'`UPDATE` d'une création concurrente — cas d'interblocage résiduel « courant ». Mesure T0 sur
  `kesh_151ai` (MariaDB 10.11.16, 6000 lignes, 500 groupes) : l'`UPDATE … SET lettering_key = 4000 …`
  d'une création **n'attend pas** pendant la dissolution de la plus grande clé (ni pendant celle d'une clé
  dont le trou suivant contient la nouvelle valeur) ; un `INSERT` direct d'une ligne portant la clé 4000,
  lui, attend (1205) — le trou est tenu, mais la mise à jour d'un index secondaire ne s'y heurte pas.
  Le cas F3-5 (insertion d'une ligne **ouverte** pendant la dissolution de la plus petite clé) est, lui,
  **confirmé** (1205 au bout de 3 s, sur une écriture sans rapport).
- **Retenu** : R7 de la fiche corrigé (puce F4-1 et paragraphe des verrous d'intervalle) — le cas
  « création contre dissolution de la plus grande clé » n'est plus un cycle résiduel ; le reste de R7
  inchangé. Les routes restent rejouées (aucune règle ni critère ne bouge).
- **Écartées** : garder le texte raisonné « par prudence » — il décrivait un cycle que la mesure n'observe pas.
- **Réversible** : oui (texte de fiche).

## C-15-1a-i-2 — 15-1a-i (T3) : le plafond de 200 lignes est un refus de forme de la route, pas de la primitive
- **Contexte** : AC6 fixe 200 lignes (400 `LETTERING_TOO_MANY_LINES`) ; AC3 ne le range dans aucun rang.
- **Retenu** : variante `DbError::LetteringTooManyLines { max }` (pour réutiliser la correspondance d'erreurs
  du dépôt), contrôlée par `letterings::check_manual_line_count` **dans le handler**, avant toute lecture
  (et avant `LETTERING_TOO_FEW_LINES`). La primitive ne la contrôle pas : les appelants `System` (15-1a-ii,
  15-1a2) ne doivent pas buter sur un plafond d'écran pour une pièce réglée en beaucoup de fois.
- **Écartées** : un `AppError` dédié (une seconde famille de codes pour un seul refus) ; le contrôle dans
  `check_line_ids` (il aurait borné le mode `System`).
- **Réversible** : oui.

## C-15-1a-i-3 — 15-1a-i (T3) : l'acte 1 lit aussi `je.entry_number` ; le rang 4 se lit après les verrous d'exercice
- **Contexte** : R7 point 1 liste les colonnes de l'acte 1 sans `entry_number`, que la réponse et l'audit
  exigent (AC6, AC10). Et l'ordre des **refus** (AC3) intercale des lectures en base (rangs 4, 4 bis, 5)
  entre des contrôles purs, sans fixer l'ordre des **lectures**.
- **Retenu** : `je.entry_number` ajouté à la lecture verrouillante (l'en-tête est déjà tenu, aucun verrou de
  plus) ; séquence : acte 1 → rang 3 (pur) → en `Manual`, R7 point 2 (a)(b)(c) → rang 4 (lettrabilité,
  lecture ordinaire) → borne et rang 4 bis → rang 5 → rangs 6 et 7 (purs). L'ordre des refus est celui
  d'AC3 ; la lettrabilité est une des lectures ordinaires que R7 tolère après (a).
- **Écartées** : une seconde lecture des en-têtes pour le numéro ; lire la lettrabilité avant (a) (elle
  ouvrirait la vue `REPEATABLE READ` plus tôt, sans rien garantir de plus).
- **Réversible** : oui.

## C-15-1a-i-4 — 15-1a-i (T10) : le vocabulaire du lettrage en allemand, anglais et italien
- **Contexte** : la fiche arrête les dix textes français ; les trois autres locales sont à écrire. Les
  catalogues emploient déjà « Abgleich » / « match » / « riconciliazione » pour le **rapprochement
  bancaire**, que le lettrage ne doit pas paraître désigner.
- **Retenu** : de-CH **Ausgleich** (ausgleichen, Ausgleich aufheben — le « OP-Ausgleich » des comptables
  suisses), en-CH **matching** (« matching group », match / unmatch), it-CH **abbinamento** (abbinare,
  disabbinare — le terme que le catalogue italien employait déjà pour « à lettrer »). Libellés d'audit :
  « Lettrage posé / retiré », « Ausgleich gesetzt / aufgehoben », « Matching set / removed »,
  « Abbinamento posto / rimosso ».
- **Écartées** : « Abgleich » et « reconciliation » (réservés au rapprochement bancaire) ; « lettering » en
  anglais (calque du français, inconnu d'un lecteur anglophone).
- **Réversible** : oui (textes de catalogue ; la 15-1c, qui fait l'écran, pourra les reprendre).

## C-15-1a-i-5 — 15-1a-i (T0) : clés 15-13* ajoutées au registre de sprint de cette branche, statut relevé sur leurs branches
- **Contexte** : consigne de l'orchestrateur — ajouter les clés 15-1*, 15-12*, 15-13* manquantes à
  `sprint-status.yaml`. Les 15-12* y sont déjà ; manquent `15-1a-i`, `15-1a-ii`, `15-1a2`, et les trois
  clés 15-13* (aucune fiche 15-13 sur `main` : elles vivent sur les branches `story/15-13*`).
- **Retenu** : les six clés ajoutées ; les statuts 15-13* recopiés de leurs propres branches au
  2026-10-09 (15-13 `split`, 15-13a `done`, 15-13b `review`), avec un commentaire disant que **leur
  statut fait foi sur leur branche** — l'union au merge garde la valeur la plus récente.
- **Écartées** : ne pas les ajouter (contraire à la consigne) ; inventer un statut propre.
- **Réversible** : oui.

## C-15-1a-i-6 — 15-1a-i (revue de code P1) : B1 reclassé LOW — la destruction de la marque par la modification et la suppression relève de la 15-1a-ii
- **Contexte** : la lentille B (B1, MEDIUM conditionnel) relève qu'`update_in_tx` (`DELETE` puis `INSERT` des lignes, nouveaux `id`, sans `lettering_*`) et `delete_in_tx` (cascade `fk_jel_entry`) détruisent en silence la marque d'une écriture lettrée.
- **Retenu** (décision de l'orchestrateur) : **LOW**. C'est le périmètre de la 15-1a-ii (AC8), et C124 interdit tout tag entre les merges de la 15-1a-i et de la 15-1a-ii. Vérifié sur la fiche 15-1a-ii (`main`, lecture seule) : AC8 pose `ModificationGuard::Lettered` / `409 ENTRY_LETTERED` par une fonction propre, `lettering_guard`, **inconditionnelle**, à l'étape **3-quinquies** de `delete_in_tx` (après le verrou de période, hors `enforce_ownership`, donc aussi pour la dévalidation) et à l'étape **7-bis** d'`update_in_tx` (après le verrou de période, **avant** l'instantané, le court-circuit no-op et le `DELETE`+`INSERT` de l'étape 8) ; T4 et T12 les testent (`PUT`, `DELETE`, en-tête seul, `PUT` identique, `delete_in_tx(…, false)`). Les deux sites de B1 sont donc couverts.
- **Écartées** : un refus minimal dans `update_in_tx`/`delete_in_tx` dès la 15-1a-i (doublon de la garde que la 15-1a-ii pose avec son rang, son message et son écran).
- **Réversible** : oui — tant qu'aucun tag ne sépare les deux merges (C124).

## C-15-1a-i-7 — 15-1a-i (revue de code P1) : les textes publics disent ce que fait le code livré — seul le lettrage manuel existe
- **Contexte** : B2 = A-L3 (MEDIUM) — le CHANGELOG, `docs/api-external.md` et le glossaire du manuel décrivaient au présent des groupes `reversal` et `document` posés par Kesh, qu'aucun appelant de production ne pose (`Mode::System` sans appelant) ; A-L2 — « le code du groupe est visible sur chaque ligne » alors qu'aucun composant n'affiche `letteringCode`.
- **Retenu** : CHANGELOG — « Le lettrage manuel, par l'API », exemples réduits à ceux qu'un lettrage manuel peut poser (un acompte et sa reprise, une écriture et sa contre-passation — ni l'une ni l'autre n'est une pièce), phrase « Seul le lettrage manuel existe à ce stade : Kesh ne lettre pas encore de lui-même une facture soldée… », « exposé … dans les réponses de l'API » ; le motif du bump cite « modifier, supprimer ou annuler une écriture lettrée ». `api-external.md` — `letteringOrigin` vaut `manual`, `document`/`reversal` réservés ; « À ce stade, Kesh ne lettre rien de lui-même » ; les deux refus du `DELETE` propres à ces origines portent « (aucun groupe … n'existe encore) ». Glossaire (`.tex` et PDF régénéré) — exemples manuels, et « Kesh ne lettre encore rien de lui-même ». ⚠️ **À reprendre par la 15-1a-ii** (contre-passation qui lettre : retirer « ni une écriture et sa contre-passation », la réserve sur `reversal` et l'annotation du refus `LETTERING_LINE_OWNED_BY_DOCUMENT`) **et par la 15-1a2** (pièces : réintroduire « une facture et ses règlements », la réserve sur `document`, l'annotation de `LETTERING_IS_DOCUMENT`) — relevé par la valeur : `git grep -nE "Seul le lettrage manuel|ne lettre (pas encore|rien|encore rien)|réservé|n'existe encore" -- CHANGELOG.md docs`.
- **Écartées** : laisser les textes en l'état au motif que la v0.13.0 sort en une release (C124) — un texte faux entre deux merges est un texte faux, et rien ne garantit qu'on se souviendra de le vérifier ; retirer les origines du contrat (`letteringOrigin` les déclare, la contrainte SQL aussi).
- **Réversible** : oui (textes).

## C-15-1a-i-8 — 15-1a-i (revue de code P1) : la preuve de la sauvegarde lettrée est un aller-retour HTTP en trois temps
- **Contexte** : E-1 = A-L4 (MEDIUM) — l'exemption (ii) de R3 (« `backup.rs` rétablit les marques telles qu'exportées ») ne reposait que sur la lecture dynamique des colonnes ; AC14 demandait aussi de vérifier `is_no_op_change` et `entry_snapshot_json`.
- **Retenu** : `full_import_round_trip_keeps_lettering_marks` dans `admin_full_import_e2e.rs` — (1) l'export porte les deux colonnes (manifeste et NDJSON), (2) base rendue divergente par délettrage puis import : clé et origine rétablies, (3) sauvegarde antérieure simulée par `strip_column` : lignes ouvertes. Mutations rejouées : colonnes exclues de l'export (`non_generated_columns`) → rouge au temps 1 ; colonnes écartées de l'`INSERT` de restauration → rouge au temps 2. `is_no_op_change` et `entry_snapshot_json` relus : tous deux construits champ par champ, sans la marque — non faussés ; ils ne la voient pas, ce qui ne compte qu'à partir de la 15-1a-ii, où une écriture lettrée n'atteint plus l'étape 8.
- **Écartées** : un test de `backup.rs` seul sur `column_names` (prouve l'export, pas la restauration).
- **Réversible** : oui.

## C-15-1a-i-9 — 15-1a-i (revue de code P1) : LOW appliqués, LOW écartés
- **Appliqués** : E-2 (`fiscal_year_names` et `group_account_number` rendent `Invariant` au lieu d'un nom ou d'un numéro vide, comme le mode `Manual` ; `find_group` partage la lecture — DRY) ; E-4 (`{ $max }` dans les quatre catalogues, `t_args` au site ; le test de mapping passe `max: 7`, et l'E2E asserte le message rendu par le catalogue chargé) ; B6 (détecteur classé sur le **mot** `UPDATE`/`INSERT`/`REPLACE`, quel que soit le blanc, hors `FOR UPDATE` et `ON UPDATE` — la première version, « tout mot », comptait les lectures `FOR UPDATE` de la primitive : 4 au lieu de 2) ; E-6 = A-L6 (`lettering_invariants` pose un groupe `reversal` en mode `System` et un groupe dans une seconde société, plus deux contrôles négatifs isolés, origines et sociétés) ; E-7 (portée du test de concurrence écrite : il prouve l'absence de 500, pas une attente de verrou — aucun test ne l'observe, angle mort assumé) ; E-8 (`a_read_only_key_reads_but_cannot_letter`) ; A-L1 (décompte des tests recompté : 60) ; A-L5 (titre du tableau de gates).
- **Écartés, et pourquoi** : B3 (code `System` sans appelant de production — voulu, ses appelants sont la 15-1a-ii et la 15-1a2 ; il est testé au dépôt) ; B4 (`find_group` en trois lectures sans instantané commun — sans effet comptable, et depuis E-2 aucune course ne peut produire un `Invariant` : exercices et comptes ne disparaissent pas sous des lignes, FK sans cascade) ; B5 (verrous d'exercice pris avant le refus `document` — inatteignable tant qu'aucun groupe `document` n'existe ; à reconsidérer par la 15-1a2) ; B7 (présence lexicale de `check_rows_affected` — limite déjà écrite au doc-comment du test, seule garde possible sans crochet de production) ; B8 = E-5 (zéros de tête acceptés et réutilisation d'une clé après dissolution — sans conséquence de sécurité ; l'audit distingue les deux vies par l'horodatage ; trancher la forme canonique relève de l'écran, 15-1c) ; E-3 (plan des lectures verrouillantes non épinglé — écart déjà écrit au T0 pour les tables vides, sans effet sur l'exactitude, routes rejouées) ; A-L6 seconde moitié (rangs 5 `OwnedByCreditNote`/`OwnedBySupplierInvoice`/`OwnedBySettlement` non exercés par la primitive : la détection réutilise `reversal_blockers`, dont `OwnedBySettlement` et `OwnedBySupplierInvoice` sont exercés par leurs suites (`invoice_settlement.rs:761`, `supplier_invoices_repository.rs:1374`) et dont les quatre motifs de pièce sont inventoriés par `journal_entries_modification.rs:54-69` ; `OwnedByCreditNote` n'y est qu'inventorié — angle mort assumé, risque faible : un seul chemin de détection).
- **Réversible** : oui.

## C-15-1a-i-10 — 15-1a-i (revue de code P1) : tmpfs de MariaDB plein — nouvelles tables basculées dans `ibdata1`
- **Contexte** : au gate complet de la remédiation, le tmpfs de `kesh-mariadb-dev` (4 Go) est plein : `ibdata1` pèse 3,5 Go (espace d'annulation des gates de la journée, purgé mais jamais rendu au système de fichiers) ; toute création de table échoue en `1114 table is full` (1523 puis 330 échecs, aucun du code). Redémarrer le conteneur est interdit (bases des autres agents).
- **Retenu** : `SET GLOBAL innodb_file_per_table = OFF` — les nouvelles tables vont dans l'espace libre **interne** d'`ibdata1` (sondé : 50 Mo insérés sans croissance du fichier) ; mes bases de test résiduelles supprimées (1200 + 259 enregistrées dans `kesh_151ai._sqlx_test_databases`, quatre `_sqlx_test_guard_tracking_*` de mes deux runs) ; gate complet rejoué à `--test-threads=2` (profil `ci`) pour tenir dans l'espace. Réglage volatil : un redémarrage le remet à `ON`. ⚠️ **À l'orchestrateur** : planifier un redémarrage du conteneur à un moment creux (il rend les 3,5 Go), et prévenir les agents dont les gates tournent.
- **Écartées** : redémarrer (interdit) ; attendre (l'espace ne revient pas) ; s'arrêter sans gate (la remédiation touche `kesh-db`, gate complet obligatoire).
- **Réversible** : oui (`SET GLOBAL innodb_file_per_table = ON`, ou redémarrage).

## C-15-1a-i-11 — 15-1a-i (revue de code P2) : les deux MEDIUM nés de la remédiation P1
- **Contexte** : B2-1 = E2-1 — le détecteur lexical de R3, réécrit en P1 (B6) pour classer sur le **mot**, découpe le texte **brut** du littéral : `"…;\nUPDATE …"` rend `nUPDATE`, aucun verbe (idem `\t`, `\r`, `\0`) ; l'ancien `contains("UPDATE ")` voyait ce cas. A2-1 — le relais des six textes provisoires de C-15-1a-i-7 (« Kesh ne lettre rien de lui-même », origines « réservées ») n'existait qu'au registre ; la phrase prescrite par AC15 (ii) de la 15-1a-ii (« La contre-passation lettre… ») les contredirait.
- **Retenu** : (1) `neutraliser_echappements` — la barre oblique inverse **et** le caractère qui la suit deviennent deux espaces avant le découpage, appliqué aussi aux chaînes brutes (élargit, ne rétrécit jamais — P7) ; cinq littéraux à l'auto-test (`N` à `R`), 9 → 14 écritures ; mutation « neutralisation retirée » → rouge (10 ≠ 14). (2) Section « Reçu de la 15-1a-i — revue de code P2 » dans `15-1a-ii-gardes-du-lettrage.md` et `15-1a2-lettrage-des-pieces.md` (branche de planification, `fecf18ae`) : sites, relevé par la valeur — le `git grep` de C-15-1a-i-7 (62 lignes dont 56 hors sujet) et sa forme resserrée (`sont réservés aux lettrages`, exactement les six sites) plus le PDF aplati —, part de chaque story, contradiction avec AC15 (ii) ; règle : chaque story réécrit ces textes **dans le même commit** que le comportement qui les rend faux.
- **Écartées** : (1) remplacer seulement `\n`, `\t`, `\r`, `\0` par une liste de `replace` — une forme oubliée (`\x0A`, `\u{a}`) referait un garde-fou muet, la paire entière les couvre toutes ; décoder réellement les échappements — plus de code pour le même résultat sur le découpage. (2) Réécrire AC15 (ii) et T11 de la 15-1a-ii — hors du mandat du remédiateur, le patron « Reçu de » transmet sans réécrire.
- **Réversible** : oui.

## C-15-1a-i-12 — 15-1a-i (revue de code P2) : LOW appliqués, LOW écartés
- **Appliqués** : B2-2 (`build_group` rend `Result` et refuse en `Invariant` un exercice absent des noms — **code de production** ; un `expect` aurait fait paniquer la tâche, ce que la doctrine du dépôt proscrit au profit d'une erreur remontée) ; B2-5 = A2-5 (doc-comments `routes/letterings.rs:12`, `:45`, `errors.rs` `LetteringTooManyLines` renvoient à `MAX_LINES_PER_GROUP` ; T10 de la fiche en `{ $max }`) ; A2-3 (au contrôle négatif (2) de `lettering_invariants`, `find_group` par la seconde société atteint l'`Invariant` du compte puis, l'écriture intruse passée sur un exercice de la première société, celui de l'exercice — deux messages distincts, assertés par leur mot ; bon marché : la donnée corrompue y est déjà construite) ; A2-4 (sept littéraux) ; A2-2 (gate P1 qualifié au Status, au Change Log et au Dev Agent Record ; journaux non conservés écrits comme tels ; effet global du réglage sur les autres agents écrit).
- **Écartés, et pourquoi** : B2-3 = E2-3 (le temps (3) de l'aller-retour garde les clés NDJSON hors manifeste — patron des seize autres appels de `strip_column` du fichier (`grep -c "strip_column("` → 19, moins la définition et les deux appels de ce temps), et l'importeur lit par `columnNames` ; retirer les clés n'éprouverait rien de plus aujourd'hui, et le test rougirait bruyamment, non en silence, si l'importeur changeait de lecture) ; B2-4 (contrôles négatifs des quatre autres clauses de `lettering_invariants` — `COUNT(*) < 2`, compte unique, somme nulle, `MIN(id)` — : ces clauses sont vraies en positif sur le scénario mêlé du test, mais aucune mutation de clause n’a été rejouée pour elles — dette de test mineure, écrite comme telle, pour partie antérieure à P1, sans effet sur le code livré) ; E2-2 (`code_of` et l'absence de `CHECK (lettering_key >= 1)` — une clé ≤ 0 ne naît que d'une écriture directe : la primitive pose `MIN(id)` d'une ligne, ≥ 1 par construction, et la restauration rétablit ce qui fut exporté ; P8 interdit de toucher la migration appliquée, une migration neuve pour une donnée inatteignable n'est pas proportionnée) ; E2-4 (`find_group` rend 500 sur une ligne dont le compte est d'une autre société — doctrine voulue par E-2, aucun chemin applicatif ne produit cette donnée ; désormais **testé**, A2-3).
- **Réversible** : oui.

## C-15-1a-i-13 — 15-1a-i (revue de code P2) : gate ciblé, gate complet reporté au redémarrage de MariaDB
- **Contexte** : la remédiation P2 touche `kesh-db/src/repositories/letterings.rs` (B2-2) — l'exception `kesh-db` du `CLAUDE.md` exige alors le gate complet même en cours de boucle ; mais le tmpfs de `kesh-mariadb-dev` est plein (C-15-1a-i-10) et l'orchestrateur interdit le gate complet.
- **Retenu** : gate ciblé — `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `nextest` sur `binary(letterings_lexical)` et les tests unitaires de `repositories::letterings` (6/6) ; le test A2-3 est compilé, non exécuté. Écrit tel quel au Dev Agent Record. Le gate complet de référence (huit threads) et l'E2E complet reviennent à l'orchestrateur après redémarrage du conteneur, sur le dernier commit de code de cette remédiation.
- **Écartées** : gate complet à deux threads comme en P1 — interdit par l'orchestrateur, et ce ne serait toujours pas le gate de référence.
- **Réversible** : sans objet (report de gate, non une décision de code).


## C-15-1a-i-14 — 15-1a-i (intégration) : rebasée sur `origin/main` `803f3e15` ; union partout, partition d'audit recomptée, statuts 15-13* de `main`
- **Contexte** : depuis `dc4bc58b`, `main` a reçu la 15-13a, la 15-7b1, la 15-13b et la 15-6c. Conflits sur le registre des choix, le sprint-status, le module `tests` d'`errors.rs`, la partition de `audit_route_registry.rs` et le PDF du manuel utilisateur. Aucune migration mergée entre-temps ; version Cargo de `main` restée `0.12.1`.
- **Retenu** : union pour le registre, le sprint-status et les tests d'`errors.rs` ; les clés 15-13* prennent le statut de `main` — **C-15-1a-i-5 est dépassé** (les statuts relevés au T0 sur les branches, dont « 15-13b review », étaient périmés) ; partition des routes recomptée, 114 = 107 tracées (105 + lettrage + délettrage) + 5 exemptées + 2 sans objet ; PDF utilisateur régénéré par `make -B fr`, les deux autres PDF rendus à leur version de `main` (`.tex` inchangés, octets d'horodatage seulement). Gate de référence à huit threads et E2E complet rejoués sur l'état rebasé.
- **Signalé, non traité** : le § « Passer à la 0.13.0 » du manuel d'administration (`admin-manual.tex`, `sec:maj-0-13`, écrit par la 15-13a sans connaître la migration du lettrage) ne dit pas que la 0.13.0 relève `kesh_version_min_required` et interdit le retour à un binaire antérieur ; seul l'avertissement général « toujours sauvegarder avant la mise à jour » le couvre. Le CHANGELOG, lui, le dit. À trancher par l'orchestrateur — candidat naturel : la 15-1a-ii, ou la préparation de la release v0.13.0.
- **Écartées** : réécrire le manuel d'administration à l'intégration (texte neuf hors de toute passe de revue, sur une story dont la boucle est close) ; reprendre les statuts 15-13* de la branche.
- **Réversible** : oui (branche de sauvegarde `backup/15-1a-i-avant-rebase-803f3e15`).
## C-15-6d-1 — 15-6d (T0) : le test 11 asserte la prop reçue par la doublure de la modale, non les options de la modale

- **Contexte** : la fiche (test 11) demande que « la modale ouverte ne propose pas le `journalAccountId` du compte bancaire monté » dans `ReconciliationProposals.test.ts`. Or ce fichier remplace `ManualMatchModal` par `ModalSuccessStub.test.svelte` depuis la revue P1 de la 15-5c (B1, E3) : les options de l'autocomplétion n'y existent pas.
- **Option retenue** : la doublure expose la prop `bankLedgerAccountId` qu'elle reçoit (attribut `data-bank-ledger-account-id`) ; le test 11 asserte la valeur résolue (le `journalAccountId` du compte monté ; `null` si `listBankAccounts` échoue, si le compte manque ou n'est pas lié). Le filtre lui-même est asserté dans la vraie modale par le test 10. Les deux tests couvrent ensemble le câblage parent → prop → filtre.
- **Alternatives** : retirer la doublure pour ce test (défait la décision de la 15-5c et oblige à piloter l'autocomplétion depuis le parent) ; un second fichier de test sans doublure (dédouble le montage pour un seul cas).
- **Réversibilité** : totale — un test et une doublure de test.

## C-15-14-1 — 15-14 : tri des P3/P4 de documentation et de libellés, et découpage d'emblée en 15-14a / 15-14b

- **Contexte** : lot décidé par Guy pour faire baisser le nombre de bugs (70 ouverts au 2026-10-09). Tri des issues
  ouvertes `bug`/`known-failure` (et `documentation`) P3/P4 sur `dc4bc58b`, chaque défaut vérifié au code.
  Retenues : #539, #547, #488, #291, #458, #449, #432, #569, #321, #323 (lot « manuels et libellés ») ; #575,
  #554, #127 (lot « exploitation »). Les secondes réécrivent le manuel d'administration et le compose de
  développement, que la 15-13a (*done*, non mergée : +187 lignes au manuel d'administration, refonte de
  `DOCKER_START.md`) et la 15-13b (*ready-for-dev*) touchent aussi — #575 porte même un complément qui n'existe
  qu'après l'AC 3 b de la 15-13a (deux lignes `MARIADB_ROOT_PASSWORD`).
- **Retenu** : découpage d'emblée en **15-14a** (manuel utilisateur, brochure, `api-external.md`, README,
  catalogues et replis — indépendante, développable tout de suite) et **15-14b** (manuel d'administration :
  Synology, volumes, multi-société ; `docker-compose.dev.yml` — **après** le merge de 15-13a, 15-13b et 15-14a).
  La fiche `15-14-lot-documentation-libelles.md` devient l'index du tri. Motif : une dépendance, non le nombre
  de modules — la 15-14a seule ne touche que trois paquets de code (kesh-i18n, kesh-api, frontend) et la 15-14b
  aucun module de production.
- **Écartées** : une story unique (bloquée tout entière par la 15-13b, encore à développer) ; une coupe par nature
  « doc / libellés » (les deux moitiés réécriraient le manuel utilisateur — #569 y a cinq sites, C-15-14-16 — et se
  disputeraient les mêmes lignes).
- **Réversible** : oui (fiches seules).

## C-15-14-2 — 15-14 : issues écartées du lot, avec leur raison

- **Contexte** : le tri a examiné toutes les P3/P4 ouvertes ; l'orchestrateur avait exclu d'office #551, #552,
  #576, #474, #524, #544, #528, #542, #279, #518 et #577.
- **Retenu — écartées** :
  - **#579** (P4, libellés du refus `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`) : B-2 exige un champ neuf dans
    la variante `DbError::SettlementCounterpartyIsClaimAccount` et chez tous ses producteurs (`kesh-db`), B-6 une
    refonte d'extraction ; C-15-6b-3 en a confié le solde à la 15-6c, qui retouche les écrans de liaison bancaire
    et est en développement dans `kesh-15-6c`. Corriger B-1/B-4 ici et laisser B-2/B-6 ne fermerait pas l'issue.
  - **#324** (P3, KF-042, « Valider ») : sa **prémisse est fausse au code** — une écriture saisie n'est pas figée,
    elle reste modifiable et supprimable tant que son exercice est ouvert (manuel utilisateur `:476-481` :
    « il n'y a rien à valider »). `Speichern`/`Salva`/`Save` disent donc vrai ; c'est le français « Valider »
    qui est le faux ami, et le changer touche une quinzaine de sélecteurs E2E et Vitest
    (`journal-entries.spec.ts`, `vat-purchase-assistant.spec.ts`, `JournalEntryForm.*.test.ts`) pour un
    arbitrage de vocabulaire qui revient à Guy. **À commenter sur l'issue** (orchestrateur).
  - **#469** (P4) : traduire les refus de validation de la route du journal d'audit demande un mécanisme
    `AppError` résolu par clé — changement de comportement, non un libellé.
  - **#339** (P4, KF-046) : 48 sites de markup en dur sur 12 fichiers — un rollout i18n, pas un lot.
  - **#504** (P4) : la langue de la communication QR dépend du contact — règle métier.
  - **#253** (P4) : contraste CSS, ni documentation ni libellé.
  - Les KF de tests et de couverture (#76, #97, #125, #126, #287, #310, #421, #424, #478, #498) et les défauts de
    comportement P3 (#293, #522, #537, #538, #546, #548, #555, #568, #578) : hors de la nature du lot.
- **Écartée** : prendre #579 en partie (une issue ne se ferme pas à moitié) ; corriger #324 dans le sens que
  propose l'issue (`Buchen`/`Post`), qui ferait dire aux cibles l'inverse de ce que fait le bouton en édition.
- **Réversible** : oui.

## C-15-14-3 — 15-14a : les fonctions promises et absentes se retirent du manuel, elles ne se livrent pas ici

- **Contexte** : #291 (« Contacts → Import CSV ») laisse l'arbitrage au Project Lead entre retirer et livrer ; la
  vérification de #488 révèle une seconde fonction fictive du même type, « Administration → Plan comptable →
  Import CSV » (`user-manual.tex:409-422`, ligne « Personnalisé CSV » du tableau `admin-manual.tex:1369`) — aucune
  route ni écran d'import de plan. #458 (« dossier surveillé ») hésite entre corriger le texte et automatiser
  l'import, automatisation que suit déjà #459.
- **Retenu** : retirer les deux sous-sections d'import fictives et la promesse de surveillance ; le texte décrit
  ce qui existe (plans choisis par le type d'organisation, comptes ajoutés un par un ; import déclenché à la main).
  #458 se ferme sur la correction du texte, `refs #459` pour l'automatisation.
- **Écartées** : livrer un import de contacts ou de plan (fonctionnalité, hors lot) ; garder le texte « en
  attendant » (il promet ce qui n'existe pas, défaut que l'issue nomme).
- **Réversible** : oui — si l'une de ces fonctions est livrée, sa story réécrit la section.

## C-15-14-4 — 15-14a : formulation des prescriptions de réouverture (#569)

- **Contexte** : six clés (×4 locales), six replis Rust, quatre replis frontend, quatre phrases du manuel (cinq : C-15-14-16) et une
  ligne d'`api-external.md` prescrivent « rouvrir l'exercice » sans l'ordre LIFO. La 15-12b a en outre renvoyé à
  #569 (B-1 = E-1) la clause « une écriture existante se corrige par une contre-passation » de
  `error-later-fiscal-year-closed`, qu'un exercice du jour suivi d'un exercice clôturé ferait refuser.
- **Retenu** : la prescription devient « rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le
  plus récent » (forme de C111, « jusqu'à celui-ci » ajouté parce que ces messages désignent l'exercice à
  atteindre). La clause de contre-passation de `error-later-fiscal-year-closed` et de
  `journal-entries-modify-blocked-later-fiscal-year-closed` est **examinée et conservée** : l'exercice du jour
  suivi d'un exercice clôturé n'existe que dans l'état hérité, que l'écran des exercices signale par son propre
  bandeau, et l'alternative que donne déjà le message (« sinon, un administrateur rouvre … en commençant par le
  plus récent ») reste juste dans ce cas. Les corps de confirmation (« seul un administrateur peut le rouvrir »,
  « Vous êtes sur le point de rouvrir l'exercice ») décrivent, ils ne prescrivent pas : hors inventaire.
- **Écartées** : nommer l'exercice le plus récent dans chaque message (argument neuf à porter par six
  producteurs) ; réécrire la clause de contre-passation (elle alourdirait un message juste dans l'état sain).
- **Réversible** : oui.

## C-15-14-5 — 15-14a : « Réglages » devient « Paramètres » partout où le manuel ou un message désigne l'écran

- **Contexte** : #547. Le menu affiche « Paramètres » (`nav-settings`). Le manuel utilisateur écrit « Réglages »
  quatre fois (`:575`, `:907`, `:952`, `:2085`) ; la propagation trouve un cinquième site, un **message** :
  `invoice-default-revenue-account-unusable` (« corrigez-le dans les Réglages ») et son repli
  (`InvoiceForm.svelte:768`).
- **Retenu** : les quatre phrases du manuel et le message (4 locales, chemin complet `Paramètres → Facturation`
  aligné sur `settings-invoicing-title` de chaque locale, repli identique au catalogue `fr-CH`). Restent assumés :
  les noms d'entité du journal d'audit (« Réglages de facturation », « Réglages de recouvrement » — des noms
  d'objet, non des renvois au menu), les commentaires de code, et la ligne historique `README.md:213` (feuille de
  route publiée).
- **Écartée** : renommer aussi les entités d'audit (vocabulaire du journal, autre domaine).
- **Réversible** : oui.

## C-15-14-6 — 15-14a : les références d'issues du README deviennent des liens explicites (#432)

- **Contexte** : #432 propose d'uniformiser sur `#NNN` nu, « auto-lié par GitHub ». Vérifié par l'API de rendu
  de GitHub (`POST /markdown`) : en mode `markdown` (rendu de fichier sans contexte de dépôt), **ni** `(#195)`
  **ni** `[#164]` ne deviennent des liens ; en mode `gfm` avec contexte, **les deux** le deviennent (les crochets
  restant affichés). La prémisse « l'une est cliquable, l'autre non » ne tient donc dans aucun des deux modes.
- **Retenu** : toute référence d'issue du README (54 occurrences, lignes 211-223 sur `dc4bc58b`) s'écrit
  `[#NNN](https://github.com/guycorbaz/kesh/issues/NNN)` — la forme du CHANGELOG, lien dans tous les modes ; un
  test l'impose.
- **Écartées** : `#NNN` nu (non lié si le rendu de fichier est en mode `markdown`) ; des définitions de référence
  en pied de fichier (une oubliée redevient du texte sans signal — l'issue elle-même l'écarte).
- **Réversible** : oui.

## C-15-14-7 — 15-14a : vocabulaire de la clôture d'exercice dans les trois cibles (#323)

- **Contexte** : `fiscal-year-close-button` et `fiscal-year-close-confirmation-action` portent `Schliessen` /
  `Chiudi` / `Close`, le verbe des panneaux (`api-keys-actions-close`, `supplier-invoices-form-close`,
  `payment-batches-form-close`, `reconciliation-cancel-dismiss`).
- **Retenu** : `Abschliessen` (de-CH, verbe déjà employé par les messages de clôture dans l'ordre :
  « Schliessen Sie … ab ») ; `Chiudi l’esercizio` (it-CH) ; `Close fiscal year` (en-CH) ; titres de confirmation
  alignés (`Geschäftsjahr abschliessen?`). Le français ne change pas (la suite E2E tourne en français : aucun
  sélecteur touché). Un test interdit que le libellé de la clôture égale celui d'un panneau.
- **Écartée** : `Jahresabschluss` (substantif sur un bouton d'action) ; garder `Close` seul en anglais.
- **Réversible** : oui.

## C-15-14-8 — 15-14 : une garde testée lit la documentation

- **Contexte** : « aucun gate ne lit le manuel » (rétrospective de l'Epic 24) ; les dix défauts de ce lot sont des
  phrases que rien n'a contrôlées. Un test qui fige un texte de catalogue existe (parité, replis) ; aucun ne lit
  `docs/manual/`, `README.md` ni `api-external.md`.
- **Retenu** : un fichier de test pur, sans base, `crates/kesh-api/tests/textes_coherents.rs` (nom rectifié en
  validation P1, C-15-14-16), qui lit ces
  fichiers et y interdit les valeurs corrigées par le lot (et, pour les taux de TVA, un test de module de
  `vat_rates.rs` qui confronte le manuel à `DEFAULT_SWISS_RATES`). La 15-14b l'étend. Chaque assertion se prouve
  par mutation (valeur réintroduite → rouge).
- **Écartées** : se fier au grep du développeur (c'est le régime qui a laissé passer ces défauts) ; un script hors
  gate (rien ne le lancerait).
- **Réversible** : oui (un fichier de test).

## C-15-14-9 — 15-14b : le compose de développement démarre sans `.env` par l'écran `/setup` (#554)

- **Contexte** : `docker-compose.dev.yml:25` pose `${KESH_ADMIN_PASSWORD:-admin}` (5 caractères), refusé par
  `WeakAdminPassword`. L'issue admet deux remèdes : un défaut de 12 caractères au moins, ou pas de défaut.
- **Retenu** : `${KESH_ADMIN_PASSWORD:-}` — absent ou vide, l'administrateur se crée à `/setup`, comme avec
  `docker-compose.prod.yml` ; aucun mot de passe publié dans un fichier du dépôt.
- **Écartée** : un défaut long « de développement » (un mot de passe publié de plus — précisément ce que la 15-13a
  retire de `docker-compose.yml`).
- **Réversible** : oui.

## C-15-14-10 — 15-14b : le pré-script Hyper Backup vise la base de `DATABASE_URL`, par un conteneur jetable

- **Contexte** : #575. `docker-compose.prod.yml` n'a pas de service `mariadb` : la base est celle de l'exploitant
  (paquet DSM, conteneur séparé, base gérée). Le pré-script du manuel fait `docker compose exec -T mariadb` et lit
  `MARIADB_ROOT_PASSWORD` dans `.env` par `grep | cut` — deux lignes possibles après la 15-13a (complément de
  l'issue).
- **Retenu** : le pré-script exécute `mariadb-dump` dans un conteneur jetable `mariadb:10.11` attaché au réseau
  `frontend`, avec un fichier d'identifiants dédié (`--defaults-extra-file`, droits 600, hors du `.env`), contre
  l'hôte et la base que nomme `DATABASE_URL`. Le manuel ne demande plus d'écrire `MARIADB_ROOT_PASSWORD` dans le
  `.env` d'une installation Synology.
- **Écartées** : le client du paquet MariaDB de DSM (chemin propre à la version du paquet, absent si la base est
  ailleurs) ; garder `grep .env` (deux lignes, et un secret root dans un fichier lu par Compose).
- **Réversible** : oui.

## C-15-14-11 — 15-14a (validation P1) : « réglages » en minuscule est un nom commun, sauf deux renvois à l'écran

- **Contexte** : R3 ≈ F-11. L'inventaire de #547 était relevé par un grep sensible à la casse. Le minuscule
  « réglages » apparaît 22 fois hors commentaires de code *(faux : 28 lignes, 26 assumées — rectifié par
  C-15-14-17)* (catalogue fr-CH, replis Rust, deux manuels), et deux
  renvois à l'écran avaient échappé : fr-CH `error-invoice-pdf-header-overflow` (« Supprimez une coordonnée …
  dans les réglages », où de-CH, it-CH et en-CH nomment le menu) avec son repli `errors.rs:1807`, et
  `.env.example:305` (« (Réglages) », fichier hors du périmètre du grep).
- **Retenu** : ces deux renvois deviennent « Paramètres » ; les 22 *(26 : C-15-14-17)* autres sont le **nom commun** — les valeurs
  configurées (« le compte de TVA due désigné dans les réglages », « les trois réglages sont vides ») — et entrent
  à l'inventaire des non résolus de l'AC 2, avec les notes de versions publiées du CHANGELOG (`:347`, `:433`,
  `:451`). T2, T3 et T9 gardent les deux renvois corrigés.
- **Écartées** : remplacer tout « réglages » par « paramètres » (réécriture de style sans défaut, 22 sites dont
  des replis Rust gardés par test) ; laisser `error-invoice-pdf-header-overflow` (même symptôme que #547, seul
  écart de la locale fr avec les trois autres).
- **Réversible** : oui (texte).

## C-15-14-12 — 15-14a (validation P1) : « dossier surveillé » se corrige aussi dans `.env.example` ; « scruté à l'import » reste

- **Contexte** : F-4, R7. `.env.example:197` (sur `bcded0c8`) écrit « déposées dans un dossier surveillé » —
  le fichier que l'exploitant lit pour configurer l'inbox. `admin-manual.tex:781` et `.env.example:204` disent
  « Dossier inbox scruté à l'import ».
- **Retenu** : `.env.example:197` corrigé et gardé par T5 ; « scruté à l'import » conservé — il dit que le dossier
  est lu au moment de l'import, ce qui est exact (l'import, lancé à la main, parcourt le dossier). Les notes de la
  0.4.0 (`CHANGELOG.md:499`) sont historiques, assumées.
- **Écartée** : réécrire « scruté à l'import » (pas de défaut : la phrase ne prête à l'import aucun déclenchement).
- **Réversible** : oui.

## C-15-14-13 — 15-14a (validation P1) : le test des taux de TVA vit dans un `mod tests` neuf de `vat_rates.rs`, et ce choix coûte le gate complet

- **Contexte** : F-6 ≈ R6. La fiche plaçait T1 « dans le `mod tests` de `vat_rates.rs` » ; ce module n'existe
  pas. `DEFAULT_SWISS_RATES` est un `const` privé. Trois lieux possibles : (a) un `#[cfg(test)] mod tests` neuf
  dans `vat_rates.rs` ; (b) rendre la constante `pub` et tester depuis `textes_coherents.rs` (kesh-api) ; (c) un
  fichier `kesh-db/tests/`, qui exige aussi une constante publique.
- **Retenu** : (a). Aucun changement de visibilité ni de code de production ; précédents du même répertoire
  (`accounts.rs:1136`, `bank_profiles.rs:343`). Le test est un `#[test]` pur, **pas** un `#[sqlx::test]` :
  `test_schema_guard.rs` ne recense que ces derniers (`TOKEN = "#[sqlx::test"`) et ne le voit pas. Il lit
  `docs/manual/fr/user-manual.tex` par `env!("CARGO_MANIFEST_DIR")/../../` — couplage d'un crate de
  persistance à la documentation, accepté : kesh-db n'est pas publié (seul `kesh-import` l'est).
  **Rayon du gate** : `vat_rates.rs` est un repository — l'**exception `kesh-db`** du `CLAUDE.md` s'applique :
  tout patch qui touche ce fichier, y compris en boucle de revue, impose le gate complet ; un patch limité à
  `textes_coherents.rs`, `loader.rs` ou aux catalogues relève du gate ciblé. La mutation `380 → 370` touche la
  constante de production : restaurée par `git checkout` puis `touch`.
- **Écartées** : (b) et (c) — modifier la visibilité d'une constante de production pour un test de
  documentation ; dupliquer les taux dans le test (une règle recopiée peut diverger, mémoire *Tests qui prouvent
  moins*).
- **Réversible** : oui (un module de test).

## C-15-14-14 — 15-14a (validation P1) : le domaine du test des prescriptions de réouverture se prend en fr-CH, et se contrôle dans les quatre locales

- **Contexte** : R5 ≈ F-2. T8 demandait « le verbe de réouverture de la locale » sans le donner. En de-CH, ce
  verbe prend au moins cinq formes (« wieder öffnen », « öffnen Sie dieses zuerst », « Öffnen Sie es wieder »,
  « Wiedereröffnung », « wieder eröffnet ») ; un motif par locale passait à vide sur la clé même que la story
  corrige (`error-fiscal-year-reopen-blocked`, de-CH « öffnen Sie dieses zuerst »), et un motif large (`öffn`)
  ramasse « Detailansicht öffnen », « Einstellungen öffnen ». La liste d'exemptions ne couvrait pas les clés
  `-reopen-*` qui nomment l'acte, ni `error-reminder-amounts-changed` hors fr.
- **Retenu** : le domaine est l'ensemble des clés dont la valeur **fr-CH** matche `[Rr]ouvr|[Rr]éouv` — univoque
  en français ; 20 clés sur `bcded0c8`. Pour chacune, hors exemptions, **chaque** locale doit porter son marqueur
  d'ordre. Exemptions **par clé**, liste fermée de 10 (six qui nomment l'acte, deux corps qui décrivent, la clé
  qui nomme l'exercice à rouvrir, `error-reminder-amounts-changed`) ; partition recomptée 6 + 4 + 10 = 20. Une
  mutation de-CH s'ajoute à la mutation fr.
- **Écartées** : un motif par locale (ouvert par nature : une forme imprévue le contourne — § *Inventorier les
  sites NON RÉSOLUS*) ; des exemptions par clé × locale (la parité des catalogues rend la clé suffisante).
- **Limite assumée** : une valeur non française qui prescrirait la réouverture sous une clé dont la valeur fr-CH
  ne la prescrit pas échapperait au test — la parité des catalogues rend ce cas improbable.
- **Réversible** : oui.

## C-15-14-15 — 15-14b (validation P1) : le pré-script Hyper Backup dit ses conditions de fonctionnement

- **Contexte** : R14, F-10. Le pré-script de C-15-14-10 ne disait ni l'ordre des options de `mariadb-dump`, ni
  la forme du mot de passe (celui de `DATABASE_URL` est pourcentage-encodé), ni les conditions réseau du paquet
  MariaDB de DSM, ni le cas d'une image absente hors ligne. Aucune n'a été rejouée par les lentilles (Docker
  interdit en lecture seule).
- **Retenu** : l'AC 1 les prescrit au manuel et à la recette du T1 — `--defaults-extra-file` en première option ;
  mot de passe décodé et entre guillemets dans le fichier d'options ; port 3307, accès TCP et pare-feu du paquet
  DSM, `localhost` qui désigne le conteneur ; `docker pull mariadb:10.11` à la mise en place, et l'échec visible
  de la tâche s'il manque (voulu). Ce qui relève de la documentation Synology et non de la recette est écrit
  comme tel.
- **Écartée** : renvoyer ces points à la recette sans les écrire (le manuel est ce que l'exploitant suit).
- **Réversible** : oui.

## C-15-14-16 — 15-14 (validation P1) : rectificatifs et réalignement sur `bcded0c8`

- **Rebase** : la 15-13a est mergée (`bcded0c8`, #551 fermée) entre la passe et sa remédiation ; la branche est
  rebasée (sauvegarde `backup/15-14-avant-rebase-bcded0c8`), `sprint-status.yaml` résolu par union (ligne
  d'en-tête de la 15-14 renumérotée (41)), registre fusionné sans conflit. Numéros de ligne réalignés dans les
  deux fiches ; ceux de la spécification restent entre parenthèses. La 15-14b ne dépend plus que de la 15-13b et
  de la 15-14a.
- **Rectificatif de C-15-14-8** : le fichier de test se nomme `crates/kesh-api/tests/textes_coherents.rs`, comme
  le disent les deux fiches — non `documentation_coherente.rs`. C-15-14-8 est corrigé en place, avec renvoi ici.
- **Rectificatif de C-15-14-1 et C-15-14-4** : #569 a **cinq** sites au manuel utilisateur, non quatre
  (`user-manual.tex:1208-1209` trouvé par R1 = F-1).
- **Rectificatif de C-15-14-3** : la ligne « Personnalisé CSV » est à `admin-manual.tex:1383` sur `bcded0c8`
  (`:1369` sur `dc4bc58b`).
- **`user-manual.tex:182`** (R12) : corrigé par la 15-14a sans condition, texte identique à celui de la 15-7b1
  (non mergée), pour que le conflit de rebase se résolve en gardant l'un ou l'autre.
- **Réversible** : oui.

## C-15-14-17 — 15-14 (validation P2) : un inventaire s'écrit comme une commande comptée et partitionnée, PDF normalisé, `docs/user-guide/` compris

- **Contexte** : trois MEDIUM de la validation P2 sont nés de la remédiation P1, tous des inventaires déclarés
  complets qui ne l'étaient pas (R-1 = F-5 « le seul autre site de-CH » ; R-2 = F-6 « 22 » réglages, 26 en
  réalité ; F-8, la liste de formes de G5 étendue par la P1, qui laisse passer les affirmations réécrites) ;
  un quatrième, manqué depuis la spécification, est de même nature (R-3 = F-2, Snapshot Replication hors
  inventaire de #575). F-1 : le contrôle aplati des PDF prescrit
  (`pdftotext | tr | grep -F`) passe à vide sur toute apostrophe (`'` y devient `’`) et sur les traits d'union
  coupés — vert avant correction. F-4 : `docs/user-guide/fr/getting-started.md:29`, lié depuis le README,
  promettait KMU, Verein et un import de plan, hors de tout périmètre de grep.
- **Retenu** : chaque AC qui corrige une famille de textes écrit la **commande** d'inventaire, son **compte** sur
  `bcded0c8` et la **partition** corrigé / assumé-avec-raison, recalculable ; le T0 relance chaque commande et
  ventile tout écart avant d'écrire. Le contrôle des PDF passe par une fonction `occ` qui normalise PDF **et**
  motif (apostrophes, ligatures, espaces insécables, espaces et traits d'union retirés), et chaque contrôle
  d'absence est précédé du même contrôle de présence sur le PDF d'avant (≥ 1). `docs/user-guide/` entre dans le
  périmètre commun ; la ligne 29 du guide est corrigée (15-14a, AC 3) et gardée par G4. Rectificatif de
  C-15-14-11 : **26** lignes « réglages » minuscules assumées (28 − 2), non 22 — `user-manual.tex:364`, `:380`,
  `:394` manquaient ; `:965` (« reste intacte dans vos réglages ») est assumé, il ne prescrit aucun geste. Les
  tests sont renommés **G1-G12** (15-14a) et **G1-G5** (15-14b) pour ne plus se confondre avec les tâches
  T0-T8 (R-14).
- **Écartées** : recopier des listes de sites (c'est ce qui a produit les trois MEDIUM) ; contrôler les PDF sur
  `pdftotext` brut ; laisser `docs/user-guide/` hors périmètre (document vivant, lié depuis le README).
- **Réversible** : oui (fiches).

## C-15-14-18 — 15-14a (validation P2) : en de-CH, l'état d'un exercice clôturé se dit « abgeschlossen », comme son action

- **Contexte** : R-1 = F-5. L'AC 10 corrigeait le verbe du bouton et réécrivait déjà deux participes
  (« geschlossen bleibt », « ist geschlossen ») ; six autres valeurs de-CH gardaient « geschlossen » /
  « Schliessung » pour un exercice clôturé, dont le toast qui suit immédiatement le clic sur « Abschliessen ».
  Inventaire `\b([Gg]eschlossen|[Ss]chliessen|[Ss]chliessung)\b` : 20 lignes.
- **Retenu** : corriger les six (`:861`, `:869`, `:873`, `:877`, `:890`, `:893` → « abgeschlossen », « erneuten
  Abschluss ») — même critère que les deux participes déjà réécrits, glossaire « clôture = Abschluss », forme
  déjà employée au catalogue (`:241`, `:288`, `:2224`). Partition 5 + 1 + 6 + 4 + 4 = 20 ; restent 8 lignes,
  verbe séparable « Schliessen Sie … ab » (4) et fermeture de panneau (4). G11 garde la partition.
- **Écartée** : n'inventorier les six qu'en « assumés » (le participe d'état est défendable, mais la story
  corrige déjà ce même participe ailleurs : deux traitements pour un même cas). Aucun test ni repli ne les fige.
- **Réversible** : oui (catalogue).

## C-15-14-19 — 15-14 (validation P2) : dépendance déclarée envers la 15-7b1, qui corrige les mêmes lignes

- **Contexte** : F-7, L-8. La 15-7b1 (PR #583, en cours de merge) réécrit `user-manual.tex:173`-`:189` — dont
  `:182` (« Sterchi PME », AC 3 de la 15-14a) et les deux « nouvelle company » (`:173`, `:189`) que F-7 trouve
  hors de l'AC 3 de la 15-14b — et modifie un doc-comment de `vat_rates.rs`, où la 15-14a ajoute un `mod tests`.
- **Retenu** (consigne de l'orchestrateur) : ne pas doubler son travail. La 15-14a et la 15-14b se développent
  **après** son merge ; le T0 constate les sites corrigés. Repli de la 15-14a si elle n'était pas mergée : texte
  identique au sien à `:182` (G4 interdit `Sterchi`). Rebase sur `vat_rates.rs` : gate complet (exception
  `kesh-db`). Ses deux « réglages de facturation » ajoutés portent l'inventaire (B) de l'AC 2 à 30.
- **Écartée** : réécrire `:173`/`:189` dans la 15-14b (conflit certain, travail fait deux fois).
- **Réversible** : oui.

## C-15-14-20 — 15-14a (validation P2) : deux verbes de réouverture coexistent en de-CH

- **Contexte** : L-3. Les quatre clés qui portent déjà le marqueur d'ordre écrivent « eröffnet eine
  Administratorin oder ein Administrator … wieder » ; les six de l'AC 8 écriront « muss ein Administrator …
  wieder öffnen » (validation P1, F-2 : le verbe du bouton `Wieder öffnen`).
- **Retenu** : laisser coexister, chaque clé gardant sa forme ; le marqueur « beginnend mit dem neuesten », seul
  contrôlé par G8, est identique.
- **Écartée** : harmoniser les quatre autres clés (hors #569, et réécriture de style sans défaut).
- **Réversible** : oui.

## C-15-14-21 — 15-14b (validation P2) : l'AC 1 est écrit pour l'état après la 15-13b, et la base se restaure par son dump

- **Contexte** : F-3 — la 15-13b (*done*, PR #584) monte `./backup` (secrets) dans les deux compose : « trois
  montages » et la liste Hyper Backup de l'AC 1 seraient faux à son merge. R-3 = F-2 — la sous-section Snapshot
  Replication et la copie 1 du 3-2-1 promettent de restaurer des écritures que le snapshot du dossier du compose
  ne contient pas, et la procédure de recovery ne recharge aucun dump.
- **Retenu** (consigne de l'orchestrateur pour F-3) : texte cible pour l'état **après** la 15-13b — quatre
  montages, `backup/` coché avec sa mise en garde « secrets » ; numéros de `bcded0c8`, réalignés au T0 (la 15-13b
  décale de +4, même contenu vérifié sur sa branche). Snapshot Replication, copie 1, introduction (« remplacent »)
  et « cohérence transactionnelle » réécrits : le dossier du compose ne contient pas la base ; elle se restaure
  par le dump du pré-script, rechargé par le même conteneur jetable ; la recette rejoue la recovery ; G3 exige
  que la sous-section Snapshot cite le fichier de dump. Inventaire borné aux deux sections : 34 lignes, 13 + 21.
- **Écartées** : écrire contre l'état de `bcded0c8` (faux au merge de la dépendance) ; formuler la liste par
  la seule commande `grep` sur le compose (moins lisible pour l'exploitant) ; laisser Snapshot Replication en
  angle mort (c'est le symptôme même de #575).
- **Réversible** : oui (texte).

## C-15-14-22 — 15-14b (validation P2) : un commentaire de `docker-compose.prod.yml` se corrige

- **Contexte** : R-4. `docker-compose.prod.yml:112-113` dit le dossier `./log` « co-localisé avec .env + DB pour
  le scope unique Hyper Backup » — même faux que `admin-manual.tex:1575` ; la fiche s'interdisait ce fichier.
- **Retenu** : autoriser cette seule ligne de commentaire (aucune clé, aucun montage) ; `configuration_transmise`
  et l'étape CI « Validate compose files » rejouées au T7.
- **Écartée** : l'inventorier comme assumé — c'est le fichier que l'exploitant télécharge et lit.
- **Réversible** : oui.

## C-15-14-23 — 15-14 (validation P2) : signal D5 levé, pas de nouveau découpage

- **Contexte** : la validation P2 rend **9 MEDIUM distincts** (12 bruts, recoupements R-1 = F-5, R-2 = F-6,
  R-3 = F-2) contre 8 en P1 : sévérité égale (MEDIUM → MEDIUM), et **trois** sont nés de la remédiation P1
  (F-5, F-6, F-8 — inventaires ou listes déclarés complets) — recyclage au sens de l'amendement D5 de la
  § *Règle de splitting préventif*.
- **Retenu** (décision de l'orchestrateur) : **pas** de nouveau découpage. La story est déjà coupée en 15-14a /
  15-14b, et le recyclage porte sur la **complétude des inventaires**, que C-15-14-17 traite à la racine
  (commandes comptées et partitionnées, recalculables). Signal déclaré au Change Log.
- **Écartée** : découper encore (par manuel, ou par issue) : la cause — des listes recopiées au lieu de
  commandes — se reproduirait dans chaque morceau.
- **Réversible** : oui (une passe P3 qui verrait recycler un inventaire écrit selon C-15-14-17 rouvrirait la
  question).

## C-15-14-24 — 15-14b (validation P2) : le test du multi-société porte sur un domaine fermé, non sur des formes

- **Contexte** : F-8. G5 interdisait six formes ; les affirmations réécrites par l'AC 3 en ont d'autres
  (« plusieurs sociétés (companies) sur une même instance », « plusieurs sociétés\n peuvent coexister sur une
  même instance ») — remises en place, aucune ne le faisait rougir. Et `compte dédié`, interdit partout,
  aurait rougi sur un emploi légitime (« un compte MariaDB dédié » de l'AC 1).
- **Retenu** : domaine calculé sur le texte normalisé (motif de l'AC 3, 26 occurrences sur `bcded0c8`) ; toute
  occurrence doit tomber dans une liste fermée de 7 fragments assumés, chacun encore présent ; `compte dédié`
  n'entre au domaine que dans `ou un compte dédié` / `via un compte dédié`, et une contre-mutation (« un compte
  dédié aux frais bancaires ») doit rester verte.
- **Écartée** : allonger la liste de formes interdites (ouverte par nature).
- **Réversible** : oui.

## C-15-14-25 — 15-14 (validation P3) : un inventaire porte sur tout le dépôt suivi, et c'est l'exclusion qui se justifie

- **Contexte** : F-1 (MEDIUM) et R-3 = F-2 (MEDIUM). Les comptes de la remédiation P2 se recomptent à
  l'identique ; le défaut est le **périmètre** des commandes. (B) « réglages » était bornée à quatre
  fichiers (`README.md:34`, un renvoi à l'écran où l'on agit, lui échappait) ; l'inventaire de l'AC 1 de la
  15-14b était borné par un `awk` aux deux sections Synology (`admin-manual.tex:2475-2476`, « Hyper Backup
  au niveau NAS » comme sauvegarde de la base, et `:1748` lui échappaient). Dans les deux cas, la commande
  choisissait d'avance où regarder. Fait nouveau : la 15-7b1 est mergée (`origin/main = 245b91ee`).
- **Retenu** : chaque commande d'inventaire est un `git grep -I` sur **tout le dépôt suivi**, moins un
  ensemble d'exclusions commun `E` écrit **une fois**, avec la raison de chaque exclusion (convention de la
  15-14a : `_bmad-output/`, `_bmad/`, `.claude/`, `CLAUDE.md`, `CHANGELOG.md`, `.svelte-kit/`, spécification
  et PRD datés, fichiers archivés, tests) ; `LC_ALL=C.UTF-8` forcé. Pour l'AC 1 de la 15-14b, plus de borne
  de section : le manuel d'administration **entier** (144 lignes, partitionnées par section — 43 dans les
  sections Synology, 101 ailleurs, dont 6 corrigées), plus le reste du dépôt pour le symptôme de #575 (5).
  Toutes les commandes relancées sur `245b91ee` : 15-14a AC 1 : 4 ; AC 2 : 21 et 132 ; AC 3 : 16 ; AC 5 :
  25 ; AC 6 : 5 ; AC 8 : 153 ; AC 9 : 6 ; AC 10 : 21 ; 15-14b AC 3 : 477. Branche rebasée sur `245b91ee`
  (sauvegarde `backup/15-14-avant-rebase-245b91ee`, registre et sprint-status par union). Les **numéros de
  ligne** restent ceux de `bcded0c8`, avec la table des décalages de la 15-7b1 (`user-manual.tex` +1 à
  partir de 190, +15 à partir de 2243 ; `CHANGELOG.md` +1 à partir de 42 ; `vat_rates.rs` +1 à partir de
  353) — un réalignement partiel mêlerait deux bases dans les mêmes listes, et le T0 re-trouve chaque site
  par la valeur.
- **Écartées** : élargir chaque commande d'un cran (`+ README.md`, `+ frontend/src`…) — c'est recommencer
  l'énumération qui a manqué ; réaligner tous les numéros sur `245b91ee` (la 15-13b, en cours
  d'intégration, les décalera de nouveau).
- **Réversible** : oui (fiches seules).

## C-15-14-26 — 15-14a (validation P3) : les replis frontend à site unique ont leur garde, G13

- **Contexte** : R-1 (MEDIUM). La fiche disait le repli d'`InvoiceForm.svelte` et celui des soldes de départ
  « gardés par `i18n-repli-divergent-actif.test.ts` ». Faux au code : cette garde ne retient que les clés à
  **au moins deux** replis distincts (`parTexte.size > 1 && auCatalogue.has(cle)`, ligne 141) ; les cinq clés
  visées n'ont qu'un site d'appel chacune — trois replis (`invoice-cancel.ts`, `opening-balances/+page.svelte`,
  `InvoiceForm.svelte`) n'avaient aucun test.
- **Retenu** : G13, un `describe` neuf du même fichier, qui réutilise son relevé `replisParCle()` : pour une
  table fermée de cinq clés, l'ensemble des replis égale `[valeur fr-CH]` (même patron que l'assertion des
  deux titres d'avoir, ligne 169). Mutations : ancien repli d'`invoice-cancel.ts:43`, « Réglages » remis à
  `InvoiceForm.svelte:768`, ancien repli des soldes de départ → rouges. L'affirmation fausse est retirée
  partout (AC 2, AC 8, mutations).
- **Écartées** : étendre G9 (Rust) aux fichiers frontend — il faudrait réécrire un lecteur de littéraux
  TS/Svelte que `i18n-literal-reader.js` fournit déjà ; retirer l'affirmation sans test — T4 « replis égaux
  au fr-CH » n'aurait plus de vérification pour trois de ses sites.
- **Réversible** : oui.

## C-15-14-27 — 15-14 (validation P3) : `regex` en dépendance de test de kesh-api et kesh-i18n

- **Contexte** : L-1 (LOW). G2-G7, G9, G12 et G3, G8, G10, G11 sont écrits en motifs ; ni `kesh-api` ni
  `kesh-i18n` n'ont le crate `regex` ; G18 (ex-G5 de la 15-14b) compte des occurrences qui se recouvrent,
  ce que la commande perl fait par lookahead — absent du crate `regex`.
- **Retenu** : `regex = "1.10"` en `[dev-dependencies]` des deux crates (version de `kesh-db` et
  `kesh-import`, déjà dans `Cargo.lock`), à la T5 de la 15-14a ; G18 cherche chaque alternative séparément.
- **Écartée** : se passer de `regex` (motifs réécrits à la main, plus fragiles) ; `fancy-regex` (dépendance
  neuve pour un seul test).
- **Réversible** : oui.

## C-15-14-28 — 15-14b (validation P3) : le pré-script se passe de `--routines`/`--events`, et son compte de `SELECT` et `LOCK TABLES`

- **Contexte** : L-4 (LOW). La fiche exigeait `SHOW VIEW`, `TRIGGER`, `EVENT` et une vérification de
  `--routines` sur 10.11 ; le schéma de Kesh n'a ni routine, ni déclencheur, ni événement, ni vue (`grep`
  sur les migrations → 0). La restauration depuis Hyper Backup (copies 2 et 3 du 3-2-1) n'avait pas de
  procédure ; et « Hyper Backup ne voit pas la base » ignorait que DSM permet de cocher le paquet MariaDB.
- **Retenu** : `--single-transaction --add-drop-database --databases <base>`, compte à `SELECT` + `LOCK
  TABLES`, prouvé par la recette avec un compte qui n'a que ces privilèges ; angle mort écrit (une routine,
  un déclencheur, un événement ou une vue ajoutés un jour rendraient le dump incomplet sans bruit).
  Restauration Hyper Backup : renvoi à la procédure de rechargement de la recovery Snapshot. Le manuel dit
  qu'Hyper Backup ne sauvegarde pas la base **au titre du dossier du compose**, la copie applicative du
  paquet DSM, non vérifiable ici, s'ajoutant au dump sans le remplacer.
- **Écartée** : garder `--routines --events` et leurs privilèges « par précaution » (privilèges inutiles,
  et une vérification sur 10.11 que personne ne rejoue).
- **Réversible** : oui.

## C-15-14-29 — 15-14b (validation P3) : le pré-script écrit son dump dans un `.tmp`, puis le renomme

- **Contexte** : F-5 (LOW). `… | gzip > kesh_pre_backup.sql.gz` ouvre la cible avant que le dump ne
  réussisse : un échec la tronque, et le snapshot horaire capture le fichier vide à la place du dump de la
  veille.
- **Retenu** : écriture dans `kesh_pre_backup.sql.gz.tmp`, `mv` après succès, empreinte sur le fichier
  renommé ; G16 extrait le nom du dump de la cible du `mv` et exige le couple `.tmp` + `mv` (mutation :
  `| gzip >` directement sur la cible → rouge).
- **Écartée** : garder le comportement actuel (« la tâche échoue, c'est voulu ») — l'échec est voulu, la
  perte de la dernière copie saine ne l'est pas.
- **Réversible** : oui.

## C-15-14-30 — 15-14b (validation P3) : les formes « toutes les sociétés », « chaque company » restent hors du motif de G18

- **Contexte** : L-6 et F-8 (LOW). Le domaine fermé de G18 reste défini par un motif ; des formes voisines
  lui échappent (`admin-manual.tex:987`, `:1393`, `:1440`, `:1663`, `README.md:48`, la clé
  `admin-backup-page-description` et son repli, la brochure `:97`, `:314`).
- **Retenu** : relues une à une, aucune n'est fausse pour une installation à une société (définition du
  modèle, contenu d'une sauvegarde, ciblage commercial compatible avec « une instance par dossier ») :
  écrites en liste « hors motif, assumés » dans l'AC 3, pour qu'une revue la conteste ; motif inchangé.
- **Écartée** : élargir le motif (`chaque company`, `toutes les sociétés`…) — G18 rougirait sur des phrases
  vraies, et chaque forme ajoutée en appellerait une autre.
- **Réversible** : oui.

## C-15-14-31 — 15-14 (validation P3) : les gardes de la 15-14b deviennent G14-G18

- **Contexte** : F-6 et une remarque de la lentille R (LOW). Les deux fiches numérotaient leurs gardes à partir
  de G1 ; « G5 » désignait `aucun_dossier_surveille` (15-14a) et `une_installation_une_societe` (15-14b),
  qui vivent dans le **même** fichier `textes_coherents.rs`.
- **Retenu** : la 15-14a garde G1-G13 (G13 neuf, C-15-14-26) ; la 15-14b passe à G14-G18 (G1 → G14 … G5 →
  G18), correspondance écrite en tête de ses tâches ; les Change Logs antérieurs gardent leurs numéros.
- **Écartée** : préfixer (`15-14b-G5`) — plus long à chaque citation, pour le même effet.
- **Réversible** : oui.

## C-15-14-32 — 15-14a (validation P4) : validation close

- **Contexte** : la P4 (Opus ×2) ne rend **aucun MEDIUM** sur la 15-14a ; ses LOW (R L-2, L-5, L-7, L-8, F
  L-2) sont justes et mineurs. Trend du lot 8 → 9 → 4 → 3 MEDIUM, les trois de la P4 sur la 15-14b.
- **Retenu** (décision de l'orchestrateur) : critère d'arrêt du `CLAUDE.md` atteint — validation de la
  15-14a **close**, LOW appliqués dans la même remédiation, statut `ready-for-dev` (déjà posé). R L-2
  (« Paramètres → Facturation », titre d'écran) hors périmètre : #585 (P4) ouverte par l'orchestrateur ;
  R L-5 : #459 commentée par l'orchestrateur.
- **Écartée** : une passe P5 sur la 15-14a — aucun MEDIUM à vérifier, et ses LOW ne touchent que la fiche.
- **Réversible** : oui (une revue ultérieure peut rouvrir la validation).

## C-15-14-33 — 15-14b (validation P4) : « Hyper Backup » hors des sections Synology, par une liste fermée de fragments

- **Contexte** : R-2 = F-1 (MEDIUM). La clause de G16 posée en P3 — chaque « Hyper Backup » hors des sections
  suivi d'un `\ref{sec:backup-dsm}` à moins de 300 caractères — rougissait sur le texte cible de la fiche
  même : `:912` (logs de `root`, assumée, sans renvoi), seconde occurrence dans la cible de `:2476`, renvoi
  de la cellule `:1741` non dit conservé.
- **Retenu** : G16 (e), **liste fermée** de quatre fragments normalisés — trois à renvoi (`:1741`, `:1748`,
  `:2476`, chaque fragment contenant le `\ref{sec:backup-dsm}` qui suit l'occurrence), un exempté (`:912`) ;
  toute autre occurrence rouge, toute exemption morte rouge. Cible de `:2476` réécrite à une seule
  occurrence ; renvoi de `:1741` conservé. Bornes de section prises sur le source brut au `\subsection{`
  qui porte le label (le titre de `sec:backup-dsm` nomme Hyper Backup avant le label — relevé en appliquant
  la garde au texte cible).
- **Écartée** : garder une règle de distance en l'assortissant d'exemptions (deux mécanismes pour un même
  contrôle, et une distance arbitraire) ; ajouter un renvoi à `:912` (le texte y est juste et ne parle pas de
  la base : un renvoi y serait du bruit).
- **Réversible** : oui.

## C-15-14-34 — 15-14b (validation P4) : le post-script ne supprime plus le dump

- **Contexte** : R-1 = F-3 (MEDIUM). `admin-manual.tex:1608` propose un post-script qui supprime le dump ;
  avec des snapshots horaires et un dump qui n'existe que le temps de la tâche, presque aucun snapshot ne
  contiendrait la base, et la recovery n'aurait rien à recharger. La P3 l'avait classée « assumée ».
- **Retenu** : `:1608` passe dans les corrigées (14 + 29 = 43, contrôlé par `comm`) ; texte cible « Laissez
  le post-script vide : le dump doit rester … » ; G16 (d) : aucun `rm` sur la cible du dump dans les sections
  Synology (un `rm` du `.tmp` permis), aucun « supprimer le dump », phrase positive présente ; angle mort
  écrit (une autre tournure en prose).
- **Écartée** : garder l'option en ajoutant une réserve (« … au prix de ne plus pouvoir restaurer la base
  depuis un snapshot ») — une option qui défait la copie 1 du 3-2-1 n'a pas sa place dans une recette.
- **Réversible** : oui.

## C-15-14-35 — 15-14b (validation P4) : deux comptes, deux fichiers, deux recettes ; la restauration par le compte Kesh

- **Contexte** : F-2 (MEDIUM). Le compte de sauvegarde à `SELECT` + `LOCK TABLES` (C-15-14-28) et la
  recovery « par le même fichier d'identifiants que le pré-script » s'excluaient : un dump
  `--add-drop-database --databases` exige `DROP`/`CREATE DATABASE`, `CREATE TABLE`, `INSERT`. Faite telle
  qu'écrite, la recette échouait au rechargement — le jour du sinistre.
- **Retenu** : sauvegarde par un compte `kesh_backup` (`GRANT SELECT, LOCK TABLES ON kesh.*`) dans
  `kesh-dump.cnf`, lu par le seul pré-script ; restauration par le **compte Kesh** de `DATABASE_URL`, que
  le manuel crée déjà avec `GRANT ALL PRIVILEGES ON kesh.*` (`admin-manual.tex:974`), dans
  `kesh-restore.cnf`, lu par la seule commande de rechargement. Deux recettes plus un contrôle négatif
  (rechargement avec `kesh-dump.cnf` → refusé), listings rejoués à l'identique, substitutions listées.
  G16 (c) exige un fichier de recovery différent de celui du pré-script.
- **Écartées** : le `root` du SGBD pour la restauration (mot de passe d'administration sur disque pour une
  opération que le compte Kesh sait faire ; un texte qui nommerait `MARIADB_ROOT_PASSWORD` dans les sections
  Synology rougirait G16 (a)) ; un seul compte Kesh pour les deux usages (perd le moindre privilège d'une
  tâche planifiée nocturne).
- **Réversible** : oui — si la recette montre que le compte Kesh ne peut pas `CREATE DATABASE`, le
  consigner et rouvrir.

## C-15-14-36 — 15-14b (validation P4) : signal D5 levé de nouveau, pas de découpage

- **Contexte** : les trois MEDIUM de la P4 sont nés de la remédiation P3 (`b02e9af7`), tous dans la
  recette de sauvegarde et ses gardes — le recyclage que vise la décision D5.
- **Retenu** (décision de l'orchestrateur) : pas de découpage. Quatre AC ; un seul fichier de test pour
  les gardes de l'AC 1 ; la story attend de toute façon le merge de la 15-13b (PR #584). La remédiation
  ajoute un contrôle de cohérence garde par garde contre le texte cible, et la P5 sera une passe ciblée sur
  la seule 15-14b, braquée sur cette remédiation.
- **Écartée** : sortir l'AC 1 dans une 15-14c — elle partagerait le manuel d'administration et
  `configuration_transmise.rs` avec le reste de la 15-14b, pour une séquence de merges plus longue.
- **Réversible** : oui (découpage possible à la P5 si la recette recycle encore).

## C-15-14-37 — 15-14b (validation P4) : l'invitation et la connexion par e-mail du manuel utilisateur entrent dans l'AC 3

- **Contexte** : L-1 de la lentille F (LOW). `user-manual.tex:79` décrit une invitation par e-mail et
  `:91` une connexion par e-mail ; aucune route d'invitation n'existe, la connexion prend un `username`
  (`LoginRequest`), l'écran affiche « Identifiant ». Ce sont les voisines de `:83`, que l'AC 3 corrige déjà.
- **Retenu** (décision de l'orchestrateur) : intégrées à l'AC 3, vérifiées au code, avec contrôle `occ`
  « présent avant » relevé (1 et 1) ; hors du motif de G18, sites ajoutés hors décompte ; une phrase au
  CHANGELOG de l'AC 4.
- **Écartée** : ouvrir une issue P3 à part (deux lignes dans une sous-section que la story réécrit déjà).
- **Réversible** : oui.

## C-15-14-38 — 15-14a (développement) : le tableau des plans du manuel d'administration décrit le contenu réel des trois plans

- **Contexte** : l'AC 3 demande de réécrire le tableau « Choix du plan comptable » sur les trois plans
  réels et leur choix par le type d'organisation, sans dicter les cellules. Une première rédaction
  disait le plan indépendant « allégé » ; les fichiers le démentent (`pme.json` et `independant.json`
  portent 86 comptes chacun, `association.json` 83).
- **Retenu** : en-têtes « Type d'organisation / Plan mis en place » ; chaque cellule nomme ce qui
  distingue réellement le plan, relu dans les JSON (PME : capital social ; indépendant : capital de
  l'exploitant, prélèvements et apports privés ; association : capital de l'association, cotisations
  des membres, dons reçus) ; une phrase « Kesh n'importe pas de plan comptable », qui remplace la ligne
  « Personnalisé CSV ». Le `keshtip` qui suit reste, relu : il est juste.
- **Écartées** : décrire les plans par leur public (« pour les artisans »…), ce que les fichiers ne
  portent pas ; garder une seule ligne générique (le tableau perdrait sa raison d'être).
- **Réversible** : oui (texte seul, garde G4 sur les formes interdites).

## C-15-14-39 — 15-14a (développement) : en de-CH, it-CH et en-CH, la proposition de but passe en tête

- **Contexte** : l'AC 8 donne la prescription par locale (« … muss ein Administrator die abgeschlossenen
  Geschäftsjahre bis zu diesem wieder öffnen, beginnend mit dem neuesten ») et demande de reprendre le
  début de chaque valeur. Laissée à sa place, la proposition finale (« damit sie storniert werden
  kann », « per poterlo annullare », « before it can be cancelled ») serait rejetée après une longue
  incise d'ordre.
- **Retenu** : la proposition de but passe avant la prescription, comme le texte fr-CH de l'AC 8
  (« pour pouvoir l'annuler, un administrateur doit … ») : « Damit sie storniert werden kann, muss ein
  Administrator … », « per poterlo annullare, un amministratore deve … », « before it can be cancelled,
  an administrator must … ». La prescription dictée par l'AC figure mot pour mot dans chaque valeur ; le
  marqueur d'ordre, seul contrôlé par G8, est intact.
- **Écartée** : garder l'ordre d'origine (phrase plus lourde, sans gain de fidélité).
- **Réversible** : oui (libellés).

## C-15-14-40 — 15-14a (développement) : les gardes de catalogue lisent les valeurs brutes, sans repli fr-CH

- **Contexte** : `I18nBundle::all_messages` complète chaque locale par les clés fr-CH. Une garde qui
  s'en servirait verrait, pour une clé absente en de-CH, la valeur française — et G8, G10, G11
  pourraient passer sur une locale amputée.
- **Retenu** : `valeurs_brutes(locale)` (`loader.rs`, `mod tests`) lit le `.ftl` lui-même (lignes de
  continuation jointes, commentaires ignorés) ; une clé absente panique en nommant locale et clé, et un
  catalogue lu à moins de 100 clés rougit (anti-test-muet). G8 contrôle au moins 10 clés (`>= 10`,
  6 de #569 + 4 qui portaient déjà le marqueur) : une clé neuve qui prescrit la réouverture entre au
  domaine sans casser la garde.
- **Écartée** : `all_messages` (repli masquant) ; un compte exact du domaine (rougirait à chaque clé
  neuve légitime).
- **Réversible** : oui.

## C-15-14-41 — 15-14a (développement) : G2 exige que tout `.tex` de `docs/manual/fr` soit gardé

- **Contexte** : les gardes documentaires parcourent une liste de trois manuels. Un quatrième `.tex`
  ajouté au répertoire échapperait à G2, G4 et G5 sans que rien ne le signale.
- **Retenu** : `manuels_fr()` (`textes_coherents.rs`) compte les `.tex` du répertoire et rougit si ce
  nombre diffère de la liste ; chaque manuel lu doit contenir `\begin{document}`. Une story qui ajoute
  un manuel l'ajoute à la liste.
- **Écartée** : lire tout `.tex` trouvé sans liste (un fichier vide ou mal nommé passerait en silence).
- **Réversible** : oui.

## C-15-14-42 — 15-14a (développement) : les comptes d'inventaire « après correction » de la fiche ne sont pas tous atteints, et c'est attendu

- **Contexte** : au T0, les dix-sept commandes rendent exactement les comptes de la fiche. Après
  correction, quatre diffèrent de l'« après » annoncé.
- **Retenu** (ventilation, aucune correction supplémentaire) : AC 1 → 3 (2 attendus) : le doc-comment
  de G1 (`vat_rates.rs`) nomme les taux interdits ; AC 2 (B) → 131 (129) : deux lignes de G3
  (`loader.rs`) ; AC 6 → 4 (2) : les deux textes neufs dictés par l'AC 6 disent eux-mêmes « jusqu'à la
  v0.9.0 incluse » (`api-external.md:484`, `admin-manual.tex:2053`) — c'est la fiche qui se trompait en
  annonçant 2 ; AC 9 → 2 (0) et AC 10 → 11 sur le dépôt (9) : lignes des gardes G10 et G11. AC 8 : 153
  → 155 = 153 − 3 commentaires `#569` retirés d'`errors.rs` + 5 lignes de G8. Toutes sont des gardes ou
  des textes prescrits ; aucun site affiché ne reste à corriger.
- **Écartée** : réécrire les gardes pour éviter les mots qu'elles interdisent (elles cesseraient de dire
  ce qu'elles gardent).
- **Réversible** : sans objet (constat).

## C-15-14-43 — 15-14a (revue de code P1, B-1) : « Réglages » interdit sous toute forme dans les manuels, le README et `.env.example`

- **Contexte** : `README.md:213` (« éditables (Réglages, Admin) ») renvoyait encore au menu, que la fiche
  avait classé « feuille de route publiée ». G2 cherchait quatre formes (`\emph{Réglages}`, `Réglages et`,
  `(Réglages)`, `dans les réglages`) ; celle-ci passait entre elles.
- **Retenu** : la ligne est corrigée (« (Paramètres, Admin) ») — le tableau de feuille de route du README
  est tenu à jour, ce n'est pas une note publiée (celles-ci vivent au CHANGELOG, exclu). G2 interdit le mot
  capitalisé « Réglages » **sans forme** dans les trois manuels, le README et `.env.example` ; aucun de ces
  textes ne l'emploie comme nom d'objet. Inventaire AC 2 (A) rejoué sur tout le dépôt : 13 lignes, les
  5 noms d'objet du journal d'audit et les 8 commentaires assumés ; (B) 131, inchangé.
- **Écartée** : ajouter la forme `(Réglages,` à la liste (la suivante passerait).
- **Réversible** : oui.

## C-15-14-44 — 15-14a (revue de code P1, B-2) : la ligne v0.4.0 du README perd « dossier surveillé », G5 perd son exemption

- **Contexte** : la fiche gardait `README.md:211` (« import de factures depuis un dossier surveillé ») au
  titre de l'historique publié, et G5 l'exemptait.
- **Retenu** : corriger. Vérifié au tag `v0.4.0` : l'import passait déjà par `POST /api/v1/inbox-import`,
  sans tâche de fond — la phrase était fausse **dès cette version** ; ce n'est donc pas réécrire
  l'historique, c'est corriger une description. Nouveau texte : « import de factures déposées dans un
  dossier d'import, lancé depuis l'écran ». G5 n'a plus d'exemption. Inventaire AC 5 : 25 → 20 (5 corrigées).
- **Écartée** : garder l'exemption (deux affirmations contraires dans le même README sur le même flux).
- **Réversible** : oui.

## C-15-14-45 — 15-14a (revue de code P1, E-1, E-2, E-5) : un numéro de compte cité existe dans un plan livré, sous son nom ; garde G4-bis par inventaire

- **Contexte** : l'exemple « Association » faisait créer 3600/3601 alors que le plan association porte
  3000 « Cotisations des membres » et 3100 « Dons reçus ». L'inventaire de tous les nombres de quatre
  chiffres du manuel utilisateur et du guide de démarrage contre les trois JSON (script en lecture) en a
  trouvé d'autres : `1030` (absent des trois plans, manuel et guide), `1020 CCP` / `1020 Caisse` (1020 =
  Banque), `3200 Ventes de services` / `3200 Honoraires` (3200 = Prestations de services en PME),
  `4200 Charges de personnel` (4200 = Achats de matières premières), `5700 AVS/AI/APG` (Charges sociales),
  `6500 Entretien` (Administration), `3400 Maintenance` (3400 = Autres produits), `6997 ou 7997` (le compte
  d'arrondi des trois plans est 6940), `2800 Capital` (nom incomplet). Le manuel d'administration, relu de
  même, n'a que des numéros justes. Ventilation des sites corrigés : manuel utilisateur `:205` (voir
  ci-dessous), `:318`-`:323`, `:332`, `:416`, `:419`, `:859`, `:906`, `:909`, `:917`, `:2249` ; guide
  `:46`-`:58`. Les sous-comptes (`1200.1`, `3000.1`, `3200.1`, `4000.1`, `1100.1`, `1020.1`) sont présentés
  « à créer » et restent.
- **Retenu** : chaque exemple prend le compte livré et son nom ; quand l'exemple veut un compte qui
  n'existe pas (maintenance), il devient un sous-compte **dit à créer** (`3200.1 Maintenance`). Garde
  **G4-bis** (`les_comptes_cites_en_exemple_existent_dans_les_plans_livres`, `textes_coherents.rs`) :
  inventaire de tous les nombres de quatre chiffres isolés (années 2001-2099 écartées, trou vérifié dans
  les plans), chacun doit être un compte livré ; les formes nommées (`« »`, `\texttt`, `\emph`, backticks,
  listes entre parenthèses) doivent porter le nom du plan. Source : `kesh_core::chart_of_accounts::load_chart`,
  non une copie.
- **E-2, guide de démarrage** : le §3 est réécrit sur `onboarding/+page.svelte` (sept étapes, dans
  l'ordre du code) ; l'exercice est créé à la finalisation. En le relisant, la phrase que la fiche avait
  prescrite à `user-manual.tex:204` s'est révélée fausse : le plan est mis en place **à l'étape de la langue
  comptable** (`set_accounting_language` → `load_chart`), non à la finalisation. Corrigée dans le manuel et
  dans le guide.
- **Écartée** : interdire la liste des numéros faux trouvés (une forme imprévue passerait) ; contrôler
  aussi le manuel d'administration par inventaire (ports, codes d'erreur, modèles de NAS : la garde y
  serait une liste d'exceptions — ses numéros de compte ont été vérifiés à la main, tous justes).
  Angles morts déclarés : un nom écrit hors des formes reconnues (« Crédit 3000 Ventes … ») n'est contrôlé
  que par son numéro ; `1000` (montant, borne) passe parce qu'il est aussi le compte Caisse.
- **Réversible** : oui.

## C-15-14-46 — 15-14a (revue de code P1, B-4) : trois analyseurs de catalogue alignés, non factorisés

- **Contexte** : `catalogue_fr` (`textes_coherents.rs`) et `valeurDuCatalogueFr` (Vitest G13) ne lisaient
  que la première ligne d'une valeur ; `valeurs_brutes` (`kesh-i18n`, `mod tests`) joint les continuations.
- **Retenu** : les deux premiers joignent désormais les lignes qui commencent par un blanc (espace de
  jonction), comme le troisième ; un commentaire ou une ligne vide clôt la valeur. Anti-test-muet Rust :
  `le_catalogue_fr_joint_les_continuations` lit `email-password-reset-body` (cinq lignes) entier — la
  mutation « ne pas joindre » le rend rouge.
- **Écartée** : factoriser. Les trois vivent dans deux crates de test et un fichier TypeScript ; un
  analyseur partagé exigerait d'exposer une fonction publique dans `kesh-i18n` (code de production) pour
  des tests, ou une crate utilitaire de test — disproportionné pour douze lignes. La règle commune est
  écrite dans chacun des doc-comments.
- **Réversible** : oui.

## C-15-14-47 — 15-14a (revue de code P1, B-6) : la réouverture bloquée nomme les exercices **postérieurs** ; le référent italien et allemand est explicite

- **Contexte** : `error-fiscal-year-reopen-blocked` disait « rouvrez d'abord les exercices clôturés ».
  Vérifié au code : la garde LIFO (`find_later_closed_in_tx`, `start_date > ? AND status = 'Closed'`) ne
  bloque que sur un exercice **postérieur** clôturé. En it/de, « fino a questo » / « bis zu diesem » après
  « Questo pagamento » / « Dieser Abgleich » pouvait se lire comme le paiement ou le rapprochement.
- **Retenu** : fr « les exercices postérieurs clôturés », de « die späteren abgeschlossenen
  Geschäftsjahre », it « gli esercizi successivi chiusi », en « the later closed fiscal years » (marqueur
  d'ordre de G8 conservé dans chaque locale) ; repli Rust `fiscal_years.rs` identique (G9). it « fino a
  questo esercizio », de « bis zu diesem Geschäftsjahr » sur les cinq clés de la famille, pour qu'elles
  restent parallèles.
- **Non touché en P1, ~~signalé à la 15-12b~~ — rectifié en revue de code P2 (A-1, E2-2, B2-1)** : la même
  formule large (« un administrateur rouvre [d'abord] les exercices clôturés, en commençant par le plus
  récent ») restait dans **quatre** clés de la famille `LATER_FISCAL_YEAR_CLOSED`, quatre locales chacune
  (`error-fiscal-year-create-later-closed`, `error-later-fiscal-year-closed`,
  `journal-entries-modify-blocked-later-fiscal-year-closed`, `fiscal-year-out-of-order-warning` : 16
  valeurs — l'inventaire de P1 n'en citait que 4), leurs quatre replis (`errors.rs:1556`, `:2960` — numéros
  de P1 `:1539`, `:2925` périmés —, `blocker-messages.ts:94`, `settings/fiscal-years/+page.svelte:355`) et
  `user-manual.tex:684`. ⚠️ **Le renvoi « à traiter par la 15-12b » était faux dès son écriture** : la
  15-12b était mergée (`dc4bc58b`, ancêtre de la branche, sur lequel la 15-14a a été spécifiée) ; plus
  personne ne reprenait ces textes, et la dette n'avait ni propriétaire ni story. Traités dans la 15-14a :
  C-15-14-49.
- **Réversible** : oui.

## C-15-14-48 — 15-14a (revue de code P1) : les LOW appliqués, et l'unique écarté

- **B-3 = E-3** : `.env.example` réécrit en deux phrases (« … déposées dans un dossier. L'import se lance
  à la demande depuis l'écran « Importer des factures » : Kesh décode alors le QR … et crée … »).
- **B-5, A-3** : G1 reconnaît toute graphie d'un taux périmé (`\b(2[.,]50?|3[.,]70?|7[.,]70?)[ ~]?\\?%`)
  et exige les quatre taux posés sur la ligne `\textbf{TVA due}` aussi.
- **B-7** : l'écran s'intitule `vat-rates-title = Taux de TVA` (`settings/+page.svelte:596`) ; fr-CH
  `vat-purchase-no-rates` et son repli (`VatPurchaseAssistant.svelte:149`) disent « Aucun taux de TVA
  configuré — voir Paramètres → Taux de TVA. » ; la clé entre dans G13. Les trois autres locales
  nommaient déjà leur titre d'écran.
- **A-1** : les « après correction » des AC 1, 6, 9, 10 portent les valeurs atteintes, avec renvoi à
  C-15-14-42 ; l'AC 2 (A) et l'AC 5 portent la partition révisée.
- **A-4** : le `keshtip` du manuel d'administration dit choisir le **type d'organisation** (seul choix
  existant, fixé à l'onboarding : `companies.rs` recopie `org_type` sans le modifier).
- **A-5** : de-CH `settings-fiscal-years-link` à l'impératif de politesse, comme ses voisines :
  « Verwalten Sie die Geschäftsjahre Ihres Unternehmens: erstellen, umbenennen, abschliessen. »
- **E-4** : « Kesh peut décoder automatiquement » → « Kesh peut lire, sans saisie de votre part » (plus
  de tension avec « l'import ne se déclenche pas tout seul »).
- **A-2** (non demandé par l'orchestrateur, traité par le geste) : `npm run check` et
  `lint-i18n-ownership` sont journalisés cette fois dans `kesh-gate-logs/15-14a-review-p1-*.log`.
- **Écarté** : aucun.
- **Réversible** : oui.

## C-15-14-49 — 15-14a (revue de code P2, A-1 = E2-2 = B2-1, A-2, E2-7) : toute la famille `LATER_FISCAL_YEAR_CLOSED` prescrit de rouvrir les exercices **postérieurs** ; garde G8-bis par inventaire des non-bornées

- **Contexte** : P1 avait borné la seule `error-fiscal-year-reopen-blocked` et renvoyé ses sœurs à une
  15-12b déjà mergée (C-15-14-47, rectifié). Vérifié au code : `FIND_LATER_CLOSED_SQL`
  (`kesh-db/src/repositories/fiscal_years.rs`, `start_date > ? AND status = 'Closed'`), utilisée par
  `create` (garde de création), `find_later_closed*` (saisie, modification, suppression d'une écriture,
  15-12a/b) et la garde LIFO de `reopen` : seul un exercice clôturé **postérieur** bloque. Cas atteignable où
  la formule large est fausse : 2024 et 2026 clos, création de 2025 — seule la réouverture de 2026 est
  requise ; le message faisait rouvrir aussi 2024 (verrou CO 957-964 levé, entrée d'audit) sans raison.
- **Retenu** : la formule de B-6, déjà écrite dans les quatre langues — fr « les exercices postérieurs
  clôturés », de « die späteren abgeschlossenen Geschäftsjahre », it « gli esercizi successivi chiusi », en
  « the later closed fiscal years » — dans les quatre clés × quatre locales ; `user-manual.tex:684` aligné
  sur `:726` (« les exercices clôturés postérieurs ») ; les deux replis Rust réécrits **depuis** le catalogue
  (apostrophes typographiques comprises, E2-7) et entrés dans **G9**, qui compare désormais un repli
  paramétré après réécriture des variables Fluent (`{ $name }` → `{fiscal_year_name}`) ; les deux replis
  frontend entrés dans **G13** (site unique vérifié par la garde elle-même). CHANGELOG : « rouvrir les
  exercices clôturés postérieurs, ou jusqu'à celui-ci ». Pour `fiscal-year-out-of-order-warning`, la même
  formule que ses sœurs, plutôt que « postérieurs à « { $open } » » (B2-1) : la phrase vient de nommer
  `{ $open }` et l'exercice postérieur clôturé ; une seule formule rend la garde exacte.
- **Garde G8-bis** (`les_prescriptions_de_reouverture_sont_bornees`, `kesh-i18n/src/loader.rs`) :
  inventaire des **non-bornées** — tout le domaine de G8 (verbe fr-CH, mêmes exemptions) doit porter, dans
  chaque locale, l'une des deux bornes justes (« postérieurs » ou « jusqu'à celui-ci ») ; plus une assertion
  **positive** « postérieurs » par clé et par locale sur les cinq clés qui la portent (couvre A-2 : B-6
  n'était gardée par rien).
- **Écartée** : une issue P3 avec propriétaire (A-1, alternative) — le correctif est mécanique, dans le
  périmètre de #569, et la raison du report a disparu ; ouvrir une issue pour un défaut qu'on peut fermer
  dans la même passe déplacerait la dette sans raison.
- **Réversible** : oui.

## C-15-14-50 — 15-14a (revue de code P2, E2-1, B2-2, B2-3, E2-3) : l'écran *Comptes bancaires* cite les comptes livrés ; G4-bis lit les catalogues, G4-ter la forme libre partout

- **Contexte** : l'en-tête et l'info-bulle de l'écran *Comptes bancaires* (`bank-accounts-labels-page-subtitle`,
  `bank-accounts-tooltip-journal-account`, quatre locales, deux replis Svelte) et deux doc-comments
  (`errors.rs:725`, `bank_account.rs:10`) disaient « 1020 Caisse, 1030 Banque », « sous-compte 1030.001,
  pas au parent 1030 » — le symptôme corrigé dans le guide en P1, non grepé ailleurs. Les trois JSON :
  `1000 Caisse`, `1010 Poste`, `1020 Banque` ; aucun 1030.
- **Retenu** : « 1010 Poste, 1020 Banque » (de Post/Bank, it Posta/Banca, en Postal account/Bank) — les deux
  comptes qu'un compte bancaire alimente ; « Caisse » (proposé par E2-1) n'est pas un compte bancaire.
  Sous-compte `1020.001 BCV CHF`, parent `1020`. it/en : l'exemple « BCV + PostFinance » devient
  « BCV + UBS » (PostFinance relève de 1010). Commentaire de la migration
  `20260507200001_bank_account_journal_link.sql:4` **non touché** (P8 : le checksum). CHANGELOG `:627`
  (entrée d'une release publiée, `1030.001/1030.002`) non touché : l'historique ne se réécrit pas.
- **Gardes** : G4-bis lit désormais les **quatre catalogues** en entier (inventaire de numéros) ; ses bornes
  sont relevées par caractère voisin **sans consommation** (`1000 1030` : les deux lus), `{`/`}` ne bornent
  plus (`\textbf{3600}` lu ; vérifié : aucun faux rouge neuf sur les deux textes), `/` en tête non plus ;
  montants (devise adjacente), NPA (mot à majuscule qui n'ouvre aucun nom de compte, en fin de segment) et
  années (2001-2099, aussi dans les listes entre parenthèses) écartés ; noms comparés apostrophes
  normalisées ; forme `NNNN (Nom)` ajoutée. **G4-ter** (`la_forme_libre_nnnn_nom_est_juste_partout`) :
  « NNNN Mot », quand `Mot` ouvre un nom de compte de la langue, doit être juste dans le manuel
  d'administration, la brochure, le README, `api-external.md`, et le code (Svelte, TS, Rust hors tests et
  `test_fixtures.rs`, dont « 1100 Banque » / « 2000 Capital » sont des données de test). Angles morts
  écrits dans les doc-comments (`-` collé, borne/année hors 2001-2099, NPA à plusieurs mots, montant sans
  devise).
- **Écartée** : l'inventaire complet sur le code et le manuel d'administration (ports, codes, montants de
  test : une liste d'exceptions) ; une liste des numéros faux (une forme imprévue passerait).
- **Réversible** : oui.

## C-15-14-51 — 15-14a (revue de code P2, B2-4, E2-4, A-4) : les trois analyseurs de catalogue appliquent la même règle, et chacun a son anti-test-muet

- **Contexte** : le doc-comment de `catalogue_fr` les disait « identiques » ; `valeurs_brutes` ajoutait
  toute ligne non indentée (le `}` de sélecteur), le Vitest acceptait `cle=valeur`, les trois laissaient un
  blanc de tête à une clé en forme bloc, et seul `catalogue_fr` avait un anti-test-muet, qui n'assertait
  pas la clé suivante.
- **Retenu** : aligner plutôt que corriger le doc-comment — tête `^([a-zA-Z][\w-]*) = ?(.*)$`,
  continuation indentée seule, jonction sans blanc sur valeur vide ; un anti-test-muet par analyseur, sur
  les trois mêmes cas réels (`email-password-reset-body` entier sans blanc de tête,
  `auth-recovery-forgot-title` intact, `}` de `error-account-not-postable` non ajouté). Tout est code de
  test (`valeurs_brutes` vit dans `mod tests`).
- **Écartée** : factoriser (C-15-14-46, inchangé).
- **Réversible** : oui.

## C-15-14-52 — 15-14a (revue de code P2) : les LOW appliqués, et ce qui ne l'est pas

- **B2-5 = E2-6** : `user-manual.tex:333` « CCP 12-345-6 » sous 1020 → « UBS compte courant » ; guide
  « BCV + PostFinance » / `1020.002 PostFinance épargne` → « BCV + UBS » / `1020.002 UBS CHF`.
- **B2-6 = E2-5** : `user-manual.tex:199` (« une PME … renseigne obligatoirement son IDE ») faux au code —
  `set_coordinates` ne valide l'IDE que s'il est fourni, sans condition sur `org_type` ; l'écran dit
  « optionnel » ; le formulaire des coordonnées ne varie pas non plus selon le type. Réécrit : le type
  détermine le plan et ne se modifie plus après l'onboarding ; `:201` dit l'IDE facultatif quel que soit
  le type.
- **A-3** : AC 1, AC 8 (table, locales, famille P2) et G13 de la fiche annotés de la valeur que le code
  porte, avec renvoi au choix ; table des tests complétée (G4-bis, G4-ter, G8-bis).
- **A-6** : guide « dans les Paramètres » → « depuis **Administration → Exercices comptables** » (entrée de
  menu réelle, `nav-fiscal-years`). `user-manual.tex:708` « \emph{Paramètres} → \emph{Exercices
  comptables} » reste : chemin réel (bouton « Gérer » de `/settings`), territoire de #585.
- **A-5** : les 30 mutations du développement n'ont **pas de journal** — déclarées au Dev Agent Record, non
  vérifiables ; écrit tel quel dans la fiche, sans réécrire le chiffre. Celles de P1 et P2 sont journalisées.
- **Écarté** : aucun.
- **Réversible** : oui.

## C-15-14-53 — 15-14a (revue de code P3) : E3-1 reclassé en dette documentée (#589) ; LOW documentaires appliqués, LOW de garde écrits en dette

- **Contexte** : passe P3 (Sonnet ×3) — B 0 MEDIUM / 4 LOW, A 0 MEDIUM / 4 LOW, E 1 MEDIUM / 6 LOW. Le seul
  MEDIUM, E3-1 (l'infobulle du bouton « Réouvrir » nomme l'exercice clos le plus **proche**, que la garde LIFO
  refuse à son tour), tient à une logique d'écran antérieure à la story ; vérifié au code par l'orchestrateur.
- **Retenu** :
  - **E3-1 → dette documentée** : issue **#589** (P3), propriétaire = cette issue (exception « dette
    documentée » de la § *Review Iteration Rule*). La boucle de revue est **close** : P1 3 MEDIUM → P2 2
    MEDIUM → P3 0 MEDIUM + 1 reclassé.
  - **LOW documentaires appliqués** : B3-3 = LOW-2 (le CHANGELOG nomme le changement visible de l'écran
    *Comptes bancaires*) ; LOW-1 (fiche : G9 « huit clés, neuf sites » à l'AC 8 et au T4) ; E3-2
    (`admin-manual.tex:1401` aligné sur `:1714` — saisie, règlement, dévalidation, modification, suppression ;
    et « exercices clôturés postérieurs » dans la même phrase) + PDF ; **B3-2** (commentaire Rust
    `errors.rs`, « ne prescrit jamais de rouvrir l'exercice nommé **seul** … il prescrit les exercices
    postérieurs clôturés ») — appliqué plutôt que reporté : c'est du code au sens D7, mais le rebase sur
    `0724904c` impose de toute façon un gate complet et un E2E complet sur le dernier commit, qui le couvrent ;
    le reporter aurait laissé un commentaire contradictoire sans gain.
  - **LOW de garde laissés en dette, sans modification de code** (écrits ici, à reprendre avec la prochaine
    story qui touche `textes_coherents.rs` ou `loader.rs`) : B3-1 = LOW-3 (G4-bis lit les catalogues entiers :
    une limite « 4000 caractères », « ISO 8601 », « port 8080 » rougirait — déjà déclaré « rouge bruyant »
    au doc-comment ; correction possible : liste fermée d'exemptions `(fichier, sous-chaîne)` motivées) ; E3-3
    (lecture ligne à ligne : un numéro en fin de ligne dont le nom passe à la ligne suivante échappe à la forme
    nommée) ; E3-4 (`montant_ou_npa` : « 1030 CHF » passe pour un montant) ; E3-5 (G4-ter lit aussi les
    `#[cfg(test)] mod tests` des `src/`) ; E3-6 (exemptions de G8 et G8-bis dupliquées en dur) ; E3-7 (G9 table
    fermée, non inventaire des non-résolus).
  - **Sans suite** : B3-4 (guide et écran proposent des exemples différents, tous deux vrais) ; LOW-4 (deux
    chemins vers l'écran des exercices) — territoire de #585.
## C-15-1a-ii-1 — 15-1a-ii (T0) : la doc d'`UnvalidationBlocker` disait encore « trois autres empêchements »
- **Contexte** : C132 et la réserve R6-3 prescrivent de nommer la marque de lettrage sans la compter parmi les refus de la dévalidation, en relisant les totaux sur le `main` du moment. Sur `0724904c`, la 15-12b avait porté la doc d'`unvalidate` à « quatre autres » et les totaux à « neuf », mais la doc d'`UnvalidationBlocker` (`kesh-db/src/errors.rs`) disait toujours « trois autres (exercice clos, contre-passée, période verrouillée) » — l'exercice postérieur clos, rendu inconditionnel par la 15-12b, y manquait.
- **Retenu** : « quatre autres », `LaterFiscalYearClosed` nommé, et la marque nommée comme inatteignable, non comptée. Totaux « neuf » laissés tels quels.
- **Écartées** : garder « trois » (la fiche le prescrivait sur la foi de `ec745d0c` ; la réserve R6-3 interdit précisément de remettre un chiffre périmé) ; compter la marque (« cinq »), contraire à C132.
- **Réversible** : oui (doc-comment seul).

## C-15-1a-ii-2 — 15-1a-ii (T10) : terminologie du lettrage en allemand et en italien
- **Contexte** : les textes allemands et italiens arrêtés par la fiche pour la clé neuve et les deux clés des soldes de départ sont à « aligner sur les traductions que la 15-1a-i aura retenues ». La 15-1a-i a retenu *Ausgleich / ausgleichen / Ausgleich aufheben*, *match / unmatch*, *abbinamento / abbinare / disabbinare* (vouvoiement pluriel en italien : « annullate », « riprovate »).
- **Retenu** : `journal-entries-modify-blocked-lettered` = « Diese Buchung ist ausgeglichen: Heben Sie den Ausgleich zuerst auf. » / « This entry is matched: unmatch it first. » / « Questa scrittura è abbinata: disabbinatela prima. » ; pour les deux clés des soldes de départ, le verbe italien de la 15-1a-i (« disabbinatela prima », « dopo averla disabbinata ») au lieu de « annulla prima l'abbinamento » de la fiche (tutoiement et périphrase) ; allemand et anglais repris de la fiche tels quels.
- **Écartées** : reprendre l'italien de la fiche mot pour mot (deux désignations et deux registres pour le même geste).
- **Réversible** : oui (catalogues).

## C-15-1a-ii-3 — 15-1a-ii : la PR référence #518, elle ne la ferme pas
- **Contexte** : la consigne de développement annonçait que la 15-1a-ii « clôt le socle » du lettrage, et la clé 15-1a-i du sprint-status disait « la 15-1a-ii fermera l'issue ». Or #518 est la fonctionnalité entière (« savoir ce qui reste ouvert sur un compte »), la fiche porte `refs #518`, et la 15-1c se déclare « closes #518 (dernière des quatre) ».
- **Retenu** : commits et PR en `refs #518` ; la PR de la 15-1c portera `closes #518`. Le commentaire de la clé 15-1a-i est rectifié dans le sprint-status.
- **Écartées** : `closes #518` ici (fermerait l'issue avant l'écran, la vue des postes ouverts et le lettrage des pièces).
- **Réversible** : oui (mot-clé de PR).

## C-15-1a-ii-4 — 15-1a-ii (T12) : où vivent les tests
- **Contexte** : les tests de la fiche visent la route, `delete_in_tx(…, false)` (fonction `pub(crate)`), et deux annulations dont le montage existe au dépôt.
- **Retenu** : les tests de route (AC8, AC9 (a)(b)(c)(e)(f), AC10, entrelacement) dans `crates/kesh-api/tests/journal_entry_reversal_e2e.rs`, qui porte déjà les auxiliaires `PUT`/`DELETE`/détail et la table de correspondance ; les tests de `delete_in_tx(…, false)` (marque hors drapeau, paire C117, verrou de période sur le chemin `false`) en module de `journal_entries.rs`, dans une transaction annulée (base partagée, KF-039) ; (d) et le dé-rapprochement au dépôt (`supplier_invoices_repository.rs`, `reconciliation_cancel.rs`), le refus de délettrage éprouvé par `dissolve_group_in_tx` en mode `Manual` — la fonction qu'appelle la route, dont le mappage HTTP est testé par la 15-1a-i.
- **Écartées** : un fichier de test neuf (aurait recopié cent lignes de montage) ; rendre `delete_in_tx` public pour un test.
- **Réversible** : oui.

## C-15-1a-ii-5 — 15-1a-ii (T12) : le lettrage concurrent d'un `PUT`, entrelacé dans les deux ordres
- **Contexte** : la fiche demande un `POST /letterings` concurrent d'un `PUT` (« l'un réussit, l'autre rend un refus métier, jamais un 500 »), avec les motifs de `attendre_une_requete_en_cours`.
- **Retenu** : un test déterministe, `lettering_and_put_interleaved_never_answer_500` : le test tient l'en-tête de l'écriture, lance les deux requêtes l'une après l'autre en attendant de voir chacune bloquée (motifs `["jel.id IN", "FOR UPDATE"]` et `["je.version", "FOR UPDATE"]`), puis relâche. Lettrage d'abord → `201` puis `409 ENTRY_LETTERED` ; `PUT` d'abord → le cycle lignes ↔ écriture, résolu par le rejeu : `(201, 409 ENTRY_LETTERED)` ou `(404, 200)`. Jamais un 500.
- **Écartées** : une course libre répétée (prouve l'absence de 500 sans prouver que le cycle a été exercé).
- **Réversible** : oui.

## C-15-1a-ii-6 — 15-1a-ii (AC15 ii) : la phrase de la réouverture d'un exercice, hors du relevé de la fiche
- **Contexte** : le contrôle du PDF utilisateur aplati par la valeur (`se modifie`) a rendu une promesse sans réserve absente du relevé d'AC15 (ii) : « L'exercice rouvert redevient modifiable — ses écritures saisies à la main se modifient de nouveau, si aucun exercice postérieur n'est clôturé » (`user-manual.tex`, section de la clôture).
- **Retenu** : la réserve « et après délettrage pour celles dont une ligne est lettrée », PDF régénéré.
- **Signalé, non traité** : le § « Passer à la 0.13.0 » du manuel d'administration (relevé par C-15-1a-i-14) n'est pas touché — hors du périmètre de cette fiche, à la préparation de la release.
- **Réversible** : oui.

## C-15-1a-ii-7 — 15-1a-ii (revue de code P1, B-5) : le 409 `ENTRY_LETTERED` porte `details.letteringCode`
- **Contexte** : le 409 réemployait `details.documentNumber` pour le code du premier groupe (et `documentId: null`) ; ce champ porte un numéro de pièce dans tous les autres refus, et un client générique afficherait « pièce n° AB ». Aucun client ne le lit : `grep -rn "documentNumber" frontend/src` hors tests → aucune occurrence ; la v0.13.0, qui introduit le code, n'est pas publiée.
- **Retenu** : `details: { letteringCode }`, sans `documentId` ni `documentNumber` ; message toujours suffixé du code. Fonction commune `refusal_409` (message + `details` fourni) sous `entry_document_refusal_response`, pour ne pas dupliquer la construction du message. `api-external.md` (trois sites) et CHANGELOG suivent ; test AC8 asserte le `details` entier au `PUT` (trois cas) et au `DELETE`. Écart à la lettre d'AC8 (« `details` de la forme commune ») assumé.
- **Écartées** : garder `documentNumber` (ambiguïté de contrat, plus coûteuse à lever après publication) ; `details.code` (la forme des refus du lettrage, mais homonyme de `error.code` dans un refus d'écriture ; `letteringCode` est le nom du champ des lignes).
- **Réversible** : oui jusqu'au tag v0.13.0 ; ensuite changement de contrat.

## C-15-1a-ii-8 — 15-1a-ii (revue de code P1, B-3/E-1) : `lettering_guard` privée plutôt que scopée par jointure
- **Contexte** : la garde ne lit que `journal_entry_lines WHERE entry_id = ?` et ignorait son `_company_id`, alors qu'elle était `pub`.
- **Retenu** : `lettering_guard` et `Lecture` **privées au module** (leurs trois appelants y sont), paramètre `_company_id` retiré, liens rustdoc vers l'item privé ramenés à du texte. La doc dit pourquoi.
- **Écartées** : la jointure `journal_entries je ON … AND je.company_id = ?` — sous `FOR UPDATE`, un plan partant de l'index de société verrouillerait les en-têtes parcourus, ce qui change l'ensemble des verrous de la garde ; non mesuré, donc non retenu.
- **Réversible** : oui.

## C-15-1a-ii-9 — 15-1a-ii (revue de code P1, A1) : quelles mutations prouvent le rollback du lettrage
- **Contexte** : A1 demande de prouver que le groupe `reversal` et l'audit `lettering.created` partent avec la transaction de l'appelant, et de jouer une mutation qui « écrit la marque ou l'audit hors de la transaction ».
- **Retenu** : le test lit, dans la transaction, les marques des quatre lignes (origine et miroir, origine `reversal`), deux clés distinctes et deux audits ; après le rollback, les marques des lignes d'origine nulles et zéro audit pour ces clés. Mutations (journal `15-1a-ii-review-p1-mutations.log`) : M-A1-1 audit écrit et commité sur une connexion distincte → **tuée** (la vue `REPEATABLE READ` de la transaction ne le voit pas : assertion « visible dans la transaction ») ; M-A1-2 R6 ne lettre rien → **tuée** (assertion de marque dans la transaction) ; M-A1-3 `COMMIT` pour le compte de l'appelant après R6 → **tuée** (par l'assertion préexistante sur l'écriture inverse, la première à parler).
- **Non jouable, et pourquoi** : écrire la **marque** hors de la transaction de l'appelant. Le groupe apparie une ligne d'origine, tenue `FOR UPDATE` par cette transaction (étape 2), et un miroir non commité, invisible ailleurs : toute autre connexion attendrait le verrou (1205) ou ne trouverait pas le miroir. Les assertions négatives d'après le rollback sur les marques et l'audit ne sont donc tuées seules par aucune mutation réaliste ; elles gardent la propriété contre un futur chemin qui commiterait le lettrage à part.
- **Réversible** : oui (test).

## C-15-1a-ii-10 — 15-1a-ii (revue de code P1) : ce qui reste en dette, et B-4
- **B-2** (rafale de requêtes de R6 dans la transaction) : gardé en dette. Correction sans effet sur l'exactitude (les lectures sont dans la même vue) ; la mémoïsation de `is_letterable_account` par compte ne retirerait qu'une requête sur six à huit par ligne, `create_group_in_tx` restant par paire. Mesure à faire sur une écriture de 60 lignes (déjà relevée F-12) avant de toucher la primitive ; à reprendre si un rejeu de contre-passation est observé.
- **A4** (F2 survivante : l'identité de la clé i18n d'un motif d'écran n'est gardée par aucun test) : angle mort préexistant, commun aux douze branches de `modificationBlockerLabel` ; le fermer pour une seule branche serait trompeur. À ouvrir en issue P3 par l'orchestrateur (test qui résout chaque clé dans les quatre catalogues et compare au repli).
- **A5** (AC9 (d) éprouvé au dépôt, non à la route) : écart déjà motivé à C-15-1a-ii-4 ; la route `DELETE /letterings/{key}` n'ajoute que le mappage HTTP, testé par la 15-1a-i. Rien à faire.
- **B-4** (le message prescrit un délettrage sans écran) : **déjà tranché** à C131 (7) — message inchangé, la fenêtre sans écran de délettrage n'est jamais publiée (l'epic sort en une release, la 15-1c apporte l'écran). Rien à faire ici.
- **Réversible** : oui.

## C-15-1a-ii-11 — 15-1a-ii (revue de code P1) : rebase sur `1ae3963e` (15-6d)
- **Contexte** : `origin/main` avait avancé de la 15-6d (#590) depuis `0724904c`.
- **Retenu** : sauvegarde `backup/15-1a-ii-avant-rebase-p1`, rebase ; CHANGELOG, `api-external.md` et `user-manual.tex` fusionnés sans conflit ; `user-manual.pdf` régénéré (`make -B user`) ; registre et sprint-status **par union** (entrée de la 15-1a-ii renumérotée (49)).
- **Réversible** : oui (branche de sauvegarde).

## C-15-1a-ii-12 — 15-1a-ii : intégration sur `181efa3c` (15-14a)
- **Contexte** : `origin/main` a reçu la 15-14a (#591 : manuels, catalogues, gardes de texte) et le tmpfs à 8 Go (#588) après la clôture de la revue.
- **Retenu** : sauvegarde `backup/15-1a-ii-avant-rebase-181efa3c`, rebase. Conflit des catalogues ×4 tranché clé par clé : `opening-balances-locked-first-year-closed` = texte de la 15-14a (la 15-1a-ii n'y touchait pas), `opening-balances-locked-already-has-entries` = texte de la 15-1a-ii (celui de main plus la réserve du délettrage). PDF régénérés par `make -B`. Registre et sprint-status par union.
- **Réversible** : oui (branche de sauvegarde).

## C-15-7b3-1 — 15-7b3 (développement) : deux helpers de montage partagés dans `kesh_db::test_fixtures`
- **Contexte** : la fiche demande de factoriser le montage du déclencheur du test 8 de la 15-7b2 « si les deux fiches le portent » (Dev Notes), et son montage commun de l'état orphelin sert trois fichiers dans deux crates (`kesh-api/src/auth/bootstrap.rs`, `kesh-api/tests/admin_full_import_e2e.rs`, `kesh-db/tests/companies_repository.rs`).
- **Retenu** : `test_fixtures::rendre_principaux_orphelins(pool, dead_id)` (connexion détachée, `FOREIGN_KEY_CHECKS = 0` de session, `users` et **toutes** les `api_keys`) et `test_fixtures::poser_declencheur_en_echec(pool, nom, quand, condition)` (pré-requis `@@log_bin = 0` **asserté** avec un message qui le nomme, corps `SIGNAL` éventuellement sous `IF`). Le test 8 de la 15-7b2 (`reset_failure_erases_nothing_and_never_returns_its_connection`) passe par le second : seul son message `MESSAGE_TEXT` change.
- **Écartées** : une copie par fichier (DRY) ; un module de test partagé entre crates (aucun n'existe ; `test_fixtures` est déjà le lieu des montages communs, compilé en permanence).
- **Réversible** : oui (tests seulement).

## C-15-7b3-2 — 15-7b3 (développement) : l'acteur du démarrage en une requête ; `user_ids` au détail
- **Contexte** : AC 1, étape 7 — l'administrateur actif de plus petit `id`, à défaut l'utilisateur de plus petit `id` ; détail `users_repointed` **et** `user_ids`, alors que l'entrée `installation.reset` de la 15-7b2 ne porte que le nombre (T0, E3).
- **Retenu** : `SELECT id FROM users ORDER BY (role = 'Admin' AND active = TRUE) DESC, id LIMIT 1` — une requête, les deux branches ; mutations 7a et 7b rouges. Détail conforme à l'AC (`user_ids` en plus) : l'entrée désigne des utilisateurs qui n'ont rien fait, les nommer est l'objet de la trace.
- **Écartées** : deux requêtes successives (même résultat, un aller-retour de plus) ; aligner sur `installation.reset` en retirant `user_ids` (contraire à l'AC).
- **Réversible** : oui avant publication (le détail d'audit devient une archive à la release).

## C-15-7b3-3 — 15-7b3 (documentation) : où vit le texte, et ce qui devait changer au passage
- **Contexte** : AC 6 relocalisé par titre de section (fiche du 2026-10-08, manuel réécrit depuis par les 15-11a, 15-13b, 15-14a).
- **Retenu** : (1) le texte principal est un `\paragraph` étiqueté `sec:reparation-installation` placé **après** l'énumération et l'encadré « Toujours sauvegarder » de la § *Procédure de mise à jour standard*, avant la sous-section 0.13.0 de la 15-11a, sans la réécrire (C-15-7-55) ; les autres sites y renvoient. (2) L'étape *Parcours* disait « sa réparation au démarrage relève d'une version ultérieure » et le CHANGELOG de la 15-7b2 « correction ultérieure » (deux fois) : devenus faux, réécrits pour renvoyer à la réparation. (3) Ajouts **hors liste de l'AC 6**, parce que l'action paraît au journal : un paragraphe *La réparation de l'installation* dans la section journal d'audit du manuel utilisateur, une mention d'`installation.repaired` au § *Journal d'audit* et à la ligne du bootstrap (`#542`) du manuel d'administration. (4) Trois débordements créés par ces ajouts enveloppés dans `sloppypar` (57 → 54 au journal LaTeX) ; le symptôme du Dépannage reformulé (le nom `kesh-api` sortait de la page en fin de ligne, « kesh-a » au PDF aplati). (5) `make -B fr` régénère aussi la brochure : ses octets changent sans changement de source.
- **Réversible** : oui.

## C-15-7b3-4 — 15-7b3 (revue de code P1, B-4 = E3 = A6) : les aides de montage quittent le code de production
- **Contexte** : `rendre_principaux_orphelins` (`FOREIGN_KEY_CHECKS = 0`) et `poser_declencheur_en_echec` (`CREATE TRIGGER` composé par `format!`) vivaient dans `kesh_db::test_fixtures`, compilé avec la production (C-15-7b3-1). Décision de l'orchestrateur : hors du code de production. Appelants réels : `kesh-db/tests/companies_repository.rs`, `kesh-api/tests/admin_full_import_e2e.rs`, `kesh-api/tests/onboarding_audit_e2e.rs` (test 8 de la 15-7b2) et `mod tests` de `kesh-api/src/auth/bootstrap.rs`.
- **Retenu** : un fichier `crates/kesh-db/tests/support/installations_atteintes.rs` (`#![allow(dead_code)]`, en-tête qui dit pourquoi et interdit de le réexposer par `test_endpoints`), inclus par `#[path]` dans les trois tests d'intégration, et dans `bootstrap.rs` sous `#[cfg(test)]` au niveau du module (un `#[path]` dans le `mod tests` en ligne se résoudrait sous `src/auth/bootstrap/tests/`, répertoires inexistants). `test_fixtures.rs` est revenu **octet pour octet** à sa version de `e892dcfa` : son en-tête (« juste des INSERTs + TRUNCATEs ») redevient exact. Révise C-15-7b3-1.
- **Écartées** : une *feature* Cargo `test-support` (les tests d'intégration de `kesh-db` ne verraient la feature que par une auto-dépendance de développement, montage fragile) ; `#[cfg(test)]` seul dans `kesh-db/src` (invisible des tests d'intégration et de `kesh-api`).
- **Réversible** : oui (tests seulement).

## C-15-7b3-5 — 15-7b3 (revue de code P1) : remédiation des MEDIUM et des LOW retenus
- **B-1 (MEDIUM)** : test `repair_on_restore_keeps_superfluous_stubs` (une société, deux provisoires non référencées, `Restore` ⇒ trois sociétés restent, aucune cible, clé révoquée) ; mutation 15 (retirer `trigger == Startup`).
- **A1 (MEDIUM)** : test 2 bis `full_import_is_undone_when_the_repair_fails` (déclencheur sur l'entrée `installation.repaired` ⇒ 500 `ADMIN_FULL_IMPORT_FAILED`, société témoin posée après l'export toujours là, principaux orphelins, clé active, ni `installation.repaired` ni `admin.full_import`) ; mutation 16 (avaler l'erreur). Le manuel garde « tout ou rien », désormais prouvé. Montage du test 2 factorisé (`monter_528`).
- **B-2 (MEDIUM)** : le Dépannage prescrit `docker compose restart kesh-api` (chaque démarrage rejoue la réparation) et dit pourquoi `up -d` ne suffit pas. Les recettes `up -d` de la 15-7b2 et de la 15-11a (recharger une variable de `.env`, `:1045`, `:1293`, `:1345`, `:1372`) sont d'un autre besoin et restent.
- **E1** : la garde de liste vide est posée **dans** `company_referencing_columns` (et non dans l'appelant), ce qui la rend testable : `company_referencing_columns_refuses_an_empty_answer` (base vide choisie par `USE` sur une connexion détachée) ; mutation 17. Code de production touché.
- **B-3** : doc-comments de `repair_installation_in_tx` et de `bootstrap.rs` exacts : sur une base sans société le verrou de `companies` ne tient aucune ligne ; la sérialisation passe par les lectures verrouillantes de `users` et `api_keys` — raisonné, non testé.
- **B-7** : manuel et CHANGELOG : « le journal d'audit excepté, qui n'en retient que le numéro ».
- **A7** : ligne de `repair_installation_in_tx` à la table des verrous de `MULTI-TENANT-SCOPING-PATTERNS.md` ; brochure PDF remise à sa version d'`origin/main` ; la réparation sort de la phrase des « routes à verbe mutant ».
- **B-5, A2, A3, A4, A5, B-6, E2, E4, E5** : écrits au Dev Agent Record (journal des mutations, chemins non testés, décomptes, angles morts).
## C-15-1b-1 — 15-1b (validation P1, R1/F-2, L5) : le refus d'un compte non lettrable est celui de la 15-1a-i, et vaut pour un compte devenu non lettrable
- **Contexte** : la fiche écrivait `400 ACCOUNT_NOT_LETTERABLE` et « deux clés i18n » ; le socle livré porte déjà `DbError::LetteringAccountNotLetterable` → 409 `LETTERING_ACCOUNT_NOT_LETTERABLE`, clé présente dans les quatre locales. C104 laissait à la 15-1b ce qu'elle montre d'un compte retypé ou rattaché à un compte bancaire.
- **Retenu** : réutiliser le 409 sur les deux routes de lecture ; une seule clé neuve (`error-lettering-proposals-too-many-lines`). Un compte **devenu** non lettrable rend le même 409 : ses groupes restent consultables par `GET /letterings/{key}` et dissolubles (C104), la vue ne sert que les comptes où l'on peut lettrer.
- **Écartées** : un 400 propre à la lecture (second code pour le même refus) ; servir la vue d'un compte devenu non lettrable en lecture seule (une exception de plus à R4, sans usage nommé).
- **Réversible** : oui jusqu'au tag v0.13.0.

## C-15-1b-2 — 15-1b (validation P1, R4/F-4) : la propriété d'une écriture se lit par lot, d'une seule source
- **Contexte** : `reversal_blockers` lit une écriture à la fois ; la vue (page de 500) et le filtre R5 des propositions (jusqu'à 2 000 lignes) en feraient un N+1 ou une seconde liste.
- **Retenu** : `journal_entries::document_owners(executor, company_id, &[entry_id])`, une requête ensembliste découpée par 500, source unique des motifs de propriété (rangs 3 à 7) ; `reversal_blockers` et `first_document_owner` réécrits dessus ; la facture d'un règlement y est jointe. Test de parité avec l'ancien comportement. Gate complet à chaque passe (repository du socle).
- **Écartées** : boucler sur `reversal_blockers` (N+1) ; une seconde requête de propriété propre à la vue (deux listes qui divergent — C-15-8-5).
- **Réversible** : oui.
- ⚠️ **Révisée à la validation P2 (2026-10-09) — voir C-15-1b-9 et C-15-1b-10** : la refonte est extraite en 15-1b-0 ; signature `&mut MySqlConnection` (et non `executor`) ; le « test de parité avec l'ancien comportement » compare désormais à un oracle **indépendant** (requête gelée en module de test, valeurs écrites), la version P1 comparant deux sorties de la même fonction.

## C-15-1b-3 — 15-1b (validation P1, R2/R6/F-3) : le motif d'une ligne ouverte est à X, l'état de sa pièce est d'aujourd'hui
- **Contexte** : `partiallySettled` et `paidWithoutSettlementEntry` se calculaient sur l'état présent de la facture, présentés comme motif « à X » ; précédence implicite ; `letteringAfterAsOf` redondant.
- **Retenu** : deux champs — `reason` (`unlettered` | `letteredAfterAsOf`, à X, depuis le seul grand livre) et `documentState` (`paidWithoutSettlementEntry` > `nothingDue` > `partiallySettled` > `unpaid`, aujourd'hui, factures client seules) avec `amountDue` par les constantes du reste dû ; `letteredOn` remplace `letteringAfterAsOf`. Sélection (requête A) sans pièce, enrichissement (requête B) sur la page.
- **Écartées** : calculer le reste dû « à X » (forker `INVOICE_AMOUNT_DUE_DERIVED_SQL`, interdit depuis #416) ; réserver les motifs de pièce à `asOf` = aujourd'hui (l'écran de clôture perdrait l'information).
- **Réversible** : oui jusqu'au tag v0.13.0.

## C-15-1b-4 — 15-1b (validation P1, R3/R5/F-5/F-6) : les propositions — R7 filtrée, plafond après R5, contrat écrit
- **Contexte** : le moteur proposait des paires que `POST /letterings` refuserait (`LETTERING_ALL_LINES_IN_CLOSED_PERIODS`) ; le plafond ne disait pas ce qu'il comptait ; aucun JSON.
- **Retenu** : une paire tout entière hors période ouverte n'est pas proposée (prédicat pur partagé avec `any_line_in_open_period`, lu sans verrou) ; plafond de 2 000 sur les candidates **après** les filtres « ouverte » et R5 ; `limit` défaut 100, plafond 500 ; réponse `{accountId, candidateCount, total, limit, items[{amount, daysApart, reversalPair, debit, credit}]}` ; paires contre-passation/origine en tête.
- **Écartées** : proposer et marquer `acceptable: false` (l'écran montrerait des paires inutilisables) ; plafond avant R5 (422 sur un compte sans candidate).
- **Réversible** : oui jusqu'au tag v0.13.0.
- ⚠️ **Révisée à la validation P2 (2026-10-09)** : le « prédicat pur partagé avec `any_line_in_open_period` » n'est plus créé par la 15-1b en `kesh-core` ; c'est **la** factorisation de la 15-1a2-i (`open_period_rule` / `OpenPeriodRule::line_in_open_period` / `lines_in_open_period`, `letterings.rs`, publique, rend `Result`), employée telle quelle (C-15-1a2-18). Le sous-objet de ligne gagne `fiscalYearName` ; le 422 porte le plafond dans sa variante.

## C-15-1b-5 — 15-1b (validation P1, L6/F-13) : entrées de la route des postes ouverts
- **Retenu** : `asOf` parsé comme `dateFrom` des écritures (400 `VALIDATION_ERROR`) ; défaut `Utc::now().naive_utc().date()`, convention de la balance âgée (l'écart UTC la nuit est hérité, non corrigé ici) ; aucune borne de date ; `limit` 50 par défaut, `clamp(1, 500)`, `offset.max(0)`, renvoyés dans la réponse comme `ListResponse`.
- **Écartées** : date suisse (`chrono-tz` absent du dépôt — un changement transversal, pas propre à cette route) ; refuser un `asOf` futur (des écritures peuvent être datées dans le futur).
- **Réversible** : oui.

## C-15-1b-6 — 15-1b (validation P1, L8/F-11/F-12) : le code de lettrage au Grand livre, en JSON seulement
- **Retenu** : `LedgerLine.lettering_code` sérialisé `letteringCode` ; CSV (colonnes écrites une à une) et PDF inchangés ; changement de contrat additif inscrit au CHANGELOG ; type TypeScript laissé à la 15-1c.
- **Écartées** : colonne CSV/PDF (élargit le rapport sans demande nommée ; reprenable à la 15-1c).
- **Réversible** : oui.

## C-15-1b-7 — 15-1b (validation P1, R7/F-7) : `letterable` par une règle pure, en lot pour la liste
- **Contexte** : `AccountResponse` se construit par `From<Account>` dans cinq handlers ; `is_letterable_account` fait une requête par compte.
- **Retenu** : prédicat pur `is_letterable(account_type, bank_linked)` ; `letterable_account` et un lot `letterable_account_ids(conn, company_id)` l'appellent, avec la même expression SQL en constante ; `letterable: bool` dans les cinq réponses (lot pour la liste, requête unitaire ailleurs) ; `AccountResponse::new(account, letterable)`.
- **Écartées** : appel unitaire par compte dans la liste (N+1) ; seconde implémentation SQL de R4 ; champ présent sur la liste seule (contrat variable selon la route).
- **Réversible** : oui jusqu'au tag v0.13.0.

## C-15-1b-8 — 15-1b (validation P1, L7/F-10) : les postes ouverts vivent dans `kesh-db`
- **Retenu** : `letterings::open_items` en `kesh-db` — la lettrabilité et la propriété y vivent ; `kesh-report` n'est touché que pour `LedgerLine`.
- **Écartées** : `kesh-report` (il devrait importer les deux règles de `kesh-db`, pour un « rapport » qui est une liste paginée).
- **Réversible** : oui.

## C-15-1a2-1 — 15-1a2 (validation P1, F-9) : découpée en 15-1a2-i (pièces clientes) et 15-1a2-ii (fournisseurs et rattrapage)
- **Contexte** : la fiche touchait au moins sept modules de premier niveau (lettrage, règlements clients, avoirs, factures fournisseurs, migrations et rejeu, rapprochement `kesh-api`, i18n) pour un seuil de cinq (`CLAUDE.md`, splitting préventif, premier critère). Décision de l'orchestrateur, autonomie déléguée.
- **Retenu** : fiche mère en index (`split`, corps vidé, numérotation conservée — les renvois des fiches sœurs restent justes) ; **15-1a2-i** : groupe client, synchronisation, périodes (P7), audit, cinq sites dont `accept_one_invoice` et l'avoir, documentation client ; **15-1a2-ii** : fournisseurs (`pay_in_tx` qui couvre `pay` et `confirm_batch`, les deux annulations), rattrapage (deux migrations), rejeu, documentation fournisseur et admin. Dépendance dure : ii après i. Cinq modules chacune. ⚠️ La consigne rangeait `confirm_batch` côté client : il appelle `supplier_invoices::pay_in_tx`, il est en ii.
- **Écartées** : dérogation écrite (garder une fiche de sept modules) ; découpage en trois (client / fournisseur / rattrapage, proposé par F-9) — le rattrapage dépend des deux familles et son test d'accord des deux synchronisations : seul, il n'aurait rien à comparer avant le merge des deux autres.
- **Réversible** : oui avant développement.

## C-15-1a2-2 — 15-1a2 (validation P1, R2 = F-6 ; Reçu points 3, 9, 10) : en période close, la synchronisation s'abstient de lettrer ET de délettrer
- **Contexte** : le mode `System` n'évalue pas la règle des périodes du socle ; la fiche laissait ouvertes l'abstention du rattrapage (points 3, 10) et l'annulation d'un règlement de période verrouillée (point 9), avec AC5/AC6 et la D1 de la 15-1b contradictoires.
- **Retenu** (décision de l'orchestrateur) : la synchronisation évalue elle-même, sans verrou, « au moins une ligne en période ouverte » (prédicat `any_line_in_open_period` factorisé, `lines_in_open_period`) et s'abstient sinon — à la création comme à la dissolution ; le rattrapage applique la même règle. AC5 borné aux pièces dont une ligne de C est en période ouverte ; AC6 admet `AbstainedClosedPeriods` ; AC14 nouveau. Ce que voit l'utilisateur : pièce historique close soldée → ouverte, **non** lettrable à la main (R5) ; annulation d'un règlement de période verrouillée → groupe gardé, miroir ouvert portant le reste dû ; nouveau règlement ensuite → règlement et miroir ouverts, de somme nulle.
- **Heurt signalé** : la consigne disait « lettrable à la main si la règle le permet » — faux pour une ligne de pièce (rang 5, `LETTERING_LINE_OWNED_BY_DOCUMENT`). Le côté « délettrer » produit une facture due à ligne de vente lettrée (exception d'AC5 et d'AC9).
- **Écartées** : poser les groupes clos (le rattrapage écrirait des groupes entièrement clos que la règle `Manual` interdit) ; dissoudre quand même à l'annulation (réécrit la vue « au 31.03 » d'un trimestre verrouillé) ; refuser en amont l'annulation d'un règlement de période verrouillée (refus neuf sur un geste existant).
- **Réversible** : oui jusqu'au tag v0.13.0 ; à rouvrir si la recette juge trop cher le côté « délettrer ».
- ⚠️ **Révisée deux fois à la validation P2 (2026-10-09) — voir C-15-1a2-10.** Le côté « lettrer » (abstention sur une pièce historique entièrement close) est **maintenu** ; le côté « délettrer » est remplacé : d'abord par « délettrer toujours » (écartée : contredit C113 et `user-manual.tex:577-579`), puis par le **refus** du geste qui l'exigerait (rang 2 bis de la file commune). Le « groupe gardé », le « heurt signalé » et l'exception d'AC5/AC9 ci-dessus n'ont plus d'objet.

## C-15-1a2-3 — 15-1a2 (validation P1, R5 = F-2) : un compte non lettrable est sauté, jamais une erreur
- **Contexte** : `create_group_in_tx` exige la lettrabilité y compris en mode `System` ; sans garde, un règlement, un avoir ou un paiement échouerait en 409 de lettrage.
- **Retenu** : `is_letterable_account` avant la primitive, `SyncOutcome::AccountNotLetterable`, aucune erreur — patron de la contre-passation (`journal_entries.rs:2578`). Le SQL de rattrapage recopie le prédicat (`Asset`/`Liability`, pas de compte bancaire). La dissolution n'exige pas la lettrabilité (C104). Exception (b) nommée d'AC5 ; AC13 et AC7.
- **Écartées** : refuser le geste (l'utilisateur ne peut pas lever la cause) ; lettrer quand même hors primitive (violerait R3/R4 du socle).
- **Réversible** : oui.

## C-15-1a2-4 — 15-1a2 (validation P1, R3 = F-1) : rattrapage en deux migrations — documents au rejeu (classe A), paires `reversal` libres exemptées
- **Contexte** : rejouer à chaque import les paires `reversal` relettrerait une paire délettrée à la main (`NULL` choisi) — le critère que `CLAUDE.md` P7 déclare faux. La classe B est interdite (DDL dans `20261009000001`, autre fichier). Décision de l'orchestrateur : paires faites une fois, exemptées du rejeu ; documents au registre si leur classe le permet.
- **Retenu** : **M1** (registre, classe A, migration entière) : groupes `document` client et fournisseur, et paires `reversal` dont l'origine est l'**achat** d'une facture fournisseur (non dissolubles à la main, C106 — un `NULL` n'y est pas un choix ; exemptées, elles resteraient ouvertes ET non lettrables à la main après restauration). **M2** (`EXEMPT_MIGRATIONS`, `Durable`, justification qui ne commence pas par « Hors fenêtre ») : les autres paires `reversal`. Coût assumé : une sauvegarde d'avant la 15-1a2 importée laisse ces paires-là ouvertes, lettrables à la main.
- **Heurt signalé** : la consigne parlait d'une migration (compteurs 76 → 77) ; le registre exige qu'un extrait porte toutes les écritures de sa migration (`extract_carries_every_write_statement_of_its_source_migration`) et interdit qu'une migration soit à la fois au registre et exemptée — d'où deux fichiers, compteurs 76 → **78**, `EXEMPT_MIGRATIONS` 16 → 17. `20261009000001` intacte (P8).
- **Écartées** : une seule migration exemptée en entier (les documents ne seraient plus rattrapés après restauration) ; une seule migration au registre (R3) ; replier le rattrapage dans `20261009000001` (P8, migration mergée).
- **Réversible** : oui jusqu'au tag v0.13.0 (migrations non publiées).

## C-15-1a2-5 — 15-1a2-i (validation P1, R1 = F-5, R6) : signatures et audit — exercice tenu et acteur passés par l'appelant, `document` par une fonction privée
- **Retenu** : `sync_*_in_tx(tx, company_id, id, held_open_fiscal_year_id, actor)` et `dissolve_*_document_group_in_tx(…)` ; l'exercice tenu est celui que le geste verrouille déjà (aucun verrou neuf) ; acteur `Actor { user_id, api_key_id: None }` (écart nommé, comme la contre-passation) sauf `accept_one_invoice` (`actor_api_key_id`). Les primitives gardent leur signature publique ; leur corps passe dans `create_group_inner` / `dissolve_group_inner` (+ `document: Option<&DocumentRef>`), qui ajoutent `documentType`, `documentId`, `documentNumber` aux `details` — rien quand `None`. `letterings_lexical.rs` réaligné sur `*_inner`.
- **Écartées** : paramètre ajouté aux primitives publiques (touche routes, contre-passation et tests) ; champ dans `Mode::System` (`Mode` est `Copy`, le numéro de pièce est une `String`) ; seconde entrée d'audit (deux traces pour un geste).
- **Réversible** : oui.

## C-15-1a2-6 — 15-1a2-i (validation P1, R1, F-3) : appels explicites après le dernier `UPDATE` du geste, pas dans `invoice_settlements::create_in_tx`
- **Contexte** : `create_in_tx` n'a ni acteur ni exercice, et `accept_one_invoice` l'appelle avant son contrôle de version (g) : une synchronisation là verrouillerait les lignes avant la ligne `invoices` (cycle avec `settle_invoice`) et une course sortirait en erreur de lettrage.
- **Retenu** : trois appels explicites (`settle_invoice`, `write_off_invoice` après leur `UPDATE invoices` ; `accept_one_invoice` après (g)) ; l'avoir après la bascule `cancelled` ; le test lexical d'AC8 ferme l'inventaire (toute fonction qui appelle `create_in_tx` appelle ensuite la synchronisation). Erreur de synchronisation dans `accept_one_invoice` → `FailedProposal` (`LETTERING_CONCURRENT_CHANGE`, `INTERNAL_ERROR` + `tracing::error!`, `DATABASE_ERROR`).
- **Écartées** : enveloppe `create_and_sync_in_tx` (même défaut d'ordre pour le rapprochement) ; déplacer (g) avant (f) (le reste dû après règlement en dépend).
- **Réversible** : oui.

## C-15-1a2-7 — 15-1a2-i (Reçu points 6 et 19) : la facture créditée ET réglée (héritée) reste ouverte
- **Retenu** : `Σ C ≠ 0` → aucun groupe ; créance, avoir et règlement ouverts, non lettrables à la main ; les textes posés par la 15-1a-i (« le règlement reste ouvert au compte débiteurs », quatre locales, replis, manuel, `api-external.md`) restent justes — aucun réécrit. État produit par des données héritées seulement (l'avoir est refusé sur une facture réglée).
- **Écartées** : exception à R5 pour ce règlement ; groupe `document` qui inclurait la contrepartie de l'avoir (somme non nulle au compte débiteurs, viole la règle du groupe).
- **Réversible** : oui.

## C-15-1a2-8 — 15-1a2-i (validation P1, R9) : message `LETTERING_IS_DOCUMENT` neutre
- **Contexte** : « annulez le règlement plutôt que de délettrer » est faux pour un groupe facture + avoir (aucun règlement, aucun avoir annulable).
- **Retenu** : « Ce lettrage est celui d'une pièce : il suit ses règlements et son avoir, il ne se défait pas à la main. » — quatre `.ftl` et le repli Rust.
- **Écartées** : variante par `documentType` (le refus n'a pas la pièce sous la main au rang 1 sans lecture de plus).
- **Réversible** : oui.
- ⚠️ **Déplacée à la validation P3 (2026-10-09)** : la réécriture du message est portée par la **15-1a2-0** (D5, AC9), avec les autres textes et `kesh-api/src/errors.rs` (C-15-1a2-19).

## C-15-1a2-9 — 15-1a2-i : pas de synchronisation après le `DELETE` de l'annulation d'un règlement
- **Retenu** : la dissolution avant la contre-passation suffit ; après le retrait d'un règlement de montant positif, `Σ C` vaut le reste dû, positif : C ne qualifie jamais. Le Reçu point 12 (C116, marques rendues par la contre-passation) est donc sans objet.
- **Écartées** : l'appel ② de la fiche d'origine (inutile, et il exigerait un exercice tenu qui couvre une ligne de C — celui du règlement retiré ne la couvre plus).
- **Réversible** : oui.

## C-15-1a2-10 — 15-1a2-i / 15-1a2-ii (validation P2) : refus plutôt qu'abstention au délettrage — C-15-1a2-2 révisée deux fois
- **Contexte** : l'abstention « partout » de C-15-1a2-2 gardait un groupe `document` sans ligne en période ouverte quand l'annulation d'un règlement l'aurait dissous. La validation P2 (Opus ×2 sur chacune des trois fiches) en a tiré huit findings MEDIUM+ nés de cette seule décision : facture due à ligne de vente lettrée (exception d'AC5 et d'AC9), paires ouvertes à jamais, `Invariant` (500) au premier règlement après un déverrouillage (15-1a2-i R-1, M-1, M-2, M-3 ; 15-1a2-ii R-6), groupe orphelin indissoluble sur facture fournisseur annulée payée (15-1a2-ii F2-1 HIGH = R-1), et un AC de la 15-1b qui prescrivait l'inverse (15-1b R-1 = F-1 HIGH). Décision de l'orchestrateur.
- **Historique, tel qu'il s'est déroulé** : (1) P1 — abstention partout ; (2) révision 1, **écartée** — délettrer toujours, sans évaluer la règle (le mode `System` le permet) : contredit C113, qui fait entrer le verrou dans « période ouverte » précisément pour que la vue « au 31.03 » d'un trimestre verrouillé ne soit plus réécrite, et le manuel livré (`user-manual.tex:577-579` : « un groupe dont toutes les lignes sont dans la période verrouillée (ou dans des exercices clôturés) ne se lettre ni ne se délettre ») ; (3) révision 2, **retenue** — refus.
- **Retenu** : lettrer une pièce **s'abstient** toujours quand la règle des périodes l'interdit (pièce historique entièrement close) ; tout geste qui exigerait de **dissoudre** un groupe `document` dont aucune ligne n'est en période ouverte est **refusé** — annulation d'un règlement ou d'un solde client, dé-rapprochement d'une facture, annulation d'un paiement fournisseur, annulation d'une facture fournisseur payée. Rang 2 bis de la file commune `settlement_entry_cancel_blocker` (C-15-1a2-12), code réemployé (C-15-1a2-11), remède : un administrateur déverrouille (`unlock_books`, motif obligatoire) ou rouvre les exercices. Conséquences : plus de groupe gardé, plus d'exception de période à AC5/AC9, plus d'`Invariant` après déverrouillage (vérifié au code : aucun autre chemin ne dissout un groupe hors période — `Manual` évalue R7, la contre-passation et le rattrapage ne dissolvent rien —, sauf la tolérance nommée d'un `lock_books` concurrent), plus de groupe orphelin fournisseur ; la 15-1b retire l'AC12 (b) « l'annulation sous verrou fait réapparaître » et écrit la vraie stabilité.
- **Écartées** : abstention (états définitifs que rien ne répare) ; délettrer toujours (ci-dessus) ; abstention plus une issue de dette pour les paires définitives (une dette qu'on fabrique).
- **Coût** : un refus neuf sur des gestes existants — pesé faible : v0.13.0 non taguée, aucune comptabilité réelle tenue dans Kesh, forme exacte du rang 2 existant. Un sixième module pour la 15-1a2-i (C-15-1a2-13).
- **Réversible** : oui jusqu'au tag v0.13.0.
- ⚠️ **Validation P3 (2026-10-09)** : le refus est **extrait** dans la story préalable **15-1a2-0** (C-15-1a2-19) ; le « sixième module » de la 15-1a2-i et sa dérogation (C-15-1a2-13) disparaissent. La décision de fond (refus au délettrage) est inchangée.

## C-15-1a2-11 — 15-1a2-i (validation P2) : le refus du rang 2 bis réemploie `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`
- **Contexte** : la consigne de l'orchestrateur disait « nouveau code d'erreur ». `SettlementCancelBlocker::code` (`kesh-db/src/errors.rs:399-418`) pose : « ⚠️ **Tous** ces codes réemploient ceux d'états du monde déjà nommés […] : un même fait ne reçoit pas un second nom » (précédent : `INVOICE_CREDITED` repris d'`UnvalidationBlocker`).
- **Retenu** : un **refus** neuf — variante `DocumentLetteringInClosedPeriods`, trois clés de texte par famille × quatre locales — dont le **code** est `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, l'état « toutes les lignes du groupe sont en période close » que le socle nomme déjà (15-1a-i R7). Heurt avec la consigne signalé ; l'esprit (un refus nommé, ses textes, son écran) est tenu.
- **Écartées** : un code neuf (`SETTLEMENT_LETTERING_FROZEN`…) — second nom pour un même fait, contre la règle écrite du type.
- **Réversible** : oui jusqu'au tag v0.13.0 (renommer un code avant publication).

## C-15-1a2-12 — 15-1a2-i / 15-1a2-ii (validation P2) : place et forme du rang 2 bis
- **Retenu** : évalué **dans la file commune** sur l'écriture examinée (règlement, rapprochement, achat), par `letterings::document_group_frozen_by_periods` (sur `open_period_rule`) — une évaluation, deux lecteurs (prédicteur de l'écran, geste). Placé **entre** `FiscalYearClosed` (2) et `MatchedBankTransaction` (3) : après 2 (l'exercice clos se nomme par son motif, même remède), avant 3 (« annulez d'abord le rapprochement » serait vain, le dé-rapprochement étant refusé par ce même rang). Le dé-rapprochement le refuse dans **sa** famille (`ReconciliationNotCancellable`) avant de défaire le lien. La 15-1a2-i livre la variante, la file, les trois familles de textes, les quatre locales et l'écran (y compris le cas fournisseur, dormant) ; la 15-1a2-ii ajoute le rang aux refus des deux gestes fournisseurs. La dissolution elle-même n'évalue pas la règle (une seule garde par motif).
- **Écartées** : évaluer dans chaque dissolution (deux gardes du même motif, et une erreur hors famille pour le dé-rapprochement) ; placer le rang après 5 (ferait annoncer au rang 3 un remède vain).
- **Réversible** : oui.
- ⚠️ **Révisée à la validation P3 (2026-10-09)** : la variante, la file, les textes, l'écran **et** les refus des deux gestes fournisseurs sont livrés par la **15-1a2-0** (C-15-1a2-19) — la 15-1a2-ii n'ajoute plus rien aux refus. Le dé-rapprochement lie le motif (`blocker @ (FiscalYearClosed | DocumentLetteringInClosedPeriods)`) au lieu de le coder en dur (P3, L-2).

## C-15-1a2-13 — 15-1a2-i (validation P2) : dérogation à la règle de splitting — six modules
- **Contexte** : le refus ajoute l'écran et ses textes ; la fiche passe de cinq à six modules. Décision de l'orchestrateur : dérogation, sauf autre débordement constaté.
- **Retenu** : dérogation écrite dans la fiche (§ « Dérogation règle de splitting ») : un refus nommé de plus, sans logique neuve hors du blocker existant ; le séparer de la dissolution laisserait entre deux merges l'état défaillant ; revue mécanique des textes. Aucun autre débordement : les gestes fournisseurs restent à la 15-1a2-ii, qui garde cinq modules (écran et textes fournisseurs livrés par la 15-1a2-i).
- **Écartées** : une sous-story « refus » (voir ci-dessus) ; porter l'écran fournisseur à la 15-1a2-ii (elle passerait à six modules à son tour).
- **Réversible** : oui avant développement.
- ⛔ **RETIRÉE à la validation P3 (2026-10-09)** — finding F-1 (lentille F) : l'argument (2) n'examinait qu'un ordre de merge, le refus **après** la synchronisation ; l'ordre inverse, **le refus d'abord, dormant**, ne laisse aucun état défaillant et se teste en SQL brut. Décision de l'orchestrateur : story préalable **15-1a2-0** (C-15-1a2-19). Le décompte « six » était en outre un regroupement par domaine que la règle n'emploie pas (L-7) : voir C-15-1a2-21.

## C-15-1a2-14 — 15-1a2-i (validation P2, M-4) : `LETTERING_CONCURRENT_CHANGE` n'entre pas dans `failed[]`
- **Contexte** : la table per-proposal de P4 le mappait sur son propre code, sans libellé à l'écran (`failed-proposal-label.ts`, 28 codes en dur dans son test).
- **Retenu** : le code naît du seul `UPDATE` final de la primitive, sur des lignes tenues `FOR UPDATE` par l'acte 1 de la même transaction (`letterings.rs:739`, `:850`, `CLIENT_FOUND_ROWS`) : **inatteignable par construction**. Mappé sur `INTERNAL_ERROR` (+ `tracing::error!`), déjà libellé ; aucun changement frontend ; aucun ajout aux tableaux de refus des routes de règlement (L-3, même motif).
- **Écartées** : libellé ×4 et décompte 28 → 29 (un sixième module pour un code qui ne sort jamais).
- **Réversible** : oui.

## C-15-1a2-15 — 15-1a2-ii (validation P2, R-3 = F2-3) : la découverte d'une facture fournisseur dépend de son statut
- **Retenu** : `paid` → ancre + règlement ; `open` → ancre seule ; `cancelled` → ensemble **vide** (`Unchanged`, sans lecture verrouillante) — l'achat d'une facture annulée reste possédé et R6 l'a lettré `reversal`, que l'étape 2 commune déclarerait `Invariant`.
- **Écartées** : tolérer `reversal` sur l'ancre d'une facture annulée dans l'étape 2 (une exception de plus dans l'algorithme commun) ; restreindre AC6 (e) aux factures non annulées (cache le cas au lieu de le traiter).
- **Réversible** : oui.

## C-15-1a2-16 — 15-1a2-ii (validation P2, R-2 = F2-2, F2-5, R-5) : AC6 en deux régimes, classe A justifiée honnêtement
- **Retenu** : AC6 (d) classe chaque pièce au moment du rattrapage — une ligne en période ouverte : identité stricte ; aucune : abstention attendue, assertée pièce par pièce. La justification de classe A de M1 ne dit plus « no-op strict » : le rejeu ne délettre ni ne réécrit jamais, mais peut **lettrer** une pièce restée ouverte à bon droit (exercice rouvert depuis l'abstention, compte devenu lettrable) — ce que la synchronisation poserait aujourd'hui, pas un choix écrasé ; AC16 (c) borné et (d) ajouté, manuel d'administration nuancé. AC16 (c) asserte `rows_affected == 0` pour M1 sur une base à jour portant des pièces lettrées — le seul discriminant d'une garde manquante (`CLIENT_FOUND_ROWS`) ; `post_restore_class_a.rs` compte par entrée.
- **Écartées** : classe B pour M1 (interdite : DDL dans un autre fichier) ; exempter M1 (les pièces ne seraient plus rattrapées après restauration).
- **Réversible** : oui jusqu'au tag v0.13.0.

## C-15-1a2-17 — 15-1a2-ii (validation P2, R-8 = F2-6) : `documentNumber` nul pour une facture fournisseur sans numéro
- **Retenu** : `null` (et `documentId` porte l'identifiant) — règle de `reversal_blockers`, « le numéro accompagne l'identifiant quand il existe ».
- **Écartées** : le repli de `pay_in_tx` (l'identifiant en chaîne) — c'est un libellé d'écriture, pas un numéro de pièce.
- **Réversible** : oui.
- ⚠️ **Complétée à la validation P3 (2026-10-09)** : la clé `documentNumber` est **présente** et vaut `null` (jamais omise) ; `DocumentRef` est défini par la 15-1a2-i (C-15-1a2-22) avec `number: Option<String>` ; la valeur de `documentType` devient `"supplierInvoice"`.

## C-15-1a2-18 — 15-1a2-i / 15-1b (validation P2, R-6, L-1 ; 15-1b R-3, L-2) : UNE factorisation du prédicat des périodes, publique, dans `letterings.rs`
- **Retenu** : `OpenPeriodRule` (champs privés), `open_period_rule(conn, company_id, &fiscal_year_ids) -> Result<OpenPeriodRule, DbError>` (lecture sans verrou), `OpenPeriodRule::line_in_open_period(fiscal_year_id, entry_date)`, `lines_in_open_period(conn, company_id, &[(fy, date)]) -> Result<bool, DbError>` ; le mode `Manual` (`any_line_in_open_period`) et elles appellent le même prédicat par ligne, privé. La 15-1b l'emploie telle quelle ; sa seconde factorisation en `kesh-core` est supprimée.
- **Écartées** : prédicat pur en `kesh-core` créé par la 15-1b (seconde factorisation, après la livraison de la première) ; `-> bool` (la fonction lit la base).
- **Réversible** : oui.
- ⚠️ **Validation P3 (2026-10-09)** : la factorisation est livrée par la **15-1a2-0** (D1), et non plus par la 15-1a2-i, qui l'emploie ; `open_period_rule` lit aussi la `start_date` des exercices (argument de `find_later_closed`) ; borne stricte testée (jour de la borne clos, lendemain ouvert).

## C-15-1b-9 — 15-1b (validation P2, signal D5) : la propriété des lignes par lot devient la story 15-1b-0
- **Contexte** : signal D5 levé — P1 0 HIGH → P2 1 HIGH, né d'une remédiation (la collision des remédiations P1 de la 15-1b et de la 15-1a2) ; deux MEDIUM sur la refonte de `reversal_blockers` (R-4 = F-5 signature et appelants, F-4 parité verte par construction). Décision de l'orchestrateur.
- **Retenu** : extraire la refonte du socle — `document_owners` par lot, `reversal_blockers` et `first_document_owner` réécrits dessus, signature, inventaire des appelants, parité par oracle indépendant, mutations éprouvées — dans une story **patron** 15-1b-0, dont la 15-1b dépend ; la 15-1b garde la vue et les propositions (numéros T2 et test 8 conservés, marqués « déplacés »). Le découpage proposé au Change Log P1 (15-1b-i postes ouverts / 15-1b-ii propositions) n'isolait aucun des findings : abandonné.
- **Écartées** : garder la refonte dans la 15-1b (gate complet à chaque passe de la vue, et la garde verte par construction) ; le découpage vue/propositions de P1.
- **Réversible** : oui avant développement.

## C-15-1b-10 — 15-1b-0 (validation P2 de la 15-1b, R-4 = F-5, F-4) : signature, instantané, oracle
- **Retenu** : `document_owners(conn: &mut MySqlConnection, …)` et `reversal_blockers(conn: &mut MySqlConnection, …)` (patron C-15-8-24) ; deux requêtes, un instantané sous transaction ; la route de contre-passation lit dans une transaction de lecture ; les tests sur pool passent une connexion acquise. Au plus un propriétaire par type, le plus petit `id` (rend déterministe le `LIMIT 1` sans `ORDER BY` des transactions bancaires). Parité contre la requête **gelée** de `056997b0` en module de test et contre des valeurs écrites à la main ; trois mutations éprouvées.
- **Écartées** : fragment SQL partagé inclus dans la requête unique de `reversal_blockers` (signature inchangée, mais deux textes SQL à tenir d'accord — la parité redevient la seule garde) ; garder `E: Executor` (impossible : deux requêtes).
- **Réversible** : oui.

## C-15-1b-11 — 15-1b (validation P2, R-1 = F-1 HIGH, F-2, L-3) : la stabilité « au X » réécrite
- **Retenu** : stable pour `X` en période close (exercice clôturé, ≤ borne, ou exercice suivi d'un clôturé) **tant qu'aucun administrateur ne déverrouille (`unlock_company_books`), ne rouvre (`reopen_fiscal_year`) ni ne restaure** ; non stable au-dessus. AC12 : gestes **réussis** en (a), refus puis déverrouillage en (b), délettrage manuel au-dessus de la borne en (c).
- **Écartées** : « non stable sous le verrou » (version P1, contraire au refus de la 15-1a2-i) ; « stable pour un X dans un exercice clos » sans réserve (ignorait la réouverture et le déverrouillage).
- **Réversible** : oui jusqu'au tag v0.13.0 (texte d'`api-external.md`).

## C-15-1b-12 — 15-1b (validation P2, LOW) : reste dû au centime, avoir sans état, ordre des refus
- **Retenu** : `amountDue` par `amount_due_to_centime`, seuils de `documentState` sur cette valeur ; `documentState` nul pour une ligne d'avoir ; ordre des refus : paramètres (400) → compte (404) → lettrabilité (409) ; 422 `{ max }` interpolé.
- **Écartées** : reste brut (#490 : 0,004 classé `partiallySettled`) ; état de la facture recopié sur la ligne d'avoir (deux lignes disant la même chose).
- **Réversible** : oui jusqu'au tag v0.13.0.

## C-15-1a2-19 — 15-1a2-i / 15-1a2-ii (validation P3, F-1) : le refus du rang 2 bis devient la story préalable 15-1a2-0
- **Contexte** : la dérogation C-15-1a2-13 gardait le refus dans la 15-1a2-i au motif qu'une sous-story « refus » mergée **après** la synchronisation laisserait un état défaillant ; la lentille F (Sonnet, P3) a montré que l'ordre inverse n'avait pas été examiné. Décision de l'orchestrateur.
- **Retenu** : story **15-1a2-0 « le lettrage se fige avec la période — les annulations qui le dissoudraient sont refusées »**, mergée **avant** la 15-1a2-i : la règle des périodes (`open_period_rule`, D1), la variante `DocumentLetteringInClosedPeriods` et `document_group_frozen_by_periods` (D2), les refus des **quatre** gestes — règlement et solde clients, dé-rapprochement, **paiement et facture fournisseurs** (D3) —, le code réemployé, les textes ×4 locales, l'écran, la documentation (listes exhaustives du manuel, table des codes d'`api-external.md`), le message `LETTERING_IS_DOCUMENT` et le doc-comment d'`InvoiceCredited` (D5). **Dormante** : aucun groupe `document` n'existe avant la 15-1a2-i ; éprouvée sur des groupes posés en SQL brut. Ordre : 15-1a2-0 → 15-1a2-i → 15-1a2-ii ; la dérogation C-15-1a2-13 est retirée. Les gestes **fournisseurs** sont dans la 15-1a2-0 (et non à la 15-1a2-ii) : la file étant commune, leurs prédicteurs héritent du rang dès cette story, et des gestes qui ne le refuseraient pas feraient annoncer un motif que le clic ne refuse pas. Le message `LETTERING_IS_DOCUMENT` y vient aussi, pour qu'une seule story touche les textes et `kesh-db/src/errors.rs`.
- **Écartées** : garder la dérogation en la réécrivant (le refus d'abord est sans risque : rien ne la justifie) ; un découpage serveur / écran de la 15-1a2-0 elle-même (possible, l'écran tolérant un code inconnu tant qu'aucun groupe n'existe ; non retenu sans arbitrage — signal déclaré, C-15-1a2-21).
- **Réversible** : oui avant développement.

## C-15-1a2-20 — 15-1a2-0 (validation P3 de la 15-1a2-i, F-6, L-6) : le remède écrit est précis, la date n'est pas dans le refus
- **Retenu** : le texte (trois familles ×4 locales), le manuel et `api-external.md` disent que la borne doit reculer **avant la date de la ligne la plus récente du lettrage** (en pratique : du dernier règlement), qu'un recul qui la laisse à ou après cette date ne lève rien, que la clôture se lève en rouvrant du plus récent au plus ancien, et « si les deux s'appliquent, les deux » — jamais « ou » seul. La variante reste **sans champ** ; aucune date dans `details`.
- **Écartées** : porter la date dans `details` (variante à champ dans un enum dont tous les rangs sont sans champ, `cancelBlockedBy` ne transporte qu'un code ; la date se lit sur la pièce) ; « déverrouiller la période » sans précision (l'administrateur peut croire avoir levé le refus).
- **Réversible** : oui jusqu'au tag v0.13.0.

## C-15-1a2-21 — 15-1a2-0, -i, -ii, 15-1b-0, 15-1b (validation P3, L-7 = F-1, F L-13) : les modules se comptent aux DEUX grains de la règle
- **Contexte** : la 15-1a2-i comptait « par domaine » (six), grain que la règle de splitting n'emploie pas ; au grain des modules métier, c'était plus de dix.
- **Retenu** : chaque fiche écrit les deux comptes, comme la 15-1a-i et la 15-1a-ii : « crates Rust, packages npm » (le seuil s'y lit d'abord) et « modules métier de premier niveau » (patron `kesh-api/routes/invoices` : un repository, un module de routes, un dossier `features/` chacun) ; la documentation, qui n'est ni crate, ni package, ni module métier, est déclarée à part. Résultats : 15-1a2-0 — 4 et **11** (dont 7 mécaniques ; **signal déclaré**) ; 15-1a2-i — 2 et 5 ; 15-1a2-ii — 1 et 4 ; 15-1b-0 — 2 et 4 ; 15-1b — 5 et **8** (dont 4 mécaniques ; **signal déclaré**). Le dépassement au grain fin est déclaré à l'orchestrateur, non dérogé en silence.
- **Écartées** : le compte par domaine (non conforme) ; ne donner que le grain le plus favorable.
- **Réversible** : sans objet (méthode de compte).

## C-15-1a2-22 — 15-1a2-i (validation P3, F-7 = L-1 ; 15-1a2-ii R3-4 = F3-6) : `DocumentRef` défini une fois, sur les valeurs de la vue
- **Retenu** : `pub struct DocumentRef { pub document_type: &'static str, pub id: i64, pub number: Option<String> }` dans `letterings.rs` (15-1a2-i P3) ; `audit_details` émet `documentType`, `documentId`, `documentNumber` (clé présente, `null` sans numéro) ; valeurs `"invoice"` et `"supplierInvoice"` — celles de `document.type` de la vue des postes ouverts (15-1b, `DocumentKind::as_str`, 15-1b-0) — et non plus `"supplier_invoice"`. Précédence des issues : `AccountNotLetterable` avant `AbstainedClosedPeriods`.
- **Écartées** : laisser la forme à deviner (la 15-1a2-ii devrait la changer) ; deux vocabulaires (snake_case à l'audit, camelCase à la vue) pour une même pièce.
- **Réversible** : oui jusqu'au tag v0.13.0.

## C-15-1b-0-1 — 15-1b-0 (validation P1, R1 = F-1) : la fiche d'écriture lit ce qu'on peut en faire dans UNE transaction de lecture
- **Contexte** : `modification_blocker` lit sur `pool.acquire()` (six lectures en autocommit, `journal_entries.rs:1203-1233`), appelée par la route même que la refonte mettait en transaction.
- **Retenu** : `modification_blocker(conn: &mut MySqlConnection, …)` (seul appelant : la route) ; la route ouvre **une** transaction de lecture et y lit `reversal_blocker`, `reversed_by` et `modification_blocker`. `find_by_id` (son propre lot de requêtes, `&MySqlPool`) reste hors : écart préexistant, assumé — l'en-tête n'est pas un motif. La propriété repose sur l'isolation par défaut (`REPEATABLE READ`), que Kesh ne configure pas : écrit.
- **Écartées** : `modification_blocker` ouvrant sa propre transaction (deux instantanés pour la même réponse) ; assumer l'écart sans rien changer (le motif de gel et le motif de contre-passation pourraient se contredire).
- **Réversible** : oui.

## C-15-1b-0-2 — 15-1b-0 (validation P1, R4 = F-5, R2 = F-4, R3 = F-2) : `DocumentKind` porte ses trois correspondances ; un `UNION ALL` par tranche
- **Retenu** : `DocumentKind::reversal_blocker()`, `blocks_manual_lettering()`, `as_str()` publics — la table type → motif, la liste R5 et les chaînes sérialisées écrites **une** fois, employées par `reversal_blockers`, `first_document_owner` et la 15-1b ; test à table fermée. Forme : une instruction par tranche de 500 (`UNION ALL` de cinq blocs, plus petit `id` par dérivée `MIN` jointe en retour, identifiant et numéro de la même ligne) ; mesure par le delta de `Com_select` sur la même connexion, étalonné ; écriture absente = sans propriétaire. Oracle : fixture SQL directe avec une écriture aux cinq types, `NotFound` comparé ; quatre mutations, dont une sur `blocks_manual_lettering`. Module gelé sous `tests/document_owners/`, inclus par `#[path]` (pas `tests/common/`).
- **Écartées** : cinq requêtes par tranche (15 pour 1 200) ; compteur de requêtes par journalisation (non disponible sur les pools de `#[sqlx::test]`) ; liste R5 privée à `first_document_owner` (la 15-1b l'aurait recopiée).
- **Réversible** : oui.

## C-15-1b-13 — 15-1b (validation P3, LOW) : propositions sans `offset`, vue sans audit de lecture, tri du Grand livre
- **Retenu** : `GET …/lettering-proposals` n'a pas d'`offset` (l'écran accepte puis recharge ; `total` dit ce qui reste) ; les deux routes n'émettent aucun audit de lecture (écran de travail du lettrage, comme `GET /letterings/{key}` et `GET /accounts` ; les audits de lecture restent aux rapports) ; tri de la vue = celui du Grand livre (date, exercice, numéro, `line_order`, `lineId`) ; R7 filtrée avant le glouton ; `open_items` ouvre sa propre transaction.
- **Écartées** : `offset` sur les propositions (un glouton paginé change de résultat à chaque acceptation) ; audit best-effort de la vue (bruit au journal sans usage de contrôle identifié).
- **Réversible** : oui jusqu'au tag v0.13.0.
