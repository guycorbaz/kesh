# Prompt — revue de code P3 ciblée de la Story 35-5a0 (refs #929, #1022, #907), une lentille, braquée sur la remédiation de la P2

Passe **ciblée**, une seule lentille, contexte frais, **Opus** (rotation : P1 Opus, P2 Sonnet), en lecture seule. Worktree
`/home/gcorbaz/devel/kesh-e355a0`, branche `story/35-5a0-horloge-plomberie`. Historique : P1 (trois lentilles) 3 MEDIUM → remédiation
`c8cda87e5` → P2 ciblée (`35-5a0-horloge-plomberie-review-prompt-p2-ciblee.md`, rapport
`/home/gcorbaz/devel/kesh-gate-logs/e355a0-review-p2-ciblee.md`) 1 MEDIUM (C1 : l'inventaire de D4 — un seul jour par test, tiré de
l'horloge de test — n'était pas clos ; son détecteur ne relevait que des variables nommées `today`/`aujourd_hui`/`jour`) et 1 LOW (C2,
tracé en #1107). Le même défaut revient pour la deuxième fois : ta passe juge si la troisième forme est fermée.

## Périmètre : le seul commit `1a9af93b9`

`git -C /home/gcorbaz/devel/kesh-e355a0 show 1a9af93b9` — onze fichiers de tests seulement (aucun fichier de production : vérifie-le par
`git show --stat`). Compte rendu : Change Log de la fiche `_bmad-output/implementation-artifacts/35-5a0-horloge-plomberie.md` (commit
`cfbbfcff4`). Détecteur parti du symptôme : `/home/gcorbaz/devel/kesh-gate-logs/e355a0/inventaire-d4-symptome-e355a0.py` (à **lire**,
jamais à exécuter), sorties `r2-inventaire-symptome.txt` (avant : 47 fonctions) et `r2-inventaire-symptome-apres.txt` (après : 6, hors
champ).

## Ce que tu vérifies

1. **Le détecteur, sans partir de sa liste** : lis le script. Relève-t-il vraiment toute lecture de l'heure réelle (`Utc::now`,
   `Local::now`, `SystemTime::now`, `chrono::offset::…`, `NaiveDate::from…(now)`, une macro, un `use chrono::Utc as …`), et tout helper
   qui en dérive **transitivement**, y compris d'un **autre fichier** (un module `support/` ou `common/` appelé depuis un test) ? Sa
   portée (`crates/*/tests/**`, et dans `src` seulement après `#[cfg(test)]`) manque-t-elle un lieu (`mod tests` déclaré sans
   `#[cfg(test)]` sur la même ligne, fichier `tests.rs` séparé, `benches/`) ? Puis cherche **toi-même** :
   `git grep -nE 'Utc::now|Local::now|SystemTime::now' -- 'crates/*/tests/*' 'crates/*/tests/**'` et, pour chaque fichier qui porte aussi
   `aujourd_hui_test`, regarde si une même fonction de test mêle les deux. Copie au rapport ce que tu trouves, même si c'est vide.
2. **Les conversions** : un échantillon d'au moins dix fonctions sur les 41, dont `open_items.rs` (`aujourdhui()`, `:764`),
   `filet_bilan_clos.rs` (`societe_du_jour`, `:567`, `:683`), `invoice_settlement.rs` (`facture_lettree`, `:2234`),
   `supplier_invoices_repository.rs` (`monter`, `monter_achat`, `:2979`), `letterings.rs:1463`, `:2392`, `reconciliation_cancel.rs:469`,
   `open_items_invariant.rs:235`, `admin_full_import_e2e.rs:2008`, `reconciliation_e2e.rs:4285` et le `jours_avant` du support. Chaque
   test a-t-il **un seul** jour, passé à l'appel, aux dates relatives du montage et aux assertions ? Une conversion a-t-elle changé ce
   que le test prouve (un intervalle de dates qui ne couvre plus l'exercice, une date de pièce qui tombe hors exercice si l'horloge de
   test est fixée au 1er janvier ou au 31 décembre, une comparaison devenue tautologique) ?
3. **Les 6 hors champ** (`admin_full_import_e2e.rs:123`, `reconciliation_e2e.rs:131`, `:550`, `letterings_e2e.rs:790`,
   `support/lettering_documents.rs:1047`) : chacun ne lit-il qu'un **instant** (jeton, empreinte) sans en tirer de jour comparé à celui
   de l'appel ? Ligne lue copiée pour chacun.
4. **La garde** (`date_du_jour_garde.rs`, −52 lignes) : neuf entrées retirées, `admin_full_import_e2e.rs` 4→3. Les comptes restants se
   recomptent-ils au code (`git grep -c`) ? La garde garde-t-elle encore une prise : rougirait-elle si l'on réintroduisait
   `Utc::now().date_naive()` dans l'un des fichiers retirés (lis sa logique : une entrée retirée veut-elle dire « 0 attendu » ou « plus
   contrôlé ») ? Si elle ne contrôle plus ces fichiers, est-ce un affaiblissement par rapport à `c8cda87e5` ?
5. **Le compte rendu** : l'entrée P1 qui disait l'inventaire clos est-elle annotée (datée) sans être réécrite (`git show cfbbfcff4`,
   lignes `-`) ? Les décomptes (47 relevées, 41 converties, 6 hors champ ; neuf entrées retirées) se recomptent-ils depuis les sorties et
   le diff ? L'écart `pkill -f … -n` est-il déclaré tel quel ?
6. **Les gates** au commit `1a9af93b9` : test-fast 5344/5344, gate-ci 5351, garde 7/7 — au journal sous `kesh-gate-logs/e355a0/`,
   postérieurs au commit ? La dispense d'E2E et de frontend tient-elle (aucun fichier de production ni sous `frontend/`) ?

## Interdits absolus

Lecture seule : aucun fichier écrit hormis ton rapport, aucune redirection (`>`, `>>`, `tee`, `2>`, `2>&1`, `2>/dev/null`, `> /dev/null`
compris), aucun fichier temporaire. Aucun `rm`, `pkill`, `kill`, aucun `git` qui écrit. Aucun interpréteur ni script (`python3`, `perl`,
`node`, `npm`, `npx`, `bash -c`, `sh -c`, `jq`, `scripts/*` dont **`scripts/prepare-release.sh`**, les scripts de `kesh-gate-logs`,
`make`, `cargo`, `docker`, `mariadb`, `curl`, **`claude`**, `sed -i`, `vitest`, `playwright`), aucune commande `gh`, aucun accès réseau.
Ne lis rien sous `~/kesh/` ni `~/kesh-test/`. Permis : `git show/diff/log/grep/ls-files`, `/usr/bin/grep`, `sed -n`, `awk` sans
redirection ni `system()`, `cat`, `ls`, `find` sans `-exec`/`-delete`, `wc`, `sort`, `uniq`. ⚠️ `grep` est une fonction vers `ugrep` :
emploie `/usr/bin/grep` ou `git grep`. Toute affirmation de présence ou d'absence s'appuie sur une sortie copiée au rapport. Un fichier
créé par erreur se déclare.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e355a0-review-p3-ciblee.md` : findings numérotés (`D1`…), sévérité (CRITICAL/HIGH/MEDIUM/LOW), lieu,
ligne lue copiée, preuve, remède ; **axes exercés et non exercés** ; décompte final recompté depuis la liste.
