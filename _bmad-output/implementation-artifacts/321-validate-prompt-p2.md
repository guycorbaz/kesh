# Prompt — validation P2, Story 32-1 (défauts courts : 32-1a, 32-1b, 32-1c)

*Versionné le 2026-10-24. Passe **complète** (non ciblée), **deux lentilles, R et F**, chacune en contexte frais,
**Opus**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-1`, branche `story/32-1`. Fiche :
`_bmad-output/implementation-artifacts/32-1-defauts-courts.md` (trois parties A, B, C, une par clé) ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-321.md` (C-32-1-1 à C-32-1-28). Cahier des charges :
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` § *32-1*. Règles : `CLAUDE.md` (§ *Review Iteration Rule*,
*La passe ciblée* — une boucle ne se clôt que sur une passe qui ne rend **aucun** MEDIUM+ —, *Règle de splitting
préventif*, *Exception kesh-db*, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON RÉSOLUS*,
*Recette visuelle*, *Propagation post-patch*), `docs/agents-orchestration.md` § 3. Lis les issues et leurs commentaires
en lecture seule : `gh issue view 711 --json title,body,comments`, idem **714**, **716**, et les issues ouvertes par
l'orchestrateur **725** (E-1, E-2 de #711), **726** (gestes de #714 pour Guy), **710** (refus de validation en français).
Relevés d'origine, hors dépôt, en lecture : `/home/gcorbaz/devel/kesh-gate-logs/311c-review-p1-E.md`,
`315-low-residuels.md`, `314-review-p4.md`. Rapports de la passe précédente, à lire **après** ta propre lecture de la
fiche (pour ne pas t'y ancrer) : `/home/gcorbaz/devel/kesh-gate-logs/321-validate-p1-R.md`, `321-validate-p1-F.md`.

**Ce que la fiche prescrit au développement n'existe pas encore : son absence dans le code n'est pas un défaut.** Ce qui
se juge, c'est la vérité de ce que la fiche affirme du code **actuel**, et la faisabilité de ce qu'elle prescrit.

## Attention — la remédiation de la P1

La P1 a rendu 4 MEDIUM distincts et 14 LOW ; tous ont été remédiés dans la fiche et le registre (Change Log,
« Validation P1 » ; choix C-32-1-20 à C-32-1-28 ; amendements datés de C-32-1-2, 4, 6, 7, 9, 11, 14, 15, 16, 17). **La
sévérité se déplace vers ce qu'on vient d'écrire** : relis d'abord, et avec le plus de soin, les passages réécrits —
**AC-a4** (trois colonnes, montage transactionnel, M-a1a/b/c), **AC-a8** (séquence des verrous), **AC-a9** (sonde
`free_bytes` seule), **AC-a11** (balayeur à états, assertions (1) à (5), M-a7), **AC-b2** (trois cas, prédicat
d'`issueInconnue` extrait, M-b8, `loadError`), **AC-b4** (`enabled`, fixtures), **AC-b5** (garde existante, M-b7),
**AC-b7** (phrases des l. 2347, 2349), **AC-c7** (26 occurrences, U+FE0F, `\keshAttention\ `), **AC-c9** (brochure),
Dev Notes § *Pourquoi trois clés* (fichiers communs, `closes #711`) et § *Contrôle aplati d'un PDF*, *Inventaire*
(décompte multiligne, copies de l'assistant lexical), *Fichiers touchés* (CHANGELOG). Pour chacun : est-il vrai au code,
faisable, cohérent avec le reste de la fiche, et a-t-il laissé un résidu ailleurs (même symptôme écrit autrement) ?

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie** dans
ton rapport. Un LOW se vérifie comme un HIGH : un site a pu bouger. Axes, tous à exercer :

1. **Tableaux « Constats établis au code ».** Chaque ligne (8 de #716, 12 de #711, 14 de #714) : le site dit-il ce que
   la fiche en dit ? Classement juste (32-1a / 32-1b / 32-1c / hors, avec #725 et #726) ? Prémisses corrigées
   (C-32-1-19) justes ? La ligne E7 réécrite (séquence de dix comptes au plan PME) est-elle exacte — ordre du code,
   numéros de `crates/kesh-core/assets/charts/pme.json`, lignes 443-493, 501-521, 512, 1118 de
   `crates/kesh-db/src/repositories/company_invoice_settings.rs` ?
2. **32-1a, tests.** Chaque test prescrit rougit-il **avant** le correctif pour la bonne raison, ou, quand la fiche dit
   « vert avant », rougit-il sous la mutation qu'elle nomme ? Chaque mutation (M-a1a, M-a1b, M-a1c, M-a2 à M-a7) est-elle
   tuée par le test nommé, ou équivalente ? En particulier : le montage de l'AC-a4 (t1 commitée, `UPDATE` sur le pool,
   t2, t3) est-il sans attente de verrou, et les comptes 2000, 1170, 2201 sont-ils imputables et du type de
   `VAT_SETTING_ACCOUNT_TYPES` ? Le balayeur de l'AC-a11 : les cinq fichiers lus par `lancement_par_main_seulement` et
   `gardes_possedes` (`main.rs`, `lib.rs`, `routes/admin.rs`, `routes/backup_schedule.rs`, `auto_backup.rs`, partie
   avant `#[cfg(test)]`) contiennent-ils une forme qu'il ne couvre pas (chaîne d'octets `b"…"`, `br#"…"#`, littéral `'\''`,
   `'"'`, `'\u{…}'`, label `'a:`) ? Les assertions (1) à (5) rendent-elles aujourd'hui ce que la fiche dit ? Un balayeur
   conforme garde-t-il vertes `lancement_par_main_seulement` et `gardes_possedes` ?
3. **32-1b, comportement.** C-32-1-8 (`record_*` ne touche jamais `version`) ; le classement en trois cas de l'AC-b2
   contre `frontend/src/lib/shared/utils/api-client.ts` (formes réelles des erreurs : `status`, `code`, retries) et contre
   `AdminRestorePanel.svelte` (l'extraction du prédicat ne change-t-elle rien à l'import ?) ; le 503, réessayé par
   `api-client` pour les méthodes idempotentes, l'est-il pour le `POST …/run` ? `loadError` posé par la relecture : que
   voit l'administrateur, et un `loadSchedule` ultérieur l'efface-t-il ? AC-b3 (séquence, deux sondages concurrents) ;
   AC-b4 (`enabled &&`, doctrine « le navigateur ne compare aucune date ») ; AC-b6 (faisabilité dans `i18n_e2e.rs`) ;
   AC-b5 et M-b7 contre `frontend/src/lib/shared/i18n-entrees-a-variables.test.ts` (les maillons 2 et 3 voient-ils
   réellement la clé `admin-backup-server-too-large` et son site ?).
4. **32-1c, phrases contre gestes.** Chaque AC-c1 à AC-c8 est-il **une phrase** (aucune commande changée, aucune
   consigne neuve d'agir — C-32-1-15) ? Les faits invoqués sont-ils vrais ? AC-c7 : les 26 occurrences de `⚠️` se
   recomptent-elles (`grep -o $'\xe2\x9a\xa0\xef\xb8\x8f' docs/manual/fr/*.tex | wc -l`), aucune n'est-elle dans un
   argument de macro ou un environnement où `\keshAttention\ ` casserait (titre de section, `\textbf{…}`, `tcolorbox`
   `title=`) ? La phrase témoin « ! Un dossier = une instance » sortira-t-elle du PDF aplati ?
5. **Manuel — `.tex` ET PDF.** Les lignes citées de `docs/manual/fr/admin-manual.tex` (1625, 1635, 1686, 1704-1711,
   1714-1726, 1772, 1788, 1791, 1844, 1851-1852, 1880-1918, 2345-2357, 2923) et de `user-manual.tex` (3262) disent-elles
   ce que la fiche prétend ? Contrôle le **PDF aplati** avec la commande de la fiche (Dev Notes, § *Contrôle aplati d'un
   PDF*), apostrophe typographique normalisée :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | sed "s/’/'/g" | tr -s ' ' | grep -oF "<phrase>"` — témoin
   positif d'abord : « trop grande pour être restaurée par l'écran » doit sortir ; puis « ne fait que lire la version »,
   « Aucune sauvegarde automatique réussie depuis le », « ce seuil protège aussi la base » ; et
   `pdftotext docs/manual/fr/admin-manual.pdf - | grep -c "⚠"` (attendu 0). Les retouches prescrites (AC-b7, AC-c1 à
   AC-c5) seront-elles **vraies après** le correctif, et aucune phrase voisine ne deviendra-t-elle fausse ? Le manuel
   utilisateur, le README, `website/` ou le CHANGELOG disent-ils quelque chose de contraire ?
6. **i18n.** Clés neuves et modifiée de l'AC-b8 contre les gardes (parité, G9, `lint-i18n-ownership`,
   `i18n-entrees-a-variables.test.ts`, `i18n-repli-divergent-actif.test.ts`) ; un test existant asserte-t-il un texte qui
   changera (dont le message d'échec actuel de « Sauvegarder maintenant », désormais réservé aux 4xx et 5xx) sans que la
   fiche le nomme ?
7. **Inventaire des sites non résolus.** Est-il clos ? Recompte les deux décomptes de la classe #710 avec les commandes
   de la fiche (145 et 101 annoncés) et dis si leur périmètre est le bon ; les copies de l'assistant lexical (cinq, plus
   `audit_label_registry.rs`) sont-elles toutes là (`grep -rn '"#\[cfg(test)\]"' crates`) ? D'autres sites qui affichent
   une date future, d'autres `⚠️` servis à l'utilisateur ?

## Lentille F — faisabilité et fausses prémisses

1. **Prémisses** : chaque constat retenu est-il établi au code ou supposé ? Une prémisse corrigée à tort ? E-1 et E-2
   (#725) sont-ils vraiment non mécaniques ? Une correction retenue comme mécanique cache-t-elle une conception — en
   particulier le classement en trois cas de l'AC-b2 et l'extraction du prédicat ?
2. **Découpage** : modules annoncés (A : 3 ; B : 4 ; C : 1) contre *Fichiers touchés* (qui ajoute désormais
   `admin-restore/` et un module partagé à la 32-1b, la brochure à la 32-1c) ; une clé dépasse-t-elle cinq modules ?
   L'ordre de fusion et la règle « `closes #711` sur la dernière fusionnée » (C-32-1-27) tiennent-ils dans tous les
   ordres possibles ? Deux clés touchent-elles les mêmes lignes ?
3. **Faisabilité** : sonde `free_bytes` dans `TestEnv` sans changer `BackupEnv` ; balayeur dans `mod tests` sans
   dépendance ; tests de dépôt de l'AC-a4 avec les helpers existants (`pme_company_with_chart`, `account_with_role`) et
   une recherche par numéro ; extraction du prédicat sans changer `AdminRestorePanel` ; E2E de l'AC-b10 compatible avec le
   409 de la tâche de fond ; création du bloc `## [0.16.0] — Non publié` compatible avec `scripts/prepare-release.sh`
   (l. 237-255, en **lecture** seulement) et avec d'autres stories de l'Epic 32 qui l'auraient créé.
4. **Décomptes** : recompte chaque nombre à sa source — `[Cas; 10]` → 12 ; `TABLE: [(&str, Cles); 4]` → 5 ; 26 `⚠️`
   (17 + 9, 16 + 9 lignes) ; 2 `I18nBundle::load` ; neuf mutations en A (M-a1a, M-a1b, M-a1c, M-a2 à M-a7), huit en B
   (M-b1 à M-b8) ; « −3 + 2 » issues, « −2 + 1 » bugs (étiquettes réelles de #711, #714, #716, #725, #726) ; 18 constats
   distincts et 4 MEDIUM au Change Log de la P1 (recompte depuis les deux rapports) ; signale celui qui ne se retrouve pas.
5. **Cohérence interne** : AC-a1 à AC-a13, AC-b1 à AC-b11, AC-c1 à AC-c10, tâches, registre (C-32-1-1 à 28 et
   amendements : une décision amendée contredit-elle encore un passage de la fiche ?), *Issues ouvertes*, *Inventaire*,
   Change Log — contradiction, critère invérifiable, commande mal formée (essaie mentalement chaque `grep`, `sed`,
   `pdftotext` contre le dépôt actuel) ? Aucune donnée réelle ?
6. **Ce que la fiche omet** : un test existant qui rougira sans être nommé ; une conséquence pour la recette visuelle
   (captures des AC-b9, AC-c6, AC-c7) ; le `.keshbackup` (aucune table ne change : vérifie) ; une phrase du manuel que les
   correctifs de la 32-1b ou de la 32-1c rendraient fausse sans être nommée.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, partie de la fiche (A, B, C, Dev Notes, registre) et
ligne, site du dépôt, preuve (commande et sortie copiée), correction proposée ; pour chacun, dis s'il vient de la
**remédiation P1** ou de la rédaction d'origine. ⛔ **La liste des axes exercés ET non exercés** (R1 à R7, F1 à F6), avec
pour chacun ce qui a été lu — sans elle, « 0 finding » ne compte pas.

Rapport écrit **d'un seul jet**, un seul fichier par lentille, **par l'outil d'écriture de fichier** (jamais par une
redirection du shell) : `/home/gcorbaz/devel/kesh-gate-logs/321-validate-p2-R.md` pour la lentille R,
`/home/gcorbaz/devel/kesh-gate-logs/321-validate-p2-F.md` pour la lentille F. Dernier message : chemin, bilan par
sévérité (recompté sur ta liste), une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris **aucun fichier** hors de ton rapport ; **aucun fichier temporaire** ; **aucune redirection** du shell vers un
fichier (`>`, `>>`, `tee`, `-o`). Aucun `git checkout`/`switch` sous aucune forme. **Aucune commande qui écrit, compile ou
exécute** : `scripts/*` (dont **`scripts/prepare-release.sh`**, qui bumpe les versions du workspace — à **lire**
seulement), `make`, `latexmk`, `lualatex`, `xelatex`, `cargo`, `rustup`, `npm`, `npx`, `node`, `docker`, `curl`,
**`python3`, `python`, `perl`, `ruby`**, `mariadb`/`mysql`, `sqlx`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture (`pr`, `issue create|edit|close|comment`).
`pdftotext` vers la sortie standard seulement (`-` comme destination). `sed` sans `-i`, en filtre seulement. Aucune
donnée réelle de Guy (rien de `~/kesh-test/`). Autorisés : lecture de fichiers, `grep`, `sed -n` et `sed` en filtre de
tube, `awk` en lecture, `wc`, `tr`, `git log|show|diff|blame`, `gh issue view`, `pdftotext … -`.
