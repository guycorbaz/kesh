# Prompt — revue de code P2 de la Story v0172-session, **lot S (serveur)**, remédiation de la P1, trois lentilles, Opus

Tu relis la **remédiation de la revue de code P1** du **lot S** (serveur) de la **Story v0172-session** de Kesh (comptabilité
suisse, Rust/Axum, SvelteKit, MariaDB 10.11), version **0.17.3** : la durée d'inactivité de la session, réglée dans
l'interface (#876, CR #891). Modèle : **Opus**. Passe **complète** sur un **périmètre restreint** : le diff de la
remédiation et ses interactions avec le code qu'il touche. Contexte frais. Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-0172s`, branche `story/v0172-session`. Le diff à relire, aplati :
`git -C /home/gcorbaz/devel/kesh-0172s diff 11e5a38b ff20983d` (`11e5a38b` = tête relue par la P1 ; `ff20983d` = commit
de remédiation du lot S). Les commits suivants (lot N, comptes rendus) sont hors de ta lentille, sauf pour relire la fiche,
le registre et le PDF du manuel.

**Ce que la remédiation prétend faire** (à vérifier, non à croire) :
1. **S-B1/S-E1, reclassé MEDIUM par l'orchestrateur** : deux premiers `PUT /api/v1/admin/session-settings` simultanés
   (version 0, table vide) s'interbloquaient (InnoDB 1213 ⇒ 500). La route `session_settings::update_session_settings`
   enveloppe désormais l'appel du dépôt dans `kesh_db::retry::retry_on_deadlock` ; le rejeu relit la ligne et rend 409
   (choix **C-v0172s-59**). Test de course `concurrent_first_puts_yield_a_conflict_not_a_500`
   (`crates/kesh-api/tests/session_settings_e2e.rs`), rouge avant correctif (`kesh-gate-logs/v0172s/rem-p1-rouge-S1.log`).
2. **S-B4/S-E8** : `routes/auth.rs`, `activity` — le corps est extrait en `Result<ActivityBody, Response>` et jugé **après**
   `ensure_not_pat` ; le corps du battement passe dans `activity_with`. Test : `heartbeat_refuses_api_keys` (corps illisibles
   ⇒ 403, témoin session ⇒ 400). Mutation `MS2-pat-apres-corps` (`kesh-gate-logs/v0172s/mut-rem-p1/`).
3. **S-B5/S-E7** : `jwt::decode` sous `#[cfg(test)]`.
4. **S-A1** : le manuel d'administration (§ *Durée d'inactivité de la session*, paragraphe **Mise à jour**) et le CHANGELOG
   `[0.17.3]` disent qu'avec le contournement `KESH_JWT_EXPIRY_MINUTES=480` une session ouverte avant la mise à jour prend fin
   à sa première activité, et qu'il faut se reconnecter une fois. Le code ne change pas.
5. **S-E9** : un paragraphe **Clients hors navigateur** (manuel) et une entrée du CHANGELOG — une connexion par mot de passe
   renouvelée par le corps ne se prolonge plus ; les clés d'API ne sont pas concernées.
