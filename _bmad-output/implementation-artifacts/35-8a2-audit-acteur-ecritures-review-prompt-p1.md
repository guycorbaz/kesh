# Prompt — revue de code P1 de la Story 35-8a2 (refs #431), trois lentilles, première passe complète

Tu relis la **Story 35-8a2** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.7**, Epic 35
(« corrections par impact ») : **l'audit attribue à la clé d'API les écritures qu'elle passe, contre-passe, modifie ou supprime**, à la
main ou par un rapprochement bancaire. La 35-8a1 (close) a posé le type `Acteur`, `NewAuditLogEntry::par`, `From<&CurrentUser>` et la
garde des sites non résolus (`crates/kesh-api/tests/audit_acteur_garde.rs`). Cette story fait prendre l'`Acteur` aux fonctions de
`crates/kesh-db/src/repositories/journal_entries.rs` (`create`, `create_in_tx`, `reverse*`, `update`, `delete_by_id`, `delete_in_tx`),
le passe depuis les routes du journal, les cinq sites de `routes/reconciliation.rs`, le dé-rapprochement d'une écriture seule
(`reconciliation_cancel.rs`) et `supplier_invoices::pay_in_tx_as` ; les autres appelants passent `Acteur::utilisateur`, inscrits à la
garde. **Aucune migration.** Kesh tient des livres réels : une trace d'audit qui ment sur l'auteur est une piste de contrôle fausse
(CO art. 957-964, OLICo art. 9).

