# Story 58-3 : Manuel et guide de l'API — écarts avec le code (#730, #748, #767, #814, #1061, #1067, #1074, #1076, #1087, #1405)

Status: ready-for-dev

*Fiche rédigée le 2026-11-26 (Sonnet 5.5), worktree `kesh-58-3`, branche `story/58-3-manuel-guide-api-ecarts` partie de
`epic/58-nettoyage` à `ee684a69f`, par le workflow `bmad-create-story`, en autonomie. Premier epic de nettoyage (décision de Guy du
2026-11-26), release du vendredi 2026-12-04 (numéro attribué au tag). **Aucune migration, aucune règle métier, aucun changement d'API.** Texte
seulement : manuel d'administration et manuel utilisateur (`.tex`), guide de l'API, `.env.example`, un commentaire de compose, l'audit
d'idempotence, les chaînes de justification de `post_restore.rs` (voir D2) et un doc-comment de `config.rs` (seuls fichiers de `crates/`). Aucune compilation ni aucun
gate lancé pour écrire la fiche ; les deux manuels ont été construits **dans une copie** du dépôt (journal d'Overfull cité plus bas), rien
n'est commité hors de ce fichier, du statut `drafted` et du prompt de validation.*

## Story

En tant qu'administrateur ou utilisateur qui lit le manuel et le guide de l'API pour décider ou diagnostiquer, je veux que dix phrases
que le code dément (un rôle dit « sans mutation » qui écrit au journal, un fichier de log qui n'existe pas sous ce nom, un « Elle » sans
antécédent, un champ de formulaire qui n'existe pas, …) disent ce que Kesh fait, afin de ne plus chercher ce que le texte promet à tort.

## Contrôle de couverture (règle du 2026-11-26 : on vérifie qu'une issue n'est pas déjà couverte avant de la retenir)

Pour chaque issue : l'écart relu **au texte** (`grep -nF` de la chaîne copiée) et **au PDF** (flatten de
`docs/manual/fr/{admin,user}-manual.pdf`, `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, comptes d'occurrences de la chaîne), les
commits qui la citent (`git log --grep '#N'` sur `origin/main` et `origin/epic/58-nettoyage` : aucun ne la corrige, #730 et #748 ne
sont citées que par les commits qui les ont **relevées**, `89d687045` et `b723e3d96`), les issues voisines (`gh issue list --state all
--search`, trois requêtes : `kesh.log`, `report.generated Consultation`, `Compte comptable par défaut`, plus le titre des 690 issues
ouvertes de `kesh-gate-logs/e58/all.json` : aucun doublon ; #1070 touche la même règle d'ambiguïté mais pas ces phrases).

| # | Écart au texte | Au PDF | Verdict |
|---|---|---|---|
| #730 | `admin-manual.tex:1688` et `:3509` | `aucune mutation` ×1 | **gardée** (deux sites, trois écritures) |
| #748 | `migrations-idempotence-audit.md:85` et `:89` ; `post_restore.rs:461` et `:484` | — (hors manuels) | **gardée** ; sites voisins découverts (D2) ; avec #767 (`config.rs`), seule à toucher `crates/` |
| #767 | `admin-manual.tex:1031`, `:1045` ; `.env.example:350` ; `docker-compose.yml:91` ; descriptions « Chemin complet du fichier de log » : `admin-manual.tex:1037`, `.env.example:330`, `config.rs:1346` | `kesh.log` ×5 (dont 3 justes) | **gardée, périmètre resserré** : quatre des sept sites du fichier d'epic sont des valeurs de variable, justes |
| #814 | `admin-manual.tex:3803`, `:3922` (1), `:3913` (2), `:3791` (3), `:3805` (4) | `relecture ne rend que` ×1, `donc éprouvé` ×1 | **gardée** (les lignes de l'issue ont bougé de 560 lignes ; le débordement de marge existe toujours, Overfull de 49,7 pt) |
| #1061 | `api-external.md:1415` et `:1416` | — | **gardée** ; **le CHANGELOG ne porte pas la phrase** (grep vide), rien à y corriger |
| #1067 | `user-manual.tex:537` | `restent contrôlés` ×1 | **gardée** |
| #1074 | `user-manual.tex:2969` ; `CHANGELOG.md:132` | `après le séparateur décimal` ×1 | **gardée** ; CHANGELOG par erratum d'intégration (D3) |
| #1076 | `user-manual.tex:2969` ; `CHANGELOG.md:132` et `:133` | `milliers déjà séparés` ×1 | **gardée** ; même erratum ; **même paragraphe que #1074** |
| #1087 | `user-manual.tex:3744` | `Elle reste dans` ×1 | **gardée** |
| #1405 | `user-manual.tex:1385` | `Compte comptable par défaut` ×1 | **gardée** |

**Aucune issue ne sort**, aucune ne révèle un défaut du code : pour chacune, le code est juste et c'est le texte qui diverge.
Trois remarques d'honnêteté sur le fichier d'epic (`epic-58-nettoyage.md`, section 58-3), à ne pas compter comme un défaut de la fiche :
(a) sa ligne « Module » cite `already-booked.ts` (commentaire) : **aucune des dix issues ne le touche**, il n'est pas au périmètre ;
(b) elle dit `docs/manual/{fr,de,it,en}/` : `de/`, `en/`, `it/` ne portent qu'un `README.md` (`ls docs/manual/de docs/manual/en docs/manual/it`),
**seul le français existe** ;
(c) elle ne déclare ni `crates/kesh-api/src/config.rs` (doc-comment, #767) ni `docker-compose.yml` (commentaire, D4), et son critère « aucun code exécutable (hors commentaire) » ne vaut pas pour les deux chaînes de `post_restore.rs` (littéraux compilés, D2) : écarts à consigner à l'epic par l'orchestrateur.

## Ce qui se lit au code (à `ee684a69f`)

### #730 — le rôle Consultation et les clés `read` « n'écrivent rien »

Texte faux (deux sites) :

- `docs/manual/fr/admin-manual.tex:1688` : `Consultation & Lecture seule : consultation des écritures, factures, contacts, produits, rapports — aucune mutation, à une exception près : le \textbf{premier téléchargement} du PDF d'une facture le \textbf{fige} et l'inscrit au journal d'audit, quel que soit le rôle. \\`
- `docs/manual/fr/admin-manual.tex:3509` : `une lecture n'écrit rien, à une exception près : le \textbf{premier téléchargement} du PDF d'une facture --- y compris une simple requête \texttt{HEAD} --- le \textbf{fige} et l'inscrit au journal d'audit au nom de la clé`.

Ce que fait le code (trois écritures omises, la quatrième est l'exception déjà dite) :

1. chaque consultation et chaque export d'un rapport écrit une ligne `report.generated` / `report.exported` au journal d'audit, quel que
   soit le rôle ou la clé — `crates/kesh-api/src/services/rapports.rs:80` (`/// Usage « consultation à l'écran » : trace report.generated.`),
   `:84` (`/// Usage « export » dans un format : trace report.exported.`), `:893` (`"report.generated",`), `:925` (`"report.exported",`) ;
   l'issue citait `routes/reports.rs` : les huit fonctions d'audit vivent aujourd'hui dans `services/rapports.rs` (« la trace — huit fonctions
   d'audit », `:876`), le fond est inchangé ;
2. l'**export global** écrit `exports.global` — `crates/kesh-api/src/routes/exports.rs:13` (`//! 8. Émet l'audit \`exports.global\` best-effort.`),
   `:146` (`async fn emit_global_export_audit(`), appelée `:92`, route « tous rôles authentifiés » (`:37`) ;
3. chaque appel authentifié par **clé d'API** met à jour `last_used_at` de la clé — `crates/kesh-api/src/auth/api_key.rs:215`
   (`if let Err(e) = api_keys::touch_last_used(pool, &key_hash).await {`). Le manuel dit déjà vrai pour le serveur MCP
   (`admin-manual.tex:3621` : « un appel n'écrit que sa trace (la dernière utilisation de la clé, son entrée au journal d'audit) »).

### #748 — « inscrite au registre » et « dans la fenêtre » périmés

- `docs/migrations-idempotence-audit.md:85` (20260828000001) et `:89` (20260910000001) : `**inscrite au registre \`POST_RESTORE_BACKFILLS\` en CLASSE B**`
  — le jeton figure sur **quatre** lignes (`:85`, `:89`, `:99`, `:112`) ; `:99` (classe A, « Retirée du registre par la 32-5a » dit la même ligne) et `:112` (entrée vivante) sont justes. Faux, pour `:85` et `:89`, depuis la Story 25-2-c : les deux
  versions sont dans `RETIRED_BACKFILLS` (`crates/kesh-db/src/post_restore.rs`, versions 20260828000001 et 20260910000001 dans le bloc
  `:271-…`) et exemptées dans `EXEMPT_MIGRATIONS` (`:425-…`) sous « Hors fenêtre depuis 20260917000001 ». Le registre vivant n'a qu'une
  entrée, `20261106000001` (`:210-226`). La ligne 85 dit aussi « **Première entrée du registre depuis que la Story 24-2 l'avait vidé** » :
  vrai à sa date, périmé aujourd'hui, même racine.
- `crates/kesh-db/src/post_restore.rs:461` (20260909000001) et `:484` (20260915000001) : `"Parc vide, et NON « hors fenêtre » — cette migration EST dans la fenêtre d'importabilité, \`
  et `… elle EST dans \` : **faux** : `registry_entries_are_within_import_window` et son symétrique calculent la fenêtre par
  `last_table_creating_migration()` (`:968`), aujourd'hui au moins `20261025000001` (`mcp_keys`) ; **mais ce n'est pas cette borne qui a mis ces deux migrations hors fenêtre** :
  `20260917000001` crée `journal_entry_number_sequences` (`backup.rs:82`, dans `TABLES_TO_TRUNCATE`), **postérieure** à `20260909000001` et `20260915000001`, donc elles sont **hors fenêtre depuis `20260917000001`**
  (Story 25-2-c), **comme `20260828000001` et `20260910000001`**, dont les exemptions le disent déjà (`post_restore.rs:429` et `:438` : `"Hors fenêtre depuis 20260917000001 (journal_entry_number_sequences, Story 25-2-c)`).
  Un backup assez ancien pour les déclencher est refusé au contrôle de couverture de `parse_and_verify`. (La date `20261025000001` de l'issue #748, qui parle de la 32-5a, est celle de la fermeture sur le lettrage, non sur ces deux migrations.)
- **Même symptôme, sites voisins que l'issue ne nomme pas** (grep du symptôme, § *Inventaire*) :
  `migrations-idempotence-audit.md:88` (`non sur la fenêtre, où elle se trouve`), `:90` (`fondement de **parc** et non de fenêtre` — vrai
  du fondement déclaré, mais la phrase voisine « où elle se trouve » ne l'est plus), `:91` (`cette migration *est* la fenêtre` : vraie
  **à la date où la ligne a été écrite**, fausse depuis `20261025000001` — **seule ligne pour laquelle cette date vaut** ; le commentaire de `post_restore.rs:454-455` l'a déjà reconnu :
  « Elle ne l'est plus depuis 20261025000001 »).

### #767 — le fichier de log n'est pas `kesh.log`

Le code : `crates/kesh-api/src/logging.rs:64` (`LogRotation::Daily => Rotation::DAILY,`) puis `:79` (`let mut builder = RollingFileAppender::builder()` avec `.filename_prefix(prefix)`),
`decompose_path` `:37` (`/var/log/kesh/kesh.log` → dossier `/var/log/kesh`, préfixe `kesh`, suffixe `log`). `tracing-appender` 0.2.5
(`~/.cargo/registry/src/*/tracing-appender-0.2.5/src/rolling.rs:640-652`) : `(_, Some(filename), Some(suffix)) => format!("{}.{}.{}", filename, date, suffix)`
sauf `Rotation::NEVER` (`kesh.log`) ; format de la date `[year]-[month]-[day]` (quotidien, `:564`) ou `[year]-[month]-[day]-[hour]` (horaire, `:563`).

**Ce qui est juste et ne se touche pas** : la valeur de la variable `KESH_LOG_FILE_PATH`, qui *désigne* dossier + préfixe + suffixe :
`admin-manual.tex:1037` (tableau, défaut `/var/log/kesh/kesh.log`), `:3297` (extrait du compose), `.env.example:319`, `:331`, `:334`,
`docker-compose.prod.yml:118`, `docker-compose.yml:95` ; et le commentaire de `.env.example:330-334` est **contrôlé par un test** (`controle_raisons_valeurs`, `crates/kesh-api/tests/configuration_transmise.rs:128-140` et `:2677-2692` : la raison « vide, contrairement aux autres variables » doit précéder la ligne d'affectation).
`admin-manual.tex:3967` (`rotation \texttt{never}` → `kesh.log`) est juste aussi. **Le fichier d'epic, qui liste `l. 1037` et `.env.example l. 319, 331, 334`, se trompe**
(voir D4).

**Ce qui est faux** (le nom du **fichier écrit**) :

- `admin-manual.tex:1031` : `les logs sont écrits dans \keshpath{/var/log/kesh/kesh.log} à l'intérieur du conteneur` ;
- `admin-manual.tex:1045` : `utilisez \keshcommand{sudo tail -f ./log/kesh.log}` ;
- `.env.example:350` : `# ./log/kesh.log`) ou ajustez les permissions du répertoire.` ;
- `docker-compose.yml:91` : `# Écrit AUSSI les logs dans un fichier /var/log/kesh/kesh.log (bind-monté` (commentaire YAML ; hors liste du fichier d'epic, voir D4).

Le triage du 2026-11-19 (commentaire de l'issue) dit « kesh.log figure encore à `:1008` et dans `.env.example:300` » : ces numéros ont
bougé (`:1031`/`:1045`, `:350`), la substance reste.

### #814 — quatre imprécisions du manuel MCP

Les numéros de l'issue (`:3231`, `:3350`, `:3341`, `:3219`) ont bougé ; les phrases se retrouvent par leur texte :

1. `admin-manual.tex:3803` : `Tant que l'envoi attend, la relecture ne rend que « en attente »` ; `:3922` : `S'il relit avant votre décision, Kesh rend « en attente » \emph{(non éprouvé au parcours : décrit d'après le code)}`.
   Texte réel : `crates/kesh-api/src/mcp/textes.rs:210` (`"L'envoi n° {n} attend encore la validation de l'utilisateur dans Kesh. Rien n'est parti."`), cité par le manuel lui-même
   à `:2999` (`« L'envoi n° N attend encore\ldots » à la relecture en attente`). L'accusé du dépôt, lui, commence par « Envoi n° {n} en attente de validation : … »
   (`textes.rs:198-203`), de sorte que `:3803` (`un numéro d'envoi, « en attente », et la consigne`) en est un fragment exact : à garder.
2. `admin-manual.tex:3913` : `Ce que le serveur répond est donc éprouvé` — le parcours a été joué « vers la 0.16.0 » (même ligne) ; la forme de la 0.17.0 (aucun contenu
   structuré, `mcp_contrat.rs:602` et `:1084` ; montants sous une seule forme, test `sortie.rs:678`) est établie par le test de contrat et les tests des outils, non par le parcours.
   Voisine à relire : `:3918` (`Ce que le serveur répond, lui, est éprouvé : à \texttt{initialize}, …`), qui parle de la 0.16.0 aussi.
3. `admin-manual.tex:3791` : `Sous une clé dispensée, la réponse d'un outil commence par un en-tête fixe` — vrai des outils de lecture seulement :
   `TEXTE_ENTETE_DONNEES` n'est posé que par `sortie.rs:500` (`let texte = format!("{TEXTE_ENTETE_DONNEES}\n\n{joli}");`), alors que
   `kesh_lire_envoi` et `proposer_prompt` ont leurs textes propres (`textes.rs:208-241`).
4. `admin-manual.tex:3805-3807` (§ `sec:mcp-envois`, paragraphe *L'écran*) : **Overfull de 49,7 pt** à la construction du manuel dans une copie
   (`Overfull \hbox (49.6949pt too wide) in paragraph at lines 3805--3807`, `admin-manual.log:3512`, page 163 du PDF reconstruit — page 129 à l'époque de l'issue) ;
   la ligne qui déborde porte `\keshpath{/admin/mcp/sends}` insécable, puis `l'\textbf{Administrateur}.`. D'autres Overfull existent dans le manuel
   (188 au journal) : **hors périmètre** (voir « Issues à ouvrir »).

Garde-fous qui lisent ce manuel : `crates/kesh-api/tests/mcp_matrice_manuel.rs` (bloc `% matrice-mcp:debut/fin`) et `textes_coherents.rs`
`le_serveur_mcp_dit_vrai` (blocs `mcp-reseaux-defaut`, `mcp-portees`, § `sec:mcp-filtre`) : aucune des quatre phrases n'y est dans un bloc marqué.

### #1061 — « triés par numéro » : l'ordre est celui de la chaîne

- `docs/api-external.md:1415` (`ACCOUNT_NOT_POSTABLE`) : `\`details.rejected[{accountId, accountNumber}]\` nomme chaque compte refusé, triés par numéro.` ;
  `:1416` (`DESIGNATED_ACCOUNT_WRONG_TYPE`) : `les types en valeurs de l'API (\`Asset\`, \`Liability\`, \`Revenue\`, \`Expense\`), triés par numéro ;`.
- Le code : `crates/kesh-db/src/errors.rs:764-773` (`/// … qui trie par numéro dans l'**ordre lexicographique de la chaîne** (\`"1000" < "10000" < "1010" < "2000"\`…`),
  `:788-794` (`list.sort_by(|a, b| { a.account_number.cmp(&b.account_number).then(a.account_id.cmp(&b.account_id))`), `:898-926`
  (`WrongTypeDesignatedAccounts::new`, tri par numéro-chaîne, puis identifiant, puis réglage). Le doc-comment le dit déjà ; le guide, non.
- `CHANGELOG.md` : `grep -nE 'tri[ée]s? par num'` → une seule ligne (`:148`), qui parle **d'un autre tri** (la liste des comptes de
  l'écran *Lier* d'un compte bancaire). La correction « CHANGELOG » du fichier d'epic n'a donc **pas d'objet** pour #1061.

### #1067 — le compte de produit par défaut n'est contrôlé qu'à l'usage

- `docs/manual/fr/user-manual.tex:537` : `le \textbf{compte de produit par défaut} n'est pas soumis au contrôle d'\textbf{imputabilité} à l'usage --- son type et son archivage restent contrôlés à la validation.`
- Le code : `crates/kesh-db/src/repositories/invoices.rs:2372-2389` : `if lines_before.iter().any(priced_line_without_account) { … sites.push((None, revenue_account_id)); }` puis
  `validate_line_revenue_accounts_in_tx(` ; le compte par défaut n'est **ajouté aux sites contrôlés que si une ligne chiffrée n'a pas de compte propre**
  (une ligne de commentaire, `l.line_type.is_priced()` faux, n'y fait pas entrer le défaut : `:2373-2377`). Au brouillon (`:1367-1383`) le défaut n'est lu que pour exempter l'imputabilité.

### #1074, #1076 — la règle d'ambiguïté du solde d'ouverture (même paragraphe, un seul remplacement)

- `docs/manual/fr/user-manual.tex:2969` (deux phrases, une par issue) : `Ne sont pas ambigus, et passent~: … \texttt{1'234,567} (milliers déjà séparés). Sont refusés comme \textbf{illisibles}~: une espace ou une apostrophe après le séparateur décimal (\texttt{1.\ 234}), …`
- `CHANGELOG.md:132` : `\`1'234.567\` (milliers déjà séparés). Sont désormais refusés comme **illisibles** : une espace ou une apostrophe après le séparateur décimal (\`1. 234\`, \`1.5'0\`)` ; `:133` : `une partie entière nulle ou dont les milliers sont séparés ne laisse aucun doute.`
- Le code : `frontend/src/lib/features/bank-accounts/format.ts:258` (`const EDGES = new RegExp(…)`, « Séparateurs de groupe et apostrophes de tête et de queue (L1) »), `:293`
  (`let rest = raw.replace(EDGES, '');`) : un blanc ou une apostrophe **de queue** est rogné ; la table `opening-amount-cases.json` le fixe
  (`{ "raw": "1.5'", "kind": "clear", "payload": "1.5" }`, l. 135 ; `{ "raw": "1. 234", "kind": "unreadable" }` ; `{ "raw": "1.5'0", "kind": "unreadable" }`). La règle juste :
  refusé **entre le séparateur décimal et le dernier chiffre** (L5, `format.ts:317`, `if (!/^[0-9]*$/.test(typedFraction)) return UNREADABLE;`).
- `frontend/src/lib/shared/utils/amount-ambiguity.ts` (`isAmbiguousThousands`) : ambigu seulement si la partie entière est **non groupée** (`!parts.grouped`) **et** de un à trois chiffres.
  `1'234,567` a **quatre** chiffres avant la virgule : il passe avec ou sans séparateur ; le groupement ne le sauve pas. Les saisies que seul le groupement sauve
  sont celles dont la partie entière a un à trois chiffres **et** un séparateur : `1 2.345` et `12'3.456` (`opening-amount-cases.json`, lignes « clear »).

### #1087 — « Elle » sans antécédent

`docs/manual/fr/user-manual.tex:3744` : `…sinon, le rejet part aussitôt. Elle reste dans le \textbf{détail de son import}…` — la phrase insérée par la v0.17.7 s'est glissée entre
« la transaction » et le pronom ; les « elle » suivants (`elle compte comme non rapprochée`, `elle revient dans la liste`) se rattachent ensuite à « La transaction ».

### #1405 — un champ « Compte comptable par défaut » que la fiche contact n'a pas

`docs/manual/fr/user-manual.tex:1385` : `\item \textbf{Compte comptable par défaut} : par exemple 1100 (Débiteurs) pour un client.` Le formulaire
(`frontend/src/routes/(app)/contacts/+page.svelte`) a pour champs `form-type, form-salutation, form-firstname, form-lastname, form-name, form-ide, form-client-number, form-street,
form-building, form-npa, form-city, form-country, form-email, form-phone, form-language, form-payment-terms(-days)` : **aucun compte** ; `contacts.types.ts` et
`crates/kesh-db/src/entities/contact.rs` non plus (`grep -niE "account|compte"` : aucun champ). L'autre `1100 (Débiteurs)` du manuel (`:611`) est un exemple d'écriture : juste.

## Inventaire par le symptôme (propagation post-patch, tout le dépôt)

Commandes à rejouer **avant** de patcher puis **après** ; hors `_bmad-output/`, `node_modules/`, `target/`, `.git/` ; PDF aplatis pour les manuels.
Chaque ligne est dans le périmètre (AC) ou écrite « sans objet » avec sa raison — l'ensemble des sites qui **ne se résolvent pas** est clos ci-dessous.

| Symptôme | Commande | Sites trouvés à `ee684a69f` | Sort |
|---|---|---|---|
| nom du fichier de log | `grep -rnE 'kesh\.log\|kesh\.\*\.log\|kesh\.<' . --exclude-dir=…` | manuel 1031 (faux), 1037 (valeur juste, **description fausse**), 3297, 3967 (justes) ; `.env.example` 319, 331, 334 (justes), 350 (faux) ; compose 91 (faux), 95, prod 118, dev 40 (justes : valeur de la variable) ; `crates/kesh-api/src/logging.rs` et `tests/configuration_transmise.rs` (valeur ou test du découpage) : sans objet ; **`config.rs:1346`, `.env.example:330`, `admin-manual.tex:1037` : « Chemin complet du fichier de log » (`grep -rnF 'Chemin complet du fichier de log'`), description fausse** | 4 faux + 3 descriptions → AC4 |
| « mutation » / « n'écrit rien » du rôle Consultation et des clés | `grep -nF 'aucune mutation' …` ; `python3 -I` (contexte) sur `n'écrit rien` | `admin-manual.tex:1688`, `:3509` ; `:3621` (MCP, déjà juste) ; `user-manual.tex:279` (« voit les mêmes chiffres, sans les boutons de saisie » : juste) ; `api-external.md:282`, `:1050`, `:1086` (« n'écrit rien » s'entend des livres ; la trace `last_used_at` est celle des clés : **sans objet**, même raison que `:219`) ; `api-external.md:219` (« une clé `read` … ne pourra jamais modifier vos données » : juste si « données » s'entend des livres, ce que les écritures de trace ne contredisent pas : **sans objet**) ; `marketing-brochure.tex:264` (« Consultation (lecture seule) » : libellé de rôle, **sans objet**) | 2 → AC1 |
| `inscrite au registre` / `dans la fenêtre` / `est la fenêtre` | `grep -nF` ×3 sur l'audit et `post_restore.rs` | audit 85 et 89 (+ « Première entrée » en 85), 88, 90, 91 ; 99 et 112 justes ; `post_restore.rs` 461, 484 ; `:81` et `:251` (doc-comment du mécanisme : « le backup est dans la fenêtre » = définition, juste) | AC2, AC3 |
| « en attente » cité comme texte de Kesh | `python3 -I` (contexte) sur `« en attente »` : manuels, guide, CHANGELOG, README, site | `admin-manual.tex:3803` (relecture : faux ; accusé : fragment exact), `:3922` (faux) ; ailleurs : aucun | AC5a |
| `éprouvé` | `grep -noF 'éprouvé'` | `:2145-2168` (procédure de sauvegarde), `:2290-2295` (Synology), `:3742` (proxy) : autres sujets, justes ; `:3913` (faux), `:3918` (à relire) | AC5b |
| « triés par numéro » | `grep -nE 'tri[ée]s? par num' docs README.md website` | `api-external.md:1415`, `:1416` ; `user-manual.tex:4185` (« triés par numéro de compte » : bilan, trié par numéro **en chaîne**, comme le plan — **sans objet** : hors des dix issues, voir angle mort (2)) ; CHANGELOG `:148` (autre liste) | AC6 ; `:4185` sans objet sauf si l'ordre est contredit |
| `restent contrôlés` | `grep -rnE 'restent contrôlés\|son type et son archivage'` | `user-manual.tex:537` seul | AC7 |
| `après le séparateur décimal`, `milliers déjà séparés`, `milliers sont séparés` | `grep -rnE` | manuel 2969 ; CHANGELOG 132, 133 ; code et tests : aucun | AC8, AC9 |
| `Elle reste dans` | `grep -rn` | `user-manual.tex:3744` ; `:622` (autre bloc, antécédent présent) | AC10 |
| `Compte comptable par défaut` | `grep -rnF` | `user-manual.tex:1385` seul (manuels, README, website, `docs/`, frontend) | AC11 |

Angles morts assumés : (1) le **CHANGELOG** est hors story (D3) ; (2) l'ordre « en chaîne » de `user-manual.tex:4185` n'est pas vérifié au code par cette fiche
(hors des dix issues) ; (3) les **PDF** ne sont pas commités : leur contrôle est un aplati d'une construction dans une copie.

## Décisions

- **D1 — Une phrase fausse se réécrit, on n'y ajoute pas de rustine.** Chaque correction remplace la phrase fautive par la juste (au manuel, au guide, aux
  commentaires) ; aucune note « corrigé en … ». *Option écartée* : une `keshnote` d'avertissement à côté de la phrase ; elle laisserait deux textes contradictoires.
- **D2 — #748 reste dans la story, y compris sa partie `.rs`, sous deux conditions** (arbitrage demandé, Q1). La partie `.rs` ne change que le **texte** de deux
  justifications de `EXEMPT_MIGRATIONS` (la donnée `EXEMPT_MIGRATIONS` n'est lue qu'en test et par `crates/kesh-db/examples/perishable_exemptions.rs:23` (`for (version, basis, _) in EXEMPT_MIGRATIONS`, qui ignore la justification), jamais à l'exécution du serveur). Mais c'est un fichier exécutable de `kesh-db` : l'exception du `CLAUDE.md` (« dès qu'un patch touche … `post_restore.rs`,
  gate complet ») s'applique. Contraintes de réécriture, vérifiées par des tests existants : le fondement **reste** `ExemptionBasis::PerishableSince(20260827000001)` (déclarer ces deux entrées
  « Hors fenêtre » ferait rougir `every_exemption_declares_a_coherent_basis`, qui juge « périssable et Hors fenêtre » exclusifs) ; la justification **ne commence pas** par `Hors fenêtre` ; le couple `(version, borne)` que fige `the_release_script_sees_every_perishable_exemption` (`post_restore.rs:1272`) ne change pas (`prepare-release.sh` ne lit aucune justification : `:135` « CE BLOC NE GREPE RIEN »). Ce que la réécriture dit : le fondement déclaré est celui du **parc** ;
  la migration est devenue **hors fenêtre** depuis `20260917000001` (Story 25-2-c), comme `20260828000001` et `20260910000001`, de sorte que l'argument du parc n'est plus qu'une seconde raison. *Option écartée A* : reclasser les deux entrées en
  `Durable` « Hors fenêtre depuis 20260917000001 » — c'est la vraie conclusion, mais elle **change une donnée de garde-fou** (le contrôle de parc de `prepare-release.sh`, par `perishable_exemptions.rs`, cesserait de les lire) :
  décision de Guy/orchestrateur et story à part, à ouvrir (« Issues à ouvrir », I1). *Option écartée C* : écrire « Hors fenêtre depuis `20260917000001` » **en tête** des justifications, comme l'exige le doc-comment `post_restore.rs:384-391` : `every_exemption_declares_a_coherent_basis` (`post_restore.rs:1194-1210`, `assert!(!claims_window, …)`) juge « périssable et Hors fenêtre » exclusifs et rougirait ; la règle `:384-391` vise les exemptions `Durable`. Tension assumée : la sortie de fenêtre s'écrit dans les justifications **sans le préfixe** (donc non contrôlée par `exemptions_claiming_out_of_window_really_are_out_of_window`) et dans l'audit ; si l'orchestrateur préfère retirer toute affirmation de fenêtre des justifications (ne garder que « faux » → « voir l'audit »), c'est admis. *Option écartée B* : sortir #748 vers la 58-2 (qui a déjà le gate Rust) : la 58-2 est à dix issues, la borne
  de l'epic est dix. La question de l'E2E est posée en **Q1** (§ *Questions*).
- **D3 — Le CHANGELOG n'est pas écrit par la story.** Règle du `CLAUDE.md` (« `CHANGELOG.md` : pas écrit dans les stories ; assemblé à l'intégration ») et déclaration du kickoff
  (« une ligne au plus, à l'intégration »). Les deux phrases fausses du CHANGELOG (`:132`, `:133`, section 0.17.7) se corrigent **à l'intégration par l'orchestrateur**, sur le texte
  donné dans la *Note de version* ci-dessous. Effet voulu : aucun conflit avec la release v0.20.0, qui insère 33 lignes en tête du CHANGELOG. Le choix de réécrire une section **publiée** (0.17.7) plutôt que d'ajouter une ligne d'erratum à l'entrée de la nouvelle release est soumis au Project Lead (Q2). *Option écartée* : éditer le CHANGELOG dans la story (règle du `CLAUDE.md`, et conflit avec la release).
- **D4 — #767 : périmètre resserré aux sites qui désignent le fichier, pas la variable.** Le fichier d'epic liste `admin-manual.tex` 1037 et `.env.example` 319, 331, 334 : ce sont des
  **valeurs de `KESH_LOG_FILE_PATH`**, justes (et le commentaire voisin contrôlé par `configuration_transmise`). Les « corriger » en `kesh.<date>.log` ferait **écrire une variable fausse** (`KESH_LOG_FILE_PATH=/var/log/kesh/kesh.<date>.log`
  désignerait un autre fichier). Le commentaire de `docker-compose.yml:91`, qui dit « un fichier /var/log/kesh/kesh.log », est ajouté (même symptôme, commentaire seul, hors liste du kickoff : à déclarer ;
  s'il est refusé, il sort et l'AC4 le dit). Les trois **descriptions** « Chemin complet du fichier de log » (manuel `:1037`, `.env.example:330`, doc-comment `config.rs:1346`) entrent, la valeur restant inchangée. *Option écartée* : tout remplacer par `kesh.*.log` (casse la valeur de la variable et le test).
- **D5 — #1067 : « dès qu'une ligne chiffrée le suit »**, non « dès qu'une ligne le suit » comme dit l'issue : depuis 49-1a, une ligne de commentaire n'entraîne pas le contrôle (`invoices.rs:2372-2377`,
  `priced_line_without_account`, `:762`). C'est plus exact que le texte de l'issue ; la validation le tranche. *Option écartée* : reprendre mot pour mot « dès qu'une ligne le suit » (fausse pour une facture qui ne porte que des lignes de commentaire et des lignes à compte propre).
- **D6 — #1076 : `1'234,567` reste cité, comme saisie claire à quatre chiffres** (« quatre chiffres avant la virgule », comme `1000.000`), sans « milliers déjà séparés » ; **aucune saisie mal groupée n'est citée** : `12'3.456` et `1 2.345` sont acceptées par l'écran du solde d'ouverture (`opening-amount-cases.json:81-82`), mais le journal les refuse (`balance.ts:90`, `JOURNAL_AMOUNT_RE` : groupes de trois chiffres) et le manuel le dit (`user-manual.tex:602`, « par groupes de trois chiffres ; une espace entre les milliers n'est pas admise ») ; une partie entière bien groupée a toujours quatre chiffres ou plus, le groupement ne sauve donc jamais une saisie bien formée.
  Si la règle doit nommer le groupement : « une partie entière groupée n'est jamais ambiguë », sans exemple. *Option écartée A* : citer `12'3.456` (apprend une tolérance que le manuel refuse ailleurs). *Option écartée B* : retirer toute mention de `1'234,567` — le paragraphe garde ses trois exemples (`1000.000`, `0.500`, `1'234,567`), qu'il faut seulement justifier juste.
- **D7 — Pas de PDF commité** (`CLAUDE.md` : « PDF des manuels : jamais commités dans `epic/…` ou `story/…` ») ; les PDF se régénèrent à la release. Le contrôle des PDF se fait sur une **copie**. *Option écartée* : régénérer et commiter les PDF dans la story (conflits binaires avec la release, interdit par la règle).
- **D8 — Reprise de `main` après la release v0.20.0 avant le développement** (voir « Fichiers communs »). *Option écartée* : développer sur la base actuelle et résoudre à l'intégration (le tableau des erreurs de `api-external.md` est touché des deux côtés).

## Critères d'acceptation

Chaque AC se prouve par un `grep -nF` **avant** (la chaîne fausse est présente : preuve du « rouge ») et **après** (absente, la juste présente), sur le texte **et** sur le PDF reconstruit dans une copie (l'aplati porte l'apostrophe typographique ’ : normaliser avec `| sed "s/’/'/g"` **avant** de chercher une chaîne à apostrophe, ou chercher une chaîne sans apostrophe ; le compte « avant » au PDF est non nul pour chaque AC, relevé dans `avant.txt` — **exception** : la description de `admin-manual.tex:1037` s'aplatit en entrelaçant les trois colonnes du `tabularx` (`grep -oF 'Chemin complet du fichier de log'` → 0 au PDF) ; elle se prouve **au `.tex`** (`grep -cF 'Chemin complet du fichier de log' docs/manual/fr/admin-manual.tex` : 1 → 0) et, au PDF, par la phrase de `:1031`)
(`cp -r docs/manual <scratchpad>/man && make -C <scratchpad>/man admin user`, puis `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`). Les lignes ci-dessous sont celles de `ee684a69f` ; après la reprise de `main`, le
développeur les **retrouve par la chaîne**, non par le numéro.

- **AC1 (#730)** — `grep -cF 'aucune mutation' docs/manual/fr/admin-manual.tex` : 1 → 0 ; `grep -cF "une lecture n'écrit rien, à une exception près" docs/manual/fr/admin-manual.tex` : 1 → 0. Le tableau des rôles (`:1688`)
  dit « aucune modification des livres » et nomme ce qui s'écrit malgré tout : les consultations et exports de rapports (`report.generated`, `report.exported`) au journal d'audit, l'export global (`exports.global`),
  le premier téléchargement du PDF d'une facture (qui le fige) ; le point des clés `read` (`:3509`) nomme les **trois** écritures (journal des rapports, journal de l'export global, `last_used_at` de la clé — mise à jour à chaque appel authentifié, **requête refusée comprise** : `touch_last_used`, `auth/api_key.rs:215`, précède le refus `403 API_KEY_READ_ONLY` de `middleware/auth.rs:145`) **et** la
  quatrième (le PDF), sans « n'écrit rien ». Chaque écriture citée a sa ligne de code au § *Ce qui se lit au code*. Le texte MCP (`:3621`) reste intact.
- **AC2 (#748, audit)** — `grep -cF '**inscrite au registre `POST_RESTORE_BACKFILLS` en CLASSE B** (garde-fou P7)' docs/migrations-idempotence-audit.md` : 3 (`:85`, `:89`, `:112`) → 1 (`:112`, entrée vivante) ; `:99` (classe A, retirée, la ligne le dit) n'est pas comptée. Les lignes 85 et 89 disent, **sans cette chaîne** : inscrite au registre en classe B **à sa création**, **retirée** ensuite (Story 25-2-c)
  et **exemptée** « Hors fenêtre depuis 20260917000001 » ; « Première entrée du registre depuis que la Story 24-2 l'avait vidé » (`:85`) devient un fait daté (« à sa création »). Les lignes 88, 90 et 91 ne disent plus
  « où elle se trouve » / « cette migration *est* la fenêtre » au présent : elles disent l'état d'aujourd'hui : pour 88 et 90, **hors fenêtre depuis `20260917000001`** ; pour 91, `20261025000001` (`mcp_keys`) a refermé la fenêtre sur la migration elle-même ; le fondement déclaré (parc, couverture) est gardé. **Aucune ligne de migration ajoutée, aucune retirée** :
  `ls crates/kesh-db/migrations/*.sql | wc -l` = `grep -c '^| \`20' docs/migrations-idempotence-audit.md` = 98 avant et après ; en-tête `## Table d'audit (98 migrations)` inchangé (P5).
- **AC3 (#748, `post_restore.rs`)** — `grep -cF 'EST dans' crates/kesh-db/src/post_restore.rs` : 2 (lignes 461 et 484) → 0.
  Les deux justifications ne contiennent plus « dans la fenêtre » au présent, ne commencent pas par `Hors fenêtre`, **gardent** `ExemptionBasis::PerishableSince(20260827000001)`, la chaîne `CE FONDEMENT SE PÉRIME` et leurs lignes
  de borne (diff de la région : seules les phrases sur la fenêtre changent) ; elles disent « hors fenêtre depuis `20260917000001` » sans commencer par `Hors fenêtre`. Tests verts, ciblés puis complets : `every_exemption_declares_a_coherent_basis`, `exemptions_claiming_out_of_window_really_are_out_of_window`,
  `every_data_backfill_migration_is_triaged`, `registry_entries_are_within_import_window` (lib `kesh-db`), `the_release_script_sees_every_perishable_exemption` (`post_restore.rs:1272`). Aucun test de `kesh-api` ne lit `EXEMPT_MIGRATIONS` : le doc-comment `admin_full_import_e2e.rs:2694-2697` (« aucune version publiée ne s'y trouve ») énonce le **parc**, fondement que D2 garde : sans objet. `git diff --stat` de `crates/` : **deux fichiers**, `kesh-db/src/post_restore.rs` et `kesh-api/src/config.rs` (AC4).
- **AC4 (#767)** — `admin-manual.tex:1031` dit que les fichiers du dossier `/var/log/kesh/` (monté sur `./log/`) s'appellent `kesh.<AAAA-MM-JJ>.log` avec la rotation quotidienne (le défaut), la date étant **UTC** (`rolling.rs:198-222`) et la rotation horaire donnant `kesh.<AAAA-MM-JJ-HH>.log`, en renvoyant à la valeur de
  `KESH_LOG_FILE_PATH` pour le préfixe et le suffixe ; **la description de la variable** (`admin-manual.tex:1037`, `.env.example:330`, doc-comment `crates/kesh-api/src/config.rs:1346`, tous trois « Chemin complet du fichier de log ») dit « dossier, préfixe et suffixe du fichier de log (avec la rotation, le nom reçoit la date) », **valeur inchangée** ;
  `:1045` et `.env.example:350` donnent une commande **exécutable** qui désigne un fichier qui existe (`sudo tail -f ./log/kesh.$(date -u +%F).log` (au `.tex` : `\keshcommand{sudo tail -f ./log/kesh.\$(date -u +\%F).log}`, le `$` et le `%` s'échappent, comme `admin-manual.tex:2092` ; le `%` non échappé commenterait la fin de ligne), ou `kesh.*.log` avec la mise en garde de `:3967` sur son développement par le shell ; jamais `kesh.<AAAA-MM-JJ>.log` tel quel, que le shell lirait comme une redirection ; une demi-phrase dit que la rotation est paresseuse (`rolling.rs:227-230`) : le fichier d'un nouveau jour UTC n'existe qu'à la première écriture, et un `tail -f` en cours ne suit pas le changement de fichier — relancer la commande après minuit UTC) ; `docker-compose.yml:91` (commentaire) dit de même. `grep -nF './log/kesh.log' docs/manual/fr/admin-manual.tex .env.example` : 2 → 0. **Valeur inchangée** (la ligne 1037 et `.env.example:334` gardent `/var/log/kesh/kesh.log` ; `.env.example:330-334` continue de dire « contrairement aux autres variables » et jamais « ou absent » : `controle_raisons_valeurs`, `configuration_transmise.rs:983-1008`, lit ce commentaire et rougirait sinon, d'où `binary(configuration_transmise)` nommé ci-dessous) ; **inchangés octet pour octet** : les
  lignes 3297, 3967 du manuel, `.env.example` 319, 331, 334, `docker-compose.yml:95`, `docker-compose.prod.yml:118`, `docker-compose.dev.yml:40` ; `cargo nextest run -E 'binary(configuration_transmise)'` vert.
- **AC5a (#814-1)** — `grep -cF "la relecture ne rend que « en attente »" docs/manual/fr/admin-manual.tex` : 1 → 0 ; `grep -cF "Kesh rend « en attente »" …` : 1 → 0. Les deux sites citent le **texte réel** (« L'envoi n° N attend encore la validation de
  l'utilisateur dans Kesh. Rien n'est parti. », `textes.rs:210`) ou le décrivent sans guillemets. `:3803` (accusé, « un numéro d'envoi, « en attente », … ») est relu et gardé ou aligné sur « Envoi n° N en attente de validation » (`textes.rs:200`).
- **AC5b (#814-2)** — `grep -cF 'Ce que le serveur répond est donc éprouvé' docs/manual/fr/admin-manual.tex` : 1 → 0. La phrase dit ce que le parcours a éprouvé (la 0.16.0) et que la forme de la 0.17.0 l'est par le test de contrat
  (`mcp_contrat.rs`, aucun `structuredContent`), non par le parcours ; `:3918` (`Ce que le serveur répond, lui, est éprouvé`) est relue et alignée si elle dit la même chose.
- **AC5c (#814-3)** — `grep -cF "la réponse d'un outil commence par un en-tête fixe" docs/manual/fr/admin-manual.tex` : 1 → 0 ; la phrase dit « la réponse **réussie** d'un outil de lecture » (et que `kesh_lire_envoi`, `proposer_prompt` et les textes d'erreur, `textes.rs:247` `texte_erreur_outil`, ont leurs textes propres, sans l'en-tête).
  Les blocs `% matrice-mcp:debut/fin`, `mcp-reseaux-defaut`, `mcp-portees` sont intacts (`binary(mcp_matrice_manuel)` et `textes_coherents` vert).
- **AC5d (#814-4)** — à la construction dans une copie, `grep -c 'in paragraph at lines 3805--3807' admin-manual.log` (numéros à retrouver après la reprise de `main`, par le texte `L'écran.` du § `sec:mcp-envois`) : 1 → 0 : **aucun Overfull** pour ce paragraphe (le journal en compte 188 au total ; les autres sont hors périmètre, I3). Correction par un point de coupure (`\allowbreak`
  dans le chemin, comme ailleurs dans le manuel), jamais par `\sloppy` global.
- **AC6 (#1061)** — Les deux sites sont `…nomme chaque compte refusé, triés par numéro.` (`ACCOUNT_NOT_POSTABLE`) et `…, triés par numéro ;` (`DESIGNATED_ACCOUNT_WRONG_TYPE`, après la liste des types) ; `grep -cF 'triés par numéro' docs/api-external.md` : 2 → 0. Les deux lignes disent « par numéro, en ordre de chaîne »
  (`"10000"` avant `"2000"`), puis par identifiant (et par réglage pour `DESIGNATED_ACCOUNT_WRONG_TYPE`), comme `errors.rs:764-773` et `:898-905`. `CHANGELOG.md` : sans objet (D3, § *Ce qui se lit au code*).
- **AC7 (#1067)** — `grep -cF 'restent contrôlés à la validation.' docs/manual/fr/user-manual.tex` : 1 → 0 ; la phrase dit que type et archivage ne sont contrôlés à la validation que **dès qu'une ligne chiffrée sans compte propre le suit** (D5), avec
  `invoices.rs:2378-2389` comme preuve citée au Dev Agent Record.
- **AC8 (#1074)** — `grep -cF 'une apostrophe après le séparateur décimal' docs/manual/fr/user-manual.tex` : 1 → 0 ; la phrase dit **entre le séparateur décimal et le dernier chiffre** (`1. 234`, `1.5'0` refusés ; `1.5'` et `1.5 ` acceptés, rognés par L1).
  Texte de remplacement du CHANGELOG `:132` donné en *Note de version* (erratum d'intégration, D3).
- **AC9 (#1076)** — `grep -cF 'milliers déjà séparés' docs/manual/fr/user-manual.tex` : 1 → 0. `\texttt{1'234,567}` n'est plus expliqué par un séparateur ; il est donné « (quatre chiffres avant la virgule) », comme `\texttt{1000.000}`, **sans citer aucune saisie mal groupée** (D6). Erratum CHANGELOG `:132` et `:133` donné en *Note de version*.
- **AC10 (#1087)** — `grep -cF "le rejet part aussitôt. Elle reste dans" docs/manual/fr/user-manual.tex` : 1 → 0 ; `grep -cF 'La transaction reste dans' …` : 0 → 1.
- **AC11 (#1405)** — `grep -cF 'Compte comptable par défaut' docs/manual/fr/user-manual.tex` : 1 → 0 ; la liste des champs de la fiche contact (`\begin{itemize}` de `:1370-1387`) ne porte plus la ligne ; `1100 (Débiteurs)` de `:611` reste.
  Les champs réellement présents au formulaire mais absents de la liste (civilité, langue, case client/fournisseur) sont **hors périmètre** (« Issues à ouvrir », I2).
- **AC12 (transverse)** — (a) `git diff --name-only epic/58-nettoyage...HEAD` = exactement : `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/user-manual.tex`, `docs/api-external.md`, `.env.example`, `docker-compose.yml`,
  `docs/migrations-idempotence-audit.md`, `crates/kesh-db/src/post_restore.rs`, `crates/kesh-api/src/config.rs`, `_bmad-output/implementation-artifacts/sprint-status.yaml`, ce fichier (+ prompts de revue) ; **ni `CHANGELOG.md`, ni `README.md`, ni `website/`, ni PDF**.
  (b) Après les corrections, le grep du symptôme de chaque ligne de l'inventaire est rejoué sur tout le dépôt et sur les PDF reconstruits ; sortie citée au Dev Agent Record. (c) `binary(textes_coherents)` (G2, G4-bis/ter, G23…) et
  `binary(mcp_matrice_manuel)` verts ; `npm run lint-i18n-ownership` : sans objet (aucun catalogue touché).

## Preuve du rouge

Aucun test neuf : la story ne change aucun comportement. La preuve du rouge de chaque AC est le `grep` **avant** (chaîne fausse présente, comptes ci-dessus, relevés à `ee684a69f` au texte et au PDF) puis **après** (absente). Le développeur
enregistre les deux sorties de chaque AC dans `kesh-gate-logs/e583/` (`avant.txt`, `apres.txt`). Une garde de texte neuve (un test `textes_coherents` pour une phrase du manuel) est **écartée** : elle ne vaudrait que pour ces dix phrases, et le
manuel a déjà les gardes de `textes_coherents.rs` (G2 à G25), qui couvrent des thèmes entiers, non des phrases.

## Tâches

0. **Étape 0** : reprendre `main` (D8) ; **recontrôler les corrections des validations P1 et P2** (appliquées après la dernière passe : D2 et option C, AC4, AC9/D6, comptes « avant »), ligne lue citée au Dev Agent Record ; rejouer les `grep` du § *Inventaire* et chaque chaîne des AC ; si une chaîne a disparu (corrigée par la release ou un autre epic), l'AC correspondant se déclare « déjà couvert » avec le commit.
1. AC1 → `admin-manual.tex` (2 sites). 2. AC4 → manuel, `.env.example`, `docker-compose.yml` (commentaire), `crates/kesh-api/src/config.rs` (doc-comment ; commit ciblé : `cargo fmt`, clippy, `binary(configuration_transmise)`). 3. AC5 → manuel MCP, quatre points, construction du manuel pour le point 4. 4. AC6 → guide de l'API. 5. AC7, AC8-9, AC10, AC11 → manuel utilisateur.
6. AC2 → audit d'idempotence. 7. AC3 → `post_restore.rs` (dernier commit de code ; seuls fichiers de code de la story : `config.rs` et `post_restore.rs`). 8. Construction des deux manuels dans une copie, aplati, greps après. 9. Gates. 10. *Note de version* et erratum du CHANGELOG remis à l'orchestrateur.

## Gates

- Texte seul (commits des tâches 1 à 6) : `cargo nextest run -E 'binary(textes_coherents)'`, `-E 'binary(mcp_matrice_manuel)'`, `-E 'binary(configuration_transmise)'`, `-E 'binary(audit_route_registry)'` (il lit `admin-manual.tex`, près des passages sur le journal d'audit que l'AC1 modifie) sous `scripts/mem-guard.sh` ; `make admin user` **dans une copie** du dépôt (jamais dans le worktree) ;
  `pdftotext` aplati.
- Commit de `config.rs` (tâche 2) : `cargo fmt --all -- --check`, clippy, `binary(configuration_transmise)`. Commit de `post_restore.rs` (tâche 7, dernier commit de code ; il porte le gate complet pour les deux) : exception `kesh-db` du `CLAUDE.md` — **gate complet** : `scripts/test-fast.sh` (fmt, clippy, nextest) puis `scripts/gate-ci.sh <base>`, base remise à zéro avant. **E2E complet** : la story ne change aucun chemin d'exécution (les chaînes ne sont lues
  qu'en test et par `perishable_exemptions.rs`) ; on propose de **ne pas le rejouer** (Q1) ; sans réponse, il est rejoué une fois à la clôture, selon le `CLAUDE.md`.
- Recette visuelle : sans objet (aucun contrôle visible). Captures : aucune.
- Push de la seule branche `story/58-3-…` à chaque étape franchie (règle des agents du `CLAUDE.md`) ; la fiche elle-même est poussée avant sa validation.

## Note de version

*(Pour le CHANGELOG de la release du vendredi 2026-12-04, rubrique « Corrigé », assemblée à l'intégration ; la story n'écrit pas le CHANGELOG.)*

**Corrigé** — La documentation dit ce que Kesh fait : le manuel d'administration ne présente plus le rôle Consultation ni les clés en lecture comme « sans écriture » (les consultations de rapports, l'export global et la dernière utilisation d'une clé s'inscrivent
au journal), donne le vrai nom du fichier de log (`kesh.<date>.log`), et corrige quatre imprécisions du chapitre sur le serveur MCP ; le guide de l'API dit que les comptes d'un refus sont triés par numéro **en ordre de chaîne** ; le manuel utilisateur ne décrit plus un champ
« Compte comptable par défaut » que la fiche contact n'a pas, précise quand le compte de produit par défaut est contrôlé, et dit juste la règle du solde d'ouverture (une espace ou une apostrophe est refusée *entre* le séparateur décimal et le dernier chiffre). (refs #730, refs #748,
refs #767, refs #814, refs #1061, refs #1067, refs #1074, refs #1076, refs #1087, refs #1405)

**Errata du CHANGELOG (section 0.17.7, à appliquer par l'orchestrateur à l'intégration, deux phrases)** :

- `:132` : « `1'234.567` (milliers déjà séparés) » → « `1'234.567` (quatre chiffres avant le point : le groupement n'y change rien) » ; « une espace ou une apostrophe après le séparateur décimal (`1. 234`, `1.5'0`) » →
  « une espace ou une apostrophe **entre le séparateur décimal et le dernier chiffre** (`1. 234`, `1.5'0` ; `1.5'` en fin de saisie est rogné et accepté) ».
- `:133` : « une partie entière nulle ou dont les milliers sont séparés ne laisse aucun doute » → « une partie entière nulle, ou de quatre chiffres ou plus, ne laisse aucun doute ».

## Questions (pour l'orchestrateur et le Project Lead)

- **Q1** — Le commit de `post_restore.rs` (et de `config.rs`, doc-comment) impose le gate Rust complet (exception `kesh-db`). **Faut-il rejouer l'E2E complet ?** Proposition : non, la story ne change aucun chemin d'exécution ; à défaut de réponse, il est rejoué une fois à la clôture.
- **Q2** — Les deux phrases fausses du CHANGELOG (section 0.17.7, `:132-133`) : réécrites à l'intégration (D3), ou corrigées par une ligne d'erratum dans l'entrée de la release ? Le kickoff prévoit « une ligne au plus ».

## Dev Notes

### Modules touchés (règle de splitting) — **deux**

Un **module** est un crate ou un domaine du frontend dont le **code de production** change : ici **`kesh-db`** (`post_restore.rs`, chaînes de justification) et **`kesh-api`** (`config.rs:1346`, un doc-comment) — **deux crates**, texte seulement, gate complet de l'exception `kesh-db` ; les manuels, le guide, `.env.example`, l'audit et un commentaire de compose ne sont pas du code de production.
Splitting préventif : sans objet (deux modules, dix issues de texte).

### Fichiers communs avec la release v0.20.0 (branche `docs/release-0200`, `980d17ffe`) — relevé

`git diff -U0 ee684a69f...origin/docs/release-0200` sur les fichiers de la story :

| Fichier | Hunks de la release | Sites de la 58-3 | Recoupement |
|---|---|---|---|
| `docs/manual/fr/admin-manual.tex` | 7 hunks : `:703`, `:1719`, `:2754`, `:2760`, `:2763`, `:4075`, `:4229` (aucune ligne nette ajoutée) | `:1031`, `:1045`, `:1688`, `:3509`, `:3791`, `:3803`, `:3805-3807`, `:3913`, `:3918`, `:3922` | **aucun hunk à moins de 30 lignes** (le plus proche : `:1719` contre `:1688`) ; conflit de texte improbable ; numéros stables (aucune ligne nette ajoutée) |
| `docs/manual/fr/user-manual.tex` | 1 hunk : `:944` | `:537`, `:1385`, `:2969`, `:3744` | aucun |
| `docs/api-external.md` | 33 hunks, presque tous d'une ligne (102 lignes touchées au total) dont `:1412`, `:1422`, `:1425-1441` (**tableau des erreurs**), `:1457`, `:1473`, `:1489` | `:1415`, `:1416` | **voisins immédiats** : `:1412` est à 3 lignes de `:1415`, `:1425` à 9 de `:1416` ; **la 58-3 reprend `main` après la release avant le développement**, sans quoi un conflit dans le tableau des erreurs est probable |
| `CHANGELOG.md` | +33 lignes en tête (section 0.20.0, `:11`) | **aucun** (D3 : erratum d'intégration) | les numéros `:132`, `:133`, `:148` deviennent `:165`, `:166`, `:181` après la release ; l'erratum cite les phrases, non les lignes |
| autres branches ouvertes (relevé P2) | `epic/53-ecritures-justes` : `user-manual.tex` +22 lignes à `:606` (voisin de la règle citée par D6, `:602`), `api-external.md` `:234`, `:318`, `:1467` ; `fix/manuel-guillemets` (fusionnée) : `user-manual.tex:3569` ; `story/1582-nom-comptabilite` : `user-manual.tex:150` ; la 58-1 : `.env.example`/manuel non touchés à ce jour | aucun hunk sur un site de la 58-3 | décalage de +22 lignes de `user-manual.tex` après l'Epic 53 : **retrouver chaque site par sa chaîne**, recontrôler la citation de `:602` (D6) après la reprise de `main` |
| `README.md` | 3 hunks (fonctions, feuille de route) | **aucun** | aucun ; la feuille de route n'a pas à changer pour cette story (aucun statut d'epic ne change à ce commit) |

Conséquence : **la story reprend `main` après la release v0.20.0** (D8) avant sa tâche 1 ; après la reprise, les numéros de ligne de la fiche sont à **retrouver par la chaîne copiée** (chaque AC en donne une). Epic 54 (décompte de TVA) touche le même `user-manual.tex` (fichier d'epic) :
le second arrivé résout.

### Prémisses à confirmer à l'étape 0

1. Les six chaînes « avant » des AC sont encore présentes après la reprise de `main`. 2. L'export global est bien ouvert à une clé `read` (route dans `authenticated_routes`, méthode `GET`, `exports.rs:37`) — sinon AC1 ne le nomme pas pour les clés. 3. `20260917000001` crée une table de `TABLES_TO_TRUNCATE` et est postérieure à `20260909000001` et `20260915000001` (`backup.rs:82`) — sinon AC2/AC3 changent de date. 4. Les deux manuels se construisent sans erreur dans une copie.

### Ce qui doit rester vrai

Les valeurs de `KESH_LOG_FILE_PATH` (manuel, `.env.example`, compose) ; le contrôle de `.env.example:330-334` par `configuration_transmise` ; le fondement `PerishableSince(20260827000001)` des deux exemptions et le couple `(version, borne)` figé par `the_release_script_sees_every_perishable_exemption` ; l'ordre et les colonnes de la table d'audit (98 lignes) ;
les blocs marqués du manuel MCP ; le texte MCP `:3621`.

### Issues à ouvrir (par l'orchestrateur ; aucune `gh` qui écrit ici ; une issue par problème, `gh issue list --state all --search` avant)

- **I1** — #748 : 20260909000001 et 20260915000001 sont **hors fenêtre** depuis 20260917000001 (comme 20260828000001 et 20260910000001) mais déclarées `PerishableSince` : les reclasser `Durable` « Hors fenêtre depuis … » retirerait au contrôle de parc de `prepare-release.sh` deux fondements devenus inutiles ; à décider (P3, `technical-debt`).
- **I2** — `user-manual.tex:1370-1387` : la liste des champs de la fiche contact omet la civilité, la langue et la case client/fournisseur (relevé en passant sur `+page.svelte` : `form-salutation`, `form-language`, cases `is_client`/`is_supplier`) (P4, `documentation`).
- **I3** — `admin-manual.tex` : 188 `Overfull \hbox` à la construction (dont plusieurs de plus de 80 pt, p. ex. `:3848`, `:3874`) ; #814 ne couvre que celui de `:3805` (P4, `documentation`).

### Charge estimée

Une demi-journée de développement (dix remplacements de phrases, un aplati, une construction de manuel, un gate Rust complet), plus la validation. Coût de chaque issue : S.

### Références

`_bmad-output/planning-artifacts/epic-58-nettoyage.md` (section 58-3, « Fichiers et modules touchés », « Recoupements ») ; `CLAUDE.md` (§ *Documentation à synchroniser*, *Propagation post-patch*, *Priorités des défauts*, *Review Iteration Rule*) ;
`docs/migrations-idempotence-audit.md` ; `crates/kesh-db/src/post_restore.rs` ; `crates/kesh-api/tests/{textes_coherents,mcp_matrice_manuel,configuration_transmise}.rs`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-11-26 — création** (Sonnet 5.5, `bmad-create-story`, autonomie) : fiche `drafted` ; dix issues relues au texte, au PDF et au code, **aucune ne sort** ; trois écarts au fichier d'epic déclarés (`already-booked.ts` hors périmètre, manuels de/en/it inexistants,
  sites de #767 resserrés) ; #1061 sans objet pour le CHANGELOG ; CHANGELOG hors story (erratum d'intégration, D3) ; un module (`kesh-db`, chaînes) ; sites communs avec la release v0.20.0 relevés (`api-external.md:1415-1416` voisin du tableau des erreurs : reprise de `main` après la
  release) ; prompt de validation P1 (Opus) versionné à côté.
- **2026-11-26 — validation P1 remédiée** (P1 Opus : 0 CRITICAL / 1 HIGH / 3 MEDIUM / 12 LOW ; remédiation Sonnet 5.5, chaque correction vérifiée au code par `grep -nF`). V1-1 : la sortie de fenêtre de 20260909000001 et 20260915000001 date de **20260917000001**
  (`post_restore.rs:429`, `:438` ; `backup.rs:82`), `20261025000001` ne vaut que pour la ligne 91 de l'audit (D2, AC2, AC3, I1, prémisse 3). V1-2 : la description « Chemin complet du fichier de log » (`admin-manual.tex:1037`, `.env.example:330`, `config.rs:1346`) entre dans l'AC4,
  valeur inchangée ; `config.rs` s'ajoute (deux crates, texte seulement, gate complet), `controle_raisons_valeurs` suit (« contrairement aux autres variables », pas « ou absent »), `binary(configuration_transmise)` nommé. V1-3 : D6 et AC9 ne citent plus de saisie mal groupée (`balance.ts:90`, `user-manual.tex:602`), motif de
  l'option écartée corrigé. V1-4 : l'aplati normalise ’ avant la recherche. V1-5 : commande exécutable (`$(date -u +%F)`), date UTC, rotation horaire. V1-6 : « réponse réussie ». V1-7 : `prepare-release.sh` ne lit aucune justification (`perishable_exemptions.rs:23`), aucun test de `kesh-api` ne lit `EXEMPT_MIGRATIONS`
  (`admin_full_import_e2e.rs:2694-2697` sans objet), `the_release_script_sees_every_perishable_exemption` nommé. V1-8 : AC2 compte la chaîne fautive (3 → 1). V1-9 : `api-external.md:219` et la brochure sans objet, avec raison. V1-10 : AC5d « aucun Overfull ». V1-11 : § *Questions* (Q1, Q2), options écartées de D3, D5, D7, D8,
  renvoi « voir D6 » retiré, push. V1-12 : README 3 hunks. V1-13 : `binary(audit_route_registry)` aux gates. V1-14 : date au lieu du numéro de version. V1-15 : Q2. V1-16 : refus 403 nommé à l'AC1. Tendance des findings : P1 0/1/3/12. Prompt de la validation P2 (Sonnet) versionné à côté.
- **2026-11-26 — validation terminée, fiche `ready-for-dev`** (P1 Opus 0/1/3/12, remédiée ; P2 Sonnet 0/0/0/7, sept LOW appliqués, aucun recyclage de sévérité). V2-1 : `config.rs` partout (tableau de couverture, tâches 2 et 7, gates, intitulé « deux » modules). V2-2 : quatrième écart au kickoff déclaré (`config.rs`, `docker-compose.yml`, littéraux de `post_restore.rs`).
  V2-3 : recontrôlé au code — le préfixe `Hors fenêtre` que demande `post_restore.rs:384-391` **ferait rougir** `every_exemption_declares_a_coherent_basis` (`:1194-1210`, `!claims_window` pour un fondement `PerishableSince`) : il n'est **pas** imposé ; la tension est nommée à D2 (option écartée C). V2-4 : ligne « autres branches ouvertes » (Epic 53, `fix/manuel-guillemets`, `story/1582-…`, 58-1), consigne de retrouver chaque site par sa chaîne.
  V2-5 : échappement `\$` et `\%`, rotation paresseuse. V2-6 : `api-external.md:282`, `:1050`, `:1086` sans objet. V2-7 : `:1037` prouvé au `.tex`. Ces LOW, appliqués après la dernière passe, **se recontrôlent à l'étape 0 du développement** (décision de Guy du 2026-10-24), ligne lue citée. Le développement attend la fin de la release v0.20.0 et la reprise de `main` ; Q1 (E2E à la clôture) et Q2 (CHANGELOG 0.17.7) restent au Project Lead.
