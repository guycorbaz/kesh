# Prompt — validation P1, Story 32-8 (la validation avant envoi — la poignée)

*Versionné le 2026-10-25. Passe **complète** (première passe). **Deux lentilles, R et F**, chacune en contexte frais,
**Sonnet**. Lecture seule stricte, **sans `gh`**, **sans interpréteur**, **sans fichier temporaire**.*

Dépôt `/home/gcorbaz/devel/kesh-32-8`, branche `story/32-8` (partie de `main` `fa5b3468` : 32-5a, 32-5b et 32-5d
fusionnées, 32-5c en développement). Fiche : `_bmad-output/implementation-artifacts/32-8-validation-avant-envoi.md`
(trois parties : 32-8a, 32-8b, 32-8c) ; registre : `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-328.md`
(C-32-8-1 à 26) ; annotation datée du § 32-8 de `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` ; entrées
`32-8*` de `_bmad-output/implementation-artifacts/sprint-status.yaml`. Contexte : le fichier d'epic (§ *32-8*, § *Ordre
de dépendance*, § *Release v0.16.0*, § *Réponses de Guy*) ; la spike `_bmad-output/planning-artifacts/epic-32-spike-mcp.md`
(**en entier** : § 2.1 et § 2.4 (d), § 3, § 5.3.1 E1 à E6, § 5.3.3 M-1 et M-2, § 6 entrée 32-8, § 6.1, § 6.2 points 3,
4 et 6, § 7 et ses **réponses** Q10, Q11, Q12) ; la fiche 32-5 `_bmad-output/implementation-artifacts/32-5-cles-et-reglages-mcp.md`
(routes de la 32-5c, l. 495-530). **Deux fiches validées mais non fusionnées**, à lire **en lecture seule** dans
d'autres worktrees : `/home/gcorbaz/devel/kesh-32-6/_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md`
(AC-a1, AC-a15 à AC-a23, AC-a30, AC-a31, AC-b1 à AC-b4, § *La frontière avec la 32-8*, Dev Notes *Esquisse des
modules*) et son registre `epic-32-choix-autonomes-326.md` (C-32-6-14, 15, 22, 36, 37, 41) ;
`/home/gcorbaz/devel/kesh-32-10a/_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md` (AC-ii1, AC-ii3,
AC-ii8, AC-iii1 à AC-iii3) et son registre `epic-32-choix-autonomes-3210a.md` (C-32-10a-18, 23, 26, 29, 30). Règles :
`CLAUDE.md` (§ *Review Iteration Rule* et *La passe ciblée* — une boucle ne se clôt que sur une passe sans finding
au-dessus de LOW —, *Inventorier les sites NON RÉSOLUS*, *Règle de splitting préventif*, *Pendant une boucle de revue*
et son exception `kesh-db`, *Migration breaking policy* P1 à P8, *Le prompt d'une passe doit NOMMER le manuel*,
*Recompter ses propres comptes rendus*, *Recette visuelle*, *Priorités des défauts* — « opération enregistrée mais
annoncée en échec » est un P2), `docs/agents-orchestration.md`.

**Contraintes de la story, à tenir pour vraies et à contrôler** : **Q10 (c)** — la poignée : l'outil rend aussitôt
« envoi n° X en attente », l'utilisateur valide tel quel, modifie ou refuse dans un écran de Kesh à son rythme, l'agent
relit ; **une table** (story 🗄️) ; **Q12** — par clé, `required` par défaut, `exempt` désignée par l'administrateur,
chaque envoi journalisé ; **Q11** — lecture seule applicative ; **Q4** — Kesh n'est pas en service, installation neuve ;
la propriété centrale — **ce que l'utilisateur voit est exactement ce qui part, octet pour octet, modification
comprise** ; au plus cinq modules par partie ; aucune donnée réelle. Ce que la fiche **prescrit** au développement
n'existe pas encore : son absence n'est pas un défaut. Ce qu'elle cite **des fiches 32-6 et 32-10a** n'existe pas
encore au code : contrôle-le **contre ces fiches**, non contre le dépôt.

## Les deux axes obligatoires, pour les deux lentilles

- ⛔ **Motifs et mesures greppés sur le dépôt réel.** Dresse d'abord la liste des motifs et des mesures écrits dans la
  fiche (`grep -nF '\b' …/32-8-validation-avant-envoi.md`, puis `'grep -'`, `'→ **0**'`, `'→ **un**'`) : gardes de
  l'AC-B14 et de l'AC-C6, faits 6, 10, 13 et 14, mesures de l'analogue `mcp_keys`. Rejoue **chacun** dans le périmètre
  que la fiche lui donne et **copie la sortie** : une mesure que ton grep ne reproduit pas est un finding ; un motif qui
  prend une ligne légitime hors liste blanche aussi ; un **témoin vert existant** nommé doit exister — ouvre la ligne.
