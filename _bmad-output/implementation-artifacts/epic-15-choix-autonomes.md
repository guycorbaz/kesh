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
