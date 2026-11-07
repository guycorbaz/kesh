# Prompt — revue de code P1 de la Story 33-4, trois lentilles (B, E, A), Sonnet ou Opus

Tu relis la **Story 33-4** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11), version **0.17.5**, sans
migration : **une date reçue du client dont l'année sort de 1000 à 9999 est refusée, jamais rendue en 500 ni acceptée en silence
(#606) ; une clé d'API voulue temporaire ne devient pas permanente (#755)**. Kesh tient des livres réels.

Worktree `/home/gcorbaz/devel/kesh-e334`, branche `story/33-4-bornes-annees`. Diff **aplati** à relire :
`git -C /home/gcorbaz/devel/kesh-e334 diff bdc3faa4 HEAD`. Passe **complète**, contexte frais, langue du rapport : français.
Rapports : `/home/gcorbaz/devel/kesh-gate-logs/e334-review-p1-{B,E,A}.md`, un par lentille, et rien d'autre.

⚠️ **Les propriétés centrales.**
1. **Aucun champ daté reçu n'échappe.** L'inventaire de la fiche et la garde `tests/dates_recues_garde.rs` : cherche un site qu'ils
   n'énumèrent pas (une chaîne de date analysée à la main hors de `analyser_date`, `parse_as_of`, `audit_log::parse_date` ; un type daté
   derrière un alias ; un `Deserialize` manuel ; un fichier reçu ; une date calculée liée en SQL).
2. **La classe est celle du relevé**, pas une lecture : `kesh-gate-logs/e334/t1-code-inchange-{1,2,3}.log`, `s2.txt`, contre `Attendu` dans
   `tests/dates_bornes_e2e.rs` et `EXCEPTIONS` dans la garde. Un champ classé A dont une valeur rendrait 500 ou 2xx ?
3. **Le `borner` précède tout usage de la date** dans chaque handler de classe B, et précède toute lecture de la base quand c'est possible ;
   `sent_at` après le refus du futur. Un chemin d'écriture de la même donnée qui ne passe pas par le handler borné (route sœur, service, import, MCP) ?
4. **Le champ absent reste absent** : tout `date_opt` porte `default` ; un `null` reste `None` ; `Query` avec `?dateFrom=` (vide) ne change pas de comportement.
5. **La frontière avec la 33-2** : `git diff bdc3faa4 HEAD -- crates/kesh-import/src/camt053/mod.rs` n'ajoute que `parse_corrige` et
   `refuser_annees_hors_plage` ; `parse_date`, `parse_with_version`, `parse_legacy_0173`, `handle_text`, `flush_pending` ne bougent pas. Une clé de doublon qui changerait ?
6. **#755** : l'année se lit en UTC, `mcp_keys.rs` garde sa clé i18n.
7. **Contrats existants** : `bank_accounts_closed_e2e`, `audit_log_e2e`, `open_items_e2e`, `contact_payment_terms_e2e`, `mcp_outils_e2e`,
   `dunning_reminders_e2e` sans modification de leurs corps ; `docs/api-external.md:241` (*closedOn*) inchangé.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant, seule) : la fiche
`_bmad-output/implementation-artifacts/33-4-bornes-annees.md` en entier, dont le **Change Log** et le **Dev Agent Record** ; le registre
`e33-choix-autonomes-e334.md` (C-e334-1 à 4, 26, 32 à 36). Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux
hors dépôt, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/e334/` : `rouges.log`, `mut/synthese.log`, `mut/M*.log` (le script `mut/run.py`
est à **lire**, jamais à exécuter), `gate-*.log`, `front-*.log`, `e2e-r*/`.

**Captures** : aucune, aucun contrôle visible ne change (à contester si tu en trouves un). **Manuel** : la fiche conclut qu'aucun `.tex` ne change ; **vérifie-le** :
`docs/manual/fr/user-manual.tex`, `docs/manual/fr/admin-manual.tex`, et leurs **PDF** `docs/manual/fr/user-manual.pdf`, `docs/manual/fr/admin-manual.pdf`, aplatis
(`pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`) : une phrase qui promet qu'une date passe, une borne, l'import d'un relevé à date ancienne, le rappel manuel ?
`docs/api-external.md` (§ 5, nouveau paragraphe *Les dates*, et § sur `POST /reconciliation/accept`) et `CHANGELOG.md` : le statut dit est-il celui du code ?

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

1. **Les manuels (`.tex` et PDF aplati) contre le code**, comme dit plus haut.
2. **L'inventaire** : un site non résolu ; refais le balayage sans partir de la liste, depuis le symptôme (un type daté reçu, `parse_from_str`, `NaiveDate::from_str`, `.parse::<NaiveDate>()`, `push_bind` d'une date).
3. **Les statuts** : chaque `Attendu` contre le handler ; la position du `borner` ; le refus d'un `Query` et d'un `Json` (texte) ; le refus du lot `accept` (extracteur à message fixe, *Pattern batch*).
4. **La garde** : ses formes manquées ; une sonde qui passe à vide ; ce qu'elle laisse passer (un `borner` sur le mauvais champ, une preuve de classe A trop lâche : `"+10000"`).
5. **Les tests** : un test muet ; un témoin de forme qui ne prouve rien ; ce que le relevé `0` (connexion coupée) masque ; le `Monde` qui fausserait une mesure.
6. **Le CAMT, le CSV et la saisie manuelle** : l'ordre des refus, la réponse HTTP de l'import, `guard_header_skips_data`, l'aperçu contre la création.
7. **Les décomptes** du Change Log et du Dev Agent Record, recomptés à la source.

Lentille **B** (Blind Hunter) : le diff seul, sans fiche. Lentille **E** (Edge Case Hunter) : chaque branche, chaque borne. Lentille **A** (Acceptance Auditor) : chaque AC-1 à AC-13 et chaque mutation M1 à M11 contre la preuve citée.

## Le rapport

Sévérités CRITICAL, HIGH, MEDIUM, LOW, chaque finding avec fichier, ligne, extrait lu et remédiation. **La liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas. Aucun mot-clé de fermeture (`closes`, `fixes`, `resolves`).