Worktree `/home/gcorbaz/devel/kesh-e358a2`, branche `story/35-8a2-audit-acteur-ecritures`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e358a2 diff 4531669f8 6d68be121` (`4531669f8` = la base fusionnée — 35-5c, 35-5b, 35-8a1 avec la
35-1, 35-3 ; `6d68be121` = le dernier commit de code, précédé du commit des rouges `3be2d577c` ; les commits suivants ne portent que les PDF, la fiche,
`sprint-status.yaml`, le registre et ce prompt). ⚠️ Le diff compte ≈ 65 fichiers dont ≈ 50 de tests à conversion **mécanique**
(l'argument utilisateur devient `Acteur::utilisateur(…)` ; la paire `(u, None)` de `update`/`delete_by_id` devient un seul argument) :
la conversion des tests a été faite par trois agents, à relire **fichier par fichier** (une valeur changée au passage, un `Some(clé)`
perdu, un appel d'une autre fonction touché à tort). Langue du rapport : français. Modèle : **Opus** (première passe : jamais Haiku).
Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/35-8a2-audit-acteur-ecritures.md` en entier (*Ce qui se lit au code*, D3 à D5, AC1, AC2, AC4,
AC5b, AC6b, mutations, **Change Log** et **Dev Agent Record**) ; la fiche de la 35-8a1 (`35-8a1-audit-acteur-mecanisme.md`, D1, D2 —
la garde, ses motifs, ses raisons, `REFERENCES`) ; les registres `e35-choix-autonomes-e358a2.md` (C-e358a2-1 à -6) et
`e35-choix-autonomes-e358a1.md`. Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt
(lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e358a2/` : `rouges.log`, `conversion-erreurs-2.txt`, `garde-releve-1.log`,
`verts-cibles.log`, `mut/*.log`, `gate-test-fast.log`, `gate-ci.log`, `clippy-1.log`, `manuels-make.log`, et ceux de la base
(`base-*.log`). Déjà tracés, à ne pas rapporter de nouveau : **#1130** (le `None` positionnel des autres fonctions à
`actor_api_key_id: Option<i64>`), **#1131** (annulation d'un rapprochement sans clé, `reconciliation_cancel::cancel`), **#1132**
(`pieges.rs`, `mark_emailed`).

**Livrables visibles** : aucun écran ni route ne change ; ce qui change est **ce que le journal d'audit inscrit** (`actor_type`,
`actor_api_key_id`). **Le manuel** : `docs/manual/fr/admin-manual.tex` (item *Journal d'audit* de la section des clés d'API, la
phrase « Depuis la v0.17.7 encore… ») et `docs/manual/fr/user-manual.tex` (le tableau du journal d'audit, « de même que les
écritures… ») **et leurs PDF** `docs/manual/fr/admin-manual.pdf`, `docs/manual/fr/user-manual.pdf`, lus **aplatis** :
`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` (lecture seule, sortie à l'écran, sans redirection). Le
`CHANGELOG.md` (`## [0.17.7] — Non publié`, *Corrigé*, l'entrée « Les écritures qu'une clé d'API passe ou contre-passe… »). Le manuel
dit-il **exactement** ce que le code fait, geste par geste (AC6b) ? C'est en vérifiant le manuel qu'on trouve les chemins oubliés.
Pas de capture : aucun contrôle visible ne change.

⚠️ **Ce qui n'a pas tourné** : l'**E2E (Playwright)** — à la clôture, avec un second `gate-ci` (consignes de l'Epic 35). Le frontend
n'est pas touché (frontend complet joué sur la base seulement).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle part.
- **Aucune redirection, sous aucune forme** : ni `>`, `>>`, `<`, `tee`, ni `2>`, `&>`, **ni `2>&1`**, **ni `2>/dev/null`**, ni
  `> /dev/null`. **Aucune substitution de processus** (`<(…)`, `>(…)`) ni de commande qui écrive. Un fichier créé par erreur se
  **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`reset`/`commit`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`,
  `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`. `pdftotext` est permis, sortie à l'écran.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext … -`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Les signatures de `journal_entries.rs`** : chaque fonction convertie fait-elle parvenir l'`acteur` **jusqu'à chaque trace** qu'elle
   écrit (`journal_entry.created`, `.reversed`, `.updated`, `.deleted`, le lettrage `reversal`) ? Un `acteur.user_id` resté là où
   l'acteur entier devait passer ? `create_opening_entry` garde `user_id: i64` (C-e358a2-5) : est-ce cohérent avec AC6b ?
2. **Les six sites qui ont la clé** (`accept_one_invoice`, `accept_one_split`, `accept_one_rule`, `post_manual_once`,
   `post_split_once`, `reconciliation_cancel.rs` genre `Entry`) et **`pay_in_tx_as`** (C-e358a2-4, hors de la liste de la fiche) :
   chacun passe-t-il la **bonne** clé (celle de la requête), et l'acteur porté jusqu'au `create_in_tx` est-il le même que celui des
   autres traces du geste (`reconciliation.accepted`, `.manual_matched`, `lettering.created`) ? Un chemin de rapprochement qui écrit
   une écriture sans passer par ces sites (`accept_one_supplier_invoice` → `pay_in_tx_as` ; le règlement client d'`accept_one_invoice`
   par `invoice_settlements` ?) — énumère **tous** les chemins d'écriture d'un rapprochement par clé et dis, pour chacun, ce que le
   journal inscrit.
3. **Les appelants convertis à `Acteur::utilisateur`** : chacun écrit-il exactement l'entrée d'avant (AC4) ? Un appelant qui **avait**
   la clé en main et l'a jetée (un site `AConvertir` qui aurait dû recevoir l'acteur, comme `pay_in_tx_as`) ?
4. **La garde** (`audit_acteur_garde.rs`) : la table (93 entrées, 98 sites ; `Utilisateur` 12 ; `DeleteInTxNone` 0) recomptée depuis
   la source ; **aucune** entrée `Utilisateur` pour `routes/reconciliation.rs`, `routes/journal_entries.rs`,
   `repositories/reconciliation_cancel.rs` (AC5b) ; la raison `Fixture` et sa référence close (`pieges::semer`, liste vide : vrai ?) ;
   les colonnes « appelant » des entrées neuves (lignes de `lib.rs` citées : justes ?) ; l'angle mort 1 recompté (23 lignes de
   paramètre) ; le motif 5 désormais sans site — sa sonde (h) tient-elle encore ?
5. **Les tests neufs** (AC1, AC2, AC4) : rouges prouvés sur la base (`rouges.log`) pour la **bonne** raison ; AC2 asserte d'abord
   qu'un `lettering.created` d'origine `reversal` **existe** ; les deux contre-passations (du jour, à l'exercice suivant) ; le témoin
   AC4 (validation d'une facture par clé → `'user'`) asserte-t-il la limite **telle quelle** ? Un test qui passerait à vide (une trace
   cherchée sur la mauvaise entité, un `fetch_one` qui prend la première de plusieurs) ?
6. **La conversion mécanique des tests** (≈ 50 fichiers) : relis-la fichier par fichier ; un `create_opening_entry` converti à tort
   (corrigé dans `journal_entry_reversal_e2e.rs`), une destructuration touchée par un remplacement (`let (…, admin_user_id, …)`), une
   valeur d'utilisateur changée ?
7. **Mutations** (`mut/`) : M2', M3, M7, M8, M9 tuées par les tests ou la garde **nommés** ; chaque journal finit par « restauré ».
   Propose des mutations plausibles non jouées (un `pay_in_tx_as` qui jette la clé ; `update` qui trace par `acteur.user_id` seul).
8. **Manuel, PDF, CHANGELOG** (AC6b) : la phrase, genre par genre, dit-elle le code ? En particulier : « celles qu'un rapprochement
   bancaire passe » couvre-t-il le règlement d'une facture **fournisseur** par rapprochement (C-e358a2-4) et d'une facture **client** ?
   Le dé-rapprochement d'un règlement « au nom du créateur de la clé, contre-passation comprise » : vrai au code
   (`reconciliation_cancel.rs`, `cancel_settlement_in_tx`) ? Et `docs/api-external.md` : une phrase devenue fausse ?
9. **Le compte rendu** : chaque chiffre (appels convertis, table, gates) recompté depuis la source, avec son périmètre et son journal ?
   Le Dev Agent Record n'affirme-t-il que ce qui a tourné (E2E : **non lancé**) ?

## Lentille B — Blind (le diff seul, sans la fiche)

Relis `journal_entries.rs` (production puis `mod tests`), `routes/journal_entries.rs`, `routes/reconciliation.rs`,
`reconciliation_cancel.rs`, `supplier_invoices.rs`, `invoices.rs`, `invoice_settlements_write.rs`, `credit_notes.rs`,
`opening_complement.rs`, `letterings.rs`, `kesh-seed/src/pieges.rs`, la garde, puis les tests. Cherche : une clé perdue entre la
route et la trace ; une fermeture `async move` qui capture le mauvais acteur ; un commentaire devenu faux ; un import inutile.

## Lentille E — Edge (chaque branche, chaque borne)

Une clé `read-write` d'un Comptable, d'un Administrateur ; une session ; un rejeu sur interblocage (`retry_on_deadlock` : l'acteur est
calculé hors de la fermeture — même valeur au second passage ?) ; une contre-passation qui ne lettre rien (aucun compte lettrable) ;
un compte lié à un compte bancaire (non lettrable) ; le dé-rapprochement des trois genres (`Entry`, règlement client, règlement
fournisseur) et le simple détachement ; un lot `accept` mêlant les genres. Chaque cas : comportement au code, test qui le couvre (ou
absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1, AC2, AC4, AC5b, AC6b : livré, testé, au lieu prescrit ? AC1 « un cas de `POST /reconciliation/accept` si le
montage tient, sinon les trois sites couverts par la garde et nommés au Change Log » : le choix est-il dit ? Les écarts déclarés
(C-e358a2-4, -5, -6) sont-ils exacts ? L'étape 0 (lignes relues, rayon recompté) citée ? Les gates du passage en `review` : chaque
chiffre au journal, au commit `6d68be121`, horodaté après lui ? L'E2E déclaré « à la clôture » ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e358a2-review-p1-B.md`, `…-p1-E.md`, `…-p1-A.md`. Chaque
rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue
copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte final **recompté depuis la
liste**.
