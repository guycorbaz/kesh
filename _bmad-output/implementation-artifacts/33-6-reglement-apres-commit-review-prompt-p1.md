# Prompt — revue de code P1 de la Story 33-6, trois lentilles (B, E, A), Sonnet ou Opus

Tu relis la **Story 33-6** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11), version **0.17.5**, sans migration :
**le règlement n'est plus annoncé en échec après son commit (#450)** ; même famille : la décision d'un envoi MCP (#768, point 1) et la réponse perdue (#773, point 3).
Kesh tient des livres réels : ce code écrit au journal.

Worktree `/home/gcorbaz/devel/kesh-e335`, **où plusieurs stories se succèdent** : relis par `git show`/`git diff` sur la branche
`story/33-6-reglement-apres-commit`, jamais le fichier sur disque. Diff **aplati** à relire :
`git -C /home/gcorbaz/devel/kesh-e335 diff bdc3faa4 story/33-6-reglement-apres-commit`. Passe **complète**, contexte frais, langue du rapport : français.
Rapports : `/home/gcorbaz/devel/kesh-gate-logs/e336-review-p1-{B,E,A}.md`, un par lentille, et rien d'autre.

⚠️ **Les propriétés centrales.**
1. **Ce que rend la réponse est ce que le commit écrit** : lecture après la dernière écriture et avant le commit, aux trois fonctions. Cherche une écriture placée après la lecture
   (lettrage, audit, version, `updated_at`), et les niveaux d'isolation (`REPEATABLE READ` : qu'est-ce qui établit l'instantané, et voit-on ses propres écritures ?).
2. **Pas de commit sans réponse** : une lecture en échec annule tout, y compris dans `cancel_settlement`, dont l'enveloppe a changé (rollback, chemins d'erreur).
3. **Le rejeu sur interblocage reste sûr** : la lecture est dans la fermeture de `retry_on_deadlock`.
4. **`cancel_settlement_in_tx` ne lit rien de plus** : le dé-rapprochement ne change pas.
5. **`find_by_id_with_lines` et `mcp_sends::trouver` passent par une connexion acquise** : tout appelant garde son comportement ; le pool, sa charge.
6. **`erreurDeDecision`** : la fiche parlait d'une erreur « hors `ApiError » ; le client lève en fait une `ApiError` de statut 0 pour le réseau et le délai (C-e334-33). Une réponse 500 ne propose pas *Recharger* : est-ce juste ?

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant, seule) : la fiche `_bmad-output/implementation-artifacts/33-6-reglement-apres-commit.md` en entier, dont le **Change Log** et le **Dev Agent Record** ;
le registre `e33-choix-autonomes-e334.md` (C-e334-8 à 10, 33) ; `crates/kesh-db/src/repositories/invoice_settlements_write.rs`, `invoices.rs`, `mcp_sends.rs` ; `crates/kesh-api/src/routes/invoices.rs`, `mcp_envois.rs` ;
`crates/kesh-api/tests/invoice_settlement_response_e2e.rs` ; `frontend/src/lib/features/mcp-sends/mcp-sends-format.ts`. Ne prends pas le Dev Agent Record pour argent comptant :
recoupe-le avec les journaux hors dépôt, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/e335/` : `rouges.log`, `mut/synthese-33-6.log`, `mut/33-6-M*.log`
(`mut-33-6.py` est à **lire**, jamais à exécuter), `33-6/*.log`, `e2e-r33-6/`. **Capture** (thème clair) : `/home/gcorbaz/devel/kesh-gate-logs/e335/captures/33-6-reponse-perdue-recharger.png`.

⛔ Ne lis rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`, ni le NAS. Fixtures synthétiques seulement.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`reset`/`rebase`/`push`/`tag`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`,
  `latexmk`, `cargo`, `docker`, `mariadb`, `curl`, `claude`, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`** ; aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep` (dans ce shell, `grep` nu est une fonction),
  `sed -n`, `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Les manuels (`.tex` et PDF aplati) contre le code** : `docs/manual/fr/admin-manual.tex` (§ des envois du serveur MCP, *Envoyer tel quel*), son PDF aplati dans
   `/home/gcorbaz/devel/kesh-gate-logs/e335/admin-manual-33-6.txt` ; `docs/manual/fr/user-manual.tex` et `docs/api-external.md:503-521` : décrivent-ils un 404 après règlement (`/usr/bin/grep -n 'règlement.*404' docs/api-external.md docs/manual/fr/*.tex`) ? `CHANGELOG.md` dit-il vrai ?
2. **Les appelants** : `settle_invoice`, `write_off_invoice`, `cancel_settlement` en `tests/`, `reconciliation`, `kesh-report` ; un autre chemin d'écriture de règlement non traité côté client ?
3. **La garde lexicale** : peut-elle passer à vide ? La sonde positive tient-elle ? Le découpage du corps et la borne `.await?;` ?
4. **La cohérence des montants** : `amount_due` du solde, `amount_due_after` du règlement, arrondi au centime du GET (`factures::detail`) contre valeur de la tx.
5. **Les tests qui prouvent moins** : un test vert par construction, un montage qui n'exerce pas la branche voulue, les mutations déclarées (survivante ? équivalente abusive ?).
6. **L'écran** : le bouton *Recharger* sur une réponse perdue, la capture.

## Lentilles

- **B (Blind Hunter)** : le diff seul d'abord, sans fiche ni contexte du projet ; puis le code autour. Que casse ce diff ?
- **E (Edge Case Hunter)** : chaque branche et chaque borne : facture sans ligne, avoir partiel, solde après règlement, annulation qui laisse un reste, échec du commit, interblocage rejoué, envoi refusé (contenu masqué) ou échu dans la réponse de décision.
- **A (Acceptance Auditor)** : chaque AC de la fiche (1 à 8), T1 à T5, chaque mutation (M1 à M5), chaque choix du registre : présent, testé, vrai.

## Le rapport

Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), preuve copiée, conséquence, remédiation proposée. Ensuite :
**la liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe. Le décompte par sévérité,
recompté, en tête.
