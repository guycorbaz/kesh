# Prompt — validation P3, Story 32-1 (défauts courts : 32-1a, 32-1b, 32-1c, 32-1d)

*Versionné le 2026-10-24. Passe **complète** (non ciblée), **deux lentilles, R et F**, chacune en contexte frais,
**Sonnet**. Lecture seule stricte. **Amendé le 2026-10-24**, après le découpage de la 32-1b en écran (32-1b) et
serveur (32-1d), pour que la passe vérifie aussi ce découpage (§ *Attention — le découpage de la 32-1b*).*

Dépôt `/home/gcorbaz/devel/kesh-32-1`, branche `story/32-1`. Fiche :
`_bmad-output/implementation-artifacts/32-1-defauts-courts.md` (quatre parties A, B, C, D, une par clé — la D, 32-1d,
née du découpage de la 32-1b) ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-321.md` (C-32-1-1 à C-32-1-37). Cahier des charges :
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` § *32-1*. Règles : `CLAUDE.md` (§ *Review Iteration Rule*,
*La passe ciblée* — une boucle ne se clôt que sur une passe qui ne rend **aucun** MEDIUM+ —, *Règle de splitting
préventif*, *Exception kesh-db*, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON RÉSOLUS*,
*Recette visuelle*, *Propagation post-patch*, règle DRY de *Code Quality Rules*), `docs/agents-orchestration.md` § 3.
Lis les issues et leurs commentaires en lecture seule : `gh issue view 711 --json title,body,comments`, idem **714**,
**716**, et les issues ouvertes par l'orchestrateur **725** (E-1, E-2 de #711), **726** (gestes de #714 pour Guy),
**727** (copies de l'assistant lexical), **710** (refus de validation en français). Relevés d'origine, hors dépôt, en
lecture : `/home/gcorbaz/devel/kesh-gate-logs/311c-review-p1-E.md`, `315-low-residuels.md`, `314-review-p4.md`.
Rapports des passes précédentes, à lire **après** ta propre lecture de la fiche (pour ne pas t'y ancrer) :
`/home/gcorbaz/devel/kesh-gate-logs/321-validate-p1-{R,F}.md`, `321-validate-p2-{R,F}.md`.

**Ce que la fiche prescrit au développement n'existe pas encore : son absence dans le code n'est pas un défaut.** Ce qui
se juge, c'est la vérité de ce que la fiche affirme du code **actuel**, et la faisabilité de ce qu'elle prescrit.

## Attention — le découpage de la 32-1b

Après la remédiation P2, l'orchestrateur a **découpé** la 32-1b (règle de splitting préventif, amendement D5 : 7 modules
au grain métier, et MEDIUM → MEDIUM avec un recyclage en P2 ; C-32-1-37, Change Log « Découpage de la 32-1b ») :
**32-1b « écran »** (partie B : E-6, E-7, E-8, L-B6, le bandeau de L-B5, le manuel entier, L-B3 compris) et **32-1d
« serveur »** (partie D : L-B5 drapeau servi, L-B7). C'est une écriture **neuve** : la sévérité s'y déplace. Vérifie,
dans les deux lentilles :

- **Aucune AC entre les deux** : chaque élément de l'ancien AC-b4 (fonction, champ servi, test `prochaine_et_retard`,
  type frontend, fixtures, bandeau, clé), de l'ancien AC-b6, de l'AC-b8 (clés réparties : deux neuves et une modifiée
  pour la B, une pour la D), de l'AC-b9 (captures (a)–(d) à la B, (e) à la D, AC-d4) et des mutations (M-b5 → M-d1,
  M-b6 → M-d2, M-b11 neuve) est-il attribué une fois et une seule ? Un renvoi (AC-b6, T-b1, M-b5, M-b6) pointe-t-il
  juste ? Lis `git show e312b425:_bmad-output/implementation-artifacts/32-1-defauts-courts.md` (la fiche d'avant le
  découpage) et compare.
- **Au code** : la 32-1d est-elle réellement sans frontend — un champ de plus dans `BackupScheduleResponse` laisse-t-il
  vert `npm run check` et chaque test frontend actuel (un test compare-t-il la réponse entière, par exemple par
  `toEqual` sur un objet servi) ? Le refus traduit a-t-il une phrase au manuel ou un texte assert ailleurs
  (`grep -rn 'hors domaine' docs frontend/src crates`) ? La 32-1b est-elle réellement sans fichier Rust ? La phrase
  L-B3 de l'AC-b7 ne décrit-elle qu'un comportement existant (`seuil_d_espace`, `auto_backup.rs:496-500`) ?
