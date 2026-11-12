# Prompt — passe ciblée P3 de la Story 34-11b (une lentille, Opus), sur la seule remédiation de la revue P2

Tu relis **un seul commit** de la Story 34-11b de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11 ; release 0.17.6,
Epic 34, #972) : **`a9c7b967`**, la remédiation de la revue de code P2. Modèle : **Opus**, contexte frais. Langue du rapport : français.
Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e34b-review-p3-ciblee.md`.

Worktree `/home/gcorbaz/devel/kesh-e34b`. Le diff : `git -C /home/gcorbaz/devel/kesh-e34b show a9c7b967` (et, pour le contexte, les fichiers
touchés à ce commit). ⚠️ Le commit suivant, `8abf9b50` (prompt P3), a **retiré par erreur** le code de `a9c7b967` (un script de rouges
avait remis à l'index des fichiers d'avant) ; `225a90de` le **rétablit à l'identique** — vérifie-le : `git diff a9c7b967 225a90de` ne doit
rendre que ce prompt. Le dernier commit de code est donc `225a90de`, et les gates s'y rapportent. Les rapports P2 qu'il remédie : `/home/gcorbaz/devel/kesh-gate-logs/e34b-review-p2-{B,E,A}.md`. Le registre :
`_bmad-output/implementation-artifacts/e34-choix-autonomes-e34b.md` (C-e34b-13 à 15, nouveaux). La fiche :
`_bmad-output/implementation-artifacts/34-11b-carte-credit-releve.md` (§ *Écarts à la fiche*, § *Renvois à l'intégration*, *Annotations de
la remédiation P2*, Dev Agent Record).

**La question unique : la remédiation a-t-elle introduit un défaut, ou laissé une contradiction ?** (La sévérité se déplace vers ce qu'on
vient d'écrire : cherche là.)

## Ce que le commit change

1. **B2-1** — `POST /reconciliation/transactions/{id}/cancel` lit son corps **brut** (`axum::body::Bytes`) ;
   `parse_cancel_expectation` (`crates/kesh-api/src/routes/reconciliation.rs`) : vide, `null`, `{}`, champ absent ou `null`, ou corps non JSON →
   aucun contrôle ; booléen → contrôle (409 `RECONCILIATION_CANCEL_CHANGED` si la décision a changé) ; autre type → `400 VALIDATION_ERROR`.
   Test `s34_11b_b2_1_le_corps_de_l_annulation_reste_facultatif`.
2. **B2-2** — test du bras **règlement client** de l'exclusion (`s34_11b_b2_2_un_reglement_client_n_a_pas_d_autre_cote`) ; mutation M13
   scindée (M13a fournisseur, M13b client).
3. **B2-3/E2-4** — `other_side_predicate` (`crates/kesh-db/src/repositories/reconciliation_link.rs`) borne ses deux sous-requêtes par
   `company_id = bt.company_id` ; test unitaire `the_settlement_exclusion_matches_the_classification`.
4. **E2-3** — (b′) au pluriel (`already-booked.ts`, clé `-transfer-other`, quatre locales, Vitest, `sitesTotal` 2958).
5. **B2-4** — `docs/api-external.md` : tableau des refus de `…/cancel`, phrase du corps facultatif. **E2-2** — liens rustdoc.
6. Fiche et registre (A2-2 à A2-7).

## Axes obligatoires

1. **Le corps brut** : une route qui avait un extracteur `Json` passe à `Bytes` — limite de taille du corps (`DefaultBodyLimit`) ; corps
   UTF-16 / BOM ; tableau JSON `[...]`, nombre, chaîne : `value.get(...)` sur un non-objet ; un objet avec la clé en double ; la clé en
   `snake_case` (`expect_unlink_only`) silencieusement ignorée — juste ou piège ? ; l'ordre des extracteurs (`Bytes` en dernier) ; le rejeu
   `retry_app_on_deadlock` (le corps est lu une fois avant) ; les **clés d'API** (`api-external.md`, contrat « accepte tout ») ; le test HTTP
   couvre-t-il vraiment « sans corps » (reqwest sans `.body`) et `-d ''` ?
2. **L'exclusion bornée** : ordre des `?` et des `bind` inchangé (aucun paramètre ajouté ?) dans les trois requêtes (liste, geste, page) et
   `transfer_other_side` ; `bt.company_id` toujours renseigné ; le test unitaire de non-dérive — prouve-t-il ce qu'il dit, ou rougirait-il pour
   une raison textuelle sans rapport (comptage des `AND`) ?
3. **Les tests neufs** : B2-2 et B2-1 sont-ils déterministes, et chacun tue-t-il la mutation qu'il vise (relis `kesh-gate-logs/e34b/mut/`,
   `resume-j4.log`, `M13a-j3.log`, `M13b-j3.log`, `M14-j3.log`, `F5-j3.log` ; le runner suffixe `-j3`) ? Les rouges :
   `kesh-gate-logs/e34b/rouges-rem2.log`.
4. **Les textes** : (b′) pluriel dans les quatre locales, accords ; repli = fr-CH ; `sitesTotal`.
5. **La fiche et la documentation** : les § neufs de la fiche disent-ils vrai au code (M8/M9 couvertes par M2 ; seconde défense de la
   dévalidation fournisseur — vérifie au code de `supplier_invoices` qu'une écriture d'achat rapprochée refuse bien sa suppression) ;
   `api-external.md` contre `parse_cancel_expectation`.
6. **Les gates** au dernier commit de code : `kesh-gate-logs/e34b/gates-6.txt`, `front-6.txt`, `gate-*-6.log`, `e2e-r6/` — chiffres,
   échecs E2E nommés contre `docs/testing.md` (sept de la KF-029, pollution #931 possible).

## Interdits absolus

- **Lecture seule.** N'écris aucun fichier hormis ton rapport ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`) ; aucun fichier
  temporaire. Un fichier créé par erreur se déclare.
- **Aucun `rm`**, `pkill`, `killall` ; **aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Rien qui compile ou exécute** : ni `cargo`, `npm`/`npx`/`node`, `python3`, `perl`, `bash -c`, `jq`, `make`, `docker`, `mariadb`, `sqlx`,
  `curl`, `playwright`, `vitest`, ni aucun script du dépôt ou de `kesh-gate-logs` (**`scripts/prepare-release.sh`** compris), ni `claude`,
  ni `sed -i`. **Aucune commande `gh`**, aucun réseau. Rien sous `~/kesh/` ni le NAS.
- Permis : `git show`, `git diff`, `git log`, `git grep`, `/usr/bin/grep` (`grep` est une fonction vers `ugrep`), `sed -n`, `awk` sans
  redirection, `cat`, `ls`, `find` sans `-exec`, `wc`, `sort`, `uniq`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` ou `sed -n` copiée.

## Rapport

Pour chaque finding : `C3-n`, sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier:ligne, preuve, conséquence, correction proposée. Puis la
**liste des axes exercés et de ceux qui ne l'ont pas été** — un « 0 finding » sans elle ne compte pas.
