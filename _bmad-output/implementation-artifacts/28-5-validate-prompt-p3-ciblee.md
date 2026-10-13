# Prompt — validation P3 ciblée de la spec, Story 28-5 (28-5a serveur / 28-5b écran)

*Versionné le 2026-10-13. Passe ciblée (`CLAUDE.md` § « La passe ciblée ») : **une seule lentille, Haiku**, contexte
frais, en lecture seule, braquée sur la **seule remédiation P2** de la fiche. P1 était Sonnet, P2 Opus ; la
remédiation P2 a été écrite par Opus. Haiku est réservé à ce type de passe (D6, rétrospective de l'Epic 25).*

Dépôt `/home/gcorbaz/devel/kesh-28-5`, branche `story/28-5`. **Objet — un diff unique, aplati** (un seul commit,
`a4a17ef3`, sur `4e133f3e`) :

```sh
git diff 4e133f3e a4a17ef3 -- _bmad-output/
```

Quatre fichiers : la fiche `_bmad-output/implementation-artifacts/28-5-transaction-rejetee-atteignable.md`, le
registre `epic-28-choix-autonomes.md` (notes à C-28-5-4, C-28-5-16, C-28-5-19, C-28-5-23 ; entrées neuves
C-28-5-25 à C-28-5-30), `sprint-status.yaml` (clés `28-5*`) et l'epic `planning-artifacts/epic-28-recette-v0-13-2.md`
(§ *Ordre*). Lis le diff, puis ouvre chaque fichier **autour des hunks** pour relire le texte final (les numéros de
ligne d'un hunk sont ceux du fichier d'après, pas d'avant). Rapports remédiés :
`/home/gcorbaz/devel/kesh-gate-logs/28-5-validate-p2-F.md` et `…-p2-R.md` (MEDIUM F1 = R1, F2 = R2 ; LOW F3-F10,
R3-R12).

⚠️ **Erreur de catégorie à ne pas commettre.** La story n'est **pas développée** : tu relis une SPÉCIFICATION. Une
route, un test (B10, M17…), un doc-comment, une clé i18n ou un texte de manuel que la fiche **prescrit** et qui
n'existe pas encore dans le code n'est **PAS** un défaut — c'est le travail à faire. Un défaut est : une
contradiction entre deux endroits du texte, une prescription inexécutable ou fausse au code **actuel**, un fait cité
faux (numéro de ligne, code d'erreur, nom de fonction), un décompte faux, ou un site oublié. Trois fois déjà, des
« MEDIUM » de passes Haiku ont été des livrables prescrits mais non écrits : vérifie ce point avant chaque finding.

⚠️ **Grep ground-truth obligatoire.** Toute affirmation de présence ou d'absence s'appuie sur une sortie de
`grep -nF "<chaîne>" <fichier>` (ou `sed -n 'A,Bp'`) **copiée dans le rapport**. Pas de finding CRITICAL ou HIGH
« le patch n'a pas été appliqué » sans cette sortie.

## Lentille unique — chasseur de régressions de la remédiation P2

1. **Code d'erreur (F1 = R1).** `OPTIMISTIC_LOCK_CONFLICT` est-il écrit partout où la remédiation parle du `409` de
   la garde `rows_affected() != 1` (AC3, AC9 (a), séquence de la route point 6, références, C-28-5-4, C-28-5-19) ?
   Reste-t-il une graphie `OPTIMISTIC_LOCK` sans suffixe **hors** des entrées historiques (Change Logs passés,
   prompts P1/P2, et la note de C-28-5-19 qui cite l'ancienne graphie) ?
   `grep -nE "OPTIMISTIC_LOCK($|[^_])" <fichier>`. Le producteur est-il bien décrit (mapping
   `DbError::OptimisticLockConflict`, `crates/kesh-api/src/errors.rs:2922-2925` ; `conclude_locked_attempt`, bras
   `ReconciliationError::Db`, `crates/kesh-api/src/routes/reconciliation.rs:4325-4399`) ?
2. **B10 et M15 (F2 = R2).** Le test B10 prescrit est-il exécutable sur le patron qu'il cite
   (`crates/kesh-api/tests/reconciliation_e2e.rs:7225-7330` ; sonde `crates/kesh-db/src/test_fixtures.rs:561`) ?
   Les motifs `["bank_transactions", "bank_account_id = ?"]` figurent-ils **tous deux** dans le texte SQL du
   `SELECT … FOR UPDATE` (`bank_tx_by_id_for_account_sql`, `crates/kesh-db/src/repositories/reconciliation.rs`) et
   dans l'`UPDATE` prescrit (fiche, § *Séquence de la route*, point 4) ? Sont-ils **absents** de la lecture hors
   verrou (`routes/reconciliation.rs:4516-4518`) et du verrou nommé (`crates/kesh-reconciliation/src/mutex.rs:89`,
   `:132`) ? Le raisonnement « code juste → `404` ; M15 → `409 OPTIMISTIC_LOCK_CONFLICT` » tient-il pas à pas ? Le
   rejet par la route, qui précède le `DELETE`, laisse-t-il une transaction ouverte ou une requête qui pourrait
   satisfaire la sonde avant l'appel à `…/unreject` ? La ligne M15 du tableau, l'AC12 (a), l'AC3, T6a, le registre
   (C-28-5-23 marqué remplacé, C-28-5-25) et `sprint-status.yaml` disent-ils la même chose ? Reste-t-il quelque part
   (hors historique) « survivante assumée » pour M15 ou l'affirmation d'un angle mort **écrit au doc-comment** de la
   lecture ?
3. **M17 et B1 étendu (R8).** M17 (`updated_at = NOW(3)` retiré) est-elle tuée par B1 tel qu'écrit (valeur forcée
   dans le passé au montage, puis « postérieure ») ? B1 compte-t-il des tables qui existent (`journal_entries`,
   `invoice_settlements` — vérifie les noms dans `crates/kesh-db/migrations/` ou `test-schema/`) ?
4. **Inventaire D4-ter (F4 = R4).** Rejoue
   `grep -rn "auto_match_rejected_at" crates --include=*.rs --include=*.sql | grep -v /tests/` : 45 lignes ?
   Chacune est-elle au tableau, et chaque ligne rangée « commentaire » en est-elle une (`sed -n`) ? Les numéros
   `:3359`, `:3409`, `csv_tables.rs:740` / `:764`, `bank_imports.rs:488-490`, et les `#[cfg(test)]` l. 178, 127,
   192, 137 sont-ils justes ?
5. **Découpage (F5, R7).** La 28-5a ne touche-t-elle plus **aucun** fichier frontend, partout (tableau d'en-tête,
   § *Décompte des modules*, *Régions touchées*, AC10, T3, T7a, T8, registre C-28-5-26, `sprint-status.yaml`) ? Son
   décompte de sites est-il cinq ? T4 dit-il « gate complet à chaque passe » sans contredire la ligne de
   `sprint-status.yaml` ni le `CLAUDE.md` (§ *Pendant une boucle de revue*, exception `kesh-db`) ?
6. **Interblocage (F9 = R3, F8).** L'ordre de verrouillage attribué à `kesh_seed::reset_demo` est-il celui du code
   (`crates/kesh-seed/src/lib.rs` autour de 459-480 et 576 ; `docs/MULTI-TENANT-SCOPING-PATTERNS.md:324`) ? La
   conclusion « victime → `500` sans rien de partiel, comme `post_reject` » est-elle cohérente avec le point (iv) de
   `crates/kesh-api/tests/audit_route_registry.rs` (l. 88-115) où l'AC10 fait écrire cet angle mort ? La phrase
   « aucun cycle » subsiste-t-elle quelque part comme absolue (hors historique) ?
7. **`conclude_locked_attempt` (F7, R12).** Les lignes 4366 et 4384 citées sont-elles bien les deux commentaires
   d'appelants ? « Logique et `match` inchangés » est-il compatible avec le renommage `Ok(value)` partout où la
   fiche en parle (Régions, AC10, Séquence point 6, *Ce qui doit rester intact*, C-28-5-28) ?
8. **Ordre (F6 = R10).** Fiche (en-tête), registre C-28-5-29, epic § *Ordre*, `sprint-status.yaml` : même ordre,
   « 28-5a, puis 28-6 ∥ 28-5b, puis 28-7 » ? Les régions partagées 28-5b / 28-6 (`failed-proposal-label.ts`,
   `failed-proposal-label.test.ts`, `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts:601`, catalogues)
   sont-elles les mêmes aux trois endroits ? La ligne `errorMessageOf` (R11) est-elle exacte
   (`frontend/src/lib/shared/utils/api-client.ts:38` aujourd'hui) ?
9. **API et manuel (R6, F10 = R9).** `docs/api-external.md:540` est-il bien le paragraphe « Transaction disparue,
   rapprochée ou rejetée » ? Les commandes de contrôle PDF de l'AC8 sont-elles exactes ? Tu peux aplatir le PDF
   **dans `/home/gcorbaz/devel/kesh-gate-logs/` seulement** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' > /home/gcorbaz/devel/kesh-gate-logs/28-5-validate-p3-manuel-plat.txt`,
   puis rejoue les deux `grep -oE … | wc -l` de l'AC8 : la fiche annonce `0` et `3`.
10. **Recomptes depuis la source.** B1-B10, V1-V6, M1-M17 (tableau des mutations), C-28-5-1 à C-28-5-30 au registre
    (`grep -c "^### C-28-5-"`), 13 AC (16 en-têtes `**AC…`), et le Change Log « Validation P2 » : 4 MEDIUM dont
    2 distincts, 18 LOW (8 + 10) dont cinq recoupements — conforme aux deux rapports P2 ? Chaque numéro de ligne
    **neuf** de la remédiation se vérifie par `sed -n` (notamment `:4312-4399`, `bank_imports.rs:338-344` et
    `:350-355`, `reconciliation_e2e.rs:3451-4552`).

## Ce que tu rends

Findings numérotés (CRITICAL / HIGH / MEDIUM / LOW), chacun avec `fichier:ligne`, **origine** (« remédiation P2 »
ou « antérieur »), **preuve** (sortie de `grep -nF` ou `sed -n` copiée), correction proposée. ⛔ **Liste des axes
exercés ET non exercés** (avec, pour chacun, ce qui a réellement été lu ou exécuté) : un « 0 finding » sans elle ne
compte pas comme passe. Rapport complet dans
**`/home/gcorbaz/devel/kesh-gate-logs/28-5-validate-p3-ciblee.md`** ; dernier message : le chemin, le bilan par
sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris **aucun** fichier hors de `/home/gcorbaz/devel/kesh-gate-logs/` (le rapport et le PDF aplati). Aucune
commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute quoi que ce soit :
`scripts/*` (dont **`scripts/prepare-release.sh`** et `scripts/test-fast.sh`), `make`, `latexmk`, `cargo`, `sqlx`,
`npm`, `npx`, `docker`, `mariadb`/`mysql`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`/`push`/
`tag`, `gh` en écriture (`issue create`/`edit`/`comment`, `pr …`). **Aucun interpréteur** : ni `python`/`python3`,
ni `node`, ni `perl`, ni `ruby`, ni `awk`, ni `bash -c`/`sh -c` de script — y compris pour lire le YAML (lis
`sprint-status.yaml` par `grep`). Autorisés : lecture, `grep`, `sed -n`, `wc`, `sort`, `tr`, `cut`, `head`,
`tail`, `git log`/`show`/`diff`/`grep`, `pdftotext` vers `kesh-gate-logs/`.