- **Comptes** : 5 modules (B) et 3 (D) au grain métier, 3 et 2 au grain des crates et paquets, contre *Fichiers touchés* ;
  neuf mutations en B (M-b7 en deux formes), deux en D ; **+2** et **+1** clés `admin-backup-` par locale, et les bornes
  de leur recompte.
- **Ordre de fusion et mots-clés** (Dev Notes, § *Pourquoi quatre clés*) : 32-1c → 32-1b et 32-1d → 32-1b ; fichiers de
  code communs 32-1a/32-1d (trois, hunks distincts) et catalogues communs 32-1b/32-1d ; 32-1a et 32-1b sans fichier
  commun (vérifie). `closes #711` sur la dernière fusionnée des 32-1a, 32-1b, 32-1d — la 32-1d toujours `refs` : tient-il
  dans **tous** les ordres permis ? Le CHANGELOG (bloc créé par la première des 32-1c et 32-1d ; une ligne pour la D) ;
  le bilan d'issues inchangé. L'E2E final et la capture (c) de la 32-1b après la fusion de la 32-1d : exigés partout où
  il le faut (AC-b9, AC-b10, T-b2, T-b7) ?
- **Registre et `sprint-status.yaml`** : C-32-1-37 et les amendements datés de C-32-1-17, 18, 27, 28, 36 contredisent-ils
  un passage de la fiche ? La clé `32-1d-defauts-sauvegarde-serveur` et les commentaires datés disent-ils la même chose
  que la fiche ? Le motif du découpage (recyclage R1 de la P2, au sens de l'amendement D5) est-il juste au regard des
  rapports P2 ?

## Attention — la remédiation de la P2

La P2 a rendu 6 MEDIUM distincts et 12 LOW ; tous ont été remédiés dans la fiche et le registre (Change Log,
« Validation P2 » ; choix C-32-1-29 à C-32-1-36 ; amendements datés de C-32-1-8, 16, 20, 26). La remédiation a aussi
**changé de conception** sur un point qu'aucune lentille n'avait relevé : l'AC-a11 ne prescrit plus un balayeur neuf,
elle **réemploie** `decouper` de `crates/kesh-db/tests/lexique/mod.rs`, étendu aux commentaires imbriqués (C-32-1-32).
**La sévérité se déplace vers ce qu'on vient d'écrire** : relis d'abord, et avec le plus de soin :

- **AC-a11** (troisième rédaction) : l'inclusion `#[cfg(test)] #[path = "../../kesh-db/tests/lexique/mod.rs"] mod lexique;`
  en tête d'`auto_backup.rs` se résout-elle (règle des chemins de `#[path]` hors module en ligne, fichier non `mod.rs`) ?
  `decouper` couvre-t-il réellement les formes des cinq fichiers lus (lis-le en entier : chaînes, brutes, `//`, `/* */`,
  `'x'`, `'\x'`, durées de vie) ? Les assertions (1) à (6), dont (3'), rendent-elles **aujourd'hui** ce que la fiche
  dit (rouges avant ou vertes avant), et sous **M-a7a** (bras `b'"'` retiré) et **M-a7b** (retrait non gourmand) ce
  qu'elle annonce ? `code()` rendant désormais une copie **masquée** (contenu des chaînes effacé), une assertion de
  `lancement_par_main_seulement` ou de `gardes_possedes` cherche-t-elle un jeton **dans une chaîne** ? L'extension aux
  blocs imbriqués laisse-t-elle vertes `letterings_lexical.rs` et `bank_account_closed_inventory.rs` (lis ce qu'ils
  assertent) ? `argument_de` sur la copie masquée rend-il la tranche de la source aux mêmes positions (l'effacement
  garde-t-il la longueur en **octets** d'un caractère multioctet) ?
- **AC-b1** (`formBase`, `save` qui envoie `formBase.version`, Vitest (iii), M-b9) : le 409 joue-t-il dans le cas
  concurrent, et seulement là ? Le cas normal (même `version`) et le « Recharger » existant (`loadSchedule` → `apply`)
  restent-ils justes ? Une saisie revenue à la main aux valeurs de `formBase` ?
- **AC-b2** (non-`ApiError` → issue inconnue, message « … ou s'écrit encore … », `loadError` remis à `null` au départ
  de `runNow`, M-b10) : le message neuf est-il vrai dans **chacun** des cas qu'il couvre (TIMEOUT, NETWORK_ERROR, 502,
  504, 200 au corps illisible) ? Le prédicat extrait reste-t-il inchangé pour `AdminRestorePanel` ?
- **AC-b7** (phrase de l'espace disque, phrase « Ce que l'écran montre ») contre `seuil_d_espace`
  (`auto_backup.rs:496-500`), `:680-683` et `record_attempt`/`record_failure` ; **T-b6** (`make admin`).
- **AC-c7** (replis `\faCheckCircle` → `\textbf{+}`, `\faExchangeAlt` → `\textbf{→}`, `literate` des listings, contrôle
  U+FFFD 26/9/16 → 0) et **AC-c9** (brochure comparée à `git merge-base origin/main HEAD`, hors ses 16 signes) : les
  signes de repli existent-ils dans TeX Gyre Heros et TeX Gyre Cursor ? Le `literate` touche-t-il un listing de
  `sec:repetition-cloture` ? La commande de comparaison de la brochure est-elle bien formée et discriminante ?
- *Origine* (bilan **−3 + 3 = 0** issue, **−1** bug), *Inventaire* (copies de l'assistant lexical sur tout `crates/`,
  `lexique` nommé, signes sans glyphe, « tentative en cours »), *Commentaires à verser* (#725, #727), *Pourquoi quatre
  clés* (sous-sections du Dev Agent Record, quatre depuis le découpage), *Fichiers touchés* (CHANGELOG), lignes *Modules*
  de la 32-1b (3 / 5) et de la 32-1d (2 / 3) — voir aussi *Attention — le découpage de la 32-1b*.

Pour chacun : est-il vrai au code, faisable, cohérent avec le reste de la fiche, et a-t-il laissé un résidu ailleurs
(même symptôme écrit autrement — dans la fiche, le registre, les tâches, les Dev Notes) ?

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie** dans
ton rapport. Un LOW se vérifie comme un HIGH : un site a pu bouger. Axes, tous à exercer :

1. **Tableaux « Constats établis au code ».** Chaque ligne (8 de #716, 12 de #711, 14 de #714) : le site dit-il ce que
   la fiche en dit ? Classement juste (32-1a / 32-1b / 32-1c / 32-1d / hors, avec #725, #726, #727 ; L-B5 partagé
   entre 32-1d et 32-1b, L-B7 à la 32-1d) ? Prémisses corrigées
   (C-32-1-19) justes ? La ligne P2 B2-L5 réécrite (U+FFFD, ventilation, lignes 1547, 3313, 3318, 3320, `kesh-style.sty:39`
   et `:42`) et la ligne P2 B2-L4 nuancée sont-elles exactes ?
2. **32-1a, tests.** Chaque test prescrit rougit-il **avant** le correctif pour la bonne raison, ou, quand la fiche dit
   « vert avant », rougit-il sous la mutation qu'elle nomme ? Chaque mutation (M-a1a, M-a1b, M-a1c, M-a2 à M-a6, M-a7a,
   M-a7b) est-elle tuée par le test nommé, ou équivalente ? AC-a4 : l'assertion de montage « trois `NULL` après (t1) »
   est-elle vraie au code (`insert_with_defaults_in_tx`) ? AC-a11 : voir *Attention*.
3. **32-1b, comportement.** `formBase` contre `AutoBackupSection.svelte` entier (script et gabarit) ; le classement de
   l'AC-b2 contre `frontend/src/lib/shared/utils/api-client.ts` (formes réelles des erreurs, `request()` sur un corps
   illisible, retries d'un `POST`) et `AdminRestorePanel.svelte` ; AC-b3 (quatre commentaires nommés, séquence) ; AC-b4
   (bandeau, type, fixtures) et M-b11 ; AC-b5 et M-b7.
3-bis. **32-1d, serveur.** AC-d1 contre `auto_backup.rs` (`passe()`, `is_overdue`, `mod tests`) et
   `routes/backup_schedule.rs` (`BackupScheduleResponse`, `new`) ; `prochaine_et_retard` ; AC-d2 contre
   `update_backup_schedule`, `errors.rs` (`t()`), la table de G9 et `i18n_e2e.rs` (`spawn_app_with_locale`,
   `get_i18n_lock`) ; M-d1, M-d2 ; AC-d4 (la capture du refus est-elle provocable depuis l'écran ?) ; AC-d5 (régime de
   gates justifié ?).
4. **32-1c, phrases contre gestes.** Chaque AC-c1 à AC-c8 est-il **une phrase** (aucune commande changée, aucune
   consigne neuve d'agir — C-32-1-15) ? Le `literate` et les replis d'icônes changent-ils le **rendu** d'une commande de
   `sec:repetition-cloture` ? Le repli « 9 signes en ASCII dans les deux scripts » touche-t-il une commande d'une autre
   procédure que le lecteur exécute ?
5. **Manuel — `.tex` ET PDF.** Les lignes citées de `docs/manual/fr/admin-manual.tex` (1547, 1625, 1635, 1686, 1704-1711,
   1714-1726, 1772, 1788, 1791, 1844, 1851-1852, 1880-1918, 2345-2357, 2923, 3313-3320) et de `user-manual.tex` (3262)
   disent-elles ce que la fiche prétend ? Contrôle le **PDF aplati** avec la commande de la fiche (Dev Notes, § *Contrôle
   aplati d'un PDF*) : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | sed "s/’/'/g" | tr -s ' ' | grep -oF "<phrase>"`
   — témoin positif d'abord (« trop grande pour être restaurée par l'écran ») ; puis le **symptôme** :
   `pdftotext docs/manual/fr/<pdf> - | grep -o $'\xef\xbf\xbd' | wc -l` pour les trois PDF (26, 9, 16 annoncés). Les
   retouches prescrites (AC-b7, AC-c1 à AC-c5, AC-c7) seront-elles **vraies après** le correctif, et aucune phrase voisine
   ne deviendra-t-elle fausse ? Le manuel utilisateur, la brochure, le README, `website/` ou le CHANGELOG disent-ils
   quelque chose de contraire ?
6. **i18n.** Clés neuves et modifiée de l'AC-b8 et de l'AC-d3 contre les gardes (parité, G9, `lint-i18n-ownership`,
   `i18n-entrees-a-variables.test.ts`, `i18n-repli-divergent-actif.test.ts`) ; la valeur fr-CH neuve
   d'`admin-backup-auto-run-unknown` ; un test existant asserte-t-il un texte qui changera sans être nommé ?
7. **Inventaire des sites non résolus.** Est-il clos ? Recompte les deux décomptes de #710 (145 et 101) ; les copies de
   l'assistant lexical sur **tout** `crates/` (`grep -rn '"#\[cfg(test)\]"\|split("#\[cfg(test)\]")' crates --include=*.rs`,
   `grep -rn 'starts_with("//")' crates --include=*.rs`) — en manque-t-il une ? Un autre analyseur que `lexique` existe-t-il
   déjà (`grep -rln 'fn decouper\|fn masquer\|fn assainir\|strip_comments' crates`) ? D'autres signes sans glyphe servis à
   l'utilisateur ?

## Lentille F — faisabilité et fausses prémisses

1. **Prémisses** : chaque constat retenu est-il établi au code ou supposé ? Une prémisse corrigée à tort ? Une correction
   retenue comme mécanique cache-t-elle une conception — en particulier `formBase`, le classement « non-`ApiError` →
   issue inconnue », le réemploi de `lexique` (dépendance d'un fichier de `kesh-api/src` vers `kesh-db/tests`), les
   replis de glyphes (choix de présentation) ?
2. **Découpage** : modules annoncés (A : 3 ; B : 3 au grain des crates et paquets, 5 au grain métier ; C : 1 ; D : 2 et
   3) contre *Fichiers touchés* ; le découpage (C-32-1-37, qui amende C-32-1-36) est-il juste, la coupe tenable, et
   chaque clé développable et revue seule (voir *Attention — le découpage de la 32-1b*) ? Ordre de fusion (32-1c → 32-1b,
   32-1d → 32-1b), `closes #711` sur la dernière fusionnée des 32-1a, 32-1b, 32-1d (C-32-1-27 amendé), sous-sections du
   Dev Agent Record : tiennent-ils dans tous les ordres permis ? Deux clés touchent-elles les mêmes lignes (la 32-1a et
   la 32-1d partagent `auto_backup.rs` — tête et `mod tests` pour l'une, près d'`is_overdue` et dans le même `mod tests`
   pour l'autre —, `admin_auto_backup_e2e.rs`, `textes_coherents.rs` ; la 32-1b et la 32-1d les quatre catalogues) ?
3. **Faisabilité** : l'inclusion `#[path]` en `src` sous `#[cfg(test)]` (compile-t-elle avec `clippy --all-targets -D
   warnings` : lints de `kesh-api` contre un module à `#![allow(dead_code)]`) ; l'extension de `decouper` sans casser ses
   deux utilisateurs ; les Vitest de l'AC-b1 (iii) et de l'AC-b2 (non-`ApiError`, `loadError`) avec les mocks existants
   d'`AutoBackupSection.test.ts` ; `literate` avec des caractères multioctets sous XeLaTeX, et son repli écrit ;
   `make admin` (cible du `docs/manual/Makefile`) ; la commande de comparaison de la brochure.
4. **Décomptes** : recompte chaque nombre à sa source — `[Cas; 10]` → 12 ; `TABLE: [(&str, Cles); 4]` → 5 ; 26 `⚠️`
   (17 + 9, 16 + 9 lignes) ; U+FFFD 26 / 9 / 16 et leur ventilation (17 + 9 ; 9 ; 15 + 1) ; 2 `I18nBundle::load` ; neuf
   mutations en A (M-a7 en deux formes), neuf en B (M-b1 à M-b4, M-b7 à M-b11, M-b7 en deux formes), deux en D (M-d1,
   M-d2) ; **+2** (B) et **+1** (D) clés `admin-backup-` par locale ; « −3 + 3 » issues, « −2 + 1 »
   bugs (étiquettes réelles de #711, #714, #716, #725, #726, #727) ; 6 MEDIUM et 12 LOW distincts au Change Log de la P2
   (recompte depuis les deux rapports P2) ; signale celui qui ne se retrouve pas.
5. **Cohérence interne** : AC-a1 à AC-a13, AC-b1 à AC-b11 (AC-b6 renvoyé), AC-c1 à AC-c10, AC-d1 à AC-d6, tâches,
   `sprint-status.yaml` (quatre clés 32-1), registre (C-32-1-1 à 37 et
   amendements : une décision amendée contredit-elle encore un passage de la fiche ?), *Issues ouvertes*, *Commentaires à
   verser*, *Inventaire*, Change Log — contradiction, critère invérifiable, commande mal formée (essaie mentalement chaque
   `grep`, `sed`, `pdftotext`, `git show` contre le dépôt actuel) ? Aucune donnée réelle ?
6. **Ce que la fiche omet** : un test existant qui rougira sans être nommé ; une conséquence pour la recette visuelle
   (captures des AC-b9, AC-c6, AC-c7, AC-d4 — le cas concurrent de l'AC-b1 en mérite-t-il une ?) ; le `.keshbackup`
   (aucune table ne change : vérifie) ; une phrase du manuel que les correctifs de la 32-1b, de la 32-1c ou de la 32-1d
   rendraient fausse sans être nommée.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, partie de la fiche (A, B, C, D, Dev Notes, registre) et
ligne, site du dépôt, preuve (commande et sortie copiée), correction proposée ; pour chacun, dis s'il vient de la
**remédiation P2**, de la remédiation P1 ou de la rédaction d'origine. ⛔ **La liste des axes exercés ET non exercés**
(R1 à R7 et R3-bis, F1 à F6, et chaque point d'*Attention — le découpage de la 32-1b*), avec pour chacun ce qui a été lu — sans elle, « 0 finding » ne compte pas.

Rapport écrit **d'un seul jet**, un seul fichier par lentille, **par l'outil d'écriture de fichier** (jamais par une
redirection du shell) : `/home/gcorbaz/devel/kesh-gate-logs/321-validate-p3-R.md` pour la lentille R,
`/home/gcorbaz/devel/kesh-gate-logs/321-validate-p3-F.md` pour la lentille F. Dernier message : chemin, bilan par
sévérité (recompté sur ta liste), une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris **aucun fichier** hors de ton rapport ; **aucun fichier temporaire** ; **aucune redirection** du shell vers un
fichier (`>`, `>>`, `tee`, `-o`). Aucun `git checkout`/`switch` sous aucune forme. **Aucune commande qui écrit, compile ou
exécute** : `scripts/*` (dont **`scripts/prepare-release.sh`**, qui bumpe les versions du workspace — à **lire**
seulement), `make`, `latexmk`, `lualatex`, `xelatex`, `cargo`, `rustup`, `npm`, `npx`, `node`, `docker`, `curl`,
**`python3`, `python`, `perl`, `ruby`**, `mariadb`/`mysql`, `sqlx`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture (`pr`, `issue create|edit|close|comment`).
`pdftotext` vers la sortie standard seulement (`-` comme destination). `sed` sans `-i`, en filtre seulement. Pour lire la
table de glyphes d'une police, `fc-query` ou `otfinfo` en lecture seule (sinon, déclare l'axe non exercé). Aucune donnée
réelle de Guy (rien de `~/kesh-test/`). Autorisés : lecture de fichiers, `grep`, `sed -n` et `sed` en filtre de tube,
`awk` en lecture, `wc`, `tr`, `od`, `git log|show|diff|blame|merge-base`, `gh issue view`, `pdftotext … -`.