6. **LOW** : S-A3 (doc de module d'`audit_route_registry.rs`), S-A4 (tableau de `crates/kesh-api/README.md`), S-A7
   (typographie, `admin-manual.tex`, § recovery).

Lis d'abord : la fiche `_bmad-output/implementation-artifacts/v0172-session.md` (*Conception retenue* § 1 à 4, AC-1 à AC-5,
le **Change Log**, entrée « Revue de code P1 », et le **Dev Agent Record**), le registre `v0172-choix-autonomes.md`
(C-v0172s-58 à 61), les six rapports de la P1 (`/home/gcorbaz/devel/kesh-gate-logs/v0172s-review-p1-{S,N}-{B,E,A}.md`),
`crates/kesh-db/src/retry.rs` (doc de module : ce que fait InnoDB d'un interblocage, la règle « une tentative = une
transaction neuve »), et le registre `crates/kesh-api/tests/audit_route_registry.rs` (enum `Rejeu`, volets (c) et (c bis)).
Journaux hors dépôt, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/v0172s/` : `rem-p1-rouge-S1.log`,
`rem-p1-cible-S1.log`, `mut-rem-p1/*.log` (le script `scripts/mut-v0172s-rem-p1.py` est à **lire**, jamais à exécuter),
`rem-p1-gate-backend.log` (remise à neuf des bases, `test-fast.sh --ci`, build), `e2e-r4.log` et `rem-p1-e2e-run-r4.log` (le r2 a été interrompu par un OOM de la station le 2026-11-04 à 21:48 ; le r3, arrêté au démarrage sur une base non remise à neuf, ne compte pas ; le r4 a été rejoué le 2026-11-05 sur `d38fcd8b`, base `kesh_e2e_v0172s` recréée vide)
(E2E complet de la remédiation).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`, `tee`,
  `2>`, `&>`, `2>&1`) : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au rapport.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**.
- **Aucun `git checkout`/`switch`/`restore`**, aucun `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`,
  `worktree`, `add`, `fetch`, `merge`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, ni `perl`, ni `node`/`npm`/`npx`, ni
  `bash -c`/`sh -c`, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`,
  ni `scripts/regen-test-schema.sh`, ni **aucun script** du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`,
  **`cargo`**, **`docker`**, `mysql`/`mariadb`, `sqlx`, **`curl`**, **`claude`**, **`sed -i`**, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep` (dans ce shell, `grep` est une
  fonction vers `ugrep`), `sed -n`, `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`),
  `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⛔ Sur le PDF aplati, `grep -oF … | wc -l` ou `grep -cF` sur des **motifs fixes**, sous `timeout 60` ; l'apostrophe s'y rend
  `’` : compte les deux graphies.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Le manuel (`.tex` et PDF aplati) contre le code** : `docs/manual/fr/admin-manual.tex`, § *Durée d'inactivité de la
   session* (`\label{sec:duree-inactivite}`, paragraphes **Mise à jour** et **Clients hors navigateur**), le § recovery
   (« révoquées par le recovery ; elles ne peuvent plus être renouvelées »), et `admin-manual.pdf` aplati. Chaque phrase
   neuve est-elle vraie au code (le 401 du battement sur une ligne échue, `set_idle_deadline` ; le chemin des clés d'API au
   middleware, `validate_pat`, que `idle_exp` ne concerne pas ; `refresh` qui recopie l'échéance) ? Le PDF porte-t-il le
   texte du `.tex` ? Le CHANGELOG `[0.17.3]` dit-il la même chose ?
2. **Les témoins et les mutations** : le test de course peut-il passer **à vide** (cycle non formé, transaction de test
   victime, rejeu non journalisé) — qu'exige-t-il pour l'exclure ? Le test du battement par clé d'API : le 403 vient-il bien
   d'`ensure_not_pat` et non d'une autre couche (lis le routage de `/api/v1/auth/activity`) ? Chaque mutation de
   `mut-rem-p1/` est-elle tuée **par le test attendu** ? Une mutation plausible survivrait-elle (opération du rejeu mal
   nommée ; rejeu appliqué aussi à 1205 ; `activity_with` appelé avant `ensure_not_pat` sur un autre chemin) ?
3. **Les compteurs, recomptés à la source** : le Change Log et le Dev Agent Record de la fiche (mutations 40 = 19 + 21 ; T3 :
   12 ; rouges 18 = 4 + 12 + 1 + 1 depuis `rouges-S.log` ; tests ajoutés par la remédiation, aux deux bornes `11e5a38b` →
   tête) ; `audit_route_registry.rs` (doc de module 144 / 147 contre les assertions) ; une entrée datée passée n'a pas été
   réécrite (elle s'annote).
4. **Grep du symptôme, la valeur, sur tout le worktree** : « ne déconnecte personne », « huit heures », `jwt::decode`,
   `retry_on_deadlock(` dans `routes/session_settings.rs`, `ensure_not_pat` dans `routes/auth.rs` ; le symptôme d'un 500 sur
   interblocage ailleurs dans les réglages d'installation (`backup_schedule`, `mcp_server_settings` : même motif
   `FOR UPDATE` + `INSERT` à version 0 ?).

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne le diff `11e5a38b..ff20983d` et le code qu'il touche : la fermeture passée à `retry_on_deadlock`
(est-elle `Fn`, rejoue-t-elle une transaction neuve, aucun effet de bord hors transaction ?) ; `activity` / `activity_with`
(le type de retour `Response` : tout chemin d'erreur garde-t-il son code et son corps ? les cookies du `CookieJar` sont-ils
toujours posés ?) ; le `#[cfg(test)]` de `jwt::decode` (aucun appelant hors tests, y compris dans `tests/` d'intégration) ;
le test de course (fuite de la table `lest_rejeu`, isolation de la base éphémère).

## Lentille E — Edge (chaque branche, chaque borne)

Le rejeu : épuisé (trois interblocages) ⇒ que rend la route ? un `PUT` à version 1 concurrent d'un autre ? une valeur
inchangée pendant la course ? Le battement : clé d'API + corps vide, clé d'API sans `Content-Type`, session + corps illisible
sans cookie de renouvellement (400 ou 401 ?), session + corps valide sans cookie. Les phrases du manuel : une session ouverte
sous le contournement **depuis moins de quinze minutes** est-elle fermée ? une installation qui avait relevé
`KESH_REFRESH_INACTIVITY_MINUTES` au lieu de `KESH_JWT_EXPIRY_MINUTES` ?

## Lentille A — Acceptance (le mandat de la remédiation contre le livré)

Chaque point 1 à 6 ci-dessus : livré, testé, au lieu prescrit, avec la ligne lue citée au Change Log ? Le reclassement de
S-B1 en MEDIUM et les rejets éventuels sont-ils motivés ? Le registre C-v0172s-59 justifie-t-il le choix du rejeu contre le
mappage de 1213 ? Le Dev Agent Record n'affirme-t-il que ce qui a tourné (gates, E2E, recette : chiffres recoupés aux
journaux) ? L'issue préparée pour les LOW non corrigés (`/home/gcorbaz/devel/kesh-gate-logs/v0172s/issue-low-p1.md`)
couvre-t-elle chacun d'eux, sans donnée réelle ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/v0172s-review-p2-S-B.md`, `…-S-E.md`, `…-S-A.md`.
Chaque rapport : findings numérotés (`S2-B1`, `S2-E1`, `S2-A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu
(`fichier:ligne`), ligne lue copiée, preuve, remède proposé ; **la liste des axes exercés ET non exercés** (une passe qui ne
la donne pas ne compte pas) ; le décompte final **recompté depuis la liste**.