- ⛔ **Chaque affirmation sur une dépendance se prouve à sa source**, version du `Cargo.lock`
  (`~/.cargo/registry/src/*/`) : `serde_json-1.0.150/src/ser.rs` (l'échappement d'une chaîne, fait 11),
  `axum-core-0.5.6/src/ext_traits/request.rs` (la limite de corps, fait 12), `sqlx-core-0.8.6` si la fiche s'appuie sur
  un comportement de transaction ou de pool, `unicode-properties-0.1.4` pour les cas de l'AC-B3 qui relèvent de
  `is_hidden_format`. Une affirmation dont tu ne trouves pas la ligne est un finding. Une affirmation sur MariaDB 10.11
  (`CHECK`, `NOW(3)`, `FOR UPDATE`, collation `utf8mb4_unicode_ci`) que la fiche ne renvoie pas au test est un finding.

## Lentille R — la spécification contre le code et les fiches

Pour chaque affirmation, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**. Axes, tous à exercer :

1. **Les faits 1 à 14** (§ *Ce que le code dit*) : chaque ligne citée — `migrations/20261025000001_…sql:54`, `:65` ;
   `services/cles_mcp.rs:36`, `:55-64`, `:78-117` ; `services/poignees.rs` ; `services/acces.rs:84-123` ;
   `tests/services_motif.rs:1885-1913`, `:2557` ; `kesh-db/src/backup.rs:44-146`, `:832-1051` (les **quatre règles** du
   test de partition : la table `mcp_sends`, enfant de `companies`, `mcp_keys` et `users`, les tient-elle **sans
   modification de leurs listes**, comme l'AC-A10 l'affirme ?) ; `post_restore.rs:210-216` ;
   `tests/admin_full_export_e2e.rs:281-286` ; `tests/onboarding_audit_e2e.rs:1961-2042` ; `lib.rs:222-403` ;
   `middleware/rbac.rs` ; `tests/admin_pat_denied_e2e.rs:46-53` ; `errors.rs` (`IllegalState`, `OptimisticLockConflict`,
   `Validation`) ; `audit_labels.rs` ; `kesh-core/src/text.rs` ; `frontend/src/routes/(app)/+layout.svelte:147-170` ;
   `frontend/scripts/lint-i18n-ownership.js` ; `docs/manual/fr/admin-manual.tex:1414`, `:2230`, `:2237`, `:2249`.
2. **La frontière avec la 32-6** (fiche 32-6, § *La frontière avec la 32-8* et AC-a15, AC-a22, AC-a23, AC-a31,
   AC-b1) : la fiche 32-8 change-t-elle **exactement** ce que la frontière lui donne (`autoriser`, `appliquer`, leurs
   signatures, leurs deux sites d'appel) — et chaque autre changement du code de la 32-6 qu'elle fait (`tools/list`,
   `registre_coherent`, `instructions`, `ErreurOutil`, énumération des issues, tests amendés) est-il **nommé** comme
   amendement en connaissance de cause ? Les noms qu'elle emprunte à la 32-6 (`Donnees`, `BlocContenu`,
   `DonneesEnvoyables`, `en_resultat`, `EtatMcp`, `arguments::<T>()`, `journaliser_appel`, les gardes) existent-ils
   dans la fiche 32-6 sous ces noms ? Le contrat d'`e1_rien_ne_part_sous_cle_requise` amendé reste-t-il un E1 ?
3. **La 32-10a** : `ReponseMcp::Accuse(AccuseProposition)` n'est-il ni réemployé ni contredit ? Le patron
   `LecturePrompts`/`EcriturePrompts` (AC-ii3, AC-iii1) est-il repris fidèlement par `LectureEnvois`/`EcritureEnvois`
   (pool en lecture seule pour la lecture, garde v-2, `MOTIF_V2`) ? Les dépendances (`is_hidden_format`,
   `entree_appel`, `LimiteAtteinte`) sont-elles décrites telles que la 32-10a les crée ?
4. **La spike** : E1 (chaque chemin visé), E2 (la ligne « outil de relecture »), E4 (table, purge, relecture tel que
   validé, même clé même société, tests), mutations 4 et 6 d'E6, M-1, M-2, § 6.2 points 3 et 4 : chacun a-t-il son AC et
   son test ? Une décision de la fiche contredit-elle la spike sans le dire ?
5. **La migration et la sauvegarde** (AC-A1 à AC-A12) : P1 à P8 ; le schéma indicatif (contraintes `CHECK` en MariaDB
   10.11, collation, `MEDIUMTEXT`) ; `is_required` à l'import pour les colonnes sans `DEFAULT` (`backup.rs`, cherche
   `is_required`) ; l'aller-retour d'import ; la remise à zéro (`EMPTY_BEFORE_RESET`, la liste « doit être peuplée »).
