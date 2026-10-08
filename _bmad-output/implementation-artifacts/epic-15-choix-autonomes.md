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
