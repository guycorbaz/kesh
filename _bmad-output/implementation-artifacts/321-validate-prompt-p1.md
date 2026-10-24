# Prompt — validation P1, Story 32-1 (défauts courts : 32-1a, 32-1b, 32-1c)

*Versionné le 2026-10-24. **Deux lentilles, R et F**, chacune en contexte frais, **Sonnet**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-1`, branche `story/32-1`. Fiche :
`_bmad-output/implementation-artifacts/32-1-defauts-courts.md` (trois parties A, B, C, une par clé) ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-321.md` (C-32-1-1 à C-32-1-19). Cahier des charges :
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` § *32-1*. Règles : `CLAUDE.md` (§ *Review Iteration Rule*,
*La passe ciblée* — une boucle ne se clôt que sur 0 MEDIUM+ —, *Règle de splitting préventif*, *Exception kesh-db*,
*Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON RÉSOLUS*, *Recette visuelle*),
`docs/agents-orchestration.md` § 3. Lis les trois issues et leurs commentaires (`gh issue view 711 --json
title,body,comments`, idem 714 et 716 — lecture seule ; pour #714, le commentaire de Guy du 2026-10-24 tranche les deux
choix de procédure). Les relevés d'origine des constats sont hors dépôt, en lecture :
`/home/gcorbaz/devel/kesh-gate-logs/311c-review-p1-E.md`, `315-low-residuels.md`, `314-review-p4.md`.

**Ce que la fiche prescrit au développement n'existe pas encore : son absence dans le code n'est pas un défaut.** Ce qui
se juge, c'est la vérité de ce que la fiche affirme du code **actuel**, et la faisabilité de ce qu'elle prescrit.

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie** dans
ton rapport. Un LOW se vérifie comme un HIGH : un site a pu bouger. Axes, tous à exercer :

1. **Tableaux « Constats établis au code ».** Pour chacune des 8 lignes de #716, des 12 de #711 et des 14 de #714 :
   le site cité (fichier, lignes) dit-il ce que la fiche en dit ? Le classement (32-1a / 32-1b / 32-1c / hors) est-il
   juste ? Les **prémisses corrigées** (C-32-1-19 : E-7 l. 95-100 et non 111 ; L-B3 l. 496-500 ; E-9 (1) ; #714 « bloque
   une reprise » ; E6 l. 277-278 et 296) le sont-elles à raison ?
2. **32-1a, tests.** Chaque test prescrit (AC-a3 à AC-a7, AC-a9, AC-a11) rougit-il **avant** le correctif pour la bonne
   raison, et non par un montage ? Chaque mutation M-a1 à M-a6 est-elle tuée par le test nommé, ou serait-elle
   équivalente ? En particulier : C-32-1-2 (la M4 de la 31-5 reste équivalente ; c'est la mutation du `COALESCE` qui se
   tue) est-il juste au code (`crates/kesh-seed/src/lib.rs` autour de la l. 400, `company_invoice_settings.rs:1314-1334`) ?
   L'AC-a4 pose-t-il une valeur que la route accepterait (passif imputable) ? C-32-1-4 (création / lecture du dossier :
   équivalentes quant au statut) est-il vrai pour **tous** les cas d'échec de `DirBuilder::create` ? La M-a3 tue-t-elle
   vraiment par la sonde (que fait `ecrire` si les deux `return` sont neutralisés : `supprimer_les_partiels`,
   `taille_des_documents`, `free_bytes`, `write_backup_file` sur un `auto` qui est un fichier) ? Les exemples de l'AC-a7
   et de l'AC-a11 rendent-ils aujourd'hui ce que la fiche dit (lis `nombres_isoles`, `code`, `argument_de`, `corps`) ?
3. **32-1b, comportement.** `record_*` ne touche vraiment jamais `version` (C-32-1-8) ? La relecture de l'AC-b2 n'appelle
   pas `loadSchedule` — est-ce juste et suffisant ? Le motif de séquence de l'AC-b3 : les deux cas (i) et (ii) sont-ils
   tenus par la conception décrite, y compris deux sondages concurrents ? L'AC-b4 respecte-t-il la doctrine « le
   navigateur ne compare aucune date à son horloge » ? `is_success_in_future` et `is_overdue` peuvent-ils diverger ? Le
   test de l'AC-b6 est-il faisable dans `crates/kesh-api/tests/i18n_e2e.rs` (montage `spawn_app_with_locale`, connexion
   administrateur, garde `require_not_pat`, verrou `get_i18n_lock`) ? La table de G9 accepte-t-elle l'entrée prescrite
   (forme de `TABLE`, `textes_coherents.rs:694`) ?
4. **32-1c, phrases contre gestes.** Pour chaque AC-c1 à AC-c8, la correction prescrite est-elle bien **une phrase**
   (aucune commande changée, aucune consigne neuve d'agir), selon la règle de C-32-1-15 ? Une ligne rangée dans l'issue
   pour Guy serait-elle en fait une simple phrase, ou l'inverse ? Les faits invoqués sont-ils vrais : l'export de
   souveraineté crée-t-il les lignes de réglages (`crates/kesh-api/src/exports/global.rs:304`, `:313`) et la page des
   relances pose-t-elle en plus les niveaux (`company_dunning_settings.rs:168-200`) ? `realpath -e` retire-t-il une barre
   finale ? `grep` sur un fichier absent écrit-il sur la sortie d'erreur ? Le `\needspace` de l'AC-c6 est-il au bon endroit ?
   Les 16 et 9 occurrences de ⚠️ se recomptent-elles (`grep -c "⚠" docs/manual/fr/*.tex`) ?
5. **Manuel — `.tex` ET PDF.** Les lignes citées de `docs/manual/fr/admin-manual.tex` (1625, 1635, 1686, 1704-1710,
   1714-1726, 1772, 1788, 1791, 1844, 1851-1852, 1880-1918, 2345-2357) et de `user-manual.tex` (3262) disent-elles ce que
   la fiche prétend ? Contrôle le **PDF aplati** :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF "ne fait que lire la version"`, idem
   « ne mesure », « Aucune sauvegarde automatique réussie depuis le », « trop grande pour être restaurée par l'écran »,
   et `pdftotext docs/manual/fr/admin-manual.pdf - | grep -c "⚠"` (attendu 0). Les retouches prescrites (AC-b7, AC-c1 à
   AC-c5) seront-elles **vraies après** le correctif ? Le manuel utilisateur ou le CHANGELOG disent-ils quelque chose de
   contraire (bandeau, sauvegarde automatique, communication de la QR) ?
6. **i18n.** Les clés neuves et la clé modifiée de l'AC-b8 entrent-elles en conflit avec une garde (parité des locales,
   G9, `lint-i18n-ownership`, propriété du préfixe `admin-backup-`) ? Un test existant asserte-t-il un texte qui changera
   (le français de L-B7, « limite N Mio », « Aucune sauvegarde automatique réussie depuis le ») sans que la fiche le
   nomme ?
7. **Inventaire des sites non résolus.** Est-il clos ? Cherche en particulier les autres `AppError::Validation("…")` en
   français dans `crates/kesh-api/src/routes/` (le compte et l'issue à demander), d'autres sites qui affichent une date
   future au bandeau ou au panneau, et d'autres ⚠️ servis à l'utilisateur.

## Lentille F — faisabilité et fausses prémisses

1. **Prémisses** : chaque constat retenu est-il **établi** au code ou supposé ? Une prémisse que la fiche corrige à tort ?
   E-1 et E-2 sont-ils vraiment non mécaniques (une correction de quelques lignes existerait-elle) ? Inversement, une
   correction retenue comme mécanique cache-t-elle une conception (E-8, L-B5) ?
2. **Découpage** : les modules annoncés (A : 3 ; B : 4 ; C : 1) se recomptent-ils sur la liste *Fichiers touchés* ? Une
   clé dépasse-t-elle cinq modules ? Les trois clés sont-elles indépendantes (fichiers communs : `admin-manual.tex` et son
   PDF, `auto_backup.rs` — partie A dans `mod tests`, partie B hors `mod tests`, `admin_auto_backup_e2e.rs`,
   `textes_coherents.rs`) ? L'ordre de fusion de C-32-1-17 suffit-il, ou deux clés touchent-elles les mêmes lignes ?
3. **Faisabilité** : `TestEnv` peut-il porter des compteurs sans changer le trait `BackupEnv` ? `catalogues_charges()`
   peut-elle servir `spawn_app_de` (ordre de déclaration, signature) ? Le test de dépôt de l'AC-a4 a-t-il les helpers qu'il
   suppose (`pme_company_with_chart`, `account_with_role`, l. 1635, 1665) ? L'E2E de l'AC-b10 est-il compatible avec le
   409 de la tâche de fond (l. 159-170 de la spec) ?
4. **Décomptes** : recompte chaque nombre de la fiche à sa source (dix cas du test du journal d'audit, `[Cas; 10]` ;
   `TABLE: [(&str, Cles); 4]` ; 16 et 9 ⚠️ ; 2 `I18nBundle::load` dans `audit_log_e2e.rs` ; 142 issues au kickoff — chiffre
   du fichier d'epic ; « −3 + 2 ») ; signale celui qui ne se retrouve pas.
5. **Cohérence interne** : AC-a1 à AC-a13, AC-b1 à AC-b11, AC-c1 à AC-c10, tâches, registre, *Issues à ouvrir*,
   *Inventaire* — contradiction, critère invérifiable, commande de contrôle mal formée (essaie mentalement chaque `grep`
   et chaque `pdftotext` contre le dépôt actuel) ? Les deux textes d'issue sont-ils justes et autonomes (lisibles sans la
   fiche), sans aucune donnée réelle ?
6. **Ce que la fiche omet** : un test existant qui rougira sans être nommé ; une conséquence pour la recette visuelle ;
   le CHANGELOG ; le `.keshbackup` (aucune table ne change : vérifie) ; une phrase du manuel que les correctifs de la 32-1b
   rendraient fausse sans que l'AC-b7 la nomme (cherche « bandeau », « Sauvegarder maintenant », « trop grande »,
   « horloge » dans `admin-manual.tex`).

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, partie de la fiche (A, B, C) et ligne, site du dépôt,
preuve (commande et sortie copiée), correction proposée. ⛔ **La liste des axes exercés ET non exercés** (R1 à R7, F1 à
F6), avec pour chacun ce qui a été lu — sans elle, « 0 finding » ne compte pas.

Rapport écrit **d'un seul jet**, un seul fichier par lentille, **par l'outil d'écriture de fichier** (jamais par une
redirection du shell) : `/home/gcorbaz/devel/kesh-gate-logs/321-validate-p1-R.md` pour la lentille R,
`/home/gcorbaz/devel/kesh-gate-logs/321-validate-p1-F.md` pour la lentille F. Dernier message : chemin, bilan par
sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris **aucun fichier** hors de ton rapport ; **aucun fichier temporaire** ; **aucune redirection** du shell vers un
fichier (`>`, `>>`, `tee`, `-o`). Aucun `git checkout`/`switch` sous aucune forme. **Aucune commande qui écrit, compile ou
exécute** : `scripts/*` (dont **`scripts/prepare-release.sh`**, qui bumpe les versions du workspace), `make`, `latexmk`,
`lualatex`, `xelatex`, `cargo`, `rustup`, `npm`, `npx`, `node`, `docker`, `curl`, **`python3`, `python`, `perl`, `ruby`**,
`mariadb`/`mysql`, `sqlx`, `sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture (`pr`,
`issue create|edit|close|comment`). `pdftotext` vers la sortie standard seulement (`-` comme destination). Aucune donnée
réelle de Guy (rien de `~/kesh-test/`). Autorisés : lecture de fichiers, `grep`, `sed -n`, `awk` en lecture, `wc`,
`git log|show|diff|blame`, `gh issue view`, `pdftotext … -`.