6. **Le manuel et la documentation** (axe obligatoire de `CLAUDE.md`) : les quatre passages de l'AC-A12 et la section
   de l'AC-C10 contre le code prescrit ; et **le PDF aplati** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n'
   ' ' | tr -s ' ' | grep -oiE ".{60}(MCP|validation).{80}"`, idem `user-manual.pdf`. Un passage existant devient-il
   faux avec la 32-8 sans que la fiche le nomme (`README.md`, `website/*.html`, `docs/api-external.md` :
   `grep -n "MCP\|montr" README.md website/*.html docs/api-external.md`) ?

## Lentille F — faisabilité, découpage, sécurité et fausses prémisses

1. **La propriété centrale** (§ *La propriété centrale*) : cherche **un octet** qui pourrait différer entre la valeur
   montrée par l'écran et le second bloc reçu par le client — `textarea` (fins de ligne, espace de tête), sérialisation
   JSON aller et retour (frontend → `serde`, `serde_json` → client), collation et stockage MariaDB, `rendre_visible`
   (idempotence, ambiguïté d'une suite `\u{…}` déjà présente dans la donnée), « tel quel » sans renvoi du texte et sa
   version. Chaque maillon est-il tenu par ce que la fiche dit ?
2. **L'inventaire clos D1 à D23** : cherche un chemin **qui n'y est pas** par lequel un octet lu atteint un porteur de
   requête — une méthode JSON-RPC, une erreur, un en-tête, une durée, un journal, une route, l'export, l'import, la
   remise à zéro, une restauration, un changement de politique de la clé (PUT de la 32-5c) entre le dépôt et la
   relecture, une clé révoquée puis restaurée, un envoi d'une société désactivée. Chaque ligne est-elle **fermée par ce
   qu'elle dit** (relis le type ou la garde : un faux vert est-il possible) ?
3. **Le découpage (C-32-8-1)** : recompte les modules avec la convention écrite — dix distincts, 5 / 5 / 4 ? Les
   fichiers de chaque partie restent-ils dans ses modules (la 32-8b touche-t-elle `kesh-db` malgré l'AC-A4) ? L'état
   intermédiaire après la 32-8a, puis après la 32-8b, ouvre-t-il un accès ou laisse-t-il une clé `required` sans issue ?
4. **Concurrence, échéance, redémarrage, relecture double** : la décision sous `FOR UPDATE` et version, la limite
   souple, la purge hors transaction, l'horloge de la base — un interblocage, une double décision, un envoi validé après
   l'échéance, une relecture qui diffère ?
5. **Les types et les gardes** : `Autorisation`, `Sortie`, `EnvoiAValider`, `ContenuValide`, `AccuseEnvoi`,
   `LectureEnvois`, `EcritureEnvois` — la visibilité Rust tient-elle vraiment ce que la fiche affirme (un `pub(crate)`,
   un module descendant, un `impl` ailleurs) ? Les gardes de l'AC-B14 et de l'AC-C6 sont-elles conformes à la **règle
   commune** (propriété, périmètre, formes, témoin rouge, témoin vert **existant** ou factice dit tel) ?
6. **Tests et mutations** : chaque AC est-il vérifiable tel qu'écrit, avec son test nommé ? Chaque mutation MA1 à MA14,
   MB1 à MB27, MC1 à MC6 est-elle **réalisable** et tuée par le test **nommé** — une passerait-elle à vide (MA2 et les
   règles du test de partition ; MB2 ; MB17 et le calcul des octets d'U+200B ; MC5) ? Une garde nommée sans mutation ?
7. **Décomptes** : recompte à la source — 36 AC (12 / 14 / 10), 15 tâches (5 / 6 / 4), 47 mutations (14 / 27 / 6),
   C-32-8-1 à 26, D1 à D23 — et la cohérence des entrées `32-8*` de `sprint-status.yaml` avec la fiche. Le YAML se
   lit-il (indentation, `clé: valeur  # commentaire`) ?
8. **Ce que la fiche omet** : un report de la spike ou des fiches 32-5, 32-6, 32-10a non repris ; les « points à
   soumettre à Guy » qui devraient y être et n'y sont pas, et inversement ; ce que la 32-8 laisse à la 32-7, 32-9, 32-10a
   et 32-11 — juste, complet ?

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la dépendance,
preuve (commande et sortie copiée), correction proposée. Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste
des axes exercés ET non exercés** (R1 à R6 ou F1 à F8, et les deux axes obligatoires) — sans elle, « 0 finding » ne
compte pas comme passe. Rapport écrit **d'un seul jet**, un seul fichier par lentille :
`/home/gcorbaz/devel/kesh-gate-logs/328-validate-p1-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/328-validate-p1-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`** et
**`scripts/regen-test-schema.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, `wget`,
**`python3`, `python`, `perl`, `ruby`, `node`** et tout autre interpréteur, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, et **`gh` sous toutes ses formes** (même en lecture). `pdftotext`
vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal
`/home/gcorbaz/devel/kesh` est hors de ton périmètre ; les worktrees `kesh-32-6` et `kesh-32-10a` se lisent, ne
s'écrivent pas. Autorisés : lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `pdftotext … -`, et les
filtres en lecture seule `tr`, `sort`, `cut`, `head`, `tail`, `awk`, `wc`, `uniq` vers la sortie standard.
